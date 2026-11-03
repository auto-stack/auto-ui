//! PLAN-035 components-3d-audio-hud：普通 Node/use/局部 model-msg-on 上的
//! 三维声音/HUD/姿态组合接线（装配身份 autographics.components3d/1，T-03
//! 冻结 number 117 / profile 28 / cli `components3d`）。
//!
//! 作者面（与 025 同文的 `presentation3d` 根块 + 既有 024..034 全部三维
//! 作者构造）：
//! - `presentation3d { audio { bank/font/license } cue <name> { span, gain }
//!   hud { slot 0..3 { owner/anchor/offset/size/rgba/text_fn/text_field/
//!   visible/init } } }`——根 node 恰一块；cue id = 声明序；槽 owner 恰一
//!   根视图实例或根本身；无 trigger 行（三维触发归属 030 池/bind 与作者
//!   有界快照消费，双机制回流具名拒绝）。
//! - 声音请求：作者 handler 内 `sfx.play(<cue>, <gain>)`（typed 映射，0
//!   额外计费；恰 5 行缓冲，第 5 行只为把 n=5 送入具名 fault 51）。
//! - 事件消费（作者所有，B 路径有界快照）：`events.snapshot()`（1 计费
//!   ≤1/回调）、`events.count/run/tick()` 与 `events.pair(i)/phase(i)/
//!   source(i)/generation(i)/source_kind(i)/template(i)/source_generation(i)/
//!   geometry(i)`（快照 getter 0 计费）；`objects.despawn_batch(h1..h8)`
//!   显式批销毁（1 计费/批，全有全无）。
//! - 姿态收口：`refs.<body>.pose(clip, phase, loop, yaw)` 降层为生成基础
//!   设施写（0 额外计费），角色姿态随 bundle 收口发出；actor-free（无
//!   character3d）组合的 bundle 为 kind=-1，pose 具名拒绝。
//! - 一次收口：每回调末恰一次
//!   `ag_present3d(kind, entity, clip, phase, loop, yaw, n, 4×(o,c,g),
//!   4×(visible,text))`（bundle 整体 1 计费；四 HUD 槽完整状态）。
//!
//! 保留名（本 profile 具名拒绝）：根 model 字段 ord/np/o0..o3/c0..c3/
//! g0..g3/pose_clip/pose_phase/pose_loop/pose_yaw；作者 fn 名 sfx_play/
//! ah_pose/ah_closeout。其余根名/业务字段任意合法命名（无计分/胜败/
//! Idle 特判，不要求 root 叫 state）。

use crate::load::{node_of, Project};
use crate::lower_basic2d::fmt_f;
use crate::lower_components::{at, inst_name_of, line_at};
use crate::lower_components_game::{parse_wav, GameAsset};
use crate::lower_node3d_presentation::PresentationMeta3;
use crate::scene::sha256_hex;
use crate::translate::LowerError;
use auto_ui_dialect::node2d::{Game2dCue, Presentation2dBlock};
use std::collections::BTreeMap;
use std::path::Path;

/// translate 面环境：components3d 组合门（事件快照/批销毁/bundle 姿态）。
#[derive(Clone, Copy)]
pub struct AhudEnv3 {
    pub has_character: bool,
}

/// 一条已解析的 HUD 槽（closeout 参数文本已按 025 规则生成）。
pub struct AhudSlot {
    pub slot: i64,
    pub anchor: i64,
    pub offset: (f64, f64),
    pub size: f64,
    pub rgba: (f64, f64, f64, f64),
    /// closeout visible 实参（owner int 字段寻址；None → "0"）。
    pub visible: Option<String>,
    /// closeout text 实参（作者 helper 调用文本；None → ""）。
    pub text: Option<String>,
    pub init: String,
}

/// 本 profile 的组合配置（传入共享 3D 管线；None = 旧档字节不变）。
pub struct AhudCfg3 {
    /// cue 名 → scene cue id（声明序；sfx.play 的 typed 映射目标）。
    pub cues: BTreeMap<String, usize>,
    /// 恰四槽完整状态（未声明槽取缺省隐藏空文本）。
    pub slots: Vec<AhudSlot>,
    /// 角色组合（恰一 character3d 定义在案；bundle kind=0 带姿态）。
    pub has_character: bool,
    /// scene cues 声明（scene/8 cues 数组逐字面）。
    pub cue_rows: Vec<Game2dCue>,
    /// scene/8 资产面（wav/ttf 行 + 字体/库文本；entry 填充）。
    pub audio_rows: String,
    pub bank_sha: String,
    pub font_sha: String,
    pub license_text: String,
}

