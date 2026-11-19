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
        let Some(rest) = t
            .strip_prefix("pub style ")
            .or_else(|| t.strip_prefix("style "))
        else {
            continue;
        };
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
/// PLAN-105 T-02（U03 真语料 012）：把**类型化列表数据字段**的初值改写为无类型
/// 形——`var cities []str = ["…"]` → `var cities <blank> = ["…"]`（类型段等长
/// 空白化，保 span）。AGE 组件/包面把无类型列表字面量当**只读视图数据**
/// （1071 边界）；类型化列表 state 尚不在切片内，写面由投影后的具名拒绝兜底。
///
/// 只改「类型含 `[]` 且初值以 `[` 开头」的字段；其余行原样返回。
fn rewrite_typed_list_field(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    if !line[i..].starts_with("var ") {
        return None;
    }
    i += 4;
    // 名字段
    let name_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    if i == name_start {
        return None;
    }
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    // 类型段（到空白）
    let ty_start = i;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let ty = &line[ty_start..i];
    if !ty.contains("[]") {
        return None;
    }
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'=' {
        return None;
    }
    i += 1;
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'[' {
        return None;
    }
    let mut out = String::with_capacity(line.len());
    out.push_str(&line[..ty_start]);
    out.push_str(&" ".repeat(ty.len()));
    out.push_str(&line[ty_start + ty.len()..]);
    Some(out)
}

/// 准备组件面源：内联依赖样式 + 空白化 `use` 行 + 类型化列表数据字段改写。
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
            // 非 use 行：按需改写类型化列表数据字段（`var x []T = [...]`）。
            if let Some(rewritten) = rewrite_typed_list_field(body) {
                out.push_str(&rewritten);
                out.push_str(nl);
            } else {
                out.push_str(line);
            }
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
    // 内联样式必须落在**组件头之前**：模块样式表按源码顺序收集，组件之后声明的
    // 样式在投影期解析不到（016 实测：追加到文件尾 → `style caption_text` 不解析；
    // 前置到 widget 之前 → 整例通过）。
    if !inlined.is_empty() {
        let decls: Vec<String> = inlined
            .iter()
            .map(|s| format!("style {} = {}", s.name, s.literal))
            .collect();
        let mut lines: Vec<String> = out.split_inclusive('\n').map(|s| s.to_string()).collect();
        let head = lines.iter().position(|l| {
            let t = l.trim_start();
            t.starts_with("widget ") || t.starts_with("node ") || t.starts_with("component ")
        });
        match head {
            Some(idx) => {
                let mut block = decls.join("\n");
                block.push('\n');
                lines.insert(idx, block);
                out = lines.concat();
            }
            None => {
                for d in &decls {
                    out.push_str(d);
                    out.push('\n');
                }
            }
        }
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
        // 内联样式落在**组件头之前**（模块样式表按源码序收集；016 实测追加到
        // 文件尾解析不到）。
        let style_pos = prepared
            .source
            .find("style caption_text = \"text-xs text-muted-foreground\"")
            .expect("inlined style present");
        let widget_pos = prepared.source.find("widget App").expect("component head present");
        assert!(style_pos < widget_pos, "inlined styles land before the component head");
        // use 行等长空白化：`use` 文本消失，原行被等长空白替换（保 span）。
        assert!(!prepared.source.contains("use stylekit"));
        assert!(!prepared.source.contains("use themes"));
        assert!(prepared
            .source
            .contains(&" ".repeat("use stylekit.styles: caption_text".len())));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn unknown_dep_style_is_not_inlined() {        let root = temp_project();
        let prepared = prepare_component_source(
            &root,
            "use stylekit.styles: nope\nwidget App { view { text \"x\" {} } }\n",
        )
        .expect("prepare");
        assert!(prepared.inlined.is_empty());
        assert!(!prepared.source.contains("style nope"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn rewrites_typed_list_data_fields() {
        // 012 形：类型化列表数据字段 → 无类型视图数据（类型段等长空白化）。
        let rewritten = rewrite_typed_list_field("        var cities []str = [\"Beijing\"]")
            .expect("list data field rewritten");
        assert_eq!(rewritten.len(), "        var cities []str = [\"Beijing\"]".len());
        assert!(!rewritten.contains("[]str"));
        assert!(rewritten.trim_end().ends_with("= [\"Beijing\"]"));
        // 非列表类型、非列表初值、普通语句一律不动。
        assert!(rewrite_typed_list_field("    var n int = 0").is_none());
        assert!(rewrite_typed_list_field("    var xs []str = \"a\"").is_none());
        assert!(rewrite_typed_list_field("    text \"x\" {}").is_none());
        // 嵌套列表类型与空列表初值。
        let nested = rewrite_typed_list_field("    var xs [][]int = []").unwrap();
        assert!(!nested.contains("[][]int"));
    }

    #[test]
    fn real_u03_apps_project_after_prep() {
        // 真语料端到端：U03 六个应用经源准备后必须能被组件面投影（012 的列表
        // 索引在组件面已开；包面语义另刀）。路径相对本 crate。
        let ui_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui");
        let mut failed = Vec::new();
        // 012-clock 的 `on` 块仍有未接语义（列表索引求值等，见 plan-105 缺口表），
        // 不在本测断言内；其余五例经源准备后组件面必须整体投影通过。
        for app in [
            "013-todo",
            "015-notes",
            "016-calendar",
            "018-book-reader",
            "022-kanban",
        ] {
            let app_root = ui_root.join(app);
            let src = std::fs::read_to_string(app_root.join("src/front/app.at")).expect("app source");
            // 工程根（pac.at 所在）才是依赖解析基准；源文件在 src/front 下。
            let prepared = prepare_component_source(&app_root, &src).expect("prepare");
            if let Err(ds) = ace_core::source::age_component::project_component_candidates(&prepared.source) {
                let d = ds.first().map(|d| format!("{}: {}", d.code, d.message)).unwrap_or_default();
                failed.push(format!("{app} ({d})"));
            }
        }
        assert!(failed.is_empty(), "still failing: {failed:?}");
    }

}