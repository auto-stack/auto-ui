//! `presentation2d` 作者入口块（auto-ui PLAN-025 T-02/T-03，presentation-2d
//! profile）。
//!
//! 通用二维 Node 声音/HUD 接线的声明面：typed 资源（bank/cue/font）、HUD
//! 槽到 owner 实例的绑定、generic 触发行（无业务效果类别）。只服务 pilot
//! 的 presentation 降层（`--profile presentation-2d`）；不进 `UiCode`，
//! 不改变生产 widget/store 文法。文法（块内属性行，空行/逗号宽容）：
//!
//! ```text
//! presentation2d {
//!     audio {
//!         bank: "<entry-relative wav path>"
//!         font: "<entry-relative ttf path>"
//!         license: "<entry-relative license text path>"
//!     }
//!     cue <name> { span: (start_frame, end_frame) gain: <0.0..=1.0> }
//!     hud {
//!         slot <0..3> {
//!             owner: <root-view instance>      // 槽的唯一 owner（必填）
//!             anchor: <0..3>?  offset: (x, y)?  size: <16..48>?  rgba: (r,g,b,a)?
//!             text_fn: <author fn>?  text_field: <owner int field>?   // 文字绑定对
//!             visible: <owner int field>?      // 槽可见性字段（缺省恒可见）
//!             init: "<initial>"?
//!         }
//!     }
//!     trigger <name> {
//!         shape: circle radius: <r>   |   shape: box
//!         target: <root-view instance>
//!         event: <msg 名>              // 目标定义的成员事件（接收方派发名）
//!         guard: <target model field>?
//!     }
//! }
//! ```
//!
//! 冻结语义（T-02 §9；与 game2d 的差别即本计划的功能面）：
//! - 无 `on: collect|fail|goal`、无 cue/full_at/full_cue/value/when_count/
//!   phase/`text <field> {...}` 业务载荷——计数/销毁/胜败/文字/cue 选择/
//!   gain 全部由作者普通 model/msg/on/helper 决定；编译器不内置任何游戏
//!   业务枚举，也不要求根声明 count/state。
//! - 触发转发：pair = trigger 声明序（0..7）；事件 tick 由生成 poll 前奏
//!   派发 `<leaf>_<event 小写>(j, phase)`（phase 1 enter / 2 stay / 3 exit
//!   作为 handler payload 转发）；guard 为目标定义 model 字段（==1 才派发）。
//! - 声音请求：作者 handler 内 `sfx.play(<cue>, <gain>)`；cue 名在本块
//!   cue 表内解析为 scene cue id（typed 映射，作者不写数字 ID）；gain 为
//!   任意 float 表达式（[0,1] 域由运行时 `_ag_audio_put` fault 54 强制）。
//! - 槽 owner：恰一根视图组件实例；同实例不得绑定两槽（具名拒绝）。

