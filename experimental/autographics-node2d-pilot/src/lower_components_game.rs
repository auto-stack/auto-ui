//! PLAN-023 components-2d-game profile（T-03：作者入口 → 生成器）。
//!
//! 把 T-02 手扩证明的世界（scratch t02/gameplay/world-src/world-game.at）
//! 变成 GENERATOR 产物：`game2d` 作者块（触发表/cue 表/HUD 槽/音频资源）
//! 驱动 F-2..F-6 冻结骨架的机器生成。生成面（全部在共享 components 管线
//! 上的增量）：
//! 1. F-4 根合同：作者 model 必须声明 `count`/`state`（缺失具名拒绝，
//!    语义冻结 0=Playing/1=Won/2=Failed）；生成基础设施 `gate`/`ord`/
//!    `np`/`p0`/`p1` 由本档案注入（作者声明同名 → 具名拒绝——生成器
//!    不发明游戏状态）。root 标量序 = 作者字段（声明序）+ 注入字段。
//! 2. F-2a 触发绑定：每个 trigger 声明行一条 `ag_trigger_bind`（pair =
//!    行序 0..7，subject 恒为唯一 character 实例的 scene id，shape/
//!    radius 来自 authoring，source 恒为实例 handle），app_start 内
//!    紧跟 spawns。
//! 3. F-2b 接收方补全（F-8.2 的 game 修复面）：为每个声明 msg handler
//!    发 `fn <leaf>_<ev>(j, p..)`（root 成员为 `fn <root>_<ev>(p..)`），
//!    不再要求作者书写 emit 站点。frozen components-2d 保持 020 字节
//!    （emit 站点驱动命名不变）。
//! 4. F-2c/F-6 帧骨架：`main_lifetick` 以 `game_events()` poll 前奏开头
//!    （每回调恰一次 `ag_event_next` 首读；事件 tick 才 `ag_event_pair`；
//!    goal 分支才 `ag_event_phase`）；latch tick 置 `r_gate=1`，根
//!    Life.Tick 体的 `.gate` 分派 `player.Idle(1)`（idle-turn 门，该回合
//!    跳过两个 held 读）；`app_update` 末尾 `game_closeout()`。
//! 5. F-3 closeout 骨架：`game_cue`（单根序数 r_ord 严格递增、≤2 pending）
//!    + `hud_<field>_text`（分支选择字面量 helper，F-5）+ `game_closeout`
//!    （每回调恰一次 `ag_present2d(kind=-1, ...)`，整体 1 计费，携带 ≤2
//!    cue + 四槽完整状态；kind 0/1 保留可用）。
//! 6. scene.agscene = autographics.scene/8：assets（ttf + wav 元数据）+
//!    顶层 cues 数组（cue id = 数组下标）+ hud{font,license,slots×4}；
//!    identity.json api = `autographics.hud2d/1`；closure/ 含模块副本 +
//!    bank/font/license 资产副本（A→B 冷再生）。
//!
//! 区域规则（F-6 region rule）：trigger 目标必须是运行时实例——目标定义
//! 须恰有一个动态 shape2d（模板行 + ag_object_spawn + handle）；静态场景
//! 叶是 frozen 角色扫移的碰撞候选，行走角色永不与其重叠（T-02 实测），
//! 违反即具名拒绝。
//!
//! 字符串/文字边界（F-5）：helper 返回分支选择字面量；裸 handler 参数作
//! 赋值 RHS 仍是解析层拒绝（不变）；helper 输出 >32 可见 ASCII 是运行时
//! fault 56（HUD 单元强制），降层不拒绝。
use crate::load::{node_of, Project};
use crate::lower_basic2d::{fmt_f, obj_json, roles_json_basic, Collected, LowerOutputBasic2D};
use crate::lower_components::{
    at, build_world, collect_scene, gen_user_source, inst_name_of, plan_dispatch, WorldIR,
};
use crate::scene::sha256_hex;
use crate::translate::LowerError;
use auto_ui_dialect::node2d::{
    EventPattern, Game2dBlock, Game2dShape, Game2dSlot, Game2dText, Game2dTrigger,
    Game2dTriggerKind,
};
use std::collections::BTreeMap;
use std::path::Path;

// --------------------------------------------------------- resolved IR ----

/// 一条已解析的触发行（世界文本与 scene 消费）。
pub(crate) struct GameTrigger {
    /// 触发对（= authoring 行序）。
    pub(crate) pair: usize,
    /// 目标实例（root-view 声明名）。
    pub(crate) target: String,
    /// 目标定义前缀（小写）与 SoA 下标。
    pub(crate) prefix: String,
    pub(crate) def_idx: usize,
    /// 接收方 fn（`<leaf>_<event 小写>`）。
    pub(crate) recv: String,
    /// guard 字段（可选；目标定义 model 字段）。
    pub(crate) guard: Option<String>,
    /// 绑定形状：Some(radius) = 圆；None = AABB（取实例几何）。
    pub(crate) circle: Option<f64>,
    /// 效果类别（cue 名已解析为 scene cue id）。
    pub(crate) kind: GameTriggerKind,
}

