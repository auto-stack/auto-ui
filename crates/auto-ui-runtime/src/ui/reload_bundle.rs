//! PLAN-056: 候选 bundle 来源快照与内容级失效检测。
//!
//! 快照 = root + transitive `use` 模块文件的规范路径 + mtime + blake3 内容
//! hash。mtime 只作快路径；内容 hash 才是身份（同 mtime 改内容也能检出）。
//! 缺源/不可读 = Err（旧 bundle 继续服役），恢复后下一次合法编辑成功。
//! 与构建管线同一模块解析规则（模块目录 → src/ 父级 → 语言仓根回退；
//! c/rust/py/auto.*/.web/use.py 不参与文件快照）。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use auto_ui_compiler::ui_ast::{resolve_use_module, UseModuleResolution};

/// One source file in the bundle: canonical path + fast mtime + content hash.
#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub path: PathBuf,
    pub mtime: Option<SystemTime>,
    pub hash: [u8; 32],
}

/// The whole candidate-bundle source graph snapshot (root + imports).
#[derive(Debug, Clone, Default)]
pub struct SourceSnapshot {
    /// Canonical root source path (the graph walk starts here).
    pub root: PathBuf,
    pub entries: Vec<SourceEntry>,
}

/// Hard bound on the walked module graph (cycle/large-project guard).
const MAX_SNAPSHOT_FILES: usize = 128;

/// Scan `code` (root or a module) for filesystem `use` statements and collect
/// every transitive module path, mirroring `dynamic_build`'s resolution:
/// base dir first, then `src/` parent, then the language-root fallback.
/// Cycle-guarded by canonical path; bounded by [`MAX_SNAPSHOT_FILES`].
pub fn collect_source_snapshot(
    root_path: &Path,
    root_code: &str,
) -> Result<SourceSnapshot, String> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut entries: Vec<SourceEntry> = Vec::new();
    collect_one(root_path, root_code, &mut visited, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    let root = entries
        .iter()
        .find(|e| e.path == std::fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf()))
        .map(|e| e.path.clone())
        .unwrap_or_else(|| root_path.to_path_buf());
    Ok(SourceSnapshot { root, entries })
}

fn collect_one(
    path: &Path,
    code: &str,
    visited: &mut HashSet<PathBuf>,
    entries: &mut Vec<SourceEntry>,
) -> Result<(), String> {
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canon.clone()) {
        return Ok(());
    }
    if visited.len() > MAX_SNAPSHOT_FILES {
        return Err(format!(
            "source graph exceeds {MAX_SNAPSHOT_FILES} files (cycle or oversized project)"
        ));
    }
    entries.push(read_entry(&canon, path)?);

    // Recurse into filesystem `use` targets of this module.
    let use_stmts = auto_lang_core::use_scanner::scan_use_statements(code);
    let base_dir = path.parent().unwrap_or(Path::new("."));
    let mut base_dirs: Vec<PathBuf> = vec![base_dir.to_path_buf()];
    if let Some(src_dir) = base_dir.parent() {
        if src_dir.file_name().map_or(false, |n| n == "src") {
            base_dirs.push(src_dir.to_path_buf());
        }
    }
    let lang_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|p| p.to_path_buf());
    if let Some(root) = &lang_root {
        let fb = root.join(base_dir);
        if !base_dirs.contains(&fb) {
            base_dirs.push(fb);
        }
    }

    for use_stmt in &use_stmts {
        if use_stmt.is_c_import || use_stmt.is_rust_import || use_stmt.is_python_import {
            continue;
        }
        if use_stmt.module.starts_with("auto.") || use_stmt.module.starts_with(".web") {
            continue;
        }
        let mut resolved = UseModuleResolution::None;
        for base in &base_dirs {
            resolved = resolve_use_module(base, use_stmt);
            if !matches!(resolved, UseModuleResolution::None) {
                break;
            }
        }
        match resolved {
            UseModuleResolution::Module(p) => {
                let Ok(code) = read_module(&p) else {
                    return Err(format!("missing imported module: {}", p.display()));
                };
                collect_one(&p, &code, visited, entries)?;
            }
            UseModuleResolution::StoreFiles(found) => {
                for (_, p) in &found {
                    let Ok(code) = read_module(p) else {
                        return Err(format!("missing store module: {}", p.display()));
                    };
                    collect_one(p, &code, visited, entries)?;
                }
            }
            UseModuleResolution::None => {
                // Unresolvable at build time = not part of the graph; a module
                // that resolved at build but is missing now surfaces as a read
                // error in `read_entry` instead.
            }
        }
    }
    Ok(())
}

fn read_module(path: &Path) -> Result<String, String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read imported module {}: {e}", path.display()))?;
    Ok(auto_lang_core::back_prefix::apply(path, code))
}

fn read_entry(canon: &Path, display: &Path) -> Result<SourceEntry, String> {
    let bytes = std::fs::read(canon).map_err(|e| {
        format!(
            "cannot read source {} (old bundle keeps serving): {e}",
            display.display()
        )
    })?;
    let mtime = std::fs::metadata(canon).ok().and_then(|m| m.modified().ok());
    Ok(SourceEntry {
        path: canon.to_path_buf(),
        mtime,
        hash: *blake3::hash(&bytes).as_bytes(),
    })
}

/// Fast-path guard: `true` when every entry's current mtime matches the
/// snapshot's recorded mtime (content unchanged by construction). Kept as a
/// diagnostic/future fast path; the authoritative check is
/// [`snapshot_changed`] (content hashes — same-mtime edits are detectable).
pub fn mtimes_unchanged(snapshot: &SourceSnapshot) -> bool {
    for entry in &snapshot.entries {
        let now = std::fs::metadata(&entry.path).ok().and_then(|m| m.modified().ok());
        if now != entry.mtime {
            return false;
        }
    }
    true
}

/// Compare two snapshots by content hash + entry set (not mtime).
/// `true` = the source graph changed (any file edited / module added/removed).
pub fn snapshot_differs(a: &SourceSnapshot, b: &SourceSnapshot) -> bool {
    if a.entries.len() != b.entries.len() {
        return true;
    }
    for ea in &a.entries {
        match b.entries.iter().find(|e| e.path == ea.path) {
            Some(eb) if eb.hash != ea.hash => return true,
            Some(_) => {}
            None => return true,
        }
    }
    false
}

/// Re-read the current graph and compare against `snapshot`. Returns
/// `Ok(true)` when any entry's content hash differs (a reload is needed);
/// `Err` when a file is missing/unreadable (old bundle keeps serving and the
/// caller keeps the previous snapshot so a later legal edit still detects).
pub fn snapshot_changed(snapshot: &SourceSnapshot) -> Result<bool, String> {
    if snapshot.root.as_os_str().is_empty() {
        return Ok(false);
    }
    let code = read_module(&snapshot.root)?;
    let current = collect_source_snapshot(&snapshot.root, &code)?;
    Ok(snapshot_differs(snapshot, &current))
}
