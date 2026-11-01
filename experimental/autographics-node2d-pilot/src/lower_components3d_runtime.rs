//! PLAN-027 components-3d-runtime：3D 角色/运行相机/内置视觉的声明接线。
//!
//! 复用 024 的共享三维降低管线（world/plan/scene/gen，见
//! lower_components3d::Cfg3），只新增声明面与注入位：
//! - `character3d { }`：本定义绑定为本 Run 恰一物理角色（bind 调用注入
//!   app_start；形状 = 唯一 cylinder3d 叶的冻结 0.8×1.0×0.8 box）。
//! - `solid: true`（cylinder3d 叶）：角色 solid（全静态 AABB；solid 调用
//!   按声明先序注入 app_start）。
//! - `camera3d { distance/pitch/projection/ortho_size }`（仅 root）：运行
//!   相机一次性配置（target 恒为本 Run 恰一角色；值域沿 runtime_camera3d/1
//!   冻结域具名预检）。
//! - `player3d`/`mob3d` 叶：具名内置视觉（原 Kind::Player/Mob 映射；mesh/
//!   材质沿 driver 原 owner 暖上传，声明不携带资源字节）。
//!
//! 控制公式（重力/跳跃/方向组合/归一）留在作者 Auto（020 生命周期面）；
//! 编译器只做声明/引用/服务映射（六动作输入表、refs 角色服务、camera3d
//! 读取沿受信 character3d/1 与 runtime_camera3d/1 单元）。相机 target/
//! 重复注册/预算等运行时拒绝仍由受信单元在真实 Run 内执行（V-S03）。

use auto_parser_api::ast::Expr;
use auto_ui_dialect::node2d::{Camera3dBlock, NodeDecl, PropValue};

use crate::load::{node_of, Project};
use crate::lower::numeric_lit;
use crate::lower_basic2d::LowerOutputBasic2D;
use crate::lower_components::{at, CVal, DefIR};
use crate::lower_components3d::{
    build_world_3d, collect_scene_3d, lower_project_components3d_cfg, Cfg3, RuntimeCfg3,
    SceneBuild3, World3d,
};
use crate::translate::LowerError;
/// character3d/1 冻结形状（受信 `_ag_char3_shape`：box 0.8×1.0×0.8，
/// 即 scale 0.4/0.5/0.4，容差 1e-4）。
pub(crate) const CHARACTER_SCALE: [f64; 3] = [0.4, 0.5, 0.4];

/// 定义级诊断捷径（at() 的 module/line/symbol 三元组）。
fn err(def: &DefIR, what: &str) -> LowerError {
    at(&def.module, def.line, &def.name, what)
}

/// solid 叶校验与收集（cylinder3d 限定、全静态；id = 文档 id）。
fn push_solid(
    def: &DefIR,
    e: &crate::lower_components3d::ElemIR3,
    scene: &SceneBuild3,
    row: &str,
    cfg: &mut RuntimeCfg3,
) -> Result<(), LowerError> {
    if e.kind != "cylinder" {
        return Err(err(
            def,
            &format!(
                "solid leaf '{}' must be a cylinder3d (v1 solid surface)",
                e.elem
            ),
        ));
    }
    if e.dyn_pos {
        return Err(err(
            def,
            &format!(
                "solid leaf '{}' must be static (the solid surface is frozen at Life.Start)",
                e.elem
            ),
        ));
    }
    let Some(id) = row_document_id(scene, row) else {
        return Err(err(def, "solid row not in scene table"));
    };
    cfg.solid_ids.push(id);
    Ok(())
}

/// components-3d-runtime 降低入口：声明校验 → 注入配置 → 共享管线。
pub fn lower_project_components3d_runtime(
    project: &Project,
) -> Result<LowerOutputBasic2D, LowerError> {
    let w3 = build_world_3d(project)?;
    let scene = collect_scene_3d(project, &w3)?;
    let runtime = collect_runtime_cfg(project, &w3, &scene)?;
    // 装配身份：声明 camera3d -> runtime_camera3d/1（主机织入相机候选）；
    // 声明 character3d -> character3d/1（entry 织入角色 freeze/tick 序）；
    // 仅有为行为的其它场景 -> behavior/1（角色/相机单元不进入装配，避免
    // 无 bind 的 freeze fault 25）。驱动 start_selected_play 按该串选择
    // 受信装配。
    let behavior_api = if runtime.camera.is_some() {
        "autographics.runtime_camera3d/1"
    } else if runtime.character_def.is_some() {
        "autographics.character3d/1"
    } else {
        "autographics.behavior/1"
    };
    lower_project_components3d_cfg(
        project,
        Cfg3 {
            runtime: Some(&runtime),
            doc_prefix: "plan027-components-3d-runtime",
            gen_note: "PLAN-027 components-3d-runtime",
            regen_note: "tools/verify_plan027.py",
            behavior_api,
        },
    )
}

