//! design_tokens/theme_tree —— PLAN-007 树状主题表示、extend 合成、模式选择
//! 与类型化槽引用（试用约定 C-01..11；T-04）。
//!
//! 主题/token 是 Auto 模块中的树状数据（`pub theme <Name> = { ... }`，UI 域
//! 声明），普通 `use` 导入；本模块把它 lower 为 [`Tree`]，提供：
//!
//! - **extend 合成**（C-04）：左到右、唯一命名字段递归合并、标量后胜、
//!   数组整体替换、输入只读；
//! - **模式选择**（C-06/07）：`modes.light/dark` 只写公共差异；每次切模式
//!   从原始公共数据重建（不叠加上次解析结果）；
//! - **类型化槽引用**（C-05）：`tokens.radius.control` 点链保存为
//!   [`Tree::Ref`]，按当前主题+模式解析，不先复制值；
//! - **Schema 校验与可定位诊断**（C-02/03/11）：骨架键 tokens/styles/
//!   components/modes + app 命名空间；tokens.colors 公共键对账既有 31 键
//!   封闭词表；未知键/错型/未知引用路径给带路径的错误。
//!
//! 与 PLAN-601 的 pac `theme{}` 块（[`super::decl`]）正交：decl 面向既有
//! 31 键 CSS/VM 单源投影（不变），本模块是新增树状扩展面。

use super::registry::{self, TokenName};
use auto_parser_api::ast::Expr;
use std::collections::BTreeMap;

// ── 树值模型 ──────────────────────────────────────────────────────────

/// 主题树节点（lower 后的只读数据）。
#[derive(Debug, Clone, PartialEq)]
pub enum Tree {
    Str(String),
    Num(f64),
    Bool(bool),
    /// 类型化槽引用（C-05）：点链/裸名，按当前主题+模式在消费点解析。
    Ref(Vec<String>),
    /// `extend(base, patch, ...)` 组合调用——build 期展开为 Object。
    Extend(Vec<Tree>),
    Object(Vec<(String, Tree)>),
    Array(Vec<Tree>),
}

