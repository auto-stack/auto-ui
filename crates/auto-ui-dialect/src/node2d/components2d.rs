//! `components2d` 作者入口块（auto-ui PLAN-032 T-04，components-2d-runtime
//! profile）。
//!
//! 完整二维游戏组合的声明面：在 presentation2d 的全部文法（audio/cue/hud/
//! trigger，逐字相同语义）之上增加有限的游戏组合行。业务逻辑仍是作者普通
//! model/msg/on/helper——本块没有任何业务枚举；冻结的只有生成骨架
//! （T-02 冻结合同：快照 poll / 批销毁服务 / 巡逻 / 角色物理 / 帧收口 /
//! 终局 Enter）。文法（块内属性行，空行/逗号宽容）：
//!
//! ```text
//! components2d {
//!     audio { ... }                       // 与 presentation2d 逐字相同
//!     cue <name> { span: (s,e) gain: <g> }
//!     hud { slot <0..3> { ... } }
//!     trigger <name> { shape: ... target: ... event: <msg> guard: <field>? }
//!     spawn <name> { template: <root-view instance> slot: <0..7> at: (x, y) }
//!     patrol { node: <instance name> range: (from, to) at_y: <y> speed: <s> }
//!     atlas {
//!         asset: "<entry-relative png path>"
//!         frame_size: (w, h)
//!         idle: (x, y)  run: (x, y)  rise: (x, y)  fall: (x, y)
//!     }
//! }
//! ```
//!
//! 冻结语义（PLAN-032 T-02/T-04）：
//! - 根 model 必须声明 `count: int` 与 `state: int`（缺一具名拒绝；语义
//!   冻结 0=Playing/1=Won/2=Failed）；生成基础设施名 gate/np/o0..o3/c0..c3
//!   等由降层注入（作者声明同名 → 具名拒绝）。
//! - spawn 行：pair 池之外的实例表（slot 0..7，模板 = 根视图实例的动态
//!   shape2d）；`<name>_handle` SoA 由生成器注入。
//! - patrol 行：恰一巡逻行驱动一个 spawn 实例的持续位移（生成 Life.Tick
//!   段，任何状态为 Playing 的回调恰好一次 ag_object_move）。
//! - atlas 行：角色四帧（idle/run/rise/fall）atlas 子矩形；生成收口
//!   bundle 以 kind=0 携带当前帧 + flip。
//! - 作者 handler 内 `sfx.play(<cue>, <gain>)` 沿 025；`obj.despawn(<spawn
//!   name>)` 生成一条 ag_object_despawn_batch 单元素批调用（1 credit）。
//! - 触发转发沿 025：pair = trigger 声明序；生成 poll 前奏每回调恰一次
//!   ag_events_snapshot，事件 i 派发 `<leaf>_<event 小写>(i, phase)`。

use auto_parser_api::error::{pos_to_span, span_from, AutoError, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;
use auto_parser_api::DomainParseContext;

use super::ast::{LightSpan, Prop, PropValue};
use super::game::{parse_audio, parse_cue, Game2dAudio, Game2dCue};
use super::presentation::{parse_pres_hud, parse_pres_trigger, PresHud, PresTrigger};

/// `components2d { ... }` 完整块。
#[derive(Debug, Clone)]
pub struct Components2dBlock {
    pub audio: Option<Game2dAudio>,
    pub cues: Vec<Game2dCue>,
    pub hud: Option<PresHud>,
    pub triggers: Vec<PresTrigger>,
    pub spawns: Vec<CmpSpawn>,
    pub patrol: Option<CmpPatrol>,
    pub atlas: Option<CmpAtlas>,
    pub span: LightSpan,
}

/// `spawn <name> { template slot at }`：一个运行时实例行。
#[derive(Debug, Clone)]
pub struct CmpSpawn {
    pub name: String,
    /// 模板：根视图实例名（须恰一动态 shape2d）。
    pub template: String,
    /// 保留槽 0..7。
    pub slot: i64,
    pub at: (f64, f64),
    pub span: LightSpan,
}

/// `patrol { node span at_y speed }`：持续巡逻行（恰一）。
#[derive(Debug, Clone)]
pub struct CmpPatrol {
    /// 巡逻的 spawn 行名。
    pub node: String,
    pub from: f64,
    pub to: f64,
    pub at_y: f64,
    /// 单位/秒（G2 冻结 1.0）。
    pub speed: f64,
    pub span: LightSpan,
}

/// `atlas { asset frame_size idle run rise fall }`：角色四帧。
#[derive(Debug, Clone)]
pub struct CmpAtlas {
    pub asset: String,
    /// (w, h) 每帧子矩形大小。
    pub frame_size: (i64, i64),
    /// 每帧 (x, y) 原点。
    pub idle: (i64, i64),
    pub run: (i64, i64),
    pub rise: (i64, i64),
    pub fall: (i64, i64),
    pub span: LightSpan,
}

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
        _ => Err(SyntaxError::Generic {
            message: format!("components2d prop '{}' must be a name", pr.name),
            span: span_from(pr.span.0, pr.span.1),
        }
        .into()),
    }
}

