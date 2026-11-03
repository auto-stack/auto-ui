//! PLAN-030 components-3d-bindings：具名引用参数装配（profile 入口）。
//!
//! 沿 028 components-3d-events 的全部池/模板/主题机器（字节语义一致），
//! 新增的只有作者引用参数面：
//! - 四类只读名义端口 `SlotRef3D/TemplateRef3D/PairRef3D/ActorRef3D` +
//!   `msg`（017 血统）——参数在下降期解析为原数值 ABI（SoA int 列）与
//!   静态实例/根事件绑定文本；作者不写 slot/pair/scene ID。
//! - 逻辑 Node 可无 view-3d（局部状态/msg/on；§5.3）；实例 Life.Start
//!   自持 spawn/bind（020 冻结初始化序）；子 Node 零 Life.Tick，根显式
//!   调度（推进→事件→提交→重开；028 阶梯形状原样）。
//! - behavior_api 恒 autographics.triggers3d/1（零新版本）；旧 024/027/
//!   020/028 档字节不变（refs=false / bindings=None 全路径保持）。
//!
//! 池/模板校验与行表构建镜像 lower_components3d_events 同名逻辑
//! （028 冻结面不重构，双份一致性由双方 gates 固定）。

use crate::load::{node_of, Project};
use crate::lower_basic2d::LowerOutputBasic2D;
use crate::lower_components::{at, DefIR, ParamVal};
use crate::lower_components3d::{
    build_def_3d_with, build_world_3d_with, collect_scene_3d_with, cval_init_value,
    lower_project_components3d_cfg, Cfg3, TemplateRowIn, View3dIR,
};
use crate::node3d_binding_refs::{ref_kind_of, RefKind};
use crate::translate::{BindingsEnv, LowerError};
use std::collections::BTreeMap;

/// bindings 档装配身份：与 events 档同一受信组合，不新增运行时版本。
pub(crate) const BINDINGS_API: &str = "autographics.triggers3d/1";
/// PLAN-034：动画档装配身份（node_animation3d/1 受信单元超集）。
pub(crate) const BINDINGS_API_FOR_ANIMATION: &str = "autographics.node_animation3d/1";

/// 每定义每参数的解析后引用绑定（数值 ABI 已就位）。
#[derive(Debug, Clone)]
pub(crate) enum RefBinding {
    /// ActorRef3D → 目标实例（定义前缀 + def_idx；行 id 在使用点内联）。
    Actor { prefix: String, def_idx: usize },
    /// msg → 根事件接收 fn 名（如 `main_collected`；零预绑实参形态）。
    MsgEvent { call: String },
}

/// bindings 档下降期配置（Cfg3.bindings 载体；None = 旧档字节不变）。
#[derive(Clone, Default)]
pub(crate) struct BindingsCfg3 {
    /// 定义名 → (参数名 → 解析绑定)。Slot/Template/Pair 不入表（生成期
    /// 直接以 ParamVal::Res 名字查池/模板常量表），Actor/msg 在此。
    pub refs: BTreeMap<String, BTreeMap<String, RefBinding>>,
}

/// 按定义盖章 translate 环境（BindingsEnv；pool/主题常量沿 events3d）。
pub(crate) fn stamp_bindings_env(
    ctx: &mut crate::translate::Ctx,
    def_name: &str,
    project: &Project,
    bindings: Option<&BindingsCfg3>,
) {
    let Some(b) = bindings else { return };
    let decl = node_of(project, def_name);
    let mut ref_params = BTreeMap::new();
    let mut actors = BTreeMap::new();
    let mut msg_params = BTreeMap::new();
    for (pname, ty) in &decl.params {
        let Some(rk) = ref_kind_of(ty) else { continue };
        match rk {
            RefKind::Slot | RefKind::Template | RefKind::Pair => {
                ref_params.insert(pname.clone(), rk);
            }
            RefKind::Actor => {
                if let Some(RefBinding::Actor { prefix, def_idx }) =
                    b.refs.get(def_name).and_then(|m| m.get(pname))
                {
                    actors.insert(pname.clone(), (prefix.clone(), *def_idx));
                }
            }
            RefKind::MsgEvent => {
                if let Some(RefBinding::MsgEvent { call }) =
                    b.refs.get(def_name).and_then(|m| m.get(pname))
                {
                    msg_params.insert(pname.clone(), format!("{call}()"));
                }
            }
        }
    }
    if std::env::var("PLAN030_DEBUG").is_ok() {
        eprintln!("[plan030] stamp {def_name}: refs={:?} actors={:?} msg={:?}",
            ref_params.keys().collect::<Vec<_>>(),
            actors.keys().collect::<Vec<_>>(),
            msg_params.keys().collect::<Vec<_>>());
    }
    ctx.bindings3d = Some(BindingsEnv {
        ref_params,
        actors,
        msg_params,
    });
}

