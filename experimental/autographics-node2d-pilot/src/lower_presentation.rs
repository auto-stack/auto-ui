//! PLAN-025 presentation-2d profile（T-02 冻结合同 + T-03/T-04 生成器）。
//!
//! 通用二维 Node 声音/HUD 接线：`presentation2d` 作者块（typed 资源/cue
//! 表/HUD 槽 owner 绑定/generic 触发行）驱动共享 components 管线上的生成。
//! 与 components-2d-game（PLAN-023）的边界即本计划的功能面：
//! - 无业务枚举：不存在 Collect/Goal/Fail 触发类别、无根 count/state
//!   要求、无 `text <field> {..}` 业务映射表。计数/销毁/胜败/文字/cue
//!   选择/gain 全部由作者普通 model/msg/on/helper 决定。
//! - 有界请求收集：作者 handler 内 `sfx.play(<cue>, <gain>)` → 生成
//!   `sfx_play(<cue_id>, <gain>)`（纯 Auto 计算，0 额外计费）。单 Run
//!   ordinal 计数器严格递增；恰 5 行缓冲（第 5 行只为把真实计数 n=5
//!   送入 `_ag_audio_begin` 的具名 fault 51——无部分声音接受）。
//! - 一次收口：每回调末恰一次 `ag_present2d(kind=-1, n=r_np, .., 四槽
//!   完整状态)`（bundle 整体 1 计费，沿原 Hud2V1/Hud API 价格）。
//! - 触发转发：`pres_events` poll 前奏每回调恰一次 `ag_event_next` 首读；
//!   事件 tick 恰一次 `ag_event_pair` + `ag_event_phase`，phase（1 enter/
//!   2 stay/3 exit）作为 handler payload 转发给作者接收方。guard 为目标
//!   定义 model 字段（==1 才派发）。区域规则沿 F-6（目标定义须恰一动
//!   态 shape2d）；触发存在时要求恰一 character2d subject。
//! - 无角色模式：无触发时无需 character2d/跟随相机；scene actors.two_d
//!   为 null，装配为 autographics.presentation2d/1（driver 侧 actor-free
//!   组合；声明了角色的受检组合仍走 autographics.hud2d/1 原装配）。
//!
//! 保留名（本 profile 内具名拒绝，沿 023 F-4 先例）：根 model 字段
//! ord/np/o0..o3/c0..c3/g0..g3；作者 fn 名 sfx_play/pres_events/
//! pres_closeout。其余根名/业务字段任意合法命名。
use crate::load::{node_of, Project};
use crate::lower_basic2d::{fmt_f, obj_json, roles_json_basic, Collected, LowerOutputBasic2D};
use crate::lower_components::{
    at, build_world, collect_scene, gen_user_source, inst_name_of, line_at, WorldIR,
};
use crate::lower_components_game::{complete_receivers, parse_wav, GameAsset};
use crate::scene::sha256_hex;
use crate::translate::LowerError;
use auto_ui_dialect::node2d::{Game2dShape, Presentation2dBlock};
use std::collections::BTreeMap;
use std::path::Path;

// --------------------------------------------------------- resolved IR ----

/// 一条已解析的触发行（pair = 声明序；无业务类别）。
pub struct PresTriggerOut {
    pub pair: usize,
    /// 目标定义 SoA 前缀与下标。
    pub prefix: String,
    pub def_idx: usize,
    /// 接收方 fn（`<leaf>_<event 小写>`）。
    pub recv: String,
    /// guard 字段（可选；目标定义 model 字段）。
    pub guard: Option<String>,
    /// 绑定形状：Some(radius) = 圆；None = AABB（取实例几何）。
    pub circle: Option<f64>,
}

/// 一个 HUD 槽的 scene 元数据 + closeout 绑定（生成读取文本）。
pub struct PresSlotOut {
    pub slot: i64,
    pub anchor: i64,
    pub offset: (f64, f64),
    pub size: f64,
    pub rgba: (f64, f64, f64, f64),
    /// closeout 可见性读取文本（None → 恒 0/未声明槽）。
    pub visible: Option<String>,
    /// closeout 文字表达式（None → ""/未声明槽）。
    pub text: Option<String>,
    /// scene 初始文字。
    pub init: String,
}