impl Tree {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Tree::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Tree> {
        match self {
            Tree::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// 可定位诊断（C-11）：`path` = 主题树内的点路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeError {
    pub path: String,
    pub msg: String,
}

impl ThemeError {
    fn at(path: impl Into<String>, msg: impl Into<String>) -> Self {
        ThemeError {
            path: path.into(),
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}（主题路径 `{}`）", self.msg, self.path)
    }
}

impl std::error::Error for ThemeError {}

// ── Expr lower ────────────────────────────────────────────────────────

/// UI 域 `theme` 声明树表达式 → [`Tree`]。
///
/// 规则：对象字面量/数组/标量照抄；点链与裸名 lower 为 [`Tree::Ref`]
/// （主题数据上下文没有局部变量，点访问即槽引用——C-05）；`extend(...)`
/// 调用保留为 [`Tree::Extend`]，由 [`build_theme`] 展开；模板串/lambda 等
/// 非数据表达式报错。
pub fn lower_theme_tree(expr: &Expr) -> Result<Tree, ThemeError> {
    lower_at(expr, "")
}

fn lower_at(expr: &Expr, path: &str) -> Result<Tree, ThemeError> {
    match expr {
        Expr::Str(s) => Ok(Tree::Str(s.to_string())),
        Expr::CStr(s) => Ok(Tree::Str(s.to_string())),
        Expr::Int(i) => Ok(Tree::Num(*i as f64)),
        Expr::Uint(u) => Ok(Tree::Num(*u as f64)),
        Expr::I8(v) => Ok(Tree::Num(*v as f64)),
        Expr::U8(v) | Expr::Byte(v) => Ok(Tree::Num(*v as f64)),
        Expr::I64(v) => Ok(Tree::Num(*v as f64)),
        Expr::U64(v) => Ok(Tree::Num(*v as f64)),
        Expr::Float(v, _) | Expr::Double(v, _) => Ok(Tree::Num(*v)),
        Expr::Bool(b) => Ok(Tree::Bool(*b)),
        Expr::Object(pairs) => {
            let mut out = Vec::new();
            for pair in pairs {
                let key = pair_key(pair, path)?;
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                out.push((key, lower_at(&pair.value, &child_path)?));
            }
            Ok(Tree::Object(out))
        }
        Expr::Array(items) => {
            let mut out = Vec::new();
            for (i, item) in items.iter().enumerate() {
                out.push(lower_at(item, &format!("{path}[{i}]"))?);
            }
            Ok(Tree::Array(out))
        }
        Expr::Dot(root, field) => {
            let mut segs = flatten_dot(root, path)?;
            segs.push(field.to_string());
            Ok(Tree::Ref(segs))
        }
        Expr::Ident(name) => Ok(Tree::Ref(vec![name.to_string()])),
        Expr::Call(call) => {
            let callee = match call.name.as_ref() {
                Expr::Ident(n) => n.to_string(),
                other => {
                    return Err(ThemeError::at(
                        path,
                        format!("调用目标必须是名字，遇到 {other:?}"),
                    ))
                }
            };
            if callee != "extend" {
                return Err(ThemeError::at(
                    path,
                    format!("主题数据只支持 extend(...) 组合调用，遇到 `{callee}(...)`"),
                ));
            }
            let mut args = Vec::new();
            for (i, arg) in call.args.args.iter().enumerate() {
                args.push(lower_at(&arg.get_expr(), &format!("{path}.extend[{i}]"))?);
            }
            Ok(Tree::Extend(args))
        }
        other => Err(ThemeError::at(
            path,
            format!("主题数据不支持该表达式形态：{other:?}"),
        )),
    }
}

fn flatten_dot(expr: &Expr, path: &str) -> Result<Vec<String>, ThemeError> {
    match expr {
        Expr::Ident(name) => Ok(vec![name.to_string()]),
        Expr::Dot(root, field) => {
            let mut segs = flatten_dot(root, path)?;
            segs.push(field.to_string());
            Ok(segs)
        }
        other => Err(ThemeError::at(
            path,
            format!("点链根必须是名字，遇到 {other:?}"),
        )),
    }
}

fn pair_key(pair: &auto_parser_api::ast::Pair, path: &str) -> Result<String, ThemeError> {
    use auto_parser_api::ast::Key;
    let text = match &pair.key {
        Key::NamedKey(name) => name.to_string(),
        Key::StrKey(s) => s.to_string(),
        Key::IntKey(i) => i.to_string(),
        Key::BoolKey(b) => b.to_string(),
    };
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(ThemeError::at(path, "对象键为空"));
    }
    Ok(text)
}

// ── extend 合成（C-04）────────────────────────────────────────────────

/// extend 语义：左到右 patch；唯一命名字段递归合并（双方皆 Object）、
/// 标量后胜、数组整体替换；输入只读（产新树）。
pub fn extend_trees(base: &Tree, patches: &[Tree]) -> Result<Tree, ThemeError> {
    let mut acc = base.clone();
    for (i, patch) in patches.iter().enumerate() {
        let path = format!("extend[{i}]");
        if !matches!(patch, Tree::Object(_)) {
            return Err(ThemeError::at(
                path,
                "extend 的 patch 参数必须是对象字面量",
            ));
        }
        acc = merge_object(&acc, patch, &path)?;
    }
    Ok(acc)
}

fn merge_object(base: &Tree, patch: &Tree, path: &str) -> Result<Tree, ThemeError> {
    match (base, patch) {
        (Tree::Object(b), Tree::Object(p)) => {
            let mut out = b.clone();
            for (k, pv) in p {
                let child = format!("{path}.{k}");
                if let Some(slot) = out.iter_mut().find(|(bk, _)| bk == k) {
                    slot.1 = merge_object(&slot.1.clone(), pv, &child)?;
                } else {
                    out.push((k.clone(), pv.clone()));
                }
            }
            Ok(Tree::Object(out))
        }
        // 数组整体替换；标量后胜（含跨型覆盖 = patch 决定）
        (_, p) => Ok(p.clone()),
    }
}

// ── Schema 校验（C-02/03/11）──────────────────────────────────────────

/// 公共骨架键（C-03）。其余顶层键 = app/Feature 命名空间（C-02）。
pub const SKELETON_KEYS: [&str; 4] = ["tokens", "styles", "components", "modes"];
/// 模式键（C-06：system 是选择策略，不是第三棵树）。
pub const MODE_KEYS: [&str; 2] = ["light", "dark"];

/// 校验合成后的主题树。`namespaces` = 允许的 app 命名空间（消费者声明）。
pub fn validate_theme(tree: &Tree) -> Result<(), ThemeError> {
    let Tree::Object(pairs) = tree else {
        return Err(ThemeError::at("", "主题根必须是对象"));
    };
    let mut namespaces: Vec<&str> = Vec::new();
    for (k, v) in pairs {
        match k.as_str() {
            "tokens" => validate_tokens(v, "tokens")?,
            "styles" | "components" => {
                if !matches!(v, Tree::Object(_) | Tree::Extend(_)) {
                    return Err(ThemeError::at(k, "styles/components 必须是对象"));
                }
            }
            "modes" => validate_modes(v)?,
            other => {
                if !is_namespace_name(other) {
                    return Err(ThemeError::at(
                        other,
                        "顶层键必须是 tokens/styles/components/modes 或合法 app 命名空间名",
                    ));
                }
                namespaces.push(other);
            }
        }
    }
    Ok(())
}

fn is_namespace_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn validate_tokens(tree: &Tree, path: &str) -> Result<(), ThemeError> {
    let Tree::Object(pairs) = tree else {
        return Err(ThemeError::at(path, "tokens 必须是对象"));
    };
    for (k, v) in pairs {
        let child = format!("{path}.{k}");
        match k.as_str() {
            "colors" => validate_colors(v, &child)?,
            _ => validate_token_subtree(v, &child)?,
        }
    }
    Ok(())
}

/// `tokens.colors`：公共键对账 31 键封闭词表（C-02）；app 命名空间子对象
/// 里的键不受 31 键限制（值仍须是颜色）。
fn validate_colors(tree: &Tree, path: &str) -> Result<(), ThemeError> {
    let Tree::Object(pairs) = tree else {
        return Err(ThemeError::at(path, "tokens.colors 必须是对象"));
    };
    for (k, v) in pairs {
        let child = format!("{path}.{k}");
        if TokenName::from_css_var(k).is_none() {
            // 非公共键：仅允许作为 app 命名空间子对象出现
            return Err(ThemeError::at(
                &child,
                format!("未知公共颜色键 `{k}`（公共键须在 31 键封闭词表内；app 扩展色须放在 app 命名空间下）"),
            ));
        }
        validate_color_value(v, &child)?;
    }
    Ok(())
}

fn validate_token_subtree(tree: &Tree, path: &str) -> Result<(), ThemeError> {
    match tree {
        Tree::Object(pairs) => {
            for (k, v) in pairs {
                validate_token_subtree(v, &format!("{path}.{k}"))?;
            }
            Ok(())
        }
        Tree::Ref(_) => Ok(()),
        Tree::Str(s) => {
            if parse_color(s).is_some() || parse_length(s).is_some() {
                Ok(())
            } else {
                Err(ThemeError::at(
                    path,
                    format!("token 值 `{s}` 既非 #rrggbb 颜色也非长度（如 8dp/12px）"),
                ))
            }
        }
        Tree::Num(_) | Tree::Bool(_) => Ok(()),
        Tree::Array(_) => Ok(()),
        Tree::Extend(_) => Err(ThemeError::at(path, "extend 须先在 build 期展开")),
    }
}

fn validate_color_value(tree: &Tree, path: &str) -> Result<(), ThemeError> {
    match tree {
        Tree::Str(s) => {
            if parse_color(s).is_some() {
                Ok(())
            } else {
                Err(ThemeError::at(path, format!("颜色值 `{s}` 非 #rrggbb 形态")))
            }
        }
        Tree::Ref(_) => Ok(()),
        Tree::Object(pairs) => {
            // app 命名空间对象（如 my_app: { panel: "#eee8d5" }）
            for (k, v) in pairs {
                if !is_namespace_name(k) {
                    return Err(ThemeError::at(format!("{path}.{k}"), "app 扩展键名不合法"));
                }
                validate_color_value(v, &format!("{path}.{k}"))?;
            }
            Ok(())
        }
        other => Err(ThemeError::at(
            path,
            format!("颜色槽不接受该形态：{other:?}"),
        )),
    }
}

fn validate_modes(tree: &Tree) -> Result<(), ThemeError> {
    let Tree::Object(pairs) = tree else {
        return Err(ThemeError::at("modes", "modes 必须是对象"));
    };
    for (k, v) in pairs {
        if !MODE_KEYS.contains(&k.as_str()) {
            return Err(ThemeError::at(
                format!("modes.{k}"),
                "modes 只支持 light/dark（system 是选择策略，不是模式树）",
            ));
        }
        if !matches!(v, Tree::Object(_)) {
            return Err(ThemeError::at(format!("modes.{k}"), "模式差异必须是对象"));
        }
    }
    Ok(())
}

// ── 值解析 ────────────────────────────────────────────────────────────

/// `#rrggbb` → RGB。
pub fn parse_color(s: &str) -> Option<(u8, u8, u8)> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(h, 16).ok()?;
    Some(((n >> 16) as u8, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8))
}

