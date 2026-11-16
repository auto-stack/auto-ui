//! PLAN-980 WEB：976 TS/JS provider 的 **UI 消费切片**（fragment 接口）。
//!
//! 链路：UI 表达式/语句**源文本**（fixture 真源或适配器 seam 供给）→ 合成
//! 包装函数 `fn __frag() { <body> }` → ACE 前门编译（text-record）→
//! `SemanticView` → 976 `emit_expr_fragment`/`emit_stmt_fragment`（HIR 真值
//! 发射，`p<i>_*` 命名 + `a2x_*` helper 义务）→ **宿主映射**（reactive
//! `.count` → `count.value` 等 UI 侧职责，976 合同：反应性/捕获/宿主变量
//! 替换留适配器）→ 可选真 node 执行验证。
//!
//! 边界（具名）：本切片证明生产表达式形态经新 provider 可发射可执行；完整
//! 适配器切换（ts_adapter 四处旧 TypeScriptTrans 委托替换 + 全 UI 合法表面
//! + 浏览器挂载档）为 WEB 批后续步——本模块不回退旧 emitter（新车道未覆盖
//! 形态=具名拒绝）。

use crate::ui::ace_state_bridge::AceStateValue;
use ace_a2x_core::SemanticView;

/// 一次 fragment 发射的宿主侧产物。
#[derive(Clone, Debug)]
pub struct UiFragment {
    /// 语句行（或表达式行——表达式 fragment 的最后一行为表达式本体）。
    pub lines: Vec<String>,
    /// 表达式本体（表达式 fragment 专有）。
    pub expr: Option<String>,
    /// 需要从 `a2x_rt.mjs` 导入的 helper 名（首用序）。
    pub helpers: Vec<String>,
    /// 局部映射（index → 发射名/源名/类型/角色）——宿主对齐用。
    pub locals: Vec<(usize, String, String)>,
}

/// 合成包装源并编译为语义视图所需的最小闭包（core 面：int 域，语句面=
/// 初始化/赋值/return/条件——与 976 tsfrag 同形）。**宿主状态建模为参数**
///（`p<i>_<name>` 发射名即 reactive 宿主映射源），`host_map` 的源名即参数名。
fn wrapper_source(body_source: &str, params: &[&str]) -> String {
    // 参数表：`a int, b int`（宿主状态=参数，见函数注记）。
    let decl = params
        .iter()
        .map(|p| format!("{p} int"))
        .collect::<Vec<_>>()
        .join(", ");
    // 体已含 return（表达式住 return/赋值/条件）时不补尾 return——避免
    // 不可达块（HIR block-graph 连通性校验）。
    let tail = if body_source.trim_start().starts_with("return") {
        String::new()
    } else {
        "
return 0"
            .to_string()
    };
    format!(
        "fn __frag({decl}) int {{
{body_source}{tail}
}}
"
    )
}

/// 发射**表达式** fragment：`body_source` 须含该表达式的源文本；按源 span
/// 定位（`innermost_expr_in_span`）到 HIR 表达式。state 名→宿主映射由
/// `host_map` 提供（如 `count` → `count.value`——反应性职责在适配器）。
pub fn emit_ui_expr_fragment(
    body_source: &str,
    expr_source: &str,
    params: &[&str],
    host_map: &[(String, String)],
) -> Result<UiFragment, String> {
    let out = emit_fragment(
        wrapper_source(body_source, params),
        Some(expr_source),
        host_map,
    )?;
    Ok(out)
}

/// 发射**语句** fragment（包装函数体 block 0 的第 `stmt_idx` 条语句）。
pub fn emit_ui_stmt_fragment(
    body_source: &str,
    stmt_idx: usize,
    params: &[&str],
    host_map: &[(String, String)],
) -> Result<UiFragment, String> {
    let source = wrapper_source(body_source, params);
    emit_fragment_at(source, None, Some(stmt_idx), host_map)
}

#[allow(clippy::too_many_arguments)]
fn emit_fragment(
    source: String,
    expr_locator: Option<&str>,
    host_map: &[(String, String)],
) -> Result<UiFragment, String> {
    emit_fragment_at(source, expr_locator, None, host_map)
}