/// 文档 id（1-based）= scene.rows 声明序位置 + 1；行名 → id。
fn row_document_id(scene: &SceneBuild3, row: &str) -> Option<i64> {
    scene
        .rows
        .iter()
        .position(|r| r.name == row)
        .map(|i| i as i64 + 1)
}

/// 实例元素的行名：elem_rows[def][elem][def_idx]。
fn instance_elem_row(scene: &SceneBuild3, def: &str, elem: &str, def_idx: usize) -> Option<String> {
    scene
        .elem_rows
        .get(def)
        .and_then(|m| m.get(elem))
        .and_then(|rows| rows.get(def_idx))
        .cloned()
}

/// 定义在 w3.views 中的叶元素引用表。
fn view_elems<'a>(w3: &'a World3d, def: &str) -> &'a [crate::lower_components3d::ElemIR3] {
    &w3.views
        .get(def)
        .unwrap_or_else(|| panic!("def view missing: {def}"))
        .elems
}

/// 恰一角色声明收集（(def name, def_idx)；root 为 None 哨兵时另行处理）。
fn collect_runtime_cfg(
    project: &Project,
    w3: &World3d,
    scene: &SceneBuild3,
) -> Result<RuntimeCfg3, LowerError> {
    let world = &w3.world;
    let mut cfg = RuntimeCfg3::default();
    let mut character: Option<(String, usize)> = None; // (def, def_idx)

    // 恰一角色 + solid 收集（实例声明序）。
    for inst in &world.instances {
        let decl = node_of(project, &inst.def);
        let def = &world.defs[&inst.def];
        if decl.character3d.is_some() {
            if character.is_some() {
                return Err(err(
                    def,
                    &format!(
                        "character3d must bind exactly one instance per Run (second candidate instance '{}')",
                        inst.path
                    ),
                ));
            }
            character = Some((inst.def.clone(), inst.def_idx));
        }
        for e in view_elems(w3, &inst.def) {
            if !e.solid {
                continue;
            }
            let row = instance_elem_row(scene, &inst.def, &e.elem, inst.def_idx)
                .ok_or_else(|| err(def, "solid row missing in scene table"))?;
            push_solid(def, e, scene, &row, &mut cfg)?;
        }
    }
    // root 自身视图的 solid 叶（root 行 = root_rows[elem]；root 可无视图）。
    // root 元素在文档表中先于实例，solid id 序保持升序。
    if let Some(root_view) = w3.views.get(&world.root) {
        let root_def = &world.defs[&world.root];
        for e in &root_view.elems {
            if !e.solid {
                continue;
            }
            let row = scene
                .root_rows
                .get(e.elem.as_str())
                .cloned()
                .ok_or_else(|| err(root_def, "solid row missing in scene table"))?;
            push_solid(root_def, e, scene, &row, &mut cfg)?;
        }
    }

    // camera3d 仅 root；其余定义携带即具名拒绝。
    let root_decl = node_of(project, &world.root);
    for dname in world.defs.keys() {
        if dname == &world.root {
            continue;
        }
        if node_of(project, dname).camera3d.is_some() {
            return Err(err(
                &world.defs[dname],
                "camera3d block is root-only (the Run camera is one Run-owned object)",
            ));
        }
    }

    // root 自身作为角色（v1 支持面；行 = root_rows[body]）。
    let mut root_character_body: Option<String> = None;
    if root_decl.character3d.is_some() {
        if character.is_some() {
            return Err(err(
                &world.defs[&world.root],
                "character3d must bind exactly one instance per Run (root and an instance both declare it)",
            ));
        }
        root_character_body = Some(character_body_elem(&world.defs[&world.root], w3)?);
    }

    // 角色声明形状预检（受信 _ag_char3_shape 的早具名版本）。
    let character_body: Option<(String, String)> = match (&character, &root_character_body) {
        (Some((cdef, cidx)), _) => {
            let def = &world.defs[cdef];
            validate_character_def(def, w3)?;
            let body = character_body_elem(def, w3)?;
            let row = instance_elem_row(scene, cdef, &body, *cidx)
                .ok_or_else(|| err(def, "character row missing in scene table"))?;
            Some((cdef.clone(), row))
        }
        (None, Some(body)) => {
            let def = &world.defs[&world.root];
            validate_character_def(def, w3)?;
            let row = scene
                .root_rows
                .get(body.as_str())
                .cloned()
                .ok_or_else(|| err(def, "character row missing in scene table"))?;
            Some((world.root.clone(), row))
        }
        _ => None,
    };

    // 相机配置（root-only；target 恒为恰一角色）。
    if let Some(cam) = &root_decl.camera3d {
        let Some((_, row)) = &character_body else {
            return Err(err(
                &world.defs[&world.root],
                "camera3d requires exactly one character3d instance (target = the Run character)",
            ));
        };
        let target = row_document_id(scene, row)
            .ok_or_else(|| err(&world.defs[&world.root], "character row not in scene table"))?;
        cfg.camera = Some(parse_camera_cfg(&world.defs[&world.root], cam, target)?);
    }

    if let Some((cdef, row)) = &character_body {
        cfg.character_def = Some(cdef.clone());
        cfg.character_bind_id = Some(
            row_document_id(scene, row)
                .ok_or_else(|| err(&world.defs[cdef], "character row not in scene table"))?,
        );
    }

    Ok(cfg)
}