/// presentation profile 的下降期配置（gen_user_source 的增量注入面）。
pub struct PresGen {
    /// cue 名 → scene cue id（`sfx.play` 降层映射；= 生成侧 cues 表）。
    pub cues: BTreeMap<String, usize>,
    pub triggers: Vec<PresTriggerOut>,
    /// 恰四槽（未声明槽补默认：隐藏/空文字）。
    pub slots: Vec<PresSlotOut>,
}

/// 本 profile 的 scene 文档 + identity 材料。
pub struct LowerOutputPresentation {
    pub base: LowerOutputBasic2D,
    pub assets: Vec<GameAsset>,
}

fn err0(what: String) -> LowerError {
    LowerError::Unsupported { what, line: 0 }
}

/// 本 profile 保留的根字段名（生成基础设施标量）。
const INFRA_ROOT_FIELDS: [&str; 13] = [
    "ord", "np", "o0", "o1", "o2", "o3", "c0", "c1", "c2", "c3", "g0", "g1", "g2",
];
/// 生成基础设施 fn 名（作者 fn 具名拒绝）。
const INFRA_FNS: [&str; 3] = ["sfx_play", "pres_events", "pres_closeout"];

// ------------------------------------------------------------ validate ----

/// presentation2d 块：root-only 定位 + 提取。
fn pres2d_of(project: &Project, world: &WorldIR) -> Result<Presentation2dBlock, LowerError> {
    for (mname, module) in &project.modules {
        for n in &module.nodes {
            let Some(p) = &n.presentation2d else { continue };
            let line = line_at(&module.source, n.span.0);
            if n.name != world.root {
                return Err(at(
                    mname,
                    line,
                    &n.name,
                    "presentation2d block is root-node only (slots bind root-view instances)",
                ));
            }
            if mname != &world.root_module {
                return Err(at(
                    mname,
                    line,
                    &n.name,
                    "presentation2d block must live in the root node's module",
                ));
            }
            return Ok(p.clone());
        }
    }
    Err(at(
        &world.root_module,
        0,
        &world.root,
        "presentation-2d requires a `presentation2d` block in the root node (audio assets / cue table / hud slots / trigger rows)",
    ))
}

/// 保留名检查（根字段 + 作者 fn）。
fn reject_reserved(project: &Project, world: &WorldIR) -> Result<(), LowerError> {
    let root_decl = node_of(project, &world.root);
    if let Some(model) = &root_decl.model {
        for f in &model.fields {
            if INFRA_ROOT_FIELDS.contains(&f.name.as_str()) {
                return Err(at(
                    &world.root_module,
                    world.defs[&world.root].line,
                    &f.name,
                    "root model field is generated presentation infrastructure (ord/np/o0..o3/c0..c3/g0..g2 are reserved in this profile); declare any other name",
                ));
            }
        }
    }
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            if INFRA_FNS.contains(&fname.as_str()) {
                return Err(at(
                    &module.name,
                    0,
                    fname,
                    "author fn name is generated presentation infrastructure (sfx_play/pres_events/pres_closeout are reserved in this profile); rename the fn",
                ));
            }
        }
    }
    Ok(())
}

