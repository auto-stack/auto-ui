//! PLAN-980 WEB 切片#6c（路线 B 核心）：lane 派生 **handler 函数体**。
//!
//! 输入=widget 源文本 + handler pattern 清单；输出=每个 handler 的 Vue
//! script 函数体行（reactive 宿主语义：`field.value = <rhs>`）。算法：
//! 组件面 lane（states-only，ops=全部 handler msg 重放）→ 按 pattern 文本
//! 锚定位 handler 体区间（`-> {` 括号配对）→ 逐状态函数找区间内**最宽**
//! 表达式（赋值 RHS——复合/NOT/字面量）→ 发射 + 宿主映射（发射局部名
//! `l<i>___age_s_<inst>_<field>` → `field.value`，字段名从发射名自动提取）
//! → 组装 `[helper 行..., field.value = expr;]`。写目标字段=命中区间的
//! 状态函数（自动推导）。未命中区间的 handler=具名缺席（调用方决定旧道）。
//!
//! 切片#6j（PLAN-981 B-W1 消费）：**param handler 走声明级检查函数**
//! （`__age_hv_*`，形参=真 handler 形参，载荷类型来自声明拼写）——不做
//! 场景重放、不造代表值；ops 只含 unit handler（`Send` 对载荷变体=arity
//! 域错误，981 IF-981-EVENT-PAYLOAD-B）。不可承载载荷（float 等）不铸检
//! 查函数=具名缺席（fail-closed，旧道承载），不造假数值。
//!
//! 边界（具名）：本模块是路线 B 的**体级发射核心**，不含 SFC 拼接/默认
//! 切换（#6d）；一行单赋值 handler 形态（002 族）为已验面——多语句体/
//! 局部 var/控制流体的区间-最宽启发式不适用者具名拒绝（旧道承载）。

use crate::ui::ace_ts_provider::ComponentFragmentLane;

/// 一个 handler 的 lane 派生函数体（已做 reactive 宿主映射）。
#[derive(Clone, Debug)]
pub struct LaneHandlerBody {
    /// handler pattern（如 `.Inc`）。
    pub pattern: String,
    /// 写目标字段（如 `count`）。
    pub field: String,
    /// 函数体行（helper 行在前，赋值行在后；每行已带分号语义由调用方包装）。
    pub lines: Vec<String>,
    /// PLAN-981 PARAMS-B：按位形参名（SFC 函数签名侧；空=unit handler）。
    pub params: Vec<String>,
}

/// lane 发射的 handler 输入（切片#6j）：pattern + **按位形参名**（来自
/// widget `handler_params`——SFC 函数签名同名；空=unit handler 走状态函
/// 数重放道）。切片#6k：`lambda_region`（view 内联 lambda 铸造 handler
/// `__evt_*` 专用）——源无 `->` 锚，区间由调用方从 `=> {` 定位，走
/// `__age_vl_*` 值函数发射道（真实 body/origin）。
#[derive(Clone, Debug)]
pub struct LaneHandlerSpec<'a> {
    /// handler pattern（如 `.Add`；`__evt_*` 铸造形亦可）。
    pub pattern: &'a str,
    /// 按位形参名（如 `["t"]`）；空=unit。
    pub params: &'a [String],
    /// 切片#6k：预解析 lambda 体区间（脱糖源坐标）；Some=`__age_vl_` 道。
    pub lambda_region: Option<std::ops::Range<usize>>,
}

/// PLAN-980 WEB #6p：通道调用发射上下文（方言侧预扫描）——
/// - `op_exempt`：handler 体含**调用语句**（裸 `f(...)` 表达式语句）的 msg
///   名（去点）。这些 unit handler **不进 ops**：通道 handler（callee=头
///   参数）走检查函数道（语言侧场景重放对通道零效果具名拒——重放车道
///   无宿主）；域外调用 handler（body_foreign）编译期不触发 ops 拒——
///   两者的**兄弟 handler 不被连带缺席**（此前任一调用体进 ops = 整 lane
///   编译失败 = 全体缺席）。
/// - `variants`：本 widget 的 msg 变体名（emit 形判定：`on_x` 去 `on_` 前
///   缀的 Pascal 形 ∈ variants → `emit('<Pascal>', args)`（契约事件，
///   defineEmits 已声明）；否则 `props.on_x(args)`（回调 prop——老道
///   PLAN-668 R-23/Plan 444 同约定，SFC 侧 defineProps 承载）。
#[derive(Clone, Debug, Default)]
pub struct LaneChannelCtx {
    pub op_exempt: Vec<String>,
    pub variants: Vec<String>,
}

