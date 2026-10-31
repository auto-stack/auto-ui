//! PLAN-017 `--profile g11-2d` 降层：完整 G11/2d 作者入口（T-02 冻结）。
//!
//! 与 016 legacy 路径的原则差异（全部声明驱动，无业务名特判）：
//! - 模板按实例生成（每触发实例一个场景模板行，几何/颜色来自实例的
//!   Mesh2D/Material2D 资源参数）；角色/地面/相机按声明端口识别。
//! - `input.axis(Left, Right)` 为真实差值（Left=-1/Right=+1/都无与
//!   同时按=0，held 计费 2）；`input.pressed(Enter)` 为消耗性边沿。
//! - 事件路由读 `ag_event_phase()`：on_enter/on_stay/on_exit 按实际
//!   phase 分派到对应处理器，不取首个 handler。
//! - 调度：移动 tick（本 tick 发生真实 char 扫掠）读最多 1 条事件、
//!   静止 tick 最多 3 条；首条 next==0 的 tick 才执行动态位置图形同步
//!   （原 001 "有事件 tick 不写危险图形位置、巡逻状态仍推进"的等价
//!   策略，预算算式见合同）。
//! - 重开：根 Life.Tick 在 tick 起点状态上判断终局，fresh Enter 边沿
//!   → `round.request()`（ag_round_request），宿主 accepted 后开新 Run。
//! 玩法规则（计数/守卫/胜负/巡逻公式/重开条件）全部在生成的 Auto 源。

use auto_parser_api::ast::{Expr, Stmt, Type};
use auto_ui_dialect::node2d::{EventPattern, NodeDecl, Prop, PropValue, View2dElement};

use crate::load::{node_of, Project};
use crate::lower::{
    collect_instances, fmt_f, mark_triggers, numeric_lit, slice_fn_source, snake, Instance,
};
use crate::scene;
use crate::translate::{translate_stmts, Ctx, LowerError, Target};

pub struct LowerOutputG11 {
    pub user_source: String,
    pub scene_json: String,
    pub source_sha256: String,
    pub source_bytes: usize,
    /// 角色/地面/相机/pair→实例 映射（identity.json 的 roles 段）。
    pub roles_json: String,
}

/// 参数端口类别（按声明类型识别，不按名字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// float/int 数值状态。
    Number,
    /// `msg` 事件端口。
    MsgEvent,
    /// `ActorRef2D` actor 引用。
    ActorRef,
    /// `Mesh2D` 资源 symbol。
    MeshSym,
    /// `Material2D` 资源 symbol。
    MatSym,
}

pub fn param_kind(ty: &Type) -> Result<ParamKind, String> {
    match ty {
        Type::Unknown => Ok(ParamKind::MsgEvent),
        Type::Float | Type::Int | Type::Double | Type::Byte => Ok(ParamKind::Number),
        Type::User(d) => match d.name.as_str() {
            "ActorRef2D" => Ok(ParamKind::ActorRef),
            "Mesh2D" => Ok(ParamKind::MeshSym),
            "Material2D" => Ok(ParamKind::MatSym),
            other => Err(format!(
                "unsupported port type '{other}' (research surface: float/int/msg/ActorRef2D/Mesh2D/Material2D)"
            )),
        },
        other => Err(format!(
            "unsupported port type {other:?} (research surface: float/int/msg/ActorRef2D/Mesh2D/Material2D)"
        )),
    }
}

fn unsupported(what: impl Into<String>) -> LowerError {
    LowerError::Unsupported {
        what: what.into(),
        line: 0,
    }
}

/// 实例属性解析结果（按参数端口类别）。
#[derive(Debug, Clone)]
pub enum ResolvedProp {
    /// 数值字面量 → SoA 初始化。
    NumberLit(f64),
    /// 表达式绑定（数值或 actor 端口）→ owner 上下文内联。
    Expr(Expr),
    /// 事件端口绑定：`.Evt` 或 `.Evt(args...)`（常量载荷）。
    EventBinding { event: String, args: Vec<Expr> },
    /// 资源 symbol：`resources.<name>`。
    Resource(String),
    /// `refs.<instance>.actor`。
    ActorRefOf(String),
}

fn resolve_prop_value(value: &PropValue) -> Result<ResolvedProp, LowerError> {
    let e = match value {
        PropValue::One(e) => e.clone(),
        PropValue::Pair(_) => return Err(unsupported("pair prop value in g11-2d profile")),
    };
    match &e {
        // 资源 symbol：resources.<name>；actor 引用：refs.<inst>.actor。
        // 两者先于事件绑定判定（同为 Dot 形态）。
        Expr::Dot(inner, name) => {
            if let Expr::Ident(base) = &**inner {
                if base == "resources" {
                    return Ok(ResolvedProp::Resource(name.to_string()));
                }
                return Ok(ResolvedProp::EventBinding {
                    event: name.to_string(),
                    args: Vec::new(),
                });
            }
            if name == "actor" {
                if let Expr::Dot(i2, inst_name) = &**inner {
                    if let Expr::Ident(base2) = &**i2 {
                        if base2 == "refs" {
                            return Ok(ResolvedProp::ActorRefOf(inst_name.to_string()));
                        }
                    }
                }
            }
            Err(unsupported(format!("prop value {e:?}")))
        }
        // 事件绑定：`.Evt(args...)`
        Expr::Call(c) => match &*c.name {
            Expr::Dot(inner, ev) => {
                if matches!(&**inner, Expr::Ident(_)) {
                    let mut args = Vec::new();
                    for a in &c.args.args {
                        match a {
                            auto_parser_api::ast::Arg::Pos(x) => args.push(x.clone()),
                            other => return Err(unsupported(format!("binding arg {other:?}"))),
                        }
                    }
                    return Ok(ResolvedProp::EventBinding {
                        event: ev.to_string(),
                        args,
                    });
                }
                Err(unsupported(format!("binding target {c:?}")))
            }
            _ => Err(unsupported(format!("prop call {c:?}"))),
        },
        Expr::Float(..) | Expr::Int(_) | Expr::Unary(..) => match numeric_lit(&e) {
            Some(v) => Ok(ResolvedProp::NumberLit(v)),
            None => Ok(ResolvedProp::Expr(e)),
        },
        _ => Ok(ResolvedProp::Expr(e)),
    }
}

/// node 定义的状态字段（g11-2d）：数值参数 + model 字段，带 int 判定。
/// 事件/actor/资源端口不是状态（按类型排除，不按名字）。
fn g11_fields(node: &NodeDecl) -> Vec<(String, bool)> {
    let mut fields = Vec::new();
    for (pname, ty) in &node.params {
        match param_kind(ty) {
            Ok(ParamKind::Number) => fields.push((pname.clone(), false)),
            _ => continue,
        }
    }
    if let Some(model) = &node.model {
        for f in &model.fields {
            let is_int = matches!((&f.ty, &f.init), (auto_parser_api::ast::Type::Int, _))
                || matches!(f.init, Expr::Int(_));
            fields.push((f.name.to_string(), is_int));
        }
    }
    fields
}

