//! PLAN-971 B/T-08 + V-08: the permanent real-binary LSP fixture for the ACE
//! producer's multi-file diagnostics and versioned publication.
//!
//! It spawns the REAL `auto-lsp` binary (`CARGO_BIN_EXE_auto-lsp`) and speaks
//! LSP/JSON-RPC (Content-Length framing) over stdio — no in-process shortcut.
//!
//! Oracle:
//! - a temp workspace (`ws/util.at` library + `ws/main.at` importing it with
//!   the packages-lane dotted form `use ws.util.hello`) opens with NO
//!   diagnostics for either document;
//! - changing `util.at` to `return nosuch` publishes a diagnostic for the
//!   **util** URI only, whose range slices exactly the offending token inside
//!   util's own text (module-relative span → LSP range), with the producer's
//!   code (`source.undefined`) and source (`ace`);
//! - changing it back clears util's diagnostics and publishes no bogus
//!   diagnostics anywhere;
//! - breaking `main.at` publishes for **main** only;
//! - a rapid two-step change (v1 bad, v2 good, sent back-to-back) leaves the
//!   LAST published diagnostics for that URI empty, carrying the document
//!   version of the content actually checked (stale results are discarded,
//!   never published over the newer publish — V-08);
//! - `shutdown`/`exit` terminates the server with code 0.
//!
//! Platform: Windows x64 (the ACE producer host crate is Windows-only, the
//! same gate as the language fixture `p971_tool_face.rs`).

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::time::timeout;

/// Per-read deadline; a closed/garbled stream is a NAMED failure, never a hang.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Quiet window after a `didOpen` (immediate publish path).
const SETTLE_OPEN: Duration = Duration::from_millis(500);

/// Quiet window after a `didChange`: the backend keeps a 150ms debounce, so the
/// publish arrives well inside this window.
const SETTLE_CHANGE: Duration = Duration::from_millis(700);

// --- fixture sources --------------------------------------------------------
//
// The synthetic package is the workspace root directory name (`ws`), so the
// packages-lane import form is `use ws.util.hello`.
const UTIL_GOOD: &str = "pub fn hello() int { return 1 }\n";
const UTIL_BAD: &str = "pub fn hello() int { return nosuch }\n";
const MAIN_GOOD: &str = "use ws.util.hello\nfn main() int { return hello() }\n";
const MAIN_BAD: &str = "use ws.util.hello\nfn main() int { return nosuch_main }\n";
const MAIN_BAD_2: &str = "use ws.util.hello\nfn main() int { return also_bad() }\n";

/// A temp workspace whose root directory is literally `ws` (stable package id).
struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new() -> TempWorkspace {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let group =
            std::env::temp_dir().join(format!("auto-lsp-971-{}-{stamp}", std::process::id()));
        let root = group.join("ws");
        std::fs::create_dir_all(&root).expect("create the temp workspace root");
        TempWorkspace { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    /// Write one workspace source file (the documents are also opened with
    /// their text over LSP; the on-disk copy keeps the fixture realistic).
    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.root.join(name);
        std::fs::write(&path, text).expect("write the workspace source");
        path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.root.parent().unwrap_or(&self.root));
    }
}

fn file_uri(path: &Path) -> String {
    url::Url::from_file_path(path)
        .unwrap_or_else(|_| panic!("{path:?} is not expressible as a file URI"))
        .to_string()
}

// --- LSP client over the child process' stdio -------------------------------

struct LspCli {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    /// Bytes already read but not yet framed (a timed-out read must never lose
    /// a partially received frame).
    buffer: Vec<u8>,
    next_id: i64,
    /// Every server→client message received so far (publishes + log noise).
    seen: Vec<Value>,
}

impl LspCli {
    fn spawn(exe: &str) -> Result<LspCli, String> {
        let mut child = Command::new(exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn `{exe}` failed: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "server stdin not piped".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "server stdout not piped".to_string())?;
        Ok(LspCli {
            child,
            stdin,
            stdout,
            buffer: Vec::new(),
            next_id: 0,
            seen: Vec::new(),
        })
    }

