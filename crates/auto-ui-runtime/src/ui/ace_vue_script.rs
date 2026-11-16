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
}

/// 并发发射的宿主映射：发射局部名（`l<i>___age_s_<inst>_<field>` 或裸
/// 字段引用）→ `field.value`。词法整词替换（沿 ace_ts_provider 口径）。
fn map_emitted(line: &str) -> String {
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
                None => word.clone(),
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
        if trimmed.starts_with("->") {
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

/// 逐 handler 发射 lane 派生函数体。`state_fn_prefix`=`__age_state_0_`（实例
/// 0；多实例面归 #6d 后）。未命中任何状态函数区间的 handler 具名缺席
/// （返回 `(bodies, absent)` 二元组——调用方按合同决定旧道或拒绝）。
pub fn lane_handler_bodies(
    src: &str,
    path: &str,
    patterns: &[&str],
) -> Result<(Vec<LaneHandlerBody>, Vec<String>), String> {
    // ops msg 无点前缀（ScenarioOp::Send 合同）；patterns 保留原拼写返回。
    let msgs: Vec<&str> = patterns.iter().map(|p| p.trim_start_matches('.')).collect();
    let lane = ComponentFragmentLane::compile(src, path, &msgs)?;
    let ds = lane.desugared_source().to_string();
    let state_fns = lane.state_fns();

    let mut bodies = Vec::new();
    let mut absent = Vec::new();
    for pat in patterns {
        let region = match handler_body_region(&ds, pat) {
            Ok(r) => r,
            Err(_) => {
                absent.push((*pat).to_string());
                continue;
            }
        };
        // 写目标=体文本 LHS 首个 `.field`（赋值形 handler 家族——002 族；
        // 所有状态函数都含全部 handler 的重放表达式，span 包含性不可区分
        // 写路径——具名边界：非单赋值形归 #6d 结构分析）。
        let region_text = &ds[region.clone()];
        let write_field = region_text
            .find('.')
            .and_then(|d| {
                let rest = &region_text[d + 1..];
                let end = rest
                    .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                (end > 0).then(|| rest[..end].to_string())
            })
            .ok_or_else(|| format!("no `.field` LHS in {pat:?} body: {region_text:?}"))?;
        let fn_name = format!("__age_state_0_{write_field}");
        if !state_fns.contains(&fn_name) {
            absent.push((*pat).to_string());
            continue;
        }
        let frag = match lane.widest_expr_in(&fn_name, region.clone())? {
            Some(f) => f,
            None => {
                absent.push((*pat).to_string());
                continue;
            }
        };
        let field = write_field;
        let mut lines: Vec<String> = frag.text.iter().map(|l| map_emitted(l)).collect();
        if let Some(expr) = &frag.expr {
            lines.push(format!("{}.value = {};", field, map_emitted(expr)));
        }
        bodies.push(LaneHandlerBody {
            pattern: (*pat).to_string(),
            field,
            lines,
        });
    }
    Ok((bodies, absent))
}

/// 切片 #6d：**SFC 拼接**——lane 派生体替换编译器 SFC 内对应 handler 函数
/// 体（锚=`function <Variant>(): void {`…配对 `}`；pattern 去点=Variant）。
/// 只动 handler 函数体；其余段落字节不变。Variant 缺席/锚未命中=具名拒。
pub fn splice_lane_handler_bodies(sfc: &str, bodies: &[LaneHandlerBody]) -> Result<String, String> {
    let mut out = sfc.to_string();
    for b in bodies {
        let variant = b.pattern.trim_start_matches('.');
        let anchor = format!("function {variant}(): void {{");
        let start = out
            .find(&anchor)
            .ok_or_else(|| format!("handler fn `{variant}` not found in the SFC"))?;
        let open = start + anchor.len();
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
