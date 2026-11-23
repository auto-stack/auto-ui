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
//! 域错误，981 IF-981-EVENT-PAYLOAD-B）。不可承载载荷（未识别的类型等）不铸检
//! 查函数=具名缺席（fail-closed，旧道承载），不造假数值。
//!
//! 边界（具名）：本模块是路线 B 的**体级发射核心**，不含 SFC 拼接/默认
//! 切换（#6d）；一行单赋值 handler 形态（002 族）为已验面——多语句体/
//! 局部 var/控制流体的区间-最宽启发式不适用者具名拒绝（旧道承载）。

use crate::ui::ace_ts_provider::ComponentFragmentLane;
pub use ace_a2x_ts::script_runtime_declaration;

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

/// PLAN-1072 刀C（str 边界统一）：lane 发射体的**宿主文本边界 marshal**。
///
/// 裁定（证据=005 产品浏览器档：`a2x_str_ne(email.value, …)` 对宿主裸串
/// trap "text value is not UTF-8 bytes"；`a2x_str_free(旧串)` trap "cannot
/// free a non-text value"——原 980 §10-1 owned-wrap 统一决策点）：宿主
/// （Vue ref/SFC 形参）以**普通 JS 字符串**承载 str 状态与 str 形参；rt
/// 文本值（`Uint8Array`/`A2xOwnedStr`）只存在于表达式求值内部。宿主映
/// 射层据此改写发射器确定的 owned-swap 形状（lang 侧 rt 语义发射对 rt
/// 世界正确，本层只做宿主边界）：
/// - 读：`a2x_str_*` 调用行内的 `F.value`/形参名 → `a2x_str_lit(x)` 包裹；
/// - 写：`F.value = <tmp>;`（tmp=owned 临时）→ `F.value = a2x_str_utf8(<tmp>); a2x_str_free(<tmp>);`
/// - 删：owned-swap 快照行（`const <tmp> = F.value;`）与旧值释放块
///   （`if (<tmp> !== null && … a2x_ret_owned_skip) { a2x_str_free(<tmp>); }`）
///   ——宿主 GC 拥有字符串。
/// int/bool/f32 字段与通道实参不经本层（宿主原生类型）。
fn marshal_str_boundary(lines: &[String], str_fields: &[String], params: &[String]) -> Vec<String> {
    let is_temp = |w: &str| -> bool {
        w.len() > 1 && w.starts_with('t') && w[1..].chars().all(|c| c.is_ascii_digit() || c == '_')
    };
    // str 宿主名集：字段（映射后 `F.value`）+ SFC 形参。
    let host_names: Vec<String> = str_fields
        .iter()
        .map(|f| format!("{f}.value"))
        .chain(params.iter().cloned())
        .collect();
    // owned-swap 快照临时（`const <tmp> = <host>;`）——唯一消费者是旧值
    // 释放块，两处一并删。
    let mut snapshot_temps: Vec<String> = Vec::new();
    for l in lines {
        let t = l.trim();
        let Some(rest) = t.strip_prefix("const ") else {
            continue;
        };
        let Some((tmp, rhs)) = rest.split_once(" = ") else {
            continue;
        };
        let rhs = rhs.trim_end_matches(';').trim();
        if is_temp(tmp) && host_names.iter().any(|h| h == rhs) {
            snapshot_temps.push(tmp.to_string());
        }
    }
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut drop_depth: i32 = 0;
    for l in lines {
        let t = l.trim();
        if drop_depth > 0 {
            drop_depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if drop_depth <= 0 {
                drop_depth = 0;
            }
            continue;
        }
        // ① owned-swap 快照行 → 删。
        if let Some(rest) = t.strip_prefix("const ") {
            if let Some((tmp, rhs)) = rest.split_once(" = ") {
                let rhs = rhs.trim_end_matches(';').trim();
                if is_temp(tmp) && host_names.iter().any(|h| h == rhs) {
                    continue;
                }
            }
        }
        // ② 旧值释放块头（引用已删快照临时）→ 整块删（单行形同删）。
        if t.starts_with("if (") && snapshot_temps.iter().any(|tmp| t.contains(tmp.as_str())) {
            let opens = t.matches('{').count() as i32;
            let closes = t.matches('}').count() as i32;
            if opens > 0 && opens - closes > 0 {
                drop_depth = opens - closes;
            }
            continue;
        }
        // ③ 写行：`F.value = <tmp>;`（tmp=owned 临时）→ 解码+释放（原缩进
        // 保持）。
        if !l.contains("a2x_str_") {
            if let Some(eq) = l.find(" = ") {
                let lhs = l[..eq].trim();
                let rhs = l[eq + 3..].trim_end_matches(';').trim();
                if host_names.iter().any(|h| *h == lhs) && is_temp(rhs) {
                    let indent = &l[..l.len() - l.trim_start().len()];
                    out.push(format!(
                        "{indent}{lhs} = a2x_str_utf8({rhs}); a2x_str_free({rhs});"
                    ));
                    continue;
                }
            }
        }
        // ④ 读行：`a2x_str_*` 调用行内的宿主名 → `a2x_str_lit(…)` 包裹
        //（写 LHS 行不含 `a2x_str_`，由 ③ 承载）。
        if l.contains("a2x_str_") {
            let mut mapped = l.clone();
            for h in &host_names {
                mapped = wrap_host_reads(&mapped, h);
            }
            out.push(mapped);
            continue;
        }
        out.push(l.clone());
    }
    out
}