/// 并发发射的宿主映射：发射局部名（`l<i>___age_s_<inst>_<field>` 或裸
/// 字段引用）→ `field.value`。词法整词替换（沿 ace_ts_provider 口径）。
fn map_emitted(line: &str) -> String {
    map_emitted_with(line, &[])
}

/// 切片#6j：param handler 版宿主映射。两族发射名：
/// - `l<i>___age_s_<inst>_<field>` → `field.value`（状态字段，同上）；
/// - `p<i>___age_hv_<n>_<h>_<pos>` → **按位 SFC 形参名**（`params[pos]`——
///   声明级检查函数的真函数参数；无该位形参=保持原样（后续 splice 拒）。
fn map_emitted_with(line: &str, params: &[String]) -> String {
    let mut out = String::with_capacity(line.len());
    let mut word = String::new();
    let flush = |out: &mut String, word: &mut String| {
        if !word.is_empty() {
            // `l2___age_s_0_dark_mode` → `dark_mode.value`：`___age_s_` 后
            // 是 `<inst>_<field>`——剥实例段取字段名。
            let mapped = match word.rfind("___age_s_") {
                Some(at) => {
                    let suffix = &word[at + "___age_s_".len()..];
                    match suffix.split_once('_') {
                        Some((_, field)) => format!("{field}.value"),
                        None => word.clone(),
                    }
                }
                // `p0___age_hv_0_2_0` → 形参名（pos=尾段）：981 检查函数
                // 参数按位 ↔ SFC 函数签名形参。
                None => match word.rfind("___age_hv_") {
                    Some(at) => {
                        let suffix = &word[at + "___age_hv_".len()..];
                        let pos: Option<usize> =
                            suffix.rsplit('_').next().and_then(|p| p.parse().ok());
                        match pos.and_then(|p| params.get(p)) {
                            Some(name) => name.clone(),
                            None => word.clone(),
                        }
                    }
                    None => word.clone(),
                },
            };
            out.push_str(&mapped);
            word.clear();
        }
    };
    for ch in line.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            flush(&mut out, &mut word);
            out.push(ch);
        }
    }
    flush(&mut out, &mut word);
    out
}

/// 切片#6l：k>0 片段的临时名整词改号（`t3` → `t2_3`）——多片段同域拼
/// 接防 `const t0` 重复声明。片段文本里裸 `t<digits>` 只可能是发射器临时
/// 名（其它名字族走 `l<i>_*`/`p<i>_*`/宿主映射名），整词边界安全。
fn rename_temps(line: &str, k: usize) -> String {
    let mut out = String::with_capacity(line.len());
    let mut word = String::new();
    let flush = |out: &mut String, word: &mut String| {
        if !word.is_empty() {
            let renamed = match word
                .strip_prefix('t')
                .and_then(|rest| rest.parse::<usize>().ok())
            {
                Some(i) => format!("t{k}_{i}"),
                None => word.clone(),
            };
            out.push_str(&renamed);
            word.clear();
        }
    };
    for ch in line.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            flush(&mut out, &mut word);
            out.push(ch);
        }
    }
    flush(&mut out, &mut word);
    out
}