pub(crate) enum GameTriggerKind {
    Collect {
        cue: usize,
        full_at: i64,
        full_cue: usize,
    },
    Fail {
        value: i64,
        cue: usize,
    },
    Goal {
        when_count: i64,
        phase: i64,
        value: i64,
        cue: usize,
    },
}

/// 一个 HUD 槽的 scene 元数据 + closeout 绑定。
pub(crate) struct GameSlotOut {
    pub(crate) slot: i64,
    pub(crate) anchor: i64,
    pub(crate) offset: (f64, f64),
    pub(crate) size: f64,
    pub(crate) rgba: (f64, f64, f64, f64),
    /// closeout 绑定：可见=1 + `hud_<field>_text(r_<field>)`。
    pub(crate) text: Option<String>,
    /// scene 初始文字（未绑定为 ""，未绑定槽 visible=false）。
    pub(crate) init: String,
}

/// 一个文字映射（`fn hud_<field>_text(n int) str`）。
pub(crate) struct GameTextOut {
    pub(crate) field: String,
    pub(crate) entries: Vec<(i64, String)>,
    pub(crate) default: String,
}

/// 一个 scene cue（id = 声明序）。
pub(crate) struct GameCueOut {
    pub(crate) name: String,
    pub(crate) start_frame: i64,
    pub(crate) end_frame: i64,
    pub(crate) gain: f64,
    /// 作者声明行（具名诊断）。
    pub(crate) line: usize,
}

/// game profile 的下降期配置（gen_user_source 的增量注入面）。
pub(crate) struct GameGen {
    pub(crate) triggers: Vec<GameTrigger>,
    pub(crate) slots: Vec<GameSlotOut>,
    pub(crate) texts: Vec<GameTextOut>,
}

impl GameGen {
    /// F-8.1（game 面）：呈现缓存 `<prefix>_visq`，与 T-02 手扩世界同形
    /// （`var zone_visq=[...]` + `zone_present` 比较 visq vs vis）。
    pub(crate) fn vis_suffix() -> &'static str {
        "visq"
    }

    /// F-4：注入的根基础设施字段名（gate/ord/np/p0/p1，声明序即输出序）。
    pub(crate) fn infra_field_names(&self) -> [&'static str; 5] {
        ["gate", "ord", "np", "p0", "p1"]
    }
}

/// 资源准备产物（scene assets + closure 副本清单）。
pub struct GameAsset {
    /// scene/closure 文件名（basename）。
    pub name: String,
    pub sha256: String,
    pub bytes: usize,
    /// 源文件路径（main.rs 复制进 closure/assets/）。
    pub src: std::path::PathBuf,
    /// wav 元数据（bank）。
    pub audio: Option<(i64, i64, i64)>,
    /// scene assets[] 是否携带（license 只进 hud.license + closure）。
    pub in_scene: bool,
}

pub struct LowerOutputGame {
    pub base: LowerOutputBasic2D,
    pub assets: Vec<GameAsset>,
}

fn err0(what: String) -> LowerError {
    LowerError::Unsupported { what, line: 0 }
}

// ------------------------------------------------------------ validate ----

/// game2d 块：root-only 定位 + 提取。
fn game2d_of(project: &Project, world: &WorldIR) -> Result<Game2dBlock, LowerError> {
    for (mname, module) in &project.modules {
        for n in &module.nodes {
            let Some(g) = &n.game2d else { continue };
            let line = line_at_src(&module.source, n.span.0);
            if n.name != world.root {
                return Err(at(
                    mname,
                    line,
                    &n.name,
                    "game2d block is root-node only (the trigger table binds root-view instances)",
                ));
            }
            if mname != &world.root_module {
                return Err(at(
                    mname,
                    line,
                    &n.name,
                    "game2d block must live in the root node's module",
                ));
            }
            return Ok(g.clone());
        }
    }
    Err(at(
        &world.root_module,
        0,
        &world.root,
        "components-2d-game requires a `game2d` block in the root node (trigger table / cue table / hud slots / audio assets)",
    ))
}