/// 长度字面量：`<数>dp|px`（如 8dp、12px）。
pub fn parse_length(s: &str) -> Option<(f64, &'static str)> {
    let (num, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(s.len()));
    if num.is_empty() {
        return None;
    }
    let v: f64 = num.parse().ok()?;
    let unit = match unit {
        "dp" => "dp",
        "px" => "px",
        _ => return None,
    };
    Some((v, unit))
}

// ── build / 模式合成 ──────────────────────────────────────────────────

/// 一份已解析的具名主题（build 产物；`raw` 恒为原始公共数据——C-06 每次
/// 切模式从它重建，不叠加上次结果）。
#[derive(Debug, Clone)]
pub struct ThemeTree {
    pub name: String,
    /// 合成后的原始公共树（extend 已展开；不含 modes 差异叠加）。
    pub raw: Tree,
}

/// 展开主题声明：解析 → extend 展开（base 引用经 `known` 解析）→ 校验。
///
/// `known`：已知的具名主题（本模块先前 build 的 + use 导入的），键为
/// 访问路径点串（如 `themes.Auto` 或 `Auto`）。
pub fn build_theme(
    name: &str,
    tree_expr: &Expr,
    known: &BTreeMap<String, Tree>,
) -> Result<ThemeTree, ThemeError> {
    let lowered = lower_theme_tree(tree_expr)?;
    let raw = resolve_extends(&lowered, known)?;
    validate_theme(&raw)
        .map_err(|e| ThemeError::at(format!("{name}.{}", e.path), e.msg))?;
    Ok(ThemeTree {
        name: name.to_string(),
        raw,
    })
}