/// 触发体 shape 声明 → ag_trigger_bind (shape, radius)。缺省 AABB(0,0)。
fn trigger_shape(props: &[Prop]) -> Result<(i64, f64), LowerError> {
    let Some(pr) = props.iter().find(|p| p.name == "shape") else {
        return Ok((0, 0.0));
    };
    let e = match &pr.value {
        PropValue::One(e) => e,
        PropValue::Pair(_) => return Err(unsupported("shape pair value")),
    };
    let Expr::Call(c) = e else {
        return Err(unsupported("shape must be circle(radius: r)"));
    };
    // `circle(radius: r)` 在属性位解析为普通 Ident 调用；同时接受限定形。
    let kind = match &*c.name {
        Expr::Ident(n) => n.to_string(),
        Expr::Dot(_, m) => m.to_string(),
        other => return Err(unsupported(format!("shape target {other:?}"))),
    };
    if kind != "circle" {
        return Err(unsupported(format!("shape kind '{kind}'")));
    }
    for a in &c.args.args {
        match a {
            auto_parser_api::ast::Arg::Pair(n, v) if n.as_str() == "radius" => {
                let r = numeric_lit(v).ok_or_else(|| unsupported("shape radius literal"))?;
                if r <= 0.0 {
                    return Err(unsupported("shape radius must be > 0"));
                }
                return Ok((1, r));
            }
            auto_parser_api::ast::Arg::Pos(v) => {
                let r = numeric_lit(v).ok_or_else(|| unsupported("shape radius literal"))?;
                if r <= 0.0 {
                    return Err(unsupported("shape radius must be > 0"));
                }
                return Ok((1, r));
            }
            other => return Err(unsupported(format!("shape arg {other:?}"))),
        }
    }
    Err(unsupported("circle shape needs radius"))
}

/// view 元素查找辅助。
fn find_element<'a>(
    node: &'a NodeDecl,
    pred: impl Fn(&View2dElement) -> bool,
) -> Option<&'a View2dElement> {
    node.view2d.as_ref()?.elements.iter().find(|e| pred(e))
}

fn element_props(el: &View2dElement) -> Option<&Vec<Prop>> {
    match el {
        View2dElement::Solid { props, .. }
        | View2dElement::Trigger { props, .. }
        | View2dElement::Character { props, .. }
        | View2dElement::Camera { props, .. } => Some(props),
        _ => None,
    }
}

fn prop_of<'a>(props: &'a [Prop], name: &str) -> Option<&'a Prop> {
    props.iter().find(|p| p.name == name)
}

/// 定义是否声明 msg 事件。
fn has_event(node: &NodeDecl, event: &str) -> bool {
    node.msg
        .as_ref()
        .is_some_and(|m| m.variants.iter().any(|e| e.name.as_str() == event))
}

fn handler_of<'a>(
    node: &'a NodeDecl,
    pred: impl Fn(&EventPattern) -> bool,
) -> Option<&'a auto_ui_dialect::node2d::Handler> {
    node.on.as_ref()?.handlers.iter().find(|h| pred(&h.pattern))
}

fn member_handler<'a>(node: &'a NodeDecl, event: &str) -> Option<(&'a String, usize)> {
    handler_of(
        node,
        |p| matches!(p, EventPattern::Member { name, .. } if name.as_str() == event),
    )
    .map(|h| match &h.pattern {
        EventPattern::Member { name, params } => (name, params.len()),
        _ => unreachable!(),
    })
}

fn life_handler(node: &NodeDecl, event: &str) -> bool {
    handler_of(
        node,
        |p| matches!(p, EventPattern::Life { event: e, .. } if e == event),
    )
    .is_some()
}

/// 位置表达式是否引用可变状态（model 字段）→ 动态位置（需要图形同步）。
fn position_is_dynamic(node: &NodeDecl, props: &[Prop]) -> Result<bool, LowerError> {
    let model_names: Vec<String> = node
        .model
        .as_ref()
        .map(|m| m.fields.iter().map(|f| f.name.to_string()).collect())
        .unwrap_or_default();
    let Some(pos) = prop_of(props, "position") else {
        return Err(unsupported("trigger/character element missing position"));
    };
    let mut hits = false;
    for e in pos.value.exprs() {
        let mut cursor = e;
        loop {
            match cursor {
                Expr::Dot(inner, name) => {
                    if matches!(&**inner, Expr::Ident(_))
                        && model_names.iter().any(|m| m == name.as_str())
                    {
                        hits = true;
                    }
                    cursor = inner;
                }
                Expr::Ident(n) => {
                    if model_names.iter().any(|m| m == n.as_str()) {
                        hits = true;
                    }
                    break;
                }
                _ => break,
            }
        }
    }
    Ok(hits)
}

/// 位置表达式 → 生成文本 (x_text, y_text)（实例上下文）。
fn position_text(
    project: &Project,
    node_name: &str,
    props: &[Prop],
    prefixes: &std::collections::BTreeMap<String, String>,
    member_hook: &dyn Fn(&str) -> Option<String>,
    index: &str,
) -> Result<(String, String), LowerError> {
    let pos = prop_of(props, "position").ok_or_else(|| unsupported("missing position"))?;
    let es = match &pos.value {
        PropValue::Pair(es) => es.clone(),
        PropValue::One(Expr::Tuple(es)) => es.clone(),
        other => return Err(unsupported(format!("position {other:?}"))),
    };
    if es.len() != 2 {
        return Err(unsupported("position arity"));
    }
    let mut ctx = base_ctx_g11(project, node_name, prefixes, member_hook);
    if !is_root_g11(project, node_name) {
        ctx.target = Target::Instance {
            prefix: prefixes[node_name].clone(),
            index: index.to_string(),
        };
    }
    let mut out = String::new();
    translate_stmts(&[Stmt::Expr(es[0].clone())], &ctx, &mut out, 0)?;
    let x = out.trim().to_string();
    out.clear();
    translate_stmts(&[Stmt::Expr(es[1].clone())], &ctx, &mut out, 0)?;
    let y = out.trim().to_string();
    let _ = &mut ctx;
    Ok((x, y))
}

fn is_root_g11(project: &Project, node_name: &str) -> bool {
    node_name == project.root
}

fn base_ctx_g11<'a>(
    project: &'a Project,
    node_name: &str,
    prefixes: &'a std::collections::BTreeMap<String, String>,
    member_hook: &'a dyn Fn(&str) -> Option<String>,
) -> Ctx<'a> {
    let is_root = node_name == project.root;
    Ctx {
        target: if is_root {
            Target::Root
        } else {
            Target::Instance {
                prefix: prefixes[node_name].clone(),
                index: "i".to_string(),
            }
        },
        emit_hook: Box::new(|event: &str| {
            Err(unsupported(format!("emit .{event} without binding")))
        }),
        enum_path: Box::new(|ename: &str, variant: &str| enum_lookup(project, ename, variant)),
        axis_differential: true,
        member_hook: Some(Box::new(member_hook)),
        moved_flag: Some("_n2d_moved".to_string()),
        refs_any_elem: true,
    }
}

fn enum_lookup(project: &Project, ename: &str, variant: &str) -> Option<String> {
    for module in project.modules.values() {
        if let Some(variants) = module.enums.get(ename) {
            if variants.contains_key(variant) {
                return Some(format!(
                    "{}_{}",
                    ename.to_uppercase(),
                    variant.to_uppercase()
                ));
            }
        }
    }
    None
}

/// 常量载荷翻译（枚举路径/数值字面量）→ 数值字面量文本（顶层初始化
/// 只接受字面量；枚举常量在表列里内联为其判整值）。
fn const_arg_text(project: &Project, e: &Expr) -> Result<String, LowerError> {
    match e {
        Expr::Int(v) => Ok(v.to_string()),
        Expr::Float(v, _) => Ok(fmt_f(*v)),
        Expr::Dot(outer, variant) => {
            if let Expr::Ident(ename) = &**outer {
                for module in project.modules.values() {
                    if let Some(variants) = module.enums.get(ename.as_str()) {
                        if let Some(value) = variants.get(variant.as_str()) {
                            return Ok(value.to_string());
                        }
                    }
                }
                Err(unsupported(format!("unknown enum path {ename}.{variant}")))
            } else {
                Err(unsupported(format!("binding arg {e:?}")))
            }
        }
        Expr::Unary(auto_parser_api::ast::Op::Sub, inner) => {
            Ok(format!("-{}", const_arg_text(project, inner)?))
        }
        other => Err(unsupported(format!(
            "binding arg must be a constant (enum path or number), got {other:?}"
        ))),
    }
}

