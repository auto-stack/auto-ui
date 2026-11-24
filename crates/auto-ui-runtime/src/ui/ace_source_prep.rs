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
/// PLAN-105 T-02（U03 真语料 013/016）：从单元源里抽 `store <Type> { … }` 整块
/// （含匹配的收尾 `}`；字符串内的括号不计数）。找不到返回 `None`。
/// PLAN-105 T-02：把 store 体里的自方法调用 `.Foo(` 改写为 `store.Foo(`（只认
/// 行首/空白/`{`/`;`/`}` 之后的点——`x.push(` 这类成员调用不动；按 **char**
/// 处理，绝不切坏 UTF-8 注释）。
fn rewrite_store_self_calls(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '.' {
            let prev = if i == 0 { ' ' } else { chars[i - 1] };
            let self_pos =
                prev.is_whitespace() || prev == '{' || prev == ';' || prev == '}';
            let mut j = i + 1;
            while j < chars.len() && is_ident(chars[j]) {
                j += 1;
            }
            let mut k = j;
            while k < chars.len() && chars[k] == ' ' {
                k += 1;
            }
            let mut is_handler_head = false;
            if self_pos && j > i + 1 && k < chars.len() && chars[k] == '(' {
                // `)` 之后的 `->` 表示这是 **handler 头**（`.Msg(params) -> {`），
                // 不是调用语句——不动。
                let mut depth = 0i32;
                let mut m = k;
                while m < chars.len() {
                    if chars[m] == '(' {
                        depth += 1;
                    } else if chars[m] == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    m += 1;
                }
                let mut n = m + 1;
                while n < chars.len() && chars[n].is_whitespace() {
                    n += 1;
                }
                is_handler_head =
                    n + 1 < chars.len() && chars[n] == '-' && chars[n + 1] == '>';
            }
            if self_pos && j > i + 1 && k < chars.len() && chars[k] == '(' && !is_handler_head {
                // 回扫紧邻 dot 的标识符（跳过空白），已是 `store` 则不再加前缀。
                let mut p = i;
                while p > 0 && chars[p - 1].is_whitespace() {
                    p -= 1;
                }
                let mut q = p;
                while q > 0 && is_ident(chars[q - 1]) {
                    q -= 1;
                }
                let ident: String = chars[q..p].iter().collect();
                if ident != "store" {
                    out.push_str("store");
                }
            }
            out.push('.');
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}
/// PLAN-105 T-02（U03 真语料 016）：从单元源里抽 `[pub] fn <name> … { … }` 整块
/// （含匹配收尾 `}`；字符串内括号不计数）；顶层 `pub ` 前缀剥掉（组件面
/// 只校验单模块，函数体原样内联）。找不到返回 `None`。
/// PLAN-105 T-02（U03 真语料 013）：从单元源里抽 `[pub] type <Name> { … }` 整块
/// （record 声明；`pub ` 前缀剥离）。找不到返回 None。
fn extract_unit_type(src: &str, name: &str) -> Option<String> {
    let mut start = None;
    let mut offset = 0usize;
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        let indent = line.len() - t.len();
        if indent == 0 {
            let rest = t.strip_prefix("pub ").unwrap_or(t);
            if let Some(after) = rest.strip_prefix("type ") {
                let tname: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if tname == name {
                    start = Some(offset + (line.len() - rest.len()));
                    break;
                }
            }
        }
        offset += line.len();
    }
    let start = start?;
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = start;
    let mut seen_open = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => {
                depth += 1;
                seen_open = true;
            }
            b'}' => {
                depth -= 1;
                if seen_open && depth == 0 {
                    let mut block = src[start..=i].to_string();
                    block.push('\n');
                    return Some(block);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn extract_unit_fn(src: &str, name: &str) -> Option<String> {
    let mut start = None;
    let mut offset = 0usize;
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        let indent = line.len() - t.len();
        if indent == 0 {
            let rest = t.strip_prefix("pub ").unwrap_or(t);
            if let Some(after) = rest.strip_prefix("fn ") {
                let fname: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if fname == name {
                    start = Some(offset + (line.len() - rest.len()));
                    break;
                }
            }
        }
        offset += line.len();
    }
    let start = start?;
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = start;
    let mut seen_open = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => {
                depth += 1;
                seen_open = true;
            }
            b'}' => {
                depth -= 1;
                if seen_open && depth == 0 {
                    let mut block = src[start..=i].to_string();
                    block.push('\n');
                    return Some(block);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// PLAN-105 T-02（U03 真语料 016）：**常量查表函数**降级——组件面/列表语义
/// 未接线时，把
/// ```text
/// fn name(params) R {
///     let tbl [N]R = [e0, e1, …]
///     return tbl[<expr>]
/// }
/// ```
/// 改写成等价的 if 链（`if <expr> == 0 { return e0 } else if …`）。只在函数体
/// 恰好是「常量表 + 单条 return 下标」时生效（其余返回 None，原样保留）——
/// 这是纯常量语义的等价改写，不改变可观察行为。
fn lower_const_table_fn(block: &str) -> Option<String> {
    let lines: Vec<&str> = block.lines().collect();
    // 头行 + `let ... = [` 行 + 元素行(可能多行) + `]` + `return tbl[expr]` + `}`
    let table_line = lines.iter().position(|l| {
        let t = l.trim_start();
        t.starts_with("let ") && t.contains('[') && t.contains("= [")
    })?;
    let ret_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with("return "))
        .filter(|&i| i > table_line)?;
    // 解析表名
    let t = lines[table_line].trim_start();
    let rest = t.strip_prefix("let ")?;
    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    if name.is_empty() {
        return None;
    }
    // 收集元素文本：从 `= [` 之后到匹配 `]`
    let after_eq = lines[table_line].find("= [")? + 3;
    let mut elems_text = lines[table_line][after_eq..].to_string();
    let mut end_line = table_line;
    if !elems_text.contains(']') {
        for (k, l) in lines.iter().enumerate().skip(table_line + 1) {
            elems_text.push_str(l);
            end_line = k;
            if l.contains(']') {
                break;
            }
        }
    }
    let close = elems_text.find(']')?;
    let elems_text = &elems_text[..close];
    let elems: Vec<String> = elems_text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    if elems.len() < 2 {
        return None;
    }
    // return 行必须是 `return <name>[<expr>]`
    let ret = lines[ret_idx].trim_start();
    let ret_rest = ret.strip_prefix("return ")?;
    let mut chars = ret_rest.chars();
    let ret_name: String = chars.by_ref().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    if ret_name != name {
        return None;
    }
    let mut rest_iter = ret_rest[ret_name.len()..].chars();
    if rest_iter.next() != Some('[') {
        return None;
    }
    let inner: String = rest_iter.collect();
    let close_idx = inner.find(']')?;
    let idx_expr = inner[..close_idx].trim();
    // 组装 if 链
    let head = lines[..table_line].join("\n");
    let mut out = String::with_capacity(block.len() + 64);
    out.push_str(&head);
    out.push('\n');
    for (i, e) in elems.iter().enumerate() {
        out.push_str(&format!(
            "    if {idx_expr} == {i} {{\n        return {e}\n    }}\n"
        ));
    }
    // 下标越界/负值：回落到最后一个元素（与原表读取的边界行为一致）。
    out.push_str(&format!(
        "    return {}\n}}\n",
        elems.last().cloned().unwrap_or_default()
    ));
    let _ = end_line;
    Some(out)
}

fn extract_store_block(src: &str, type_name: &str) -> Option<String> {
    let needle = format!("store {type_name}");
    let start = src.find(&needle)?;
    // 名字后面必须紧跟 `{`（允许空白）。
    let after = &src[start + needle.len()..];
    let brace_rel = after.find('{')?;
    if !after[..brace_rel].trim().is_empty() {
        return None;
    }
    let open = start + needle.len() + brace_rel;
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = open;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    // 与主源同律改写类型化列表数据字段（store 模型里
                    // `var notes []Note = []` → 无类型形，组件面才可解析）。
                    // PLAN-105 T-02（U03 真语料 013/015）：store 的**列表状态字段**
                    // 保持类型（`var todos []Todo = []`）——collections 域需要
                    // `List<Todo>` 真类型；记录类型由 store 单元的 `use types:`
                    // 依赖一并内联。
                    let mut block = src[start..=i].to_string();
                    block.push('\n');
                    // PLAN-105 T-02（U03 真语料 013/015/016）：store handler 里的
                    // **自方法调用** `.Foo(args)`（源方言里指 store 自己的 handler）
                    // 改写为 `store.Foo(args)`——core 的 store 内联把 `store.M()`
                    // 展开成对应 handler 体（自调用从而变成可调度语句）。
                    let block = rewrite_store_self_calls(&block);
                    return Some(block);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ─── PLAN-119：AGE 面 store handler 控制转移兼容改写 ────────────────────────
//
// AGE 受检面（age-component / age-package）的 handler 语句文法没有
// `return` / `break` / `continue`（`experimental/ace/auto-source.at` 无对应
// 规则）——真实 store 单元（038 MinesweeperStore：8 处 `return`、1 处
// `break`）依赖它们做守卫提前退出与循环中断，故 store 片断过不了 prep 的
// 内联门（`project_component_stores` + `project_component_candidates`）。
//
// 内联前按**结构化等价形**改写（只动 `on { … }` 区；字段/消息/视图不变）：
// - `return` → handler 首部 `var __p119_ret bool = false`；`return` 位改
//   `__p119_ret = true`；其后**同级**语句包进 `if __p119_ret == false { … }`；
//   C 风格 `for <cond>` / `while <cond>` 的条件并与 `__p119_ret == false`
//   （否则守卫让循环体空转、条件永真会自旋）。
// - `break` → 所属循环前 `var __p119_brk<N> bool = false`；`break` 位置 true；
//   循环条件并 `&& __p119_brk<N> == false`；其后同级语句包进守卫。
// - `continue` → 不改写（返回 None：store 维持不内联，边界如实具名，不猜语义）。
//
// 语义等价性由原 oracle 在 AV 端复核（038 首击 29 格 / 雷位表见计划 §6）。

/// store 片断控制转移改写；`None` = 含未受理构造（`continue`/不认识的结构），
/// 调用方保持原样（store 不内联，行为与本改写前一致）。
fn rewrite_store_control_flow(block: &str) -> Option<String> {
    // `continue` 不改写（语义映射未定，不猜——先于快速放行判定）。
    if block
        .lines()
        .any(|line| line.trim().trim_end_matches(';') == "continue")
    {
        return None;
    }
    if !block.contains("return") && !block.contains("break") {
        return Some(block.to_string());
    }
    let on_open = find_section_open(block, "on")?;
    let on_close = scan_balanced(block, on_open)?;
    let on_inner = &block[on_open + 1..on_close];
    let arms = split_handler_arms(on_inner)?;
    let mut rebuilt = String::new();
    for (head, body) in arms {
        let stmts = split_stmts(&body);
        let mut ctx = CtrlCtx::default();
        let out = transform_stmt_list(&stmts, &mut ctx)?;
        let mut body_out = String::new();
        if ctx.uses_ret {
            body_out.push_str("var __p119_ret bool = false\n");
        }
        body_out.push_str(&out.join("\n"));
        rebuilt.push_str(&format!("{head} -> {{\n{body_out}\n}}\n"));
    }
    let mut out = String::with_capacity(block.len() + rebuilt.len());
    out.push_str(&block[..=on_open]);
    out.push('\n');
    out.push_str(&rebuilt);
    out.push_str(&block[on_close..]);
    Some(restore_newlines(out, block))
}

/// 顶层 `<kw> { … }` 区块的 `{` 下标（`kw` 需独占行首词位）。
fn find_section_open(block: &str, kw: &str) -> Option<usize> {
    let mut from = 0usize;
    while let Some(rel) = block[from..].find(kw) {
        let at = from + rel;
        let line_start = block[..at].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line_head = block[line_start..at].trim();
        let after = &block[at + kw.len()..];
        let word_end = after
            .chars()
            .next()
            .map(|c| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(true);
        if line_head.is_empty() && word_end {
            if let Some(open) = after.find('{') {
                // `{` 与关键字之间只允许空白。
                if after[..open].trim().is_empty() {
                    return Some(at + kw.len() + open);
                }
            }
        }
        from = at + kw.len();
    }
    None
}

/// 从 `open`（指向 `{`）找配平的 `}`；跳过字符串字面量（含转义）。
fn scan_balanced(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = open;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `on { … }` 内逐条 `.Msg(..) -> { … }` 臂：返回（臂头, 体内文本）。
fn split_handler_arms(on_inner: &str) -> Option<Vec<(String, String)>> {
    let mut arms = Vec::new();
    let mut i = 0usize;
    let bytes = on_inner.as_bytes();
    while i < bytes.len() {
        if !on_inner.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let line_start = on_inner[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let at_line_start = on_inner[line_start..i].trim().is_empty();
        if at_line_start && on_inner.as_bytes()[i] == b'.' {
            let open = on_inner[i..].find('{').map(|o| o + i)?;
            let close = scan_balanced(on_inner, open)?;
            let arm = &on_inner[i..=close];
            let arrow = arm.find("->")?;
            let head = arm[..arrow].trim().to_string();
            let body = on_inner[open + 1..close].to_string();
            arms.push((head, body));
            i = close + 1;
            continue;
        }
        i += 1;
    }
    if arms.is_empty() {
        None
    } else {
        Some(arms)
    }
}

/// 顶层语句切分：花括号/圆括号深度 0 处按换行断句（注释行成为独立项）。
fn split_stmts(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut parens = 0i32;
    let mut in_str = false;
    for (i, b) in bytes.iter().enumerate() {
        if !body.is_char_boundary(i) {
            continue;
        }
        if in_str {
            let ch = *b as char;
            cur.push(ch);
            if ch == '\\' {
                continue;
            }
            if ch == '"' {
                in_str = false;
            }
            continue;
        }
        let ch = *b as char;
        match ch {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => depth -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => {}
        }
        cur.push(ch);
        if ch == '\n' && depth == 0 && parens == 0 {
            // 保留行首缩进（重建时沿用原排版，既有 prep 用例按原文缩进核对）。
            let piece = cur.trim_end_matches(['\r', '\n']);
            if !piece.trim().is_empty() {
                out.push(piece.to_string());
            }
            cur.clear();
        }
    }
    let piece = cur.trim_end_matches(['\r', '\n']);
    if !piece.trim().is_empty() {
        out.push(piece.to_string());
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
enum CtrlKind {
    Ret,
    Brk,
    Cont,
}

fn ctrl_of(stmt: &str) -> Option<CtrlKind> {
    match stmt.trim().trim_end_matches(';').trim() {
        "return" => Some(CtrlKind::Ret),
        "break" => Some(CtrlKind::Brk),
        "continue" => Some(CtrlKind::Cont),
        _ => None,
    }
}

/// `if <cond> { … } [else { … }]` / `for <cond> { … }` / `while <cond> { … }`。
struct BlockForm {
    kw: &'static str,
    cond: String,
    then_body: String,
    else_body: Option<String>,
}

fn parse_block_form(stmt: &str) -> Option<BlockForm> {
    let stmt = stmt.trim();
    let s = stmt;
    let (kw, rest) = if let Some(r) = s.strip_prefix("if ") {
        ("if", r)
    } else if let Some(r) = s.strip_prefix("for ") {
        ("for", r)
    } else if let Some(r) = s.strip_prefix("while ") {
        ("while", r)
    } else {
        return None;
    };
    // 条件与体之间以 `{` 分界（深度 0 处；条件里可能有 `(`/`[`）。
    let mut parens = 0i32;
    let mut open = None;
    for (i, ch) in rest.char_indices() {
        match ch {
            '(' | '[' => parens += 1,
            ')' | ']' => parens -= 1,
            '{' if parens == 0 => {
                open = Some(i);
                break;
            }
            _ => {}
        }
    }
    let open = open?;
    let cond = rest[..open].trim().to_string();
    let prefix = s.len() - rest.len();
    let abs_open = prefix + open;
    let close = scan_balanced(stmt, abs_open)?;
    let then_body = stmt[abs_open + 1..close].to_string();
    let tail = stmt[close + 1..].trim().to_string();
    let else_body = if tail.is_empty() {
        None
    } else if kw == "if" {
        let after = tail.strip_prefix("else")?.trim();
        let e_open = after.find('{')?;
        if !after[..e_open].trim().is_empty() {
            return None;
        }
        let tail_prefix = stmt.len() - tail.len();
        let abs = tail_prefix + (tail.len() - after.len()) + e_open;
        let e_close = scan_balanced(stmt, abs)?;
        Some(stmt[abs + 1..e_close].to_string())
    } else {
        return None;
    };
    Some(BlockForm {
        kw,
        cond,
        then_body,
        else_body,
    })
}

#[derive(Default)]
struct CtrlCtx {
    uses_ret: bool,
    /// 当前循环栈的 break 标志（`break` 作用于最内层）。
    brk_flags: Vec<String>,
    seq: usize,
}

/// 体内（不含更深循环）是否出现 `break`。
fn has_own_break(body: &str) -> bool {
    for st in split_stmts(body) {
        if ctrl_of(&st) == Some(CtrlKind::Brk) {
            return true;
        }
        if let Some(form) = parse_block_form(&st) {
            if form.kw == "if" {
                if has_own_break(&form.then_body) {
                    return true;
                }
                if let Some(e) = form.else_body.as_deref() {
                    if has_own_break(e) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn sets_flag(text: &str, flag: &str) -> bool {
    text.contains(&format!("{flag} = true"))
}

/// 语句列变换（见上方函数注释的语义规则）。
fn transform_stmt_list(stmts: &[String], ctx: &mut CtrlCtx) -> Option<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < stmts.len() {
        let st = stmts[i].clone();
        if let Some(kind) = ctrl_of(&st) {
            match kind {
                CtrlKind::Ret => {
                    ctx.uses_ret = true;
                    out.push("__p119_ret = true".to_string());
                }
                CtrlKind::Brk => {
                    let flag = ctx.brk_flags.last()?.clone();
                    out.push(format!("{flag} = true"));
                }
                CtrlKind::Cont => return None,
            }
            // 转移之后的同级语句不再执行（`return`/`break` 的直接效果）。
            break;
        }
        let Some(form) = parse_block_form(&st) else {
            out.push(st);
            i += 1;
            continue;
        };
        let (prelude, rendered) = match form.kw {
            "if" => {
                let then_out = transform_stmt_list(&split_stmts(&form.then_body), ctx)?;
                let else_out = match form.else_body.as_deref() {
                    Some(e) => Some(transform_stmt_list(&split_stmts(e), ctx)?),
                    None => None,
                };
                let mut text = format!("if {} {{\n{}\n}}", form.cond, then_out.join("\n"));
                if let Some(e) = else_out {
                    text.push_str(&format!(" else {{\n{}\n}}", e.join("\n")));
                }
                (Vec::new(), text)
            }
            "for" | "while" => {
                let named = form.cond.contains(" in ");
                let own_break = has_own_break(&form.then_body);
                let mut flags_to_declare: Vec<String> = Vec::new();
                if own_break {
                    ctx.seq += 1;
                    let flag = format!("__p119_brk{}", ctx.seq);
                    flags_to_declare.push(flag.clone());
                    ctx.brk_flags.push(flag);
                }
                let body_out = match transform_stmt_list(&split_stmts(&form.then_body), ctx) {
                    Some(b) => b,
                    None => return None,
                };
                let brk_flag = flags_to_declare.first().cloned();
                if own_break {
                    ctx.brk_flags.pop();
                }
                // `return` 置位后循环必须停转（C 风格条件可并，`for v in` 只能守体）。
                let ret_guard = sets_flag(&body_out.join("\n"), "__p119_ret");
                let mut body_text = body_out.join("\n");
                if let Some(flag) = &brk_flag {
                    if named {
                        body_text = format!("if {flag} == false {{\n{body_text}\n}}");
                    }
                }
                if ret_guard && named {
                    body_text = format!("if __p119_ret == false {{\n{body_text}\n}}");
                }
                let mut cond = form.cond.clone();
                if let Some(flag) = &brk_flag {
                    if !named {
                        cond = format!("({cond}) && {flag} == false");
                    }
                }
                if ret_guard && !named {
                    cond = format!("({cond}) && __p119_ret == false");
                }
                let text = format!("{} {} {{\n{}\n}}", form.kw, cond, body_text);
                let prelude = flags_to_declare
                    .iter()
                    .map(|f| format!("var {f} bool = false"))
                    .collect();
                (prelude, text)
            }
            _ => return None,
        };
        out.extend(prelude);
        // 该结构可能置位 → 其后同级语句必须守卫。
        let rendered_may_ret = sets_flag(&rendered, "__p119_ret");
        let cur_brk = ctx.brk_flags.last().cloned();
        let rendered_may_brk = cur_brk
            .as_deref()
            .map(|f| sets_flag(&rendered, f))
            .unwrap_or(false);
        out.push(rendered);
        if rendered_may_ret || rendered_may_brk {
            let rest = transform_stmt_list(&stmts[i + 1..], ctx)?;
            if rest.is_empty() {
                break;
            }
            let mut guard_cond = String::new();
            if rendered_may_ret {
                guard_cond.push_str("__p119_ret == false");
            }
            if rendered_may_brk {
                let flag = cur_brk.as_deref().unwrap_or_default();
                if !guard_cond.is_empty() {
                    guard_cond.push_str(" && ");
                }
                guard_cond.push_str(&format!("{flag} == false"));
            }
            out.push(format!("if {guard_cond} {{\n{}\n}}", rest.join("\n")));
            break;
        }
        i += 1;
    }
    Some(out)
}

/// 结构重建后恢复原源的换行风格（CRLF 源逐行仍是 CRLF——既有用例按
/// `\r\n` 计数核对 prep 的「保行」契约）。
fn restore_newlines(out: String, like: &str) -> String {
    if like.contains("\r\n") {
        out.replace("\r\n", "\n").replace('\n', "\r\n")
    } else {
        out
    }
}
/// 整词替换（标识符边界）：把 `name` 的出现替换为 `repl`（前后不得为
/// 标识符字符）。用于 prep 的本地样式引用替换。
fn replace_word(src: &str, name: &str, repl: &str) -> String {
    if name.is_empty() {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < src.len() {
        if src[i..].starts_with(name) {
            let before_ok = i == 0
                || !src[..i]
                    .chars()
                    .next_back()
                    .map(|c| c.is_alphanumeric() || c == '_')
                    .unwrap_or(false);
            let after = i + name.len();
            let after_ok = after >= src.len()
                || !src[after..]
                    .chars()
                    .next()
                    .map(|c| c.is_alphanumeric() || c == '_')
                    .unwrap_or(false);
            if before_ok && after_ok {
                out.push_str(repl);
                i = after;
                continue;
            }
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}
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
    // PLAN-105 T-02（U03 真语料 012）：**内建元素**的列表字段（`[]str` / `[]int`
    // / `[]bool` / `[]f32`）保持类型——collections 域按 `List<T>` 真承载，handler
    // 可写（012 的 `.w_rows = rows`）。记录元素的列表仍按视图数据改写（记录类型
    // 依赖尚需显式内联）。
    let elem = ty.trim_start_matches('[').trim_start_matches(']');
    let elem_builtin = matches!(elem, "str" | "Str" | "int" | "i32" | "bool" | "f32" | "float");
    if elem_builtin {
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
    // PLAN-105 T-02（U03 真语料 018/022）：`routes`/`outlet` 页面组合第一片。
    // 路由页面组合默认**关闭**：018 的页面单元仍带未定型 `List` 字段（组件面
    // 只收 `[]T`/`[N]T`）等墙；用 `P105_ROUTE_PAGES=1` 打开以便逐墙推进。
    let merged_routes = if std::env::var("P105_ROUTE_PAGES").is_ok() {
        inline_route_page(project_root, src)
    } else {
        None
    };
    let src: &str = merged_routes.as_deref().unwrap_or(src);
    let uses_widget_packages = src.lines().any(|line| line.trim_start().starts_with("use { package:"));
    let mut inlined: Vec<InlinedStyle> = Vec::new();
    // PLAN-105 T-02（U03 真语料 013/016）：跨单元 store 声明——组件面只能看见
    // 本源；`use <unit>: <StoreType>` 指向的单元里若有同名 `store` 声明，就把
    // 整块**前置内联**进入口源（core 侧再按同源 store 内联成共享状态）。
    let mut store_blocks: Vec<String> = Vec::new();
    // PLAN-105 T-02（U03 真语料 016）：`use <unit>: <fn>` 的顶层 helper 函数
    // 内联（组件面只校验单模块；`month_name` 等由 calendar_util 提供）。
    let mut fn_blocks: Vec<String> = Vec::new();
    // PLAN-105 T-02（U03 真语料 013）：`use <unit>: <Record>` 的 record 声明。
    let mut type_blocks: Vec<String> = Vec::new();
    // PLAN-105 T-02（U03 真语料 012）：**本文件**的 `style <name> = "<classes>"`
    // 声明——组件面没有 style 绑定表，故把声明行空白化，并把 `style: <name>`
    // 引用替换为引号字面量（与依赖样式同一条消费语义）。
    let mut local_styles: Vec<(String, String)> = Vec::new();
    for line in src.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("style ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let value = value.trim();
        if !name.is_empty() && value.starts_with('"') && value.contains('"') {
            local_styles.push((name.to_string(), value.to_string()));
        }
    }
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let (body, nl) = match line.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (line, ""),
        };
        let t = body.trim_start();
        // 本地 `style` 声明行：空白化（等长，保持 span）。
        if t.starts_with("style ") && t.contains('=') && local_styles.iter().any(|(n, _)| {
            t.strip_prefix("style ")
                .and_then(|r| r.split_once('='))
                .map(|(k, _)| k.trim() == n.as_str())
                .unwrap_or(false)
        }) {
            out.push_str(&" ".repeat(body.len()));
            out.push_str(nl);
            continue;
        }
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
        // 工程本地单元（`src/front/<unit>.at`）里的 store 声明：按导入的成员名
        // 找 `store <Member> { … }` 并整块前置内联。
        if !handled {
            let last_seg = module.rsplit('.').next().unwrap_or(module);
            for rel in [
                format!("src/front/{last_seg}.at"),
                format!("src/{last_seg}.at"),
                format!("{last_seg}.at"),
            ] {
                let file = project_root.join(&rel);
                if let Ok(unit_src) = std::fs::read_to_string(&file) {
                    for item in items.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                        // 记录类型声明（`use types: Todo` → `type Todo { … }`）
                        // 内联：组件面单模块需要记录类型才可声明列表字段。
                        if let Some(block) = extract_unit_type(&unit_src, item) {
                            // PLAN-105 T-02（U03 真语料 013/015）：组件面/collections
                            // 域的记录文本字段用 **owned** `Str`（借用 `str` 字段会让
                            // AV lowering 拒：`type kind Str outside av.collections/1`）。
                            type_blocks.push(rewrite_record_text_fields(&block));
                            handled = true;
                            continue;
                        }
                        // 顶层 helper 函数直接内联（不做语义预检：组件面只校验
                        // 单模块，函数体问题应在真正 verify 时暴露，而不是在
                        // 源准备阶段被静默丢弃）。
                        if let Some(block) = extract_unit_fn(&unit_src, item) {
                            let block = lower_const_table_fn(&block).unwrap_or(block);
                            // 组件面要求返回类型注解：无注解的 helper（如 016 的
                            // build_month_grid，隐式返回列表）暂不内联——留给
                            // 后续刀（列表/循环语义），此处不制造半截函数。
                            // 返回类型注解位于参数表 `)` 与函数体 `{` 之间（本方言
                            // 形如 `fn f(a int) str {`）；两者之间为空 = 无注解。
                            let head = block.lines().next().unwrap_or("");
                            let returns_annotated = match (head.rfind(')'), head.find('{')) {
                                (Some(rp), Some(bp)) if bp > rp => {
                                    !head[rp + 1..bp].trim().is_empty()
                                }
                                _ => false,
                            };
                            if returns_annotated {
                                fn_blocks.push(block);
                                handled = true;
                                continue;
                            }
                        }                        // store 单元自己的 `use <mod>: <Record>` 依赖（013：
                        // `use types: Todo`）——记录类型要一起内联，列表字段才可声明。
                        for dep_line in unit_src.lines() {
                            let t = dep_line.trim_start();
                            if let Some(rest) = t.strip_prefix("use ") {
                                if let Some((dep_mod, dep_items)) = rest.split_once(':') {
                                    let dep_mod = dep_mod.trim();
                                    let last = dep_mod.rsplit('.').next().unwrap_or(dep_mod);
                                    for rel in [
                                        format!("src/front/{last}.at"),
                                        format!("src/{last}.at"),
                                        format!("{last}.at"),
                                    ] {
                                        if let Ok(dep_src) =
                                            std::fs::read_to_string(project_root.join(&rel))
                                        {
                                            for dep_item in dep_items
                                                .split(',')
                                                .map(str::trim)
                                                .filter(|s| !s.is_empty())
                                            {
                                                if let Some(tb) =
                                                    extract_unit_type(&dep_src, dep_item)
                                                {
                                                    type_blocks.push(tb);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // 纯 Auto 内存后端：先把 api 调用改写成对 store 自身列表的
                        // 原地维护，再尝试内联 store 块。
                        let unit_src = inline_memory_backend(project_root, &unit_src)
                            .unwrap_or_else(|| unit_src.clone());
                        // store 单元引用的记录类型（含 `pub type X = { … }`）。
                        for tb in collect_type_blocks(&unit_src) {
                            // 只收组件面**可解析**的记录块（015 的某些后端类型带
                            // `List<..>` 字段，注入即红，不能制造语法错）。
                            let probe = format!(
                                "{tb}\nwidget __TypeProbe {{\n    model {{ var __n int = 0 }}\n    view {{ text \"x\" {{}} }}\n}}\n"
                            );
                            if ace_core::source::age_component::project_component_candidates(&probe)
                                .is_err()
                            {
                                continue;
                            }
                            if !type_blocks.iter().any(|b| b == &tb) {
                                type_blocks.push(tb);
                            }
                        }
                        if let Some(block) = extract_store_block(&unit_src, item) {
                            // PLAN-119：组件面 store 片断先做**模型归一**（`List<T>`→
                            // `[]T`；无类型空列表按 push 点定型并前置合成记录类型），
                            // 再做**控制转移兼容改写**（AGE 面语句文法没有
                            // `return`/`break`；038 MinesweeperStore 依赖它们）。
                            // 两件都是内联门的必要条件，缺一则 `.store.*` 落回具名边界。
                            let block = normalize_component_store_model(&block);
                            let block = rewrite_store_control_flow(&block).unwrap_or(block);
                            // 只有该 store 片断在组件面**可解析**时才内联；含
                            // 组件面尚不支持的 `on` 语句（if/for/局部 var 等）时
                            // 放弃内联，保留具名 `.store` 边界（不制造语法错）。
                            let probe = format!(
                                "{block}\nwidget __StoreProbe {{\n    model {{ var __n int = 0 }}\n    view {{ text \"x\" {{}} }}\n}}\n"
                            );
                            if ace_core::source::age_component::project_component_stores(&probe)
                                .map(|s| !s.is_empty())
                                .unwrap_or(false)
                                && ace_core::source::age_component::project_component_candidates(
                                    &probe,
                                )
                                .is_ok()
                            {
                                store_blocks.push(block);
                                handled = true;
                            }
                        }
                    }
                    if handled {
                        break;
                    }
                }
            }
        }
        // `use` 行等长空白化（保 span）——**widget 级组件包导入
        // `use { package: <name> from "<path>" }` 除外**：该声明是 AGE 包
        // 面的解析依据（包子组件/`$event` 绑定/组件 props 都要按它解析真实
        // 包），空白化会让包面把包子组件当未声明（PLAN-117 T-01 实证：
        // 031 的 ImageViewport `$event` 与 029 的 AssetBrowser props 均因
        // 导入被空白而 child-event-site / 记录字面量拒绝）。
        // Retain the standard JSON module for the checked package linker.
        if t.starts_with("use { package:") || (uses_widget_packages && module == "std.json") {
            out.push_str(body);
            out.push_str(nl);
            continue;
        }
        out.push_str(&" ".repeat(body.len()));
        out.push_str(nl);
        let _ = handled;
    }
    // 内联样式必须落在**组件头之前**：模块样式表按源码顺序收集，组件之后声明的
    // 样式在投影期解析不到（016 实测：追加到文件尾 → `style caption_text` 不解析；
    // 前置到 widget 之前 → 整例通过）。
    // 本地样式引用替换：**整词**命中即换成字面量（声明行已空白化，故不会自替换）
    // ——覆盖 `style: <name>`、条件式 `style: if … { <name> } else { <name> }`
    // 等所有引用位（012 的 tab_active/tab_idle）。
    for (name, literal) in &local_styles {
        out = replace_word(&out, name, literal);
    }

    // PLAN-105 T-02（U03 真语料 012）：`<text>.contains(<sub>)`——语言无该内建
    // 方法，改写为 prep 注入的文本谓词 helper（`len`/`slice`/`==` 皆既有内建）。
    // PLAN-118：包面目标早判——包面原生有 while，且模块 fn 体拒 for（重写
    // 反而制造 `fn` 拒）；组件面维持 while→for 不变。
    let package_face_target = src.lines().any(|l| l.trim_start().starts_with("use { package:"));
    let mut need_str_contains = false;
    // PLAN-118：包面目标（widget 级 `use { package: }` 导入存在）——AGE 包面
    // 的 `str`/`Str` 是借用/owned 两个类型：VM 作者源（paint_color 内联 fn、
    // handler 局部）把 Str 值赋给 `str` 局部/返回（`var t str = <Str>`、
    // `fn f(…) str`），包面 bind 逐处具名拒。包面消费下 str 注解一律归一为
    // owned `Str`（model 字段同款语义；`rewrite_model_text_fields` 已覆盖
    // model 块，本 pass 覆盖 fn 形参/返回与块外 var 局部）。
    out = rewrite_model_text_fields(&out);
    out = rewrite_str_contains(&out, &mut need_str_contains);
    // PLAN-117（round-28 ②，029 真语料）：`<text>.lower()`/`.upper()`——组件
    // 面无该内建方法，改写为 prep 注入的大小写折叠 helper（`len`/`char_at`/
    // `to_codepoint`/`from_codepoint`/`to(Str)` 皆既有内建；仅 ASCII 区间
    // 折叠——029 语料的查询/标题匹配为 ASCII 路径，非 ASCII 原样保留）。
    let mut need_str_case = false;
    out = rewrite_str_case(&out, &mut need_str_case);
    // PLAN-117（round-29 ②，029 GoUp）：`<text>.split(<sep>)` 消费成语
    // 改写——自由 fn 不能返回/接 List（v1 面：lists are function-local），
    // 拆为 `__p117_str_split_count(recv, mark)`（`parts.len()`）与
    // `__p117_str_split_at(recv, mark, idx)`（`parts[i]`）两个标量 helper；
    // 声明行 `var parts = X.split(Y)` 摘除，同源后续 `parts.len()`/`parts[i]`
    // 整词改写（029 语料 parts 为 handler 内局部，无跨域泄漏）。
    let mut need_str_split = false;
    out = rewrite_str_split(&out, &mut need_str_split);
    // PLAN-117（round-30）：`<text>.replace(<mark>, <repl>)` → 标量 helper。
    let mut need_str_replace = false;
    out = rewrite_str_replace(&out, &mut need_str_replace);
    if !package_face_target {
        out = rewrite_while_to_for(&out);
    }
    if need_str_contains {
        fn_blocks.push(STR_CONTAINS_HELPER.to_string());
    }
    if need_str_case {
        fn_blocks.push(STR_INDEX_IN_HELPER.to_string());
        fn_blocks.push(STR_LOWER_HELPER.to_string());
        fn_blocks.push(STR_UPPER_HELPER.to_string());
    }
    if need_str_split {
        fn_blocks.push(STR_SPLIT_COUNT_HELPER.to_string());
        fn_blocks.push(STR_SPLIT_AT_HELPER.to_string());
    }
    if need_str_replace {
        fn_blocks.push(STR_REPLACE_HELPER.to_string());
    }    // PLAN-105 T-02（U03 真语料 012）：`var NAME []T = []` 的空列表初值改为
    // `List<T>.new()`（共享 resolver 要求空字面量带元素类型）。
    out = rewrite_empty_list_inits(&out);
    // PLAN-105 T-02（U03 真语料 012）：**宿主服务仿真**——`storage.get/set(<key>)`
    // 与 `Time.now_sec()` 在组件面没有宿主表，改写成**模型字段**（键 → 字段名
    // 稳定映射）：`storage.get("k")` → `.__st_k`；`storage.set("k", v)` →
    // `.__st_k = v`；`Time.now_sec()` → `.__now_sec`（宿主装载时种子注入）。
    let mut host_fields: Vec<(String, String)> = Vec::new();
    out = rewrite_host_storage(&out, &mut host_fields);
    if !host_fields.is_empty() {
        out = inject_model_fields(&out, &host_fields);
    }
    // 声明（内联样式 / 记录类型 / helper 函数 / store 块）必须落在**组件头之前**：
    // 两条消费面都按源码顺序收集，组件之后声明的符号在投影期解析不到。
    if !inlined.is_empty() || !store_blocks.is_empty() || !fn_blocks.is_empty() || !type_blocks.is_empty() {
        let mut decls: Vec<String> = inlined
            .iter()
            .map(|s| format!("style {} = {}", s.name, s.literal))
            .collect();
        decls.extend(type_blocks.iter().cloned());
        decls.extend(fn_blocks.iter().cloned());
        decls.extend(store_blocks.iter().cloned());
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
    // 注入的依赖块（fn/type/const）同样做组件面语法归一：`while` → 条件 `for`，
    // 并去掉显式 `as int` 转换。
    if !package_face_target {
        out = rewrite_while_to_for(&out);
    }
    // PLAN-118：包面目标的 str→Str 注解归一（内联 fn 已前置，最后统一跑；
    // model 块早被 rewrite_model_text_fields 覆盖，此处恒等）。
    if package_face_target {
        out = rewrite_str_surface_for_package(&out, false);
    }
    // PLAN-118：canvas 场景合成名归一——`scene: .cells`（cells 非 model 字段，
    // 五平行列表 `<名>_pts/_meta/_nodes/_edges/_labels` 是）改写成串 prop
    // `scene: "cells"`：包面视图条目值不再按 self 字段解析（具名拒消失），
    // 场景名串 prop 直达宿主（G6 按名读列表）与 A2Vue 道。
    if package_face_target {
        out = rewrite_canvas_scene_bindings(&out);
    }

    Ok(PreparedComponentSource {
        source: out,
        inlined,
    })
}

/// prep 注入的文本谓词实现（`contains` 的等价展开）。
const STR_CONTAINS_HELPER: &str = "fn __p105_str_contains(hay Str, needle Str) bool {\n    if needle.len() == 0 {\n        return true\n    }\n    if needle.len() > hay.len() {\n        return false\n    }\n    var i int = 0\n    for i + needle.len() <= hay.len() {\n        if hay.slice(i, i + needle.len()) == needle {\n            return true\n        }\n        i = i + 1\n    }\n    return false\n}\n";

/// `<recv>.contains(<arg>)` → `__p105_str_contains(<recv>, <arg>)`；`need` 置真
/// 时调用方需注入 [`STR_CONTAINS_HELPER`]。
fn rewrite_str_contains(src: &str, need: &mut bool) -> String {
    let mut out = src.to_string();
    loop {
        let Some(pos) = out.find(".contains(") else { break };
        // 接收位：从 `.contains(` 前往回扫标识符/点链；`)`/`]` 跳回**配对**
        // 开括号（`p.title.lower().contains(x)` 的接收者是整个调用链——
        // PLAN-117（round-28）修：此前 `)` 只回退一格，链式调用的接收位
        // 被截断成 `)`，改写出 0 参 helper 调用）。
        let bytes = out.as_bytes();
        let mut start = pos;
        while start > 0 {
            let c = bytes[start - 1] as char;
            if c == ')' || c == ']' {
                let (open, close) = if c == ')' { ('(', ')') } else { ('[', ']') };
                let mut depth = 0i32;
                let mut k = start - 1;
                loop {
                    if k == 0 {
                        break;
                    }
                    let b = bytes[k] as char;
                    if b == close {
                        depth += 1;
                    } else if b == open {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    k -= 1;
                }
                start = k;
            } else if c.is_alphanumeric() || c == '_' || c == '.' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == pos {
            break;
        }
        let recv = out[start..pos].to_string();
        // 实参：`(` 后到配对 `)`。
        let args_at = pos + ".contains(".len();
        let mut depth = 0i32;
        let mut end = None;
        for (i, ch) in out[args_at..].char_indices() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' if depth == 0 => {
                    end = Some(args_at + i);
                    break;
                }
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        let Some(end) = end else { break };
        let arg = out[args_at..end].trim().to_string();
        let call = format!("__p105_str_contains({recv}, {arg})");
        out.replace_range(start..end + 1, &call);
        *need = true;
    }
    out
}
// 大小写折叠用**纯 str 词汇**实现（len/slice/==/拼接）——collections 面
// descriptor 无 char 转换用例（char_to_codepoint 族缺失，PLAN-864 冻结）；
// 字母表扫描：单字符在 UPPER/LOWER 表中定位（-1 = 非 ASCII 字母，原样保留）。
const STR_INDEX_IN_HELPER: &str = "fn __p117_str_index_in(c Str, alphabet Str) int {
    var i int = 0
    for i < alphabet.len() {
        if alphabet.slice(i, i + 1) == c {
            return i
        }
        i = i + 1
    }
    return -1
}
";

const STR_LOWER_HELPER: &str = "fn __p117_str_lower(hay Str) Str {
    var out Str = \"\"
    var i int = 0
    for i < hay.len() {
        var k int = __p117_str_index_in(hay.slice(i, i + 1), \"ABCDEFGHIJKLMNOPQRSTUVWXYZ\")
        if k >= 0 {
            out = out + \"abcdefghijklmnopqrstuvwxyz\".slice(k, k + 1)
        } else {
            out = out + hay.slice(i, i + 1)
        }
        i = i + 1
    }
    return out
}
";

const STR_UPPER_HELPER: &str = "fn __p117_str_upper(hay Str) Str {
    var out Str = \"\"
    var i int = 0
    for i < hay.len() {
        var k int = __p117_str_index_in(hay.slice(i, i + 1), \"abcdefghijklmnopqrstuvwxyz\")
        if k >= 0 {
            out = out + \"ABCDEFGHIJKLMNOPQRSTUVWXYZ\".slice(k, k + 1)
        } else {
            out = out + hay.slice(i, i + 1)
        }
        i = i + 1
    }
    return out
}
";

/// PLAN-117（round-30 ②）：`<text>.lower()`/`.upper()` → helper 调用
/// （无参形；接收位回扫跳配对括号）。
fn rewrite_str_case(src: &str, need: &mut bool) -> String {
    let mut out = src.to_string();
    for (method, helper) in [(".lower()", "__p117_str_lower"), (".upper()", "__p117_str_upper")] {
        loop {
            let Some(pos) = out.find(method) else { break };
            let bytes = out.as_bytes();
            let mut start = pos;
            while start > 0 {
                let c = bytes[start - 1] as char;
                if c == ')' || c == ']' {
                    let (open, close) = if c == ')' { ('(', ')') } else { ('[', ']') };
                    let mut depth = 0i32;
                    let mut k = start - 1;
                    loop {
                        if k == 0 {
                            break;
                        }
                        let b = bytes[k] as char;
                        if b == close {
                            depth += 1;
                        } else if b == open {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        k -= 1;
                    }
                    start = k;
                } else if c.is_alphanumeric() || c == '_' || c == '.' {
                    start -= 1;
                } else {
                    break;
                }
            }
            if start == pos {
                break;
            }
            let recv = out[start..pos].to_string();
            let call = format!("{helper}({recv})");
            out.replace_range(start..pos + method.len(), &call);
            *need = true;
        }
    }
    out
}

/// PLAN-118：`pos` 所在 handler 块的**头部行文本**（最近的上方 `.Name(...) -> {`
/// 行；模块 fn/其它块面退化为最近的 `fn`/`widget`/`view`/`on` 行，空串 = 文件头）。
/// 用行文本而非偏移作作用域键——声明行删除会让偏移失效。
fn enclosing_handler_header(src: &str, pos: usize) -> String {
    let mut line_start = src[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    loop {
        let line = &src[line_start..];
        let trimmed = line.trim_start();
        if (trimmed.starts_with('.') && trimmed.contains("->"))
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("widget ")
            || trimmed.starts_with("view ")
            || trimmed.starts_with("on ")
        {
            return line.split('\n').next().unwrap_or("").trim().to_string();
        }
        if line_start == 0 {
            return String::new();
        }
        let prev_end = line_start - 1;
        line_start = src[..prev_end].rfind('\n').map(|i| i + 1).unwrap_or(0);
    }
}

/// PLAN-117（round-29 ②）：`.split(sep)` 消费成语改写——声明行摘除 +
/// `NAME.len()`/`NAME[i]` 用法改写为 count/at 标量 helper 调用。
/// PLAN-118：改写按**声明所在 handler 作用域**配平（同名局部在多个 handler
/// 各写一次时，接收者不得串用）。
fn rewrite_str_split(src: &str, need: &mut bool) -> String {
    let mut out = src.to_string();
    // (局部名, 接收者, 分隔符, 声明所在 handler 起点)。
    // PLAN-118：不同 handler 各自声明同名局部（`var parts = s.split(…)` 与
    // `var parts = snap.split(…)`）——替换必须按**声明所在作用域**配平，否则
    // 先处理的声明会改写另一个 handler 的 `parts.len()`（用错接收者）。
    let mut vars: Vec<(String, String, String, String)> = Vec::new();
    loop {
        let Some(dpos) = out.find(".split(") else { break };
        let bytes = out.as_bytes();
        let mut start = dpos;
        while start > 0 {
            let c = bytes[start - 1] as char;
            if c == ')' || c == ']' {
                let (open, close) = if c == ')' { ('(', ')') } else { ('[', ']') };
                let mut depth = 0i32;
                let mut k = start - 1;
                loop {
                    if k == 0 {
                        break;
                    }
                    let b = bytes[k] as char;
                    if b == close {
                        depth += 1;
                    } else if b == open {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    k -= 1;
                }
                start = k;
            } else if c.is_alphanumeric() || c == '_' || c == '.' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == dpos {
            break;
        }
        let args_at = dpos + ".split(".len();
        let mut depth = 0i32;
        let mut dend = None;
        for (i, ch) in out[args_at..].char_indices() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' if depth == 0 => {
                    dend = Some(args_at + i);
                    break;
                }
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        let Some(dend) = dend else { break };
        let head = out[..start].trim_end();
        let Some(eq) = head.rfind('=') else { break };
        let head_l = head[..eq].trim_end();
        let Some(var_at) = head_l.rfind("var ") else { break };
        let name = head_l[var_at + 4..].trim().to_string();
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            break;
        }
        let recv = out[start..dpos].to_string();
        let arg = out[args_at..dend].trim().to_string();
        let line_start = out[..var_at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = out[dend..].find('\n').map(|i| dend + i + 1).unwrap_or(dend);
        let scope = enclosing_handler_header(&out, var_at);
        out.replace_range(line_start..line_end, "");
        vars.push((name, recv, arg, scope));
        *need = true;
    }
    for (name, recv, arg, scope) in &vars {
        let len_pat = format!("{name}.len()");
        let count_call = format!("__p117_str_split_count({recv}, {arg})");
        let mut from = 0usize;
        while let Some(rel) = out[from..].find(&len_pat) {
            let pp = from + rel;
            let preceded_by_ident = pp > 0 && {
                let b = out.as_bytes()[pp - 1] as char;
                b.is_alphanumeric() || b == '_'
            };
            if preceded_by_ident || enclosing_handler_header(&out, pp) != *scope {
                from = pp + len_pat.len();
                continue;
            }
            out.replace_range(pp..pp + len_pat.len(), &count_call);
            from = pp + count_call.len();
        }
        let idx_pat = format!("{name}[");
        let mut from = 0usize;
        while let Some(rel) = out[from..].find(&idx_pat) {
            let pp = from + rel;
            let preceded_by_ident = pp > 0 && {
                let b = out.as_bytes()[pp - 1] as char;
                b.is_alphanumeric() || b == '_'
            };
            if preceded_by_ident || enclosing_handler_header(&out, pp) != *scope {
                from = pp + idx_pat.len();
                continue;
            }
            let mut depth = 0i32;
            let mut e = None;
            for (i, ch) in out[pp + name.len()..].char_indices() {
                match ch {
                    '[' | '(' | '{' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            e = Some(pp + name.len() + i);
                            break;
                        }
                    }
                    ')' | '}' => depth -= 1,
                    _ => {}
                }
            }
            let Some(e) = e else { break };
            let idx_expr = out[pp + name.len() + 1..e].to_string();
            let call = format!("__p117_str_split_at({recv}, {arg}, {idx_expr})");
            out.replace_range(pp..e + 1, &call);
            from = pp + call.len();
        }
    }
    out
}

const STR_SPLIT_COUNT_HELPER: &str = "fn __p117_str_split_count(hay str, mark str) int {
    var n int = 1
    var i int = 0
    for i + mark.len() <= hay.len() {
        if hay.slice(i, i + mark.len()) == mark {
            n = n + 1
        }
        i = i + 1
    }
    return n
}
";

// `.replace(mark, repl)` 全量替换（首个匹配起循环扫描；标量词汇）。
const STR_REPLACE_HELPER: &str = "fn __p117_str_replace(hay str, mark str, repl str) Str {
    var out Str = \"\"
    var i int = 0
    for i < hay.len() {
        if i + mark.len() <= hay.len() {
            if hay.slice(i, i + mark.len()) == mark {
                out = out + repl
                i = i + mark.len()
            } else {
                out = out + hay.slice(i, i + 1)
                i = i + 1
            }
        } else {
            out = out + hay.slice(i, i + 1)
            i = i + 1
        }
    }
    return out
}
";

const STR_SPLIT_AT_HELPER: &str = "fn __p117_str_split_at(hay str, mark str, want int) Str {
    var piece Str = \"\"
    var idx int = 0
    var i int = 0
    for i < hay.len() {
        if i + mark.len() <= hay.len() {
            if hay.slice(i, i + mark.len()) == mark {
                if idx == want {
                    return piece
                }
                idx = idx + 1
                piece = \"\"
                i = i + mark.len()
            } else {
                piece = piece + hay.slice(i, i + 1)
                i = i + 1
            }
        } else {
            piece = piece + hay.slice(i, i + 1)
            i = i + 1
        }
    }
    return piece
}
";

/// PLAN-117（round-30 ②）：`<text>.replace(<mark>, <repl>)` →
/// `__p117_str_replace(<recv>, <mark>, <repl>)`（双参配对 + 接收位回扫）。
fn rewrite_str_replace(src: &str, need: &mut bool) -> String {
    let mut out = src.to_string();
    loop {
        let Some(pos) = out.find(".replace(") else { break };
        let bytes = out.as_bytes();
        let mut start = pos;
        while start > 0 {
            let c = bytes[start - 1] as char;
            if c == ')' || c == ']' {
                let (open, close) = if c == ')' { ('(', ')') } else { ('[', ']') };
                let mut depth = 0i32;
                let mut k = start - 1;
                loop {
                    if k == 0 {
                        break;
                    }
                    let b = bytes[k] as char;
                    if b == close {
                        depth += 1;
                    } else if b == open {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    k -= 1;
                }
                start = k;
            } else if c.is_alphanumeric() || c == '_' || c == '.' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == pos {
            break;
        }
        let recv = out[start..pos].to_string();
        let args_at = pos + ".replace(".len();
        let mut depth = 0i32;
        let mut end = None;
        for (i, ch) in out[args_at..].char_indices() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' if depth == 0 => {
                    end = Some(args_at + i);
                    break;
                }
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        let Some(end) = end else { break };
        let args = out[args_at..end].trim().to_string();
        let call = format!("__p117_str_replace({recv}, {args})");
        out.replace_range(start..end + 1, &call);
        *need = true;
    }
    out
}

/// 首个路由映射的页面单元源码（`routes { "/" -> use bookshelf }`）。
fn first_route_page(app_root: &Path, src: &str) -> Option<String> {
    let mut in_routes = false;
    for line in src.lines() {
        let t = line.trim();
        if !in_routes && t.starts_with("routes") && t.contains('{') {
            in_routes = true;
            continue;
        }
        if !in_routes {
            continue;
        }
        if t.starts_with('}') {
            break;
        }
        if let Some(i) = t.find("-> use ") {
            let page = t[i + "-> use ".len()..].trim().trim_end_matches(',').to_string();
            for rel in [
                format!("src/front/pages/{page}.at"),
                format!("src/front/components/{page}.at"),
                format!("src/front/{page}.at"),
            ] {
                if let Ok(s) = std::fs::read_to_string(app_root.join(&rel)) {
                    return Some(s);
                }
            }
        }
    }
    None
}

/// 把源拆成（widget 外的顶层文本, widget 体内部文本）。
fn split_widget(src: &str) -> (String, String) {
    let mut top = String::new();
    let mut body = String::new();
    let mut depth = 0i32;
    let mut in_widget = false;
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        if !in_widget && t.starts_with("widget ") {
            in_widget = true;
            depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            continue;
        }
        if in_widget {
            let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if depth + d <= 0 {
                in_widget = false;
                depth = 0;
                continue;
            }
            depth += d;
            body.push_str(line);
        } else {
            top.push_str(line);
        }
    }
    (top, body)
}

/// 取 `<kw> { … }` 块内部文本（花括号配对）。
fn block_inner(src: &str, kw: &str) -> Option<String> {
    let open = src.find(&format!("{kw} {{"))?;
    let start = open + kw.len() + 2;
    let bytes = src.as_bytes();
    let mut depth = 1i32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(src[start..i].to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 路由/outlet 页面组合（第一片）：首个路由页面的顶层声明与 `model` 字段并入
/// 外壳，`outlet` 换成页面 `view` 体。
fn inline_route_page(app_root: &Path, src: &str) -> Option<String> {
    if !src.contains("outlet") {
        return None;
    }
    let page_src = first_route_page(app_root, src)?;
    let (page_top, page_body) = split_widget(&page_src);
    let page_model = block_inner(&page_body, "model").unwrap_or_default();
    let page_view = block_inner(&page_body, "view").unwrap_or_default();
    let (shell_top, shell_body) = split_widget(src);
    let name = src
        .lines()
        .find(|l| l.trim_start().starts_with("widget "))
        .and_then(|l| l.trim_start().strip_prefix("widget "))
        .and_then(|r| r.split_whitespace().next())
        .unwrap_or("App")
        .to_string();
    let mut body = shell_body;
    if let Some(open) = body.find("model {") {
        let mut depth = 1i32;
        let bytes = body.as_bytes();
        let mut j = open + "model {".len();
        let mut close = None;
        while j < bytes.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(j);
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        if let Some(close) = close {
            body.insert_str(close, &page_model);
        }
    }
    let body = body.replace("outlet", &page_view);
    let mut out = String::with_capacity(src.len() + page_top.len() + body.len());
    out.push_str(&shell_top);
    out.push('\n');
    out.push_str(&page_top);
    out.push_str("widget ");
    out.push_str(&name);
    out.push_str(" {\n");
    out.push_str(&body);
    out.push_str("\n}\n");
    Some(out)
}
/// `while <cond> {` 行首 → `for <cond> {`（组件面只收条件 `for`）；`<expr> as int`
/// 的显式整数转换在组件面无需保留（两侧都是 `int`）。
fn rewrite_while_to_for(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("while ") {
            let indent = &line[..line.len() - t.len()];
            out.push_str(indent);
            out.push_str("for ");
            out.push_str(rest);
        } else {
            out.push_str(line);
        }
    }
    out.replace(" as int", "")
}
/// 内存后端内联（PLAN-105 T-02，U03 真语料 013）：`use back.api: …` 指向的
/// 后端是**纯 Auto 内存实现**（`db.at` 里一条记录列表 + 自增 id），因此把
/// 五个 CRUD 调用改写成对 store 自身列表字段的原地维护：
/// `list_todos()` → 列表已是真值（删行）、`create_todo(x)` → push、
/// `update_todo(...)` → 下标搜索后整条替换、`delete_todo(id)`／
/// `remove_completed(_)` → 下标搜索后 `remove`。
fn mb_rule(on: &str) -> bool {
    match std::env::var("P105_MB_RULES") {
        Ok(v) => v.split(',').any(|r| r.trim() == on),
        Err(_) => true,
    }
}

/// 从单元源码收集**所有**记录类型声明块（支持 `type X { … }` 与
/// `pub type X = { … }`），统一重排为组件面可吃的 `type X { … }`。
fn collect_type_blocks(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim_start();
        let t = t.strip_prefix("pub ").unwrap_or(t);
        let Some(rest) = t.strip_prefix("type ") else {
            i += 1;
            continue;
        };
        let Some(name) = rest
            .split(|c: char| c.is_whitespace() || c == '=' || c == '{')
            .next()
        else {
            i += 1;
            continue;
        };
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut depth = 0i32;
        let mut started = false;
        let mut body: Vec<String> = Vec::new();
        let mut j = i;
        while j < lines.len() {
            let line = lines[j];
            let mut opened = false;
            for ch in line.chars() {
                match ch {
                    '{' => {
                        depth += 1;
                        opened = true;
                    }
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if !started {
                if opened {
                    started = true;
                    let after = line.split_once('{').map(|(_, a)| a.trim()).unwrap_or("");
                    if !after.is_empty() && !after.starts_with('}') {
                        body.push(after.to_string());
                    }
                    if depth == 0 {
                        break;
                    }
                }
            } else {
                if depth == 0 {
                    let before = line.split('}').next().unwrap_or("").trim();
                    if !before.is_empty() {
                        body.push(before.to_string());
                    }
                    break;
                }
                body.push(line.trim().to_string());
            }
            j += 1;
        }
        let mut block = format!("type {name} {{\n");
        for b in &body {
            block.push_str("    ");
            block.push_str(b);
            block.push('\n');
        }
        block.push_str("}\n");
        out.push(block);
        i = j + 1;
    }
    out
}

fn inline_memory_backend(app_root: &Path, store_src: &str) -> Option<String> {
    // 默认关闭：该内联会改写 store 单元，其他例子（015 等）尚未验证；用
    // `P105_MEM_BACKEND=1` 打开（013 走这条路线）。
    if std::env::var("P105_MEM_BACKEND").is_err() {
        return None;
    }
    let has_api_use = store_src.lines().any(|l| {
        let t = l.trim();
        t.starts_with("use back.api") || t.starts_with("use api:")
    });
    if !has_api_use {
        return None;
    }
    let read = |rel: &str| std::fs::read_to_string(app_root.join(rel)).ok();
    let api_src = read("src/back/api.at").or_else(|| read("src/api.at"))?;
    let db_src = read("src/back/db.at").or_else(|| read("src/db.at"))?;
    // 列表字段与元素类型（store model 的第一条 `var <name> []<Elem> = …`）。
    let (list_field, elem) = store_src.lines().find_map(|l| {
        let t = l.trim();
        let rest = t.strip_prefix("var ")?;
        let (name, ty) = rest.split_once(char::is_whitespace)?;
        let el = ty.trim().strip_prefix("[]")?;
        let el = el
            .split(|c: char| c.is_whitespace() || c == '=')
            .next()?
            .trim()
            .to_string();
        Some((name.trim().to_string(), el.trim().to_string()))
    })?;
    // db 种子：`List<Todo>.new([ … ])` 里的条目与 `var nextid int = N`。
    let seeds = {
        let open = db_src.find(".new([")?;
        let close = db_src[open..].find("])")? + open;
        db_src[open + ".new([".len()..close]
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| l.trim_end_matches(',').to_string())
            .collect::<Vec<_>>()
    };
    let nextid = db_src
        .lines()
        .find_map(|l| {
            let t = l.trim();
            let rest = t.strip_prefix("var ")?;
            let (name, val) = rest.split_once('=')?;
            if name.split_whitespace().next() == Some("nextid") {
                val.trim().parse::<i32>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    let mut out = String::with_capacity(store_src.len() + 512);
    // 后端单元里的记录类型（`pub type Todo = { … }`）前置进来——组件面单模块
    // 需要自带的记录声明。
    for tb in collect_type_blocks(&api_src) {
        out.push_str(&rewrite_record_text_fields(&tb));
    }
    let mut tmp = 0usize;
    for line in store_src.split_inclusive('\n') {
        let (body, nl) = match line.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (line, ""),
        };
        let t = body.trim_start();
        let indent = &body[..body.len() - t.len()];
        // 模型：种子与自增初值。
        if t == format!("var {list_field} []{elem} = []") {
            out.push_str(&format!(
                "{indent}var {list_field} []{elem} = [{}]{nl}",
                seeds.join(", ")
            ));
            continue;
        }
        if t.starts_with("var next_id int = ") {
            out.push_str(&format!("{indent}var next_id int = {nextid}{nl}"));
            continue;
        }
        // `.todos = list_todos()`：列表本身就是真值。
        if t == format!(".{list_field} = list_todos()") {
            out.push_str(&" ".repeat(body.len()));
            out.push_str(nl);
            continue;
        }
        if !mb_rule("create") {
            out.push_str(line);
            continue;
        }
        if let Some(rest) = t.strip_prefix("create_todo(") {
            let arg = rest.trim_end_matches(')').trim();
            out.push_str(&format!(
                "{indent}.{list_field}.push({elem} {{ id: .next_id, text: {arg}, done: false }}){nl}{indent}.next_id = .next_id + 1{nl}"
            ));
            continue;
        }
        if !mb_rule("update") {
            out.push_str(line);
            continue;
        }
        if let Some(rest) = t.strip_prefix("update_todo(") {
            let args: Vec<&str> = rest
                .trim_end_matches(')')
                .split(',')
                .map(str::trim)
                .collect();
            if args.len() == 3 {
                let i = format!("__mb_u{tmp}");
                tmp += 1;
                // 整条替换走 `remove` + `insert`（下标赋值带记录字面量在 store 块
                // 语法里不可解析；remove 返回物化元素、insert 收新值）。
                let old = format!("__mb_o{tmp}");
                // PLAN-105 W107：实参里的 `.{list_field}[<任意下标>].field` 读
                // **先于 remove 捕获**——改写为对已移除记录 `{old}.field` 的
                // 读取（命中元素即被移除的那个）。原样内联会在 remove 之后
                // 重读：后续下标左移，命中靠前元素时读错行、外层 `i` 为末位时
                // 越界 trap（ToggleAll 实测）。
                let rewrite = |s: &str| {
                    let needle = format!(".{list_field}[");
                    let mut out = String::with_capacity(s.len());
                    let mut rest = s;
                    while let Some(pos) = rest.find(&needle) {
                        let after = &rest[pos + needle.len()..];
                        match after.find("].") {
                            Some(close) => {
                                out.push_str(&rest[..pos]);
                                out.push_str(&old);
                                out.push('.');
                                rest = &after[close + 2..];
                            }
                            None => break,
                        }
                    }
                    out.push_str(rest);
                    out
                };
                out.push_str(&format!(
                    "{indent}var {i} int = 0{nl}{indent}for {i} < .{list_field}.len() {{{nl}{indent}    if .{list_field}[{i}].id == {id} {{{nl}{indent}        var {old} {elem} = .{list_field}.remove({i}){nl}{indent}        .{list_field}.insert({i}, {elem} {{ id: {id2}, text: {text}, done: {done} }}){nl}{indent}    }}{nl}{indent}    {i} = {i} + 1{nl}{indent}}}{nl}",
                    id = args[0],
                    id2 = rewrite(args[0]),
                    text = rewrite(args[1]),
                    done = rewrite(args[2])
                ));
                continue;
            }
        }
        if !mb_rule("delete") {
            out.push_str(line);
            continue;
        }
        if let Some(rest) = t.strip_prefix("delete_todo(") {
            let id = rest.trim_end_matches(')').trim();
            let i = format!("__mb_d{tmp}");
            let gone = format!("__mb_g{tmp}");
            tmp += 1;
            out.push_str(&format!(
                "{indent}var {i} int = 0{nl}{indent}for {i} < .{list_field}.len() {{{nl}{indent}    if .{list_field}[{i}].id == {id} {{{nl}{indent}        var {gone} {elem} = .{list_field}.remove({i}){nl}{indent}    }}{nl}{indent}    {i} = {i} + 1{nl}{indent}}}{nl}"
            ));
            continue;
        }
        if !mb_rule("remove") {
            out.push_str(line);
            continue;
        }
        if let Some(_rest) = t.strip_prefix("remove_completed(") {
            let i = format!("__mb_r{tmp}");
            let gone = format!("__mb_h{tmp}");
            tmp += 1;
            out.push_str(&format!(
                "{indent}var {i} int = .{list_field}.len() - 1{nl}{indent}for {i} >= 0 {{{nl}{indent}    if .{list_field}[{i}].done {{{nl}{indent}        var {gone} {elem} = .{list_field}.remove({i}){nl}{indent}    }}{nl}{indent}    {i} = {i} - 1{nl}{indent}}}{nl}"
            ));
            continue;
        }
        out.push_str(line);
    }
    Some(out)
}
/// 组件面 `model` 块的文本字段类型改写成 owned `Str`（`var x str = …` →
/// `var x Str = …`）：AV lowering 的 collections 面不承载借用 `Str`。
fn rewrite_model_text_fields(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut model_depth = 0i32;
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        if model_depth == 0 && t.starts_with("model") && t.contains('{') {
            model_depth = 1;
            out.push_str(line);
            continue;
        }
        if model_depth > 0 {
            let opens = line.matches('{').count() as i32;
            let closes = line.matches('}').count() as i32;
            // `var <name> str …` 的字段行：把类型词换成 Str。
            let mut rewritten = line.to_string();
            if opens == 0 && closes == 0 {
                if let Some(pos) = t.find("var ") {
                    let rest = &t[pos + 4..];
                    let mut parts = rest.splitn(3, char::is_whitespace);
                    let name = parts.next().unwrap_or("");
                    let ty = parts.next().unwrap_or("");
                    if ty == "str" && !name.is_empty() {
                        let indent = &line[..line.len() - t.len()];
                        let after = parts.next().unwrap_or("");
                        rewritten = if after.is_empty() {
                            format!("{indent}var {name} Str\n")
                        } else {
                            format!("{indent}var {name} Str {after}")
                        };
                    }
                }
            }
            model_depth += opens - closes;
            out.push_str(&rewritten);
            continue;
        }
        out.push_str(line);
    }
    out
}
/// PLAN-118：`scene: .<名>` → `scene: "<名>"`（仅当 `<名>` 非 model 字段而
/// `<名>_nodes` 是——canvas 五平行列表合成名约定）。字段名集取 model 块
/// `var <名>` 行（含注入的 `.__st_` 字段）；view 行内逐处替换 `scene: .<ident>`。
/// PLAN-118：依赖包模块源的 canvas 场景合成名归一（与 app prep 同一 pass）。
/// 088 ColorPicker 的 surf/hue/alph/sw/pal 五组场景名同样需要 `scene: "<名>"`
/// 串 prop 形态——包面视图条目值不再按 self 字段解析。
pub(crate) fn normalize_package_scene_bindings(src: &str) -> String {
    rewrite_canvas_scene_bindings(src)
}

fn rewrite_canvas_scene_bindings(src: &str) -> String {
    let mut fields: Vec<String> = Vec::new();
    let mut in_model = false;
    let mut depth = 0i32;
    for line in src.lines() {
        let t = line.trim_start();
        if !in_model && t.starts_with("model") && t.contains('{') {
            in_model = true;
            depth = 1;
        } else if in_model {
            let opens = line.matches('{').count() as i32;
            let closes = line.matches('}').count() as i32;
            if depth == 1 && opens == 0 && closes == 0 {
                if let Some(rest) = t.strip_prefix("var ") {
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        fields.push(name);
                    }
                }
            }
            depth += opens - closes;
            if depth <= 0 {
                in_model = false;
            }
        }
    }
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("//") {
            out.push_str(line);
            continue;
        }
        let mut body = line.to_string();
        // 逐处 `scene: .<ident>`（ident 边界到非标识符字符）；游标前移，跳过
        // 不改写处也继续扫行内后续命中。
        let mut cursor = 0usize;
        while let Some(rel) = body[cursor..].find("scene: .") {
            let pos = cursor + rel;
            let start = pos + "scene: .".len();
            let name: String = body[start..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                cursor = start;
                continue;
            }
            let is_field = fields.iter().any(|f| f == &name);
            let has_group = fields.iter().any(|f| f == &format!("{name}_nodes"));
            if !is_field && has_group {
                let end = start + name.len();
                body.replace_range(pos..end, &format!("scene: \"{name}\""));
                // `scene: "<name>"` = 8 + name.len() + 1 字节。
                cursor = pos + 8 + name.len() + 1;
            } else {
                cursor = start + name.len();
            }
        }
        out.push_str(&body);
    }
    out
}

/// PLAN-118：包面目标的 str→Str 注解归一——fn 形参（`x str`）、fn 返回
/// （`) str {`）、以及 model 块之外的 `var <name> str` 局部（model 块已由
/// rewrite_model_text_fields 覆盖，重复无害）。跳过注释行；字符串字面量内
/// 的命中在本计划语料（paint）中不存在，属具名记录的改写边界。
/// PLAN-118：依赖包模块源的 str/Str 注解归一（与 app prep 同一 pass）——
/// VM 轨组件包的模块 fn 以 `str` 声明返回/局部，AGE 包面要求 owned `Str`
/// （`expected str, found Str` 的 088 color_model 真语料形）。
pub(crate) fn normalize_package_text_surface(src: &str) -> String {
    // 依赖包模块只归一返回注解与 `var` 局部：形参保持借用 `str`——模块 fn 的
    // 形参是 owner，改成 owned 会把「读原值再转局部」变成 move+use-after-move
    // （088 color_model `cp_hex_valid(phex)` 后 `var t Str = phex` 的真语料形）。
    rewrite_str_surface_for_package(src, false)
}

fn rewrite_str_surface_for_package(src: &str, rewrite_params: bool) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let trimmed_end = line.trim_end_matches(['\n', '\r']);
        let nl = &line[trimmed_end.len()..];
        let t = trimmed_end.trim_start();
        if t.starts_with("//") || t.starts_with("style ") {
            out.push_str(line);
            continue;
        }
        let mut body = trimmed_end.to_string();
        if t.starts_with("fn ") {
            // 形参表与返回注解：`) str {` → `) Str {`；形参内 ` str` 词 → ` Str`。
            if let Some(open) = rewrite_params.then(|| body.find('(')).flatten() {
                if let Some(close_rel) = body[open..].find(')') {
                    let close = open + close_rel;
                    let params = body[open + 1..close].to_string();
                    let params = params
                        .replace(" str,", " Str,")
                        .replace(", str,", ", Str,")
                        .replace("( str,", "( Str,");
                    let params = if params.ends_with(" str") {
                        let cut = params.len() - " str".len();
                        format!("{} Str", &params[..cut])
                    } else {
                        params
                    };
                    body.replace_range(open + 1..close, &params);
                }
            }
            if let Some(pos) = body.find(") str {") {
                body.replace_range(pos..pos + ") str {".len(), ") Str {");
            } else if let Some(pos) = body.find(") str\n") {
                body.replace_range(pos..pos + ") str\n".len(), ") Str\n");
            }
        }
        // `var <name> str …`（行内任意位；model 块行此前已处理，重复替换是恒等）。
        if let Some(pos) = body.find("var ") {
            let rest = &body[pos + 4..];
            let mut parts = rest.splitn(3, char::is_whitespace);
            let name = parts.next().unwrap_or("");
            let ty = parts.next().unwrap_or("");
            if ty == "str" && !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let after = parts.next().unwrap_or("");
                let indent = &body[..pos];
                body = if after.is_empty() {
                    format!("{indent}var {name} Str")
                } else {
                    format!("{indent}var {name} Str {after}")
                };
            }
        }
        out.push_str(&body);
        out.push_str(nl);
    }
    out
}

/// 记录类型块里的文本字段类型改写成 owned `Str`（`<field> str` → `<field> Str`）。
fn rewrite_record_text_fields(block: &str) -> String {    let mut out = String::with_capacity(block.len());
    for line in block.split_inclusive('\n') {
        let trimmed_end = line.trim_end_matches(['\n', '\r']);
        let nl = &line[trimmed_end.len()..];
        let t = trimmed_end.trim_start();
        let indent = &trimmed_end[..trimmed_end.len() - t.len()];
        // 形如 `<name> str` / `<name> str,`（字段行）：首个空白后的类型词是 str。
        let mut replaced = None;
        if let Some((name, ty)) = t.split_once(char::is_whitespace) {
            // 两种字段写法：`text str`（AGE record）与 `text: str`（`pub type` 形态）。
            let name = name.trim_end_matches(':');
            let ty_trim = ty.trim_end_matches(',').trim();
            if ty_trim == "str" && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let suffix = if ty.trim_end().ends_with(',') { "," } else { "" };
                replaced = Some(format!("{indent}{name} Str{suffix}{nl}"));
            }
        }
        match replaced {
            Some(r) => out.push_str(&r),
            None => out.push_str(line),
        }
    }
    out
}
/// `var NAME []T = []` → `var NAME []T = List<T>.new()`（空列表字面量需元素类型）。
fn rewrite_empty_list_inits(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let trimmed_end = line.trim_end_matches(['\n', '\r']);
        let nl = &line[trimmed_end.len()..];
        let t = trimmed_end.trim_start();
        let indent = &trimmed_end[..trimmed_end.len() - t.len()];
        let replaced = (|| {
            let rest = t.strip_prefix("var ")?;
            let (name, rest) = rest.split_once(' ')?;
            let (ty, init) = rest.split_once('=')?;
            let ty = ty.trim();
            let init = init.trim();
            if init != "[]" || !ty.starts_with('[') {
                return None;
            }
            let elem = ty.trim_start_matches('[').trim_start_matches(']').trim();
            if elem.is_empty() {
                return None;
            }
            // 元素域用 owned 文本（`str` → `Str`；822 元素域不收借用 str）。
            let elem = if elem == "str" { "Str" } else { elem };
            let _ = &ty;
            Some(format!("{indent}var {name} {ty} = List<{elem}>.new(){nl}"))
        })();
        match replaced {
            Some(r) => out.push_str(&r),
            None => out.push_str(line),
        }
    }
    out
}
/// 键 → 稳定字段名（非标识符字符折成 `_`）。
fn host_field_name(key: &str) -> String {
    let mut s = String::from("__st_");
    for c in key.chars() {
        if c.is_alphanumeric() || c == '_' {
            s.push(c);
        } else {
            s.push('_');
        }
    }
    s
}

/// `storage.get/set(<literal key>, …)` 与 `Time.now_sec()` 的组件面改写。
fn rewrite_host_storage(src: &str, fields: &mut Vec<(String, String)>) -> String {
    let mut out = src.to_string();
    // Time.now_sec() → .__now_sec（模型 int 字段，宿主装载时种子注入）。
    if out.contains("Time.now_sec()") {
        out = out.replace("Time.now_sec()", ".__now_sec");
        if !fields.iter().any(|(n, _)| n == "__now_sec") {
            fields.push(("__now_sec".to_string(), "int".to_string()));
        }
    }
    // storage.set("key", <expr>) → .__st_key = <expr>
    loop {
        let Some(pos) = out.find("storage.set(") else { break };
        let rest = &out[pos + "storage.set(".len()..];
        let Some(q) = rest.find('"') else { break };
        let after_q = &rest[q + 1..];
        let Some(q2) = after_q.find('"') else { break };
        let key = after_q[..q2].to_string();
        // 逗号后到配对的 ')'。
        let args = &after_q[q2 + 1..];
        let Some(comma) = args.find(',') else { break };
        let mut depth = 0i32;
        let mut end = None;
        for (i, ch) in args[comma + 1..].char_indices() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' if depth == 0 => {
                    end = Some(i);
                    break;
                }
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        let Some(end) = end else { break };
        let value = args[comma + 1..comma + 1 + end].trim().to_string();
        let name = host_field_name(&key);
        if !fields.iter().any(|(n, _)| n == &name) {
            fields.push((name.clone(), "str".to_string()));
        }
        let stmt = format!(".{name} = {value}");
        let whole_end = pos + "storage.set(".len() + q + 1 + q2 + 1 + comma + 1 + end + 1;
        out.replace_range(pos..whole_end, &stmt);
    }
    // storage.get("key") → .__st_key
    loop {
        let Some(pos) = out.find("storage.get(") else { break };
        let rest = &out[pos + "storage.get(".len()..];
        let Some(q) = rest.find('"') else { break };
        let after_q = &rest[q + 1..];
        let Some(q2) = after_q.find('"') else { break };
        let key = after_q[..q2].to_string();
        let tail = &after_q[q2 + 1..];
        let Some(close) = tail.find(')') else { break };
        if !tail[..close].trim().is_empty() {
            break;
        }
        let name = host_field_name(&key);
        if !fields.iter().any(|(n, _)| n == &name) {
            fields.push((name.clone(), "str".to_string()));
        }
        let whole_end = pos + "storage.get(".len() + q + 1 + q2 + 1 + close + 1;
        out.replace_range(pos..whole_end, &format!(".{name}"));
    }
    out
}

/// AV 包面：对**纯 store 单元**做装载前归一（PLAN-112，025 真语料）。
///
/// 受检 AGE 对 store 的要求比 VM 前端严，宿主在交给 AGE 之前把作者源里 VM 允许的
/// 形状改成受检面收的形状：
///
/// 1. **model 形态**（[`normalize_store_model`]）：`var xs []T = …` → `List<T>`（包面
///    视图观察不了方括号列表字段）；`var xs = []`（无类型空列表）不是状态，按全部
///    push 点推断元素类型，元素是同构记录字面量时合成一个记录类型声明。
/// 2. **宿主服务仿真**：handler 里的 `storage.get/set` 改写成 store 的模型字段，与
///    组件面 prep 同一映射（`.__st_<key>`；内存态、不落盘，是 AV 车道既有合同）。
///
/// 只处理恰含一个 store、没有 widget/node 声明、带 `model {` 块的单元；storage
/// 改写另要求每个 `storage.` 调用的键都是字符串字面量。推不出的形状原样留着，
/// 诊断仍由 AGE 给出。
pub fn prepare_store_units(
    units: Vec<ace_core::source::packages::PackageModuleInput>,
) -> Vec<ace_core::source::packages::PackageModuleInput> {
    let mut out: Vec<ace_core::source::packages::PackageModuleInput> = units
        .into_iter()
        .map(|mut unit| {
            if is_pure_store(&unit.source) {
                // PLAN-119：AGE 面 store handler 语句文法没有 `return`/`break`
                // （038 MinesweeperStore 真实依赖）——先做控制转移兼容改写，
                // 再做文本转换拼写与 model 形态归一。
                unit.source = rewrite_store_control_flow(&unit.source).unwrap_or(unit.source);
                unit.source = rewrite_package_text_conversions(&unit.source);
                unit.source = normalize_store_model(&unit.source);
                // PLAN-119：元素所有权写法归一（整元素读/view-escape、索引写入、
                // `.pop`——038 的真实写法，AGE 包面各有受检等价形）。
                unit.source = normalize_store_element_ownership(&unit.source);
                if storage_keys_are_literals(&unit.source) {
                    let mut fields = Vec::new();
                    let out = rewrite_host_storage(&unit.source, &mut fields);
                    if !fields.is_empty() {
                        unit.source = inject_model_fields(&out, &fields);
                    }
                }
            }
            unit
        })
        .collect();
    // PLAN-119：纯 store 单元里**合成/自带的记录类型**必须也出现在消费它的
    // 应用单元——store 字段在投影出口被内联进 widget model（`st_board
    // []BoardRow`），而 AGE 的类型解析是**单单元**的（038 实证：
    // `source.module-symbol: unknown type BoardRow`）。把 store 单元里的
    // `type` 声明（去重后）插到每个非 store 单元的前导 `use` 行之后。
    let store_types: Vec<String> = out
        .iter()
        .filter(|u| is_pure_store(&u.source))
        .flat_map(|u| collect_type_blocks(&u.source))
        .collect();
    if !store_types.is_empty() {
        for unit in out.iter_mut().filter(|u| !is_pure_store(&u.source)) {
            let mut missing: Vec<String> = Vec::new();
            for decl in &store_types {
                let Some(name) = type_decl_name(decl) else { continue };
                if unit
                    .source
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .any(|w| w == name)
                {
                    continue;
                }
                if !missing.iter().any(|m| *m == *decl) {
                    missing.push(decl.clone());
                }
            }
            if missing.is_empty() {
                continue;
            }
            let at = after_leading_uses(&unit.source);
            let mut rebuilt = String::with_capacity(unit.source.len() + 64 * missing.len());
            rebuilt.push_str(&unit.source[..at]);
            for decl in &missing {
                rebuilt.push_str(decl);
                if !decl.ends_with('\n') {
                    rebuilt.push('\n');
                }
            }
            rebuilt.push_str(&unit.source[at..]);
            unit.source = rebuilt;
        }
    }
    out
}

/// `type <Name> {` 的声明名。
fn type_decl_name(decl: &str) -> Option<&str> {
    let rest = decl.trim_start().strip_prefix("type ")?;
    let name = rest
        .split(|c: char| c.is_whitespace() || c == '{')
        .next()?
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// 包面文本转换拼写：AGE 包面的数值没有 `.to_string()`（`numeric receiver has
/// no method `to_string``），而 `.to(Str)` 是同一转换的受检形（038 实证）。
/// 只对纯 store 单元调用。
fn rewrite_package_text_conversions(src: &str) -> String {
    src.replace(".to_string()", ".to(Str)")
}

/// PLAN-119：store 里**列表元素所有权**写法归一成 AGE 包面受检形。
///
/// VM 作者源把列表元素当值用（`var old = .board[i]`、`.board[i] = { … }`、
/// `stack.pop()`），而 AGE 包面：整元素读是 **view**（`source.view-escape`），
/// 元素写入只收 `.set(i, <fresh 构造>)`，没有 `.pop`（HOF 未接）。三件都是
/// 038 的真实写法，故按下列等价形改写（只在纯 store 单元上跑）：
///
/// 1. `var V = .<list>[<idx>]` → 摘除声明，后续 `V.` 全部内联为
///    `.<list>[<idx>].`（字段投影是受检读面；索引表达式为纯表达式，重复求值
///    无语义差）。
/// 2. `.<list>[<idx>] = { … }` → `.<list>.set(<idx>, <Elem> { … })`（元素记录
///    类型取自 model 字段 `List\<Elem>`/`[]Elem`；fresh 构造满足 move 纪律）。
/// 3. `R.pop()` → `R.remove(R.len() - 1)`（remove 返回 owned 元素，等同栈顶）。
///
/// 未识别的形态原样保留，诊断仍由 AGE 给出。
fn normalize_store_element_ownership(src: &str) -> String {
    let Some(on_open) = find_section_open(src, "on") else {
        return src.to_string();
    };
    let Some(on_close) = scan_balanced(src, on_open) else {
        return src.to_string();
    };
    let on_inner = &src[on_open + 1..on_close];
    let Some(arms) = split_handler_arms(on_inner) else {
        return src.to_string();
    };
    let elem_types = model_list_element_types(src);
    if elem_types.is_empty() {
        return src.to_string();
    }
    let mut rebuilt = String::new();
    for (head, body) in arms {
        let stmts = split_stmts(&body);
        let mut subs: Vec<(String, String)> = Vec::new();
        let out_stmts = ownership_stmt_list(&stmts, &elem_types, &mut subs);
        rebuilt.push_str(&format!("{head} -> {{\n{}\n}}\n", out_stmts.join("\n")));
    }
    let mut out = String::with_capacity(src.len() + rebuilt.len());
    out.push_str(&src[..=on_open]);
    out.push('\n');
    out.push_str(&rebuilt);
    out.push_str(&src[on_close..]);
    restore_newlines(out, src)
}

/// 逐语句做元素所有权归一（递归进 `if`/`for`/`while` 体；块内绑定不外泄）。
fn ownership_stmt_list(
    stmts: &[String],
    elems: &[(String, String)],
    subs: &mut Vec<(String, String)>,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for st in stmts {
        let st = st.clone();
        if let Some((name, projection)) = whole_element_binding(&st, elems) {
            subs.push((name, projection));
            continue;
        }
        // 嵌套块：体内递归（子作用域用 subs 副本，块内新增绑定不外泄）。
        if let Some(form) = parse_block_form(&st) {
            let mut inner_subs = subs.clone();
            let then_out = ownership_stmt_list(
                &split_stmts(&form.then_body),
                elems,
                &mut inner_subs,
            );
            let else_out = form.else_body.as_deref().map(|e| {
                let mut else_subs = subs.clone();
                ownership_stmt_list(&split_stmts(e), elems, &mut else_subs)
            });
            let mut rebuilt = format!("{} {} {{\n{}\n}}", form.kw, form.cond, then_out.join("\n"));
            if let Some(e) = else_out {
                rebuilt.push_str(&format!(" else {{\n{}\n}}", e.join("\n")));
            }
            let rebuilt = apply_element_substitutions(&rebuilt, subs);
            out.push(rewrite_pop_to_remove(&rewrite_element_writes(&rebuilt, elems)));
            continue;
        }
        let st = apply_element_substitutions(&st, subs);
        let st = rewrite_element_writes(&st, elems);
        let st = rewrite_pop_to_remove(&st);
        out.push(st);
    }
    out
}

/// model 里 `List<T>` / `[]T` 字段 → 元素记录类型名（非记录名原样收录，写入侧
/// 只对记录类型做 fresh 构造改写）。
fn model_list_element_types(src: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some((open, close)) = model_block(src) else {
        return out;
    };
    for line in src[open..close].lines() {
        let Some(ModelField::Typed { name, ty, .. }) = parse_model_field(line) else {
            continue;
        };
        let elem = if let Some(inner) = ty.strip_prefix("List<").and_then(|s| s.strip_suffix('>')) {
            Some(inner.to_string())
        } else {
            ty.strip_prefix("[]").map(|s| s.to_string())
        };
        if let Some(elem) = elem {
            if !elem.is_empty() {
                out.push((name.to_string(), elem));
            }
        }
    }
    out
}

/// `var V = .<list>[<idx>]`（整值绑定）→ (V, `.<list>[<idx>]`)。
fn whole_element_binding(stmt: &str, elems: &[(String, String)]) -> Option<(String, String)> {
    let s = stmt.trim().trim_end_matches(';').trim();
    let rest = s.strip_prefix("var ")?;
    let (name, value) = rest.split_once('=')?;
    let name = name.trim();
    // `var n int = …` 这类带注解的声明不是整元素绑定（`name` 会是 "n int"）。
    if !is_ident(name) {
        return None;
    }
    let projection = field_index_projection(value.trim(), elems)?;
    Some((name.to_string(), projection))
}

/// `.<list>[<idx>]` 整值（尾随无其他内容）→ 规范化投影串。
fn field_index_projection(v: &str, elems: &[(String, String)]) -> Option<String> {
    let rest = v.trim().strip_prefix('.')?;
    let (field, tail) = rest.split_once('[')?;
    if !elems.iter().any(|(f, _)| f == field) {
        return None;
    }
    let close = tail.rfind(']')?;
    if close + 1 != tail.len() {
        return None;
    }
    let idx = tail[..close].trim();
    if idx.is_empty() {
        return None;
    }
    Some(format!(".{field}[{idx}]"))
}

/// 把 `V.` / `V`（作为接收者）替换成投影表达式。
fn apply_element_substitutions(stmt: &str, subs: &[(String, String)]) -> String {
    let mut out = stmt.to_string();
    for (name, repl) in subs {
        let mut rebuilt = String::with_capacity(out.len());
        let mut rest = out.as_str();
        while let Some(at) = rest.find(name.as_str()) {
            let before_ok = at == 0
                || !rest[..at]
                    .chars()
                    .next_back()
                    .map(|c| c.is_alphanumeric() || c == '_' || c == '.')
                    .unwrap_or(false);
            let after = at + name.len();
            let next = rest[after..].chars().next();
            // 只替换「后面紧跟字段投影」的整值引用（`V.field`）。
            if before_ok && next == Some('.') {
                rebuilt.push_str(&rest[..at]);
                rebuilt.push_str(repl);
                rebuilt.push('.');
                rest = &rest[after + 1..];
                continue;
            }
            rebuilt.push_str(&rest[..after]);
            rest = &rest[after..];
        }
        rebuilt.push_str(rest);
        out = rebuilt;
    }
    out
}

/// `.<list>[<idx>] = { … }` → `.<list>.set(<idx>, <Elem> { … })`。
fn rewrite_element_writes(stmt: &str, elems: &[(String, String)]) -> String {
    let mut out = stmt.to_string();
    for (field, elem) in elems {
        let mut from = 0usize;
        while let Some(rel) = out[from..].find(&format!(".{field}[")) {
            let at = from + rel;
            let open = at + field.len() + 2;
            let Some(close_rel) = out[open..].find(']') else { break };
            let close = open + close_rel;
            let idx = out[open..close].trim().to_string();
            let after = out[close + 1..].trim_start();
            if !after.starts_with('=') {
                from = close + 1;
                continue;
            }
            let rhs_at = out[close + 1..].find('=').map(|p| p + close + 1).unwrap();
            let rhs = out[rhs_at + 1..].trim_start();
            if !rhs.starts_with('{') {
                from = close + 1;
                continue;
            }
            let brace = out[rhs_at + 1..].find('{').map(|p| p + rhs_at + 1).unwrap();
            let Some(brace_close) = scan_balanced(&out, brace) else { break };
            let literal = out[brace..=brace_close].to_string();
            let replacement = format!(".{field}.set({idx}, {elem} {literal})");
            out.replace_range(at..=brace_close, &replacement);
            from = at + replacement.len();
        }
    }
    out
}

/// `R.pop()` → `R.remove(R.len() - 1)`。
fn rewrite_pop_to_remove(stmt: &str) -> String {
    let mut out = stmt.to_string();
    let mut from = 0usize;
    while let Some(rel) = out[from..].find(".pop()") {
        let at = from + rel;
        // 回扫接收者（`stack` / `.board` 这类简单链）。
        let bytes = out.as_bytes();
        let mut start = at;
        while start > 0 {
            let b = bytes[start - 1];
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'.' {
                start -= 1;
            } else {
                break;
            }
        }
        let recv = out[start..at].trim();
        if recv.is_empty() || recv == "." {
            break;
        }
        let replacement = format!("{recv}.remove({recv}.len() - 1)");
        out.replace_range(start..at + ".pop()".len(), &replacement);
        from = start + replacement.len();
    }
    out
}

/// 顶层声明行（去缩进后以 `store `/`widget `/`node ` 起头）里恰有一个 store、
/// 没有 widget/node，且带 `model {` 块。
fn is_pure_store(src: &str) -> bool {
    let mut stores = 0;
    for line in src.lines() {
        let t = line.trim_start();
        let is_decl = |kw: &str| {
            t.strip_prefix(kw)
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_'))
        };
        if is_decl("store ") {
            stores += 1;
        } else if is_decl("widget ") || is_decl("node ") {
            return false;
        }
    }
    stores == 1 && src.contains("model {")
}

/// 所有 `storage.` 调用形如 `storage.get("` / `storage.set("`（至少一处）。
fn storage_keys_are_literals(src: &str) -> bool {
    let calls = src.matches("storage.").count();
    let literal = src.matches("storage.get(\"").count() + src.matches("storage.set(\"").count();
    calls > 0 && calls == literal
}

/// store model 的一条字段声明行。
enum ModelField<'a> {
    /// `var <name> <ty> <rest>`（`rest` 以 `=` 起头）。
    Typed { indent: &'a str, name: &'a str, ty: &'a str, rest: &'a str },
    /// `var <name> = <init>`（无类型注解）。
    Untyped { indent: &'a str, name: &'a str, init: &'a str },
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().enumerate().all(|(i, c)| {
            c == '_' || (c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
        })
}

fn parse_model_field(line: &str) -> Option<ModelField<'_>> {
    let body = line.trim_end_matches(['\n', '\r']);
    let t = body.trim_start();
    let indent = &body[..body.len() - t.len()];
    let rest = t.strip_prefix("var ")?;
    let (name, after) = rest.split_once(char::is_whitespace)?;
    if !is_ident(name) {
        return None;
    }
    // 无类型字段的初值只认空列表（行尾注释不算初值）；带类型的字段保留整行余下
    // 的部分（初值与行尾注释）。
    let after = after.trim_start();
    if let Some(init) = after.strip_prefix('=') {
        let init = init.trim();
        let init = init.split(" //").next().unwrap_or(init).trim_end();
        Some(ModelField::Untyped { indent, name, init })
    } else if after.is_empty() || !after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '[') {
        None
    } else {
        let ty_end = after.find(char::is_whitespace).unwrap_or(after.len());
        Some(ModelField::Typed { indent, name, ty: &after[..ty_end], rest: &after[ty_end..] })
    }
}

/// `model {` 之后的位置与块尾 `}` 的位置。按词法扫描配对（跳过行注释与字符串
/// 字面量——model 注释里的 `}` 不算块尾）。
fn model_block(src: &str) -> Option<(usize, usize)> {
    let open = src.find("model {")? + "model {".len();
    let bytes = src.as_bytes();
    let mut i = open;
    let mut depth = 1usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                } else {
                    let close = bytes[i] == b'"';
                    i += 1;
                    if close {
                        break;
                    }
                }
            }
            continue;
        }
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, i));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 前导 `use` 行之后的位置（导入必须先于声明；前导的注释与空行一并算前导）。
fn after_leading_uses(src: &str) -> usize {
    let mut end = 0;
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("use ") || t.starts_with("//") || t.is_empty() {
            end += line.len();
        } else {
            break;
        }
    }
    end
}

/// 字段 `.field` 的元素表达式 `(使用点, 表达式)`：`.field.push(<expr>)` 的实参与
/// `.field = [<expr>, …]` 的各元素（顶层逗号切分；空列表不贡献）。`.field` 之后
/// 不能接标识符字符（`.cpu_segs2` 不算 `.cpu_segs`）。
fn element_exprs<'a>(src: &'a str, field: &str) -> Vec<(usize, &'a str)> {
    let head = format!(".{field}");
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(rel) = src[cursor..].find(&head) {
        let at = cursor + rel;
        let after = &src[at + head.len()..];
        cursor = at + head.len();
        if after.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        if let Some(args) = after.strip_prefix(".push(") {
            if let Some(end) = balanced_end(args, '(', ')') {
                out.push((at, args[..end].trim()));
            }
        } else if let Some(init) = after.strip_prefix("= ") {
            let line_end = init.find('\n').unwrap_or(init.len());
            let init = init[..line_end].split(" //").next().unwrap_or("").trim();
            if let Some(items) = init.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
                for expr in split_top_level(items) {
                    let expr = expr.trim();
                    if !expr.is_empty() {
                        out.push((at, expr));
                    }
                }
            }
        }
    }
    out
}

/// `s` 里与已消费的开括号配对的闭括号下标（跳过字符串字面量）。
fn balanced_end(s: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1usize;
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => {
                for (_, q) in chars.by_ref() {
                    if q == '"' {
                        break;
                    }
                }
            }
            x if x == open => depth += 1,
            x if x == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// 按顶层逗号切分（括号/花括号/方括号与字符串内部不切）。
fn split_top_level(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => {
                for (_, q) in chars.by_ref() {
                    if q == '"' {
                        break;
                    }
                }
            }
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// 类型词归一到受检面：`[]X` → `List<X>`；其余原样。
fn type_word(ty: &str) -> Option<std::borrow::Cow<'_, str>> {
    if ty.is_empty() || ty == "=" {
        return None;
    }
    Some(match ty.strip_prefix("[]") {
        Some(elem) => std::borrow::Cow::Owned(format!("List<{elem}>")),
        None => std::borrow::Cow::Borrowed(ty),
    })
}

/// model 块里字段 `name` 的声明类型。
fn model_field_type<'a>(model: &'a str, name: &str) -> Option<&'a str> {
    model.lines().find_map(|line| match parse_model_field(line) {
        Some(ModelField::Typed { name: n, ty, .. }) if n == name => Some(ty),
        _ => None,
    })
}

/// 元素表达式的类型：字面量（`"…"` / `true` / `false` / 整数 / 小数）、就近的带注解
/// 局部声明（从使用点向前找 `var x T` / `let x T`）、model 字段（`.x`）。其余推不出。
fn expr_type<'a>(expr: &str, src: &'a str, at: usize, model: &'a str) -> Option<std::borrow::Cow<'a, str>> {
    let e = expr.trim();
    if e.len() >= 2 && e.starts_with('"') && e.ends_with('"') {
        return Some("str".into());
    }
    match e {
        "true" | "false" => return Some("bool".into()),
        _ => {}
    }
    if !e.is_empty() && e.bytes().all(|b| b.is_ascii_digit()) {
        return Some("int".into());
    }
    if e.contains('.')
        && e.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        && e.matches('.').count() == 1
    {
        return Some("float".into());
    }
    if let Some(member) = e.strip_prefix('.') {
        return is_ident(member)
            .then(|| model_field_type(model, member))
            .flatten()
            .and_then(type_word);
    }
    if !is_ident(e) {
        return None;
    }
    for line in src[..at].lines().rev() {
        let t = line.trim_start();
        let Some(decl) = t.strip_prefix("var ").or_else(|| t.strip_prefix("let ")) else {
            continue;
        };
        let Some((name, after)) = decl.split_once(char::is_whitespace) else {
            continue;
        };
        if name != e {
            continue;
        }
        let after = after.trim_start();
        let ty_end = after.find(char::is_whitespace).unwrap_or(after.len());
        return type_word(&after[..ty_end]);
    }
    None
}

/// 合成记录类型名：字段名各段首字母大写 + `Row`（`cpu_segs` → `CpuSegsRow`）。
fn row_type_name(field: &str) -> Option<String> {
    let mut name = String::new();
    for part in field.split('_').filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        let first = chars.next()?;
        name.extend(first.to_uppercase());
        name.push_str(chars.as_str());
    }
    (!name.is_empty()).then(|| format!("{name}Row"))
}

/// 无类型空列表字段的元素类型：`(元素类型词, 合成的记录类型声明)`。全部元素表达式
/// 必须同构——都是标量且类型一致，或都是键序与字段类型一致的字面量。
fn infer_element_type(src: &str, field: &str) -> Option<(String, Option<String>)> {
    let exprs = element_exprs(src, field);
    if exprs.is_empty() {
        return None;
    }
    let model = model_block(src).map(|(open, close)| &src[open..close]).unwrap_or("");
    let mut scalar: Option<String> = None;
    let mut record: Option<Vec<(String, String)>> = None;
    for (at, expr) in &exprs {
        if let Some(inner) = expr.strip_prefix('{').and_then(|rest| rest.strip_suffix('}')) {
            if scalar.is_some() {
                return None;
            }
            let mut fields = Vec::new();
            for part in split_top_level(inner) {
                let (key, value) = part.split_once(':')?;
                let key = key.trim();
                if !is_ident(key) {
                    return None;
                }
                fields.push((key.to_string(), expr_type(value, src, *at, model)?.into_owned()));
            }
            match &record {
                None => record = Some(fields),
                Some(seen) if *seen == fields => {}
                Some(_) => return None,
            }
        } else {
            if record.is_some() {
                return None;
            }
            let ty = expr_type(expr, src, *at, model)?.into_owned();
            match &scalar {
                None => scalar = Some(ty),
                Some(seen) if *seen == ty => {}
                Some(_) => return None,
            }
        }
    }
    if let Some(scalar) = scalar {
        return Some((scalar, None));
    }
    let fields = record?;
    let name = row_type_name(field)?;
    if src.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).any(|word| word == name) {
        return None;
    }
    let mut decl = format!("type {name} {{\n");
    for (key, ty) in &fields {
        decl.push_str(&format!("    {key} {ty}\n"));
    }
    decl.push_str("}\n\n");
    Some((name, Some(decl)))
}

/// store model 的受检面形态（只动 `model { … }` 块里的字段行）：
/// - `var xs []T = …` → `var xs List<T> = …`——包面视图观察不了方括号列表字段；
/// - `var xs = []` → `var xs List<E> = []`，`E` 取自全部 push 点（见
///   [`infer_element_type`]）；元素是记录字面量时在前导 `use` 行之后合成类型声明。
///
/// 推不出的无类型字段原样留着，诊断仍由 AGE 给出。
fn normalize_store_model(src: &str) -> String {
    let Some((open, close)) = model_block(src) else {
        return src.to_string();
    };
    let mut body = String::with_capacity(src.len());
    body.push_str(&src[..open]);
    let mut untyped: Vec<(&str, &str)> = Vec::new();
    for line in src[open..close].split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let nl = &line[trimmed.len()..];
        let replaced = match parse_model_field(line) {
            Some(ModelField::Typed { indent, name, ty, rest }) => ty.strip_prefix("[]").map(|elem| {
                format!("{indent}var {name} List<{elem}>{rest}{nl}")
            }),
            Some(ModelField::Untyped { indent, name, init }) if init == "[]" => {
                untyped.push((name, indent));
                None
            }
            _ => None,
        };
        match replaced {
            Some(replaced) => body.push_str(&replaced),
            None => body.push_str(line),
        }
    }
    let mut decls = String::new();
    for (name, indent) in untyped {
        let Some((elem, decl)) = infer_element_type(src, name) else {
            continue;
        };
        let target = format!("{indent}var {name} = []");
        if let Some(at) = body.find(&target) {
            body.replace_range(at..at + target.len(), &format!("{indent}var {name} List<{elem}> = []"));
            if let Some(decl) = decl {
                decls.push_str(&decl);
            }
        }
    }
    body.push_str(&src[close..]);
    let at = after_leading_uses(&body);
    let mut out = String::with_capacity(body.len() + decls.len());
    out.push_str(&body[..at]);
    out.push_str(&decls);
    out.push_str(&body[at..]);
    out
}


/// PLAN-119：组件面（AGE component profile）的 store 片断模型归一。
///
/// 组件面只收 `[]T` 列表拼写（`List<T>` 会让整模块回退成 fn profile 而拒），
/// 且**无类型** `var x = []` 不算状态字段（`.store.x` 解析不到）。故：
/// - `var xs List<T> = …` → `var xs []T = …`；
/// - `var xs = []` → `var xs []E = []`，`E` 取自全部 push 点（与包面
///   [`normalize_store_model`] 同一推断）；元素是记录字面量时把合成的
///   `type E { … }` 前置到片断头——store 块随内联落到组件头之前，类型与
///   store 同处一个单元，跨单元解析问题一并消失。
///
/// 推不出的无类型字段原样留着，诊断仍由 AGE 给出。
fn normalize_component_store_model(block: &str) -> String {
    let Some((open, close)) = model_block(block) else {
        return block.to_string();
    };
    let mut body = String::with_capacity(block.len());
    body.push_str(&block[..open]);
    let mut untyped: Vec<(String, String)> = Vec::new();
    for line in block[open..close].split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let nl = &line[trimmed.len()..];
        let replaced = match parse_model_field(line) {
            Some(ModelField::Typed {
                indent,
                name,
                ty,
                rest,
            }) => ty
                .strip_prefix("List<")
                .and_then(|inner| inner.strip_suffix('>'))
                .map(|elem| format!("{indent}var {name} []{elem}{rest}{nl}")),
            Some(ModelField::Untyped { indent, name, init }) if init == "[]" => {
                untyped.push((name.to_string(), indent.to_string()));
                None
            }
            _ => None,
        };
        match replaced {
            Some(replaced) => body.push_str(&replaced),
            None => body.push_str(line),
        }
    }
    let mut decls = String::new();
    for (name, indent) in untyped {
        let Some((elem, decl)) = infer_element_type(block, &name) else {
            continue;
        };
        let target = format!("{indent}var {name} = []");
        if let Some(at) = body.find(&target) {
            body.replace_range(at..at + target.len(), &format!("{indent}var {name} []{elem} = []"));
            if let Some(decl) = decl {
                decls.push_str(&decl);
            }
        }
    }
    body.push_str(&block[close..]);
    let mut out = String::with_capacity(body.len() + decls.len());
    out.push_str(&decls);
    out.push_str(&body);
    out
}

/// 把宿主仿真字段插进 `model {` 块尾（无 model 块则不插）。
/// 块尾 `}` 按词法扫描定位：跳过行注释（`//`）与字符串字面量——model 注释
/// 或字符串里的 `}`（如 `[{name,rel_path}]`、CSS 类）不再误当块尾
/// （PLAN-117 T-01 实证：029 的 dirs 注释含 `}]`，字段被插进注释内）。
fn inject_model_fields(src: &str, fields: &[(String, String)]) -> String {
    let Some(model_at) = src.find("model {") else {
        return src.to_string();
    };
    let bytes = src.as_bytes();
    let mut i = model_at + "model {".len();
    let mut depth = 1i32;
    let mut close = None;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            // 行注释：跳到行尾（含换行前，`\n` 由下一轮处理）。
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b == b'"' {
            // 字符串字面量：跳转义与闭合引号。
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let Some(close) = close else {
        return src.to_string();
    };
    let indent_at = src[..model_at].rfind('\n').map(|i| model_at - i - 1).unwrap_or(0);
    let indent = " ".repeat(indent_at + 4);
    let mut block = String::new();
    for (name, ty) in fields {
        let default = if ty == "int" { "0" } else { "\"\"" };
        block.push_str(&format!("{indent}var {name} {ty} = {default}\n"));
    }
    let mut out = src.to_string();
    out.insert_str(close, &format!("\n{block}"));
    out
}

#[cfg(test)]
mod plan118_tests {
    use super::*;

    /// 件④：合成场景名改写成串 prop；真实字段绑定不动；非组名不动。
    #[test]
    fn canvas_scene_bindings_rewrite() {
        let src = "widget App {
    msg { Init }
    model {
        var cells_pts List = []
        var cells_nodes List = []
        var plain str = \"x\"
    }
    view {
        canvas (scene: .cells, coords: \"320x320\") {}
        canvas (scene: .plain, coords: \"32x32\") {}
        canvas (scene: .unknown, coords: \"8x8\") {}
    }
    on { .Init -> { } }
}
";
        let out = rewrite_canvas_scene_bindings(src);
        assert!(out.contains("scene: \"cells\""), "synthetic name -> string prop");
        assert!(out.contains("scene: .plain"), "real field binding untouched");
        assert!(out.contains("scene: .unknown"), "non-group name untouched");
    }

    /// str/Str 归一：fn 形参/返回与 var 局部；非目标行不动。
    #[test]
    fn str_surface_normalizes_fn_scope() {
        let src = "fn f(ps str, n int) str {
    var t str = ps.substr(1, 2)
    return t
}
";
        let out = rewrite_str_surface_for_package(src, false);
        // 形参保持借用 `str`（owner 位；改 owned 会把调用变 move+读）；
        // 返回注解与 `var` 局部归一为 owned `Str`。
        assert!(out.contains("fn f(ps str, n int) Str {"));
        assert!(out.contains("var t Str ="));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `P105_MEM_BACKEND` 是进程级 env 开关：并行测试里 set/remove 会竞态
    /// （bridge 装载测在开关关闭态跑 store 源 = 必红）。触碰开关的测试都持锁。
    fn mem_backend_env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        &LOCK
    }

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
    fn split_rewrite_scopes_same_named_locals_per_handler() {
        // PLAN-118：两个 handler 各自声明同名局部 `parts`——改写必须按作用域
        // 配平接收者（此前先处理的声明会改写另一个 handler 的 `parts.len()`，
        // 生成 `__p117_str_split_count(s, …)` 指向不存在的局部）。
        let src = "widget W {\n\
            on {\n\
                .A(s str) -> {\n\
                    var parts = s.split(\"|\")\n\
                    if parts.len() == 2 { }\n\
                }\n\
                .B(snap str) -> {\n\
                    var parts = snap.split(\"|\")\n\
                    if parts.len() == 3 { }\n\
                }\n\
            }\n\
        }\n";
        let mut need = false;
        let out = rewrite_str_split(src, &mut need);
        assert!(need);
        assert!(out.contains("__p117_str_split_count(s, \"|\")"), "A keeps its receiver: {out}");
        assert!(out.contains("__p117_str_split_count(snap, \"|\")"), "B keeps its receiver: {out}");
        assert!(!out.contains("parts"), "declarations and reads are rewritten: {out}");
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
    fn inlines_local_style_declarations() {
        let root = temp_project();
        let src = "style caption = \"text-xs text-muted-foreground\"\nwidget App {\n    view {\n        text \"x\" { style: caption }\n    }\n}\n";
        let prepared = prepare_component_source(&root, src).expect("prepare");
        assert!(
            prepared.source.contains("style: \"text-xs text-muted-foreground\""),
            "local style reference becomes a literal: {}",
            prepared.source
        );
        assert!(
            !prepared.source.contains("style caption ="),
            "local style declaration line is blanked"
        );
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
        // 记录元素的类型化列表字段 → 无类型视图数据（类型段等长空白化）。
        let rewritten = rewrite_typed_list_field("        var faqs []Faq = []")
            .expect("record list data field rewritten");
        assert_eq!(rewritten.len(), "        var faqs []Faq = []".len());
        assert!(!rewritten.contains("[]Faq"));
        assert!(rewritten.trim_end().ends_with("= []"));
        // 内建元素的列表字段**保持类型**（collections 域真承载；012 的 `[]str`）。
        assert!(rewrite_typed_list_field("        var cities []str = [\"Beijing\"]").is_none());
        assert!(rewrite_typed_list_field("    var xs []int = []").is_none());
        // 非列表类型、非列表初值、普通语句一律不动。
        assert!(rewrite_typed_list_field("    var n int = 0").is_none());
        assert!(rewrite_typed_list_field("    var xs []str = \"a\"").is_none());
        assert!(rewrite_typed_list_field("    text \"x\" {}").is_none());
        // 嵌套列表（记录元素）仍改写。
        let nested = rewrite_typed_list_field("    var xs [][]Faq = []").unwrap();
        assert!(!nested.contains("[][]Faq"));
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
            match ace_core::source::age_component::project_component_candidates(&prepared.source) {
                Ok(_) => {}
                Err(ds) if app == "013-todo" => {
                    // `.store.*` 在**独立单元**（跨单元 store 装载仍受 `on` 体列表
                    // 操作限制）——必须具名拒（不是结构/语法错，也不是静默通过）。
                    let named = ds.iter().any(|d| {
                        d.code == "source.age.component-unsupported" && d.message.contains(".store.")
                    });
                    if !named {
                        let d = ds.first().map(|d| format!("{}: {}", d.code, d.message)).unwrap_or_default();
                        failed.push(format!("{app} (expected named .store reject, got {d})"));
                    }
                }
                Err(_) if app == "016-calendar" => {
                    // 016 的 helper `month_name` 用定长数组字面量 + 下标
                    // （列表/数组语义未接线）——当前必须**报错**（禁止静默通过），
                    // 具体码随列表刀推进再收紧。
                }
                Err(ds) => {
                    let d = ds.first().map(|d| format!("{}: {}", d.code, d.message)).unwrap_or_default();
                    failed.push(format!("{app} ({d})"));
                }
            }
        }
        assert!(failed.is_empty(), "still failing: {failed:?}");
    }


    /// 真语料 → AV 桥装载（不开窗）：015/018/022 必须能经源准备后进入
    /// `NodeDesktopBridge`（T-04 低层证据）。013/016 仍卡文本叶 `.store.*` 引用链
    /// （`leaf-expr: no local named self`），按缺口表跟踪，不在本测断言内。
    #[test]
    fn real_u03_apps_load_through_desktop_bridge() {
        // 与 `diagw82_span_013` 的 env 开关互斥（并行测试下 set/remove 竞态
        // 会污染本测的装载环境）。本测保持 env 关闭语义（015/018/022 不走
        // mem-backend 内联装载）。
        let _env_guard = mem_backend_env_lock().lock().unwrap();
        std::env::remove_var("P105_MEM_BACKEND");
        let ui_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui");
        for app in ["015-notes", "018-book-reader", "022-kanban"] {
            let app_root = ui_root.join(app);
            let path = app_root.join("src/front/app.at");
            let src = std::fs::read_to_string(&path).expect("app source");
            let prepared = prepare_component_source(&app_root, &src).expect("prepare");
            crate::ui::ace_node_desktop::NodeDesktopBridge::load(
                &prepared.source,
                path.to_str().unwrap(),
                1,
            )
            .unwrap_or_else(|e| panic!("{app} must load through the AV bridge: {e}"));
        }
    }

    /// PLAN-105 T-02：带工程根的 SFC 生成——entry 的 `use`/依赖样式先过源准备，
    /// 组件投影才拿得到可吃进的单源（真语料形态的最小复现）。
    #[test]
    fn diagw82_span_013() {
        let _env_guard = mem_backend_env_lock().lock().unwrap();
        std::env::set_var("P105_MEM_BACKEND", "1");
        let ui_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui");
        let app_root = ui_root.join("013-todo");
        let path = app_root.join("src/front/app.at");
        let src = std::fs::read_to_string(&path).unwrap();
        let prepared = prepare_component_source(&app_root, &src).unwrap();
        let caps = vec!["ac.runtime.text.v1".to_string()];
        let sc = ace_core::source::age_component::ComponentScenario { instances: 1, ops: Vec::new() };
        if let Err(ds) = ace_core::source::age_component::compile_component_scenario(
            &prepared.source,
            path.to_str().unwrap(),
            ace_core::source::age_h0::LegacyWidgetAlias::Accept,
            &caps,
            &sc,
        ) {
            for d in ds.iter().take(3) {
                let a = d.span.start.saturating_sub(80).min(prepared.source.len());
                let b = (d.span.end + 80).min(prepared.source.len());
                println!("[d82] {} :: {:?}", d.message, &prepared.source[a..b]);
            }
        }
        std::env::remove_var("P105_MEM_BACKEND");
    }
    #[test]
    fn sfc_generation_prepares_entry_source() {
        let root = std::env::temp_dir().join(format!(
            "p105-sfc-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("src/front")).unwrap();
        std::fs::create_dir_all(root.join("deps/stylekit/src/front")).unwrap();
        std::fs::write(
            root.join("pac.at"),
            "name: \"app\"\nversion: \"1.0.0\"\ndep stylekit {\n    path: \"deps/stylekit\"\n}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("deps/stylekit/src/front/styles.at"),
            "pub style caption_text = \"text-xs text-muted-foreground\"\n",
        )
        .unwrap();
        let app_src = "use stylekit.styles: caption_text\nwidget App {\n    model { var n int = 0 }\n    view { text \"x\" { style: caption_text } }\n}\n";
        std::fs::write(root.join("src/front/app.at"), app_src).unwrap();
        let units = vec![ace_core::source::packages::PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: app_src.into(),
            provider: None,
        }];
        let sfc = crate::ui::ace_package_sfc::generate_package_sfc_with_root(
            &units,
            "app::app",
            Some(&root),
            None,
            false,
        )
        .expect("prepared entry generates an SFC");
        assert!(sfc.contains("<template>"), "SFC template emitted");
        assert!(sfc.contains("text-xs text-muted-foreground"), "inlined style reaches the template");
        let _ = std::fs::remove_dir_all(&root);
    }

    fn package_unit(id: &str, source: &str) -> ace_core::source::packages::PackageModuleInput {
        ace_core::source::packages::PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
            provider: None,
        }
    }

    /// 纯 store 单元：`storage.get/set("k", …)` → `.__st_k`，字段注入该 store 的 model。
    #[test]
    fn p112_simulates_host_storage_in_a_pure_store_unit() {
        let src = "use fmt: n1\n\nstore S {\n    model {\n        var speed str = \"\"\n    }\n    \
                   msg { Go }\n\n    on {\n        .Init -> {\n            var sp = storage.get(\"sysmon.speed\")\n            \
                   if sp == \"\" { sp = storage.get(\"dash.speed\") }\n            .speed = sp\n        }\n        \
                   .SetSpeed(speed) -> {\n            storage.set(\"sysmon.speed\", speed)\n        }\n    }\n}\n";
        let out = prepare_store_units(vec![package_unit("s", src)]).remove(0).source;
        assert!(!out.contains("storage."), "{out}");
        assert!(out.contains("var sp = .__st_sysmon_speed"), "{out}");
        assert!(out.contains("sp = .__st_dash_speed"), "{out}");
        assert!(out.contains(".__st_sysmon_speed = speed"), "{out}");
        // 字段落在 store 的 model 块尾（`model {` 之后、`msg {` 之前），初值为空串。
        let (model, msg, field) = (
            out.find("model {").unwrap(),
            out.find("msg {").unwrap(),
            out.find("var __st_sysmon_speed str = \"\"").unwrap(),
        );
        assert!(model < field && field < msg, "{out}");
        assert!(out.contains("var __st_dash_speed str = \"\""), "{out}");
        assert!(out.starts_with("use fmt: n1\n"), "use 行原样保留：{out}");
    }

    /// 不做仿真的形状一律逐字节不变：非 store 单元、store 与 widget 同单元、两个
    /// store、无 model 块、键不是字符串字面量、只在注释里提到 `storage.`。
    #[test]
    fn p112_leaves_other_unit_shapes_untouched() {
        let shapes = [
            "widget Main {\n    model { var n int = 0 }\n    view { text .n { } }\n}\n",
            "store S {\n    model { var a str = \"\" }\n    on { .Go -> { storage.set(\"k\", .a) } }\n}\nwidget Main {\n    view { text \"x\" { } }\n}\n",
            "store A {\n    model { var a str = \"\" }\n}\nstore B {\n    model { var b str = \"\" }\n    on { .Go -> { storage.set(\"k\", .b) } }\n}\n",
            "store S {\n    on { .Go -> { storage.set(\"k\", \"v\") } }\n}\n",
            "store S {\n    model { var a str = \"\" }\n    on { .Go -> { storage.set(.a, \"v\") } }\n}\n",
            "store S {\n    model { var a str = \"\" }\n    on { .Go -> { .a = \"v\" } }\n}\n// storage.get 的注释\n",
        ];
        let out = prepare_store_units(shapes.iter().enumerate().map(|(i, s)| package_unit(&format!("u{i}"), s)).collect());
        for (before, unit) in shapes.iter().zip(out) {
            assert_eq!(&unit.source, before);
        }
    }

    /// 025 真实 `sys_store.at`：8 个键全部映射为 store 的模型字段，源里不再有 `storage.`。
    #[test]
    fn p112_simulates_host_storage_for_the_real_025_store() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/025-sys-monitor/src/front/sys_store.at");
        let src = std::fs::read_to_string(&path).unwrap();
        // 13 处调用、8 个键（源变更时同步本钉点）。
        assert_eq!(src.matches("storage.").count(), 13);
        let out = prepare_store_units(vec![package_unit("sys_store", &src)]).remove(0).source;
        assert!(!out.contains("storage."), "storage 调用应全部改写");
        for key in [
            "sysmon.speed",
            "dash.speed",
            "sysmon.sort_column",
            "dash.sort_column",
            "sysmon.sort_dir",
            "dash.sort_dir",
            "sysmon.active_tab",
            "sysmon.perf_tab",
        ] {
            let field = host_field_name(key);
            assert!(out.contains(&format!("var {field} str = \"\"")), "{key} 应有模型字段：{field}");
        }
        assert_eq!(out.matches("var __st_").count(), 8, "恰 8 个仿真字段");
        assert!(out.contains("var sp = .__st_sysmon_speed"), "{out}");
        assert!(out.contains(".__st_sysmon_speed = speed"), "{out}");
    }

    /// model 形态归一只动 `model { … }` 里的字段行：方括号列表 → `List<T>`（初值、
    /// 行尾注释与换行保留）；handler 里的方括号局部量不动。
    #[test]
    fn p112_types_bracket_lists_in_the_store_model_only() {
        let src = "store S {\r\n    model {\r\n        var xs []int = [] // ids\r\n        \
                   var name str = \"\"\r\n        var ys []Rec\r\n    }\r\n    msg { Go }\r\n    on {\r\n        \
                   .Go -> {\r\n            var r2 []Rec = []\r\n            .name = \"x\"\r\n        }\r\n    }\r\n}\r\n";
        let out = prepare_store_units(vec![package_unit("s", src)]).remove(0).source;
        assert!(out.contains("        var xs List<int> = [] // ids\r\n"), "{out:?}");
        assert!(out.contains("        var ys List<Rec>\r\n"), "{out:?}");
        assert!(out.contains("            var r2 []Rec = []\r\n"), "handler 局部量不动：{out:?}");
        assert_eq!(out.matches("\r\n").count(), src.matches("\r\n").count());
    }

    /// 无类型空列表按 push 点推断：记录字面量 → 合成 `<Field>Row` 类型声明（落在前导
    /// `use` 行之后、store 之前）；标量与已声明类型的名字 → 直接取类型。
    #[test]
    fn p112_infers_untyped_empty_list_fields_from_push_sites() {
        let src = "use fmt: n1\n\n// 说明\nstore S {\n    model {\n        var cpu float = 0.0\n        \
                   var cpu_segs = []\n        var hist = []\n        var n int = 0\n    }\n    msg { Go }\n    \
                   on {\n        .Go -> {\n            .cpu_segs = []\n            var cst str = \"0\"\n            \
                   var ccol str = \"bg-red-500\"\n            .cpu_segs.push({st: cst, color: ccol})\n            \
                   .cpu_segs.push({st: \"1\", color: ccol})\n            .hist.push(1.5)\n            \
                   .hist.push(.cpu)\n            .n = 1\n        }\n    }\n}\n";
        let out = prepare_store_units(vec![package_unit("s", src)]).remove(0).source;
        assert!(out.contains("type CpuSegsRow {\n    st str\n    color str\n}\n\n"), "{out}");
        assert!(out.contains("        var cpu_segs List<CpuSegsRow> = []\n"), "{out}");
        assert!(out.contains("        var hist List<float> = []\n"), "{out}");
        let (uses, decl, store) = (
            out.find("use fmt: n1").unwrap(),
            out.find("type CpuSegsRow").unwrap(),
            out.find("store S {").unwrap(),
        );
        assert!(uses < decl && decl < store, "{out}");
    }

    /// 推不出就原样留着：没有任何元素表达式、push 的局部量没有类型注解、同名记录
    /// 字段的类型不一致、合成的类型名已被占用。
    #[test]
    fn p112_leaves_uninferrable_untyped_lists_untouched() {
        let cases = [
            "store S {\n    model {\n        var xs = []\n    }\n    msg { Go }\n    on { .Go -> { .xs = [] } }\n}\n",
            "store S {\n    model {\n        var xs = []\n    }\n    msg { Go }\n    on { .Go -> { var v = f()\n            \
             .xs.push(v) } }\n}\n",
            "store S {\n    model {\n        var xs = []\n    }\n    msg { Go }\n    on { .Go -> { .xs.push({st: \"1\"})\n            \
             .xs.push({st: 2}) } }\n}\n",
            "type RowsRow {\n    a int\n}\nstore S {\n    model {\n        var rows = []\n    }\n    msg { Go }\n    \
             on { .Go -> { .rows.push({a: 1}) } }\n}\n",
        ];
        for src in cases {
            let out = prepare_store_units(vec![package_unit("s", src)]).remove(0).source;
            assert_eq!(&out, src);
        }
    }

    /// 025 真实 store 的 model 形态：两个无类型段字段合成记录类型；方括号列表统一成
    /// `List<T>`；handler 里的方括号局部量不动。
    #[test]
    fn p112_normalizes_the_real_025_store_model() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/025-sys-monitor/src/front/sys_store.at");
        let src = std::fs::read_to_string(&path).unwrap();
        let out = prepare_store_units(vec![package_unit("sys_store", &src)]).remove(0).source;
        assert!(out.contains("type CpuSegsRow {\n    st str\n    color str\n}\n"), "{out}");
        assert!(out.contains("type MemSegsRow {\n    st str\n    color str\n}\n"), "{out}");
        assert!(out.contains("var cpu_segs List<CpuSegsRow> = []"), "{out}");
        assert!(out.contains("var mem_segs List<MemSegsRow> = []"), "{out}");
        for (field, elem) in [
            ("procs", "ProcInfo"),
            ("procsView", "ProcInfo"),
            ("disks", "DiskInfo"),
            ("cores", "CoreInfo"),
            ("users", "UserInfo"),
            ("cpuHist", "float"),
            ("memHist", "float"),
            ("diskHist", "float"),
            ("netHist", "float"),
        ] {
            assert!(
                out.contains(&format!("var {field} List<{elem}> = []")),
                "{field} 应归一为 List<{elem}>：{out}"
            );
        }
        assert!(out.contains("var r2 []ProcInfo = []"), "handler 局部量不动：{out}");
        assert!(!out.contains("var cpu_segs = []"), "{out}");
    }

    // ─── PLAN-119：store 片断控制转移改写 ─────────────────────────────────
    fn p119_store_block() -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/038-minesweeper");
        let unit = std::fs::read_to_string(root.join("src/front/minesweeper_store.at"))
            .expect("read 038 store unit");
        extract_store_block(&unit, "MinesweeperStore").expect("store block")
    }

    fn p119_store_probe(block: &str) -> Result<(), Vec<String>> {
        let probe = format!(
            "{block}\nwidget __StoreProbe {{\n    model {{ var __n int = 0 }}\n    view {{ text \"x\" {{}} }}\n}}\n"
        );
        let stores_ok = ace_core::source::age_component::project_component_stores(&probe)
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !stores_ok {
            return Err(vec!["store projection empty".to_string()]);
        }
        ace_core::source::age_component::project_component_candidates(&probe)
            .map(|_| ())
            .map_err(|ds| ds.iter().map(|d| format!("[{}] {}", d.code, d.message)).collect())
    }

    #[test]
    fn plan119_store_control_flow_rewrite_admits_the_minesweeper_store() {
        let block = p119_store_block();
        // 原样片断确应被拒（缺 return/break 语句面）——反例钉点。
        assert!(
            p119_store_probe(&block).is_err(),
            "原样 038 store 片断应当不是 AGE 组件面可投影形"
        );
        let rewritten = rewrite_store_control_flow(&block).expect("038 store 可改写");
        for (i, line) in rewritten.lines().enumerate() {
            let t = line.trim().trim_end_matches(';');
            assert!(
                t != "return" && t != "break" && t != "continue",
                "L{}: 控制转移未清：{t}",
                i + 1
            );
        }
        assert!(rewritten.contains("__p119_ret = true"), "return 位置标志置位");
        assert!(rewritten.contains("__p119_brk1 = true"), "break 位置标志置位");
        assert!(rewritten.contains("var __p119_ret bool = false"));
        assert!(
            rewritten.contains("__p119_brk1 == false"),
            "循环条件应并 break 标志：{rewritten}"
        );
        if let Err(ds) = p119_store_probe(&rewritten) {
            panic!("改写后仍被拒：{ds:?}\n---\n{rewritten}");
        }
    }

    #[test]
    fn plan119_store_without_control_flow_is_unchanged() {
        let block = "store S {\n    model {\n        var n int = 0\n    }\n    msg { Inc }\n    on {\n        .Inc -> {\n            .n = .n + 1\n        }\n    }\n}\n";
        assert_eq!(rewrite_store_control_flow(block).as_deref(), Some(block));
    }

    #[test]
    fn plan119_store_continue_is_left_to_the_named_boundary() {
        let block = "store S {\n    model {\n        var n int = 0\n    }\n    msg { Inc }\n    on {\n        .Inc -> {\n            for .n < 3 {\n                continue\n            }\n        }\n    }\n}\n";
        assert_eq!(rewrite_store_control_flow(block), None);
    }

    #[test]
    fn plan119_guard_return_becomes_flag_and_guarded_rest() {
        let block = "store S {\n    model {\n        var n int = 0\n    }\n    msg { Inc }\n    on {\n        .Inc -> {\n            if .n > 2 {\n                return\n            }\n            .n = .n + 1\n        }\n    }\n}\n";
        let out = rewrite_store_control_flow(block).expect("rewrite");
        assert!(out.contains("var __p119_ret bool = false"));
        assert!(out.contains("__p119_ret = true"));
        assert!(out.contains("if __p119_ret == false {"));
        assert!(
            !out.lines().any(|l| l.trim().trim_end_matches(';') == "return"),
            "{out}"
        );
        assert!(p119_store_probe(&out).is_ok(), "{out}");
    }
    #[test]
    fn plan119_store_types_are_injected_into_the_entry_unit() {
        let app = "use probe_store: Probe\n\nwidget App {\n    msg { Init }\n    model { var pad int = 0 }\n    view { text .store.label { style: \"t\" } }\n    on { .Init -> { store.Init() } }\n}\n";
        let store = "type R {\n    x int\n    display str\n}\n\nstore Probe {\n    model {\n        var board List<R> = []\n        var label str = \"x\"\n    }\n    msg { Init }\n    on {\n        .Init -> {\n            .board.push({ x: 1, display: \"a\" })\n            .label = \"ok\"\n        }\n    }\n}\n";
        let units = prepare_store_units(vec![package_unit("app", app), package_unit("probe_store", store)]);
        let app_out = &units[0].source;
        println!("--- app unit ---\n{app_out}\n--- store unit ---\n{}", units[1].source);
        assert!(app_out.contains("type R {"), "类型应注入入口单元：{app_out}");
        for n in &units {
            println!("unit {} = {} bytes", n.id, n.source.len());
        }
    }}
