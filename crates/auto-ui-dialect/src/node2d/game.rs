//! `game2d` 作者入口块（auto-ui PLAN-023 T-03，components-2d-game profile）。
//!
//! 触发表 / cue 表 / HUD 槽 / 音频资源的声明面。只服务 pilot 的 game
//! 降层（`--profile components-2d-game`）；不进 `UiCode`，不改变生产
//! widget/store 文法。文法（块内属性行，空行/逗号宽容）：
//!
//! ```text
//! game2d {
//!     audio {
//!         bank: "<entry-relative wav path>"
//!         font: "<entry-relative ttf path>"
//!         license: "<entry-relative license text path>"
//!     }
//!     cue <name> { span: (start_frame, end_frame) gain: <0.0..=1.0> }
//!     hud {
//!         slot <0..3> {
//!             anchor: <0..3>  offset: (x, y)  size: <16..48>  rgba: (r,g,b,a)
//!             text: <root field>?     // closeout 绑定（有则该槽 visible=1）
//!             init: "<initial>"?      // scene 初始文字（<=32 ASCII）
//!         }
//!         text <root field> { <int>: "<str>" ... default: "<str>" }
//!     }
//!     trigger <name> {
//!         shape: circle radius: <r>   |   shape: box
//!         target: <root-view instance>
//!         event: <msg 名>              // 目标定义的成员事件（接收方派发名）
//!         on: collect  guard: <field>? cue: <cue> full_at: <n> full_cue: <cue>
//!         | on: fail    guard: <field>? value: <v> cue: <cue>
//!         | on: goal    guard: <field>? when_count: <n> phase: <v> value: <v> cue: <cue>
//!     }
//! }
//! ```
//!
//! 冻结语义（T-02 §9 F-2a/F-4；authoring 精确拼写 = D-02 归 T-03 的
//! 普通实现选择）：
//! - 触发对序 = trigger 声明序（pair 0..7）；subject 恒为唯一 character
//!   实例（不设 authoring 字段）；shape/radius 来自本块。
//! - cue id = cue 声明序（= scene.agscene 顶层 cues 数组下标）；世界侧
//!   `game_cue(<id>)` 用同一映射。
//! - 根字段所有权：`count`/`state` 为作者所有（根 model 必须声明，缺失
//!   具名拒绝；state 语义冻结 0=Playing/1=Won/2=Failed）；`gate`/`ord`/
//!   `np`/`p0`/`p1` 为生成基础设施（作者声明同名 → 具名拒绝）。
//! - HUD 文字映射生成 `fn hud_<field>_text(n int) str`（分支选择字面量，
//!   F-5；运行时 >32 字符是 fault 56，不在降层拒绝）。