fn entry_of<'a>(
    cfg: &'a mut BindingsCfg3,
    dname: &str,
    pname: &str,
) -> std::collections::btree_map::Entry<'a, String, RefBinding> {
    cfg.refs
        .entry(dname.to_string())
        .or_default()
        .entry(pname.to_string())
}

/// bindings 档降低入口：池/模板校验 → 引用解析 → 共享管线。
pub fn lower_project_components3d_bindings(
    project: &Project,
) -> Result<LowerOutputBasic2D, LowerError> {
    lower_project_components3d_presentation_entry(project, None)
}

/// PLAN-034：components-3d-presentation 入口 = bindings 载体（池/模板/
/// 触发/引用/角色/相机收集复用同一实现）+ 具名纹理/动画元数据绑定面。
/// 动画资源在案时装配 node_animation3d/1（受信单元超集：角色/相机/触发/
/// 视觉），否则沿 bindings 冻结身份；None = 旧档字节不变。
pub fn lower_project_components3d_presentation_entry(
    project: &Project,
    presentation: Option<&crate::lower_node3d_presentation::PresentationMeta3>,
) -> Result<LowerOutputBasic2D, LowerError> {
    lower_project_components3d_presentation_entry_ahud(project, presentation, None)
}

/// PLAN-035：组合档变体——`presentation3d` 配置随档发行；装配身份恒
/// components3d/1（T-03 冻结；动画/池/相机的旧档身份选择不适用）。scene/8
/// 音频面（license 文本/资产行）由组合入口预先受检并随 AhudCfg3 携带。
pub fn lower_project_components3d_presentation_entry_ahud(
    project: &Project,
    presentation: Option<&crate::lower_node3d_presentation::PresentationMeta3>,
    ahud: Option<&crate::lower_components3d_audio_hud::AhudCfg3>,
) -> Result<LowerOutputBasic2D, LowerError> {
    let w3 = build_world_3d_with(project, true, true, presentation)?;
    let world = &w3.world;
    let root_name = world.root.clone();
    let root_decl = node_of(project, &root_name);

    // —— 池声明归属：仅 root（镜像 028 events 校验）——
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

    // —— 模板定义收集（镜像 events：模块表序 + 文件内声明序）——
    let mut template_defs: Vec<(String, DefIR, View3dIR)> = Vec::new();
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
            if !matches!(e.kind, "cylinder" | "player" | "mob") {
                return Err(at(
                    &def.module,
                    e.line,
                    &e.elem,
                    &format!(
                        "object3d template leaf '{}' must be cylinder3d/player3d/mob3d (v1 template kinds)",
                        e.elem
                    ),
                ));
            }
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
            template_defs.push((name.clone(), def, view));
        }
    }

    // —— 模板行（parked 值 = 作者声明；全部为字面量常量）——
    let mut template_rows: Vec<TemplateRowIn> = Vec::new();
    for (name, def, view) in &template_defs {
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
            resource: None,
        });
    }

    let scene = collect_scene_3d_with(project, &w3, &template_rows, presentation, &[])?;

    // —— 文档 id 解析（行名 → 1-based；与 027/028 同规则）——
    let doc_id = |row: &str| -> Option<i64> {
        scene.rows.iter().position(|r| r.name == row).map(|i| i as i64 + 1)
    };

    let mut template_ids = BTreeMap::new();
    for (name, _, _) in &template_defs {
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

    let events = crate::lower_components3d_events::EventsCfg3 {
        slot_ids,
        pair_ids,
        template_ids,
        subject_ids,
        template_rows,
    };

    // —— 引用参数解析（逐实例；数值 ABI = 原 pool/模板/实例路径身份）——
    let mut bindings_cfg = BindingsCfg3::default();
    // 实例参数查表：(def, param) → 每实例 Res 文本（世界构建期捕获）。
    let res_of = |dname: &str, pname: &str, def_idx: usize| -> Option<&String> {
        world
            .instances
            .iter()
            .find(|i| i.def == dname && i.def_idx == def_idx)
            .and_then(|i| i.params.iter().find(|(n, _)| n == pname))
            .and_then(|(_, v)| match v {
                ParamVal::Res(s) => Some(s),
                _ => None,
            })
    };
    for dname in world.defs.keys() {
        let decl = node_of(project, dname);
        for (pname, ty) in &decl.params {
            let Some(rk) = ref_kind_of(ty) else { continue };
            let n = world.def_len(dname);
            if n == 0 {
                continue;
            }
            for k in 0..n {
                // def_idx 序 = world.instances 过滤序（020 同规则）。
                let inst = world
                    .instances
                    .iter()
                    .filter(|i| i.def == *dname)
                    .nth(k)
                    .expect("instance by def_idx");
                let raw = res_of(dname, pname, inst.def_idx).ok_or_else(|| {
                    at(
                        &world.defs[dname].module,
                        inst.line,
                        pname,
                        &format!(
                            "instance '{}' ref param '{pname}' was not captured as a symbol binding",
                            inst.path
                        ),
                    )
                })?;
                let text: &str = raw.as_str();
                match rk {
                    RefKind::Slot => {
                        let name = text.strip_prefix("objects3d.").unwrap_or(text);
                        if !events.slot_ids.contains_key(name) {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds unknown slot '{name}' (objects3d pool)",
                                    inst.path
                                ),
                            ));
                        }
                        // 数值列由生成期按名字查表；无需绑定表项。
                    }
                    RefKind::Template => {
                        let name = text.strip_prefix("templates.").unwrap_or(text);
                        if !events.template_ids.contains_key(name) {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds unknown template '{name}' (no object3d template def of that name)",
                                    inst.path
                                ),
                            ));
                        }
                    }
                    RefKind::Pair => {
                        let name = text.strip_prefix("triggers.").unwrap_or(text);
                        if !events.pair_ids.contains_key(name) {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds unknown pair '{name}' (triggers pool)",
                                    inst.path
                                ),
                            ));
                        }
                    }
                    RefKind::Actor => {
                        let Some(path) = text.strip_prefix("refs.") else {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds '{text}'; ActorRef3D takes refs.<instance path>",
                                    inst.path
                                ),
                            ));
                        };
                        let target = world
                            .instances
                            .iter()
                            .enumerate()
                            .find(|(_, i)| {
                                inst_name_chain(&i.path) == path
                            })
                            .ok_or_else(|| {
                                at(
                                    &world.defs[dname].module,
                                    inst.line,
                                    pname,
                                    &format!(
                                        "instance '{}' param '{pname}' binds unknown actor path '{text}' (no static instance of that path in this Run)",
                                        inst.path
                                    ),
                                )
                            })?;
                        let (ti, tinst) = target;
                        let tdef = &world.defs[&tinst.def];
                        let entry = entry_of(&mut bindings_cfg, dname, pname).or_insert(RefBinding::Actor {
                            prefix: tdef.prefix(),
                            def_idx: tinst.def_idx,
                        });
                        match entry {
                            RefBinding::Actor { prefix, def_idx } => {
                                let want = (tdef.prefix(), tinst.def_idx);
                                if (prefix.as_str(), *def_idx) != (want.0.as_str(), want.1) {
                                    return Err(at(
                                        &world.defs[dname].module,
                                        inst.line,
                                        pname,
                                        &format!(
                                            "param '{pname}' actor binding must be uniform across instances of '{dname}' ({} binds a different target)",
                                            dname
                                        ),
                                    ));
                                }
                            }
                            _ => {
                                return Err(at(
                                    &world.defs[dname].module,
                                    inst.line,
                                    pname,
                                    "param kind collision in binding table",
                                ))
                            }
                        }
                        let _ = ti;
                    }
                    RefKind::MsgEvent => {
                        let Some(ev) = text.strip_prefix('.') else {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds '{text}'; a msg port takes `.RootEvent`",
                                    inst.path
                                ),
                            ));
                        };
                        let rdecl = node_of(project, &root_name);
                        let variant = rdecl
                            .msg
                            .as_ref()
                            .and_then(|m| m.variants.iter().find(|v| v.name.as_str() == ev))
                            .ok_or_else(|| {
                                at(
                                    &world.defs[dname].module,
                                    inst.line,
                                    pname,
                                    &format!(
                                        "instance '{}' param '{pname}' binds '.{ev}' which is not a root msg variant",
                                        inst.path
                                    ),
                                )
                            })?;
                        if !variant.payload.is_empty() {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds '.{ev}' which carries a payload (zero-arg form only in components-3d-bindings v1)",
                                    inst.path
                                ),
                            ));
                        }
                        let has_handler = rdecl.on.as_ref().is_some_and(|on| {
                            on.handlers.iter().any(|h| {
                                matches!(&h.pattern,
                                    auto_ui_dialect::node2d::EventPattern::Member { name, .. }
                                        if name.as_str() == ev)
                            })
                        });
                        if !has_handler {
                            return Err(at(
                                &world.defs[dname].module,
                                inst.line,
                                pname,
                                &format!(
                                    "instance '{}' param '{pname}' binds '.{ev}' which has no `on .{ev}` handler on the root",
                                    inst.path
                                ),
                            ));
                        }
                        // 017 血统命名：<root 前缀>_<ev>（root 载体；与
                        // plan 侧注册同名——plan 派发在 cfg 重建时已注册，
                        // 此处仅派生投递文本）。
                        let call = format!("{}_{}", world.root.to_lowercase(), ev.to_lowercase());
                        match entry_of(&mut bindings_cfg, dname, pname)
                            .or_insert(RefBinding::MsgEvent { call: call.clone() })
                        {
                            RefBinding::MsgEvent { call: c } => {
                                if c != &call {
                                    return Err(at(
                                        &world.defs[dname].module,
                                        inst.line,
                                        pname,
                                        &format!(
                                            "param '{pname}' msg binding must be uniform across instances of '{dname}' (bound '.{ev}' differs)"
                                        ),
                                    ));
                                }
                            }
                            _ => {
                                return Err(at(
                                    &world.defs[dname].module,
                                    inst.line,
                                    pname,
                                    "param kind collision in binding table",
                                ))
                            }
                        }
                    }
                }
            }
        }
    }
    // —— 装配：复用 027 runtime 收集（角色/相机注入位），身份恒 triggers3d/1 ——
    let runtime = crate::lower_components3d_runtime::collect_runtime_cfg_pub(
        project,
        &w3,
        &scene,
        presentation,
    )?;

    // PLAN-034：动画资源在案 → node_animation3d/1 装配（ag_visual_pose 等
    // 受信面）。无动画时：角色/相机/池任一在案 → bindings 载体身份
    // triggers3d/1；纯静态纹理场景（无角色纹理档）→ behavior/1（原 api v1
    // 面；triggers3d 档恒定 3D 角色装载，纯场景不该被装配成角色 Run）。
    let behavior_api = if ahud.is_some() {
        // PLAN-035：组合档恒定身份（T-03 冻结 117/28）。
        "autographics.components3d/1"
    } else {
        behavior_api_select(project, presentation, &events, &world)
    };

    lower_project_components3d_cfg_cfg(
        project,
        Cfg3 {
            runtime: Some(&runtime),
            events: Some(&events),
            bindings: Some(&bindings_cfg),
            presentation,
            doc_prefix: if ahud.is_some() {
                "plan035-components-3d-audio-hud"
            } else {
                "plan030-components-3d-bindings"
            },
            gen_note: "PLAN-030 components-3d-bindings",
            regen_note: if ahud.is_some() {
                "tools/verify_plan035.py"
            } else {
                "tools/verify_plan030.py"
            },
            behavior_api,
            ahud,
            queries3: None,
        },
    )
}

