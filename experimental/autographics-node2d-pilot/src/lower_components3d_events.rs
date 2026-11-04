//! PLAN-028 components-3d-events：typed 模板/实例/触发事件/新局作者接线。
//!
//! 复用 024 共享三维管线（world/plan/scene/gen，Cfg3.events 注入位）与
//! 020 组件状态/投递面；新增的是声明面与名字表，不是第二套对象管理器：
//! - `object3d { }`：本定义为 spawn 模板（view-3d 唯一叶；静态
//!   scale/yaw/pitch 即模板身份；模板行按声明序 park 在实例行后）。
//!   PLAN-043：叶可为 model3d（具名静态 GLB 资源，资源共享/身份独立）。
//! - `objects3d { a, b, ... }`（仅 root）：具名对象槽池（声明序 0..8）。
//! - `triggers { a, b, ... }`（仅 root）：具名触发 pair 池（声明序 0..8）。
//! - `spawn(<slot>, <template>, x, y, z)` / `bind(<pair>, refs.<elem>,
//!   aabb|sphere, r, <src>)` / `event.*` / `object.*`：作者调用端口，1:1
//!   映射受信 objects3d/1 + triggers3d/1 服务（零隐藏读取、零新单价）。
//! - `round.request()`（027 已有）→ ag_round_request。
//!
//! behavior_api 恒为 autographics.triggers3d/1（108：char3d+cam3d+objects3d
//! +triggers 组合；驱动 start_selected_play 据此装配——对象/触发 registry
//! 在 app_start 前冻结，角色/相机在 app_start 后冻结）。旧 024/027/020 档
//! 字节不变（Cfg3.events=None 全路径保持）。

use crate::load::{node_of, Project};
use crate::lower_basic2d::LowerOutputBasic2D;
use crate::lower_components::{at, DefIR};
use crate::lower_components3d::{
    build_def_3d_with, build_world_3d_with, collect_scene_3d_with, cval_init_value,
    lower_project_components3d_cfg, resolve_model_3d, Cfg3, TemplateRowIn, View3dIR,
};
use crate::translate::LowerError;
use std::collections::BTreeMap;

/// events 档装配身份：复用 001/002 已交付组合，不新增运行时版本。
pub(crate) const EVENTS_API: &str = "autographics.triggers3d/1";

/// events 档下降期常量表（生成侧 EventsEnv 的来源）。
#[derive(Clone, Default)]
pub(crate) struct EventsCfg3 {
    /// objects3d 池：名字 → 槽位（声明序 0..8）。
    pub slot_ids: BTreeMap<String, i64>,
    /// triggers 池：名字 → pair（声明序 0..8）。
    pub pair_ids: BTreeMap<String, i64>,
    /// 模板定义名 → 模板行 1-based 文档 id。
    pub template_ids: BTreeMap<String, i64>,
    /// bind 主题：（定义前缀，元素名）→ 每实例 1-based 文档 id（def_idx
    /// 序；root 定义前缀为空串）。
    pub subject_ids: BTreeMap<(String, String), Vec<i64>>,
    /// 模板行（共享管线内部重建 scene 时按此追加；声明序）。
    pub template_rows: Vec<crate::lower_components3d::TemplateRowIn>,
}