/// 降低产物：共享 3D 管线基础输出（scene/8 已换入）+ 声音资产 + 装配身份。
pub struct LowerOutputAhud3 {
    pub base: crate::lower_basic2d::LowerOutputBasic2D,
    pub api: String,
    /// 声音/HUD 资产（bank/font，in_scene；license 仅校验并嵌入文本）。
    pub audio: Vec<GameAsset>,
}

/// scene/8（三维变体）：objects/groups 沿 3D 行面，assets = GLB closure 行
/// + wav/ttf 行，cues/hud 随组合块；actors.three_d = 1（角色）或 null
/// （actor-free；driver 侧唯一合法无角色路径，T-03 负控钉死）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn scene_json_3d_ahud(
    behavior_api: &str,
    doc_prefix: &str,
    entry: &str,
    objects: &str,
    groups: &str,
    glb_rows: &str,
    audio_rows: &str,
    has_behavior: bool,
    sha: &str,
    bytes: usize,
    ahud: &AhudCfg3,
    bank_sha: &str,
    font_sha: &str,
    license_text: &str,
) -> String {
    let cues_json = ahud
        .cue_rows
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
    let slots_json = ahud
        .slots
        .iter()
        .map(|s| {
            let visible = s.visible.is_some() || s.text.is_some();
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
    let behavior = if has_behavior {
        format!(
            "{{\n  \"api\": \"{behavior_api}\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": null,\n   \"three_d\": {}\n  }}\n }}",
            if ahud.has_character { "1" } else { "null" }
        )
    } else {
        "null".to_string()
    };
    format!(
        "{{\n \"behavior\": {behavior},\n \"schema\": \"autographics.scene/8\",\n \"document_id\": \"{doc_prefix}-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"groups\": [\n{groups}\n ],\n \"objects\": [\n{objects}\n ],\n \"assets\": [\n{glb_rows}{audio_rows}\n ],\n \"cues\": [\n{cues_json}\n ],\n \"hud\": {{\n  \"font\": \"sha256:{font_sha}\",\n  \"license\": \"{}\",\n  \"slots\": [\n{slots_json}\n  ]\n }}\n}}\n",
        crate::lower_components_game::json_esc(license_text)
    )
}

/// 保留根字段（生成基础设施标量；作者占用具名拒绝）。
const RESERVED_ROOT_FIELDS: &[&str] = &[
    "ord", "np", "o0", "o1", "o2", "o3", "c0", "c1", "c2", "c3", "g0", "g1", "g2", "g3",
    "pose_clip", "pose_phase", "pose_loop", "pose_yaw",
];
/// 保留作者 fn 名（生成基础设施 fn；作者占用具名拒绝）。
const RESERVED_FNS: &[&str] = &["sfx_play", "ah_pose", "ah_closeout"];

/// PLAN-035 降低入口：bindings 载体（030 池/模板/引用/角色/相机复用）+
/// `presentation3d` 组合块。返回（产品，装配身份恒 components3d/1，声音资产）。
pub fn lower_project_components3d_audio_hud(
    project: &Project,
    entry_dir: &Path,
    meta: &PresentationMeta3,
) -> Result<LowerOutputAhud3, LowerError> {
    let w3 = crate::lower_components3d::build_world_3d_with(project, true, true, Some(meta))?;
    let world = &w3.world;
    let root_name = world.root.clone();
    let root_def = &world.defs[&root_name];
    let root_decl = node_of(project, &root_name);
    let root_module = world.root_module.clone();
    let root_src = project.modules[&root_module].source.clone();

    // —— 块归属：恰一块、仅 root（解析面宽容，档归属在此具名收紧）——
    let mut carriers: Vec<&str> = Vec::new();
    for (mname, module) in &project.modules {
        for decl in &module.nodes {
            if decl.presentation3d.is_some() {
                carriers.push(decl.name.as_str());
            }
        }
        let _ = mname;
    }
    if carriers.is_empty() {
        return Err(at(
            &root_module,
            root_def.line,
            &root_name,
            "components-3d-audio-hud requires exactly one root presentation3d block (audio/cue/hud declarations)",
        ));
    }
    if carriers.len() > 1 {
        return Err(at(
            &root_module,
            root_def.line,
            carriers[1],
            "duplicate presentation3d block (exactly one root block per project)",
        ));
    }
    if carriers[0] != root_name.as_str() {
        return Err(at(
            &world.defs[carriers[0]].module,
            world.defs[carriers[0]].line,
            carriers[0],
            "presentation3d block is root-only",
        ));
    }
    let pres = root_decl
        .presentation3d
        .as_ref()
        .expect("carrier checked above");

    // —— 角色组合判定与相机/触发一致性 ——
    let has_character = project
        .modules
        .values()
        .any(|m| m.nodes.iter().any(|d| d.character3d.is_some()));
    let camera_carrier = project.modules.values().any(|m| {
        m.nodes
            .iter()
            .any(|d| d.camera3d.is_some())
    });
    if has_character && !camera_carrier {
        return Err(at(
            &root_module,
            root_def.line,
            &root_name,
            "components-3d-audio-hud character combination requires the root camera3d block (the host wires the orbit camera for a declared 3D actor)",
        ));
    }
    if !has_character && camera_carrier {
        return Err(at(
            &root_module,
            root_def.line,
            &root_name,
            "actor-free components-3d-audio-hud must not declare camera3d (the bundle is kind=-1 and no orbit camera may be configured without a character)",
        ));
    }

    // —— 保留名（作者占用具名拒绝；023 F-4 先例）——
    if let Some(model) = &root_decl.model {
        for f in &model.fields {
            if RESERVED_ROOT_FIELDS.contains(&f.name.as_str()) {
                return Err(at(
                    &root_module,
                    root_def.line,
                    &f.name,
                    "root field collides with generated presentation3d infrastructure (rename the author field)",
                ));
            }
        }
    }
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            if RESERVED_FNS.contains(&fname.as_str()) {
                return Err(at(
                    &module.name,
                    0,
                    fname,
                    "author fn collides with generated presentation3d infrastructure (rename the author fn)",
                ));
            }
        }
    }

    // —— cue 表（id = 声明序）——
    let mut cues: BTreeMap<String, usize> = BTreeMap::new();
    for (i, c) in pres.cues.iter().enumerate() {
        if cues.insert(c.name.clone(), i).is_some() {
            return Err(at(
                &root_module,
                line_at(&root_src, c.span.0),
                &c.name,
                "duplicate presentation3d cue name (cue id = declaration order)",
            ));
        }
    }

    // —— HUD 槽（025 同文校验与 closeout 实参文本生成）——
    let slots = prepare_ahud_slots(project, world, pres)?;

    // —— 声音资产（025 同一准入面：wav 帧域/收益域/ttf/license）——
    let fake = Presentation2dBlock {
        audio: pres.audio.clone(),
        cues: pres.cues.clone(),
        hud: None,
        triggers: Vec::new(),
        span: pres.span.clone(),
    };
    if pres.audio.is_none() {
        return Err(at(
            &root_module,
            root_def.line,
            "presentation3d",
            "components-3d-audio-hud requires an `audio { bank: .. font: .. license: .. }` block (components3d scene assets)",
        ));
    }
    let audio = crate::lower_presentation::prepare_pres_assets(entry_dir, &fake, &root_module, &root_src)?;
    let license_text = {
        let lic = audio
            .iter()
            .find(|a| !a.in_scene)
            .expect("prepare_pres_assets always emits the license asset");
        std::fs::read_to_string(&lic.src).map_err(|e| {
            at(
                &root_module,
                0,
                "license",
                &format!("presentation3d license unreadable: {e}"),
            )
        })?
    };

    let mut slots = slots;
    complete_ahud_slots(&mut slots);
    let (audio_rows, bank_sha, font_sha) = audio_asset_rows(&audio);
    let ahud = AhudCfg3 {
        cues,
        slots,
        has_character,
        cue_rows: pres.cues.clone(),
        audio_rows,
        bank_sha,
        font_sha,
        license_text,
    };
    let base = crate::lower_components3d_bindings::
        lower_project_components3d_presentation_entry_ahud(project, Some(meta), Some(&ahud))?;
    let api = "autographics.components3d/1".to_string();
    Ok(LowerOutputAhud3 {
        base,
        api,
        audio,
    })
}