/// g11-2d 闭包校验（执行前定位拒绝）。返回 全局角色信息。
struct G11World {
    /// 角色 host 定义名。
    char_def: String,
    /// 角色 host 实例名。
    char_inst: String,
    /// 根 solid2d 元素（声明序）。
    grounds: Vec<(String, Vec<Prop>)>,
    /// 相机 props。
    camera: Option<Vec<Prop>>,
}

fn validate_g11(project: &Project) -> Result<G11World, LowerError> {
    let root = node_of(project, &project.root);
    let root_view = root
        .view2d
        .as_ref()
        .ok_or_else(|| unsupported("root node has no view-2d"))?;

    // 收集全部定义的 view 元素与实例。
    let mut char_defs: Vec<String> = Vec::new();
    let mut grounds: Vec<(String, Vec<Prop>)> = Vec::new();
    let mut cameras: Vec<Vec<Prop>> = Vec::new();
    let mut trigger_defs: Vec<String> = Vec::new();
    for module in project.modules.values() {
        for node in &module.nodes {
            let Some(view) = &node.view2d else { continue };
            for el in &view.elements {
                match el {
                    View2dElement::Character { .. } => {
                        char_defs.push(node.name.clone());
                    }
                    View2dElement::Solid { props, .. } => {
                        if node.name != project.root {
                            return Err(unsupported(format!(
                                "solid2d element outside the root view (node '{}')",
                                node.name
                            )));
                        }
                        grounds.push((node.name.clone(), props.clone()));
                    }
                    View2dElement::Camera { props, .. } => {
                        if node.name != project.root {
                            return Err(unsupported(format!(
                                "camera2d element outside the root view (node '{}')",
                                node.name
                            )));
                        }
                        cameras.push(props.clone());
                    }
                    View2dElement::Trigger { .. } => {
                        trigger_defs.push(node.name.clone());
                    }
                    _ => {}
                }
            }
        }
    }
    if char_defs.len() != 1 {
        return Err(unsupported(format!(
            "exactly one character2d element expected, got {char_defs:?}"
        )));
    }
    if cameras.len() > 1 {
        return Err(unsupported("at most one camera2d element"));
    }

    // 角色 host 实例（根 view 直属实例）。
    let char_def = char_defs[0].clone();
    let mut char_inst: Option<String> = None;
    for el in &root_view.elements {
        if let View2dElement::Instance { node, name, .. } = el {
            if *node == char_def {
                if char_inst.is_some() {
                    return Err(unsupported("multiple character host instances"));
                }
                char_inst = Some(name.clone());
            }
        }
    }
    let char_inst = char_inst.ok_or_else(|| {
        unsupported("character2d definition is never instantiated in the root view")
    })?;

    // 每个 trigger 定义：恰一个 trigger2d 元素；phase 绑定事件必须声明
    // 且有处理器；shape 形态合法。
    for def in &trigger_defs {
        let node = node_of(project, def);
        let count = node
            .view2d
            .as_ref()
            .map(|v| {
                v.elements
                    .iter()
                    .filter(|e| matches!(e, View2dElement::Trigger { .. }))
                    .count()
            })
            .unwrap_or(0);
        if count != 1 {
            return Err(unsupported(format!(
                "definition '{def}' must declare exactly one trigger2d element (got {count})"
            )));
        }
        let el = find_element(node, |e| matches!(e, View2dElement::Trigger { .. }))
            .ok_or_else(|| unsupported("trigger element"))?;
        let props = element_props(el).expect("trigger props");
        trigger_shape(props)?;
        position_is_dynamic(node, props)?;
        for phase_prop in ["on_enter", "on_stay", "on_exit"] {
            if let Some(p) = prop_of(props, phase_prop) {
                let ev = match resolve_prop_value(&p.value)? {
                    ResolvedProp::EventBinding { event, args } => {
                        if !args.is_empty() {
                            return Err(unsupported(format!(
                                "{phase_prop} binding must be a bare .Event (got args)"
                            )));
                        }
                        event
                    }
                    other => {
                        return Err(unsupported(format!(
                            "{phase_prop} must bind a .Event, got {other:?}"
                        )))
                    }
                };
                if !has_event(node, &ev) {
                    return Err(unsupported(format!(
                        "definition '{def}' binds {phase_prop} to undeclared event .{ev}"
                    )));
                }
                if member_handler(node, &ev).is_none() {
                    return Err(unsupported(format!(
                        "definition '{def}' binds {phase_prop} to .{ev} without a handler"
                    )));
                }
            }
        }
    }

    // 相机 props 形态（bounds 四元组 + view_size 二元组 + target）。
    if let Some(cam) = &cameras.first() {
        for key in ["target", "bounds", "view_size"] {
            if prop_of(cam, key).is_none() {
                return Err(unsupported(format!("camera2d missing '{key}'")));
            }
        }
    }

    Ok(G11World {
        char_def,
        char_inst,
        grounds,
        camera: cameras.into_iter().next(),
    })
}