/// events 档降低入口：声明校验 → 模板行 → 共享管线。
pub fn lower_project_components3d_events(
    project: &Project,
) -> Result<LowerOutputBasic2D, LowerError> {
    let w3 = build_world_3d_with(project, true, false, None)?;
    let world = &w3.world;
    let root_name = world.root.clone();
    let root_decl = node_of(project, &root_name);

    // —— 池声明归属：仅 root（其余定义携带即具名拒绝）——
    for dname in world.defs.keys() {
        if dname == &root_name {
            continue;
        }
        let d = node_of(project, dname);
        if d.objects3d.is_some() {
            return Err(at(
                &world.defs[dname].module,
                world.defs[dname].line,
                dname,
                "objects3d block is root-only (the Run object slot pool is one Run-owned table)",
            ));
        }
        if d.triggers.is_some() {
            return Err(at(
                &world.defs[dname].module,
                world.defs[dname].line,
                dname,
                "triggers block is root-only (the Run trigger pair pool is one Run-owned table)",
            ));
        }
    }

    // —— 具名槽池（≤8；名字唯一；声明序 0..8）——
    let mut slot_ids = BTreeMap::new();
    if let Some(pool) = &root_decl.objects3d {
        for (i, name) in pool.names.iter().enumerate() {
            if slot_ids.insert(name.clone(), i as i64).is_some() {
                return Err(at(
                    &world.root_module,
                    world.defs[&root_name].line,
                    name,
                    "duplicate objects3d slot name",
                ));
            }
        }
        if pool.names.len() > 8 {
            return Err(at(
                &world.root_module,
                world.defs[&root_name].line,
                "objects3d",
                &format!(
                    "objects3d slot pool exceeds the frozen 8-slot pool ({} names)",
                    pool.names.len()
                ),
            ));
        }
    }

    // —— 具名 pair 池（≤8；名字唯一；声明序 0..8）——
    let mut pair_ids = BTreeMap::new();
    if let Some(pool) = &root_decl.triggers {
        for (i, name) in pool.names.iter().enumerate() {
            if pair_ids.insert(name.clone(), i as i64).is_some() {
                return Err(at(
                    &world.root_module,
                    world.defs[&root_name].line,
                    name,
                    "duplicate triggers pair name",
                ));
            }
        }
        if pool.names.len() > 8 {
            return Err(at(
                &world.root_module,
                world.defs[&root_name].line,
                "triggers",
                &format!(
                    "triggers pair pool exceeds the frozen 8-pair pool ({} names)",
                    pool.names.len()
                ),
            ));
        }
    }

    // —— 模板定义收集（模块表序 + 文件内声明序；确定性）——
    // PLAN-043：第四分量为具名模型资源键（非 model3d 叶为 None）。
    let mut template_defs: Vec<(String, DefIR, View3dIR, Option<String>)> = Vec::new();
    for module in project.modules.values() {
        for decl in &module.nodes {
            if decl.object3d.is_none() {
                continue;
            }
            let name = &decl.name;
            if *name == root_name {
                return Err(at(
                    &world.root_module,
                    world.defs[&root_name].line,
                    name,
                    "object3d template marker is not allowed on the root definition",
                ));
            }
            if world.instances.iter().any(|i| &i.def == name) {
                return Err(at(
                    &world.root_module,
                    world.defs[&root_name].line,
                    name,
                    &format!(
                        "object3d template '{name}' must not be instanced in any view (spawn templates materialize as one parked row)"
                    ),
                ));
            }
            if !decl.params.is_empty() {
                return Err(at(
                    &world.root_module,
                    0,
                    name,
                    "object3d template must not declare params (template geometry is one frozen row)",
                ));
            }
            if decl.model.is_some() || decl.msg.is_some() || decl.on.is_some() {
                return Err(at(
                    &world.root_module,
                    0,
                    name,
                    &format!(
                        "object3d template '{name}' must not declare model/msg/on (v1 template is a parked visual row; instance business lives on the spawner)"
                    ),
                ));
            }
            if decl.character3d.is_some() || decl.camera3d.is_some() {
                return Err(at(
                    &world.root_module,
                    0,
                    name,
                    &format!(
                        "object3d template '{name}' must not declare character3d/camera3d (templates are presentation rows, not the Run character)"
                    ),
                ));
            }
            let (def, view) = build_def_3d_with(project, name, false, true, false)?;
            if !view.groups.is_empty() || view.elems.len() != 1 {
                return Err(at(
                    &def.module,
                    def.line,
                    name,
                    &format!(
                        "object3d template '{name}' must have exactly one view-3d leaf element (got {} elems, {} groups)",
                        view.elems.len(),
                        view.groups.len()
                    ),
                ));
            }
            let e = &view.elems[0];
            // PLAN-043：model3d 叶（具名静态 GLB 资源）接入 spawn 模板域，
            // 与 cylinder3d/player3d/mob3d 同为"一个静态模板叶"；资源身份
            // 经 034 公开准入链解析（平色 sha 键；closure 键随 T-02 冻结的
            // 组合入口）。其余形状仍按 028 拒绝域。
            let resource = match e.kind {
                "cylinder" | "player" | "mob" => None,
                "model" => {
                    let key = resolve_model_3d(&w3, &def, None, e, None).map_err(|err| {
                        at(
                            &def.module,
                            e.line,
                            &e.elem,
                            &format!("object3d template leaf model resource: {err}"),
                        )
                    })?;
                    Some(key.ok_or_else(|| {
                        at(
                            &def.module,
                            e.line,
                            &e.elem,
                            &format!(
                                "object3d template leaf '{}' model3d requires a named resources.<name> model3d.glb binding",
                                e.elem
                            ),
                        )
                    })?)
                }
                _ => {
                    return Err(at(
                        &def.module,
                        e.line,
                        &e.elem,
                        &format!(
                            "object3d template leaf '{}' must be cylinder3d/player3d/mob3d/model3d (v1 template kinds)",
                            e.elem
                        ),
                    ));
                }
            };
            if e.dyn_pos || e.dyn_vis {
                return Err(at(
                    &def.module,
                    e.line,
                    &e.elem,
                    "object3d template leaf position/visible must be static (the template row is frozen)",
                ));
            }
            if e.solid {
                return Err(at(
                    &def.module,
                    e.line,
                    &e.elem,
                    "object3d template leaf must not be solid (spawn templates are not the static collision surface)",
                ));
            }
            template_defs.push((name.clone(), def, view, resource));
        }
    }

    // —— 模板行（parked 值 = 作者声明；全部为字面量常量）——
    let mut template_rows: Vec<TemplateRowIn> = Vec::new();
    for (name, def, view, resource) in &template_defs {
        let e = &view.elems[0];
        let mut translation = [0.0f64; 3];
        for (i, v) in e.position.iter().enumerate() {
            translation[i] = cval_init_value(project, &w3, def, None, v, &format!("{name}:position"))?;
        }
        let mut scale = [1.0f64; 3];
        for (i, v) in e.scale.iter().enumerate() {
            scale[i] = cval_init_value(project, &w3, def, None, v, &format!("{name}:scale"))?;
        }
        let yaw = cval_init_value(project, &w3, def, None, &e.yaw, &format!("{name}:yaw"))?;
        let pitch = cval_init_value(project, &w3, def, None, &e.pitch, &format!("{name}:pitch"))?;
        let visible =
            cval_init_value(project, &w3, def, None, &e.visible, &format!("{name}:visible"))? != 0.0;
        template_rows.push(TemplateRowIn {
            name: name.clone(),
            kind: e.kind,
            translation,
            scale,
            yaw,
            pitch,
            visible,
            resource: resource.clone(),
        });
    }

    let scene = collect_scene_3d_with(project, &w3, &template_rows, None, &[])?;

    // —— 文档 id 解析（行名 → 1-based；与 027 row_document_id 同规则）——
    let doc_id = |row: &str| -> Option<i64> {
        scene.rows.iter().position(|r| r.name == row).map(|i| i as i64 + 1)
    };

    let mut template_ids = BTreeMap::new();
    for (name, _, _, _) in &template_defs {
        let id = doc_id(name).ok_or_else(|| {
            at(
                &world.root_module,
                0,
                name,
                "object3d template row missing in scene table",
            )
        })?;
        template_ids.insert(name.clone(), id);
    }

    // —— bind 主题表：root（""前缀）+ 每定义每元素每实例 ——
    let mut subject_ids: BTreeMap<(String, String), Vec<i64>> = BTreeMap::new();
    for (elem, row) in &scene.root_rows {
        let id = doc_id(row).ok_or_else(|| {
            at(
                &world.root_module,
                0,
                elem,
                "root element row missing in scene table",
            )
        })?;
        subject_ids.insert((String::new(), elem.clone()), vec![id]);
    }
    for (dname, elems) in &scene.elem_rows {
        let prefix = dname.to_lowercase();
        for (elem, rows) in elems {
            let mut ids = Vec::new();
            for r in rows {
                let id = doc_id(r).ok_or_else(|| {
                    at(
                        &world.root_module,
                        0,
                        r,
                        "element row missing in scene table",
                    )
                })?;
                ids.push(id);
            }
            subject_ids.insert((prefix.clone(), elem.clone()), ids);
        }
    }

    let events = EventsCfg3 {
        slot_ids,
        pair_ids,
        template_ids,
        subject_ids,
        template_rows,
    };

    // —— 装配：复用 027 runtime 收集（角色/相机注入位），身份恒 triggers3d/1 ——
    let runtime =
        crate::lower_components3d_runtime::collect_runtime_cfg_pub(project, &w3, &scene, None)?;

    lower_project_components3d_cfg(
        project,
        Cfg3 {
            runtime: Some(&runtime),
            events: Some(&events),
            bindings: None,
            doc_prefix: "plan028-components-3d-events",
            gen_note: "PLAN-028 components-3d-events",
            regen_note: "tools/verify_plan028.py",
            presentation: None,
            ahud: None,
        queries3: None,
            behavior_api: EVENTS_API,
        },
    )
}
