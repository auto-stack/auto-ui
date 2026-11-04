//! PLAN-039：queries-3d 档（三维 segment/sweep、具名叶/组引用、同根有限
//! 双出口 view-3d＋view-2d 诊断标记）。复用 024 components-3d 世界/场景/
//! 生成管线（`Cfg3.queries3` 随档发行；旧档路径字节不变），新增：
//! - root 有限双出口准入（仅 root 允许 view-2d 标记；子 Node 单维）；
//! - shape2d 标记行收集（屏幕坐标，z 限 0；quad/circle）；
//! - query3d 面（`query3d.segment(8 参)/sweep(5 参)`，typed let 结果沿
//!   021 同方法族；发射空间 3）与三维行 typing（组不进查询 handle）；
//! - scene 行为 API 按内容派生：仅 segment → autographics.behavior/3；
//!   含 sweep → autographics.behavior/4（G05 遗产两出口一致）。
//!
//! 非：完整 G3 游戏/角色相机、精确三角/OBB、动态拓扑、任意 UI 内嵌 3D
//! 视口、多屏呈现变体（见 Plan §1 非目标）。

use crate::lower_basic2d::LowerOutputBasic2D;
use crate::lower_components3d::{lower_project_components3d_cfg, Cfg3, MarkerRow3, Queries3Cfg};
use crate::lower_components::at;
use crate::lower::numeric_lit;
use crate::translate::LowerError;
use crate::Project;
use auto_parser_api::ast::{Expr, Stmt};
use auto_ui_dialect::node2d::{Prop, PropValue, View2dElement};

/// 有限查询世界行为 API（segment-only；G05 segment.at 遗产）。
pub const BEHAVIOR_API_V3: &str = "autographics.behavior/3";
/// 有限查询世界行为 API（含 sweep；G05 sweep.at 遗产，superset 单元）。
pub const BEHAVIOR_API_V4: &str = "autographics.behavior/4";

/// queries-3d 档入口：准入 → 桥接共享 3D 管线。
pub fn lower_project_queries3d(project: &Project) -> Result<LowerOutputBasic2D, LowerError> {
    lower_project_queries3d_identity(
        project,
        "plan039-queries-3d",
        "PLAN-039 queries-3d",
        "tools/verify_plan039.py",
    )
}

/// PLAN-040 editor-projects：同一静态双出口机制，仅换冻结身份串
/// （document_id 前缀/生成说明/再生控制注记）。机制、准入与产物字节规则
/// 与 queries-3d 完全一致；不为纯场景新增语法或渲染。支持域冻结为纯声明
/// （behavior:null）：stateful/Life 工程属运行档域，具名拒绝不静默扩域。
pub fn lower_project_editor_projects(
    project: &Project,
) -> Result<LowerOutputBasic2D, LowerError> {
    let lowered = lower_project_queries3d_identity(
        project,
        "plan040-editor-projects",
        "PLAN-040 editor-projects",
        "tools/verify_plan040.py",
    )?;
    if lowered.user_source.is_some() {
        return Err(crate::lower_components::at(
            "",
            0,
            "editor-projects",
            "editor-projects accepts pure declarative projects only: behavior must stay null (stateful/Life authoring belongs to the runtime profiles)",
        ));
    }
    Ok(lowered)
}

fn lower_project_queries3d_identity(
    project: &Project,
    doc_prefix: &'static str,
    gen_note: &'static str,
    regen_note: &'static str,
) -> Result<LowerOutputBasic2D, LowerError> {
    // 有限组合：仅 root 允许 view-2d（子 Node 单维；旧档解析期拒绝在
    // 默认 dialect 路径字节不变）。
    for (module_name, module) in &project.modules {
        for decl in &module.nodes {
            if decl.name == project.root {
                continue;
            }
            if decl.view2d.is_some() {
                return Err(at(
                    module_name,
                    decl.span.0,
                    &decl.name,
                    "view-2d is only allowed on the queries-3d root node (child nodes belong to one dimension)",
                ));
            }
        }
    }
    let markers = collect_markers(project)?;
    let behavior_api = if project_has_sweep(project) {
        BEHAVIOR_API_V4
    } else {
        BEHAVIOR_API_V3
    };
    lower_project_components3d_cfg(
        project,
        Cfg3 {
        runtime: None,
        events: None,
        bindings: None,
        presentation: None,
        doc_prefix,
        gen_note,
        regen_note,
        behavior_api,
            ahud: None,
            queries3: Some(&Queries3Cfg { markers }),
        },
    )
}

fn root_decl_of(
    project: &Project,
) -> Result<(String, auto_ui_dialect::node2d::NodeDecl), LowerError> {
    let module_name = project
        .node_modules
        .get(&project.root)
        .cloned()
        .unwrap_or_default();
    if !project.modules.contains_key(&module_name) {
        return Err(at(&module_name, 0, &project.root, "root node module missing"));
    }
    let decl = project
        .modules
        .get(&module_name)
        .and_then(|m| m.nodes.iter().find(|d| d.name == project.root))
        .cloned()
        .ok_or_else(|| at(&module_name, 0, &project.root, "root node missing"))?;
    Ok((module_name, decl))
}

/// root view-2d shape2d 标记收集 + 有限双出口准入。
fn collect_markers(project: &Project) -> Result<Vec<MarkerRow3>, LowerError> {
    let (module_name, root) = root_decl_of(project)?;
    let Some(v2) = &root.view2d else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for el in &v2.elements {
        let View2dElement::Shape { name, props } = el else {
            return Err(at(
                &module_name,
                root.span.0,
                el_name(el),
                "view-2d markers are shape2d leaves (kind: circle|quad) in the queries-3d profile",
            ));
        };
        out.push(parse_marker(&module_name, name, props)?);
    }
    Ok(out)
}