/// 角色体元素名（恰一 view-3d 叶）。
fn character_body_elem(def: &DefIR, w3: &World3d) -> Result<String, LowerError> {
    let leaves = view_elems(w3, &def.name);
    if leaves.len() != 1 {
        return Err(err(
            def,
            &format!(
                "character3d definition must have exactly one view-3d leaf element (got {})",
                leaves.len()
            ),
        ));
    }
    Ok(leaves[0].elem.clone())
}

/// 角色声明形状预检：cylinder3d 叶、静态 position、冻结 scale、非 solid。
fn validate_character_def(def: &DefIR, w3: &World3d) -> Result<(), LowerError> {
    let body = character_body_elem(def, w3)?;
    let leaves = view_elems(w3, &def.name);
    let e = leaves
        .iter()
        .find(|e| e.elem == body)
        .expect("character body elem");
    if e.kind != "cylinder" {
        return Err(err(
            def,
            &format!(
                "character body '{body}' must be a cylinder3d leaf (built-in visuals are presentation-only; Kind is not character identity)"
            ),
        ));
    }
    if e.dyn_pos {
        return Err(err(
            def,
            &format!(
                "character body '{body}' position must be static (the Run character position is owned by move3d, not view bindings)"
            ),
        ));
    }
    let want = [
        CVal::Num(CHARACTER_SCALE[0]),
        CVal::Num(CHARACTER_SCALE[1]),
        CVal::Num(CHARACTER_SCALE[2]),
    ];
    if e.scale.len() != 3
        || e.scale.iter().zip(want.iter()).any(|(a, b)| !matches!((a, b), (CVal::Num(x), CVal::Num(y)) if x == y))
    {
        return Err(err(
            def,
            &format!(
                "character body '{body}' scale must be 0.4 x 0.5 x 0.4 (character3d/1 frozen 0.8x1.0x0.8 box)"
            ),
        ));
    }
    if e.solid {
        return Err(err(
            def,
            &format!("character body '{body}' must not also be marked solid"),
        ));
    }
    Ok(())
}

/// 相机配置解析（值域沿 runtime_camera3d/1 冻结域；具名预检）。
fn parse_camera_cfg(
    def: &DefIR,
    cam: &Camera3dBlock,
    target: i64,
) -> Result<(i64, f64, f64, i64, f64), LowerError> {
    let mut distance: Option<f64> = None;
    let mut pitch: Option<f64> = None;
    let mut projection: Option<i64> = None;
    let mut ortho_size: Option<f64> = None;
    let num = |v: &PropValue| -> Option<f64> {
        match v {
            PropValue::One(e) => numeric_lit(e),
            _ => None,
        }
    };
    for pr in &cam.props {
        match pr.name.as_str() {
            "distance" => distance = num(&pr.value),
            "pitch" => pitch = num(&pr.value),
            "ortho_size" => ortho_size = num(&pr.value),
            "projection" => match &pr.value {
                PropValue::One(Expr::Ident(p)) if p == "perspective" => projection = Some(0),
                PropValue::One(Expr::Ident(p)) if p == "ortho" => projection = Some(1),
                _ => {
                    return Err(err(
                        def,
                        "camera3d projection must be `perspective` or `ortho`",
                    ))
                }
            },
            other => {
                return Err(err(
                    def,
                    &format!(
                        "unknown camera3d prop '{other}' (known: distance/pitch/projection/ortho_size)"
                    ),
                ));
            }
        }
    }
    let Some(distance) = distance else {
        return Err(err(def, "camera3d missing 'distance'"));
    };
    let Some(pitch) = pitch else {
        return Err(err(def, "camera3d missing 'pitch'"));
    };
    let Some(projection) = projection else {
        return Err(err(def, "camera3d missing 'projection'"));
    };
    if !(4.0..=12.0).contains(&distance) {
        return Err(err(
            def,
            "camera3d distance must be within [4, 12] (runtime_camera3d/1 frozen domain)",
        ));
    }
    if !(15.0..=75.0).contains(&pitch) {
        return Err(err(
            def,
            "camera3d pitch must be within [15, 75] (runtime_camera3d/1 frozen domain)",
        ));
    }
    let size = if projection == 1 {
        let Some(size) = ortho_size else {
            return Err(err(def, "camera3d ortho projection requires 'ortho_size'"));
        };
        if !(1.0..=64.0).contains(&size) {
            return Err(err(
                def,
                "camera3d ortho_size must be within [1, 64] (runtime_camera3d/1 frozen domain)",
            ));
        }
        size
    } else {
        ortho_size.unwrap_or(0.0)
    };
    Ok((target, distance, pitch, projection, size))
}