/// 旧档身份选择（动画资源 → node_animation3d/1；池/角色/相机 →
/// triggers3d/1；纯静态纹理场景 → behavior/1）。字节与 034 冻结一致。
fn behavior_api_select(
    project: &Project,
    presentation: Option<&crate::lower_node3d_presentation::PresentationMeta3>,
    events: &crate::lower_components3d_events::EventsCfg3,
    world: &crate::lower_components::WorldIR,
) -> &'static str {
    match presentation {
        Some(_) if crate::lower_node3d_presentation::has_animated_resource(project) => {
            "autographics.node_animation3d/1"
        }
        Some(_) => {
            let pools_present = !events.slot_ids.is_empty()
                || !events.pair_ids.is_empty()
                || !events.template_ids.is_empty();
            let char_or_cam_present = world.defs.keys().any(|dname| {
                let d = node_of(project, dname);
                d.character3d.is_some() || d.camera3d.is_some()
            });
            if pools_present || char_or_cam_present {
                BINDINGS_API
            } else {
                "autographics.behavior/1"
            }
        }
        _ => BINDINGS_API,
    }
}

/// 实例路径 → 作者名字链（去 root 段；嵌套段以'.'连接，如
/// "Main/parent/child" → "parent.child"；根直属 "Main/hero" → "hero"）。
fn inst_name_chain(path: &str) -> String {
    path.split('/').skip(1).collect::<Vec<_>>().join(".")
}

/// Cfg3 装配（bindings 载体经 Cfg3 进入共享管线；世界构建按档开 refs）。
fn lower_project_components3d_cfg_cfg(
    project: &Project,
    cfg: Cfg3<'_>,
) -> Result<LowerOutputBasic2D, LowerError> {
    crate::lower_components3d::lower_project_components3d_cfg_with(project, cfg, true)
}