/// 块校验 + 触发/槽解析（全部执行前具名拒绝）。
fn prepare_pres(
    project: &Project,
    world: &WorldIR,
    pres: &Presentation2dBlock,
) -> Result<PresGen, LowerError> {
    reject_reserved(project, world)?;
    let root_module = world.root_module.clone();
    let root_src = project.modules[&root_module].source.clone();
    let root_line = line_at(&root_src, pres.span.0);

    // ---- cue 表（id = 声明序；sfx.play 的 typed 映射目标）----
    let mut cues: BTreeMap<String, usize> = BTreeMap::new();
    for (i, c) in pres.cues.iter().enumerate() {
        if cues.insert(c.name.clone(), i).is_some() {
            return Err(at(
                &root_module,
                line_at(&root_src, c.span.0),
                &c.name,
                "duplicate presentation2d cue name (cue id = declaration order)",
            ));
        }
    }

    // ---- HUD 槽（owner 绑定 + closeout 读取文本）----
    let mut slots: Vec<PresSlotOut> = Vec::new();
    let mut owners: Vec<String> = Vec::new();
    if let Some(hud) = &pres.hud {
        let mut seen: Vec<i64> = Vec::new();
        for s in &hud.slots {
            let sline = line_at(&root_src, s.span.0);
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
                return Err(at(&root_module, sline, &sym, "duplicate hud slot index"));
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
            // owner：恰一根视图组件实例；一实例至多一槽（单一明确 owner）。
            let inst = world
                .instances
                .iter()
                .find(|i| i.parent.is_none() && inst_name_of(&i.path) == s.owner)
                .ok_or_else(|| {
                    at(
                        &root_module,
                        sline,
                        &s.owner,
                        "hud slot owner must be a root-view component instance",
                    )
                })?;
            if owners.contains(&s.owner) {
                return Err(at(
                    &root_module,
                    sline,
                    &s.owner,
                    "duplicate hud slot owner (each instance owns at most one slot)",
                ));
            }
            owners.push(s.owner.clone());
            let def = &world.defs[&inst.def];
            let prefix = def.prefix();
            let idx = inst.def_idx;
            // 文字绑定：作者 helper（<module>_<fn>，(int) -> str）+ owner 的
            // int model 字段。
            let text = match (&s.text_fn, &s.text_field) {
                (Some(f), Some(field)) => {
                    let (fmodule, fsrc) = project
                        .modules
                        .iter()
                        .find_map(|(m, module)| {
                            module.fns.iter().find(|(n, _)| n == f).map(|_| {
                                (m.as_str(), crate::lower::slice_fn_source(&module.source, f))
                            })
                        })
                        .ok_or_else(|| {
                            at(
                                &root_module,
                                sline,
                                f,
                                &format!("hud slot text_fn '{f}' is not a declared fn in any module"),
                            )
                        })?;
                    let Some(src_slice) = fsrc else {
                        return Err(at(
                            &root_module,
                            sline,
                            f,
                            &format!("hud slot text_fn '{f}' has no sliceable source"),
                        ));
                    };
                    if !text_fn_signature_ok(&src_slice, f) {
                        return Err(at(
                            &root_module,
                            sline,
                            f,
                            &format!(
                                "hud slot text_fn '{f}' must have signature (int) -> str (one int parameter, str return)"
                            ),
                        ));
                    }
                    if !def.model.iter().any(|(n, is_int)| n == field && *is_int) {
                        return Err(at(
                            &def.module,
                            def.line,
                            field,
                            &format!(
                                "hud slot text_field '{field}' is not an int model field of '{}'",
                                def.name
                            ),
                        ));
                    }
                    Some(format!("{fmodule}_{f}({prefix}_{field}[{idx}])"))
                }
                _ => None,
            };
            // 可见性绑定：owner 的 int model 字段（缺省恒可见 1）。
            let visible = match &s.visible {
                Some(field) => {
                    if !def.model.iter().any(|(n, is_int)| n == field && *is_int) {
                        return Err(at(
                            &def.module,
                            def.line,
                            field,
                            &format!(
                                "hud slot visible '{field}' is not an int model field of '{}'",
                                def.name
                            ),
                        ));
                    }
                    Some(format!("{prefix}_{field}[{idx}]"))
                }
                None if text.is_some() => Some("1".to_string()),
                None => None,
            };
            let init = s.init.clone().unwrap_or_default();
            if init.len() > 32 || !init.bytes().all(|b| (32..=126).contains(&b)) {
                return Err(at(
                    &root_module,
                    sline,
                    &sym,
                    "hud slot init text must be <=32 visible ASCII chars (scene-layer contract; runtime live text over 32 chars is named fault 56)",
                ));
            }
            slots.push(PresSlotOut {
                slot: s.slot,
                anchor: s.anchor,
                offset: s.offset,
                size: s.size,
                rgba: s.rgba,
                visible,
                text,
                init,
            });
        }
    }
    // 恰四槽（scene 合同）：未声明槽补默认（隐藏/空文字）。
    for i in 0..4i64 {
        if !slots.iter().any(|s| s.slot == i) {
            slots.push(PresSlotOut {
                slot: i,
                anchor: i,
                offset: (8.0, 8.0),
                size: 32.0,
                rgba: (1.0, 1.0, 1.0, 1.0),
                visible: None,
                text: None,
                init: String::new(),
            });
        }
    }
    slots.sort_by_key(|s| s.slot);

    // ---- 触发行（pair = 声明序；无业务类别）----
    if pres.triggers.len() > 8 {
        return Err(at(
            &root_module,
            root_line,
            "trigger",
            &format!(
                "{} trigger rows exceed the frozen 8-pair trigger registry",
                pres.triggers.len()
            ),
        ));
    }
    let mut triggers = Vec::new();
    for (pair, t) in pres.triggers.iter().enumerate() {
        let line = line_at(&root_src, t.span.0);
        // 目标必须是 root-view 运行时实例（区域规则沿 F-6）。
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
        if let Some(g) = &t.guard {
            if !def.model.iter().any(|(n, _)| n == g) {
                return Err(at(
                    &def.module,
                    def.line,
                    g,
                    &format!("trigger guard '{g}' is not a model field of '{}'", def.name),
                ));
            }
        }
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
        triggers.push(PresTriggerOut {
            pair,
            prefix: def.prefix(),
            def_idx: inst.def_idx,
            recv: format!(
                "{target_lower}_{ev_lower}",
                target_lower = t.target.to_lowercase()
            ),
            guard: t.guard.clone(),
            circle: match t.shape {
                Game2dShape::Circle(r) => Some(r),
                Game2dShape::Box => None,
            },
        });
    }

    Ok(PresGen {
        cues,
        triggers,
        slots,
    })
}