    async fn send(&mut self, message: &Value) -> Result<(), String> {
        let body = serde_json::to_string(message).map_err(|e| e.to_string())?;
        self.stdin
            .write_all(format!("Content-Length: {}\r\n\r\n{body}", body.len()).as_bytes())
            .await
            .map_err(|e| format!("write to server failed: {e}"))
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await?;
        self.wait_for_response(id).await
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
            .await
    }

    /// Read one framed message, waiting at most `wait` for more bytes. `None`
    /// means "nothing arrived within the window"; partial frames stay buffered.
    async fn next_frame(&mut self, wait: Duration) -> Result<Option<Value>, String> {
        loop {
            if let Some(message) = take_message(&mut self.buffer)? {
                return Ok(Some(message));
            }
            let mut chunk = [0u8; 8192];
            match timeout(wait, self.stdout.read(&mut chunk)).await {
                Err(_) => return Ok(None),
                Ok(Err(e)) => return Err(format!("read from server failed: {e}")),
                Ok(Ok(0)) => return Err("server closed the connection".to_string()),
                Ok(Ok(n)) => self.buffer.extend_from_slice(&chunk[..n]),
            }
        }
    }

    async fn wait_for_response(&mut self, id: i64) -> Result<Value, String> {
        loop {
            let message = self
                .next_frame(READ_TIMEOUT)
                .await?
                .ok_or_else(|| format!("timed out waiting for the response to request {id}"))?;
            if message.get("id").and_then(Value::as_i64) == Some(id) {
                return Ok(message);
            }
            self.seen.push(message);
        }
    }

    /// Drain server messages until `quiet` passes with nothing new.
    async fn settle(&mut self, quiet: Duration) -> Result<(), String> {
        loop {
            match self.next_frame(quiet).await? {
                Some(message) => self.seen.push(message),
                None => return Ok(()),
            }
        }
    }

    /// Every `publishDiagnostics` for `uri` received from index `from` on.
    fn publishes_since(&self, uri: &str, from: usize) -> Vec<&Value> {
        self.seen[from..]
            .iter()
            .filter(|message| {
                message.get("method").and_then(Value::as_str)
                    == Some("textDocument/publishDiagnostics")
                    && message["params"]["uri"].as_str() == Some(uri)
            })
            .collect()
    }
}

impl Drop for LspCli {
    fn drop(&mut self) {
        // Best effort: never leave a spawned server behind, even on panic.
        let _ = self.child.start_kill();
    }
}

/// Take one complete frame out of `buffer`, if a whole frame is available.
fn take_message(buffer: &mut Vec<u8>) -> Result<Option<Value>, String> {
    let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Ok(None);
    };
    let header = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
    let mut length = None;
    for line in header.split("\r\n") {
        if let Some(value) = line.strip_prefix("Content-Length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let Some(length) = length else {
        return Err(format!("frame without Content-Length: {header:?}"));
    };
    let body_start = header_end + 4;
    if buffer.len() < body_start + length {
        return Ok(None);
    }
    let body = buffer[body_start..body_start + length].to_vec();
    buffer.drain(..body_start + length);
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| format!("malformed frame body: {e}"))
}

// --- publish / range helpers ------------------------------------------------

fn diagnostics_of(publish: &Value) -> &Vec<Value> {
    publish["params"]["diagnostics"]
        .as_array()
        .unwrap_or_else(|| panic!("publishDiagnostics without a diagnostics array: {publish}"))
}

fn version_of(publish: &Value) -> Option<i64> {
    publish["params"]["version"].as_i64()
}

fn line_start(text: &str, line: u64) -> usize {
    if line == 0 {
        return 0;
    }
    let mut current = 0u64;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            current += 1;
            if current == line {
                return index + 1;
            }
        }
    }
    text.len()
}

fn line_text(text: &str, line: u64) -> &str {
    let start = line_start(text, line);
    let rest = &text[start..];
    match rest.find('\n') {
        Some(end) => &rest[..end],
        None => rest,
    }
}