/// `components2d { ... }` 块解析（node 块循环按关键字 `components2d`
/// 分派进入）。
pub fn parse_components2d_block(cx: &mut dyn DomainParseContext) -> AutoResult<Components2dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // components2d
    cx.expect(TokenKind::LBrace)?;
    let mut audio: Option<Game2dAudio> = None;
    let mut cues: Vec<Game2dCue> = Vec::new();
    let mut hud: Option<PresHud> = None;
    let mut triggers: Vec<PresTrigger> = Vec::new();
    let mut spawns: Vec<CmpSpawn> = Vec::new();
    let mut patrol: Option<CmpPatrol> = None;
    let mut atlas: Option<CmpAtlas> = None;
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        match head.as_str() {
            "audio" => {
                if audio.is_some() {
                    return cursor_err(cx, "duplicate audio block in components2d".to_string());
                }
                audio = Some(parse_audio(cx)?);
            }
            "cue" => cues.push(parse_cue(cx)?),
            "hud" => {
                if hud.is_some() {
                    return cursor_err(cx, "duplicate hud block in components2d".to_string());
                }
                hud = Some(parse_pres_hud(cx)?);
            }
            "trigger" => triggers.push(parse_pres_trigger(cx)?),
            "spawn" => spawns.push(parse_spawn(cx)?),
            "patrol" => {
                if patrol.is_some() {
                    return cursor_err(cx, "duplicate patrol row in components2d".to_string());
                }
                patrol = Some(parse_patrol(cx)?);
            }
            "atlas" => {
                if atlas.is_some() {
                    return cursor_err(cx, "duplicate atlas block in components2d".to_string());
                }
                atlas = Some(parse_atlas(cx)?);
            }
            other => {
                return cursor_err(
                    cx,
                    format!(
                        "unsupported components2d entry '{other}' (research surface: audio/cue/hud/trigger/spawn/patrol/atlas)"
                    ),
                );
            }
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(Components2dBlock {
        audio,
        cues,
        hud,
        triggers,
        spawns,
        patrol,
        atlas,
        span: super::ast::span_of(start),
    })
}

fn parse_spawn(cx: &mut dyn DomainParseContext) -> AutoResult<CmpSpawn> {
    let start = cx.cur_pos();
    cx.advance(); // spawn
    let name = cx.cur_text();
    if name.is_empty() || !name.chars().next().unwrap().is_ascii_lowercase() {
        return cursor_err(cx, format!("components2d spawn name must be an identifier, got '{name}'"));
    }
    cx.advance();
    let props = super::parse::parse_prop_block(cx)?;
    let mut template: Option<String> = None;
    let mut slot: Option<i64> = None;
    let mut at: Option<(f64, f64)> = None;
    for pr in &props {
        match pr.name.as_str() {
            "template" => template = Some(ident_of(pr)?),
            "slot" => slot = Some(super::game::int_of(pr)?),
            "at" => at = Some(super::game::pair2(pr)?),
            other => {
                return cursor_err(
                    cx,
                    format!("unsupported components2d spawn prop '{other}' (surface: template/slot/at)"),
                );
            }
        }
    }
    let template = template.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d spawn requires 'template'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    let slot = slot.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d spawn requires 'slot'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    let at = at.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d spawn requires 'at'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    if !(0..=7).contains(&slot) {
        return cursor_err(cx, format!("components2d spawn slot must be 0..7, got {slot}"));
    }
    Ok(CmpSpawn {
        name,
        template,
        slot,
        at,
        span: super::ast::span_of(start),
    })
}