fn resolve_extends(tree: &Tree, known: &BTreeMap<String, Tree>) -> Result<Tree, ThemeError> {
    match tree {
        Tree::Extend(args) => {
            if args.is_empty() {
                return Err(ThemeError::at("extend", "extend 至少需要一个 base 参数"));
            }
            let base = resolve_base(&args[0], known)?;
            let patches: Vec<Tree> = args[1..]
                .iter()
                .map(|a| resolve_base(a, known))
                .collect::<Result<_, _>>()?;
            extend_trees(&base, &patches)
        }
        Tree::Object(pairs) => {
            let mut out = Vec::new();
            for (k, v) in pairs {
                out.push((k.clone(), resolve_extends(v, known)?));
            }
            Ok(Tree::Object(out))
        }
        Tree::Array(items) => Ok(Tree::Array(
            items
                .iter()
                .map(|i| resolve_extends(i, known))
                .collect::<Result<_, _>>()?,
        )),
        other => Ok(other.clone()),
    }
}

/// extend 的参数解析：引用 → 已知主题/导入成员；对象字面量 → 原样。
///
/// 引用形态：`extend("Solarized", "my_app.design:app_defaults", {...})`。
/// 用字符串而非裸名/点链——Core 解析器对对象值中的未声明标识符报
/// `undefined variable`（T-01 探针实证）；放行会改通用 Core 语义，超出
/// 本合同边界。字符串引用仍是纯数据（C-01），键表由
/// [`collect_theme_trees`] 的真实 use 解析供给（裸名 + 点路径双键）。
fn resolve_base(tree: &Tree, known: &BTreeMap<String, Tree>) -> Result<Tree, ThemeError> {
    let dotted = match tree {
        Tree::Ref(segs) => segs.join("."),
        Tree::Str(name) => name.clone(),
        Tree::Object(_) => return Ok(tree.clone()),
        _ => {
            return Err(ThemeError::at(
                "extend",
                "extend 的参数必须是主题对象或已知主题引用（字符串名）",
            ))
        }
    };
    known.get(&dotted).cloned().ok_or_else(|| {
        ThemeError::at(&dotted, format!("extend 的 base `{dotted}` 不是已知主题或导入成员"))
    })
}

/// 模式选择（C-06/07）：从原始公共数据重建——`modes.<mode>` 差异子树
/// 递归叠加后，剔除 modes 键。不继承上次已解析结果。
pub fn compose_mode(theme: &ThemeTree, dark: bool) -> Result<Tree, ThemeError> {
    let mode_key = if dark { "dark" } else { "light" };
    let mut out = theme.raw.clone();
    if let Some(diff) = theme.raw.get("modes").and_then(|m| m.get(mode_key)) {
        out = merge_object(&out, diff, mode_key)?;
    }
    if let Tree::Object(pairs) = &mut out {
        pairs.retain(|(k, _)| k != "modes");
    }
    Ok(out)
}

/// 模式可用性（C-07）：声明了 modes 却缺请求模式 → false（拒绝换肤）；
/// 未声明 modes（单模式主题）→ 任意模式都落到基座（true）。
pub fn mode_available(theme: &ThemeTree, dark: bool) -> bool {
    let Some(modes) = theme.raw.get("modes") else {
        return true;
    };
    modes.get(if dark { "dark" } else { "light" }).is_some()
}

/// 槽引用解析（C-05）：按路径逐段走对象；miss 给可定位诊断。
pub fn resolve_slot<'a>(tree: &'a Tree, path: &[String]) -> Result<&'a Tree, ThemeError> {
    let mut cur = tree;
    let mut walked = String::new();
    for seg in path {
        if !walked.is_empty() {
            walked.push('.');
        }
        walked.push_str(seg);
        match cur.get(seg) {
            Some(next) => cur = next,
            None => {
                let miss = walked.clone();
                return Err(ThemeError::at(miss.clone(), format!("未知主题路径 `{miss}`")));
            }
        }
    }
    Ok(cur)
}

// ── use 导入收集（真实模块路径 → 具名主题表）──────────────────────────

/// 收集 `at_path`（应用源）可用的全部具名主题树：应用自身声明 + `use`
/// 导入模块中的 `pub theme` 声明（经 [`crate::ui_ast::resolve_use_module`]
/// 真实模块解析）。返回键含裸名（`Auto`）与点路径（`themes.Auto`）两种
/// 形态，供 extend base 引用。
pub fn collect_theme_trees(
    at_path: &std::path::Path,
    code: &str,
) -> Result<BTreeMap<String, Tree>, ThemeError> {
    collect_theme_trees_with_dep_roots(at_path, code, &[])
}