/// 行内整词包裹：`host` 的每次出现（非已包裹——`a2x_str_lit(` 紧前缀除
/// 外，幂等）→ `a2x_str_lit(host)`。按字符边界扫描（UTF-8 安全）；跳过引号
/// 字面量（`"..."` / `'...'`，含转义 `\"` / `\'` / `\\`），避免误包字符串
/// 字面量内容（如 `if light == "light"`）。
fn wrap_host_reads(line: &str, host: &str) -> String {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.';
    let mut out = String::with_capacity(line.len() + 16);
    let chars: Vec<char> = line.chars().collect();
    let needle: Vec<char> = host.chars().collect();
    let n = chars.len();
    let h = needle.len();
    let mut i = 0usize;
    let mut in_quote: Option<char> = None;
    while i < n {
        if let Some(q) = in_quote {
            out.push(chars[i]);
            if chars[i] == '\\' && i + 1 < n {
                i += 1;
                out.push(chars[i]);
            } else if chars[i] == q {
                in_quote = None;
            }
            i += 1;
            continue;
        }
        if chars[i] == '"' || chars[i] == '\'' {
            in_quote = Some(chars[i]);
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if i + h <= n && chars[i..i + h] == needle[..] {
            let before_ok = i == 0 || !is_word(chars[i - 1]);
            let after_ok = i + h >= n || !is_word(chars[i + h]);
            let prefix: String = chars[i.saturating_sub(12)..i].iter().collect();
            let already = prefix.ends_with("a2x_str_lit(");
            if before_ok && after_ok && !already {
                out.push_str("a2x_str_lit(");
                out.push_str(host);
                out.push(')');
                i += h;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
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
/// （无检查/值函数=不可承载载荷；或无区间表达式）具名缺席——调
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
                // 无检查函数=载荷不可承载（981 派发守卫同
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
        // PLAN-1072 残余记账（原 981 r21 漂移项）：`n_assigns` 来自语言侧
        // `handler_assigns`（自 981 r21 起由 `flat_handler_assigns(&stmts)`
        // 跨 If 分支深平铺得出）；其唯一消费方是此处的 `needs_fn` 判定——对
        // 含分支赋值的 handler `n_assigns > 0` 保守且语义正确（分支体必须
        // 发射函数承载），无 census 计数或执行语义漂移。
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
        // PLAN-1072 刀C：str 边界 marshal（宿主裸串 ↔ rt 文本；见
        // `marshal_str_boundary` doc——005 产品浏览器档实证的统一裁定）。
        // int/bool/f32 字段零影响（str_fields 只含 is_text 返回面）。
        let str_fields = lane.str_fields();
        if !str_fields.is_empty() || !spec.params.is_empty() {
            lines = marshal_str_boundary(&lines, &str_fields, spec.params);
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
pub(super) fn channel_pascal(channel: &str) -> String {
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
/// PLAN-1072 刀A：本文件 style 配方的**显式注册**（prepare→parse→replay
/// 三段中的首段；replay 段见 [`replay_file_style_recipes`]——与旧管线
/// `generate_component_from_file` 同源同序）。此前 lane 入口只以裸
/// `VueGenerator` 生成 SFC——文件级 `style N = "…"` 声明不注册，view 内
/// `style: N` ident 引用落 `:style="N"`（setup 作用域无此名=运行时
/// undefined，样式静默丢失）；产品道四调用点靠旧编译刚跑完的线程局部注
/// 册表**隐式余温**生效，独立入口（`auto run` CLI/测试）为空注册表。注
/// 册后 `extract_widget_from_decl` 的 `desugar_style_expr` 与旧管线同形
/// 展开为字面量 class。导入模块解析失败按旧管线容忍（warning）。
fn prepare_file_style_recipes(
    src: &str,
    path: &str,
) -> Result<Vec<(auto_ui_dialect::ast::StyleRecipeDecl, String)>, String> {
    let base_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    auto_ui_compiler::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, src)
        .map_err(|e| format!("style recipes: {e}"))
}

/// replay 段（parse 后：导入源标签序复放 + 宿主本地声明 + 校验）。
fn replay_file_style_recipes(
    imports: &[(auto_ui_dialect::ast::StyleRecipeDecl, String)],
    ast: &auto_ui_dialect::UiCode,
) -> Result<(), String> {
    auto_ui_compiler::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        imports, ast,
    )
    .map_err(|e| format!("style recipes: {e}"))?;
    Ok(())
}

pub fn generate_sfc_with_lane(
    src: &str,
    path: &str,
) -> Result<(String, Vec<LaneHandlerBody>, Vec<String>), String> {
    use auto_ui_compiler::ui_gen::BackendGenerator;
    use auto_ui_dialect::UiStmt;
    // 真实编译器管线。切片#6n：**多 widget 同文件源**——按声明序逐 widget
    // 生成并聚合（lane 经 node 选择器 `compile_at(i)`；单 widget 源行为不
    // 变）。返回值=首个 SFC + 全部 widget 的 bodies/absent 聚合。
    // PLAN-1072 刀A：style 配方 prepare→parse→replay（见函数 doc）。
    let recipe_imports = prepare_file_style_recipes(src, path)?;
    let session = auto_parser_api::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .map_err(|e| format!("parse: {e:?}"))?;
    replay_file_style_recipes(&recipe_imports, &ast)?;
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
        let (_name_w, sfc_w, bodies_w, absent_w, _carried_w) =
            generate_widget_with_lane(decl, src, path, wi)?;
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
) -> Result<(String, String, Vec<LaneHandlerBody>, Vec<String>, Vec<String>), String> {
    generate_widget_with_lane_stores(decl, src, path, node_index, &[], &Default::default())
}

/// PLAN-105 T-02（U03 013/015/016/022）：带 **store 依赖**的 widget 生成——
/// 与 `generate_widget_with_lane` 同一条 SFC 管线，但把 store 名交给
/// `VueGenerator::with_store_deps/with_store_surface`，模板里的 `store.<f>` /
/// `store.<Msg>()` 因此拿到 `const store = use<X>Store()` 绑定。
fn generate_widget_with_lane_stores(
    decl: &auto_ui_dialect::ast::WidgetDecl,
    src: &str,
    path: &str,
    node_index: usize,
    store_deps: &[String],
    store_surface: &std::collections::HashMap<String, Vec<String>>,
) -> Result<(String, String, Vec<LaneHandlerBody>, Vec<String>, Vec<String>), String> {
    use auto_ui_dialect::UiStmt;
    let widget = auto_ui_compiler::aura::extract_widget_from_decl(decl)
        .map_err(|e| format!("extract: {e:?}"))?;
    // 同文件模块 fn 重挂（产品道缺口，jade-garden 实测暴露）：裸
    // VueGenerator 丢文件顶层 fn 定义（vue.rs 链 `same_file_module_fns`
    // 同病同修——PLAN-075 臂）；此处与该链同款收集（UiDialect→lower→
    // core Stmt::Fn→`extract_module_fn`），失败取 None=链上行为。
    let mut gen = auto_ui_compiler::ui_gen::VueGenerator::new();
    if !store_deps.is_empty() {
        gen = gen
            .with_store_deps(store_deps.to_vec())
            .with_store_surface(store_surface.clone());
    }
    if let Some(module_fns) = same_file_module_fns(src) {
        gen = gen.with_module_fns(module_fns);
    }
    // PLAN-113（026 真语料）：lane 产品道补 **use 导入模块 fn 池**（官方链
    // `with_use_module_fns` 同款——026 `use components.tree_util: filter_tree,
    // collect_ids, toggle_id` 的兄弟模块 helper；缺失时调用点有 emission 无
    // 定义，浏览器装载即 `ReferenceError`）。收集器对解析/读取失败返回空池
    // （无导入时零挂，不阻断 lane 产品）。
    let (use_fns, use_names) =
        auto_ui_compiler::ui_gen::api::collect_use_module_fns(std::path::Path::new(path), src);
    if !use_fns.is_empty() {
        gen = gen.with_use_module_fns(use_fns, use_names);
    }
    let sfc = gen
        .generate_sfc(&widget)
        .map_err(|e| format!("SFC: {e:?}"))?;
    // lane 体（specs=widget handler 键去参段——`.AddItem(text)` 形取
    // `.AddItem`+按位形参名（`handler_params` 同键族）；缺席者保旧道）。
    // PLAN-1072 刀B（010 真语料）：方言侧为 input 值绑定合成的
    // `__bind_*` handler **不进 lane 规格集**——值绑定由模板腿的
    // `:value`/v-model 承载（AGE 面无此声明，进 ops=整 lane 编译失败
    // 连带真 handler 全体缺席；旧 bytecode 轨的合成件归旧轨）。
    let patterns: Vec<String> = widget
        .handlers
        .keys()
        .map(|k| k.split('(').next().unwrap_or(k).to_string())
        .filter(|k| !k.contains("__bind_"))
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
/// PLAN-108：expr 树内调用点递归检测——op_exempt 预扫描的完整形。旧扫描
/// 只认 `Stmt::Expr(Expr::Call(_))`（裸调用表达式语句），`var x = f(...)`
///（`Stmt::Store` 初始化子）与 `return f(...)` 漏检：域外调用进 ops 会把
/// 整个 lane 编译打成 Err，兄弟 handler 全体缺席——#6p 的裁定意图是「豁免
/// 含调用 handler、保全兄弟」。覆盖主 composite 变体递归；未列出的叶/罕见
/// 变体默认无调用（欠豁免=旧行为，不过度豁免）。
fn expr_contains_call(e: &auto_lang_core::ast::Expr) -> bool {
    use auto_lang_core::ast::Expr;
    match e {
        Expr::Call(_) => true,
        Expr::View(inner)
        | Expr::Mut(inner)
        | Expr::Move(inner)
        | Expr::Take(inner)
        | Expr::Unary(_, inner)
        | Expr::Dot(inner, _)
        | Expr::Some(inner)
        | Expr::Ok(inner)
        | Expr::Err(inner)
        | Expr::BoxExpr(inner)
        | Expr::ArcExpr(inner)
        | Expr::Yield(inner)
        | Expr::ErrorPropagate(inner)
        | Expr::Await { expr: inner }
        | Expr::Cast { expr: inner, .. }
        | Expr::To { expr: inner, .. } => expr_contains_call(inner),
        Expr::Index(a, i) => expr_contains_call(a) || expr_contains_call(i),
        Expr::Bina(l, _, r) | Expr::NullCoalesce(l, r) => {
            expr_contains_call(l) || expr_contains_call(r)
        }
        Expr::Array(items) => items.iter().any(expr_contains_call),
        Expr::Object(pairs) => pairs.iter().any(|p| expr_contains_call(&p.value)),
        Expr::Block(body) => body.stmts.iter().any(stmt_contains_call),
        Expr::If(if_expr) => {
            if_expr.branches.iter().any(|b| {
                expr_contains_call(&b.cond) || b.body.stmts.iter().any(stmt_contains_call)
            }) || if_expr
                .else_
                .as_ref()
                .is_some_and(|body| body.stmts.iter().any(stmt_contains_call))
        }
        _ => false,
    }
}

fn stmt_contains_call(s: &auto_lang_core::ast::Stmt) -> bool {
    use auto_lang_core::ast::Stmt;
    match s {
        Stmt::Expr(e) => expr_contains_call(e),
        Stmt::Store(st) => expr_contains_call(&st.expr),
        Stmt::Return(e) | Stmt::Reply(e) => expr_contains_call(e),
        Stmt::Block(body) => body.stmts.iter().any(stmt_contains_call),
        Stmt::If(if_stmt) => {
            if_stmt.branches.iter().any(|b| {
                expr_contains_call(&b.cond) || b.body.stmts.iter().any(stmt_contains_call)
            }) || if_stmt
                .else_
                .as_ref()
                .is_some_and(|body| body.stmts.iter().any(stmt_contains_call))
        }
        _ => false,
    }
}

/// PLAN-108：expr 树内 **Http 效应族**调用检测（`Http.<verb>` 限定调用，
/// verb ∈ ts_adapter 受检映射的闭合集——get/get_msg/get/post/post_json/
/// put/put_json/patch/patch_json/delete）。具名承载（carried）判定的唯一
/// 有效原因；其它域外调用仍=具名缺席硬拒。
fn expr_contains_http_effect(e: &auto_lang_core::ast::Expr) -> bool {
    use auto_lang_core::ast::Expr;
    const HTTP_VERBS: &[&str] = &[
        "get", "get_json", "get_msg", "post", "post_json", "put", "put_json", "patch",
        "patch_json", "delete",
    ];
    match e {
        Expr::Call(call) => {
            matches!(
                call.name.as_ref(),
                Expr::Dot(obj, verb)
                    if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "Http")
                        && HTTP_VERBS.contains(&verb.as_str())
            ) || call
                .args
                .args
                .iter()
                .any(|a| expr_contains_http_effect(&a.get_expr().clone()))
        }
        Expr::View(inner)
        | Expr::Mut(inner)
        | Expr::Move(inner)
        | Expr::Take(inner)
        | Expr::Unary(_, inner)
        | Expr::Dot(inner, _)
        | Expr::Some(inner)
        | Expr::Ok(inner)
        | Expr::Err(inner)
        | Expr::BoxExpr(inner)
        | Expr::ArcExpr(inner)
        | Expr::Yield(inner)
        | Expr::ErrorPropagate(inner)
        | Expr::Await { expr: inner }
        | Expr::Cast { expr: inner, .. }
        | Expr::To { expr: inner, .. } => expr_contains_http_effect(inner),
        Expr::Index(a, i) => expr_contains_http_effect(a) || expr_contains_http_effect(i),
        Expr::Bina(l, _, r) | Expr::NullCoalesce(l, r) => {
            expr_contains_http_effect(l) || expr_contains_http_effect(r)
        }
        Expr::Array(items) => items.iter().any(expr_contains_http_effect),
        Expr::Object(pairs) => pairs.iter().any(|p| expr_contains_http_effect(&p.value)),
        Expr::Block(body) => body.stmts.iter().any(stmt_contains_http_effect),
        Expr::If(if_expr) => {
            if_expr.branches.iter().any(|b| {
                expr_contains_http_effect(&b.cond)
                    || b.body.stmts.iter().any(stmt_contains_http_effect)
            }) || if_expr
                .else_
                .as_ref()
                .is_some_and(|body| body.stmts.iter().any(stmt_contains_http_effect))
        }
        _ => false,
    }
}


/// PLAN-108：expr 树内 **非 Http 效应族**调用检测——承载判定的闭合集：
/// 体里存在 Http 族之外的任何调用即不可承载（保持具名缺席硬拒）。
fn expr_contains_non_http_call(e: &auto_lang_core::ast::Expr) -> bool {
    use auto_lang_core::ast::Expr;
    const HTTP_VERBS: &[&str] = &[
        "get", "get_json", "get_msg", "post", "post_json", "put", "put_json", "patch",
        "patch_json", "delete",
    ];
    match e {
        Expr::Call(call) => {
            let http_here = matches!(
                call.name.as_ref(),
                Expr::Dot(obj, verb)
                    if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "Http")
                        && HTTP_VERBS.contains(&verb.as_str())
            );
            !http_here
                || call
                    .args
                    .args
                    .iter()
                    .any(|a| expr_contains_non_http_call(&a.get_expr().clone()))
        }
        Expr::View(inner)
        | Expr::Mut(inner)
        | Expr::Move(inner)
        | Expr::Take(inner)
        | Expr::Unary(_, inner)
        | Expr::Dot(inner, _)
        | Expr::Some(inner)
        | Expr::Ok(inner)
        | Expr::Err(inner)
        | Expr::BoxExpr(inner)
        | Expr::ArcExpr(inner)
        | Expr::Yield(inner)
        | Expr::ErrorPropagate(inner)
        | Expr::Await { expr: inner }
        | Expr::Cast { expr: inner, .. }
        | Expr::To { expr: inner, .. } => expr_contains_non_http_call(inner),
        Expr::Index(a, i) => expr_contains_non_http_call(a) || expr_contains_non_http_call(i),
        Expr::Bina(l, _, r) | Expr::NullCoalesce(l, r) => {
            expr_contains_non_http_call(l) || expr_contains_non_http_call(r)
        }
        Expr::Array(items) => items.iter().any(expr_contains_non_http_call),
        Expr::Object(pairs) => pairs.iter().any(|p| expr_contains_non_http_call(&p.value)),
        Expr::Block(body) => body.stmts.iter().any(stmt_contains_non_http_call),
        Expr::If(if_expr) => {
            if_expr.branches.iter().any(|b| {
                expr_contains_non_http_call(&b.cond)
                    || b.body.stmts.iter().any(stmt_contains_non_http_call)
            }) || if_expr
                .else_
                .as_ref()
                .is_some_and(|body| body.stmts.iter().any(stmt_contains_non_http_call))
        }
        _ => false,
    }
}

fn stmt_contains_non_http_call(s: &auto_lang_core::ast::Stmt) -> bool {
    use auto_lang_core::ast::Stmt;
    match s {
        Stmt::Expr(e) => expr_contains_non_http_call(e),
        Stmt::Store(st) => expr_contains_non_http_call(&st.expr),
        Stmt::Return(e) | Stmt::Reply(e) => expr_contains_non_http_call(e),
        Stmt::Block(body) => body.stmts.iter().any(stmt_contains_non_http_call),
        Stmt::If(if_stmt) => {
            if_stmt.branches.iter().any(|b| {
                expr_contains_non_http_call(&b.cond)
                    || b.body.stmts.iter().any(stmt_contains_non_http_call)
            }) || if_stmt
                .else_
                .as_ref()
                .is_some_and(|body| body.stmts.iter().any(stmt_contains_non_http_call))
        }
        _ => false,
    }
}

fn stmt_contains_http_effect(s: &auto_lang_core::ast::Stmt) -> bool {
    use auto_lang_core::ast::Stmt;
    match s {
        Stmt::Expr(e) => expr_contains_http_effect(e),
        Stmt::Store(st) => expr_contains_http_effect(&st.expr),
        Stmt::Return(e) | Stmt::Reply(e) => expr_contains_http_effect(e),
        Stmt::Block(body) => body.stmts.iter().any(stmt_contains_http_effect),
        Stmt::If(if_stmt) => {
            if_stmt.branches.iter().any(|b| {
                expr_contains_http_effect(&b.cond)
                    || b.body.stmts.iter().any(stmt_contains_http_effect)
            }) || if_stmt
                .else_
                .as_ref()
                .is_some_and(|body| body.stmts.iter().any(stmt_contains_http_effect))
        }
        _ => false,
    }
}

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
            // PLAN-108：调用检测从「裸调用表达式语句」扩到任意语句位
            // （Store 初始化子/return/if 分支——`var x = f(...)` 旧扫描漏检，
            // 域外调用进 ops 把整 lane 编译打成 Err、兄弟全体缺席）。
            let has_call = stmts.iter().any(stmt_contains_call);
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
    // PLAN-108：lane 编译整体失败须与逐 handler 缺席区分——失败时携带
    // 标记，承载判定的覆盖面随之收紧（见下）。
    let ((bodies, absent), compile_failed) =
        match lane_handler_bodies_spec_at(src, path, &specs, node_index, &channel_ctx) {
            Ok(v) => (v, false),
            Err(_) => ((Vec::new(), patterns.clone()), true),
        };
    // PLAN-108 具名承载：缺席 handler 满足下列条件时由旧道受检体承载并
    // 记入 carried——工具键门逐名放行并打印诊断行（不静默回落）：
    // - 逐 handler 缺席（编译成功）：体含 Http 效应族调用（闭合 VM 原生
    //   族；ts_adapter 有受检 await fetch 映射）；
    // - 编译整体失败：体除 Http 效应族外**无任何调用**（纯状态/Http 混合
    //   体——失败只可能由 Http 族符号引起；AST 可验证的闭合判定）。含其它
    //   域外调用的体保持 absent 硬拒。
    let handler_bodies_call_clean = |base: &str| {
        widget.handlers.iter().any(|(pattern, payload)| {
            let pb = pattern.split('(').next().unwrap_or(pattern).trim_start_matches('.');
            pb == base
                && matches!(
                    payload,
                    auto_ui_compiler::aura::LogicPayload::AstStmts(stmts)
                        if !stmts.iter().any(stmt_contains_non_http_call)
                )
        })
    };
    let (carried, absent): (Vec<String>, Vec<String>) =
        absent.into_iter().partition(|pat| {
            let base = pat.trim_start_matches('.');
            if !handler_bodies_call_clean(base) {
                return false;
            }
            // PLAN-108 F-1（review needs_fix 修复）：承载谓词对称化——
            // 组件面 Http 准入（M1）后 lane 编译对 Http widget 整体成功，
            // 纯状态兄弟（.Bump/.Tick 等）经「逐 handler 缺席」到达本判定；
            // 两臂统一为「体 call-clean 且 widget 确有 Http 效应」（call-clean
            // 已由外层 filter 保证）。无 Http 效应的 widget 的派生缺席保持
            // 硬拒（失败另有原因，不得掩盖）。
            let _ = compile_failed;
            widget.handlers.values().any(|payload| {
                matches!(
                    payload,
                    auto_ui_compiler::aura::LogicPayload::AstStmts(stmts)
                        if stmts.iter().any(stmt_contains_http_effect)
                )
            })
        });
    let spliced = if bodies.is_empty() {
        sfc
    } else {
        splice_lane_handler_bodies(&sfc, &bodies)?
    };
    // ④-A #8：名字随件返回（多 widget 产品道按名对位产物）。
    Ok((widget.name.clone(), spliced, bodies, absent, carried))
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

/// PLAN-980 U0 产品出口（④-A #5）：单文件 lane SFC 的**产品形态**。
///
/// 生成器（而非测试夹具）承担 U0 预验收夹具里的两步：①SFC 显式发射
/// `import { … } from "./a2x_rt.mjs"`（名字=脚本实际引用的 `a2x_*` 词法
/// 整词 ∩ 官方 rt 导出面，按首次出现序稳定）；②官方运行时模块源随产物
/// 返回（落盘约定=与 SFC 同目录的 `a2x_rt.mjs`）。`lang="ts"` 保持——
/// TS 处理由既有 Vue 构建链（vite/esbuild 同族转换）承担，产物零额外
/// 要求。absent handler 仍保编译器旧道体（双轨期合同不变）。
#[derive(Clone, Debug)]
pub struct LaneSfcProduct {
    /// 完整 SFC（含显式 a2x_rt import；脚本零 `a2x_*` 引用时不含）。
    pub sfc: String,
    /// 官方 a2x 运行时模块源（`script_runtime_body` 原样；ESM）。
    pub runtime_mjs: String,
    /// lane 派生体（诊断/记账出口——980 验收断言复用）。
    pub lane_bodies: Vec<LaneHandlerBody>,
    /// 具名缺席（旧道承载；产品侧照常运行）。
    pub absent: Vec<String>,
}

/// 词法整词收集 `a2x_*` 标识符（与 `map_emitted` 同族判定——不截断、
/// 不跨边界匹配）。
fn a2x_identifiers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut flush = |word: &mut String, out: &mut Vec<String>| {
        if word.starts_with("a2x_") && !out.contains(word) {
            out.push(word.clone());
        }
        word.clear();
    };
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            flush(&mut word, &mut out);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// ④-A #8：rt import 插入参数化（href=产物相对 rt 的引用——App.vue 用
/// `./a2x_rt.mjs`、src/components/ 下用 `../a2x_rt.mjs`）。零引用/已含时
/// 原样返回。
fn attach_rt_import(sfc: String, runtime_mjs: &str, href: &str) -> String {
    let exports: Vec<&str> = runtime_mjs
        .lines()
        .filter_map(|l| {
            l.strip_prefix("export function ")
                .and_then(|r| r.split('(').next())
        })
        .filter(|n| n.starts_with("a2x_"))
        .collect();
    let identifiers = a2x_identifiers(&sfc);
    let needed: Vec<&str> = identifiers
        .iter()
        .map(|w| w.as_str())
        .filter(|w| exports.contains(w))
        .collect();
    if needed.is_empty() || sfc.contains(href) {
        return sfc;
    }
    let tag_at = match sfc.find("<script setup") {
        Some(at) => at,
        None => return sfc,
    };
    let insert_at = match sfc[tag_at..].find('>') {
        Some(off) => tag_at + off + 1,
        None => return sfc,
    };
    let mut out = String::with_capacity(sfc.len() + 96);
    out.push_str(&sfc[..insert_at]);
    out.push_str(&format!(
        "
import {{ {} }} from \"{href}\";",
        needed.join(", ")
    ));
    out.push_str(&sfc[insert_at..]);
    out
}

/// PLAN-980 ④-A #8：**多 widget 产品件**——单文件源逐 widget 的 lane SFC
/// （每个 widget 独立拼接自己的 lane 体；rt 官方模块共享一份，import 按各
/// 产物位置参数化——根 widget 与组件 widget 的相对 href 不同）。
#[derive(Clone, Debug)]
pub struct WidgetLaneSfc {
    /// widget 名（声明序）。
    pub name: String,
    /// 该 widget 的完整 SFC（lane 体已拼接 + 按位显式 a2x_rt import）。
    pub sfc: String,
    /// 该 widget 的 lane 派生体（诊断/记账出口）。
    pub bodies: Vec<LaneHandlerBody>,
    /// 该 widget 的具名缺席（工具键门聚合拒绝；产品侧不裁决）。
    pub absent: Vec<String>,
    /// PLAN-108 具名承载：缺席中由旧道受检体承载的 Http 效应 handler
    /// （闭合 VM 原生族；名字逐条在案，工具键门放行并打印诊断行）。
    pub carried: Vec<String>,
}

/// 多 widget 产品集合（`generate_sfcs_with_lane_product` 返回）。
#[derive(Clone, Debug)]
/// PLAN-105 T-02：`store` 声明生成的 Vue composable（state refs + actions）。
pub struct LaneStoreComposable {
    pub name: String,
    pub code: String,
}

pub struct LaneSfcProductMulti {
    /// 声明序逐 widget（首件=根/App 位）。
    pub widgets: Vec<WidgetLaneSfc>,
    /// 官方 a2x 运行时模块源（全文件共享一份；落盘约定=src/a2x_rt.mjs）。
    pub runtime_mjs: String,
}

/// 多 widget 产品生成入口（④-A #8：工具完整/增量双道的多 widget 接入）。
/// `rt_href_root`=根产物（src/App.vue 位）的相对引用；`rt_href_component`
/// =组件产物（src/components/{W}.vue 位）的相对引用。
/// 旧入口（只要 widget 产品；store composable 丢弃）。
pub fn generate_sfcs_with_lane_product(
    src: &str,
    path: &str,
    rt_href_root: &str,
    rt_href_component: &str,
) -> Result<LaneSfcProductMulti, String> {
    generate_sfcs_with_lane_product_stores(src, path, rt_href_root, rt_href_component)
        .map(|(multi, _stores)| multi)
}

/// PLAN-105 T-02（U03 store 驱动 app）：同一产品 + 每个 `store X { … }` 的 Vue
/// composable（`generate_store_composable`），widget SFC 带 store 依赖/暴露面。
pub fn generate_sfcs_with_lane_product_stores(
    src: &str,
    path: &str,
    rt_href_root: &str,
    rt_href_component: &str,
) -> Result<(LaneSfcProductMulti, Vec<LaneStoreComposable>), String> {
    use auto_ui_dialect::UiStmt;
    // PLAN-1072 刀A：style 配方 prepare→parse→replay（产品道与测试道同序）。
    let recipe_imports = prepare_file_style_recipes(src, path)?;
    let session = auto_parser_api::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .map_err(|e| format!("parse: {e:?}"))?;
    replay_file_style_recipes(&recipe_imports, &ast)?;
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
    // PLAN-105 T-02：`store X { … }` → Vue composable + widget 的 store 绑定。
    // PLAN-112 T-02：实后端 store（025 `SysStore`）的 composable 与 `auto-ui-tools`
    // 旧道同语义。此前 store 的 `api_imports` 与函数池皆空——composable 无
    // `@/lib/api` 导入、API 调用不带 `await`、裸调 helper（运行期 ReferenceError）。
    // ① `use back.api: f…` → `store.api_imports`（`@/lib/api` 导入 + 调用点
    //    `await`）。合并源里多个单元可各自声明同一行（025：app.at 与
    //    sys_store.at）——按首次出现去重，否则发射重复的导入说明符（JS 语法错误）。
    // ② 同文件顶层 `fn` 与 `use <unit>: fn` 模块函数合并为一个池，经
    //    `generate_store_composable_full` 的入口门控 + 闭包拉取内联
    //    （PLAN-671 ③ 同机制：跨文件 helper 如 `fmt.at` 的 `n1`/`rate` 及其被调 `n2`）。
    let mut api_imports: Vec<String> = Vec::new();
    for name in auto_ui_compiler::ui_gen::api::extract_api_imports_from_ast(
        &ast,
        Some(std::path::Path::new(path)),
    ) {
        if !api_imports.contains(&name) {
            api_imports.push(name);
        }
    }
    let same_file_fns: Vec<auto_ui_compiler::aura::AuraModuleFn> = ast
        .core_stmts()
        .filter_map(|stmt| match stmt {
            auto_lang_core::ast::Stmt::Fn(fn_decl) => {
                auto_ui_compiler::aura::extract_module_fn(fn_decl)
            }
            _ => None,
        })
        .collect();
    // 同文件 fn 名首到先得；use 导入名只做入口门控（非 fn 的导入名无池项，
    // 天然无效）。`use back.api` 在 `collect_use_module_fns` 内被跳过。
    let mut module_fns = same_file_fns;
    let mut module_fn_names: std::collections::HashSet<String> =
        module_fns.iter().map(|f| f.name.clone()).collect();
    let (use_fns, use_names) =
        auto_ui_compiler::ui_gen::api::collect_use_module_fns(std::path::Path::new(path), src);
    for mfn in use_fns {
        if !module_fns.iter().any(|f| f.name == mfn.name) {
            module_fns.push(mfn);
        }
    }
    module_fn_names.extend(use_names);
    let mut stores: Vec<LaneStoreComposable> = Vec::new();
    let mut store_deps: Vec<String> = Vec::new();
    let mut store_surface: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for stmt in ast.domain_stmts() {
        if let UiStmt::StoreDecl(d) = stmt {
            let mut store = auto_ui_compiler::aura::extract_store_from_decl(d)
                .map_err(|e| format!("store extract: {e:?}"))?;
            store.api_imports = api_imports.clone();
            let (code, _) = auto_ui_compiler::ui_gen::VueGenerator::generate_store_composable_full(
                &store,
                &module_fns,
                &module_fn_names,
            );
            store_deps.push(store.name.clone());
            let methods: Vec<String> = store
                .handlers
                .keys()
                .map(|k| k.split('(').next().unwrap_or(k).to_string())
                .collect();
            store_surface.insert(store.name.clone(), methods);
            stores.push(LaneStoreComposable {
                name: store.name.clone(),
                code,
            });
        }
    }
    // PLAN-105 T-02（U03 015）：`view fn <Name>(…) { … }` 片段先登记——widget
    // 抽取时按名字解析成内联视图片段（否则 SFC 会 import 一个不存在的子组件）。
    for stmt in ast.domain_stmts() {
        if let UiStmt::ViewFragmentDecl(frag) = stmt {
            auto_ui_compiler::aura::register_view_fragment(frag);
        }
    }
    let runtime_mjs = ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[]);
    let mut widgets = Vec::with_capacity(decls.len());
    for (wi, decl) in decls.iter().enumerate() {
        let (name, sfc, bodies, absent, carried) = generate_widget_with_lane_stores(
            decl,
            src,
            path,
            wi,
            &store_deps,
            &store_surface,
        )?;
        let href = if wi == 0 {
            rt_href_root
        } else {
            rt_href_component
        };
        let sfc = attach_rt_import(sfc, &runtime_mjs, href);
        widgets.push(WidgetLaneSfc {
            name,
            sfc,
            bodies,
            absent,
            carried,
        });
    }
    Ok((
        LaneSfcProductMulti {
            widgets,
            runtime_mjs,
        },
        stores,
    ))
}

/// 单文件 lane 路径的产品生成入口（982 的 `auto run`（Vue）后续经 CLI
/// 调用；本函数只管生成与运行时依赖，不启动页面/服务）。④-A #8 起为多
/// widget 出口的首件委托（单 widget 源行为不变；多 widget 源=首 widget
/// SFC + 聚合 bodies/absent——完整逐件产物走 `generate_sfcs_with_lane_product`）。
pub fn generate_sfc_with_lane_product(src: &str, path: &str) -> Result<LaneSfcProduct, String> {
    let multi = generate_sfcs_with_lane_product(src, path, "./a2x_rt.mjs", "./a2x_rt.mjs")?;
    let lane_bodies = multi
        .widgets
        .iter()
        .flat_map(|w| w.bodies.clone())
        .collect();
    let absent = multi
        .widgets
        .iter()
        .flat_map(|w| w.absent.clone())
        .collect();
    let first = multi
        .widgets
        .into_iter()
        .next()
        .ok_or_else(|| "no widget decl".to_string())?;
    Ok(LaneSfcProduct {
        sfc: first.sfc,
        runtime_mjs: multi.runtime_mjs,
        lane_bodies,
        absent,
    })
}

/// 同文件模块 fn 收集（vue.rs 链 `same_file_module_fns` 同款——公开 API
/// 复刻，供单文件 lane 生成道重挂；PLAN-075 的裸生成器丢顶层 fn 同病）。
/// 解析/降级失败返回 None（不猜、不阻断——生成继续走裸道，调用方诊断）。
fn same_file_module_fns(src: &str) -> Option<Vec<auto_ui_compiler::aura::AuraModuleFn>> {
    let session = auto_lang_core::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .and_then(|ui| auto_ui_compiler::ui_ast::lower_ui(&ui).map(|lowered| lowered.ui))
        .ok()?;
    Some(
        ast.core_stmts()
            .filter_map(|s| match s {
                auto_lang_core::ast::Stmt::Fn(f) => auto_ui_compiler::aura::extract_module_fn(f),
                _ => None,
            })
            .collect(),
    )
}
