//! PLAN-032 components-2d-runtime profile（T-04：作者入口 → 生成器）。
//!
//! 完整二维游戏组合的生成器：components2d 作者块（audio/cue/hud/trigger 沿
//! 025 逐字语义 + atlas 行 + patrol 行）驱动共享 components 管线上的生成，
//! 骨架按 T-02 冻结合同（autographics.components2d/1）：
//! - 每回调恰一次 `ag_actions_snapshot` 存入 `r_snap`（作者 `input.snapshot()`
//!   读缓存值，0 额外计费；pressed 位恰一次消费）。
//! - poll 前奏每回调恰一次 `ag_events_snapshot`，事件 i 派发
//!   `<leaf>_<event 小写>(i, phase)`（025 转发语义，guard 为目标 model 字段）。
//! - 巡逻行生成持续位移（Playing 态每回调恰一次 `ag_object_move`）。
//! - 收口每回调恰一次 `ag_present2d(kind=0, …)`：角色四帧 + flip + ≤4 声音
//!   + 四槽 HUD（bundle 整体 1 计费；n=5 具名 fault 51）。
//! - 终局 fresh Enter：`r_state != 0` 且 pressed N → `ag_round_request`。
//! 作者面 = 025 全部服务（sfx.play/render2d.despawn/refs.body.*/input.*）
//! 加 `input.snapshot()` 与 `obj.despawn(h)`（单元素批销毁）。
//! 冻结要求：根 model 恰声明 `state: int`（0=Playing/1=Won/2=Failed）；角色
//! 定义 model 恰声明 `pose: int`（0 idle/1 run/2 rise/3 fall）与
//! `facing: int`（1 右 / -1 左）。
use crate::load::{node_of, Project};
use crate::lower_basic2d::{fmt_f, Collected, LowerOutputBasic2D};
use crate::lower_components::{
    at, build_world, collect_scene, gen_user_source, inst_name_of, line_at, plan_dispatch, WorldIR,
};
use crate::lower_components_game::{complete_receivers, GameAsset};
use crate::lower_presentation::{
    prepare_pres, prepare_pres_assets, scene_json_presentation, PresGen, PresMode,
};
use crate::scene::sha256_hex;
use crate::translate::LowerError;
use auto_ui_dialect::node2d::{Components2dBlock, PresTrigger};
use std::collections::BTreeMap;
use std::path::Path;

/// 本 profile 保留的根字段名（生成基础设施；state 为作者必declared）。
const INFRA_ROOT_FIELDS: [&str; 8] = ["ord", "np", "p0", "p1", "p2", "p3", "snap", "state"];
/// 生成基础设施 fn 名（作者 fn 具名拒绝）。
const INFRA_FNS: [&str; 7] = [
    "sfx_play",
    "cmp_events",
    "cmp_patrol",
    "cmp_restart",
    "cmp_closeout",
    "cmp_frame_x",
    "cmp_flip",
];

pub struct CmpSpawnOut {
    pub name: String,
    pub slot: i64,
    pub x: f64,
    pub y: f64,
}

/// components-2d-runtime 附加材料（挂在 PresGen.cmp）。
pub struct CmpGen {
    pub spawns: Vec<CmpSpawnOut>,
    /// 巡逻行（node = 根视图实例名；恰一行或缺省）。
    pub patrol: Option<PatrolOut>,
    pub atlas: Option<AtlasOut>,
    /// 角色实例的 scene 行 id（bundle kind=0 的 entity）。
    pub char_id: usize,
    /// 角色 SoA 前缀与下标（pose/facing 读取）。
    pub char_prefix: String,
    pub char_idx: usize,
}

pub struct PatrolOut {
    pub prefix: String,
    pub idx: usize,
    pub from: f64,
    pub to: f64,
    pub at_y: f64,
    pub speed: f64,
}

pub struct AtlasOut {
    pub idle: (i64, i64),
    pub run: (i64, i64),
    pub rise: (i64, i64),
    pub fall: (i64, i64),
    pub w: i64,
    pub h: i64,
}