/// 作者 helper 头签名检查：`fn <name>(<one param: int>) str` 形（Auto
/// 返回类型为空格后缀，无箭头）。load 层 fn 表只有名字；对切片头部做
/// 保守文本检查。
fn text_fn_signature_ok(src: &str, name: &str) -> bool {
    let Some(rest) = src.trim_start().strip_prefix("fn ") else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(name) else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix('(') else {
        return false;
    };
    // 恰一参数 `n int`，返回类型 `str`（紧随右括号，无箭头）。
    let Some(close) = rest.find(')') else {
        return false;
    };
    let params = rest[..close].trim();
    if params.is_empty() || !params.ends_with(" int") || params.contains(',') {
        return false;
    }
    rest[close + 1..]
        .trim_start()
        .starts_with("str")
}

// ------------------------------------------------------------- assets ----

/// 读入 + 校验 bank/font/license（沿 023 原媒体规则；cue span/gain 对界）。
fn prepare_pres_assets(
    entry_dir: &Path,
    pres: &Presentation2dBlock,
    root_module: &str,
    root_src: &str,
) -> Result<Vec<GameAsset>, LowerError> {
    let err = |what: String| LowerError::Unsupported { what, line: 0 };
    let mut out = Vec::new();
    let audio = pres.audio.as_ref().ok_or_else(|| {
        err0(
            "presentation2d requires an `audio { bank: .. font: .. license: .. }` block (Hud2V1 scene assets)"
                .to_string(),
        )
    })?;
    let read = |rel: &str, what: &str| -> Result<Vec<u8>, LowerError> {
        let p = entry_dir.join(rel);
        std::fs::read(&p).map_err(|e| err(format!("presentation2d {what} '{rel}' unreadable: {e}")))
    };
    if !pres.cues.is_empty() {
        let bytes = read(&audio.bank, "bank")?;
        let meta = parse_wav(&bytes).map_err(err)?;
        let frames = meta.2;
        for c in &pres.cues {
            if !(0 <= c.start_frame && c.start_frame < c.end_frame && c.end_frame <= frames) {
                return Err(at(
                    root_module,
                    line_at(root_src, c.span.0),
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
                    0,
                    &c.name,
                    &format!("cue gain {} out of domain 0.0..=1.0", c.gain),
                ));
            }
        }
        out.push(GameAsset {
            name: audio.bank.rsplit(['/', '\\']).next().unwrap_or(&audio.bank).to_string(),
            sha256: sha256_hex(&bytes),
            bytes: bytes.len(),
            src: entry_dir.join(&audio.bank),
            audio: Some(meta),
            in_scene: true,
        });
    }
    let font = read(&audio.font, "font")?;
    out.push(GameAsset {
        name: audio.font.rsplit(['/', '\\']).next().unwrap_or(&audio.font).to_string(),
        sha256: sha256_hex(&font),
        bytes: font.len(),
        src: entry_dir.join(&audio.font),
        audio: None,
        in_scene: true,
    });
    let license = read(&audio.license, "license")?;
    if std::str::from_utf8(&license).is_err() {
        return Err(err("presentation2d license must be UTF-8 text".to_string()));
    }
    out.push(GameAsset {
        name: audio.license.rsplit(['/', '\\']).next().unwrap_or(&audio.license).to_string(),
        sha256: sha256_hex(&license),
        bytes: license.len(),
        src: entry_dir.join(&audio.license),
        audio: None,
        in_scene: false,
    });
    Ok(out)
}