/// UTF-16 character offset (LSP) → byte offset within one line.
fn utf16_col_to_byte(line: &str, character: u64) -> usize {
    let mut utf16 = 0u64;
    for (index, ch) in line.char_indices() {
        if utf16 >= character {
            return index;
        }
        utf16 += ch.len_utf16() as u64;
    }
    line.len()
}

/// The exact text an LSP range covers in `text` (the assertion oracle for
/// "the diagnostic points at the offending token").
fn range_slice(text: &str, range: &Value) -> String {
    let start_line = range["start"]["line"].as_u64().unwrap_or(0);
    let start_char = range["start"]["character"].as_u64().unwrap_or(0);
    let end_line = range["end"]["line"].as_u64().unwrap_or(0);
    let end_char = range["end"]["character"].as_u64().unwrap_or(0);
    let start =
        line_start(text, start_line) + utf16_col_to_byte(line_text(text, start_line), start_char);
    let end = line_start(text, end_line) + utf16_col_to_byte(line_text(text, end_line), end_char);
    assert!(
        start <= end && end <= text.len(),
        "range {range} must stay inside the document (start {start}, end {end}, len {})",
        text.len()
    );
    text.get(start..end).unwrap_or_default().to_string()
}

fn open_params(uri: &str, version: i64, text: &str) -> Value {
    json!({
        "textDocument": {
            "uri": uri,
            "languageId": "auto",
            "version": version,
            "text": text,
        }
    })
}

fn change_params(uri: &str, version: i64, text: &str) -> Value {
    json!({
        "textDocument": { "uri": uri, "version": version },
        "contentChanges": [{ "text": text }],
    })
}

