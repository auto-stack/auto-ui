//! PLAN-105 T-02（U03）：**组件面源准备**——把 AutoUI 工程源整理成
//! `project_component_candidates` 可吃的单源形。
//!
//! 组件面（AGE component profile）是**单源**投影：不认 `use` 行，也没有 import
//! 上下文。真实工程源（016 的 `use stylekit.styles: caption_text`、013/015/018/
//! 022 的 `use x_store: X`、全组的 `use themes.themes`）因此整例回滚。本模块做
//! 两件确定性的事，供 AV/SFC 车道在投影前调用：
//!
//! 1. `use <dep>.<module>: item, …` —— 若 `<dep>` 是本工程 `pac.at` 声明的依赖，
//!    从依赖源里取 `pub style <item> = "…"` 字面量，**内联**为本工程的
//!    `style` 声明（样式引用仍能被投影解析）；其余 `use`（store / back.api /
//!    datetime / themes 等）只做等长空白化。
//! 2. `use` 行一律**等长空白化**（保 span；投影期无 `use` 面）。
//!
//! 边界（诚实）：只内联单行 `style NAME = "literal"`；参数化 style 与多行值
//! 留待需要时扩面（具名不猜）。`use` 行空白化后行号/字节偏移不变。
use std::path::{Path, PathBuf};

/// 一条内联进来的样式字面量（`name` + 原字面量文本，含引号）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlinedStyle {
    pub name: String,
    pub literal: String,
}

/// 源准备的产物：整理后的源 + 内联记录（证据用）。
#[derive(Debug, Clone)]
pub struct PreparedComponentSource {
    pub source: String,
    pub inlined: Vec<InlinedStyle>,
}

/// 解析工程 `pac.at` 的 `dep <name> { path: "<rel>" }` 声明（行扫描，够用即可）。
pub fn pac_dep_roots(project_root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(pac) = std::fs::read_to_string(project_root.join("pac.at")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    for line in pac.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("dep ") {
            let name = rest
                .split(|c: char| c.is_whitespace() || c == '{')
                .next()
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
            pending = Some(name);
        }
        if let (Some(name), Some(pos)) = (&pending, t.find("path:")) {
            let after = &t[pos + 5..];
            let rel = after
                .trim_start()
                .trim_start_matches('"')
                .split('"')
                .next()
                .unwrap_or("")
                .trim();
            if !rel.is_empty() {
                let root = project_root.join(rel);
                out.push((name.clone(), normalize(root)));
            }
            pending = None;
        }
        if t == "}" {
            pending = None;
        }
    }
    out
}

fn normalize(p: PathBuf) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// 在依赖根下定位模块源：`<root>/<module>.at`、`<root>/src/front/<module>.at`。
fn module_file(dep_root: &Path, module: &str) -> Option<PathBuf> {
    let rel = module.replace('.', "/");
    for cand in [
        dep_root.join(format!("{rel}.at")),
        dep_root.join("src").join("front").join(format!("{rel}.at")),
    ] {
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// 从源文本里抽单行 `[[pub] style] <name> = "<literal>"` 声明。
fn style_literal(src: &str, name: &str) -> Option<String> {
    for line in src.lines() {
        let t = line.trim();
        let rest = t
            .strip_prefix("pub style ")
            .or_else(|| t.strip_prefix("style "))
            .or_else(|| t.strip_prefix("pub style\t"))
            .or_else(|| t.strip_prefix("style\t"))?;
        let Some((head, tail)) = rest.split_once('=') else {
            continue;
        };
        if head.trim() != name {
            continue;
        }
        let v = tail.trim();
        if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
            return Some(v.to_string());
        }
    }
    None
}

/// 准备组件面源：内联依赖样式 + 空白化 `use` 行。
pub fn prepare_component_source(
    project_root: &Path,
    src: &str,
) -> Result<PreparedComponentSource, String> {
    let deps = pac_dep_roots(project_root);
    let mut inlined: Vec<InlinedStyle> = Vec::new();
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let (body, nl) = match line.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (line, ""),
        };
        let t = body.trim_start();
        let Some(rest) = t.strip_prefix("use ") else {
            out.push_str(line);
            let _ = nl;
            continue;
        };
        // `use <module>: a, b` —— 取 module 与成员表。
        let (module, items) = match rest.split_once(':') {
            Some((m, i)) => (m.trim(), i.trim()),
            None => (rest.trim(), ""),
        };
        let mut handled = false;
        if let Some((dep_name, dep_root)) = deps
            .iter()
            .find(|(n, _)| module == n || module.starts_with(&format!("{n}.")))
        {
            let sub = if module.len() > dep_name.len() {
                &module[dep_name.len() + 1..]
            } else {
                ""
            };
            if let Some(file) = module_file(dep_root, sub) {
                if let Ok(dep_src) = std::fs::read_to_string(&file) {
                    for item in items.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                        if let Some(lit) = style_literal(&dep_src, item) {
                            inlined.push(InlinedStyle {
                                name: item.to_string(),
                                literal: lit,
                            });
                        }
                    }
                    handled = true;
                }
            }
        }
        // `use` 行等长空白化（保 span）。
        out.push_str(&" ".repeat(body.len()));
        out.push_str(nl);
        let _ = handled;
    }
    for style in &inlined {
        out.push_str(&format!("style {} = {}\n", style.name, style.literal));
    }
    Ok(PreparedComponentSource {
        source: out,
        inlined,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "p105-prep-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("deps/stylekit/src/front")).unwrap();
        std::fs::create_dir_all(root.join("src/front")).unwrap();
        std::fs::write(
            root.join("pac.at"),
            "name: \"app\"\nversion: \"1.0.0\"\ndep stylekit {\n    path: \"deps/stylekit\"\n}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("deps/stylekit/src/front/styles.at"),
            "pub style caption_text = \"text-xs text-muted-foreground\"\n\
             pub style pill(bg: str = \"x\") = \"{bg}\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/front/app.at"),
            "use stylekit.styles: caption_text\nuse themes.themes: Auto\nwidget App {\n    view { text \"x\" { style: caption_text } }\n}\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn inlines_dependency_style_and_blanks_use_lines() {
        let root = temp_project();
        let src = std::fs::read_to_string(root.join("src/front/app.at")).unwrap();
        let prepared = prepare_component_source(&root, &src).expect("prepare");
        assert_eq!(
            prepared.inlined,
            vec![InlinedStyle {
                name: "caption_text".into(),
                literal: "\"text-xs text-muted-foreground\"".into(),
            }]
        );
        assert!(prepared.source.contains("style caption_text = \"text-xs text-muted-foreground\""));
        // use 行等长空白化：`use` 文本消失，且原文件的行在前 N 行逐行等长（保 span）。
        let src_lines: Vec<&str> = src.lines().collect();
        let out_lines: Vec<&str> = prepared.source.lines().collect();
        for (before, after) in src_lines.iter().zip(out_lines.iter()) {
            if before.trim_start().starts_with("use ") {
                assert_eq!(before.len(), after.len(), "use line keeps its byte length");
                assert!(after.trim().is_empty(), "use line is blanked");
            } else {
                assert_eq!(*before, *after);
            }
        }
        assert!(!prepared.source.contains("use stylekit"));
        assert!(!prepared.source.contains("use themes"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn unknown_dep_style_is_not_inlined() {
        let root = temp_project();
        let prepared = prepare_component_source(
            &root,
            "use stylekit.styles: nope\nwidget App { view { text \"x\" {} } }\n",
        )
        .expect("prepare");
        assert!(prepared.inlined.is_empty());
        assert!(!prepared.source.contains("style nope"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
