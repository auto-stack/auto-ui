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
        AceStateValue::F32(_) => "f32",
    }
}

// ---------------------------------------------------------------------------
// 组件面 fragment lane（切片 #4）：生产形态源 → 组件面编译（一次性）→
// 状态函数 HIR（**原源 span**）→ 按表达式源 span 精确定位发射。
// ---------------------------------------------------------------------------

/// 行首槽位 `state`→`model` 同长归一化（5↔5 字节，span 稳定；仅行首
/// trimmed 以 `state {`/`state{` 开头的行——字符串/注释/标识符使用不碰）。
fn normalize_state_slots(src: &str) -> String {
    src.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("state {") || trimmed.starts_with("state{") {
                let indent_len = line.len() - trimmed.len();
                format!("{}model {}", &line[..indent_len], &trimmed[5..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if src.ends_with('\n') { "\n" } else { "" }
}

/// 切片#6k：lane 脱糖源坐标基准的**公共出口**（与
/// [`ComponentFragmentLane::compile`] 同一归一化+脱糖链）——区间配对方
///（`pair_lambda_regions`）与 lane 保持同一坐标系，防止原源/脱糖源位移。
pub fn desugared_lane_source(src: &str) -> Result<String, String> {
    let norm = normalize_dollar_event(&normalize_state_slots(src));
    ace_core::source::fstr::desugar_fstrings_widget(&norm).map_err(|e| format!("fstr desugar: {e}"))
}

/// 切片#6m：`$event` 宿主事件占位的**同长归一化**（`$event`→`nevent`，
/// 6↔6 字节——span 零位移，沿 state→model 同法）。词法整词替换：方言事件
/// 绑定实参位的占位（方言合同）；归一化后按普通 ident 进 age-event-arg
///（pratt）；投影层丢弃实参（宿主通道，§13.14）。
fn normalize_dollar_event(src: &str) -> String {
    src.replace("$event", "nevent")
}

/// 一次组件面编译承载的 fragment lane（owned 编译结果；view 按调用借用）。
/// **span 坐标基准 = f-string 脱糖后源**（组件面入口统一脱糖——DIALECT
/// 批口径；无 f-string 源时与原源一致）。lane 持有脱糖源供定位。
pub struct ComponentFragmentLane {
    ds_src: String,
    checked: ace_core::verify::CheckedModule,
    /// PLAN-980 WEB #6p：selected node 每 handler 的通道事实（语言侧投影
    /// 的权威数据——callee=头参数判定在语言面完成）。
    channel_facts: Vec<LaneHandlerFacts>,
}

/// PLAN-980 WEB #6p：一个 handler 的通道事实（按 msg 取用）。
pub(crate) struct LaneHandlerFacts {
    /// handler msg 名（pattern 去点）。
    pub msg: String,
    /// 通道调用行（span 坐标=脱糖源；实参 span 供 `emit_expr_at_span`
    /// 经 a2x-ts 精确发射——实参已在语言侧死 let 验证）。
    pub rows: Vec<ace_av::age_component::ComponentChannelCall>,
    /// 域外体标记（let/if/非头参数调用——检查函数不铸，lane 缺席）。
    pub body_foreign: bool,
    /// 赋值语句数（语言侧投影——纯通道体=0：不需要发射函数）。
    pub n_assigns: usize,
}

/// PLAN-980 WEB #6p：一个通道调用行（span 坐标=脱糖源；实参 span 供
/// `emit_expr_at_span` 经 a2x-ts 精确发射——实参已在语言侧死 let 验证）。
pub struct LaneChannelRow {
    pub channel: String,
    pub arg_spans: Vec<std::ops::Range<usize>>,
    pub span: std::ops::Range<usize>,
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
        Self::compile_at(src, path, handler_msgs, 0)
    }

    /// 切片#6n：node 选择器形态（多 widget 同文件源——语言面
    /// `compile_component_scenario_states_at`；`node_index` 越界=编译期具
    /// 名拒上抛）。
    pub fn compile_at(
        src: &str,
        path: &str,
        handler_msgs: &[&str],
        node_index: usize,
    ) -> Result<Self, String> {
        // 坐标基准：入口统一 f-string 脱糖（与组件面编译同一脱糖——
        // `ace_core::source::fstr::desugar_fstrings_widget`，公共出口）。
        // 前置归一化：行首 `state {`→`model {`（组件面槽位语法；state/model
        // 同为 5 字节——**span 零位移**；真语料主导形态，vue_capabilities
        // 双轨实测发现的 face 槽位别名缺口，UI 侧同长归一化承载）。
        let src_norm = normalize_dollar_event(&normalize_state_slots(src));
        let ds_src = ace_core::source::fstr::desugar_fstrings_widget(&src_norm)
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
        let compiled = ace_av::age_component::compile_component_scenario_states_at(
            src,
            path,
            ace_av::age_component::LegacyWidgetAlias::Accept,
            &caps,
            &ace_av::age_component::ComponentScenario { instances: 1, ops },
            &ace_av::age_component::ComponentInitials {
                per_instance: vec![Vec::new()],
            },
            node_index,
        )
        .map_err(|ds| {
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        Ok(Self {
            ds_src,
            // PLAN-980 WEB #6p：通道事实随描述出口透出（selected node；
            // 单 node 形——`compile_at` 已选一编译）。n_assigns 取
            // `handler_assigns`（语言侧投影的赋值语句计数）。
            channel_facts: compiled
                .description
                .nodes
                .iter()
                .flat_map(|n| {
                    n.handler_channel_calls
                        .iter()
                        .map(|(msg, rows, foreign)| LaneHandlerFacts {
                            msg: msg.clone(),
                            rows: rows.clone(),
                            body_foreign: *foreign,
                            n_assigns: n
                                .handler_assigns
                                .iter()
                                .find(|(m, _)| m == msg)
                                .map(|(_, a)| a.len())
                                .unwrap_or(0),
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
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

    /// PLAN-980 WEB #6p：按 msg 名取该 handler 的**通道调用行**（脱糖源
    /// 坐标；`msg` = pattern 去点）。行来自语言侧投影（callee=widget 头
    /// 参数的调用语句）；实参 span 供 `emit_expr_at_span` 精确发射——
    /// 实参已在语言侧经检查函数**死 let** 验证（名字/类型/ownership），
    /// 未验证拼写绝不进入拼接体。
    pub fn channel_rows_for(&self, msg: &str) -> Vec<LaneChannelRow> {
        self.facts(msg)
            .map(|f| {
                f.rows
                    .iter()
                    .map(|cc| LaneChannelRow {
                        channel: cc.channel.clone(),
                        arg_spans: cc.args.iter().map(|a| a.span.start..a.span.end).collect(),
                        span: cc.span.start..cc.span.end,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// PLAN-980 WEB #6p：按 msg 名取该 handler 的**域外体标记**
    ///（body_foreign——let/if/非头参数调用语句：检查函数不铸、lane 必须
    /// 缺席，不得部分拼接）。
    pub fn handler_body_foreign(&self, msg: &str) -> bool {
        self.facts(msg).map(|f| f.body_foreign).unwrap_or(false)
    }

    /// #6p：按 msg 名取赋值语句数（语言侧投影；纯通道体=0）。
    pub fn handler_assign_count(&self, msg: &str) -> usize {
        self.facts(msg).map(|f| f.n_assigns).unwrap_or(0)
    }

    fn facts(&self, msg: &str) -> Option<&LaneHandlerFacts> {
        self.channel_facts.iter().find(|f| f.msg == msg)
    }

    /// checked 模块（view 构建用；调用方自建 SemanticView）。
    pub fn checked_module(&self) -> &ace_core::verify::CheckedModule {
        &self.checked
    }

    /// 实例 0 的状态函数名清单（`__age_state_0_<field>`；字段名=后缀）。
    pub fn state_fns(&self) -> Vec<String> {
        ace_a2x_core::SemanticView::from_checked(&self.checked)
            .map(|v| {
                v.functions()
                    .iter()
                    .filter(|f| f.source_name.starts_with("__age_state_0_"))
                    .map(|f| f.source_name.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// PLAN-981（IF-981-PARAMS-B，B-W1）**声明级检查函数**发现：组件面为
    /// 每个 handler 铸 `__age_hv_<n>_<h>_<pos>` 形参=真 handler 形参的检
    /// 查函数（载荷类型来自声明拼写；不可承载载荷如 float 不铸——fail-
    /// closed）。param handler 的 lane 发射源=该函数（无场景重放、无代表
    /// 值——参数是**真函数参数**，宿主按位映射到 SFC 形参名）。
    ///
    /// 匹配合同：检查函数体=字段 init（model 块 span）+ handler 赋值 RHS
    /// （**原源 span**，重命名保 span）+ return（首字段 span）——body 表达
    /// 式与 handler 体区间相交者唯一归属该 handler。
    pub fn check_fn_covering(&self, region: std::ops::Range<usize>) -> Option<String> {
        self.hidden_fn_covering("__age_hv_", region)
    }

    /// 切片#6k（V-EVT-B-ORIGIN）：**view 内联 lambda 值函数**发现
    /// （`__age_vl_<seq>`——语言侧铸造，赋值 RHS 原源 span；同检查函数的
    /// 区间相交匹配合同）。inline-lambda 铸造 handler（`__evt_*`）的 lane
    /// 发射源=该函数（真实 body/origin，无假 on 块/默认参数）。
    pub fn lambda_fn_covering(&self, region: std::ops::Range<usize>) -> Option<String> {
        self.hidden_fn_covering("__age_vl_", region)
    }

    /// 隐藏铸造函数族按区间相交发现（body 表达式与区间相交者唯一归属）。
    fn hidden_fn_covering(&self, prefix: &str, region: std::ops::Range<usize>) -> Option<String> {
        let view = ace_a2x_core::SemanticView::from_checked(&self.checked).ok()?;
        view.functions()
            .iter()
            .filter(|f| f.source_name.starts_with(prefix))
            .find(|f| {
                f.body
                    .exprs
                    .iter()
                    .any(|e| e.span.start < region.end && region.start < e.span.end)
            })
            .map(|f| f.source_name.to_string())
    }

    /// 切片#6l（真语料全量双轨）：区间内**逐赋值语句**发射（多赋值 handler
    /// 体——[57] 七连赋值族）。过滤合同：HIR `Stmt` 无 span，按 **Assign 的
    /// RHS 表达式 span ⊆ region** 选取（字段 init let 的 RHS span 在 model
    /// 块=区间外；return 的 var 读 span=首字段声明=区间外），语句序=span
    /// 起点序（源序）。语法层只收平铺 `.field (=|+=|-=) expr` 语句
    /// （`age-view-stmt`）——控制流/let/调用在**语法层被拒**，不可能被静默
    /// 展平，发射语义安全。空结果=区间内无赋值（空体/纯调用族——调用方判
    /// 缺席）。
    pub fn stmt_fragments_in(
        &self,
        fn_name: &str,
        region: std::ops::Range<usize>,
    ) -> Result<Vec<(std::ops::Range<usize>, LocatedFragment)>, String> {
        use ace_a2x_ts::{emit_stmt_fragment, FragmentProgramContext, ScriptLang};
        let view = ace_a2x_core::SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name == fn_name)
            .ok_or_else(|| format!("fn `{fn_name}` not in the view"))?;
        // (RHS span, block, idx) —— 源序发射；#6p：span 随片段返回（与通道
        // 行按源序 merge，调用/赋值相对位置不重排）。
        let mut hits: Vec<(std::ops::Range<usize>, u32, usize)> = Vec::new();
        for (bi, block) in facts.body.blocks.iter().enumerate() {
            for (si, stmt) in block.stmts.iter().enumerate() {
                if let ace_core::hir::Stmt::Assign { value, .. } = stmt {
                    if let Some(e) = facts.body.exprs.get(value.0 as usize) {
                        if region.start <= e.span.start && e.span.end <= region.end {
                            hits.push((e.span.start..e.span.end, bi as u32, si));
                        }
                    }
                }
            }
        }
        hits.sort_by_key(|(span, _, _)| span.start);
        let ctx = FragmentProgramContext::from_view(&view);
        let mut out = Vec::new();
        for (span, block, idx) in hits {
            let raw = emit_stmt_fragment(ScriptLang::TypeScript, &view, facts, block, idx, &ctx)
                .map_err(|e| format!("stmt fragment: {e:?}"))?;
            out.push((
                span,
                LocatedFragment {
                    text: raw.text,
                    expr: None,
                    helpers: raw.helpers,
                },
            ));
        }
        Ok(out)
    }

    /// 区间内**最宽**表达式发射（span ⊆ region；同宽取索引最大=组合形）。
    /// 区间无表达式 → None（调用方判缺席）。具名拒：状态函数不在视图。
    pub fn widest_expr_in(
        &self,
        state_fn: &str,
        region: std::ops::Range<usize>,
    ) -> Result<Option<LocatedFragment>, String> {
        use ace_a2x_ts::{emit_expr_fragment, FragmentProgramContext, ScriptLang};
        let view = ace_a2x_core::SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name == state_fn)
            .ok_or_else(|| format!("state fn `{state_fn}` not in the view"))?;
        let mut best: Option<(usize, usize, ace_core::hir::ExprId)> = None; // (width, idx, id)
        for (i, e) in facts.body.exprs.iter().enumerate() {
            if region.start <= e.span.start && e.span.end <= region.end {
                let width = e.span.end.saturating_sub(e.span.start);
                match best {
                    Some((w, _, _)) if w > width => {}
                    _ => best = Some((width, i, ace_core::hir::ExprId(i as u32))),
                }
            }
        }
        let Some((_, _, expr_id)) = best else {
            return Ok(None);
        };
        let ctx = FragmentProgramContext::from_view(&view);
        let raw = emit_expr_fragment(ScriptLang::TypeScript, &view, facts, expr_id, &ctx)
            .map_err(|e| format!("expr fragment: {e:?}"))?;
        Ok(Some(LocatedFragment {
            text: raw.text,
            expr: raw.expr,
            helpers: raw.helpers,
        }))
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
