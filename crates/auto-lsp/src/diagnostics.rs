//! PLAN-971 B/T-08: diagnostics come from the ACE producer.
//!
//! The legacy parser walk that used to live here is gone: every publish is a
//! **compile-only project check** through
//! `auto_lang_core::check_ace_project` (`AceEngineChoice::AceAV`), with no
//! fallback to the legacy parser. A document the ACE check cannot serve keeps
//! exactly the producer's answer (a diagnostic, or the named host/deny state
//! surfaced as one) — never a legacy parse as a backstop.
//!
//! Project shape (the packages lane the check consumes):
//! - `package`: a synthetic, stable package name derived from the workspace
//!   root directory name (all documents share it);
//! - `version`: `"0"`;
//! - `id`: the document path relative to the workspace root, extension
//!   stripped, **flattened into one ACE module id** (see
//!   [`MODULE_ID_SEPARATOR`]);
//! - `source`: the document text of the revision being checked.
//!
//! Publication is per document: a diagnostic's `module` (`pkg::mod`) maps back
//! to the document it was reported against, and its module-relative BYTE span
//! is converted with THAT document's text (UTF-16 character offsets, clamped
//! to char boundaries). Version discipline (V-08) is enforced by the caller via
//! [`GenerationSnapshot`]: a result computed from a document set/content that
//! is no longer current is discarded, never published.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use auto_lang_core::{
    check_ace_project, packages_capabilities, AceCheckOutcome, AceCheckRequest, AceDiagnostic,
    AceEngineChoice, AceModuleSpec, AceSpan,
};
use tower_lsp_server::ls_types::*;

/// Synthetic package used when the workspace root has no usable directory
/// name (or no root was announced at all).
pub const SYNTHETIC_PACKAGE_FALLBACK: &str = "ws";

/// The synthetic package version every document is checked under.
pub const SYNTHETIC_PACKAGE_VERSION: &str = "0";

/// Separator used to flatten a document path into the packages lane's
/// single-segment module id.
///
/// The producer admits module ids by `valid_id` = `[A-Za-z_][A-Za-z0-9_]*`
/// (see `ace-core` `source::packages::binding`), i.e. a module id is one
/// identifier: the packages lane has no nested ids. A document
/// `lib/util.at` therefore maps to `lib_util` (not `lib.util`, which the
/// producer rejects as `source.module-identity`). Dotted ids would need the
/// producer grammar to grow a nesting rule first; until then the LSP uses the
/// lane's own flattening convention (`-`/separator → `_`).
pub const MODULE_ID_SEPARATOR: char = '_';

/// One open document as the backend knows it, captured at one instant so the
/// text, the LSP version and the V-08 content generation always agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSnapshot {
    pub uri: String,
    pub version: i32,
    /// Backend content generation (bumped on every applied content update).
    pub generation: u64,
    pub content: String,
}

/// The exact document set/content a check result was computed from — the
/// V-08 staleness witness.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenerationSnapshot {
    /// `(uri, content generation)`, sorted by URI.
    pub documents: Vec<(String, u64)>,
}

impl GenerationSnapshot {
    /// Capture the witness of `docs` (any order).
    pub fn capture(docs: &[DocumentSnapshot]) -> Self {
        let mut documents: Vec<(String, u64)> = docs
            .iter()
            .map(|doc| (doc.uri.clone(), doc.generation))
            .collect();
        documents.sort();
        GenerationSnapshot { documents }
    }

    /// A result computed from `self` may only be published when the current
    /// document set/content still matches: same URIs, same generations.
    pub fn is_current(&self, current: &GenerationSnapshot) -> bool {
        self == current
    }
}

/// The diagnostics to publish for one document.
#[derive(Debug, Clone)]
pub struct PublishedDiagnostics {
    pub uri: String,
    /// The LSP version of the content that was actually checked.
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
}

/// The outcome of one project check, ready to publish.
#[derive(Debug, Clone)]
pub struct ProjectCheck {
    /// V-08 witness of the document set/content this result belongs to.
    pub snapshot: GenerationSnapshot,
    /// One entry per document in the snapshot (empty list = no diagnostics).
    pub publications: Vec<PublishedDiagnostics>,
    /// Engine identity reported by the producer (`"av"`).
    pub engine: &'static str,
    /// Executor-measured wall time of the check.
    pub wall_ms: f64,
    /// Short human evidence line (clean / N diagnostics / named failure).
    pub outcome: String,
}