fn emit_fragment_at(
    source: String,
    expr_locator: Option<&str>,
    locator_stmt_idx: Option<usize>,
    host_map: &[(String, String)],
) -> Result<UiFragment, String> {
    use ace_a2x_ts::{
        emit_expr_fragment, emit_stmt_fragment, innermost_expr_in_span, FragmentProgramContext,
        ScriptLang,
    };
    // 前门：core 编译（976 UI fragment 同一入口 `compile_source`；共享
    // verify，诊断带码上抛）。包装源是普通 fn 模块。
    let compiled = ace_core::source::compile_source(&source, "p980-frag", &[]).map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let view = SemanticView::from_checked(&compiled.checked)
        .map_err(|e| format!("semantic view: {e:?}"))?;
    // 按源名取函数事实（入口概念限无参 d_main——包装函数带宿主状态参数，
    // 与 976 tsfrag 的 facts_of 同法）。
    let facts = view
        .functions()
        .iter()
        .find(|f| f.source_name == "__frag")
        .ok_or_else(|| "wrapper function `__frag` missing from the view".to_string())?;
    let ctx = FragmentProgramContext::from_view(&view);
    let stmt_idx = locator_stmt_idx.unwrap_or(0);
    let raw = if let Some(expr_src) = expr_locator {
        let start = source.find(expr_src).ok_or_else(|| {
            format!("expression source `{expr_src}` not found in the wrapper source")
        })?;
        let expr_id = innermost_expr_in_span(facts, start..start + expr_src.len())
            .ok_or_else(|| "no HIR expression inside the located span".to_string())?;
        emit_expr_fragment(ScriptLang::TypeScript, &view, facts, expr_id, &ctx)
            .map_err(|e| format!("expr fragment: {e:?}"))?
    } else {
        emit_stmt_fragment(ScriptLang::TypeScript, &view, facts, 0, stmt_idx, &ctx)
            .map_err(|e| format!("stmt fragment: {e:?}"))?
    };
    // 宿主映射（两跳）：fragment 文本携带**发射名**（l<i>_<src>/p<i>_<src>/
    // env.e<i>_<src>）——先经 locals 回源名，再按适配器的 源名→宿主名 表
    // （如 `count` → `count.value`，反应性职责在适配器）。未映射名保持原样。
    let emitted_to_source: std::collections::HashMap<String, String> = raw
        .locals
        .iter()
        .map(|l| (l.emitted_name.clone(), l.source_name.clone()))
        .collect();
    let map_name = |n: &str| -> String {
        let source_name = emitted_to_source
            .get(n)
            .cloned()
            .unwrap_or_else(|| n.to_string());
        host_map
            .iter()
            .find(|(from, _)| *from == source_name)
            .map(|(_, to)| to.clone())
            .unwrap_or(source_name)
    };
    let lines = raw
        .text
        .iter()
        .map(|l| host_rewrite(l, &map_name))
        .collect();
    let expr = raw.expr.as_ref().map(|e| host_rewrite(e, &map_name));
    Ok(UiFragment {
        lines,
        expr,
        helpers: raw.helpers.clone(),
        locals: raw
            .locals
            .iter()
            .map(|l| {
                (
                    l.index as usize,
                    l.emitted_name.clone(),
                    l.source_name.clone(),
                )
            })
            .collect(),
    })
}

/// 词法安全的宿主名替换（整词边界，避免 p0_count_x 误替 p0_count）。
fn host_rewrite(line: &str, map_name: &impl Fn(&str) -> String) -> String {
    let mut out = String::with_capacity(line.len());
    let mut word = String::new();
    for ch in line.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            if !word.is_empty() {
                out.push_str(&map_name(&word));
                word.clear();
            }
            out.push(ch);
        }
    }
    if !word.is_empty() {
        out.push_str(&map_name(&word));
    }
    out
}

/// 真 node 执行验证（976 观察协议：FRAG_OK 风格由宿主驱动拼写；沿用
/// ts_ui_fragment 先例）：拼宿主驱动（预声明局部 + helper 导入 + fragment
/// 行 + 断言），node 跑，返回 stdout。
pub fn run_fragment_on_node(
    fragment: &UiFragment,
    prelude: &str,
    assertion: &str,
) -> Result<String, String> {
    let body = fragment.lines.join("\n");
    let imports = if fragment.helpers.is_empty() {
        String::new()
    } else {
        format!(
            "import {{ {} }} from \"./a2x_rt.mjs\";\n",
            fragment.helpers.join(", ")
        )
    };
    let driver = format!(
        "{imports}{prelude}\n{body}\nif (!({assertion})) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-tsfrag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("host.mjs"), driver).map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join("a2x_rt.mjs"),
        ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[]),
    )
    .map_err(|e| e.to_string())?;
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg("host.mjs")
        .current_dir(&dir)
        .output()
        .map_err(|e| format!("node spawn: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    if !out.status.success() {
        return Err(format!("node failed: {stderr}"));
    }
    Ok(stdout)
}

/// 保留：与状态桥同形的值枚举引用（宿主映射侧工具，未用则删）。
#[allow(dead_code)]
fn _value_kind(v: &AceStateValue) -> &'static str {
    match v {
        AceStateValue::Int(_) => "int",
        AceStateValue::Bool(_) => "bool",
        AceStateValue::Str(_) => "str",
    }
}