/// handler 体区间：pattern 锚 → 其后第一个 `->` 的 `{` 起做括号配对。
fn handler_body_region(ds: &str, pattern: &str) -> Result<std::ops::Range<usize>, String> {
    // 锚定=pattern 后**紧邻**箭头的出现（view 内联引用 `.ToggleMode,` 无
    // 箭头——排除；002 族无参 pattern）。逐出现检查至首个合格位。
    let mut search_from = 0usize;
    let mut p_pos = None;
    while let Some(rel) = ds[search_from..].find(pattern) {
        let pos = search_from + rel;
        let after = &ds[pos + pattern.len()..];
        let trimmed = after.trim_start();
        // #6i：带参 handler——跳过 `(param, …)` 段后再查箭头。
        let after_params = if trimmed.starts_with('(') {
            match trimmed.find(')') {
                Some(close) => trimmed[close + 1..].trim_start(),
                None => trimmed,
            }
        } else {
            trimmed
        };
        if after_params.starts_with("->") {
            p_pos = Some(pos);
            break;
        }
        search_from = pos + pattern.len();
    }
    let p_pos =
        p_pos.ok_or_else(|| format!("pattern {pattern:?} with a following arrow not found"))?;
    let arrow = ds[p_pos..]
        .find("-> ")
        .map(|a| p_pos + a)
        .ok_or_else(|| format!("no `->` after {pattern:?}"))?;
    let open = ds[arrow..]
        .find('{')
        .map(|b| arrow + b)
        .ok_or_else(|| format!("no `{{` after {pattern:?} arrow"))?;
    let mut depth = 0usize;
    let bytes = ds.as_bytes();
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(open..i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(format!("unbalanced braces for {pattern:?} body"))
}

/// 逐 handler 发射 lane 派生函数体（unit 形——patterns 无参）。切片#6j
/// 起为 [`lane_handler_bodies_spec`] 的薄包装。
pub fn lane_handler_bodies(
    src: &str,
    path: &str,
    patterns: &[&str],
) -> Result<(Vec<LaneHandlerBody>, Vec<String>), String> {
    let specs: Vec<LaneHandlerSpec<'_>> = patterns
        .iter()
        .map(|p| LaneHandlerSpec {
            pattern: p,
            params: &[],
            lambda_region: None,
        })
        .collect();
    lane_handler_bodies_spec(src, path, &specs)
}

/// 切片#6j：逐 handler 发射（param 感知）。unit handler=状态函数重放道
/// （ops=`Send`）；**param handler=981 声明级检查函数道**（`__age_hv_*`——
/// 不进 ops：`Send` 对载荷变体=arity 域错误；检查函数形参=真 handler 形
/// 参，发射后按位映射到 SFC 形参名）。切片#6k：`lambda_region` 形走
/// `__age_vl_*` 值函数道（view 内联 lambda——真实 body/origin）。未命中
/// （无检查/值函数=不可承载载荷如 float；或无区间表达式）具名缺席——调
/// 用方按合同决定旧道或拒绝。
pub fn lane_handler_bodies_spec(
    src: &str,
    path: &str,
    specs: &[LaneHandlerSpec<'_>],
) -> Result<(Vec<LaneHandlerBody>, Vec<String>), String> {
    lane_handler_bodies_spec_at(src, path, specs, 0, &LaneChannelCtx::default())
}

/// 切片#6n：node 选择器形态（多 widget 源——lane `compile_at`）。
/// PLAN-980 WEB #6p：`channel_ctx`（方言侧通道预扫描）——通道 handler 从
/// ops 排除（语言侧场景重放对通道零效果具名拒）并走检查函数道发射。
pub fn lane_handler_bodies_spec_at(
    src: &str,
    path: &str,
    specs: &[LaneHandlerSpec<'_>],
    node_index: usize,
    channel_ctx: &LaneChannelCtx,
) -> Result<(Vec<LaneHandlerBody>, Vec<String>), String> {
    // ops 只含 unit handler（msg 无点前缀，ScenarioOp::Send 合同——载荷
    // 变体的空 Send 是 981 arity 域错误，不进重放；lambda 形不是 on 块
    // handler，同样不进 ops）。#6p：**含调用语句的 handler 也不进 ops**
    //（通道=重放车道无宿主、效应不可丢；域外调用=编译期 ops 拒会连带
    // 兄弟全体缺席——豁免后通道走检查函数道、域外走缺席，兄弟保全）。
    let msgs: Vec<&str> = specs
        .iter()
        .filter(|s| s.params.is_empty() && s.lambda_region.is_none())
        .map(|s| s.pattern.trim_start_matches('.'))
        .filter(|m| !channel_ctx.op_exempt.iter().any(|c| c == m))
        .collect();
    let lane = ComponentFragmentLane::compile_at(src, path, &msgs, node_index)?;
    let ds = lane.desugared_source().to_string();
    let state_fns = lane.state_fns();

    let mut bodies = Vec::new();
    let mut absent = Vec::new();
    for spec in specs {
        let pat = spec.pattern;
        let msg = pat.trim_start_matches('.');
        // PLAN-980 WEB #6p：域外体（let/if/非头参数调用——语言侧 body_foreign）
        // → 具名缺席（检查函数不铸，不做部分体拼接；旧道承载）。
        if lane.handler_body_foreign(msg) {
            absent.push((*pat).to_string());
            continue;
        }
        // #6p：通道调用行（语言侧投影权威；callee=头参数判定在语言面）。
        let rows = lane.channel_rows_for(msg);
        let region = match spec.lambda_region.clone() {
            Some(r) => r,
            None => match handler_body_region(&ds, pat) {
                Ok(r) => r,
                Err(_) => {
                    absent.push((*pat).to_string());
                    continue;
                }
            },
        };
        // 写目标=体文本 LHS 首个 `.field`（赋值形 handler 家族——002 族；
        // 三条发射道共用；具名边界：非单赋值形归 #6d 结构分析）。#6p 起
        // 可缺席——纯通道体（无赋值语句）无字段写。
        let region_text = &ds[region.clone()];
        let write_field = region_text.find('.').and_then(|d| {
            let rest = &region_text[d + 1..];
            let end = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            (end > 0).then(|| rest[..end].to_string())
        });
        // 发射源：lambda=值函数（#6k）；**通道 handler=检查函数道**（#6p，
        // unit 亦然——其效果是宿主通知，状态函数重放道无此体）；param=
        // 声明级检查函数（981 B-W1）；unit=状态函数（ops 重放体）。
        let fn_name: Option<String> = if spec.lambda_region.is_some() {
            match lane.lambda_fn_covering(region.clone()) {
                Some(f) => Some(f),
                None => {
                    absent.push((*pat).to_string());
                    continue;
                }
            }
        } else if !rows.is_empty() {
            lane.check_fn_covering(region.clone())
        } else if spec.params.is_empty() {
            let Some(wf) = write_field.clone() else {
                absent.push((*pat).to_string());
                continue;
            };
            let f = format!("__age_state_0_{wf}");
            if !state_fns.contains(&f) {
                absent.push((*pat).to_string());
                continue;
            }
            Some(f)
        } else {
            match lane.check_fn_covering(region.clone()) {
                Some(f) => Some(f),
                // 无检查函数=载荷不可承载（float/ident 等，981 派发守卫同
                // 族 fail-closed）——不造假数值，具名缺席（旧道承载）。
                None => {
                    absent.push((*pat).to_string());
                    continue;
                }
            }
        };
        // #6p：通道 handler 需要函数承载赋值/实参发射（实参经检查函数死
        // let 验证后按精确 span 发射）；零赋值+零实参的纯通知行可无函数
        //（n_assigns=语言侧投影计数，非区间文本启发式）。
        let n_assigns = lane.handler_assign_count(msg);
        let needs_fn = n_assigns > 0 || rows.iter().any(|r| !r.arg_spans.is_empty());
        let fn_name = match fn_name {
            Some(f) => f,
            None if !needs_fn => String::new(),
            None => {
                absent.push((*pat).to_string());
                continue;
            }
        };
        // 切片#6l：**逐赋值语句发射**（多赋值体——语句序=源序；单赋值族同
        // 形：Assign 语句发射含 helper 行 + 赋值行，与旧手拼形态一致）。
        // 区间无赋值（空体）=具名缺席；**纯通道体例外**（#6p：无赋值语句、
        // 只有通道行）。**临时名按片段改号**：每片段的发射器临时名 `t<i>`
        // 从 0 起，多片段同域拼接会撞声明——片段文本里裸 `t<digits>` 只可
        // 能是临时名（局部/参数走 `l<i>_*`/`p<i>_*` 发射名或宿主映射名），
        // k>0 片段整词改号为 `t{k}_{i}`（单片段形状不变，既有断言/浏览器
        // 内联兼容）。
        let stmts = if fn_name.is_empty() {
            Vec::new()
        } else {
            lane.stmt_fragments_in(&fn_name, region.clone())?
        };
        if stmts.is_empty() && rows.is_empty() {
            absent.push((*pat).to_string());
            continue;
        }
        // #6p：赋值片段与通道行按**源 span 序 merge**（调用语句与赋值语句
        // 的相对位置保持源序——`.n = next; on_select(next)` 与倒序形都不
        // 重排）。通道行=emit 契约事件形或 props 回调形（老道 PLAN-668
        // R-23/Plan 444 同约定：`on_x` 去 on_ 的 Pascal 形 ∈ msg 变体 →
        // `emit('<Pascal>', args)`；否则 `props.on_x(args)`——SFC 侧
        // defineEmits/defineProps 承载声明）。
        enum Ev {
            Assign(Vec<String>),
            Channel(String),
        }
        let mut events: Vec<(usize, Ev)> = Vec::new();
        for (k, (span, frag)) in stmts.iter().enumerate() {
            let mut block = Vec::new();
            for l in &frag.text {
                let mapped = map_emitted_with(l, spec.params);
                if k == 0 {
                    block.push(mapped);
                } else {
                    block.push(rename_temps(&mapped, k));
                }
            }
            events.push((span.start, Ev::Assign(block)));
        }
        for row in &rows {
            let mut args: Vec<String> = Vec::new();
            let mut ok = true;
            for sp in &row.arg_spans {
                match lane.emit_expr_at_span(&fn_name, sp.clone()) {
                    Ok(frag) => match frag.expr {
                        Some(e) => args.push(map_emitted_with(&e, spec.params)),
                        None => {
                            ok = false;
                            break;
                        }
                    },
                    Err(_) => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                absent.push((*pat).to_string());
                continue;
            }
            let pascal = channel_pascal(&row.channel);
            let call = if channel_ctx.variants.iter().any(|v| *v == pascal) {
                if args.is_empty() {
                    format!("emit('{pascal}')")
                } else {
                    format!("emit('{}', {})", pascal, args.join(", "))
                }
            } else if args.is_empty() {
                format!("props.{}()", row.channel)
            } else {
                format!("props.{}({})", row.channel, args.join(", "))
            };
            events.push((row.span.start, Ev::Channel(format!("{call};"))));
        }
        events.sort_by_key(|(start, _)| *start);
        let mut lines: Vec<String> = Vec::new();
        for (_, ev) in events {
            match ev {
                Ev::Assign(block) => lines.extend(block),
                Ev::Channel(line) => lines.push(line),
            }
        }
        // 976 片段接口缺口（#6l 实测）：owned 局部的赋值/返回语句片段引用
        // 函数域 skip 簿记变量 `a2x_ret_owned_skip`——其 `let` 声明只在整函
        // 数发射路径（lower 整体 1474 行）里，片段文本裸引用。宿主边界统
        // 一补声明（引用才补；SFC 拼接/node/浏览器同治）。
        if lines.iter().any(|l| l.contains("a2x_ret_owned_skip")) {
            lines.insert(0, "let a2x_ret_owned_skip = null;".to_string());
        }
        // 复核 F-1（#6j/#6k 批级复核，authored 反例发现）：形参名与字段名
        // 同拼写碰撞——宿主映射 `field.value` 的**基名**会被 SFC 函数形参遮
        // 蔽（PARAMS-B「裸形参遮蔽字段」语义在 TS 宿主不可表达；node 实证
        // `n.value = n` 在形参数字上写属性=TypeError）→ 具名缺席（fail-
        // closed 旧道承载），不生成错误体。
        let mapped_text = lines.join("\n");
        if spec
            .params
            .iter()
            .any(|p| mapped_text.contains(&format!("{p}.value")))
        {
            absent.push((*pat).to_string());
            continue;
        }
        bodies.push(LaneHandlerBody {
            pattern: (*pat).to_string(),
            field: write_field.unwrap_or_default(),
            lines,
            params: spec.params.iter().map(|p| p.to_string()).collect(),
        });
    }
    Ok((bodies, absent))
}

/// PLAN-980 WEB #6p：`on_<snake>` → Pascal 事件名（`on_select` → `Select`；
/// 老道 `VueGenerator::snake_to_pascal` 同规则——emit 契约事件判定用）。
fn channel_pascal(channel: &str) -> String {
    channel
        .strip_prefix("on_")
        .unwrap_or(channel)
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut cs = s.chars();
            match cs.next() {
                Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// 切片 #6d：**SFC 拼接**——lane 派生体替换编译器 SFC 内对应 handler 函数
/// 体（锚=`function <Variant>(): void {`…配对 `}`；pattern 去点=Variant）。
/// 只动 handler 函数体；其余段落字节不变。Variant 缺席/锚未命中=具名拒。
pub fn splice_lane_handler_bodies(sfc: &str, bodies: &[LaneHandlerBody]) -> Result<String, String> {
    let mut out = sfc.to_string();
    for b in bodies {
        let variant = b.pattern.trim_start_matches('.');
        // #6i：带参 SFC 函数 `function Add(t: string): void {`——锚=
        // `function <Variant>(`，再找 `): void {`（零参 `(): void {` 同形）。
        let fn_prefix = format!("function {variant}(");
        let fn_start = out
            .find(&fn_prefix)
            .ok_or_else(|| format!("handler fn `{variant}` not found in the SFC"))?;
        let brace_after = out[fn_start..]
            .find("): void {")
            .map(|b| fn_start + b + "): void {".len())
            .ok_or_else(|| format!("handler fn `{variant}` signature tail not found"))?;
        let start = fn_start;
        let open = brace_after;
        // 括号配对找函数体闭括号。
        let bytes = out.as_bytes();
        let mut depth = 1usize;
        let mut i = open;
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if depth != 0 {
            return Err(format!("unbalanced braces in `{variant}` fn"));
        }
        let body_text = b
            .lines
            .iter()
            .map(|l| format!("  {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        out.replace_range(open..i, format!("\n{body_text}\n").as_str());
    }
    Ok(out)
}

/// 切片 #6e：**一体化入口**——源文本 → lane SFC（编译器管线 SFC + lane
/// handler 体拼接）。返回 `(sfc, lane_bodies, absent)`：absent 中的 handler
/// 保持编译器旧道体（双轨期合同——缺席不拒绝，产品默认道仍为编译器）。
pub fn generate_sfc_with_lane(
    src: &str,
    path: &str,
) -> Result<(String, Vec<LaneHandlerBody>, Vec<String>), String> {
    use auto_ui_compiler::ui_gen::BackendGenerator;
    use auto_ui_dialect::UiStmt;
    // 真实编译器管线。切片#6n：**多 widget 同文件源**——按声明序逐 widget
    // 生成并聚合（lane 经 node 选择器 `compile_at(i)`；单 widget 源行为不
    // 变）。返回值=首个 SFC + 全部 widget 的 bodies/absent 聚合。
    let session = auto_parser_api::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .map_err(|e| format!("parse: {e:?}"))?;
    let decls: Vec<&auto_ui_dialect::ast::WidgetDecl> = ast
        .domain_stmts()
        .filter_map(|s| match s {
            UiStmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .collect();
    if decls.is_empty() {
        return Err("no widget decl".to_string());
    }
    let (mut all_bodies, mut all_absent): (Vec<LaneHandlerBody>, Vec<String>) =
        (Vec::new(), Vec::new());
    let mut first_sfc: Option<String> = None;
    for (wi, decl) in decls.iter().enumerate() {
        let (sfc_w, bodies_w, absent_w) = generate_widget_with_lane(decl, src, path, wi)?;
        if first_sfc.is_none() {
            first_sfc = Some(sfc_w);
        }
        all_bodies.extend(bodies_w);
        all_absent.extend(absent_w);
    }
    let _ = &decls;
    return Ok((first_sfc.expect("non-empty"), all_bodies, all_absent));

    #[allow(unreachable_code)]
    {
        let decl = decls[0];
        let _ = decl;
        unreachable!("multi-widget path returns above")
    }
}

/// 切片#6n：单 widget 生成（`generate_sfc_with_lane` 的逐节点体——SFC 管
/// 线 + lane 体，node 选择器=decls 中的声明序）。
fn generate_widget_with_lane(
    decl: &auto_ui_dialect::ast::WidgetDecl,
    src: &str,
    path: &str,
    node_index: usize,
) -> Result<(String, Vec<LaneHandlerBody>, Vec<String>), String> {
    use auto_ui_dialect::UiStmt;
    let widget = auto_ui_compiler::aura::extract_widget_from_decl(decl)
        .map_err(|e| format!("extract: {e:?}"))?;
    let sfc = auto_ui_compiler::ui_gen::VueGenerator::new()
        .generate_sfc(&widget)
        .map_err(|e| format!("SFC: {e:?}"))?;
    // lane 体（specs=widget handler 键去参段——`.AddItem(text)` 形取
    // `.AddItem`+按位形参名（`handler_params` 同键族）；缺席者保旧道）。
    let patterns: Vec<String> = widget
        .handlers
        .keys()
        .map(|k| k.split('(').next().unwrap_or(k).to_string())
        .collect();
    let param_names: Vec<Vec<String>> = patterns
        .iter()
        .map(|base| {
            widget
                .handler_params
                .iter()
                .find(|(k, _)| match k.find('(') {
                    Some(i) if k.ends_with(')') => &k[..i] == base.as_str(),
                    _ => k.as_str() == base.as_str(),
                })
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        })
        .collect();
    let specs: Vec<LaneHandlerSpec<'_>> = patterns
        .iter()
        .zip(param_names.iter())
        .map(|(p, ps)| LaneHandlerSpec {
            pattern: p,
            params: ps,
            lambda_region: None,
        })
        .collect();
    // 切片#6k（V-EVT-B-ORIGIN）：`__evt_*`（view 内联 lambda 铸造 handler）
    // 区间配对——脱糖源 `=> {` 体区间按序 ↔ 带体 `__evt_*` handler 按铸造
    // 计数序（方言 DFS=源文本序；空体铸造（bare-input sync/空 lambda）无
    // 区间、跳过=旧道）。计数失配（区间数 ≠ 带体 handler 数）=全体缺席
    //（fail-closed，不猜测配对）。
    let specs = pair_lambda_regions(src, &widget, specs);
    // PLAN-980 WEB #6p：通道上下文（方言侧预扫描）——
    // - `op_exempt`：handler 体含**调用语句**（裸 `f(...)` 表达式语句）的
    //   msg（去点）。通道 handler（callee ∈ 头参数）走检查函数道；域外
    //   调用 handler（body_foreign）缺席；两者都不进 ops——兄弟保全。
    // - `variants`：msg 变体名（emit 形判定：`on_x` 的 Pascal 形 ∈ 变体
    //   → `emit('<Pascal>', args)` 契约事件；否则 `props.on_x(args)` 回调
    //   prop——老道 PLAN-668 R-23/Plan 444 同约定，SFC 侧声明机制承载）。
    let mut op_exempt: Vec<String> = Vec::new();
    for (pattern, payload) in &widget.handlers {
        if let auto_ui_compiler::aura::LogicPayload::AstStmts(stmts) = payload {
            let has_call = stmts.iter().any(|s| {
                use auto_lang_core::ast::{Expr, Stmt};
                matches!(s, Stmt::Expr(Expr::Call(_)))
            });
            if has_call {
                let base = pattern.split('(').next().unwrap_or(pattern);
                op_exempt.push(base.trim_start_matches('.').to_string());
            }
        }
    }
    let variants: Vec<String> = widget
        .messages
        .iter()
        .flat_map(|m| m.variants.iter().map(|v| v.name.clone()))
        .collect();
    let channel_ctx = LaneChannelCtx {
        op_exempt,
        variants,
    };
    // 切片#6n：node 选择器——lane 编译选本 widget（多 widget 源非本节点
    // handler 的 ops/区间自然落空=缺席）。
    let (bodies, absent) =
        match lane_handler_bodies_spec_at(src, path, &specs, node_index, &channel_ctx) {
            Ok(v) => v,
            Err(_) => (Vec::new(), patterns.clone()),
        };
    let spliced = if bodies.is_empty() {
        sfc
    } else {
        splice_lane_handler_bodies(&sfc, &bodies)?
    };
    Ok((spliced, bodies, absent))
}

/// 切片#6k：脱糖源里的 view 内联 lambda 体区间清单（`=> {` …配对 `}`，
/// 源文本序=方言铸造 DFS 序）。on 块 handler 用 `->`，不与此锚相撞。
fn lambda_body_regions(ds: &str) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let bytes = ds.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = ds[from..].find("=>") {
        let arrow = from + rel;
        from = arrow + 2;
        // 箭头后（跳空白）须紧跟 `{`——lambda 体开括号；括号配对到闭括号。
        let mut open = None;
        let mut i = arrow + 2;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'{' {
            open = Some(i);
        }
        let Some(open) = open else { continue };
        let mut depth = 0usize;
        let mut j = open;
        while j < bytes.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        out.push(open..j + 1);
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
    }
    out
}

/// `__evt_*` pattern → 铸造计数（尾段 `_n`）；非 `__evt_*` 形=None。
fn evt_mint_counter(pattern: &str) -> Option<u32> {
    let base = pattern.strip_prefix(".__evt_")?;
    let (_, n) = base.rsplit_once('_')?;
    n.parse().ok()
}

/// 配对：源 lambda 区间（序）↔ 带体 `__evt_*` handler（铸造计数序）。
/// 返回带 `lambda_region` 的 specs；失配=全部 `__evt_*` 留 None（缺席）。
fn pair_lambda_regions<'a>(
    src: &str,
    widget: &auto_ui_compiler::aura::AuraWidget,
    mut specs: Vec<LaneHandlerSpec<'a>>,
) -> Vec<LaneHandlerSpec<'a>> {
    // 区间=**脱糖源坐标**（与 lane 同一基准——经公共出口重扫 `=>`；源含
    // f-string 脱糖位移时配对方与 lane 不漂移）。计数失配=fail-closed。
    let ds =
        crate::ui::ace_ts_provider::desugared_lane_source(src).unwrap_or_else(|_| src.to_string());
    let regions = lambda_body_regions(&ds);
    // 带体 __evt_* handler：铸造计数序（空体=bare-input sync/空 lambda，
    // 无区间不配对）。
    let mut minted: Vec<(u32, usize)> = specs
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let counter = evt_mint_counter(s.pattern)?;
            let key_with_params = widget
                .handlers
                .keys()
                .find(|k| k.split('(').next() == Some(s.pattern))?;
            let has_body = widget
                .handlers
                .get(key_with_params)
                .map(|p| match p {
                    auto_ui_compiler::aura::LogicPayload::AstStmts(stmts) => !stmts.is_empty(),
                    _ => false,
                })
                .unwrap_or(false);
            has_body.then_some((counter, i))
        })
        .collect();
    minted.sort();
    if minted.len() == regions.len() {
        for ((_, spec_i), region) in minted.iter().zip(regions.iter()) {
            specs[*spec_i].lambda_region = Some(region.clone());
        }
    }
    specs
}