/// Install the reference ACE host exactly once (idempotent). The host crate
/// is Windows-only; on other platforms the producer is unavailable and that
/// is a NAMED failure, not a silent clean bill.
pub fn ensure_host() -> Result<(), String> {
    static INSTALLED: OnceLock<Result<(), String>> = OnceLock::new();
    INSTALLED
        .get_or_init(|| {
            #[cfg(windows)]
            {
                auto_lang_ace_host::install()
            }
            #[cfg(not(windows))]
            {
                Err("the ACE producer host is Windows-only in this build; \
                     the LSP has no legacy diagnostics fallback"
                    .to_string())
            }
        })
        .clone()
}

/// Run one compile-only project check over `docs`, rooted at `entry_uri`.
///
/// `entry_uri` MUST be one of `docs` (the diagnostic binding root). The module
/// set is exactly the snapshot: the packages lane has no filesystem discovery
/// of its own here, and documents that are not open are not part of the check.
pub fn check_project(
    docs: &[DocumentSnapshot],
    root: Option<&Path>,
    entry_uri: &str,
) -> ProjectCheck {
    let snapshot = GenerationSnapshot::capture(docs);
    let Some(entry) = docs.iter().find(|doc| doc.uri == entry_uri) else {
        return ProjectCheck {
            snapshot,
            publications: Vec::new(),
            engine: AceEngineChoice::AceAV.as_str(),
            wall_ms: 0.0,
            outcome: "the diagnosed document is not part of the snapshot".to_string(),
        };
    };

    if let Err(message) = ensure_host() {
        return host_failure_result(docs, &snapshot, entry_uri, "ace.host/unavailable", &message);
    }

    let package = synthetic_package(root);
    let ids = module_ids(docs, root);

    // module key (`pkg::id`) → document URI, and the module specs the
    // producer consumes (ids/content borrow from the snapshot).
    let mut module_uri: BTreeMap<String, String> = BTreeMap::new();
    let mut specs: Vec<AceModuleSpec<'_>> = Vec::with_capacity(docs.len());
    for (doc, id) in docs.iter().zip(ids.iter()) {
        let key = module_key(&package, id);
        module_uri.insert(key, doc.uri.clone());
        specs.push(AceModuleSpec {
            package: &package,
            version: SYNTHETIC_PACKAGE_VERSION,
            id,
            source: &doc.content,
        });
    }
    let entry_index = docs
        .iter()
        .position(|doc| doc.uri == entry_uri)
        .expect("the binding root was located above");
    let entry_key = module_key(&package, &ids[entry_index]);

    let caps = packages_capabilities();
    let request = AceCheckRequest::new(&specs, &entry_key, &caps);
    let report = check_ace_project(AceEngineChoice::AceAV, &request);

    let mut per_uri: BTreeMap<String, Vec<Diagnostic>> = docs
        .iter()
        .map(|doc| (doc.uri.clone(), Vec::new()))
        .collect();

    let outcome = match &report.outcome {
        AceCheckOutcome::Clean => "clean".to_string(),
        AceCheckOutcome::Diagnostics(diags) => {
            for diagnostic in diags {
                // A diagnostic names the module it belongs to; an identity
                // outside the snapshot cannot be located and is reported
                // against the binding root (clamped) instead of being dropped.
                let uri = module_uri
                    .get(&diagnostic.module)
                    .cloned()
                    .unwrap_or_else(|| entry_uri.to_string());
                let text = docs
                    .iter()
                    .find(|doc| doc.uri == uri)
                    .map(|doc| doc.content.as_str())
                    .unwrap_or(entry.content.as_str());
                per_uri
                    .entry(uri)
                    .or_default()
                    .push(producer_diagnostic(diagnostic, text));
            }
            format!("{} producer diagnostic(s)", diags.len())
        }
        AceCheckOutcome::Cancelled => {
            push_failure(
                &mut per_uri,
                entry_uri,
                "ace.check/cancelled",
                "the ACE project check was cancelled",
            );
            "cancelled".to_string()
        }
        AceCheckOutcome::ResourceDenied { kind, message } => {
            push_failure(
                &mut per_uri,
                entry_uri,
                "ace.check/resource-denied",
                &format!("{message} ({kind})"),
            );
            format!("resource denied ({kind})")
        }
        AceCheckOutcome::Denied { code, message } => {
            push_failure(
                &mut per_uri,
                entry_uri,
                code,
                &format!("the ACE producer refused the check: {message}"),
            );
            format!("denied ({code})")
        }
        AceCheckOutcome::HostFailure { code, message } => {
            push_failure(
                &mut per_uri,
                entry_uri,
                code,
                &format!("the ACE producer failed: {message}"),
            );
            format!("host failure ({code})")
        }
    };

    let publications = docs
        .iter()
        .map(|doc| PublishedDiagnostics {
            uri: doc.uri.clone(),
            version: doc.version,
            diagnostics: per_uri.remove(&doc.uri).unwrap_or_default(),
        })
        .collect();

    ProjectCheck {
        snapshot,
        publications,
        engine: report.engine,
        wall_ms: report.wall_ms,
        outcome,
    }
}

