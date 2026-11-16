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

/// 合成包装源并编译为语义视图所需的最小闭包（core 面：int/bool/str 域，
/// 语句面=初始化/赋值/return/条件——与 976 tsfrag 同形；类型域=切片 #2 扩
/// 面）。**宿主状态建模为参数**（`p<i>_<name>` 发射名即 reactive 宿主映射
/// 源），`host_map` 的源名即参数名。
fn wrapper_source(body_source: &str, params: &[(&str, &str)]) -> String {
    // 参数表：`a int, b bool, c str`（宿主状态=参数，见函数注记）。
    let decl = params
        .iter()
        .map(|(name, ty)| format!("{name} {ty}"))
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
    params: &[(&str, &str)],
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
    params: &[(&str, &str)],
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

// ---------------------------------------------------------------------------
// 组件面 fragment lane（切片 #4）：生产形态源 → 组件面编译（一次性）→
// 状态函数 HIR（**原源 span**）→ 按表达式源 span 精确定位发射。
// ---------------------------------------------------------------------------

/// 一次组件面编译承载的 fragment lane（owned 编译结果；view 按调用借用）。
/// **span 坐标基准 = f-string 脱糖后源**（组件面入口统一脱糖——DIALECT
/// 批口径；无 f-string 源时与原源一致）。lane 持有脱糖源供定位。
pub struct ComponentFragmentLane {
    ds_src: String,
    checked: ace_core::verify::CheckedModule,
}

/// 状态函数内按**源 span** 定位的发射结果（raw 面；宿主映射由调用方做）。
pub struct LocatedFragment {
    pub text: Vec<String>,
    pub expr: Option<String>,
    pub helpers: Vec<String>,
}

impl ComponentFragmentLane {
    /// 生产形态源（`!`/str/bool/比较——Core profile 拒的形态）经组件面
    /// states-only 编译。实例 1、ops=**待定位 handler 的 msg 重放**（状态
    /// 函数体含 handler 效果表达式，`__age_state_<i>_<field>`，span=原源
    /// 坐标——ops 空则 handler 体缺席，切片#4 探针实证）。
    pub fn compile(src: &str, path: &str, handler_msgs: &[&str]) -> Result<Self, String> {
        // 坐标基准：入口统一 f-string 脱糖（与组件面编译同一脱糖——
        // `ace_core::source::fstr::desugar_fstrings_widget`，公共出口）。
        let ds_src = ace_core::source::fstr::desugar_fstrings_widget(src)
            .map_err(|e| format!("fstr desugar: {e}"))?;
        let src = ds_src.as_str();
        let caps = ["ac.runtime.text.v1".to_string()];
        let ops = handler_msgs
            .iter()
            .map(|m| ace_av::age_component::ScenarioOp::Send {
                instance: 0,
                msg: (*m).to_string(),
            })
            .collect();
        let compiled = ace_av::age_component::compile_component_scenario_states(
            src,
            path,
            ace_av::age_component::LegacyWidgetAlias::Accept,
            &caps,
            &ace_av::age_component::ComponentScenario { instances: 1, ops },
            &ace_av::age_component::ComponentInitials {
                per_instance: vec![Vec::new()],
            },
        )
        .map_err(|ds| {
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        Ok(Self {
            ds_src,
            checked: compiled.checked,
        })
    }

    /// 状态函数内、**精确落在 `span`** 的表达式发射。同 span 多表达式时
    /// 取**索引序最大**者（组合形——如 `!on` 的 NOT——排在纯读形之后，
    /// 切片#4 探针实证）；span 越界/无命中=具名拒。
    #[allow(clippy::needless_range_loop)]
    pub fn emit_expr_at_span(
        &self,
        state_fn: &str,
        span: std::ops::Range<usize>,
    ) -> Result<LocatedFragment, String> {
        use ace_a2x_ts::{emit_expr_fragment, FragmentProgramContext, ScriptLang};
        if span.start >= span.end || span.end > self.ds_src.len() {
            return Err(format!(
                "span [{})..{}) is outside the desugared source (len {})",
                span.start,
                span.end,
                self.ds_src.len()
            ));
        }
        let view = ace_a2x_core::SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name == state_fn)
            .ok_or_else(|| format!("state fn `{state_fn}` not in the view"))?;
        let mut best: Option<(usize, ace_core::hir::ExprId)> = None;
        for (i, e) in facts.body.exprs.iter().enumerate() {
            if e.span.start == span.start && e.span.end == span.end {
                // 同 span 取索引最大（组合形在后）。
                best = Some((i, ace_core::hir::ExprId(i as u32)));
            }
        }
        let (_, expr_id) = best.ok_or_else(|| {
            format!(
                "no HIR expression with the exact span [{})..{}) in `{state_fn}`",
                span.start, span.end
            )
        })?;
        let ctx = FragmentProgramContext::from_view(&view);
        let raw = emit_expr_fragment(ScriptLang::TypeScript, &view, facts, expr_id, &ctx)
            .map_err(|e| format!("expr fragment: {e:?}"))?;
        Ok(LocatedFragment {
            text: raw.text,
            expr: raw.expr,
            helpers: raw.helpers,
        })
    }

    /// 脱糖源（span 坐标基准；适配器定位/映射用）。
    pub fn desugared_source(&self) -> &str {
        &self.ds_src
    }

    /// 按**源文本锚**定位发射：`needle` 在脱糖源中的第 `nth` 处（0 起），
    /// 取 span **包含**该位置的最内层表达式（同 span 组合形在前——索引
    /// 序最大优先）。适配器合同：持有源文本即可定位，无预知 span。
    pub fn emit_expr_at_text(
        &self,
        state_fn: &str,
        needle: &str,
        nth: usize,
    ) -> Result<LocatedFragment, String> {
        let mut hit: Option<std::ops::Range<usize>> = None;
        let mut from = 0usize;
        let mut seen = 0usize;
        while let Some(pos) = self.ds_src[from..].find(needle) {
            if seen == nth {
                hit = Some(from + pos..from + pos + needle.len());
                break;
            }
            seen += 1;
            from = from + pos + needle.len();
        }
        let range = hit.ok_or_else(|| {
            format!("needle {needle:?} (nth {nth}) not found in the desugared source")
        })?;
        // 包含 range 起点的最内层 expr；同 span 取索引最大（组合形）。
        let view = ace_a2x_core::SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        use ace_a2x_ts::{emit_expr_fragment, FragmentProgramContext, ScriptLang};
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name == state_fn)
            .ok_or_else(|| format!("state fn `{state_fn}` not in the view"))?;
        let mut best: Option<(usize, ace_core::hir::ExprId)> = None;
        for (i, e) in facts.body.exprs.iter().enumerate() {
            if e.span.start <= range.start && range.start < e.span.end {
                let width = e.span.end.saturating_sub(e.span.start);
                match best {
                    Some((w, _)) if w < width => {}
                    _ => best = Some((width, ace_core::hir::ExprId(i as u32))),
                }
            }
        }
        let (_, expr_id) = best.ok_or_else(|| {
            format!(
                "no HIR expression covers {needle:?} at [{})..{}) in `{state_fn}`",
                range.start, range.end
            )
        })?;
        let ctx = FragmentProgramContext::from_view(&view);
        let raw = emit_expr_fragment(ScriptLang::TypeScript, &view, facts, expr_id, &ctx)
            .map_err(|e| format!("expr fragment: {e:?}"))?;
        Ok(LocatedFragment {
            text: raw.text,
            expr: raw.expr,
            helpers: raw.helpers,
        })
    }
}