// ------------------------------------------------------------- scene ----

/// presentation profile 的 scene 文档（autographics.scene/8）。声明了角色
/// 的受检组合走 autographics.hud2d/1 原装配（相机/角色准入不变）；无角色
/// 模式走 autographics.presentation2d/1（actor-free 组合，two_d=null）。
#[allow(clippy::too_many_arguments)]
fn scene_json_presentation(
    entry: &str,
    rows: &[crate::lower_basic2d::SceneNode],
    has_char: bool,
    sha: &str,
    bytes: usize,
    cues: &[auto_ui_dialect::node2d::Game2dCue],
    slots: &[PresSlotOut],
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
            let visible = s.visible.is_some();
            format!(
                "   {{\"slot\": {}, \"visible\": {}, \"text\": \"{}\", \"anchor\": {}, \"offset\": [{}, {}], \"size\": {}, \"rgba\": [{}, {}, {}, {}]}}",
                s.slot,
                visible,
                crate::lower_components_game::json_esc(&s.init),
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
        "{{\n \"behavior\": {{\n  \"api\": \"autographics.presentation2d/1\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": {},\n   \"three_d\": null\n  }}\n }},\n \"schema\": \"autographics.scene/8\",\n \"document_id\": \"plan025-presentation-2d-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"objects\": [\n{objects}\n ],\n \"groups\": [],\n \"assets\": [\n{}\n ],\n \"cues\": [\n{}\n ],\n \"hud\": {{\n  \"font\": \"sha256:{font_sha}\",\n  \"license\": \"{}\",\n  \"slots\": [\n{}\n  ]\n }}\n}}\n",
        if has_char { "1" } else { "null" },
        assets_json.join(",\n"),
        cues_json,
        crate::lower_components_game::json_esc(license_text),
        slots_json
    )
}

// ------------------------------------------------------- world fns ----

/// presentation 生成基础设施标量（ord/np + 5 行缓冲的前 4 行）。
pub(crate) fn pres_scalars_src() -> String {
    let mut s = String::from("// presentation scalars (generated: ord/np + bounded request rows)\n");
    s.push_str("var r_ord=0\nvar r_np=0\n");
    for i in 0..4 {
        s.push_str(&format!("var r_o{i}=0\nvar r_c{i}=0\nvar r_g{i}=0.0\n"));
    }
    s
}