fn parse_patrol(cx: &mut dyn DomainParseContext) -> AutoResult<CmpPatrol> {
    let start = cx.cur_pos();
    cx.advance(); // patrol
    let props = super::parse::parse_prop_block(cx)?;
    let mut node: Option<String> = None;
    let mut span: Option<(f64, f64)> = None;
    let mut at_y: Option<f64> = None;
    let mut speed: Option<f64> = None;
    for pr in &props {
        match pr.name.as_str() {
            "node" => node = Some(ident_of(pr)?),
            "range" => span = Some(super::game::pair2(pr)?),
            "at_y" => at_y = Some(super::game::num_of(pr)?),
            "speed" => speed = Some(super::game::num_of(pr)?),
            other => {
                return cursor_err(
                    cx,
                    format!("unsupported components2d patrol prop '{other}' (surface: node/range/at_y/speed)"),
                );
            }
        }
    }
    let node = node.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d patrol requires 'node'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    let (from, to) = span.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d patrol requires 'range'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    let at_y = at_y.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d patrol requires 'at_y'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    let speed = speed.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d patrol requires 'speed'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    if from >= to {
        return cursor_err(cx, format!("components2d patrol range must be from<to, got ({from},{to})"));
    }
    if speed <= 0.0 {
        return cursor_err(cx, format!("components2d patrol speed must be > 0, got {speed}"));
    }
    Ok(CmpPatrol {
        node,
        from,
        to,
        at_y,
        speed,
        span: super::ast::span_of(start),
    })
}

fn parse_atlas(cx: &mut dyn DomainParseContext) -> AutoResult<CmpAtlas> {
    let start = cx.cur_pos();
    cx.advance(); // atlas
    cx.expect(TokenKind::LBrace)?;
    let mut asset: Option<String> = None;
    let mut frame_size: Option<(i64, i64)> = None;
    let mut frames: [(i64, i64); 4] = [(0, 0); 4];
    let mut seen = [false; 4];
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head_pos = cx.cur_pos();
        let head = cx.cur_text();
        if head.is_empty() {
            return cursor_err(cx, "components2d atlas property name expected".to_string());
        }
        let idx = match head.as_str() {
            "asset" | "frame_size" | "idle" | "run" | "rise" | "fall" => 0,
            other => {
                return cursor_err(
                    cx,
                    format!(
                        "unsupported components2d atlas entry '{other}' (surface: asset/frame_size/idle/run/rise/fall)"
                    ),
                );
            }
        };
        let _ = idx;
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        let value = super::parse::parse_prop_value(cx)?;
        let pr = Prop {
            name: head.clone(),
            value,
            span: super::ast::span_of(head_pos),
        };
        match head.as_str() {
            "asset" => asset = Some(ident_of(&pr)?),
            "frame_size" => {
                let (w, h) = super::game::pair2(&pr)?;
                if w <= 0.0 || h <= 0.0 {
                    return cursor_err(cx, "components2d atlas frame_size must be positive".to_string());
                }
                frame_size = Some((w as i64, h as i64));
            }
            "idle" | "run" | "rise" | "fall" => {
                let slot = match head.as_str() {
                    "idle" => 0usize,
                    "run" => 1,
                    "rise" => 2,
                    _ => 3,
                };
                if seen[slot] {
                    return cursor_err(cx, format!("duplicate components2d atlas frame '{head}'"));
                }
                seen[slot] = true;
                let (x, y) = super::game::pair2(&pr)?;
                if x < 0.0 || y < 0.0 {
                    return cursor_err(
                        cx,
                        "components2d atlas frame origin must be non-negative".to_string(),
                    );
                }
                frames[slot] = (x as i64, y as i64);
            }
            _ => unreachable!(),
        }
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RBrace)?;
    let asset = asset.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d atlas requires 'asset'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    let frame_size = frame_size.ok_or_else(|| -> AutoError {
        SyntaxError::Generic {
            message: "components2d atlas requires 'frame_size'".to_string(),
            span: span_from(start.pos, start.pos),
        }
        .into()
    })?;
    if !seen.iter().all(|b| *b) {
        return cursor_err(cx, "components2d atlas requires idle/run/rise/fall frames".to_string());
    }
    Ok(CmpAtlas {
        asset,
        frame_size,
        idle: frames[0],
        run: frames[1],
        rise: frames[2],
        fall: frames[3],
        span: super::ast::span_of(start),
    })
}