pub fn lower_project_g112d(project: &Project) -> Result<LowerOutputG11, LowerError> {
    let nl: &'static str = "
";
    let world = validate_g11(project)?;
    let root_name = project.root.clone();
    let root = node_of(project, &root_name);
    let root_view = root.view2d.as_ref().expect("root view");

    // ---- 实例表（根 view 声明序；017 夹具无嵌套，但保留 016 展开机制）----
    let mut instances: Vec<Instance> = Vec::new();
    let mut per_def: std::collections::BTreeMap<String, usize> = Default::default();
    let mut gid = 1usize;
    for el in &root_view.elements {
        collect_instances(el, None, None, &mut instances, &mut gid, &mut per_def);
    }
    // 嵌套展开（与 legacy 相同的广度优先机制；g11 夹具不使用）。
    loop {
        let mut expanded = false;
        let defs_with_children: Vec<String> = {
            let mut ds: Vec<String> = Vec::new();
            for inst in &instances {
                if inst.owner.is_none() {
                    let node = node_of(project, &inst.node);
                    if let Some(view) = &node.view2d {
                        if view.elements.iter().any(|e| {
                            matches!(e, View2dElement::Instance { .. } | View2dElement::If { .. })
                        }) && !ds.contains(&inst.node)
                        {
                            ds.push(inst.node.clone());
                        }
                    }
                }
            }
            ds
        };
        for def in &defs_with_children {
            let owners: Vec<Instance> = instances
                .iter()
                .filter(|i| &i.node == def)
                .cloned()
                .collect();
            for owner in &owners {
                if instances.iter().any(|i| i.owner == Some(owner.gid)) {
                    continue;
                }
                let node = node_of(project, def);
                if let Some(view) = &node.view2d {
                    let before = instances.len();
                    for el in &view.elements {
                        collect_instances(
                            el,
                            None,
                            Some(owner.gid),
                            &mut instances,
                            &mut gid,
                            &mut per_def,
                        );
                    }
                    if instances.len() > before {
                        expanded = true;
                    }
                }
            }
        }
        if !expanded {
            break;
        }
    }

    // ---- 前缀/触发标记/slot-pair 分配 ----
    let mut prefixes: std::collections::BTreeMap<String, String> = Default::default();
    prefixes.insert(root_name.clone(), "r".to_string());
    let defs = {
        let mut ds: Vec<String> = Vec::new();
        for i in &instances {
            if !ds.contains(&i.node) {
                ds.push(i.node.clone());
            }
        }
        ds.sort();
        ds
    };
    for def in &defs {
        prefixes.insert(def.clone(), def.to_lowercase());
    }
    mark_triggers(project, &mut instances);
    let mut next_slot = 0usize;
    for inst in instances.iter_mut() {
        if inst.trigger {
            inst.slot = Some(next_slot);
            next_slot += 1;
        }
    }
    let trigger_instances: Vec<usize> = instances
        .iter()
        .filter(|i| i.trigger)
        .map(|i| i.gid)
        .collect();

    // ---- 模板 id：角色 1、地面 2..、触发实例按声明序 ----
    let char_template = 1usize;
    let ground_templates: Vec<usize> = (0..world.grounds.len()).map(|i| 2 + i).collect();
    let mut next_template = 2 + world.grounds.len();
    let mut inst_template: std::collections::BTreeMap<usize, usize> = Default::default();
    for gid in &trigger_instances {
        inst_template.insert(*gid, next_template);
        next_template += 1;
    }

    // ---- 实例属性解析（每实例按端口类别）----
    let mut resolved: std::collections::BTreeMap<usize, Vec<(String, ResolvedProp)>> =
        Default::default();
    for inst in &instances {
        let node = node_of(project, &inst.node);
        let mut out = Vec::new();
        for (pname, ty) in &node.params {
            let kind = param_kind(ty).map_err(unsupported)?;
            let pr = inst
                .props
                .iter()
                .find(|p| p.name.as_str() == pname.as_str())
                .ok_or_else(|| {
                    unsupported(format!(
                        "instance '{}' missing prop '{pname}' for node '{}'",
                        inst.name, inst.node
                    ))
                })?;
            let value = resolve_prop_value(&pr.value)?;
            match (&kind, &value) {
                (ParamKind::Number, ResolvedProp::NumberLit(_))
                | (ParamKind::Number, ResolvedProp::Expr(_))
                | (ParamKind::ActorRef, ResolvedProp::ActorRefOf(_))
                | (ParamKind::MsgEvent, ResolvedProp::EventBinding { .. })
                | (ParamKind::MeshSym, ResolvedProp::Resource(_))
                | (ParamKind::MatSym, ResolvedProp::Resource(_)) => {}
                (ParamKind::ActorRef, ResolvedProp::EventBinding { event, .. }) => {
                    return Err(unsupported(format!(
                        "instance '{}' prop '{pname}' looks like an event binding .{}; ActorRef2D ports take refs.<inst>.actor",
                        inst.name, event
                    )))
                }
                (ParamKind::ActorRef, ResolvedProp::Expr(_)) => {
                    return Err(unsupported(format!(
                        "instance '{}' prop '{pname}': ActorRef2D ports take refs.<inst>.actor",
                        inst.name
                    )))
                }
                (k, v) => {
                    return Err(unsupported(format!(
                        "instance '{}' prop '{pname}': port kind {k:?} cannot take value {v:?}",
                        inst.name
                    )))
                }
            }
            out.push((pname.clone(), value));
        }
        // 呈现覆盖（mesh/material 资源 props，映射到模板行；不是节点状态）。
        for pr in &inst.props {
            if pr.name == "mesh" || pr.name == "material" {
                if let ResolvedProp::Resource(_) = resolve_prop_value(&pr.value)? {
                    if out.iter().any(|(n, _)| n == &pr.name) {
                        continue;
                    }
                    out.push((pr.name.clone(), resolve_prop_value(&pr.value)?));
                }
            }
        }
        resolved.insert(inst.gid, out);
    }

    // ---- 触发实例 phase 绑定（on_enter/on_stay/on_exit → 事件名）----
    let mut phase_bindings: std::collections::BTreeMap<usize, [Option<String>; 3]> =
        Default::default();
    for inst in &instances {
        if !inst.trigger {
            continue;
        }
        let node = node_of(project, &inst.node);
        let el = find_element(node, |e| matches!(e, View2dElement::Trigger { .. }))
            .ok_or_else(|| unsupported("trigger element missing"))?;
        let props = element_props(el).expect("props");
        let mut binds = [None, None, None];
        for (i, phase_prop) in ["on_enter", "on_stay", "on_exit"].iter().enumerate() {
            if let Some(p) = prop_of(props, phase_prop) {
                match resolve_prop_value(&p.value)? {
                    ResolvedProp::EventBinding { event, .. } => binds[i] = Some(event),
                    other => {
                        return Err(unsupported(format!(
                            "{phase_prop} must bind .Event, got {other:?}"
                        )))
                    }
                }
            }
        }
        phase_bindings.insert(inst.gid, binds);
    }

    let mut src = String::new();
    src.push_str(
        "// GENERATED by autographics-node2d-pilot (PLAN-017 g11-2d profile, T-02 contract).\n",
    );
    src.push_str(&format!(
        "// Root: {root_name}; instances: {}\n",
        instances
            .iter()
            .map(|i| format!("{}:{}", i.name, i.node))
            .collect::<Vec<_>>()
            .join(",")
    ));
    src.push_str(&format!(
        "// Regeneration must be byte-identical (tools/verify_plan017.py).\n"
    ));
    src.push_str("var ok=0.0\n");
    src.push_str("var _n2d_moved=0\n");

    // ---- 枚举常量 ----
    for module in project.modules.values() {
        for (ename, variants) in &module.enums {
            for (vname, value) in variants {
                src.push_str(&format!(
                    "var {}_{}={value}\n",
                    ename.to_uppercase(),
                    vname.to_uppercase()
                ));
            }
        }
    }

    // ---- 根标量状态 ----
    let rprefix = &prefixes[&root_name];
    if let Some(model) = &root.model {
        for f in &model.fields {
            let lit = match &f.init {
                Expr::Int(v) => format!("{v}"),
                Expr::Float(_, raw) => raw.to_string(),
                other => return Err(unsupported(format!("root model init {other:?}"))),
            };
            src.push_str(&format!("var {rprefix}_{}={lit}\n", f.name));
        }
    }

    // ---- 每定义 SoA 数组（数值参数字面量 + model 字段）----
    for def in &defs {
        let node = node_of(project, def);
        let prefix = &prefixes[def];
        let own: Vec<&Instance> = instances.iter().filter(|i| &i.node == def).collect();
        for (field, is_int) in g11_fields(node) {
            // 表达式绑定参数（如 enabled）不是存储状态：owner 上下文内联，
            // 整个字段跳过 SoA 化（一致性由处理器翻译保证）。
            if own.iter().any(|inst| {
                matches!(
                    resolved[&inst.gid].iter().find(|(n, _)| n == &field),
                    Some((_, ResolvedProp::Expr(_)))
                )
            }) {
                continue;
            }
            let mut values = Vec::new();
            for inst in &own {
                let rprops = &resolved[&inst.gid];
                let lit = match rprops.iter().find(|(n, _)| n == &field) {
                    Some((_, ResolvedProp::NumberLit(v))) => Some(*v),
                    _ => None,
                };
                if let Some(v) = lit {
                    values.push(v);
                    continue;
                }
                // model 字段初值（可引用参数）。
                let model = node.model.as_ref().ok_or_else(|| {
                    unsupported(format!("field {field} missing on instance '{}'", inst.name))
                })?;
                let mf = model
                    .fields
                    .iter()
                    .find(|f| f.name.to_string() == field)
                    .ok_or_else(|| {
                        unsupported(format!("field {field} missing on instance '{}'", inst.name))
                    })?;
                let v = match &mf.init {
                    Expr::Int(v) => *v as f64,
                    Expr::Float(v, _) => *v,
                    Expr::Ident(p) => match rprops.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                        Some(ResolvedProp::NumberLit(v)) => *v,
                        _ => {
                            return Err(unsupported(format!(
                                "model init references non-literal param {p}"
                            )))
                        }
                    },
                    other => return Err(unsupported(format!("model init {other:?}"))),
                };
                values.push(v);
            }
            let list = values
                .iter()
                .map(|v| {
                    if is_int {
                        format!("{}", *v as i64)
                    } else {
                        fmt_f(*v)
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            src.push_str(&format!("var {prefix}_{field}=[{list}]\n"));
        }
        // handle / alive / spawned。
        let needs_handle = own.iter().any(|i| i.trigger);
        let needs_alive = own.iter().any(|i| i.conditional.is_some());
        if needs_handle {
            let zeros = own.iter().map(|_| "0").collect::<Vec<_>>().join(",");
            src.push_str(&format!("var {prefix}_handle=[{zeros}]\n"));
        }
        if needs_alive {
            let ones = own.iter().map(|_| "1").collect::<Vec<_>>().join(",");
            src.push_str(&format!("var {prefix}_alive=[{ones}]\n"));
            src.push_str(&format!("var {prefix}_spawned=[{ones}]\n"));
        }
    }

    // ---- 事件绑定表（每定义 × 事件端口参数 × 事件 → 每实例常量列；
    // 同定义实例可绑定不同事件：不绑定该事件的实例列占位 0）----
    for def in &defs {
        let node = node_of(project, def);
        let prefix = &prefixes[def];
        let own: Vec<&Instance> = instances.iter().filter(|i| &i.node == def).collect();
        for (pname, ty) in &node.params {
            if param_kind(ty).map_err(unsupported)? != ParamKind::MsgEvent {
                continue;
            }
            // 收集每实例绑定（事件名 + 载荷常量）。
            let mut bindings: Vec<(String, Vec<String>)> = Vec::new();
            for inst in &own {
                let rprops = &resolved[&inst.gid];
                match rprops.iter().find(|(n, _)| n.as_str() == pname.as_str()) {
                    Some((_, ResolvedProp::EventBinding { event, args })) => {
                        let mut consts = Vec::new();
                        for a in args {
                            consts.push(const_arg_text(project, a)?);
                        }
                        bindings.push((event.clone(), consts));
                    }
                    _ => {
                        return Err(unsupported(format!(
                            "instance '{}' port {pname} unbound",
                            inst.name
                        )))
                    }
                }
            }
            // 每个被绑定的事件一组列（实例序对齐，未绑定该事件的实例占位 0）。
            let mut events: Vec<String> = Vec::new();
            for (e, _) in &bindings {
                if !events.contains(e) {
                    events.push(e.clone());
                }
            }
            for event in &events {
                let arity = bindings
                    .iter()
                    .filter(|(e, _)| e == event)
                    .map(|(_, a)| a.len())
                    .next()
                    .unwrap_or(0);
                for (e, a) in bindings.iter() {
                    if e == event && a.len() != arity {
                        return Err(unsupported(format!(
                            "definition '{def}' port {pname}: mixed payload arity for .{event}"
                        )));
                    }
                }
                for ai in 0..arity {
                    let col: Vec<String> = bindings
                        .iter()
                        .map(|(e, a)| {
                            if e == event {
                                a[ai].clone()
                            } else {
                                "0".to_string()
                            }
                        })
                        .collect();
                    src.push_str(&format!(
                        "var {prefix}_bind_{}_{}=[{}]\n",
                        snake(event).to_uppercase(),
                        ai,
                        col.join(",")
                    ));
                }
                if arity == 0 {
                    let col: Vec<String> = bindings.iter().map(|_| "0".to_string()).collect();
                    src.push_str(&format!(
                        "var {prefix}_bind_{}=[{}]\n",
                        snake(event).to_uppercase(),
                        col.join(",")
                    ));
                }
            }
        }
    }

    // ---- 池化纯函数（verbatim 源切片）----
    for module in project.modules.values() {
        for (fname, _span) in &module.fns {
            let text = slice_fn_source(&module.source, fname)
                .ok_or_else(|| unsupported(format!("fn {fname} source slice")))?;
            src.push_str(&format!("// pooled fn {fname} from {}\n", module.name));
            src.push_str(text.trim_end());
            src.push('\n');
        }
    }

    // ---- 成员解析 hook 工厂（每定义 × owner 上下文）----
    // 参数表达式绑定在 owner 上下文内联求值；actor 端口内联为 actor 读取。
    let member_hook_for =
        |project: &Project,
         def: &str,
         prefixes: &std::collections::BTreeMap<String, String>,
         instances: &[Instance],
         resolved: &std::collections::BTreeMap<usize, Vec<(String, ResolvedProp)>>|
         -> Box<dyn Fn(&str) -> Option<String> + '_> {
            let node = node_of(project, def);
            let own: Vec<&Instance> = instances.iter().filter(|i| &i.node == def).collect();
            // 表达式绑定参数在所有实例上必须一致（共享处理器）。
            let mut inline: Vec<(String, Option<String>)> = Vec::new();
            for (pname, ty) in &node.params {
                let kind = match param_kind(ty) {
                    Ok(k) => k,
                    Err(_) => continue,
                };
                let expr_val = own.iter().find_map(|inst| {
                    match resolved[&inst.gid].iter().find(|(n, _)| n == pname) {
                        Some((_, ResolvedProp::Expr(e))) => Some(e.clone()),
                        _ => None,
                    }
                });
                if let Some(e) = expr_val {
                    // owner 上下文翻译（g11 夹具 owner 均为根）。
                    let owner_def = own
                        .iter()
                        .find_map(|inst| {
                            inst.owner.map(|g| {
                                instances
                                    .iter()
                                    .find(|o| o.gid == g)
                                    .expect("owner")
                                    .node
                                    .clone()
                            })
                        })
                        .unwrap_or_else(|| project.root.clone());
                    let owner_prefix = &prefixes[&owner_def];
                    let mut ctx = Ctx {
                        target: if owner_def == project.root {
                            Target::Root
                        } else {
                            Target::Instance {
                                prefix: owner_prefix.clone(),
                                index: "oi".to_string(),
                            }
                        },
                        emit_hook: Box::new(|_| unreachable!()),
                        enum_path: Box::new(|a, b| enum_lookup(project, a, b)),
                        axis_differential: true,
                        member_hook: None,
                        moved_flag: None,
                        refs_any_elem: true,
                    };
                    let mut buf = String::new();
                    if translate_stmts(&[Stmt::Expr(e.clone())], &ctx, &mut buf, 0).is_ok() {
                        inline.push((pname.clone(), Some(buf.trim().to_string())));
                    } else {
                        inline.push((pname.clone(), None));
                    }
                    let _ = &mut ctx;
                } else if kind == ParamKind::ActorRef {
                    // ActorRef 绑定为 refs.<inst>.actor（解析为 actor 读取文本）。
                    let refs_target = own.iter().find_map(|inst| {
                        match resolved[&inst.gid]
                            .iter()
                            .find(|(n, _)| n.as_str() == pname.as_str())
                        {
                            Some((_, ResolvedProp::ActorRefOf(target))) => Some(target.clone()),
                            _ => None,
                        }
                    });
                    let text = refs_target.and_then(|target_inst| {
                        let ti = instances.iter().find(|i| i.name == target_inst)?;
                        let tnode = node_of(project, &ti.node);
                        let hosts_char = tnode.view2d.as_ref().is_some_and(|v| {
                            v.elements
                                .iter()
                                .any(|e| matches!(e, View2dElement::Character { .. }))
                        });
                        if hosts_char {
                            Some("ag_actor(2)".to_string())
                        } else if ti.trigger {
                            Some(format!("{}_handle[{}]", prefixes[&ti.node], ti.idx))
                        } else {
                            None
                        }
                    });
                    inline.push((pname.clone(), text));
                }
            }
            Box::new(move |field: &str| {
                inline
                    .iter()
                    .find(|(n, _)| n == field)
                    .and_then(|(_, t)| t.clone())
            })
        };

    // ---- 处理器 ----
    let root_node = root;
    let rbase = snake(&root_name);
    if let Some(on) = &root_node.on {
        for h in &on.handlers {
            let (suffix, params): (String, String) = match &h.pattern {
                EventPattern::Life { event, .. } => {
                    let e = event.to_lowercase();
                    let ps = if e == "tick" {
                        "dt float".into()
                    } else {
                        String::new()
                    };
                    (format!("life{e}"), ps)
                }
                EventPattern::Member { name, params } => {
                    let ps = params
                        .iter()
                        .map(|p| format!("{p} int"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    (snake(name), ps)
                }
            };
            src.push_str(&format!("fn {rbase}_{suffix}({params}) {{\n"));
            let hook = member_hook_for(project, &root_name, &prefixes, &instances, &resolved);
            let ctx = base_ctx_g11(project, &root_name, &prefixes, &*hook);
            translate_stmts(&h.body.stmts, &ctx, &mut src, 1)?;
            src.push_str("}\n");
        }
    }
    for def in &defs {
        let node = node_of(project, def);
        let Some(on) = &node.on else { continue };
        let base = snake(def);
        let hook = member_hook_for(project, def, &prefixes, &instances, &resolved);
        // 事件端口 emit hook：绑定表投递（owner 根直调；嵌套 owner 走表）。
        let emit_targets: Vec<(String, String)> = {
            let mut out = Vec::new();
            for (pname, ty) in &node.params {
                if param_kind(ty).map_err(unsupported)? != ParamKind::MsgEvent {
                    continue;
                }
                let inst = own_first(&instances, def);
                let binding = inst
                    .props
                    .iter()
                    .find(|p| p.name.as_str() == pname.as_str())
                    .map(|p| resolve_prop_value(&p.value))
                    .transpose()?;
                if let Some(ResolvedProp::EventBinding { .. }) = binding {
                    // 每实例的投递调用（同定义实例可绑定不同事件/载荷）。
                    let mut calls: Vec<String> = Vec::new();
                    for inst in instances.iter().filter(|i| i.node.as_str() == def.as_str()) {
                        let rprops = &resolved[&inst.gid];
                        let (event, args) = match rprops
                            .iter()
                            .find(|(n, _)| n.as_str() == pname.as_str())
                        {
                            Some((_, ResolvedProp::EventBinding { event, args })) => (event, args),
                            _ => unreachable!("checked above"),
                        };
                        // 接收方处理器形参数必须等于绑定载荷数（投递协议：
                        // 参数全部来自绑定常量列；接收方无该形参的载荷即
                        // 定位拒绝，不产生缺参调用）。
                        let owner_def = inst
                            .owner
                            .map(|g| {
                                instances
                                    .iter()
                                    .find(|o| o.gid == g)
                                    .expect("owner")
                                    .node
                                    .clone()
                            })
                            .unwrap_or_else(|| project.root.clone());
                        let owner_node = node_of(project, &owner_def);
                        let recv_params = member_handler(owner_node, event.as_str())
                            .map(|(_, n)| n)
                            .unwrap_or(0);
                        if recv_params != args.len() {
                            return Err(unsupported(format!(
                                "instance '{}' port {pname}: binding .{} carries {} payload(s) but the receiving handler on '{}' declares {recv_params} param(s)",
                                inst.name,
                                event,
                                args.len(),
                                owner_def
                            )));
                        }
                        let owner_node = inst.owner.map(|g| {
                            instances
                                .iter()
                                .find(|o| o.gid == g)
                                .expect("owner")
                                .node
                                .clone()
                        });
                        let ev_u = snake(event.as_str()).to_uppercase();
                        let call = match owner_node {
                            None => {
                                let obase = snake(&project.root);
                                let prefix = &prefixes[def];
                                let arity = args.len();
                                if arity == 0 {
                                    format!("{obase}_{}()", snake(event.as_str()))
                                } else {
                                    let call_args = (0..arity)
                                        .map(|ai| format!("{prefix}_bind_{ev_u}_{ai}[i]"))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    format!("{obase}_{}({call_args})", snake(event.as_str()))
                                }
                            }
                            Some(owner) => {
                                let obase = snake(&owner);
                                let prefix = &prefixes[def];
                                format!(
                                    "{obase}_{}({prefix}_bind_{ev_u}[i])",
                                    snake(event.as_str())
                                )
                            }
                        };
                        calls.push(call);
                    }
                    let uniform = calls.iter().all(|c| c == &calls[0]);
                    let call = if uniform {
                        calls[0].clone()
                    } else {
                        let mut router = String::new();
                        for (idx, c) in calls.iter().enumerate() {
                            router.push_str(nl);
                            router.push_str(&format!("  if i=={idx} {{ {c} }}"));
                        }
                        router
                    };
                    out.push((pname.clone(), call));
                }
            }
            out
        };
        for h in &on.handlers {
            let suffix = match &h.pattern {
                EventPattern::Life { event, .. } => format!("life{}", event.to_lowercase()),
                EventPattern::Member { name, .. } => snake(name),
            };
            let extra = match &h.pattern {
                EventPattern::Life { event, .. } => {
                    if event.to_lowercase() == "tick" {
                        ", dt float".to_string()
                    } else {
                        String::new()
                    }
                }
                EventPattern::Member { params, .. } => params
                    .iter()
                    .map(|p| format!(", {p} int"))
                    .collect::<Vec<_>>()
                    .join(""),
            };
            src.push_str(&format!("fn {base}_{suffix}(i int{extra}) {{\n"));
            let mut ctx = base_ctx_g11(project, def, &prefixes, &*hook);
            let targets = emit_targets.clone();
            ctx.emit_hook = Box::new(move |event: &str| {
                targets
                    .iter()
                    .find(|(p, _)| p == event)
                    .map(|(_, c)| c.clone())
                    .ok_or_else(|| unsupported(format!("emit .{event} without binding")))
            });
            translate_stmts(&h.body.stmts, &ctx, &mut src, 1)?;
            src.push_str("}\n");
        }
    }

    // ---- 读回 getter ----
    emit_getters_g11(
        project,
        &root_name,
        &instances,
        &prefixes,
        &world.char_def,
        &resolved,
        &mut src,
    )?;

    // ---- app_start ----
    src.push_str("fn app_start() {\n");
    if life_handler(root, "Start") {
        src.push_str(&format!(" {rbase}_lifestart()\n"));
    }
    for inst in &instances {
        if life_handler(node_of(project, &inst.node), "Start") {
            let base = snake(&inst.node);
            src.push_str(&format!(" {base}_lifestart({})\n", inst.idx));
        }
    }
    let char_host_inst = instances
        .iter()
        .find(|i| i.name == world.char_inst)
        .ok_or_else(|| unsupported("char host instance missing"))?;
    let _ = char_host_inst;
    for (pair, gid) in trigger_instances.iter().enumerate() {
        let inst = instances.iter().find(|i| i.gid == *gid).unwrap();
        let prefix = &prefixes[&inst.node];
        let template = inst_template[gid];
        let node = node_of(project, &inst.node);
        let hook = member_hook_for(project, &inst.node, &prefixes, &instances, &resolved);
        let el = find_element(node, |e| matches!(e, View2dElement::Trigger { .. }))
            .ok_or_else(|| unsupported("trigger element"))?;
        let props = element_props(el).expect("props").clone();
        let (sx, sy) = position_text(
            project,
            &inst.node,
            &props,
            &prefixes,
            &*hook,
            &inst.idx.to_string(),
        )?;
        let (shape, radius) = trigger_shape(&props)?;
        let cond_text = match &inst.conditional {
            Some(cond) => {
                let hook2 =
                    member_hook_for(project, &project.root, &prefixes, &instances, &resolved);
                let ctx = base_ctx_g11(project, &project.root, &prefixes, &*hook2);
                let mut buf = String::new();
                translate_stmts(&[Stmt::Expr(cond.clone())], &ctx, &mut buf, 0)?;
                Some(buf.trim().to_string())
            }
            None => None,
        };
        let spawn_lines = format!(
            "   {prefix}_handle[{idx}]=ag_object_spawn({pair}, {template}, {sx}, {sy}, 0.0)\n   ok=ok+ag_trigger_bind({pair}, 1, 1, {prefix}_handle[{idx}], {shape}, {radius})*1.0\n",
            idx = inst.idx,
            radius = fmt_f(radius)
        );
        match &cond_text {
            Some(c) => {
                src.push_str(&format!(" if {c} {{\n"));
                src.push_str(&format!(
                    "  {prefix}_spawned[{0}]=1\n  {prefix}_alive[{0}]=1\n",
                    inst.idx
                ));
                src.push_str(&spawn_lines);
                src.push_str(" }\n");
            }
            None => src.push_str(&format!(" {}", spawn_lines)),
        }
    }
    src.push_str(&format!(" ok=ok+ag_char_bind({char_template})*1.0\n"));
    for gt in &ground_templates {
        src.push_str(&format!(" ok=ok+ag_char_solid({gt})*1.0\n"));
    }
    if let Some(cam) = &world.camera {
        let hook = member_hook_for(project, &project.root, &prefixes, &instances, &resolved);
        let mut vals = Vec::new();
        for key in ["bounds", "view_size"] {
            let p = prop_of(cam, key).ok_or_else(|| unsupported(format!("camera {key}")))?;
            let es = match &p.value {
                PropValue::Pair(es) => es.clone(),
                PropValue::One(Expr::Tuple(es)) => es.clone(),
                other => return Err(unsupported(format!("camera {key} {other:?}"))),
            };
            for e in &es {
                let mut buf = String::new();
                let mut ctx = base_ctx_g11(project, &project.root, &prefixes, &*hook);
                translate_stmts(&[Stmt::Expr(e.clone())], &ctx, &mut buf, 0)?;
                vals.push(buf.trim().to_string());
                let _ = &mut ctx;
            }
        }
        src.push_str(&format!(
            " ok=ok+ag_camera_follow_2d({char_template}, {})*1.0\n",
            vals.join(", ")
        ));
    }
    src.push_str("}\n");

    // ---- app_update ----
    src.push_str("fn app_update(dt float) {\n");
    // 移动标记每 tick 开头清零（合同 §4；置位只属于本 tick 的真实扫掠）。
    src.push_str(" _n2d_moved=0\n");
    if life_handler(root, "Tick") {
        src.push_str(&format!(" {rbase}_lifetick(dt)\n"));
    }
    for inst in &instances {
        if life_handler(node_of(project, &inst.node), "Tick") {
            let base = snake(&inst.node);
            src.push_str(&format!(" {base}_lifetick({}, dt)\n", inst.idx));
        }
    }
    // 事件段：移动 tick 恰 1 条、静止 tick 最多 3 条；phase 路由。
    let dispatch = |var_p: &str, var_ph: &str| -> Result<String, LowerError> {
        let mut blocks = Vec::new();
        for (pair, gid) in trigger_instances.iter().enumerate() {
            let inst = instances.iter().find(|i| i.gid == *gid).unwrap();
            let node = node_of(project, &inst.node);
            let base = snake(&inst.node);
            let binds = &phase_bindings[gid];
            let mut lines = Vec::new();
            for (pi, ev) in binds.iter().enumerate() {
                if let Some(ev) = ev {
                    if let Some((_, arity)) = member_handler(node, ev) {
                        let actor_arg = if arity >= 1 { ", ag_actor(2)" } else { "" };
                        lines.push(format!(
                            "  if {var_ph}=={} {{ {base}_{}({}{}{}) }}",
                            pi + 1,
                            snake(ev),
                            inst.idx,
                            actor_arg,
                            ""
                        ));
                    }
                }
            }
            if !lines.is_empty() {
                blocks.push(format!(
                    "  if {var_p}=={pair} {{\n{}\n  }}",
                    lines.join("\n")
                ));
            } else {
                blocks.push(format!("  if {var_p}=={pair} {{\n   \n  }}"));
            }
        }
        Ok(blocks.join("\n"))
    };
    src.push_str(" var n=ag_event_next()\n");
    src.push_str(" if n==1 {\n");
    src.push_str("  let p=ag_event_pair()\n");
    src.push_str("  let ph=ag_event_phase()\n");
    src.push_str(&dispatch("p", "ph")?);
    src.push_str(" }\n");
    src.push_str(" if _n2d_moved==0 {\n");
    src.push_str("  if n==1 {\n");
    src.push_str("   let n1=ag_event_next()\n");
    src.push_str("   if n1==1 {\n");
    src.push_str("    let p1=ag_event_pair()\n");
    src.push_str("    let ph1=ag_event_phase()\n");
    src.push_str(&indent_block(&dispatch("p1", "ph1")?, 1)?);
    src.push_str("    let n2=ag_event_next()\n");
    src.push_str("    if n2==1 {\n");
    src.push_str("     let p2=ag_event_pair()\n");
    src.push_str("     let ph2=ag_event_phase()\n");
    src.push_str(&indent_block(&dispatch("p2", "ph2")?, 2)?);
    src.push_str("    }\n");
    src.push_str("   }\n");
    src.push_str("  }\n");
    src.push_str(" }\n");
    // 动态位置图形同步：首条 event_next==0 的 tick（无论是否移动）；
    // 有事件 tick 跳过写、状态照常推进（原 001 等价策略，合同 §1/§4）。
    src.push_str(" if n==0 {\n");
    for gid in &trigger_instances {
        let inst = instances.iter().find(|i| i.gid == *gid).unwrap();
        let node = node_of(project, &inst.node);
        let el = find_element(node, |e| matches!(e, View2dElement::Trigger { .. }))
            .ok_or_else(|| unsupported("trigger element"))?;
        let props = element_props(el).expect("props").clone();
        if !position_is_dynamic(node, &props)? {
            continue;
        }
        let prefix = &prefixes[&inst.node];
        let hook = member_hook_for(project, &inst.node, &prefixes, &instances, &resolved);
        let (sx, sy) = position_text(
            project,
            &inst.node,
            &props,
            &prefixes,
            &*hook,
            &inst.idx.to_string(),
        )?;
        src.push_str(&format!(
            "  ok=ok+ag_object_move({prefix}_handle[{}], {sx}, {sy}, 0.0)*1.0\n",
            inst.idx
        ));
    }
    src.push_str(" }\n");
    // 条件差量执行器（真 spawn/despawn + 显式重绑）。
    for inst in &instances {
        let Some(cond) = &inst.conditional else {
            continue;
        };
        let prefix = &prefixes[&inst.node];
        let has_trigger = inst.trigger;
        let hook = member_hook_for(project, &project.root, &prefixes, &instances, &resolved);
        let mut ctx = base_ctx_g11(project, &project.root, &prefixes, &*hook);
        let mut cond_text = String::new();
        translate_stmts(&[Stmt::Expr(cond.clone())], &ctx, &mut cond_text, 0)?;
        let _ = &mut ctx;
        let cond_text = cond_text.trim().to_string();
        let template = inst_template[&inst.gid];
        let pair = trigger_instances
            .iter()
            .position(|g| *g == inst.gid)
            .unwrap_or(255);
        let node = node_of(project, &inst.node);
        let el = find_element(node, |e| matches!(e, View2dElement::Trigger { .. }))
            .ok_or_else(|| unsupported("trigger element"))?;
        let props = element_props(el).expect("props").clone();
        let hook2 = member_hook_for(project, &inst.node, &prefixes, &instances, &resolved);
        let (sx, sy) = position_text(
            project,
            &inst.node,
            &props,
            &prefixes,
            &*hook2,
            &inst.idx.to_string(),
        )?;
        let (shape, radius) = trigger_shape(&props)?;
        src.push_str(&format!(" if {cond_text} {{\n"));
        src.push_str(&format!("  if {prefix}_spawned[{}]==0 {{\n", inst.idx));
        src.push_str(&format!("   {prefix}_spawned[{}]=1\n", inst.idx));
        src.push_str(&format!("   {prefix}_alive[{}]=1\n", inst.idx));
        src.push_str(&format!(
            "   {prefix}_handle[{0}]=ag_object_spawn({pair}, {template}, {1}, {2}, 0.0)\n",
            inst.idx, sx, sy
        ));
        if has_trigger {
            src.push_str(&format!(
                "   ok=ok+ag_trigger_bind({pair}, 1, 1, {prefix}_handle[{idx}], {shape}, {radius})*1.0\n",
                idx = inst.idx,
                radius = fmt_f(radius)
            ));
        }
        src.push_str("  }\n");
        src.push_str(" } else {\n");
        src.push_str(&format!("  if {prefix}_spawned[{}]==1 {{\n", inst.idx));
        src.push_str(&format!("   {prefix}_spawned[{}]=0\n", inst.idx));
        src.push_str(&format!("   {prefix}_alive[{}]=0\n", inst.idx));
        src.push_str(&format!(
            "   ok=ok+ag_object_despawn({prefix}_handle[{0}])*1.0\n",
            inst.idx
        ));
        src.push_str("  }\n");
        src.push_str(" }\n");
    }
    src.push_str("}\n");

    let source_bytes = src.len();
    let source_sha256 = scene::sha256_hex(src.as_bytes());

    // ---- 场景文档 ----
    let scene_json = scene::build_scene_json_g11(
        project,
        &instances,
        &resolved,
        &world.char_def,
        &world.char_inst,
        &world.grounds,
        &world.camera,
        char_template,
        &ground_templates,
        &inst_template,
        &trigger_instances,
        &source_sha256,
        source_bytes,
        &root_name,
    )?;

    // ---- roles 身份 ----
    let mut roles = String::from("{\n");
    roles.push_str(&format!(
        "   \"character\": {{\"instance\": \"{}\", \"template\": {char_template}}},\n",
        world.char_inst
    ));
    roles.push_str(&format!(
        "   \"grounds\": [{}],\n",
        ground_templates
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",")
    ));
    roles.push_str("   \"pairs\": [");
    let pair_rows: Vec<String> = trigger_instances
        .iter()
        .enumerate()
        .map(|(p, gid)| {
            let inst = instances.iter().find(|i| i.gid == *gid).unwrap();
            format!(
                "\n    {{\"pair\": {p}, \"instance\": \"{}\", \"template\": {}}}",
                inst.name, inst_template[gid]
            )
        })
        .collect();
    roles.push_str(&pair_rows.join(","));
    roles.push_str("\n   ]\n  }");

    Ok(LowerOutputG11 {
        user_source: src,
        scene_json,
        source_sha256,
        source_bytes,
        roles_json: roles,
    })
}

fn own_first<'a>(instances: &'a [Instance], def: &str) -> &'a Instance {
    instances
        .iter()
        .find(|i| i.node == def)
        .expect("def has instances")
}