#[tokio::test]
async fn p971_lsp_ace_multifile_diagnostics_versioned_publish() {
    let started = Instant::now();
    let workspace = TempWorkspace::new();
    let util_path = workspace.write("util.at", UTIL_GOOD);
    let main_path = workspace.write("main.at", MAIN_GOOD);
    let util_uri = file_uri(&util_path);
    let main_uri = file_uri(&main_path);
    let root_uri = file_uri(workspace.root());

    let exe = env!("CARGO_BIN_EXE_auto-lsp");
    let mut cli = LspCli::spawn(exe).expect("spawn the real auto-lsp binary");

    // ---- initialize with the workspace root ------------------------------
    let init = cli
        .request(
            "initialize",
            json!({
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {},
            }),
        )
        .await
        .expect("initialize response");
    assert!(
        init["result"]["capabilities"].is_object(),
        "initialize must succeed: {init}"
    );
    cli.notify("initialized", json!({}))
        .await
        .expect("initialized");

    // ---- 1. open both documents: no diagnostics at all -------------------
    cli.notify("textDocument/didOpen", open_params(&util_uri, 1, UTIL_GOOD))
        .await
        .expect("didOpen util");
    cli.settle(SETTLE_OPEN)
        .await
        .expect("settle after opening util");
    cli.notify("textDocument/didOpen", open_params(&main_uri, 1, MAIN_GOOD))
        .await
        .expect("didOpen main");
    cli.settle(SETTLE_OPEN)
        .await
        .expect("settle after opening main");

    // The project is clean: every publish for either document is empty, and
    // the full-set check did publish both (a partial document set — main
    // opened before util — is a stale witness and must be discarded).
    let util_publishes = cli.publishes_since(&util_uri, 0);
    let main_publishes = cli.publishes_since(&main_uri, 0);
    assert!(
        !util_publishes.is_empty(),
        "the clean project must publish for util (got {} messages)",
        cli.seen.len()
    );
    assert!(
        !main_publishes.is_empty(),
        "the clean project must publish for main (got {} messages)",
        cli.seen.len()
    );
    for publish in util_publishes.iter().chain(main_publishes.iter()) {
        assert!(
            diagnostics_of(publish).is_empty(),
            "a clean project has no diagnostics: {publish}"
        );
        assert_eq!(
            version_of(publish),
            Some(1),
            "the open publish carries the version of the content actually checked: {publish}"
        );
    }

    // ---- 2. break util: diagnosed on the util URI, at the token ---------
    let mark = cli.seen.len();
    cli.notify(
        "textDocument/didChange",
        change_params(&util_uri, 2, UTIL_BAD),
    )
    .await
    .expect("didChange util bad");
    cli.settle(SETTLE_CHANGE)
        .await
        .expect("settle after breaking util");

    let publishes = cli.publishes_since(&util_uri, mark);
    let last = publishes.last().unwrap_or_else(|| {
        panic!(
            "util must be re-published after its change: {:?}",
            &cli.seen[mark..]
        )
    });
    let diagnostics = diagnostics_of(last);
    assert!(
        !diagnostics.is_empty(),
        "`return nosuch` must be diagnosed for util: {last}"
    );
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic["source"].as_str(),
        Some("ace"),
        "the diagnostic comes from the ACE producer: {diagnostic}"
    );
    assert_eq!(
        diagnostic["code"].as_str(),
        Some("source.undefined"),
        "the producer code is carried through: {diagnostic}"
    );
    assert_eq!(
        range_slice(UTIL_BAD, &diagnostic["range"]),
        "nosuch",
        "the range must point at the offending token inside util's own text: {diagnostic}"
    );
    assert_eq!(
        version_of(last),
        Some(2),
        "the publish carries the version of the content actually checked: {last}"
    );
    for publish in cli.publishes_since(&main_uri, mark) {
        assert!(
            diagnostics_of(publish).is_empty(),
            "a util error must not be published on main: {publish}"
        );
    }

    // ---- 3. fix util: util's diagnostics clear --------------------------
    let mark = cli.seen.len();
    cli.notify(
        "textDocument/didChange",
        change_params(&util_uri, 3, UTIL_GOOD),
    )
    .await
    .expect("didChange util good");
    cli.settle(SETTLE_CHANGE)
        .await
        .expect("settle after fixing util");

    let publishes = cli.publishes_since(&util_uri, mark);
    let last = publishes.last().unwrap_or_else(|| {
        panic!(
            "util must be re-published after the fix: {:?}",
            &cli.seen[mark..]
        )
    });
    assert!(
        diagnostics_of(last).is_empty(),
        "the fix must clear util's diagnostics: {last}"
    );
    assert_eq!(
        version_of(last),
        Some(3),
        "cleared for the fixed content: {last}"
    );

    // ---- 4. break main: diagnosed on the main URI only ------------------
    let mark = cli.seen.len();
    cli.notify(
        "textDocument/didChange",
        change_params(&main_uri, 2, MAIN_BAD),
    )
    .await
    .expect("didChange main bad");
    cli.settle(SETTLE_CHANGE)
        .await
        .expect("settle after breaking main");

    let publishes = cli.publishes_since(&main_uri, mark);
    let last = publishes.last().unwrap_or_else(|| {
        panic!(
            "main must be re-published after its change: {:?}",
            &cli.seen[mark..]
        )
    });
    let diagnostics = diagnostics_of(last);
    assert!(
        !diagnostics.is_empty(),
        "`return nosuch_main` must be diagnosed for main: {last}"
    );
    assert_eq!(
        range_slice(MAIN_BAD, &diagnostics[0]["range"]),
        "nosuch_main",
        "the range points into main's own text: {last}"
    );
    assert_eq!(version_of(last), Some(2), "main's checked version: {last}");
    for publish in cli.publishes_since(&util_uri, mark) {
        assert!(
            diagnostics_of(publish).is_empty(),
            "a main error must not be published on util: {publish}"
        );
    }

    // ---- 5. V-08: rapid two-step change, stale results discarded --------
    let mark = cli.seen.len();
    cli.notify(
        "textDocument/didChange",
        change_params(&main_uri, 3, MAIN_BAD_2),
    )
    .await
    .expect("rapid didChange v3 (bad)");
    cli.notify(
        "textDocument/didChange",
        change_params(&main_uri, 4, MAIN_GOOD),
    )
    .await
    .expect("rapid didChange v4 (good)");
    cli.settle(SETTLE_CHANGE)
        .await
        .expect("settle after the burst");

    let publishes = cli.publishes_since(&main_uri, mark);
    assert!(
        !publishes.is_empty(),
        "the burst must publish for main: {:?}",
        &cli.seen[mark..]
    );
    let sequence: Vec<(Option<i64>, usize)> = publishes
        .iter()
        .map(|publish| (version_of(publish), diagnostics_of(publish).len()))
        .collect();
    let last = publishes.last().expect("a last publish");
    eprintln!(
        "rapid two-step change published sequence (document version, diagnostic count): {sequence:?}"
    );
    assert!(
        diagnostics_of(last).is_empty(),
        "the LAST publish must be for the newer (fixed) content — a stale v3 result must never \
         overwrite it; published sequence (version, diagnostics): {sequence:?}"
    );
    assert_eq!(
        version_of(last),
        Some(4),
        "the publish carries the version of the content actually checked (sequence: {sequence:?})"
    );

    // ---- 6. clean shutdown: exit code 0 ---------------------------------
    cli.next_id += 1;
    let shutdown_id = cli.next_id;
    cli.send(&json!({"jsonrpc": "2.0", "id": shutdown_id, "method": "shutdown", "params": null}))
        .await
        .expect("send shutdown");
    let shutdown = cli
        .wait_for_response(shutdown_id)
        .await
        .expect("shutdown response");
    assert!(
        shutdown.get("result").is_some(),
        "shutdown result: {shutdown}"
    );
    cli.notify("exit", json!({})).await.expect("exit notify");
    let status = timeout(READ_TIMEOUT, cli.child.wait())
        .await
        .expect("the server exits promptly after exit")
        .expect("wait for the server process");
    assert_eq!(status.code(), Some(0), "clean exit code");

    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "the ACE LSP fixture must stay under the 10s case gate, took {elapsed:?}"
    );
    eprintln!(
        "p971_lsp_ace_multifile_diagnostics_versioned_publish: {:.0}ms, {} server messages",
        elapsed.as_secs_f64() * 1000.0,
        cli.seen.len()
    );
}