pub struct LowerOutputComponents2dRuntime {
    pub base: LowerOutputBasic2D,
    pub assets: Vec<GameAsset>,
}

fn err0(what: String) -> LowerError {
    LowerError::Unsupported { what, line: 0 }
}

/// components2d 块：root-only 定位 + 提取。
fn cmp2d_of(project: &Project, world: &WorldIR) -> Result<Components2dBlock, LowerError> {
    for (mname, module) in &project.modules {
        for n in &module.nodes {
            let Some(p) = &n.components2d else { continue };
            let line = line_at(&module.source, n.span.0);
            if n.name != world.root {
                return Err(at(
                    mname,
                    line,
                    &n.name,
                    "components2d block is root-node only (triggers/slots bind root-view instances)",
                ));
            }
            if mname != &world.root_module {
                return Err(at(
                    mname,
                    line,
                    &n.name,
                    "components2d block must live in the root node's module",
                ));
            }
            return Ok(p.clone());
        }
    }
    Err(at(
        &world.root_module,
        0,
        &world.root,
        "components-2d-runtime requires a `components2d` block in the root node (audio assets / cue table / hud slots / trigger rows / atlas)",
    ))
}

/// 保留名检查（根字段 + 作者 fn）。
fn reject_reserved(project: &Project, world: &WorldIR) -> Result<(), LowerError> {
    let root_decl = node_of(project, &world.root);
    let mut has_state = false;
    if let Some(model) = &root_decl.model {
        for f in &model.fields {
            if f.name == "state" {
                has_state = true;
            }
            if INFRA_ROOT_FIELDS.contains(&f.name.as_str()) && f.name != "state" {
                return Err(at(
                    &world.root_module,
                    world.defs[&world.root].line,
                    &f.name,
                    "root model field is generated components2d infrastructure (ord/np/p0..p3/snap are reserved in this profile); declare any other name",
                ));
            }
        }
    }
    if !has_state {
        return Err(at(
            &world.root_module,
            world.defs[&world.root].line,
            &world.root,
            "components-2d-runtime requires the root model to declare `state: int` (frozen 0=Playing/1=Won/2=Failed; the generated patrol gate and end-state Enter read it)",
        ));
    }
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            if INFRA_FNS.contains(&fname.as_str()) {
                return Err(at(
                    &module.name,
                    0,
                    fname,
                    "author fn name is generated components2d infrastructure (sfx_play/cmp_* are reserved in this profile); rename the fn",
                ));
            }
        }
    }
    Ok(())
}