/// PLAN-047 T-03：dep 根感知的主题树收集。`use themes.themes: …` 的模块
/// 解析在裸 `resolve_use_module`（base_dir 相对探针 + 父目录回落）下找不到
/// pac `dep themes { path: "../../../packages/themes" }` 的包根——本应用
/// 目录树里不存在 `themes/themes.at`，收集静默为空 → Vue 轨
/// `__AUTO_TREE_THEMES__` 零注入 → 官方包族（Solarized 等）在 Vue 侧永不
/// 上身（016 B2 vue 腿于 master 实测红、018/024 同症；plan-031/037 期
/// 通过依赖的是当时代码路径，其后回归，具体引入点未深挖——本修复按
/// 「dep 名 → 包根 → {rest}.at/mod.at」补全解析序，裸路径行为不变）。
pub fn collect_theme_trees_with_dep_roots(
    at_path: &std::path::Path,
    code: &str,
    dep_roots: &[(String, std::path::PathBuf)],
) -> Result<BTreeMap<String, Tree>, ThemeError> {
    let mut known: BTreeMap<String, Tree> = BTreeMap::new();
    let base_dir = at_path.parent().unwrap_or(std::path::Path::new("."));

    let ingest_source = |source_path: Option<&std::path::Path>,
                         source: &str,
                         module: Option<&str>,
                         known: &mut BTreeMap<String, Tree>|
     -> Result<(), ThemeError> {
        // 解析失败 = 该模块不是（合法的）主题数据模块——静默跳过，交给
        // 生产解析链报告其自身诊断（plan806 变异体等负控依赖此序：主题
        // 收集不得抢先吞掉/改写生产错误）。仅「解析成功但主题声明非法」
        // 才是收集错误（C-11 只对真实主题数据生效）。
        let ui = match crate::ui_ast::parse_ui(source) {
            Ok(ui) => ui,
            Err(_) => return Ok(()),
        };
        for stmt in ui.domain_stmts() {
            let auto_ui_dialect::UiStmt::ThemeDataDecl(decl) = stmt else {
                continue;
            };
            if !decl.is_pub && module.is_some() {
                continue; // 跨模块只取 pub；本模块私有的在处理本模块时可见
            }
            let theme = build_theme(decl.name.as_str(), &decl.tree, known)?;
            let tree = theme.raw.clone();
            known.insert(theme.name.clone(), tree.clone());
            if let Some(m) = module {
                known.insert(format!("{m}.{}", theme.name), tree);
            }
            let _ = source_path;
        }
        Ok(())
    };

    // 本应用源自身的声明（先于 use 导入 ingest 前注册？不——extend base
    // 可引用导入主题，也可被后声明引用；逐序 ingest：先 use 模块，再本源，
    // 保持源内声明顺序即可覆盖「后声明引用先声明」）。
    for use_stmt in auto_lang_core::use_scanner::scan_use_statements(code) {
        if use_stmt.is_c_import || use_stmt.is_rust_import || use_stmt.is_python_import {
            continue;
        }
        let resolved = match crate::ui_ast::resolve_use_module(base_dir, &use_stmt) {
            crate::UseModuleResolution::Module(path) => Some(path),
            _ => {
                // PLAN-047 T-03：裸解析失败后按 pac dep 根回落——模块首段
                // 命中 dep 名 → 在包根下探 {rest}.at / {rest}/mod.at / mod.at。
                let first = use_stmt.module.split('.').next().unwrap_or("");
                let rest = use_stmt.module.split_once('.').map(|(_, r)| r);
                dep_roots
                    .iter()
                    .filter(|(name, _)| name == first)
                    .find_map(|(_, root)| {
                        let mut cands: Vec<std::path::PathBuf> = Vec::new();
                        if let Some(r) = rest {
                            cands.push(root.join(format!("{}.at", r.replace('.', "/"))));
                            cands.push(root.join(r.replace('.', "/")).join("mod.at"));
                        }
                        cands.push(root.join("mod.at"));
                        cands.into_iter().find(|c| c.exists())
                    })
            }
        };
        if let Some(path) = resolved {
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            ingest_source(Some(&path), &source, Some(&use_stmt.module), &mut known)?;
        }
    }
    ingest_source(Some(at_path), code, None, &mut known)?;
    Ok(known)
}

/// 运行时注册形态：规范名（裸名）→ 已 build 的 [`ThemeTree`]。
pub fn collect_named_themes(
    at_path: &std::path::Path,
    code: &str,
) -> Result<BTreeMap<String, ThemeTree>, ThemeError> {
    collect_named_themes_with_dep_roots(at_path, code, &[])
}

/// PLAN-047 T-03：dep 根感知变体（见 [`collect_theme_trees_with_dep_roots`]）。
pub fn collect_named_themes_with_dep_roots(
    at_path: &std::path::Path,
    code: &str,
    dep_roots: &[(String, std::path::PathBuf)],
) -> Result<BTreeMap<String, ThemeTree>, ThemeError> {
    let known = collect_theme_trees_with_dep_roots(at_path, code, dep_roots)?;
    // 从已知键表重建 ThemeTree：仅取裸名键（dotted 键是同一主题的别名）。
    let mut out = BTreeMap::new();
    for (name, tree) in known {
        if !name.contains('.') {
            out.insert(
                name.clone(),
                ThemeTree {
                    name,
                    raw: tree,
                },
            );
        }
    }
    Ok(out)
}

// ── 投影（V-01 的两种投影：CSS 面 / VM RGB 面）────────────────────────