use auto_parser_api::error::{pos_to_span, span_from, AutoError, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;
use auto_parser_api::DomainParseContext;

use super::ast::{LightSpan, Prop, PropValue};
use super::game::{parse_audio, parse_cue, Game2dAudio, Game2dCue, Game2dShape};
use super::parse::parse_prop_block;

/// `presentation2d { ... }` 完整块。
#[derive(Debug, Clone)]
pub struct Presentation2dBlock {
    pub audio: Option<Game2dAudio>,
    pub cues: Vec<Game2dCue>,
    pub hud: Option<PresHud>,
    pub triggers: Vec<PresTrigger>,
    pub span: LightSpan,
}

/// `hud { slot N {...} }`：槽绑定（作者 helper 文字，无业务映射表）。
#[derive(Debug, Clone)]
pub struct PresHud {
    pub slots: Vec<PresSlot>,
    pub span: LightSpan,
}

/// 一个 HUD 槽：版面 + owner 绑定 + 可选文字/可见性绑定。
#[derive(Debug, Clone)]
pub struct PresSlot {
    pub slot: i64,
    pub anchor: i64,
    pub offset: (f64, f64),
    pub size: f64,
    pub rgba: (f64, f64, f64, f64),
    /// owner：根视图组件实例名（唯一槽属主）。
    pub owner: String,
    /// 文字绑定对：作者 helper fn（int -> str）+ owner 的 int model 字段。
    pub text_fn: Option<String>,
    pub text_field: Option<String>,
    /// 可见性：owner 的 int model 字段（缺省恒可见）。
    pub visible: Option<String>,
    pub init: Option<String>,
    pub span: LightSpan,
}

/// `trigger <name> { ... }` 一行（= 一个触发对）：仅 typed 几何/目标/事件。
#[derive(Debug, Clone)]
pub struct PresTrigger {
    pub name: String,
    pub shape: Game2dShape,
    pub target: String,
    /// 目标定义的成员事件名（接收方派发 `<leaf>_<event 小写>`）。
    pub event: String,
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

fn ident_of(pr: &Prop) -> AutoResult<String> {
    match &pr.value {
        PropValue::One(auto_parser_api::ast::Expr::Ident(n)) => Ok(n.to_string()),
        PropValue::One(auto_parser_api::ast::Expr::Str(s)) => Ok(s.to_string()),
        _ => Err(prop_err(
            pr,
            format!("presentation2d prop '{}' must be a name", pr.name),
        )),
    }
}

/// `presentation2d { ... }` 块解析（node 块循环按关键字 `presentation2d`
/// 分派进入）。
pub fn parse_presentation2d_block(
    cx: &mut dyn DomainParseContext,
) -> AutoResult<Presentation2dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // presentation2d
    cx.expect(TokenKind::LBrace)?;
    let mut audio: Option<Game2dAudio> = None;
    let mut cues: Vec<Game2dCue> = Vec::new();
    let mut hud: Option<PresHud> = None;
    let mut triggers: Vec<PresTrigger> = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        match head.as_str() {
            "audio" => {
                if audio.is_some() {
                    return cursor_err(cx, "duplicate audio block in presentation2d".to_string());
                }
                audio = Some(parse_audio(cx)?);
            }
            "cue" => cues.push(parse_cue(cx)?),
            "hud" => {
                if hud.is_some() {
                    return cursor_err(cx, "duplicate hud block in presentation2d".to_string());
                }
                hud = Some(parse_pres_hud(cx)?);
            }
            "trigger" => triggers.push(parse_pres_trigger(cx)?),
            // 业务效果类别不属于本块：game2d 专有的 on:/cue:/value:/...
            // 触发载荷在此具名拒绝（防业务枚举回流）。
            other => {
                return cursor_err(
                    cx,
                    format!(
                        "unsupported presentation2d entry '{other}' (research surface: audio/cue/hud/trigger; business trigger payloads are game2d-only)"
                    ),
                );
            }
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(Presentation2dBlock {
        audio,
        cues,
        hud,
        triggers,
        span: super::ast::span_of(start),
    })
}

fn parse_pres_hud(cx: &mut dyn DomainParseContext) -> AutoResult<PresHud> {
    let start = cx.cur_pos();
    cx.advance(); // hud
    cx.expect(TokenKind::LBrace)?;
    let mut slots: Vec<PresSlot> = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        match head.as_str() {
            "slot" => slots.push(parse_pres_slot(cx)?),
            // game2d 的 `text <field> {..}` 分支字面量映射表不是本块文法：
            // 文字由作者 helper 决定（具名拒绝，防业务映射回流）。
            "text" => {
                return cursor_err(
                    cx,
                    "presentation2d hud has no 'text <field> {..}' business map (bind text_fn/text_field on the slot instead)".to_string(),
                );
            }
            other => {
                return cursor_err(
                    cx,
                    format!(
                        "unsupported presentation2d hud entry '{other}' (research surface: slot)"
                    ),
                );
            }
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(PresHud {
        slots,
        span: super::ast::span_of(start),
    })
}

fn parse_pres_slot(cx: &mut dyn DomainParseContext) -> AutoResult<PresSlot> {
    let start = cx.cur_pos();
    cx.advance(); // slot
    let idx_text = cx.cur_text();
    let slot: i64 = match idx_text.parse() {
        Ok(v) => v,
        Err(_) => {
            return cursor_err(
                cx,
                format!("presentation2d slot index must be an integer, got '{idx_text}'"),
            );
        }
    };
    cx.advance();
    let props = parse_prop_block(cx)?;
    let mut anchor = 0i64;
    let mut offset = (8.0, 8.0);
    let mut size = 32.0;
    let mut rgba = (1.0, 1.0, 1.0, 1.0);
    let mut owner: Option<String> = None;
    let mut text_fn: Option<String> = None;
    let mut text_field: Option<String> = None;
    let mut visible: Option<String> = None;
    let mut init: Option<String> = None;
    for pr in &props {
        match pr.name.as_str() {
            "anchor" => anchor = super::game::int_of(pr)?,
            "offset" => offset = super::game::pair2(pr)?,
            "size" => size = super::game::num_of(pr)?,
            "rgba" => rgba = super::game::pair4(pr)?,
            "owner" => owner = Some(ident_of(pr)?),
            "text_fn" => text_fn = Some(ident_of(pr)?),
            "text_field" => text_field = Some(ident_of(pr)?),
            "visible" => visible = Some(ident_of(pr)?),
            "init" => init = Some(super::game::str_of(pr)?),
            other => {
                return Err(prop_err(
                    pr,
                    format!(
                        "unknown presentation2d slot prop '{other}' (owner/anchor/offset/size/rgba/text_fn/text_field/visible/init)"
                    ),
                ))
            }
        }
    }
    let owner = match owner {
        Some(o) => o,
        None => {
            return cursor_err(
                cx,
                format!("presentation2d slot {slot} missing 'owner: <instance>'"),
            );
        }
    };
    // 文字绑定成对出现：text_fn 与 text_field 必须同给。
    match (text_fn.as_ref(), text_field.as_ref()) {
        (Some(_), Some(_)) | (None, None) => {}
        (Some(f), None) => {
            return cursor_err(
                cx,
                format!("presentation2d slot {slot}: text_fn '{f}' needs its paired 'text_field: <owner int field>'"),
            );
        }
        (None, Some(t)) => {
            return cursor_err(
                cx,
                format!("presentation2d slot {slot}: text_field '{t}' needs its paired 'text_fn: <author fn>'"),
            );
        }
    }
    Ok(PresSlot {
        slot,
        anchor,
        offset,
        size,
        rgba,
        owner,
        text_fn,
        text_field,
        visible,
        init,
        span: super::ast::span_of(start),
    })
}

fn parse_pres_trigger(cx: &mut dyn DomainParseContext) -> AutoResult<PresTrigger> {
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
    let mut guard: Option<String> = None;
    for pr in &props {
        match pr.name.as_str() {
            "shape" => {
                let kind = super::game::str_of(pr)?;
                shape = match kind.as_str() {
                    "circle" => Some(Game2dShape::Circle(0.0)),
                    "box" => Some(Game2dShape::Box),
                    other => {
                        return Err(prop_err(
                            pr,
                            format!("presentation2d trigger shape must be circle|box, got '{other}'"),
                        ));
                    }
                };
            }
            "radius" => {
                let r = super::game::num_of(pr)?;
                if matches!(shape, Some(Game2dShape::Box)) {
                    return Err(prop_err(
                        pr,
                        "presentation2d trigger 'radius' is circle-only (box uses the instance geometry)"
                            .to_string(),
                    ));
                }
                shape = Some(Game2dShape::Circle(r));
            }
            "target" => target = Some(super::game::str_of(pr)?),
            "event" => event = Some(super::game::str_of(pr)?),
            "guard" => guard = Some(super::game::str_of(pr)?),
            // 业务效果类别（game2d 专有）在本块具名拒绝。
            "on" | "cue" | "full_at" | "full_cue" | "value" | "when_count" | "phase" => {
                return Err(prop_err(
                    pr,
                    format!(
                        "presentation2d trigger prop '{}' is business payload (game2d-only); the author handler decides effects via sfx.play/model logic",
                        pr.name
                    ),
                ))
            }
            other => {
                return Err(prop_err(
                    pr,
                    format!(
                        "unknown presentation2d trigger prop '{other}' (shape/radius/target/event/guard)"
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
                format!("presentation2d trigger '{name}' missing 'shape: circle|box'"),
            );
        }
    };
    if let Game2dShape::Circle(r) = shape {
        if r <= 0.0 {
            return cursor_err(
                cx,
                format!("presentation2d trigger '{name}' circle radius must be > 0"),
            );
        }
    }
    let target = match target {
        Some(t) => t,
        None => {
            return cursor_err(
                cx,
                format!("presentation2d trigger '{name}' missing 'target: <instance>'"),
            );
        }
    };
    let event = match event {
        Some(e) => e,
        None => {
            return cursor_err(
                cx,
                format!("presentation2d trigger '{name}' missing 'event: <msg name>'"),
            );
        }
    };
    Ok(PresTrigger {
        name,
        shape,
        target,
        event,
        guard,
        span: super::ast::span_of(start),
    })
}