/// HUD 槽校验与 closeout 实参文本（025 lower_presentation 的三维同文；
/// owner = 恰一根视图组件实例或根本身，一 owner 至多一槽）。
fn prepare_ahud_slots(
    project: &Project,
    world: &crate::lower_components::WorldIR,
    pres: &auto_ui_dialect::node2d::Presentation3dBlock,
) -> Result<Vec<AhudSlot>, LowerError> {
    let root_module = world.root_module.clone();
    let root_src = project.modules[&root_module].source.clone();
    let mut slots: Vec<AhudSlot> = Vec::new();
    let Some(hud) = &pres.hud else {
        return Ok(slots);
    };
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
        let root_owned = s.owner == world.root;
        let inst = if root_owned {
            None
        } else {
            Some(
                world
                    .instances
                    .iter()
                    .find(|i| i.parent.is_none() && inst_name_of(&i.path) == s.owner)
                    .ok_or_else(|| {
                        at(
                            &root_module,
                            sline,
                            &s.owner,
                            "hud slot owner must be a root-view component instance or the root node",
                        )
                    })?,
            )
        };
        let def = match &inst {
            Some(i) => &world.defs[&i.def],
            None => &world.defs[&world.root],
        };
        let prefix = match &inst {
            Some(i) => def.prefix(),
            None => "r".to_string(),
        };
        let idx = inst.as_ref().map(|i| i.def_idx).unwrap_or(usize::MAX);
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
                if !crate::lower_presentation::text_fn_signature_ok(&src_slice, f) {
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
                let addr = if root_owned {
                    format!("{prefix}_{field}")
                } else {
                    format!("{prefix}_{field}[{idx}]")
                };
                Some(format!("{fmodule}_{f}({addr})"))
            }
            _ => None,
        };
        let visible = match &s.visible {
            Some(field) => {
                if !def.model.iter().any(|(n, is_int)| n == field && *is_int) {
                    return Err(at(
                        &def.module,
                        def.line,
                        field,
                        &format!(
                            "hud slot visible field '{field}' is not an int model field of '{}'",
                            def.name
                        ),
                    ));
                }
                let addr = if root_owned {
                    format!("{prefix}_{field}")
                } else {
                    format!("{prefix}_{field}[{idx}]")
                };
                Some(addr)
            }
            None => None,
        };
        slots.push(AhudSlot {
            slot: s.slot,
            anchor: s.anchor,
            offset: s.offset,
            size: s.size,
            rgba: s.rgba,
            visible,
            text,
            init: s.init.clone().unwrap_or_default(),
        });
    }
    Ok(slots)
}