/// 块校验 + 附加材料解析（全部执行前具名拒绝）。
fn prepare_cmp(
    project: &Project,
    world: &WorldIR,
    block: &Components2dBlock,
    gen: &mut PresGen,
    char_id: usize,
) -> Result<(), LowerError> {
    reject_reserved(project, world)?;
    let root_module = world.root_module.clone();
    let root_src = project.modules[&root_module].source.clone();

    // atlas：有角色则必声明（帧收口是这个 profile 的冻结面）。
    let atlas = if let Some(a) = &block.atlas {
        Some(AtlasOut {
            idle: a.idle,
            run: a.run,
            rise: a.rise,
            fall: a.fall,
            w: a.frame_size.0,
            h: a.frame_size.1,
        })
    } else {
        None
    };
    if world.defs.values().any(|d| d.char_elem.is_some()) && atlas.is_none() {
        return Err(at(
            &root_module,
            line_at(&root_src, block.span.0),
            &world.root,
            "components-2d-runtime with a character2d requires the `atlas` row (the kind=0 closeout carries the four pose frames)",
        ));
    }

    // 角色定义的 pose/facing 冻结字段。
    let (char_prefix, char_idx) = {
        let mut found = None;
        for inst in &world.instances {
            if world.defs[&inst.def].char_elem.is_some() {
                found = Some((world.defs[&inst.def].prefix(), inst.def_idx));
            }
        }
        found.ok_or_else(|| err0("components-2d-runtime requires a character2d instance".to_string()))?
    };
    let char_def = world
        .instances
        .iter()
        .find(|i| world.defs[&i.def].prefix() == char_prefix)
        .map(|i| &world.defs[&i.def])
        .ok_or_else(|| err0("character instance missing".to_string()))?;
    let char_decl = node_of(project, &char_def.name);
    let mut has_pose = false;
    let mut has_facing = false;
    if let Some(model) = &char_decl.model {
        for f in &model.fields {
            if f.name == "pose" {
                has_pose = true;
            }
            if f.name == "facing" {
                has_facing = true;
            }
        }
    }
    if !has_pose || !has_facing {
        return Err(at(
            &root_module,
            world.defs[&char_def.name].line,
            &char_def.name,
            "the character2d definition must declare model fields `pose: int` (0 idle/1 run/2 rise/3 fall) and `facing: int` (1 right / -1 left); the generated frame closeout reads them",
        ));
    }

    // spawn 行：本冻结面不提供（实例来自 020 视图链；具名拒绝防旁路）。
    if let Some(sp) = block.spawns.first() {
        return Err(at(
            &root_module,
            line_at(&root_src, sp.span.0),
            &sp.name,
            "components2d `spawn` rows are not part of the frozen T-04 surface (root-view component instances own the object table; declare the instance in view-2d)",
        ));
    }

    // 巡逻行：node = 根视图实例名（恰一实例的定义）。
    let patrol = if let Some(pt) = &block.patrol {
        let inst = world
            .instances
            .iter()
            .find(|i| i.parent.is_none() && inst_name_of(&i.path) == pt.node)
            .ok_or_else(|| {
                at(
                    &root_module,
                    line_at(&root_src, pt.span.0),
                    &pt.node,
                    "patrol node must be a root-view component instance",
                )
            })?;
        let count = world
            .instances
            .iter()
            .filter(|i| i.def == inst.def)
            .count();
        if count != 1 {
            return Err(at(
                &root_module,
                line_at(&root_src, pt.span.0),
                &pt.node,
                "patrol node's definition must have exactly one instance",
            ));
        }
        Some(PatrolOut {
            prefix: world.defs[&inst.def].prefix(),
            idx: inst.def_idx,
            from: pt.from,
            to: pt.to,
            at_y: pt.at_y,
            speed: pt.speed,
        })
    } else {
        None
    };

    gen.mode = PresMode::Components2dRuntime;
    gen.cmp = Some(CmpGen {
        spawns: Vec::new(),
        patrol,
        atlas,
        char_id,
        char_prefix,
        char_idx,
    });
    Ok(())
}