/// The flattened text of a hover payload (string / {"value"} / array forms).
fn hover_text(contents: &Value) -> String {
    match contents {
        Value::String(text) => text.clone(),
        Value::Object(map) => map
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        Value::Array(items) => items
            .iter()
            .map(hover_text)
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// The source file the editor's existing analyzer resolves, plus its caller.
/// The import form is the analyzer's package-root form (`use pac.util`) so
/// cross-file resolution reads the module from disk — the same behaviour every
/// existing Auto workspace relies on.
const UTIL_LIB: &str = "fn hello() int { return 1 }\n";
const MAIN_CALLER: &str = "use util\nfn main() int { let v = hello()\n return v }\n";

/// PLAN-971 B/T-08：真实二进制下核验 LSP 既存的语义能力面（本包 T-08 的
/// “核其现存能力”义务）：跨文件/跨包 goto 落到**原模块源文件**的正确行、
/// hover 给出符号信息、workspace/symbol 找到该符号、semanticTokens 非空、
/// completion 有响应。这些能力沿用既有分析实现（本包不重写），本用例把
/// “能工作且指向正确源”变成可复跑的真实证据。
#[tokio::test]
async fn p971_lsp_existing_semantic_capabilities_verified() {
    let started = Instant::now();
    let workspace = TempWorkspace::new();
    let util_path = workspace.write("util.at", UTIL_LIB);
    let main_path = workspace.write("main.at", MAIN_CALLER);
    let util_uri = file_uri(&util_path);
    let main_uri = file_uri(&main_path);
    let root_uri = file_uri(workspace.root());

    let mut cli = LspCli::spawn(env!("CARGO_BIN_EXE_auto-lsp")).expect("spawn auto-lsp");
    let init = cli
        .request(
            "initialize",
            json!({ "processId": null, "rootUri": root_uri, "capabilities": {} }),
        )
        .await
        .expect("initialize");
    assert!(init["result"]["capabilities"].is_object(), "{init}");
    cli.notify("initialized", json!({})).await.expect("initialized");

    cli.notify("textDocument/didOpen", open_params(&util_uri, 1, UTIL_LIB))
        .await
        .expect("didOpen util");
    cli.settle(SETTLE_OPEN).await.expect("settle util");
    cli.notify("textDocument/didOpen", open_params(&main_uri, 1, MAIN_CALLER))
        .await
        .expect("didOpen main");
    cli.settle(SETTLE_OPEN).await.expect("settle main");

    // ---- 1. cross-file/package goto lands in the module's own source -----
    // `hello` starts at line 1 column 24 in MAIN_CALLER; aim inside the word.
    let def = cli
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 1, "character": 26 },
            }),
        )
        .await
        .expect("definition response");
    let result = &def["result"];
    assert!(!result.is_null(), "goto must resolve the imported symbol: {def}");
    let location = result
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or_else(|| result.clone());
    assert_eq!(
        location["uri"].as_str(),
        Some(util_uri.as_str()),
        "goto must land in the imported module's ORIGINAL source file: {location}"
    );
    assert_eq!(
        location["range"]["start"]["line"].as_u64(),
        Some(0),
        "the definition is the `hello` declaration (line 0 of util): {location}"
    );
    assert_eq!(
        location["range"]["start"]["character"].as_u64(),
        Some(3),
        "the definition position points at the `hello` name itself (col 3): {location}"
    );

    // ---- 2. hover describes the symbol -----------------------------------
    let hover = cli
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 1, "character": 26 },
            }),
        )
        .await
        .expect("hover response");
    let text = hover_text(&hover["result"]["contents"]);
    assert!(
        text.contains("hello") || text.contains("int"),
        "hover must describe the symbol, got {text:?} ({hover})"
    );

    // ---- 3. workspace/symbol finds it ------------------------------------
    let symbols = cli
        .request("workspace/symbol", json!({ "query": "hello" }))
        .await
        .expect("workspace symbol response");
    let list = symbols["result"]
        .as_array()
        .unwrap_or_else(|| panic!("workspace/symbol result must be an array: {symbols}"));
    assert!(
        list.iter().any(|s| s["name"].as_str() == Some("hello")),
        "workspace/symbol must list `hello`: {symbols}"
    );

    // ---- 4. semantic tokens are produced ---------------------------------
    let tokens = cli
        .request(
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": main_uri } }),
        )
        .await
        .expect("semantic tokens response");
    let data = tokens["result"]["data"]
        .as_array()
        .unwrap_or_else(|| panic!("semantic tokens data must be an array: {tokens}"));
    assert!(!data.is_empty(), "semantic tokens must not be empty: {tokens}");

    // ---- 5. completion answers at the call site --------------------------
    let completion = cli
        .request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 1, "character": 24 },
            }),
        )
        .await
        .expect("completion response");
    assert!(
        !completion["result"].is_null(),
        "completion must answer at the call site: {completion}"
    );

    // ---- 6. clean shutdown ----------------------------------------------
    cli.next_id += 1;
    let shutdown_id = cli.next_id;
    cli.send(&json!({"jsonrpc": "2.0", "id": shutdown_id, "method": "shutdown", "params": null}))
        .await
        .expect("send shutdown");
    let _ = cli.wait_for_response(shutdown_id).await.expect("shutdown");
    cli.notify("exit", json!({})).await.expect("exit");
    let status = timeout(READ_TIMEOUT, cli.child.wait())
        .await
        .expect("the server exits promptly")
        .expect("wait");
    assert_eq!(status.code(), Some(0));

    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "the semantic-capability fixture must stay under the 10s case gate, took {elapsed:?}"
    );
    eprintln!(
        "p971_lsp_existing_semantic_capabilities_verified: {:.0}ms",
        elapsed.as_secs_f64() * 1000.0
    );
}