/// game2d 块校验 + 触发/槽/文字解析（全部执行前具名拒绝）。
fn prepare_game(
    project: &Project,
    world: &WorldIR,
    game: &Game2dBlock,
) -> Result<GameGen, LowerError> {
    let root_def = &world.defs[&world.root];
    let root_module = world.root_module.clone();
    let root_src = project.modules[&root_module].source.clone();
    let root_line = line_at_src(&root_src, game.span.0);

    // ---- F-4 根字段所有权：count/state 作者所有；gate/ord/np/p0/p1 生成 ----
    // 检查面 = 作者 DECL（DefIR 此时已含注入字段）。
    const INFRA: [&str; 5] = ["gate", "ord", "np", "p0", "p1"];
    let root_decl = node_of(project, &world.root);
    let author_fields: Vec<String> = root_decl
        .model
        .as_ref()
        .map(|m| m.fields.iter().map(|f| f.name.to_string()).collect())
        .unwrap_or_default();
    for f in &author_fields {
        if INFRA.contains(&f.as_str()) {
            return Err(at(
                &root_module,
                root_line,
                f,
                "root model field is generated game infrastructure (gate/ord/np/p0/p1); declare only the author-owned game state 'count'/'state'",
            ));
        }
    }
    for need in ["count", "state"] {
        if !author_fields.iter().any(|f| f == need) {
            return Err(at(
                &root_module,
                root_line,
                need,
                "the game contract owns root field semantics: the root model must declare int field 'count' (0..3) and 'state' (0 Playing / 1 Won / 2 Failed)",
            ));
        }
    }

    // ---- cue 表（id = 声明序）----
    let mut cue_ids: BTreeMap<String, usize> = BTreeMap::new();
    for (i, c) in game.cues.iter().enumerate() {
        if cue_ids.insert(c.name.clone(), i).is_some() {
            return Err(at(
                &root_module,
                line_at_src(&root_src, c.span.0),
                &c.name,
                "duplicate game2d cue name (cue id = declaration order)",
            ));
        }
    }

    // ---- HUD 槽（scene 元数据 + closeout 绑定）----
    let mut slots: Vec<GameSlotOut> = Vec::new();
    if let Some(hud) = &game.hud {
        let mut seen: Vec<i64> = Vec::new();
        for s in &hud.slots {
            let sline = line_at_src(&root_src, s.span.0);
            let sym = format!("slot {}", s.slot);
            if !(0..4).contains(&s.slot) {
                return Err(at(
                    &root_module,
                    sline,
                    &sym,
                    "hud slot index must be 0..3 (exactly four slots; the closeout writes the complete four-slot state)",
                ));
            }
            if seen.contains(&s.slot) {
                return Err(at(
                    &root_module,
                    sline,
                    &sym,
                    "duplicate hud slot index",
                ));
            }
            seen.push(s.slot);
            if !(0..=3).contains(&s.anchor)
                || !(16.0..=48.0).contains(&s.size)
                || [s.offset.0, s.offset.1, s.size]
                    .iter()
                    .any(|v| !v.is_finite())
                || [s.rgba.0, s.rgba.1, s.rgba.2, s.rgba.3]
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err(at(
                    &root_module,
                    sline,
                    &sym,
                    "hud slot layout out of range (anchor 0..3, size 16..48, finite offset, rgba 0..1)",
                ));
            }
            if let Some(t) = &s.text {
                if !root_def.model.iter().any(|f| &f.0 == t) {
                    return Err(at(
                        &root_module,
                        sline,
                        t,
                        "hud slot text binds an unknown root model field (author-owned fields only)",
                    ));
                }
            }
            let init = s.init.clone().unwrap_or_default();
            if init.len() > 32 || !init.bytes().all(|b| (32..=126).contains(&b)) {
                return Err(at(
                    &root_module,
                    sline,
                    &sym,
                    "hud slot init text must be <=32 visible ASCII chars (scene-layer contract; runtime live text over 32 chars is named fault 56)",
                ));
            }
            slots.push(GameSlotOut {
                slot: s.slot,
                anchor: s.anchor,
                offset: s.offset,
                size: s.size,
                rgba: s.rgba,
                text: s.text.clone(),
                init,
            });
        }
    }
    // 恰四槽（scene 合同）：未声明槽补默认（隐藏/空文字）。
    for i in 0..4i64 {
        if !slots.iter().any(|s| s.slot == i) {
            slots.push(GameSlotOut {
                slot: i,
                anchor: i,
                offset: (8.0, 8.0),
                size: 32.0,
                rgba: (1.0, 1.0, 1.0, 1.0),
                text: None,
                init: String::new(),
            });
        }
    }
    slots.sort_by_key(|s| s.slot);

    // ---- 文字映射 ----
    let mut texts: Vec<GameTextOut> = Vec::new();
    if let Some(hud) = &game.hud {
        for t in &hud.texts {
            let tline = line_at_src(&root_src, t.span.0);
            if !root_def.model.iter().any(|f| f.0 == t.field) {
                return Err(at(
                    &root_module,
                    tline,
                    &t.field,
                    "hud text map binds an unknown root model field",
                ));
            }
            let mut seen_keys: Vec<i64> = Vec::new();
            for (k, v) in &t.entries {
                if seen_keys.contains(k) {
                    return Err(at(
                        &root_module,
                        tline,
                        &t.field,
                        &format!("duplicate hud text key {k}"),
                    ));
                }
                seen_keys.push(*k);
                if v.len() > 32 || !v.bytes().all(|b| (32..=126).contains(&b)) {
                    return Err(at(
                        &root_module,
                        tline,
                        &t.field,
                        "hud text literal must be <=32 visible ASCII chars (scene-layer contract; runtime live text over 32 chars is named fault 56)",
                    ));
                }
            }
            if t.default.len() > 32 || !t.default.bytes().all(|b| (32..=126).contains(&b)) {
                return Err(at(
                    &root_module,
                    tline,
                    &t.field,
                    "hud text default literal must be <=32 visible ASCII chars",
                ));
            }
            texts.push(GameTextOut {
                field: t.field.clone(),
                entries: t.entries.clone(),
                default: t.default.clone(),
            });
        }
        // 每个 slot 绑定必须有映射。
        for s in &slots {
            if let Some(t) = &s.text {
                if !texts.iter().any(|x| &x.field == t) {
                    return Err(at(
                        &root_module,
                        root_line,
                        t,
                        "hud slot binds a text field with no `text <field> { ... }` map",
                    ));
                }
            }
        }
    }

    // ---- 触发表（pair = 行序）----
    if game.triggers.len() > 8 {
        return Err(at(
            &root_module,
            root_line,
            "trigger",
            &format!(
                "{} trigger rows exceed the frozen 8-pair trigger registry",
                game.triggers.len()
            ),
        ));
    }
    let mut triggers = Vec::new();
    for (pair, t) in game.triggers.iter().enumerate() {
        triggers.push(resolve_trigger(project, world, t, pair, &cue_ids)?);
    }

    Ok(GameGen {
        triggers,
        slots,
        texts,
    })
}