/// 主降层入口：components-2d-runtime。
pub fn lower_project_components2d_runtime(
    project: &Project,
    entry_dir: &Path,
) -> Result<LowerOutputComponents2dRuntime, LowerError> {
    let world = build_world(project, false, false)?;
    let block = cmp2d_of(project, &world)?;
    // audio/cue/hud/trigger 与 presentation2d 逐字同语义：合成 025 块复用
    // prepare_pres 的全部校验与解析。
    let pres_block = auto_ui_dialect::node2d::Presentation2dBlock {
        audio: block.audio.clone(),
        cues: block.cues.clone(),
        hud: block.hud.clone(),
        triggers: block.triggers.clone(),
        span: block.span,
    };
    let mut gen = prepare_pres(project, &world, &pres_block)?;
    let mut plan = crate::lower_components::plan_dispatch(project, &world, false, false, true)?;
    complete_receivers(project, &world, &mut plan);
    let scene = collect_scene(project, &world, &plan)?;
    if scene.char.is_none() {
        return Err(at(
            &world.root_module,
            world.defs[&world.root].line,
            &world.root,
            "components-2d-runtime requires exactly one character2d instance (the frozen G2 skeleton owns the physics body)",
        ));
    }
    if scene.camera.is_none() {
        return Err(at(
            &world.root_module,
            world.defs[&world.root].line,
            &world.root,
            "components-2d-runtime requires exactly one camera2d (the components2d entry freezes camera state)",
        ));
    }
    let (char_id, _) = scene.char.clone().ok_or_else(|| err0("char missing".to_string()))?;
    prepare_cmp(project, &world, &block, &mut gen, char_id)?;
    let root_src = project.modules[&world.root_module].source.clone();
    let assets = prepare_pres_assets(entry_dir, &pres_block, &world.root_module, &root_src)?;
    // atlas 的 PNG 资产（typed 副本进 scene assets）。
    let mut assets = assets;
    let mut atlas_png: Option<(String, i64, i64)> = None;
    if let (Some(a), Some(cmp)) = (&block.atlas, &gen.cmp) {
        if cmp.atlas.is_some() {
            let png_path = entry_dir.join(&a.asset);
            let bytes = std::fs::read(&png_path)
                .map_err(|e| err0(format!("components2d atlas '{}' unreadable: {e}", a.asset)))?;
            let sha = sha256_hex(&bytes);
            let sha_entry = sha.clone();
            let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
            let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
            assets.push(GameAsset {
                name: format!("{sha_entry}.png"),
                sha256: sha_entry,
                bytes: bytes.len(),
                src: png_path,
                audio: None,
                in_scene: true,
            });
            atlas_png = Some((sha, w as i64, h as i64));
        }
    }
    let license_text = {
        let lic = block
            .audio
            .as_ref()
            .map(|a| a.license.clone())
            .unwrap_or_default();
        let bytes = std::fs::read(entry_dir.join(&lic))
            .map_err(|e| err0(format!("components2d license '{lic}' unreadable: {e}")))?;
        String::from_utf8(bytes)
            .map_err(|_| err0("components2d license must be UTF-8 text".to_string()))?
    };
    let src = gen_user_source(project, &world, &plan, &scene, false, None, false, Some(&gen))?;
    let source_bytes = src.len();
    let source_sha256 = sha256_hex(src.as_bytes());
    let entry = world.root_module.clone();
    let mut scene_json = scene_json_presentation(
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
    // api 身份 + atlas sprite 绑定（components2d/1 装配）。
    scene_json = scene_json.replace(
        "\"api\": \"autographics.hud2d/1\"",
        "\"api\": \"autographics.components2d/1\"",
    );
    scene_json = scene_json.replace(
        "\"document_id\": \"plan025-presentation-2d-",
        "\"document_id\": \"plan032-components2d-runtime-",
    );
    // PNG 资产以 png 形状进 assets[]（scene_json_presentation 只写 wav/ttf
    // 形状；这里把该成员替换为带 width/height 的 png 条目）。
    if let Some((sha, w, h)) = &atlas_png {
        let blen = assets
            .iter()
            .find(|x| x.name.ends_with(".png"))
            .map(|x| x.bytes)
            .unwrap_or(0);
        let ttf_shape = format!(
            "  {{\"format\": \"ttf\", \"sha256\": \"{sha}\", \"byte_length\": {blen}, \"name\": \"{sha}.png\"}}"
        );
        let png_shape = format!(
            "  {{\"format\": \"png\", \"sha256\": \"{sha}\", \"byte_length\": {blen}, \"name\": \"{sha}.png\", \"width\": {w}, \"height\": {h}}}"
        );
        if !scene_json.contains(&ttf_shape) {
            return Err(err0("atlas png entry missing from scene assets".to_string()));
        }
        scene_json = scene_json.replace(&ttf_shape, &png_shape);
    }
    if let (Some(a), Some(cmp)) = (&block.atlas, &gen.cmp) {
        if cmp.atlas.is_some() {
            let sha = assets
                .iter()
                .find(|x| x.name.ends_with(".png"))
                .map(|x| x.sha256.clone())
                .ok_or_else(|| err0("atlas asset missing".to_string()))?;
            let bind = format!(
                "\"sprite\": {{\"asset\": \"sha256:{sha}\", \"frame\": [{}, {}, {}, {}], \"flip\": false}}",
                a.idle.0, a.idle.1, a.frame_size.0, a.frame_size.1
            );
            // 角色行（scene id = char_id）的 sprite 绑定：按行 id 定位其
            // objects 数组成员的 \"id\": <char_id> 对象，把 \"sprite\": null
            // 替换为绑定。
            let marker = format!("\"id\": {char_id},");
            let pos = scene_json
                .find(&marker)
                .ok_or_else(|| err0(format!("char row {char_id} missing in scene")))?;
            let sprite_null_pos = scene_json[pos..]
                .find("\"sprite\": null")
                .map(|p| pos + p)
                .ok_or_else(|| err0("char row has no sprite slot".to_string()))?;
            scene_json.replace_range(sprite_null_pos..sprite_null_pos + "\"sprite\": null".len(), &bind);
        }
    }
    let collected = Collected {
        nodes: scene.rows,
        camera: scene.camera,
        char_name: scene.char.map(|(_, n)| n),
    };
    let roles_json = crate::lower_basic2d::roles_json_basic(&collected);
    Ok(LowerOutputComponents2dRuntime {
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

// ------------------------------------------------------- world fns ----

/// components2d 基础设施标量（ord/np + 4 行缓冲 + snap + 巡逻 + r_state 沿
/// 作者根 model 的 SoA）。
pub fn cmp_scalars_src(pg: &PresGen) -> String {
    let mut s = String::from(
        "// components2d scalars (generated: ord/np + bounded request rows + snap + patrol)\n",
    );
    s.push_str("var r_ord=0\nvar r_np=0\n");
    s.push_str("var r_p0=0\nvar r_p1=0\nvar r_p2=0\nvar r_p3=0\n");
    s.push_str("var r_snap=0\n");
    if let Some(cmp) = &pg.cmp {
        if let Some(pt) = &cmp.patrol {
            s.push_str(&format!(
                "var r_ex={}\nvar r_edir=1\n",
                fmt_f(pt.from)
            ));
        }
    }
    s
}

/// components2d 生成骨架（poll / 巡逻 / 终局 Enter / 收口 / sfx 缓冲）。
pub fn cmp_fns_src(pg: &PresGen) -> String {
    let mut s = String::new();
    // sfx 缓冲：恰 5 行（第 5 行把真实计数 n=5 送入具名 fault 51）。
    s.push_str(
        "// bounded four-row request buffer; the fifth pending request keeps the\n// true count so the closeout bundle carries n=5 into the named fault 51\nfn sfx_play(c int, g float) {\n r_ord = (r_ord + 1)\n if (r_np == 0) {\n  r_p0 = c\n } else {\n  if (r_np == 1) {\n   r_p1 = c\n  } else {\n   if (r_np == 2) {\n    r_p2 = c\n   } else {\n    if (r_np == 3) {\n     r_p3 = c\n    }\n   }\n  }\n }\n if (r_np < 5) {\n  r_np = (r_np + 1)\n }\n}\n",
    );
    // poll：每回调恰一次快照；事件 i 派发 <leaf>_<event>(i, phase)。
    s.push_str("fn cmp_events() {\n");
    s.push_str(" let n = ag_events_snapshot()\n");
    s.push_str(" for i in 0..n {\n");
    s.push_str("  let p = ag_event_snap_pair(i)\n");
    s.push_str("  let ph = ag_event_snap_phase(i)\n");
    for t in &pg.triggers {
        s.push_str(&format!("  if p == {} {{\n", t.pair));
        match &t.guard {
            Some(g) => s.push_str(&format!(
                "   if {}_{}[{}] == 1 {{\n    {}({}, ph)\n   }}\n",
                t.prefix, g, t.def_idx, t.recv, t.def_idx
            )),
            None => s.push_str(&format!(
                "   {}({}, ph)\n",
                t.recv,
                t.def_idx
            )),
        }
        s.push_str("  }\n");
    }
    s.push_str(" }\n}\n");
    let Some(cmp) = &pg.cmp else { return s };
    // 巡逻：Playing 态每回调恰一次 move。
    if let Some(pt) = &cmp.patrol {
        s.push_str(&format!(
            "fn cmp_patrol(dt float) {{\n if (r_state == 0) {{\n  r_ex = (r_ex + ((r_edir * {}) * dt))\n  if (r_ex > {}) {{\n   r_edir = -1\n  }}\n  if (r_ex < {}) {{\n   r_edir = 1\n  }}\n  ok=ok+ag_object_move({}_handle[{}], r_ex, {}, 0.0)*1.0\n }}\n}}\n",
            fmt_f(pt.speed),
            fmt_f(pt.to),
            fmt_f(pt.from),
            pt.prefix,
            pt.idx,
            fmt_f(pt.at_y)
        ));
    }
    // 终局 fresh Enter：state != 0 且 pressed N（bit3 → snap/512）。
    s.push_str(
        "fn cmp_restart() {\n if (r_state != 0) {\n  if (((r_snap / 512) % 2) == 1) {\n   ok=ok+ag_round_request()*1.0\n  }\n }\n}\n",
    );
    // 收口：kind=0 四帧 bundle（帧 + flip + ≤4 声音 + 四槽 HUD）。
    if let Some(a) = &cmp.atlas {
        let (ix, iy) = a.idle;
        let (rx, ry) = a.run;
        let (ux, uy) = a.rise;
        let (fx, fy) = a.fall;
        s.push_str(&format!(
            "fn cmp_frame_x(p int) int {{\n if (p == 1) {{\n  return {rx}\n }}\n if (p == 2) {{\n  return {ux}\n }}\n if (p == 3) {{\n  return {fx}\n }}\n return {ix}\n}}\n"
        ));
        s.push_str(&format!(
            "fn cmp_frame_y(p int) int {{\n if (p == 1) {{\n  return {ry}\n }}\n if (p == 2) {{\n  return {uy}\n }}\n if (p == 3) {{\n  return {fy}\n }}\n return {iy}\n}}\n"
        ));
        s.push_str(&format!(
            "fn cmp_flip() int {{\n if ({}_facing[{}] < 0) {{\n  return 1\n }}\n return 0\n}}\n",
            cmp.char_prefix, cmp.char_idx
        ));
        // 收口行：ordinal/声部与 025 同构（np 行展开为四行精确值）。
        s.push_str(
            "fn cmp_closeout() {\n var o0=0\n var c0=0\n var o1=0\n var c1=0\n var o2=0\n var c2=0\n var o3=0\n var c3=0\n if (r_np > 0) {\n  o0 = (r_ord - (r_np - 1))\n  c0 = r_p0\n }\n if (r_np > 1) {\n  o1 = (r_ord - (r_np - 2))\n  c1 = r_p1\n }\n if (r_np > 2) {\n  o2 = (r_ord - (r_np - 3))\n  c2 = r_p2\n }\n if (r_np > 3) {\n  o3 = r_ord\n  c3 = r_p3\n }\n",
        );
        let slot_args = pg
            .slots
            .iter()
            .map(|slot| {
                let v = slot.visible.clone().unwrap_or_else(|| "0".to_string());
                let t = slot.text.clone().unwrap_or_else(|| "\"\"".to_string());
                format!("{v}, {t}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        s.push_str(&format!(
            " ok=ok+ag_present2d(0, {}, cmp_frame_x({}_pose[{}]), cmp_frame_y({}_pose[{}]), {}, {}, cmp_flip(), r_np, o0, c0, 0.5, o1, c1, 0.5, o2, c2, 0.5, o3, c3, 0.5, {})*1.0\n r_np = 0\n}}\n",
            cmp.char_id,
            cmp.char_prefix, cmp.char_idx,
            cmp.char_prefix, cmp.char_idx,
            a.w, a.h,
            slot_args
        ));
    }
    s
}