/// CSS 面投影：`tokens.colors`（31 键公共键）→ `--var: value;` 对；
/// app 命名空间颜色 → `--<ns>-<key>`。
pub fn css_var_pairs(tree: &Tree) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(colors) = tree.get("tokens").and_then(|t| t.get("colors")) {
        if let Tree::Object(pairs) = colors {
            for (k, v) in pairs {
                let value = match v {
                    Tree::Str(s) => s.clone(),
                    _ => continue,
                };
                if let Some(token) = TokenName::from_css_var(k) {
                    out.push((token.css_var().to_string(), value));
                }
            }
        }
    }
    // PLAN-007 T-05：尺寸槽（tokens.radius.* → `radius-*`；dp→px 供 CSS
    // var 消费——applyTheme 写 var，design 对象经 var() 引用随主题联动）。
    if let Some(radius) = tree.get("tokens").and_then(|t| t.get("radius")) {
        if let Tree::Object(pairs) = radius {
            for (k, v) in pairs {
                if let Tree::Str(s) = v {
                    if let Some((n, unit)) = parse_length(s) {
                        let px = if unit == "dp" { n } else { n };
                        out.push((format!("radius-{k}"), format!("{px}px")));
                    }
                }
            }
        }
    }
    // app 命名空间扩展色（顶层 `<ns>` 对象或 tokens.colors 下的 ns 子对象）
    if let Tree::Object(pairs) = tree {
        for (k, v) in pairs {
            if SKELETON_KEYS.contains(&k.as_str()) || !is_namespace_name(k) {
                continue;
            }
            collect_namespace_colors(v, k, &mut out);
        }
    }
    out
}

/// PLAN-031 T-06（F-1 续）：把合成后的主题树渲染成 index.css 的变量块
/// （`    --k: v;\n`，按 `order` 键序）——与 `registry::render_core`/
/// `ComposedTheme::render_core` 同形，供生成器在「App 用 `use` 选了官方包、
/// 但没有 pac `theme{}` 声明」时生成 `:root`/`.dark` 初始块，避免首帧/无容器
/// 回落落到静态表。值取 `css_hsl_var_pairs`（HSL 三段式）。
pub fn render_css_block(tree: &Tree, order: &[TokenName]) -> String {
    use std::collections::BTreeMap;
    let pairs: BTreeMap<String, String> = css_hsl_var_pairs(tree).into_iter().collect();
    let mut out = String::new();
    for t in order {
        if let Some(v) = pairs.get(t.css_var()) {
            out.push_str(&format!("    --{}: {};\n", t.css_var(), v));
        }
    }
    out
}

/// PLAN-031 T-06（F-1）：CSS 面投影的 **Tailwind 消费形**。
///
/// `css_var_pairs` 输出的是作者原值（多为 `#rrggbb`），而 Vue 脚手架的样式
/// 以 `hsl(var(--token))` 消费（tailwind config + `@apply`），hex 代入后
/// 整条声明 invalid-at-computed-value，实际回落静态表——表现为「Vue 侧
/// 不吃官方包颜色」。本函数把**公共 31 键**的颜色值转成 `H S% L%` 三段式；
/// 已是 HSL 串/非颜色值原样保留；radius 槽与 app 命名空间色不在此转换
/// （命名空间色供 `var()` 直用，保持原值）。
pub fn css_hsl_var_pairs(tree: &Tree) -> Vec<(String, String)> {
    css_var_pairs(tree)
        .into_iter()
        .map(|(k, v)| {
            if TokenName::from_css_var(&k).is_some() {
                if let Some(rgb) = parse_color(&v) {
                    return (k, registry::rgb_to_hsl_str(rgb));
                }
            }
            (k, v)
        })
        .collect()
}

fn collect_namespace_colors(tree: &Tree, ns: &str, out: &mut Vec<(String, String)>) {    match tree {
        Tree::Object(pairs) => {
            for (k, v) in pairs {
                match v {
                    Tree::Str(s) => {
                        if parse_color(s).is_some() {
                            out.push((format!("{ns}-{k}"), s.clone()));
                        }
                    }
                    _ => collect_namespace_colors(v, &format!("{ns}-{k}"), out),
                }
            }
        }
        _ => {}
    }
}