/// 恰四槽补齐（未声明槽 = 缺省隐藏空文本）。
pub(crate) fn complete_ahud_slots(slots: &mut Vec<AhudSlot>) {
    let mut slot = 0i64;
    while slots.len() < 4 {
        let taken = slots.iter().any(|s| s.slot == slot);
        if !taken {
            slots.push(AhudSlot {
                slot,
                anchor: 0,
                offset: (0.0, 0.0),
                size: 16.0,
                rgba: (1.0, 1.0, 1.0, 1.0),
                visible: None,
                text: None,
                init: String::new(),
            });
        }
        slot += 1;
    }
    slots.sort_by_key(|s| s.slot);
}

/// 组合基础设施标量（ord/np + 5 行缓冲前 4 行；姿态变量仅角色组合）。
pub(crate) fn ahud_scalars_src(ahud: &AhudCfg3) -> String {
    let mut s =
        String::from("// presentation3d scalars (generated: ord/np + bounded request rows)\n");
    s.push_str("var r_ord=0\nvar r_np=0\n");
    for i in 0..4 {
        s.push_str(&format!("var r_o{i}=0\nvar r_c{i}=0\nvar r_g{i}=0.0\n"));
    }
    if ahud.has_character {
        s.push_str("var r_pose_clip=0\nvar r_pose_phase=0.0\nvar r_pose_loop=1\nvar r_pose_yaw=0.0\n");
    }
    s
}