/// A named-failure result (host unavailable, check timeout, ...): the failure
/// is published against the binding root, everything else stays empty.
pub fn host_failure_result(
    docs: &[DocumentSnapshot],
    snapshot: &GenerationSnapshot,
    entry_uri: &str,
    code: &str,
    message: &str,
) -> ProjectCheck {
    let mut per_uri: BTreeMap<String, Vec<Diagnostic>> = docs
        .iter()
        .map(|doc| (doc.uri.clone(), Vec::new()))
        .collect();
    push_failure(&mut per_uri, entry_uri, code, message);
    let publications = docs
        .iter()
        .map(|doc| PublishedDiagnostics {
            uri: doc.uri.clone(),
            version: doc.version,
            diagnostics: per_uri.remove(&doc.uri).unwrap_or_default(),
        })
        .collect();
    ProjectCheck {
        snapshot: snapshot.clone(),
        publications,
        engine: AceEngineChoice::AceAV.as_str(),
        wall_ms: 0.0,
        outcome: format!("host failure ({code})"),
    }
}

fn push_failure(
    per_uri: &mut BTreeMap<String, Vec<Diagnostic>>,
    uri: &str,
    code: &str,
    message: &str,
) {
    per_uri
        .entry(uri.to_string())
        .or_default()
        .push(Diagnostic {
            range: Range::default(),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.to_string())),
            code_description: None,
            source: Some("ace".to_string()),
            message: message.to_string(),
            related_information: None,
            tags: None,
            data: None,
        });
}

/// One producer diagnostic in LSP form. `text` is the owning document's own
/// source: the span is relative to it and is clamped there (never into another
/// document).
fn producer_diagnostic(diagnostic: &AceDiagnostic, text: &str) -> Diagnostic {
    Diagnostic {
        range: span_range(text, diagnostic.span),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(diagnostic.code.clone())),
        code_description: None,
        source: Some("ace".to_string()),
        message: diagnostic.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// The `pkg::id` module key of one document.
pub fn module_key(package: &str, id: &str) -> String {
    format!("{package}::{id}")
}

/// The synthetic package name for a workspace root: the root directory's name
/// sanitized into a package identifier (`[a-z][a-z0-9_-]*` per the packages
/// lane), falling back to [`SYNTHETIC_PACKAGE_FALLBACK`].
pub fn synthetic_package(root: Option<&Path>) -> String {
    let raw = root
        .and_then(|root| root.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut package: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if package.is_empty() {
        return SYNTHETIC_PACKAGE_FALLBACK.to_string();
    }
    // Package ids must start with a lowercase ASCII letter.
    if !package.starts_with(|ch: char| ch.is_ascii_lowercase()) {
        package.insert_str(0, "p_");
    }
    package
}

/// The module ids of `docs`, aligned with `docs` (same order, same length).
pub fn module_ids(docs: &[DocumentSnapshot], root: Option<&Path>) -> Vec<String> {
    docs.iter().map(|doc| module_id(&doc.uri, root)).collect()
}

/// The module id of one document: the path relative to the workspace root,
/// extension stripped, flattened with [`MODULE_ID_SEPARATOR`] into one
/// identifier. A document outside the root (or with no root announced) uses
/// its file stem.
pub fn module_id(uri: &str, root: Option<&Path>) -> String {
    let path = uri_to_path(uri);
    let relative = root
        .zip(path.as_deref())
        .and_then(|(root, path)| path.strip_prefix(root).ok());
    let id = match relative {
        Some(relative) => id_from_relative(relative),
        None => path
            .as_deref()
            .and_then(|path| path.file_stem())
            .map(|stem| sanitize_module_id(&stem.to_string_lossy()))
            .unwrap_or_default(),
    };
    if id.is_empty() {
        "module".to_string()
    } else {
        id
    }
}

fn id_from_relative(relative: &Path) -> String {
    let raw = relative.to_string_lossy().replace('\\', "/");
    let segments: Vec<&str> = raw
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();
    let mut parts: Vec<String> = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        let last = index + 1 == segments.len();
        let base = if last {
            Path::new(segment)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| (*segment).to_string())
        } else {
            (*segment).to_string()
        };
        let part = sanitize_module_id(&base);
        if !part.is_empty() {
            parts.push(part);
        }
    }
    let mut id = parts.join(&MODULE_ID_SEPARATOR.to_string());
    // ACE module ids are `[A-Za-z_][A-Za-z0-9_]*`.
    if let Some(first) = id.chars().next() {
        if !(first.is_ascii_alphabetic() || first == '_') {
            id.insert_str(0, "m_");
        }
    }
    id
}