fn el_name(el: &View2dElement) -> &str {
    match el {
        View2dElement::Solid { name, .. }
        | View2dElement::Trigger { name, .. }
        | View2dElement::Character { name, .. }
        | View2dElement::Camera { name, .. }
        | View2dElement::Shape { name, .. } => name,
        View2dElement::Group { name, .. } => name,
        View2dElement::Instance { name, .. } => name,
        View2dElement::If { .. } => "if",
    }
}

fn num(e: &Expr, what: &str, module: &str, line: usize) -> Result<f64, LowerError> {
    numeric_lit(e).ok_or_else(|| {
        at(
            module,
            line,
            what,
            "marker numeric prop must be a literal number",
        )
    })
}

fn axis_vec(
    props: &[Prop],
    key: &str,
    fixed: f64,
    default: [f64; 3],
    module: &str,
    line: usize,
) -> Result<[f64; 3], LowerError> {
    let Some(p) = props.iter().find(|p| p.name == key) else {
        return Ok(default);
    };
    let items: Vec<Expr> = match &p.value {
        PropValue::Pair(es) => es.clone(),
        PropValue::One(Expr::Tuple(es)) => es.clone(),
        PropValue::One(e) => vec![e.clone()],
    };
    if items.len() < 2 || items.len() > 3 {
        return Err(at(
            module,
            line,
            key,
            &format!("{key} takes 2 or 3 literal components"),
        ));
    }
    let mut out = [0.0f64; 3];
    for (i, e) in items.iter().enumerate() {
        out[i] = num(e, key, module, line)?;
    }
    if items.len() == 2 {
        out[2] = fixed;
    } else if out[2] != fixed {
        return Err(at(
            module,
            line,
            key,
            &format!("{key} z must stay at the literal {} (2D marker rows)", fixed),
        ));
    }
    Ok(out)
}

fn parse_marker(module: &str, name: &str, props: &[Prop]) -> Result<MarkerRow3, LowerError> {
    let line = props.first().map(|p| p.span.0).unwrap_or(0);
    let kind = match props.iter().find(|p| p.name == "kind").map(|p| &p.value) {
        Some(PropValue::One(Expr::Ident(k))) if k == "quad" || k == "circle" => {
            if k == "quad" {
                "quad"
            } else {
                "circle"
            }
        }
        _ => {
            return Err(at(
                module,
                line,
                name,
                "shape2d marker needs `kind: quad` (or circle) in the queries-3d profile",
            ))
        }
    };
    let translation = axis_vec(props, "position", 0.0, [0.0, 0.0, 0.0], module, line)?;
    let scale = axis_vec(props, "scale", 1.0, [1.0, 1.0, 1.0], module, line)?;
    let rgba = match props.iter().find(|p| p.name == "rgba").map(|p| &p.value) {
        Some(PropValue::Pair(items)) => {
            if items.len() != 4 {
                return Err(at(module, line, name, "rgba takes 4 literal components"));
            }
            let mut c = [1.0f64; 4];
            for (i, e) in items.iter().enumerate() {
                c[i] = num(e, "rgba", module, line)?;
            }
            c
        }
        _ => [1.0, 1.0, 1.0, 1.0],
    };
    let visible = match props.iter().find(|p| p.name == "visible").map(|p| &p.value) {
        Some(PropValue::One(Expr::Bool(b))) => *b,
        None => true,
        _ => {
            return Err(at(
                module,
                line,
                name,
                "marker visible must be a literal true/false",
            ))
        }
    };
    for p in props {
        if !matches!(p.name.as_str(), "kind" | "position" | "scale" | "rgba" | "visible") {
            return Err(at(
                module,
                line,
                name,
                &format!(
                    "unknown marker prop '{}' (kind/position/scale/rgba/visible)",
                    p.name
                ),
            ));
        }
    }
    Ok(MarkerRow3 {
        name: name.to_string(),
        kind,
        translation,
        scale,
        rgba,
        visible,
    })
}

/// 行为 API 按内容派生：任一 handler 携带 query3d.sweep → v4。
fn project_has_sweep(project: &Project) -> bool {
    for module in project.modules.values() {
        for decl in &module.nodes {
            let Some(on) = &decl.on else { continue };
            for h in &on.handlers {
                if stmts_have_sweep(&h.body.stmts) {
                    return true;
                }
            }
        }
    }
    false
}

fn stmts_have_sweep(stmts: &[Stmt]) -> bool {
    for s in stmts {
        let hit = match s {
            Stmt::Expr(e) => expr_has_sweep(e),
            Stmt::Store(v) => expr_has_sweep(&v.expr),
            Stmt::Return(e) => expr_has_sweep(e),
            Stmt::If(if_) => {
                if_.branches.iter().any(|b| {
                    expr_has_sweep(&b.cond) || stmts_have_sweep(&b.body.stmts)
                }) || if_
                    .else_
                    .as_ref()
                    .is_some_and(|e| stmts_have_sweep(&e.stmts))
            }
            _ => false,
        };
        if hit {
            return true;
        }
    }
    false
}

fn expr_has_sweep(e: &Expr) -> bool {
    match e {
        Expr::Call(c) => {
            let here = matches!(
                &*c.name,
                Expr::Dot(inner, m)
                    if matches!(&**inner, Expr::Ident(b) if b == "query3d") && m == "sweep"
            );
            here || c.args.args.iter().any(|a| match a {
                auto_parser_api::ast::Arg::Pos(x) => expr_has_sweep(x),
                _ => false,
            })
        }
        Expr::Unary(_, inner) => expr_has_sweep(inner),
        Expr::Bina(a, _, b) => expr_has_sweep(a) || expr_has_sweep(b),
        _ => false,
    }
}