/// VM 面投影：公共 31 键 token → RGB（hex 原生；HSL 串经 registry 解析）。
pub fn resolve_rgb(tree: &Tree, token: TokenName) -> Option<(u8, u8, u8)> {
    let v = tree
        .get("tokens")?
        .get("colors")?
        .get(token.css_var())?;
    let s = v.as_str()?;
    parse_color(s).or_else(|| registry::hsl_str_to_rgb(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-047 T-03：pac dep 根感知解析。裸 resolve_use_module 找不到
    /// `use themes.themes`（包在 dep 根下，不在 app 源树内）——收集曾静默
    /// 为空 → Vue 轨 __AUTO_TREE_THEMES__ 零注入（016/018/024 实证回归）。
    #[test]
    fn plan047_collect_resolves_use_via_dep_roots() {
        let dir = std::env::temp_dir().join(format!("plan047-themetree-{}", std::process::id()));
        let pkg = dir.join("packages/themes");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("themes.at"),
            "pub theme Solarized = {\n    tokens: { colors: { background: \"#002b36\" } }\n}\n",
        )
        .unwrap();
        let app = dir.join("app.at");
        let code = "use themes.themes: Auto, Solarized\n\nwidget App { view { col { text \"x\" } } }\n";
        std::fs::write(&app, code).unwrap();

        // 裸解析：0 族（回归前的实证行为）。
        let bare = collect_named_themes(&app, code).unwrap();
        assert!(
            !bare.contains_key("Solarized"),
            "裸解析不应找到 dep 包（否则本测试前提失效）: {:?}",
            bare.keys()
        );
        // dep 根解析：Solarized 入表。
        let rooted = collect_named_themes_with_dep_roots(
            &app,
            code,
            &[("themes".to_string(), dir.join("packages/themes"))],
        )
        .unwrap();
        assert!(
            rooted.contains_key("Solarized"),
            "dep 根解析必须收录 Solarized: {:?}",
            rooted.keys()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn obj(pairs: Vec<(&str, Tree)>) -> Tree {
        Tree::Object(
            pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn s(v: &str) -> Tree {
        Tree::Str(v.to_string())
    }

    /// C-04：递归合并 / 标量后胜 / 数组整体替换 / 未触字段保留。
    #[test]
    fn extend_merges_recursively_and_replaces_arrays() {
        let base = obj(vec![
            ("tokens", obj(vec![("radius", obj(vec![("control", s("8dp"))]))])),
            ("tags", Tree::Array(vec![s("a"), s("b")])),
        ]);
        let patch = obj(vec![
            ("tokens", obj(vec![("radius", obj(vec![("control", s("12dp"))]))])),
            ("tags", Tree::Array(vec![s("c")])),
        ]);
        let merged = extend_trees(&base, &[patch]).unwrap();
        assert_eq!(
            merged.get("tokens").unwrap().get("radius").unwrap().get("control"),
            Some(&s("12dp"))
        );
        assert_eq!(merged.get("tags"), Some(&Tree::Array(vec![s("c")])));
        // 输入只读
        assert_eq!(
            base.get("tokens").unwrap().get("radius").unwrap().get("control"),
            Some(&s("8dp"))
        );
    }

    /// C-02/03/11：骨架 + app 命名空间；未知公共键/非法模式键给可定位错误。
    #[test]
    fn schema_validates_skeleton_namespaces_and_rejects_unknowns() {
        let good = obj(vec![
            (
                "tokens",
                obj(vec![("colors", obj(vec![("background", s("#ffffff"))]))]),
            ),
            (
                "modes",
                obj(vec![("dark", obj(vec![("tokens", obj(vec![]))]))]),
            ),
            (
                "my_app",
                obj(vec![("panel", s("#eee8d5"))]),
            ),
        ]);
        assert!(validate_theme(&good).is_ok());

        let bad_color = obj(vec![(
            "tokens",
            obj(vec![("colors", obj(vec![("nope", s("#000000"))]))]),
        )]);
        let e = validate_theme(&bad_color).unwrap_err();
        assert_eq!(e.path, "tokens.colors.nope");

        let bad_mode = obj(vec![("modes", obj(vec![("night", obj(vec![]))]))]);
        let e = validate_theme(&bad_mode).unwrap_err();
        assert_eq!(e.path, "modes.night");

        let bad_value = obj(vec![(
            "tokens",
            obj(vec![("radius", obj(vec![("control", s("blue"))]))]),
        )]);
        let e = validate_theme(&bad_value).unwrap_err();
        assert!(e.msg.contains("既非"), "{}", e.msg);
    }

    /// C-06/07：模式从原始数据重建；dark→light 往返无遗留；无该模式报错面
    /// 由消费方以 MODE_KEYS 判定。
    #[test]
    fn compose_mode_rebuilds_from_raw_each_time() {
        let theme = ThemeTree {
            name: "t".into(),
            raw: obj(vec![
                (
                    "tokens",
                    obj(vec![("colors", obj(vec![("background", s("#ffffff"))]))]),
                ),
                (
                    "modes",
                    obj(vec![(
                        "dark",
                        obj(vec![(
                            "tokens",
                            obj(vec![("colors", obj(vec![("background", s("#09090b"))]))]),
                        )]),
                    )]),
                ),
            ]),
        };
        let light = compose_mode(&theme, false).unwrap();
        let dark = compose_mode(&theme, true).unwrap();
        assert_eq!(
            resolve_rgb(&light, TokenName::Background),
            Some((255, 255, 255))
        );
        assert_eq!(
            resolve_rgb(&dark, TokenName::Background),
            Some((9, 9, 11))
        );
        // 往返无遗留：dark 合成体不含 modes 键，light 不携带 dark 值
        assert!(light.get("modes").is_none());
        assert!(dark.get("modes").is_none());
        let light_again = compose_mode(&theme, false).unwrap();
        assert_eq!(light_again, light);
    }

    /// C-05：槽引用按路径解析；miss 可定位。
    #[test]
    fn slot_refs_resolve_by_path() {
        let tree = obj(vec![(
            "tokens",
            obj(vec![("radius", obj(vec![("control", s("8dp"))]))]),
        )]);
        let v = resolve_slot(&tree, &["tokens".into(), "radius".into(), "control".into()]).unwrap();
        assert_eq!(v.as_str(), Some("8dp"));
        let e = resolve_slot(&tree, &["tokens".into(), "nope".into()]).unwrap_err();
        assert_eq!(e.path, "tokens.nope");
    }

    /// C-07 事务面：单模式主题请求未声明模式 → mode_available=false
    ///（eval 据此拒绝换肤并保旧；plan806 类负控不涉此路径）。
    #[test]
    fn mode_available_rejects_undeclared_mode() {
        let light_only = ThemeTree {
            name: "lightonly".into(),
            raw: obj(vec![
                ("tokens", obj(vec![("colors", obj(vec![("background", s("#ffffff"))]))])),
                ("modes", obj(vec![("light", obj(vec![]))])),
            ]),
        };
        assert!(mode_available(&light_only, false));
        assert!(!mode_available(&light_only, true), "未声明 dark 必须拒绝");
        // 无 modes 键的单模式主题：任意模式落到基座
        let plain = ThemeTree {
            name: "plain".into(),
            raw: obj(vec![("tokens", obj(vec![("colors", obj(vec![("background", s("#ffffff"))]))]))]),
        };
        assert!(mode_available(&plain, true));
        assert!(mode_available(&plain, false));
    }

    /// build 期 extend 展开 + 投影（CSS 对 + VM RGB）。
    #[test]
    fn build_theme_expands_extend_and_projects() {
        let base = obj(vec![(
            "tokens",
            obj(vec![
                ("colors", obj(vec![("primary", s("#8b5cf6"))])),
                ("radius", obj(vec![("control", s("8dp"))])),
            ]),
        )]);
        let mut known = BTreeMap::new();
        known.insert("themes.Auto".to_string(), base);

        let src = r##"extend("themes.Auto", {
    tokens: { radius: { control: "12dp" } }
    modes: { dark: { tokens: { colors: { primary: "#7c3aed" } } } }
})"##;
        let expr = parse_expr_help(src);
        let theme = build_theme("Pilot", &expr, &known).unwrap();
        let light = compose_mode(&theme, false).unwrap();
        let dark = compose_mode(&theme, true).unwrap();
        assert_eq!(resolve_rgb(&light, TokenName::Primary), Some((139, 92, 246)));
        assert_eq!(resolve_rgb(&dark, TokenName::Primary), Some((124, 58, 237)));
        let css = css_var_pairs(&light);
        assert!(css.contains(&("primary".to_string(), "#8b5cf6".to_string())));
    }

    /// PLAN-031 T-06（F-1）：Tailwind 消费形投影——公共键输出 HSL 三段式，
    /// 命名空间色/radius 槽保持原值（`var()` 直用面不被改写）。
    #[test]
    fn css_hsl_pairs_convert_public_tokens_only() {
        let tree = obj(vec![(
            "tokens",
            obj(vec![
                (
                    "colors",
                    obj(vec![
                        ("primary", s("#8b5cf6")),
                        ("background", s("#09090b")),
                        ("success", s("#22c55e")),
                    ]),
                ),
                ("radius", obj(vec![("control", s("8dp"))])),
            ]),
        )]);
        let pairs = css_hsl_var_pairs(&tree);
        let get = |k: &str| {
            pairs
                .iter()
                .find(|(name, _)| name == k)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| panic!("{k} 缺失，实际 {pairs:?}"))
        };
        // #8b5cf6 → hsl(258, 90%, 66%)（与 registry::rgb_to_hsl_str 同源）
        assert_eq!(get("primary"), "258 90% 66%");
        assert_eq!(get("background"), "240 10% 4%");
        assert_eq!(get("success"), "142 71% 45%");
        // radius 槽不参与颜色转换
        assert_eq!(get("radius-control"), "8px");
        // 原值面（未转形）仍保留 hex，供 `var()` 直用消费者
        let raw = css_var_pairs(&tree);
        assert!(raw.contains(&("primary".to_string(), "#8b5cf6".to_string())));

        // index.css 块渲染：按 CORE_ORDER 顺序 + HSL 值 + 4 空格缩进
        let block = render_css_block(&tree, &registry::CORE_ORDER);
        assert!(
            block.contains("    --primary: 258 90% 66%;\n"),
            "core 块应含 HSL primary：{block}"
        );
        let bg_pos = block.find("--background").expect("background 行");
        let fg_pos = block.find("--foreground");
        assert!(
            fg_pos.is_none() || bg_pos < fg_pos.unwrap_or(usize::MAX),
            "键序应遵循 CORE_ORDER（background 在 foreground 之前）：{block}"
        );
    }

    fn parse_expr_help(src: &str) -> Expr {        // 借真实 UI 解析器取对象表达式（不手搓树）
        let ui = crate::ui_ast::parse_ui(&format!("pub theme T = {src}\n"))
            .expect("parse theme source");
        let decl = ui
            .domain_stmts()
            .find_map(|stmt| match stmt {
                auto_ui_dialect::UiStmt::ThemeDataDecl(t) => Some(t),
                _ => None,
            })
            .expect("theme decl");
        decl.tree.clone()
    }
}