/// presentation 生成 fns（收集器 + poll 前奏 + 一次收口）。
pub(crate) fn pres_fns_src(gen: &PresGen) -> String {
    let mut s = String::new();
    s.push_str(
        "// ---- PLAN-025 presentation generated closeout (T-02 frozen skeleton)\n",
    );
    // sfx_play：有界请求收集（单 Run ordinal 严格递增；恰 5 行计数——第 5
    // 行只为把 n=5 送入 _ag_audio_begin 的具名 fault 51，无部分声音接受）。
    s.push_str("fn sfx_play(c int, g float) {\n");
    s.push_str(" r_ord = r_ord + 1\n");
    for i in 0..4 {
        s.push_str(&format!(
            " if r_np == {i} {{\n  r_o{i} = r_ord\n  r_c{i} = c\n  r_g{i} = g\n }}\n"
        ));
    }
    s.push_str(" if r_np < 5 {\n  r_np = r_np + 1\n }\n");
    s.push_str("}\n");
    // pres_events：poll 前奏（每回调恰一次 next 首读；事件 tick 恰一次
    // pair + phase；guard 命中才派发；phase 作为 handler payload）。
    if !gen.triggers.is_empty() {
        s.push_str("fn pres_events() {\n");
        s.push_str(" let e = ag_event_next()\n");
        s.push_str(" if e == 0 {\n  return\n }\n");
        s.push_str(" let p = ag_event_pair()\n");
        s.push_str(" let ph = ag_event_phase()\n");
        for t in &gen.triggers {
            s.push_str(&format!(" if p == {} {{\n", t.pair));
            match &t.guard {
                Some(g) => s.push_str(&format!(
                    "  if {}_{}[{}] == 1 {{\n   {}({}, ph)\n  }}\n",
                    t.prefix, g, t.def_idx, t.recv, t.def_idx
                )),
                None => s.push_str(&format!("  {}({}, ph)\n", t.recv, t.def_idx)),
            }
            s.push_str("  return\n }\n");
        }
        s.push_str("}\n");
    }
    // pres_closeout：每回调恰一次 ag_present2d（bundle 整体 1 计费），
    // 四槽完整状态；n = r_np（第 5 请求 → fault 51，无部分接受）。
    let mut slot_args = Vec::new();
    for slot in &gen.slots {
        let v = slot.visible.clone().unwrap_or_else(|| "0".to_string());
        let t = slot.text.clone().unwrap_or_else(|| "\"\"".to_string());
        slot_args.push(format!("{v}, {t}"));
    }
    s.push_str("fn pres_closeout() {\n");
    s.push_str(&format!(
        " ok = ok + ag_present2d(-1, 0, 0, 0, 0, 0, 0, r_np, r_o0, r_c0, r_g0, r_o1, r_c1, r_g1, r_o2, r_c2, r_g2, r_o3, r_c3, r_g3, {}) * 1.0\n",
        slot_args.join(", ")
    ));
    s.push_str(" r_np = 0\n");
    s.push_str("}\n");
    s
}

/// app_start 触发绑定行（pair = 声明序；subject = character scene id）。
pub(crate) fn pres_binds_src(gen: &PresGen, char_id: usize) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "// {} presentation trigger binds (pair = authoring order; subject = the single character instance)\n",
        gen.triggers.len()
    ));
    for t in &gen.triggers {
        let (flag, radius) = match t.circle {
            Some(r) => (1, fmt_f(r)),
            None => (0, "0.0".to_string()),
        };
        s.push_str(&format!(
            " ok=ok+ag_trigger_bind({},{},1,{}_handle[{}],{},{})*1.0\n",
            t.pair, char_id, t.prefix, t.def_idx, flag, radius
        ));
    }
    s
}

// ------------------------------------------------------------- entry ----

/// PLAN-025 presentation-2d 公共入口：块校验 → 资源准备 → 共享 components
/// 管线（presentation 注入）→ scene8/identity 材料。
pub fn lower_project_presentation(
    project: &Project,
    entry_dir: &Path,
) -> Result<LowerOutputPresentation, LowerError> {
    let world = build_world(project, false, false)?;
    let block = pres2d_of(project, &world)?;
    let mut plan = crate::lower_components::plan_dispatch(project, &world, false, false)?;
    complete_receivers(project, &world, &mut plan);
    let gen = prepare_pres(project, &world, &block)?;
    let root_src = project.modules[&world.root_module].source.clone();
    let assets = prepare_pres_assets(entry_dir, &block, &world.root_module, &root_src)?;
    let license_text = {
        let lic = block
            .audio
            .as_ref()
            .map(|a| a.license.clone())
            .unwrap_or_default();
        let bytes = std::fs::read(entry_dir.join(&lic))
            .map_err(|e| err0(format!("presentation2d license '{lic}' unreadable: {e}")))?;
        String::from_utf8(bytes)
            .map_err(|_| err0("presentation2d license must be UTF-8 text".to_string()))?
    };
    let scene = collect_scene(project, &world, &plan)?;
    let src = gen_user_source(project, &world, &plan, &scene, false, None, false, Some(&gen))?;
    let source_bytes = src.len();
    let source_sha256 = sha256_hex(src.as_bytes());
    let entry = world.root_module.clone();
    let scene_json = scene_json_presentation(
        &entry,
        &scene.rows,
        scene.char.is_some(),
        &source_sha256,
        source_bytes,
        &block.cues,
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
    Ok(LowerOutputPresentation {
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