fn sanitize_module_id(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                MODULE_ID_SEPARATOR
            }
        })
        .collect()
}

/// Convert a document URI into a filesystem path. Percent-encoded file URIs
/// are decoded; anything else falls back to the raw `file://` suffix (tests
/// and clients do hand the LSP non-drive paths).
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    if let Ok(url) = url::Url::parse(uri) {
        if url.scheme() == "file" {
            if let Ok(path) = url.to_file_path() {
                return Some(path);
            }
        }
    }
    uri.strip_prefix("file://").map(PathBuf::from)
}

/// Convert a producer span (BYTE offsets relative to `text`) into an LSP range.
/// LSP characters are UTF-16 code units; offsets are clamped to the text and
/// to char boundaries so a stale or malformed span can never panic or point
/// outside the document.
pub fn span_range(text: &str, span: Option<AceSpan>) -> Range {
    let (start, end) = match span {
        Some(span) => (span.start, span.end),
        None => (0, 0),
    };
    let start = clamp_to_char_boundary(text, start);
    let end = clamp_to_char_boundary(text, end.max(start as u64));
    Range {
        start: position_at(text, start),
        end: position_at(text, end),
    }
}

fn clamp_to_char_boundary(text: &str, offset: u64) -> usize {
    let mut offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// Byte offset → LSP position (line, UTF-16 character). `offset` must be on a
/// char boundary of `text`.
fn position_at(text: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;
    let mut current = 0usize;
    for ch in text.chars() {
        if current >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
        current += ch.len_utf8();
    }
    Position { line, character }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(uri: &str, generation: u64, content: &str) -> DocumentSnapshot {
        DocumentSnapshot {
            uri: uri.to_string(),
            version: generation as i32,
            generation,
            content: content.to_string(),
        }
    }

    #[test]
    fn synthetic_package_follows_the_root_directory_name() {
        assert_eq!(synthetic_package(Some(Path::new("D:/tmp/lsp971/ws"))), "ws");
        assert_eq!(synthetic_package(None), SYNTHETIC_PACKAGE_FALLBACK);
        // Non-identifier characters are sanitized, a leading digit is legalized.
        assert_eq!(synthetic_package(Some(Path::new("D:/tmp/my-ws"))), "my-ws");
        assert_eq!(
            synthetic_package(Some(Path::new("D:/tmp/971ws"))),
            "p_971ws"
        );
        // A root without a usable name falls back.
        assert_eq!(
            synthetic_package(Some(Path::new("D:/"))),
            SYNTHETIC_PACKAGE_FALLBACK
        );
    }

    #[test]
    fn module_ids_are_relative_flattened_paths() {
        let root = Path::new("D:/tmp/lsp971/ws");
        assert_eq!(
            module_id("file:///D:/tmp/lsp971/ws/main.at", Some(root)),
            "main"
        );
        assert_eq!(
            module_id("file:///D:/tmp/lsp971/ws/lib/util.at", Some(root)),
            "lib_util"
        );
        assert_eq!(
            module_id("file:///D:/tmp/lsp971/ws/lib/deep/one.at", Some(root)),
            "lib_deep_one"
        );
        // Outside the root (or no root): the file stem.
        assert_eq!(
            module_id("file:///D:/elsewhere/other.at", Some(root)),
            "other"
        );
        assert_eq!(module_id("file:///p971/proof/main.at", None), "main");
        // Weird names still produce a valid ACE identifier.
        assert_eq!(
            module_id("file:///D:/tmp/lsp971/ws/1-main.at", Some(root)),
            "m_1_main"
        );
        assert_eq!(module_key("ws", "lib_util"), "ws::lib_util");
    }

    #[test]
    fn spans_are_module_relative_utf16_ranges() {
        let ascii = "pub fn hello() int { return nosuch }";
        let range = span_range(ascii, Some(AceSpan { start: 28, end: 34 }));
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 28
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 0,
                character: 34
            }
        );

        // Multi-byte text before the span: bytes 31..37 are `nosuch`, which is
        // UTF-16 characters 8..14 on line 1 (` return nosuch }`).
        let multi = "fn main() int { // 汉\n return nosuch }\n";
        let range = span_range(multi, Some(AceSpan { start: 31, end: 37 }));
        assert_eq!(
            range.start,
            Position {
                line: 1,
                character: 8
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 1,
                character: 14
            }
        );

        // Surrogate pair (2 UTF-16 code units) before the span.
        let emoji = "fn main() int { // 😂\n return nosuch }\n";
        let start = emoji.find("nosuch").expect("token") as u64;
        let range = span_range(
            emoji,
            Some(AceSpan {
                start,
                end: start + 6,
            }),
        );
        assert_eq!(
            range.start,
            Position {
                line: 1,
                character: 8
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 1,
                character: 14
            }
        );

        // No span / out-of-range span: clamped to the text, never outside.
        assert_eq!(
            span_range(ascii, None),
            Range {
                start: Position {
                    line: 0,
                    character: 0
                },
                end: Position {
                    line: 0,
                    character: 0
                },
            }
        );
        let clamped = span_range(
            ascii,
            Some(AceSpan {
                start: 10_000,
                end: 20_000,
            }),
        );
        assert!(clamped.start.character <= ascii.len() as u32);
        assert_eq!(clamped.start, clamped.end, "a span past the end collapses");
        // A span that starts inside a multi-byte char is pulled back to a
        // boundary instead of panicking.
        let mid = span_range("汉x", Some(AceSpan { start: 1, end: 2 }));
        assert_eq!(
            mid.start,
            Position {
                line: 0,
                character: 0
            }
        );
    }

    #[test]
    fn generation_snapshot_is_the_v08_witness() {
        let docs = vec![
            doc("file:///ws/main.at", 1, "fn main() int { return 0 }"),
            doc("file:///ws/util.at", 2, "pub fn hello() int { return 1 }"),
        ];
        let snapshot = GenerationSnapshot::capture(&docs);
        assert!(snapshot.is_current(&GenerationSnapshot::capture(&docs)));
        assert_eq!(
            snapshot.documents,
            vec![
                ("file:///ws/main.at".to_string(), 1u64),
                ("file:///ws/util.at".to_string(), 2u64),
            ]
        );

        // Content changed (new generation) → stale.
        let mut changed = docs.clone();
        changed[1].generation = 3;
        changed[1].content.push(' ');
        assert!(!snapshot.is_current(&GenerationSnapshot::capture(&changed)));

        // Document closed → stale.
        let closed = GenerationSnapshot::capture(&docs[..1]);
        assert!(!snapshot.is_current(&closed));

        // Document opened → stale (the project the result describes changed).
        let mut opened = docs.clone();
        opened.push(doc(
            "file:///ws/more.at",
            3,
            "pub fn more() int { return 2 }",
        ));
        assert!(!snapshot.is_current(&GenerationSnapshot::capture(&opened)));
    }
}