use auto_parser_api::ast::Expr;
use auto_parser_api::error::{pos_to_span, span_from, AutoError, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;
use auto_parser_api::DomainParseContext;

use super::ast::{LightSpan, Prop, PropValue};
use super::parse::parse_prop_block;

/// `game2d { ... }` 完整块。
#[derive(Debug, Clone)]
pub struct Game2dBlock {
    pub audio: Option<Game2dAudio>,
    pub cues: Vec<Game2dCue>,
    pub hud: Option<Game2dHud>,
    pub triggers: Vec<Game2dTrigger>,
    pub span: LightSpan,
}

/// `audio { bank/font/license }`：三路径均相对 entry 目录。
#[derive(Debug, Clone)]
pub struct Game2dAudio {
    pub bank: String,
    pub font: String,
    pub license: String,
    pub span: LightSpan,
}

/// `cue <name> { span: (s, e) gain: g }`；id = 声明序。
#[derive(Debug, Clone)]
pub struct Game2dCue {
    pub name: String,
    pub start_frame: i64,
    pub end_frame: i64,
    pub gain: f64,
    pub span: LightSpan,
}

/// `hud { slot N {...} text <field> {...} }`。
#[derive(Debug, Clone)]
pub struct Game2dHud {
    pub slots: Vec<Game2dSlot>,
    pub texts: Vec<Game2dText>,
    pub span: LightSpan,
}

/// 一个 HUD 槽的版面 + 可选 closeout 文字绑定。
#[derive(Debug, Clone)]
pub struct Game2dSlot {
    pub slot: i64,
    pub anchor: i64,
    pub offset: (f64, f64),
    pub size: f64,
    pub rgba: (f64, f64, f64, f64),
    pub text: Option<String>,
    pub init: Option<String>,
    pub span: LightSpan,
}

/// `text <field> { <int>: "<str>" ... default: "<str>" }`：分支选择字面量
/// 映射（F-5），生成 `fn hud_<field>_text(n int) str`。
#[derive(Debug, Clone)]
pub struct Game2dText {
    pub field: String,
    pub entries: Vec<(i64, String)>,
    pub default: String,
    pub span: LightSpan,
}

/// 触发行形状：圆（显式半径）或盒（AABB，取实例自身几何）。
#[derive(Debug, Clone)]
pub enum Game2dShape {
    Circle(f64),
    Box,
}

/// 触发行效果类别（封闭词表；体形状由降层按 F-2/F-6 骨架生成）。
/// count/state 根字段是 F-4 冻结名（作者在根 model 声明，行内不重复）。
#[derive(Debug, Clone)]
pub enum Game2dTriggerKind {
    /// 收集：guard 命中 → 接收方 + despawn + r_count+1 + gate + cue；
    /// r_count==full_at 时再发 full_cue。
    Collect {
        cue: String,
        full_at: i64,
        full_cue: String,
    },
    /// 失败：接收方 + r_state=value + gate + cue（可选 guard）。
    Fail {
        value: i64,
        cue: String,
    },
    /// 终点：r_count==when_count 时读 phase；phase==预期 → 接收方 +
    /// r_state=value + gate + cue（不 despawn）。
    Goal {
        when_count: i64,
        phase: i64,
        value: i64,
        cue: String,
    },
}

/// `trigger <name> { ... }` 一行（= 一个触发对）。
#[derive(Debug, Clone)]
pub struct Game2dTrigger {
    pub name: String,
    pub shape: Game2dShape,
    pub target: String,
    /// 目标定义的成员事件名（接收方派发 `<leaf>_<event 小写>`）。
    pub event: String,
    pub kind: Game2dTriggerKind,
    pub guard: Option<String>,
    pub span: LightSpan,
}

/// 属性行定位拒绝（span = 该属性 token 区间）。
fn prop_err(pr: &Prop, message: String) -> AutoError {
    SyntaxError::Generic {
        message,
        span: span_from(pr.span.0, pr.span.1),
    }
    .into()
}

/// 游标定位拒绝（span = 当前 token）。
fn cursor_err<T>(cx: &mut dyn DomainParseContext, message: String) -> AutoResult<T> {
    Err(SyntaxError::Generic {
        message,
        span: pos_to_span(cx.cur_pos()),
    }
    .into())
}

fn need_str(
    cx: &mut dyn DomainParseContext,
    kind_name: &str,
    name: &str,
    v: Option<String>,
    what: &str,
) -> AutoResult<String> {
    match v {
        Some(s) => Ok(s),
        None => cursor_err(
            cx,
            format!("game2d {kind_name} trigger '{name}' missing '{what}:'"),
        ),
    }
}

fn need_int(
    cx: &mut dyn DomainParseContext,
    kind_name: &str,
    name: &str,
    v: Option<i64>,
    what: &str,
) -> AutoResult<i64> {
    match v {
        Some(s) => Ok(s),
        None => cursor_err(
            cx,
            format!("game2d {kind_name} trigger '{name}' missing '{what}:'"),
        ),
    }
}

/// 属性取数值字面量（int/float 皆可）。
fn num_of(pr: &Prop) -> AutoResult<f64> {
    let v = match &pr.value {
        PropValue::One(Expr::Int(v)) => *v as f64,
        PropValue::One(Expr::I64(v)) => *v as f64,
        PropValue::One(Expr::Float(v, _)) => *v,
        PropValue::One(Expr::Double(v, _)) => *v,
        _ => {
            return Err(prop_err(
                pr,
                format!("game2d prop '{}' must be a numeric literal", pr.name),
            ))
        }
    };
    Ok(v)
}

fn int_of(pr: &Prop) -> AutoResult<i64> {
    Ok(num_of(pr)? as i64)
}

fn numeric_component(e: &Expr) -> Option<f64> {
    match e {
        Expr::Int(v) => Some(*v as f64),
        Expr::I64(v) => Some(*v as f64),
        Expr::Float(v, _) => Some(*v),
        Expr::Double(v, _) => Some(*v),
        _ => None,
    }
}

fn pair_of(pr: &Prop, n: usize) -> AutoResult<Vec<f64>> {
    let es = match &pr.value {
        PropValue::Pair(es) if es.len() == n => es,
        _ => {
            return Err(prop_err(
                pr,
                format!(
                    "game2d prop '{}' must be a {n}-literal tuple like (a, b)",
                    pr.name
                ),
            ))
        }
    };
    let mut out = Vec::new();
    for e in es {
        match numeric_component(e) {
            Some(v) => out.push(v),
            None => {
                return Err(prop_err(
                    pr,
                    format!("game2d prop '{}' must hold {n} numeric literals", pr.name),
                ))
            }
        }
    }
    Ok(out)
}

fn pair2(pr: &Prop) -> AutoResult<(f64, f64)> {
    let v = pair_of(pr, 2)?;
    Ok((v[0], v[1]))
}

fn ipair2(pr: &Prop) -> AutoResult<(i64, i64)> {
    let v = pair_of(pr, 2)?;
    Ok((v[0] as i64, v[1] as i64))
}

fn pair4(pr: &Prop) -> AutoResult<(f64, f64, f64, f64)> {
    let v = pair_of(pr, 4)?;
    Ok((v[0], v[1], v[2], v[3]))
}

fn str_of(pr: &Prop) -> AutoResult<String> {
    match &pr.value {
        PropValue::One(Expr::Str(s)) => Ok(s.to_string()),
        PropValue::One(Expr::Ident(n)) => Ok(n.to_string()),
        _ => Err(prop_err(
            pr,
            format!("game2d prop '{}' must be a name or string literal", pr.name),
        )),
    }
}

/// `game2d { ... }` 块解析（node 块循环按关键字 `game2d` 分派进入）。
pub fn parse_game2d_block(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // game2d
    cx.expect(TokenKind::LBrace)?;
    let mut audio: Option<Game2dAudio> = None;
    let mut cues: Vec<Game2dCue> = Vec::new();
    let mut hud: Option<Game2dHud> = None;
    let mut triggers: Vec<Game2dTrigger> = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        match head.as_str() {
            "audio" => {
                if audio.is_some() {
                    return cursor_err(cx, "duplicate audio block in game2d".to_string());
                }
                audio = Some(parse_audio(cx)?);
            }
            "cue" => cues.push(parse_cue(cx)?),
            "hud" => {
                if hud.is_some() {
                    return cursor_err(cx, "duplicate hud block in game2d".to_string());
                }
                hud = Some(parse_hud(cx)?);
            }
            "trigger" => triggers.push(parse_trigger(cx)?),
            other => {
                return cursor_err(
                    cx,
                    format!(
                        "unsupported game2d entry '{other}' (research surface: audio/cue/hud/trigger)"
                    ),
                );
            }
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(Game2dBlock {
        audio,
        cues,
        hud,
        triggers,
        span: super::ast::span_of(start),
    })
}

fn parse_audio(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dAudio> {
    let start = cx.cur_pos();
    cx.advance(); // audio
    let props = parse_prop_block(cx)?;
    let mut bank: Option<String> = None;
    let mut font: Option<String> = None;
    let mut license: Option<String> = None;
    for pr in &props {
        match pr.name.as_str() {
            "bank" => bank = Some(str_of(pr)?),
            "font" => font = Some(str_of(pr)?),
            "license" => license = Some(str_of(pr)?),
            other => {
                return Err(prop_err(
                    pr,
                    format!("unknown game2d audio prop '{other}' (bank/font/license)"),
                ))
            }
        }
    }
    let bank = match bank {
        Some(b) => b,
        None => return cursor_err(cx, "game2d audio missing 'bank: \"<wav path>\"'".to_string()),
    };
    let font = match font {
        Some(f) => f,
        None => return cursor_err(cx, "game2d audio missing 'font: \"<ttf path>\"'".to_string()),
    };
    let license = match license {
        Some(l) => l,
        None => return cursor_err(cx, "game2d audio missing 'license: \"<txt path>\"'".to_string()),
    };
    Ok(Game2dAudio {
        bank,
        font,
        license,
        span: super::ast::span_of(start),
    })
}

fn parse_cue(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dCue> {
    let start = cx.cur_pos();
    cx.advance(); // cue
    let name = cx.cur_text();
    if name.is_empty() {
        return cursor_err(cx, "cue name expected".to_string());
    }
    cx.advance();
    let props = parse_prop_block(cx)?;
    let mut span_pair: Option<(i64, i64)> = None;
    let mut gain = 1.0;
    for pr in &props {
        match pr.name.as_str() {
            "span" => span_pair = Some(ipair2(pr)?),
            "gain" => gain = num_of(pr)?,
            other => {
                return Err(prop_err(
                    pr,
                    format!("unknown game2d cue prop '{other}' (span/gain)"),
                ))
            }
        }
    }
    let (start_frame, end_frame) = match span_pair {
        Some(p) => p,
        None => {
            return cursor_err(
                cx,
                format!("game2d cue '{name}' missing 'span: (start_frame, end_frame)'"),
            );
        }
    };
    Ok(Game2dCue {
        name,
        start_frame,
        end_frame,
        gain,
        span: super::ast::span_of(start),
    })
}

fn parse_hud(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dHud> {
    let start = cx.cur_pos();
    cx.advance(); // hud
    cx.expect(TokenKind::LBrace)?;
    let mut slots: Vec<Game2dSlot> = Vec::new();
    let mut texts: Vec<Game2dText> = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        match head.as_str() {
            "slot" => slots.push(parse_slot(cx)?),
            "text" => texts.push(parse_text(cx)?),
            other => {
                return cursor_err(
                    cx,
                    format!(
                        "unsupported game2d hud entry '{other}' (research surface: slot/text)"
                    ),
                );
            }
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(Game2dHud {
        slots,
        texts,
        span: super::ast::span_of(start),
    })
}

fn parse_slot(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dSlot> {
    let start = cx.cur_pos();
    cx.advance(); // slot
    let idx_text = cx.cur_text();
    let slot: i64 = match idx_text.parse() {
        Ok(v) => v,
        Err(_) => {
            return cursor_err(
                cx,
                format!("game2d slot index must be an integer, got '{idx_text}'"),
            );
        }
    };
    cx.advance();
    let props = parse_prop_block(cx)?;
    let mut anchor = 0i64;
    let mut offset = (8.0, 8.0);
    let mut size = 32.0;
    let mut rgba = (1.0, 1.0, 1.0, 1.0);
    let mut text: Option<String> = None;
    let mut init: Option<String> = None;
    for pr in &props {
        match pr.name.as_str() {
            "anchor" => anchor = int_of(pr)?,
            "offset" => offset = pair2(pr)?,
            "size" => size = num_of(pr)?,
            "rgba" => rgba = pair4(pr)?,
            "text" => text = Some(str_of(pr)?),
            "init" => init = Some(str_of(pr)?),
            other => {
                return Err(prop_err(
                    pr,
                    format!(
                        "unknown game2d slot prop '{other}' (anchor/offset/size/rgba/text/init)"
                    ),
                ))
            }
        }
    }
    Ok(Game2dSlot {
        slot,
        anchor,
        offset,
        size,
        rgba,
        text,
        init,
        span: super::ast::span_of(start),
    })
}

fn parse_text(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dText> {
    let start = cx.cur_pos();
    cx.advance(); // text
    let field = cx.cur_text();
    if field.is_empty() {
        return cursor_err(cx, "game2d text field name expected".to_string());
    }
    cx.advance();
    cx.expect(TokenKind::LBrace)?;
    let mut entries: Vec<(i64, String)> = Vec::new();
    let mut default: Option<String> = None;
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let key = cx.cur_text();
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        let value = match cx.parse_expr()? {
            Expr::Str(s) => s.to_string(),
            other => {
                return cursor_err(
                    cx,
                    format!("game2d text value must be a string literal, got {other:?}"),
                );
            }
        };
        if key == "default" {
            default = Some(value);
        } else {
            let k: i64 = match key.parse() {
                Ok(v) => v,
                Err(_) => {
                    return cursor_err(
                        cx,
                        format!("game2d text key must be an integer or 'default', got '{key}'"),
                    );
                }
            };
            entries.push((k, value));
        }
    }
    cx.expect(TokenKind::RBrace)?;
    let default = match default {
        Some(d) => d,
        None => {
            return cursor_err(
                cx,
                format!("game2d text '{field}' missing 'default: \"<str>\"'"),
            );
        }
    };
    Ok(Game2dText {
        field,
        entries,
        default,
        span: super::ast::span_of(start),
    })
}

fn parse_trigger(cx: &mut dyn DomainParseContext) -> AutoResult<Game2dTrigger> {
    let start = cx.cur_pos();
    cx.advance(); // trigger
    let name = cx.cur_text();
    if name.is_empty() {
        return cursor_err(cx, "trigger name expected".to_string());
    }
    cx.advance();
    let props = parse_prop_block(cx)?;
    let mut shape: Option<Game2dShape> = None;
    let mut target: Option<String> = None;
    let mut event: Option<String> = None;
    let mut on: Option<String> = None;
    let mut guard: Option<String> = None;
    let mut cue: Option<String> = None;
    let mut full_at: Option<i64> = None;
    let mut full_cue: Option<String> = None;
    let mut value: Option<i64> = None;
    let mut when_count: Option<i64> = None;
    let mut phase: Option<i64> = None;
    for pr in &props {
        match pr.name.as_str() {
            "shape" => {
                let kind = str_of(pr)?;
                shape = match kind.as_str() {
                    "circle" => Some(Game2dShape::Circle(0.0)),
                    "box" => Some(Game2dShape::Box),
                    other => {
                        return Err(prop_err(
                            pr,
                            format!("game2d trigger shape must be circle|box, got '{other}'"),
                        ));
                    }
                };
            }
            "radius" => {
                let r = num_of(pr)?;
                if matches!(shape, Some(Game2dShape::Box)) {
                    return Err(prop_err(
                        pr,
                        "game2d trigger 'radius' is circle-only (box uses the instance geometry)"
                            .to_string(),
                    ));
                }
                shape = Some(Game2dShape::Circle(r));
            }
            "target" => target = Some(str_of(pr)?),
            "event" => event = Some(str_of(pr)?),
            "on" => on = Some(str_of(pr)?),
            "guard" => guard = Some(str_of(pr)?),
            "cue" => cue = Some(str_of(pr)?),
            "full_at" => full_at = Some(int_of(pr)?),
            "full_cue" => full_cue = Some(str_of(pr)?),
            "value" => value = Some(int_of(pr)?),
            "when_count" => when_count = Some(int_of(pr)?),
            "phase" => phase = Some(int_of(pr)?),
            other => {
                return Err(prop_err(
                    pr,
                    format!(
                        "unknown game2d trigger prop '{other}' (shape/radius/target/event/on/guard/cue/full_at/full_cue/value/when_count/phase)"
                    ),
                ));
            }
        }
    }
    let shape = match shape {
        Some(s) => s,
        None => {
            return cursor_err(
                cx,
                format!("game2d trigger '{name}' missing 'shape: circle|box'"),
            );
        }
    };
    if let Game2dShape::Circle(r) = shape {
        if r <= 0.0 {
            return cursor_err(
                cx,
                format!("game2d trigger '{name}' circle radius must be > 0"),
            );
        }
    }
    let target = match target {
        Some(t) => t,
        None => {
            return cursor_err(
                cx,
                format!("game2d trigger '{name}' missing 'target: <instance>'"),
            );
        }
    };
    let event = match event {
        Some(e) => e,
        None => {
            return cursor_err(
                cx,
                format!("game2d trigger '{name}' missing 'event: <msg name>'"),
            );
        }
    };
    let kind_name = on.clone().unwrap_or_default();
    let kind = match on.as_deref() {
        Some("collect") => Game2dTriggerKind::Collect {
            cue: need_str(cx, &kind_name, &name, cue.take(), "cue")?,
            full_at: need_int(cx, &kind_name, &name, full_at.take(), "full_at")?,
            full_cue: need_str(cx, &kind_name, &name, full_cue.take(), "full_cue")?,
        },
        Some("fail") => {
            if when_count.is_some() || phase.is_some() {
                return cursor_err(
                    cx,
                    format!("game2d trigger '{name}': when_count/phase are goal-only props"),
                );
            }
            Game2dTriggerKind::Fail {
                value: need_int(cx, &kind_name, &name, value.take(), "value")?,
                cue: need_str(cx, &kind_name, &name, cue.take(), "cue")?,
            }
        }
        Some("goal") => Game2dTriggerKind::Goal {
            when_count: need_int(cx, &kind_name, &name, when_count.take(), "when_count")?,
            phase: phase.take().unwrap_or(1),
            value: need_int(cx, &kind_name, &name, value.take(), "value")?,
            cue: need_str(cx, &kind_name, &name, cue.take(), "cue")?,
        },
        other => {
            return cursor_err(
                cx,
                format!("game2d trigger '{name}' on: must be collect|fail|goal, got '{other:?}'"),
            );
        }
    };
    Ok(Game2dTrigger {
        name,
        shape,
        target,
        event,
        kind,
        guard,
        span: super::ast::span_of(start),
    })
}