/// 组合基础设施 fns：sfx 收集器、（角色）姿态写、一次 ag_present3d 收口。
pub(crate) fn ahud_fns_src(ahud: &AhudCfg3, kind: i64, entity: i64) -> String {
    let mut s = String::new();
    s.push_str("// ---- PLAN-035 presentation3d generated closeout (T-03 frozen skeleton)\n");
    s.push_str("fn sfx_play(c int, g float) {\n");
    s.push_str(" r_ord = r_ord + 1\n");
    for i in 0..4 {
        s.push_str(&format!(
            " if r_np == {i} {{\n  r_o{i} = r_ord\n  r_c{i} = c\n  r_g{i} = g\n }}\n"
        ));
    }
    s.push_str(" if r_np < 5 {\n  r_np = r_np + 1\n }\n");
    s.push_str("}\n");
    if ahud.has_character {
        s.push_str("fn ah_pose(clip int, phase float, loop int, yaw float) {\n");
        s.push_str(" r_pose_clip=clip\n r_pose_phase=phase\n r_pose_loop=loop\n r_pose_yaw=yaw\n");
        s.push_str("}\n");
    }
    let mut slot_args = Vec::new();
    for slot in &ahud.slots {
        slot_args.push(format!("{}, {}", slot_visible_arg(slot), slot_text_arg(slot)));
    }
    let pose_args = if ahud.has_character {
        "r_pose_clip, r_pose_phase, r_pose_loop, r_pose_yaw".to_string()
    } else {
        "0, 0.0, 0, 0.0".to_string()
    };
    s.push_str("fn ah_closeout() {\n");
    s.push_str(&format!(
        " ok = ok + ag_present3d({kind}, {entity}, {pose_args}, r_np, r_o0, r_c0, r_g0, r_o1, r_c1, r_g1, r_o2, r_c2, r_g2, r_o3, r_c3, r_g3, {}) * 1.0\n",
        slot_args.join(", ")
    ));
    s.push_str(" r_np = 0\n");
    s.push_str("}\n");
    s
}

/// 槽 visible 实参：作者绑定优先；未绑定时随文字面呈现（有文字即可见，
/// 无文字显式隐藏——隐藏/可见空态都显式状态）。
fn slot_visible_arg(slot: &AhudSlot) -> String {
    slot.visible
        .clone()
        .unwrap_or_else(|| if slot.text.is_some() { "1".to_string() } else { "0".to_string() })
}

/// 槽 text 实参：作者 helper 调用文本；未绑定 = 空串。
fn slot_text_arg(slot: &AhudSlot) -> String {
    slot.text.clone().unwrap_or_else(|| "\"\"".to_string())
}

/// 每回调恰一次的收口调用行（app_update 尾；bundle 整体 1 计费）。
pub(crate) fn ahud_closeout_call(ahud: &AhudCfg3, kind: i64, entity: i64) -> String {
    let mut slot_args = Vec::new();
    for slot in &ahud.slots {
        slot_args.push(format!("{}, {}", slot_visible_arg(slot), slot_text_arg(slot)));
    }
    let pose_args = if ahud.has_character {
        "r_pose_clip, r_pose_phase, r_pose_loop, r_pose_yaw".to_string()
    } else {
        "0, 0.0, 0, 0.0".to_string()
    };
    format!(
        "ok = ok + ag_present3d({kind}, {entity}, {pose_args}, r_np, r_o0, r_c0, r_g0, r_o1, r_c1, r_g1, r_o2, r_c2, r_g2, r_o3, r_c3, r_g3, {}) * 1.0",
        slot_args.join(", ")
    )
}

/// bank/font 的 scene 资产行（wav 带 rate/channels/frames；ttf 平行）。
pub(crate) fn audio_asset_rows(audio: &[GameAsset]) -> (String, String, String) {
    let mut rows = String::new();
    let mut bank_sha = String::new();
    let mut font_sha = String::new();
    for a in audio.iter().filter(|a| a.in_scene) {
        match &a.audio {
            Some((rate, channels, frames)) => {
                rows.push_str(&format!(
                    "  {{\"audio\": {{\"rate\": {rate}, \"channels\": {channels}, \"frames\": {frames}}}, \"format\": \"wav\", \"sha256\": \"{}\", \"byte_length\": {}, \"name\": \"{}\"}},\n",
                    a.sha256, a.bytes, a.name
                ));
                bank_sha = a.sha256.clone();
            }
            None => {
                rows.push_str(&format!(
                    "  {{\"format\": \"ttf\", \"sha256\": \"{}\", \"byte_length\": {}, \"name\": \"{}\"}},\n",
                    a.sha256, a.bytes, a.name
                ));
                font_sha = a.sha256.clone();
            }
        }
    }
    let rows = rows.trim_end_matches(",
").to_string();
    (rows, bank_sha, font_sha)
}

/// 声音资产字节身份自检（产品写入面复用）。
pub fn verify_audio_asset(asset: &GameAsset) -> Vec<u8> {
    let bytes = std::fs::read(&asset.src)
        .unwrap_or_else(|e| panic!("read audio asset {}: {e}", asset.src.display()));
    assert_eq!(
        sha256_hex(&bytes),
        asset.sha256,
        "audio asset sha drift: {}",
        asset.src.display()
    );
    assert_eq!(bytes.len(), asset.bytes, "audio asset byte drift");
    bytes
}