fn indent_block(block: &str, levels: usize) -> Result<String, LowerError> {
    let pad = " ".repeat(levels);
    let mut out = String::new();
    for line in block.lines() {
        if line.trim().is_empty() {
            continue;
        }
        out.push_str(&format!("{pad}{line}\n"));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn emit_getters_g11(
    project: &Project,
    root_name: &str,
    instances: &[Instance],
    prefixes: &std::collections::BTreeMap<String, String>,
    char_def: &str,
    resolved: &std::collections::BTreeMap<usize, Vec<(String, ResolvedProp)>>,
    src: &mut String,
) -> Result<(), LowerError> {
    let rprefix = &prefixes[root_name];
    if let Some(model) = node_of(project, root_name).model.as_ref() {
        for f in &model.fields {
            let name = f.name.to_string();
            let is_int = matches!(f.init, Expr::Int(_));
            let ty = if is_int { "int" } else { "float" };
            src.push_str(&format!(
                "fn get_{rprefix}_{name}() {ty} {{return {rprefix}_{name}}}\n"
            ));
        }
    }
    for def in defs_sorted_g11(instances) {
        let node = node_of(project, def.as_str());
        let prefix = &prefixes[def.as_str()];
        let own: Vec<&Instance> = instances
            .iter()
            .filter(|i| i.node.as_str() == def.as_str())
            .collect();
        let mut names: Vec<String> = own.iter().map(|i| i.name.clone()).collect();
        names.sort();
        names.dedup();
        let collision = names.len() != own.len()
            || own
                .iter()
                .any(|i| instances.iter().any(|o| o.node != *def && o.name == i.name));
        let getter = |inst: &Instance| -> String {
            if collision {
                format!("{}{}", def.to_lowercase(), inst.idx)
            } else {
                inst.name.clone()
            }
        };
        for (field, is_int) in g11_fields(node) {
            // 表达式绑定参数不是存储状态（owner 上下文内联），无 SoA、无 getter。
            if own.iter().any(|inst| {
                matches!(
                    resolved[&inst.gid].iter().find(|(n, _)| n == &field),
                    Some((_, ResolvedProp::Expr(_)))
                )
            }) {
                continue;
            }
            let ty = if is_int { "int" } else { "float" };
            for inst in &own {
                src.push_str(&format!(
                    "fn get_{}_{field}() {ty} {{return {prefix}_{field}[{}]}}\n",
                    getter(inst),
                    inst.idx
                ));
            }
        }
        if own.iter().any(|i| i.trigger) {
            for inst in &own {
                src.push_str(&format!(
                    "fn get_{}_handle() int {{return {prefix}_handle[{}]}}\n",
                    getter(inst),
                    inst.idx
                ));
                if inst.conditional.is_some() {
                    src.push_str(&format!(
                        "fn get_{}_alive() int {{return {prefix}_alive[{}]}}\n",
                        getter(inst),
                        inst.idx
                    ));
                }
            }
        }
        if def == char_def {
            src.push_str(
                "fn get_player_node_x() float {return ag_local(0,0)}\nfn get_player_node_y() float {return ag_local(0,1)}\n",
            );
        }
    }
    Ok(())
}

fn defs_sorted_g11(instances: &[Instance]) -> Vec<String> {
    let mut defs: Vec<String> = Vec::new();
    for i in instances {
        if !defs.contains(&i.node) {
            defs.push(i.node.clone());
        }
    }
    defs.sort();
    defs
}
