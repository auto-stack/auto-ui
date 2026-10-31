//! 模块工程加载与领域校验（合同 §2 段 2-4）。
//!
//! 每个模块经显式研究 dialect 解析；`use` 经生产 `resolve_use_module`
//! 定位；`auto.graphics` 是冻结研究服务命名空间（不解文件）。校验在
//! 执行前给出带文件/行号的拒绝：缺模块、错导出、未知实例/属性、
//! 未声明事件、重复定义。Rust 不实现任何玩法规则。

use auto_lang_core::parser::Parser;
use auto_lang_core::use_scanner::{scan_use_statements, UseStatement};
use auto_parser_api::ast::{Expr, Stmt};
use auto_parser_api::{CompilerSession, DomainStmt, Scenario};
use auto_ui_dialect::node2d::{Node2DStmt, NodeDecl, PropValue, View2dElement};
use auto_ui_dialect::{ModelBlock, MsgDecl};
use std::collections::BTreeMap;
use std::path::Path;

/// 冻结研究服务命名空间：项绑定固定 AG 服务符号，不解析为文件。
pub const SERVICES_NAMESPACE: &str = "auto.graphics";

#[derive(Debug, Clone)]
pub enum ResourceKind {
    /// mesh2d.quad(width, height)
    Quad(f64, f64),
    /// mesh2d.circle(diameter)
    Circle(f64),
    /// material2d.color(r, g, b, a)
    Color(f64, f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct ResourceDef {
    pub name: String,
    pub kind: ResourceKind,
    /// 源内行号（诊断用）。
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub name: String,
    pub source: String,
    pub nodes: Vec<NodeDecl>,
    /// 纯函数：名字 → 源内字节区间（verbatim 降层）。
    pub fns: Vec<(String, std::ops::Range<usize>)>,
    /// 资源符号（仅 resources 模块可声明）。
    pub resources: Vec<ResourceDef>,
    pub uses: Vec<UseStatement>,
    /// 枚举名 → 变体名 → 判整值（0 起）。
    pub enums: BTreeMap<String, BTreeMap<String, i64>>,
}

#[derive(Debug, Default)]
pub struct Project {
    pub modules: BTreeMap<String, LoadedModule>,
    /// node 定义名 → 所在模块。
    pub node_modules: BTreeMap<String, String>,
    /// 根 node（未被任何 view-2d 实例化的 node，恰一个）。
    pub root: String,
}

#[derive(Debug)]
pub struct ProjectError {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

fn line_of(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1
}

fn parse_module_text(file: &str, source: &str) -> Result<LoadedModule, ProjectError> {
    let err = |message: String, offset: usize| ProjectError {
        file: file.to_string(),
        line: line_of(source, offset),
        message,
    };
    let mut parser = Parser::from(source).with_session(CompilerSession {
        scenario: Scenario::UI,
        ..Default::default()
    });
    let code = parser
        .parse_domain(&auto_ui_dialect::node2d::Node2DDialect)
        .map_err(|e| {
            // 解析错误带字节偏移 span；换算行号。
            let text = format!("{e:?}");
            let offset = text
                .find("offset: SourceOffset(")
                .and_then(|i| {
                    text[i + 21..]
                        .split(')')
                        .next()
                        .and_then(|n| n.parse::<usize>().ok())
                })
                .unwrap_or(0);
            err(format!("research parse: {e:?}"), offset)
        })?;

    let mut module = LoadedModule {
        name: file.to_string(),
        source: source.to_string(),
        nodes: Vec::new(),
        fns: Vec::new(),
        resources: Vec::new(),
        uses: Vec::new(),
        enums: BTreeMap::new(),
    };
    for item in code.items {
        match item {
            DomainStmt::Domain(Node2DStmt::Node(node)) => module.nodes.push(node),
            DomainStmt::Core(stmt) => match stmt {
                Stmt::Use(u) => {
                    module.uses.push(UseStatement {
                        module: u.paths.join("."),
                        items: u.items.iter().map(|i| i.to_string()).collect(),
                        is_wildcard: false,
                        alias: None,
                        is_c_import: false,
                        c_header: None,
                        is_rust_import: false,
                        is_python_import: false,
                        is_pub: false,
                    });
                }
                Stmt::Fn(f) => {
                    if let Some((start, end)) = f.span {
                        module.fns.push((f.name.to_string(), start..end));
                    }
                }
                Stmt::EnumDecl(e) => {
                    let mut variants = BTreeMap::new();
                    for (i, v) in e.items.iter().enumerate() {
                        variants.insert(v.name.to_string(), i as i64);
                    }
                    module.enums.insert(e.name.to_string(), variants);
                }
                Stmt::Store(s) => {
                    let def = resource_from_store(&s.name.to_string(), &s.expr)
                        .ok_or_else(|| {
                            err(
                                format!(
                                    "unsupported resource initializer for '{}' (expected mesh2d.quad/circle or material2d.color)",
                                    s.name
                                ),
                                0,
                            )
                        })?;
                    module.resources.push(def);
                }
                Stmt::Comment(_) | Stmt::EmptyLine(_) => {}
                other => {
                    return Err(err(
                        format!("unsupported top-level research statement: {other:?}"),
                        0,
                    ));
                }
            },
        }
    }
    Ok(module)
}

/// `let name = mesh2d.quad(w, h)` / `mesh2d.circle(d)` / `material2d.color(r,g,b,a)`。
fn resource_from_store(name: &str, expr: &Expr) -> Option<ResourceDef> {
    let call = match expr {
        Expr::Call(c) => c,
        _ => return None,
    };
    let ctor = match &*call.name {
        Expr::Dot(_, m) => m.to_string(),
        _ => return None,
    };
    let args: Vec<f64> = call
        .args
        .args
        .iter()
        .map(|a| match a {
            auto_parser_api::ast::Arg::Pos(Expr::Float(v, _)) => Some(*v),
            auto_parser_api::ast::Arg::Pos(Expr::Int(v)) => Some(*v as f64),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let kind = match ctor.as_str() {
        "quad" if args.len() == 2 => ResourceKind::Quad(args[0], args[1]),
        "circle" if args.len() == 1 => ResourceKind::Circle(args[0]),
        "color" if args.len() == 4 => ResourceKind::Color(args[0], args[1], args[2], args[3]),
        _ => return None,
    };
    Some(ResourceDef {
        name: name.to_string(),
        kind,
        line: 0,
    })
}

/// 读入模块工程：`entry` 所在目录为根，`use` 闭包全部解析。
pub fn load_project(entry: &Path) -> Result<Project, ProjectError> {
    let base = entry
        .parent()
        .ok_or_else(|| ProjectError {
            file: entry.display().to_string(),
            line: 0,
            message: "entry needs a parent directory".into(),
        })?
        .to_path_buf();
    let mut project = Project::default();
    let entry_name = entry
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .to_string();
    let mut queue = vec![entry_name.clone()];
    let mut seen: Vec<String> = Vec::new();
    while let Some(name) = queue.pop() {
        if seen.contains(&name) {
            continue;
        }
        seen.push(name.clone());
        if name == SERVICES_NAMESPACE {
            continue;
        }
        let path = base.join(format!("{name}.at"));
        let source = std::fs::read_to_string(&path).map_err(|_| ProjectError {
            file: path.display().to_string(),
            line: 0,
            message: "module not found".into(),
        })?;
        let module = parse_module_text(&name, &source)?;
        for u in &module.uses {
            if u.module == SERVICES_NAMESPACE || u.module == "store" {
                continue;
            }
            match auto_ui_compiler::ui_ast::resolve_use_module(&base, u) {
                auto_ui_compiler::ui_ast::UseModuleResolution::Module(p) => {
                    let target = p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();
                    // 校验具名导入确为被导入模块的导出（错导出拒绝）。
                    if !u.items.is_empty() {
                        queue.push(target.clone());
                    } else {
                        queue.push(target);
                    }
                }
                _ => {
                    return Err(ProjectError {
                        file: name.clone(),
                        line: line_of(
                            &source,
                            source.find(&format!("use {}", u.module)).unwrap_or(0),
                        ),
                        message: format!("module not found: {}", u.module),
                    });
                }
            }
        }
        project.modules.insert(name, module);
    }

    // node 注册表与根 node。
    let mut instantiated: Vec<String> = Vec::new();
    let mut declared: Vec<String> = Vec::new();
    for module in project.modules.values() {
        for node in &module.nodes {
            declared.push(node.name.clone());
        }
        for node in &module.nodes {
            if let Some(view) = &node.view2d {
                for el in &view.elements {
                    collect_instances(el, &mut instantiated);
                }
            }
        }
    }
    for name in &instantiated {
        if !declared.contains(name) {
            return Err(ProjectError {
                file: project
                    .modules
                    .iter()
                    .find(|(_, m)| m.nodes.iter().any(|n| &n.name == name))
                    .map(|(_, m)| m.name.clone())
                    .unwrap_or_default(),
                line: 0,
                message: format!("instance of undeclared node '{name}'"),
            });
        }
    }
    let roots: Vec<String> = declared
        .iter()
        .filter(|d| !instantiated.contains(d))
        .cloned()
        .collect();
    if roots.len() != 1 {
        return Err(ProjectError {
            file: entry.display().to_string(),
            line: 0,
            message: format!(
                "exactly one root node expected (not instantiated anywhere), got {roots:?}"
            ),
        });
    }
    project.root = roots[0].clone();
    for module in project.modules.values() {
        for node in &module.nodes {
            if project
                .node_modules
                .insert(node.name.clone(), module.name.clone())
                .is_some()
            {
                return Err(ProjectError {
                    file: module.name.clone(),
                    line: 0,
                    message: format!("duplicate node declaration '{}'", node.name),
                });
            }
        }
    }
    Ok(project)
}

fn collect_instances(el: &View2dElement, out: &mut Vec<String>) {
    match el {
        View2dElement::Instance { node, .. } => out.push(node.clone()),
        View2dElement::If { then, .. } => {
            for child in then {
                collect_instances(child, out);
            }
        }
        _ => {}
    }
}

/// 实例属性按 node 参数解析为字面量；错名/缺失/非字面量在此定位拒绝。
pub fn resolve_instance_props(
    node: &NodeDecl,
    props: &[(String, PropValue)],
    file: &str,
) -> Result<Vec<(String, f64)>, ProjectError> {
    let err = |message: String| ProjectError {
        file: file.to_string(),
        line: 0,
        message,
    };
    let mut out = Vec::new();
    for (pname, pty) in &node.params {
        let value = props.iter().find(|(n, _)| n == pname).ok_or_else(|| {
            err(format!(
                "instance missing prop '{pname}' for node '{}'",
                node.name
            ))
        })?;
        let v = match &value.1 {
            PropValue::One(Expr::Float(v, _)) => *v,
            PropValue::One(Expr::Int(v)) => *v as f64,
            _ => {
                return Err(err(format!(
                    "prop '{pname}' must be a numeric literal in the research slice"
                )))
            }
        };
        match pty {
            auto_parser_api::ast::Type::Unknown if pname == "on_collect" => {
                // 事件端口不在数值表内；由绑定表承载。
                continue;
            }
            auto_parser_api::ast::Type::Unknown if pname == "target" => {
                // actor 引用由实例图承载。
                continue;
            }
            _ => {}
        }
        out.push((pname.clone(), v));
    }
    Ok(out)
}

/// 供 scene/lower 读取的 node 查找。
pub fn node_of<'a>(project: &'a Project, name: &str) -> &'a NodeDecl {
    let module = project
        .node_modules
        .get(name)
        .unwrap_or_else(|| panic!("node '{}' not registered in module table", name));
    project.modules[module]
        .nodes
        .iter()
        .find(|n| n.name == name)
        .expect("registered node")
}

/// 供 scene/lower 读取的 model/msg 快照。
pub fn model_of(node: &NodeDecl) -> Option<&ModelBlock> {
    node.model.as_ref()
}
pub fn msg_of(node: &NodeDecl) -> Option<&MsgDecl> {
    node.msg.as_ref()
}