fn resolve_trigger(
    project: &Project,
    world: &WorldIR,
    t: &Game2dTrigger,
    pair: usize,
    cue_ids: &BTreeMap<String, usize>,
) -> Result<GameTrigger, LowerError> {
    let root_module = world.root_module.clone();
    let line = line_at_src(&project.modules[&root_module].source, t.span.0);
    let cue_id = |name: &str| -> Result<usize, LowerError> {
        cue_ids.get(name).copied().ok_or_else(|| {
            at(
                &root_module,
                line,
                name,
                &format!(
                    "unknown game2d cue '{name}' (cue id out of scene range; declared: {})",
                    cue_ids.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            )
        })
    };
    // 目标必须是 root-view 运行时实例（区域规则）。
    let inst = world
        .instances
        .iter()
        .find(|i| i.parent.is_none() && inst_name_of(&i.path) == t.target)
        .ok_or_else(|| {
            at(
                &root_module,
                line,
                &t.target,
                "trigger target must be a root-view component instance (static scene leaves are collision candidates of the character sweep and can never be trigger regions)",
            )
        })?;
    let def = &world.defs[&inst.def];
    // 区域规则：目标定义须恰有一个动态 shape（= 有 spawn/handle 的运行时实例）。
    if def.dyn_elems.len() != 1 {
        return Err(at(
            &def.module,
            def.line,
            &t.target,
            &format!(
                "region rule: trigger regions must be runtime instances with exactly one dynamic shape2d (definition '{}' has {} model-bound shapes); a static scene leaf can never fire for a moving character",
                def.name,
                def.dyn_elems.len()
            ),
        ));
    }
    // guard 字段存在性。
    if let Some(g) = &t.guard {
        if !def.model.iter().any(|f| &f.0 == g) {
            return Err(at(
                &def.module,
                def.line,
                g,
                &format!("trigger guard '{g}' is not a model field of '{}'", def.name),
            ));
        }
    }
    // 事件端口存在性（接收方由 F-8.2 补全面保证）。
    let ev_lower = t.event.to_lowercase();
    if !def.msgs.contains_key(t.event.as_str()) {
        return Err(at(
            &def.module,
            def.line,
            &t.event,
            &format!(
                "trigger event '{}' is not a declared msg of '{}' (declared: {})",
                t.event,
                def.name,
                def.msgs.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    let prefix = def.prefix();
    let circle = match t.shape {
        Game2dShape::Circle(r) => Some(r),
        Game2dShape::Box => None,
    };
    let kind = match &t.kind {
        Game2dTriggerKind::Collect {
            cue,
            full_at,
            full_cue,
        } => GameTriggerKind::Collect {
            cue: cue_id(cue)?,
            full_at: *full_at,
            full_cue: cue_id(full_cue)?,
        },
        Game2dTriggerKind::Fail { value, cue } => GameTriggerKind::Fail {
            value: *value,
            cue: cue_id(cue)?,
        },
        Game2dTriggerKind::Goal {
            when_count,
            phase,
            value,
            cue,
        } => GameTriggerKind::Goal {
            when_count: *when_count,
            phase: *phase,
            value: *value,
            cue: cue_id(cue)?,
        },
    };
    Ok(GameTrigger {
        pair,
        target: t.target.clone(),
        prefix,
        def_idx: inst.def_idx,
        recv: format!("{target_lower}_{ev_lower}", target_lower = t.target.to_lowercase()),
        guard: t.guard.clone(),
        circle,
        kind,
    })
}

fn line_at_src(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1
}

// ------------------------------------------------------------- assets ----

/// 最小 RIFF/WAVE PCM16 解析（fmt + data）：格式/通道/采样率/帧数。
fn parse_wav(bytes: &[u8]) -> Result<(i64, i64, i64), String> {
    let err = |m: &str| format!("game bank wav invalid: {m}");
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(err("RIFF/WAVE header"));
    }
    let mut off = 12usize;
    let mut fmt: Option<(i64, i64, i64)> = None; // (format, channels, rate)
    let mut data_bytes: i64 = -1;
    while off + 8 <= bytes.len() {
        let id = &bytes[off..off + 4];
        let size = u32::from_le_bytes([
            bytes[off + 4],
            bytes[off + 5],
            bytes[off + 6],
            bytes[off + 7],
        ]) as usize;
        let body = off + 8;
        if body + size > bytes.len() {
            return Err(err("truncated chunk"));
        }
        match id {
            b"fmt " => {
                if size < 16 {
                    return Err(err("fmt chunk too small"));
                }
                let format = u16::from_le_bytes([bytes[body], bytes[body + 1]]) as i64;
                let channels = u16::from_le_bytes([bytes[body + 2], bytes[body + 3]]) as i64;
                let rate = u32::from_le_bytes([
                    bytes[body + 4],
                    bytes[body + 5],
                    bytes[body + 6],
                    bytes[body + 7],
                ]) as i64;
                fmt = Some((format, channels, rate));
            }
            b"data" => data_bytes = size as i64,
            _ => {}
        }
        off = body + size + (size & 1);
    }
    let (format, channels, rate) = fmt.ok_or_else(|| err("missing fmt chunk"))?;
    if format != 1 {
        return Err(err("only PCM (format 1) is accepted"));
    }
    if !matches!(channels, 1 | 2) {
        return Err(err("mono/stereo only"));
    }
    if !matches!(rate, 22050 | 44100 | 48000) {
        return Err(err("rate must be 22050/44100/48000"));
    }
    if data_bytes < 0 {
        return Err(err("missing data chunk"));
    }
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(err("raw bank over the 8MiB limit"));
    }
    let frames = data_bytes / (channels * 2);
    if frames as f64 / rate as f64 > 10.0 {
        return Err(err("bank over the 10s limit"));
    }
    Ok((rate, channels, frames))
}

/// 读入 + 校验 bank/font/license；cue span 与 bank 帧数对界；gain 域。
fn prepare_assets(
    entry_dir: &Path,
    game: &Game2dBlock,
    cues: &[GameCueOut],
    root_module: &str,
) -> Result<Vec<GameAsset>, LowerError> {
    let err = |what: String| LowerError::Unsupported { what, line: 0 };
    let mut out = Vec::new();
    let audio = game.audio.as_ref().ok_or_else(|| {
        err0(
            "game2d requires an `audio { bank: .. font: .. license: .. }` block (Hud2V1 scene assets)"
                .to_string(),
        )
    })?;
    let read = |rel: &str, what: &str| -> Result<Vec<u8>, LowerError> {
        let p = entry_dir.join(rel);
        std::fs::read(&p)
            .map_err(|e| err(format!("game2d {what} '{rel}' unreadable: {e}")))
    };
    if !game.cues.is_empty() {
        let bytes = read(&audio.bank, "bank")?;
        let meta = parse_wav(&bytes).map_err(err)?;
        let frames = meta.2;
        for c in cues {
            if !(0 <= c.start_frame && c.start_frame < c.end_frame && c.end_frame <= frames) {
                return Err(at(
                    root_module,
                    c.line,
                    &c.name,
                    &format!(
                        "cue span ({}, {}) out of the bank frame range 0..{frames} (cue id = scene cues array index)",
                        c.start_frame, c.end_frame
                    ),
                ));
            }
            if !c.gain.is_finite() || !(0.0..=1.0).contains(&c.gain) {
                return Err(at(
                    root_module,
                    c.line,
                    &c.name,
                    &format!("cue gain {} out of domain 0.0..=1.0", c.gain),
                ));
            }
        }
        out.push(GameAsset {
            name: basename(&audio.bank),
            sha256: sha256_hex(&bytes),
            bytes: bytes.len(),
            src: entry_dir.join(&audio.bank),
            audio: Some(meta),
            in_scene: true,
        });
    }
    let font = read(&audio.font, "font")?;
    out.push(GameAsset {
        name: basename(&audio.font),
        sha256: sha256_hex(&font),
        bytes: font.len(),
        src: entry_dir.join(&audio.font),
        audio: None,
        in_scene: true,
    });
    let license = read(&audio.license, "license")?;
    if std::str::from_utf8(&license).is_err() {
        return Err(err("game2d license must be UTF-8 text".to_string()));
    }
    out.push(GameAsset {
        name: basename(&audio.license),
        sha256: sha256_hex(&license),
        bytes: license.len(),
        src: entry_dir.join(&audio.license),
        audio: None,
        in_scene: false,
    });
    Ok(out)
}

fn basename(p: &str) -> String {
    p.rsplit(['/', '\\']).next().unwrap_or(p).to_string()
}

// ------------------------------------------------------------- scene ----

pub(crate) fn json_esc(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// game profile 的 scene 文档（autographics.scene/8：Hud2V1 准入面）。
fn scene_json_game(
    entry: &str,
    rows: &[crate::lower_basic2d::SceneNode],
    has_char: bool,
    sha: &str,
    bytes: usize,
    cues: &[GameCueOut],
    slots: &[GameSlotOut],
    assets: &[GameAsset],
    license_text: &str,
) -> String {
    let objects = rows.iter().map(obj_json).collect::<Vec<_>>().join(",\n");
    let bank_sha = assets
        .iter()
        .find(|a| a.audio.is_some())
        .map(|a| a.sha256.clone())
        .unwrap_or_default();
    let font_sha = assets
        .iter()
        .find(|a| a.audio.is_none() && a.in_scene)
        .map(|a| a.sha256.clone())
        .unwrap_or_default();
    let mut assets_json = Vec::new();
    for a in assets.iter().filter(|a| a.in_scene) {
        match &a.audio {
            Some((rate, channels, frames)) => assets_json.push(format!(
                "  {{\"audio\": {{\"rate\": {rate}, \"channels\": {channels}, \"frames\": {frames}}}, \"format\": \"wav\", \"sha256\": \"{}\", \"byte_length\": {}, \"name\": \"{}\"}}",
                a.sha256, a.bytes, a.name
            )),
            None => assets_json.push(format!(
                "  {{\"format\": \"ttf\", \"sha256\": \"{}\", \"byte_length\": {}, \"name\": \"{}\"}}",
                a.sha256, a.bytes, a.name
            )),
        }
    }
    let cues_json = cues
        .iter()
        .map(|c| {
            format!(
                "  {{\"name\": \"{}\", \"asset\": \"sha256:{bank_sha}\", \"start_frame\": {}, \"end_frame\": {}, \"gain\": {}}}",
                c.name,
                c.start_frame,
                c.end_frame,
                fmt_f(c.gain)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let slots_json = slots
        .iter()
        .map(|s| {
            let visible = s.text.is_some();
            format!(
                "   {{\"slot\": {}, \"visible\": {}, \"text\": \"{}\", \"anchor\": {}, \"offset\": [{}, {}], \"size\": {}, \"rgba\": [{}, {}, {}, {}]}}",
                s.slot,
                visible,
                json_esc(&s.init),
                s.anchor,
                fmt_f(s.offset.0),
                fmt_f(s.offset.1),
                fmt_f(s.size),
                fmt_f(s.rgba.0),
                fmt_f(s.rgba.1),
                fmt_f(s.rgba.2),
                fmt_f(s.rgba.3)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "{{\n \"behavior\": {{\n  \"api\": \"autographics.hud2d/1\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": {},\n   \"three_d\": null\n  }}\n }},\n \"schema\": \"autographics.scene/8\",\n \"document_id\": \"plan023-components-2d-game-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"objects\": [\n{objects}\n ],\n \"groups\": [],\n \"assets\": [\n{}\n ],\n \"cues\": [\n{}\n ],\n \"hud\": {{\n  \"font\": \"sha256:{font_sha}\",\n  \"license\": \"{}\",\n  \"slots\": [\n{}\n  ]\n }}\n}}\n",
        if has_char { 1 } else { 0 },
        assets_json.join(",\n"),
        cues_json,
        json_esc(license_text),
        slots_json
    )
}

// ------------------------------------------------------- world fns ----

fn at_lit(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\"', "\\\"")
}

/// game 追加 fn 块（app_update 之后；F-3 冻结骨架 + F-6 派发体，与
/// T-02 手扩世界 world-game.at 逐形同构）。
pub(crate) fn game_fns_src(gen: &GameGen) -> String {
    let mut s = String::new();
    s.push_str("// ---- PLAN-023 components-2d-game generated closeout (T-02 frozen skeleton, F-3/F-6)\n");
    // game_cue：单根序数（严格递增）+ ≤2 pending（F-3 冻结骨架）。
    s.push_str("fn game_cue(c int) {\n");
    s.push_str(" r_ord = r_ord + 1\n");
    s.push_str(" if r_np == 0 {\n  r_p0 = c\n } else {\n  if r_np == 1 {\n   r_p1 = c\n  }\n }\n");
    s.push_str(" if r_np < 2 {\n  r_np = r_np + 1\n }\n");
    s.push_str("}\n");
    // game_events：poll 前奏 + 按对序派发（next 首读恒一；事件 tick 才
    // pair；goal 分支才 phase；latch tick 置 gate，held 读由根 `.gate`
    // 分派跳过——idle-turn 门）。
    s.push_str("fn game_events() {\n");
    s.push_str(" r_gate = 0\n");
    s.push_str(" if r_state != 0 {\n  return\n }\n");
    s.push_str(" let e = ag_event_next()\n");
    s.push_str(" if e == 0 {\n  return\n }\n");
    s.push_str(" let p = ag_event_pair()\n");
    for t in &gen.triggers {
        s.push_str(&format!(" if p == {} {{\n", t.pair));
        let guarded = |body: &str| -> String {
            match &t.guard {
                Some(g) => format!(
                    "  if {}_{}[{}] == 1 {{\n{}  }}\n",
                    t.prefix, g, t.def_idx, body
                ),
                None => body.to_string(),
            }
        };
        let body = match &t.kind {
            GameTriggerKind::Collect {
                cue,
                full_at,
                full_cue,
            } => guarded(&format!(
                "   {r}({j}, 0)\n   ok = ok + ag_object_despawn({p}_handle[{j}]) * 1.0\n   r_count = r_count + 1\n   r_gate = 1\n   game_cue({cue})\n   if r_count == {full_at} {{\n    game_cue({full_cue})\n   }}\n",
                r = t.recv,
                j = t.def_idx,
                p = t.prefix,
            )),
            GameTriggerKind::Fail { value, cue } => guarded(&format!(
                "  {r}({j}, 0)\n  r_state = {value}\n  r_gate = 1\n  game_cue({cue})\n",
                r = t.recv,
                j = t.def_idx,
            )),
            GameTriggerKind::Goal {
                when_count,
                phase,
                value,
                cue,
            } => format!(
                "  if r_count == {when_count} {{\n   let ph = ag_event_phase()\n   if ph == {phase} {{\n    {r}({j}, 0)\n    r_state = {value}\n    r_gate = 1\n    game_cue({cue})\n   }}\n  }}\n",
                r = t.recv,
                j = t.def_idx,
            ),
        };
        s.push_str(&body);
        s.push_str("  return\n }\n");
    }
    s.push_str("}\n");
    // hud 文字 helper（分支选择字面量，F-5）。
    for t in &gen.texts {
        s.push_str(&format!("fn hud_{}_text(n int) str {{\n", t.field));
        for (k, v) in &t.entries {
            s.push_str(&format!(
                " if n == {k} {{\n  return \"{}\"\n }}\n",
                at_lit(v)
            ));
        }
        s.push_str(&format!(" return \"{}\"\n}}\n", at_lit(&t.default)));
    }
    // game_closeout：每回调恰一次 ag_present2d（整体 1 计费），四槽完整状态。
    let mut slot_args = Vec::new();
    for slot in &gen.slots {
        match &slot.text {
            Some(field) => slot_args.push(format!("1, hud_{}_text(r_{})", field, field)),
            None => slot_args.push("0, \"\"".to_string()),
        }
    }
    s.push_str("fn game_closeout() {\n");
    s.push_str(" var o0 = 0\n var c0 = 0\n var o1 = 0\n var c1 = 0\n");
    s.push_str(" if r_np == 1 {\n  o0 = r_ord\n  c0 = r_p0\n }\n");
    s.push_str(
        " if r_np == 2 {\n  o0 = r_ord - 1\n  c0 = r_p0\n  o1 = r_ord\n  c1 = r_p1\n }\n",
    );
    s.push_str(&format!(
        " ok = ok + ag_present2d(-1, 0, 0, 0, 0, 0, 0, r_np, o0, c0, 0.5, o1, c1, 0.5, 0, 0, 0.0, 0, 0, 0.0, {}) * 1.0\n",
        slot_args.join(", ")
    ));
    s.push_str(" r_np = 0\n");
    s.push_str("}\n");
    s
}

/// app_start 触发绑定行（pair = 行序；subject 恒为 character scene id；
/// source 恒为实例 handle；shape/radius 来自 authoring）。
pub(crate) fn trigger_binds_src(gen: &GameGen, char_id: usize) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "// {} trigger binds (pair = authoring table order; subject = the single character instance)\n",
        gen.triggers.len()
    ));
    for t in &gen.triggers {
        let (flag, radius) = match t.circle {
            Some(r) => (1, fmt_f(r)),
            None => (0, "0.0".to_string()),
        };
        // 参数无空格——与 T-02 手扩世界绑定行逐字同形。
        s.push_str(&format!(
            " ok=ok+ag_trigger_bind({},{},1,{}_handle[{}],{},{})*1.0\n",
            t.pair, char_id, t.prefix, t.def_idx, flag, radius
        ));
    }
    s
}

// ------------------------------------------------------- completion ----

/// F-8.2（game 面）：每个实例叶名的声明 handler 都有接收 fn（root 成员
/// 无索引）。与 emit 站点命名一致（`<leaf>_<ev 小写>` / `<root>_<ev 小
/// 写>`）；已有条目不覆盖（frozen 命名优先）。
fn complete_receivers(project: &Project, world: &WorldIR, plan: &mut crate::lower_components::Plan) {
    let root_prefix = world.root.to_lowercase();
    let root_decl = node_of(project, &world.root);
    if let Some(on) = &root_decl.on {
        for h in &on.handlers {
            if let EventPattern::Member { name, .. } = &h.pattern {
                let fn_name = format!("{root_prefix}_{}", name.to_lowercase());
                plan.recv_fns
                    .entry(fn_name)
                    .or_insert_with(|| (String::new(), name.to_string()));
            }
        }
    }
    for inst in &world.instances {
        let def = &world.defs[&inst.def];
        let decl = node_of(project, &inst.def);
        let Some(on) = &decl.on else { continue };
        let leaf = inst_name_of(&inst.path).to_lowercase();
        for h in &on.handlers {
            if let EventPattern::Member { name, .. } = &h.pattern {
                let fn_name = format!("{leaf}_{}", name.to_lowercase());
                plan.recv_fns
                    .entry(fn_name)
                    .or_insert_with(|| (inst.def.clone(), name.to_string()));
            }
        }
    }
}

// ------------------------------------------------------------- entry ----

/// PLAN-023 components-2d-game 公共入口：game2d 校验 → 资源准备 → 共享
/// components 管线（game 注入）→ scene8/identity 材料。
pub fn lower_project_components_game(
    project: &Project,
    entry_dir: &Path,
) -> Result<LowerOutputGame, LowerError> {
    let mut world = build_world(project)?;
    let game_block = game2d_of(project, &world)?;
    // F-4：注入生成基础设施字段（gate/ord/np/p0/p1）到 root DefIR ——
    // 之后 `.gate` 翻译、getter 发射与 closeout 引用共享同一模型面。
    {
        let root_def = world
            .defs
            .get_mut(&world.root)
            .expect("root def registered");
        for name in ["gate", "ord", "np", "p0", "p1"] {
            root_def.model.push((name.to_string(), true));
        }
    }
    // F-8.2（game 面）：先补全接收方，再走共享管线。
    let mut plan = plan_dispatch(project, &world, false)?;
    complete_receivers(project, &world, &mut plan);

    let gen = prepare_game(project, &world, &game_block)?;

    // cue 表（id = 声明序）。
    let root_src = project.modules[&world.root_module].source.clone();
    let cues: Vec<GameCueOut> = game_block
        .cues
        .iter()
        .map(|c| GameCueOut {
            name: c.name.clone(),
            start_frame: c.start_frame,
            end_frame: c.end_frame,
            gain: c.gain,
            line: line_at_src(&root_src, c.span.0),
        })
        .collect();

    let assets = prepare_assets(entry_dir, &game_block, &cues, &world.root_module)?;
    let license_text = {
        let lic = game_block
            .audio
            .as_ref()
            .map(|a| a.license.clone())
            .unwrap_or_default();
        let bytes = std::fs::read(entry_dir.join(&lic))
            .map_err(|e| err0(format!("game2d license '{lic}' unreadable: {e}")))?;
        String::from_utf8(bytes)
            .map_err(|_| err0("game2d license must be UTF-8 text".to_string()))?
    };

    let scene = collect_scene(project, &world, &plan)?;
    let src = gen_user_source(project, &world, &plan, &scene, false, Some(&gen))?;
    let source_bytes = src.len();
    let source_sha256 = sha256_hex(src.as_bytes());
    let entry = world.root_module.clone();
    let scene_json = scene_json_game(
        &entry,
        &scene.rows,
        scene.char.is_some(),
        &source_sha256,
        source_bytes,
        &cues,
        &gen.slots,
        &assets,
        &license_text,
    );
    let collected = Collected {
        nodes: scene.rows,
        camera: scene.camera,
        char_name: scene.char.map(|(_, n)| n),
    };
    let roles_json = roles_json_basic(&collected);
    Ok(LowerOutputGame {
        base: LowerOutputBasic2D {
            user_source: Some(src),
            scene_json,
            source_sha256,
            source_bytes,
            roles_json,
        },
        assets,
    })
}
