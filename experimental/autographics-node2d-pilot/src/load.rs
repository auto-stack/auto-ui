//! 模块工程加载与领域校验（合同 §2 段 2-4）。
//!
//! 每个模块经显式研究 dialect 解析；`use` 经生产 `resolve_use_module`
//! 定位；`auto.graphics` 是冻结研究服务命名空间（不解文件）。校验在
//! 执行前给出带文件/行号的拒绝：缺模块、错导出、未知实例/属性、
//! 未声明事件、重复定义。Rust 不实现任何玩法规则。

use auto_lang_core::parser::Parser;
use auto_lang_core::use_scanner::UseStatement;
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
    /// PLAN-022 sprites-2d：image2d.png("file.png")——固定 PNG 资源。
    /// 身份 = 完整原始字节 SHA256 + byte_length + png 格式 + IHDR 尺寸；
    /// 严格准入仍在 driver 资源服务（生成器只读 IHDR 元数据，不解码）。
    Image {
        file: String,
        sha256: String,
        byte_length: usize,
        width: u32,
        height: u32,
    },
    /// PLAN-022 sprites-2d：frame2d.at(<image>, x, y, w, h)——具名帧表项。
    /// 名字即资源符号；rect 是图像内的整数像素矩形（w,h>=1）。
    Frame { image: String, rect: [u32; 4] },
    /// PLAN-024 components-3d：model3d.glb("file.glb")——静态平色 GLB。
    /// 身份 = 完整原始字节 SHA256 + byte_length + glb 格式（与 driver
    /// AssetRef 键一致）；严格几何/域准入仍在 driver 受限 decoder
    /// preflight（生成器只读原始字节哈希，不解析 GLB 结构）。
    Model3D {
        file: String,
        sha256: String,
        byte_length: usize,
    },
    /// PLAN-034 components-3d-presentation：textured3d.glb("file.glb")——
    /// 静态纹理 GLB 资源。身份同 Model3D（原始字节，URI/字节不可重写）；
    /// 闭包元数据（closure id/图像边）由既有公开导入（resource-meta 工具
    /// 走 assets::import 同一准入面）产出、pilot 原样绑定；严格闭包/几何
    /// 准入仍在 driver load_declared/preflight（生成器不解析 GLB 结构，
    /// 不引入第二套解码器）。
    Model3DTextured {
        file: String,
        sha256: String,
        byte_length: usize,
    },
    /// PLAN-034：animated3d.glb("file.glb")——原节点动画 GLB 资源（身份
    /// 同上；clip 具名表保原 ordinal/名称，来自既有公开导入元数据）。
    Model3DAnimated {
        file: String,
        sha256: String,
        byte_length: usize,
    },
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

fn parse_module_text(file: &str, source: &str, base: &Path) -> Result<LoadedModule, ProjectError> {
    parse_module_text_dialect(file, source, base, false)
}

/// PLAN-039：queries-3d 组合 dialect（仅显式查询 profile 放行同 node 双
/// view 解析；默认路径旧拒绝字节不变）。
fn parse_module_text_dialect(
    file: &str,
    source: &str,
    base: &Path,
    allow_dual_view: bool,
) -> Result<LoadedModule, ProjectError> {
    let err = |message: String, offset: usize| ProjectError {
        file: file.to_string(),
        line: line_of(source, offset),
        message,
    };
    let mut parser = Parser::from(source).with_session(CompilerSession {
        scenario: Scenario::UI,
        ..Default::default()
    });
    let parsed = if allow_dual_view {
        parser.parse_domain(&auto_ui_dialect::node2d::Node2DQueriesDialect)
    } else {
        parser.parse_domain(&auto_ui_dialect::node2d::Node2DDialect)
    };
    let code = parsed
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
                    let def = resource_from_store(&s.name.to_string(), &s.expr, base)?
                        .ok_or_else(|| {
                            err(
                                format!(
                                    "unsupported resource initializer for '{}' (expected mesh2d.quad/circle, material2d.color, image2d.png or frame2d.at)",
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

/// `let name = mesh2d.quad(w, h)` / `mesh2d.circle(d)` / `material2d.color(r,g,b,a)`
/// 以及 PLAN-022 sprites-2d：`image2d.png("file.png")` / `frame2d.at(<img>, x, y, w, h)`。
fn resource_from_store(
    name: &str,
    expr: &Expr,
    base: &Path,
) -> Result<Option<ResourceDef>, ProjectError> {
    let call = match expr {
        Expr::Call(c) => c,
        _ => return Ok(None),
    };
    let ctor = match &*call.name {
        Expr::Dot(_, m) => m.to_string(),
        _ => return Ok(None),
    };
    // PLAN-034：具名纹理/动画资源构造器按完整点路径精确匹配
    // （textured3d.glb / animated3d.glb）；旧 model3d.glb 静默面不变——
    // 旧构造器不得偷偷接受纹理/动画档（新档内由元数据交叉核对具名拒绝）。
    let full_ctor = match &*call.name {
        Expr::Dot(outer, m) => {
            let mut segments = vec![m.to_string()];
            let mut cur: &Expr = outer;
            while let Expr::Dot(next, seg) = cur {
                segments.push(seg.to_string());
                cur = next;
            }
            if let Expr::Ident(root) = cur {
                segments.push(root.to_string());
                segments.reverse();
                Some(segments.join("."))
            } else {
                None
            }
        }
        _ => None,
    };
    match full_ctor.as_deref() {
        Some("textured3d.glb") => {
            return resource_from_store_present3d(name, "texture resource", false, &call.args.args, base);
        }
        Some("animated3d.glb") => {
            return resource_from_store_present3d(name, "animation resource", true, &call.args.args, base);
        }
        _ => {}
    }
    // PLAN-022：图像/帧表构造器接受字符串/资源实参，先于数值实参收集。
    // 读不到/坏签名的 PNG 具名拒绝（不静默回退到 generic unsupported）。
    if ctor == "png" {
        return resource_from_store_png(name, &call.args.args, base);
    }
    if ctor == "at" {
        return Ok(resource_from_store_at(name, &call.args.args));
    }
    // PLAN-024：model3d.glb("file.glb")——静态平色 GLB 资源（原始字节
    // SHA 身份；几何准入在 driver preflight，缺文件具名拒绝）。
    if ctor == "glb" {
        return resource_from_store_glb(name, &call.args.args, base);
    }
    Ok(resource_from_store_legacy(name, &ctor, &call.args.args))
}

/// PLAN-024：`model3d.glb("file.glb")` —— 缺文件具名拒绝；非 .glb 后缀
/// 亦具名拒绝（不静默回退）。
fn resource_from_store_glb(
    name: &str,
    args: &[auto_parser_api::ast::Arg],
    base: &Path,
) -> Result<Option<ResourceDef>, ProjectError> {
    let err = |message: String| ProjectError {
        file: "resources".to_string(),
        line: 0,
        message,
    };
    if args.len() != 1 {
        return Ok(None);
    }
    let file = match &args[0] {
        auto_parser_api::ast::Arg::Pos(Expr::Str(s)) => s.to_string(),
        _ => return Ok(None),
    };
    if !file.to_ascii_lowercase().ends_with(".glb") {
        return Err(err(format!(
            "model resource '{name}': '{}' is not a .glb path",
            file
        )));
    }
    let path = base.join(&file);
    let bytes = std::fs::read(&path).map_err(|e| {
        err(format!(
            "model resource '{name}': cannot read '{}' ({e})",
            path.display()
        ))
    })?;
    if bytes.len() < 12 || &bytes[0..4] != b"glTF" {
        return Err(err(format!(
            "model resource '{name}': '{}' is not a glTF-binary container",
            path.display()
        )));
    }
    Ok(Some(ResourceDef {
        name: name.to_string(),
        kind: ResourceKind::Model3D {
            file,
            sha256: crate::scene::sha256_hex(&bytes),
            byte_length: bytes.len(),
        },
        line: 0,
    }))
}

/// PLAN-034：`textured3d.glb("file.glb")` / `animated3d.glb("file.glb")` ——
/// 具名纹理/节点动画 GLB 资源。签名与身份规则同 model3d.glb（单字符串实参、
/// .glb 后缀、glTF magic、原始字节 SHA 身份）；闭包/clip 元数据由既有公开
/// 导入产出（T-02 冻结），生成器不解析 GLB 结构。
fn resource_from_store_present3d(
    name: &str,
    label: &str,
    animated: bool,
    args: &[auto_parser_api::ast::Arg],
    base: &Path,
) -> Result<Option<ResourceDef>, ProjectError> {
    let err = |message: String| ProjectError {
        file: "resources".to_string(),
        line: 0,
        message,
    };
    if args.len() != 1 {
        return Ok(None);
    }
    let file = match &args[0] {
        auto_parser_api::ast::Arg::Pos(Expr::Str(s)) => s.to_string(),
        _ => return Ok(None),
    };
    if !file.to_ascii_lowercase().ends_with(".glb") {
        return Err(err(format!(
            "{label} '{name}': '{}' is not a .glb path",
            file
        )));
    }
    let path = base.join(&file);
    let bytes = std::fs::read(&path).map_err(|e| {
        err(format!(
            "{label} '{name}': cannot read '{}' ({e})",
            path.display()
        ))
    })?;
    if bytes.len() < 12 || &bytes[0..4] != b"glTF" {
        return Err(err(format!(
            "{label} '{name}': '{}' is not a glTF-binary container",
            path.display()
        )));
    }
    Ok(Some(ResourceDef {
        name: name.to_string(),
        kind: if animated {
            ResourceKind::Model3DAnimated {
                file,
                sha256: crate::scene::sha256_hex(&bytes),
                byte_length: bytes.len(),
            }
        } else {
            ResourceKind::Model3DTextured {
                file,
                sha256: crate::scene::sha256_hex(&bytes),
                byte_length: bytes.len(),
            }
        },
        line: 0,
    }))
}

/// 旧三类数值构造器（quad/circle/color）。
fn resource_from_store_legacy(
    name: &str,
    ctor: &str,
    args: &[auto_parser_api::ast::Arg],
) -> Option<ResourceDef> {
    let nums: Vec<f64> = args
        .iter()
        .map(|a| match a {
            auto_parser_api::ast::Arg::Pos(Expr::Float(v, _)) => Some(*v),
            auto_parser_api::ast::Arg::Pos(Expr::Int(v)) => Some(*v as f64),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let kind = match ctor {
        "quad" if nums.len() == 2 => ResourceKind::Quad(nums[0], nums[1]),
        "circle" if nums.len() == 1 => ResourceKind::Circle(nums[0]),
        "color" if nums.len() == 4 => ResourceKind::Color(nums[0], nums[1], nums[2], nums[3]),
        _ => return None,
    };
    Some(ResourceDef {
        name: name.to_string(),
        kind,
        line: 0,
    })
}

/// PLAN-022：`image2d.png("file.png")` —— 缺文件/坏签名具名拒绝。
fn resource_from_store_png(
    name: &str,
    args: &[auto_parser_api::ast::Arg],
    base: &Path,
) -> Result<Option<ResourceDef>, ProjectError> {
    let err = |message: String| ProjectError {
        file: "resources".to_string(),
        line: 0,
        message,
    };
    if args.len() != 1 {
        return Ok(None);
    }
    let file = match &args[0] {
        auto_parser_api::ast::Arg::Pos(Expr::Str(s)) => s.to_string(),
        _ => return Ok(None),
    };
    let path = base.join(&file);
    let bytes = std::fs::read(&path).map_err(|e| {
        err(format!(
            "image resource '{name}': cannot read '{}' ({e})",
            path.display()
        ))
    })?;
    let (width, height) = png_ihdr_dims(&bytes).ok_or_else(|| {
        err(format!(
            "image resource '{name}': '{}' is not a readable PNG (signature/IHDR)",
            path.display()
        ))
    })?;
    Ok(Some(ResourceDef {
        name: name.to_string(),
        kind: ResourceKind::Image {
            file,
            sha256: crate::scene::sha256_hex(&bytes),
            byte_length: bytes.len(),
            width,
            height,
        },
        line: 0,
    }))
}

/// PLAN-022：`frame2d.at(resources.<img>, x, y, w, h)`。
fn resource_from_store_at(name: &str, args: &[auto_parser_api::ast::Arg]) -> Option<ResourceDef> {
    if args.len() != 5 {
        return None;
    }
    let image = match &args[0] {
        auto_parser_api::ast::Arg::Pos(Expr::Dot(outer, m)) => {
            if matches!(&**outer, Expr::Ident(b) if b == "resources") {
                m.to_string()
            } else {
                return None;
            }
        }
        _ => return None,
    };
    let nums: Vec<u32> = args[1..5]
        .iter()
        .map(|a| match a {
            auto_parser_api::ast::Arg::Pos(Expr::Int(v)) if *v >= 0 => Some(*v as u32),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ResourceDef {
        name: name.to_string(),
        kind: ResourceKind::Frame {
            image,
            rect: [nums[0], nums[1], nums[2], nums[3]],
        },
        line: 0,
    })
}

/// PLAN-022：`image2d.png("file.png")` / `frame2d.at(resources.<img>, x, y, w, h)`。
fn resource_from_store_sprites(
    name: &str,
    ctor: &str,
    args: &[auto_parser_api::ast::Arg],
    base: &Path,
) -> Option<ResourceDef> {
    let kind = match ctor {
        // 身份 = 完整字节 SHA256 + byte_length + IHDR 尺寸（严格准入在
        // driver 资源服务；生成器只读 IHDR 元数据）。
        "png" if args.len() == 1 => {
            let file = match &args[0] {
                auto_parser_api::ast::Arg::Pos(Expr::Str(s)) => s.to_string(),
                _ => return None,
            };
            let path = base.join(&file);
            let bytes = std::fs::read(&path).ok()?;
            let (width, height) = png_ihdr_dims(&bytes)?;
            ResourceKind::Image {
                file,
                sha256: crate::scene::sha256_hex(&bytes),
                byte_length: bytes.len(),
                width,
                height,
            }
        }
        // 具名帧表项，rect 整数像素；边界关系在 lower 层按 IHDR 尺寸核。
        "at" if args.len() == 5 => {
            let image = match &args[0] {
                auto_parser_api::ast::Arg::Pos(Expr::Dot(outer, m)) => {
                    if matches!(&**outer, Expr::Ident(b) if b == "resources") {
                        m.to_string()
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };
            let nums: Vec<u32> = args[1..5]
                .iter()
                .map(|a| match a {
                    auto_parser_api::ast::Arg::Pos(Expr::Int(v)) if *v >= 0 => Some(*v as u32),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            ResourceKind::Frame {
                image,
                rect: [nums[0], nums[1], nums[2], nums[3]],
            }
        }
        _ => return None,
    };
    Some(ResourceDef {
        name: name.to_string(),
        kind,
        line: 0,
    })
}

/// PNG IHDR 尺寸（仅元数据读：8 字节签名 + IHDR 块头 + 宽高 BE u32）。
/// 签名/长度不符返回 None（载入层具名拒绝）；完整准入属 driver 资源服务。
fn png_ihdr_dims(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 24 || bytes[..8] != SIG {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

/// 读入模块工程：`entry` 所在目录为根，`use` 闭包全部解析。
pub fn load_project(entry: &Path) -> Result<Project, ProjectError> {
    load_project_dual(entry, false)
}

/// PLAN-039：queries-3d 双 view 解析载入（默认路径字节不变）。
pub fn load_project_dual(entry: &Path, allow_dual_view: bool) -> Result<Project, ProjectError> {
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
        let module = parse_module_text_dialect(&name, &source, &base, allow_dual_view)?;
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
                    queue.push(target);
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
            // PLAN-024：view-3d 实例同计入实例化面（root 判定一致）。
            if let Some(view) = &node.view3d {
                collect_instances3d(&view.elements, &mut instantiated);
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
    // PLAN-028：object3d 模板定义不参与 root 判定（它们只作为 spawn
    // 模板被引用；旧档夹具不携带 object3d，判定面不变）。
    let roots: Vec<String> = declared
        .iter()
        .filter(|d| {
            !instantiated.contains(d)
                && !project
                    .modules
                    .values()
                    .any(|m| m.nodes.iter().any(|n| &n.name == *d && n.object3d.is_some()))
        })
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
                    line: line_of(&module.source, node.span.0),
                    message: format!("duplicate node declaration '{}'", node.name),
                });
            }
        }
    }
    // ---- 闭包后语义校验（合同 §2 段 4：执行前定位拒绝）----
    validate_closure(&project)?;
    Ok(project)
}

/// 轻量实例元素引用（校验用）。
struct InstanceRef<'a> {
    node: &'a str,
    name: &'a str,
    props: &'a [auto_ui_dialect::node2d::Prop],
}

fn walk_view<'a>(
    node_name: &'a str,
    elements: &'a [View2dElement],
    out: &mut Vec<InstanceRef<'a>>,
) {
    for el in elements {
        match el {
            View2dElement::Instance { node, name, props } => {
                out.push(InstanceRef { node, name, props });
            }
            View2dElement::If { then, .. } => walk_view(node_name, then, out),
            _ => {}
        }
    }
}

/// 闭包加载完成后的语义校验（执行前拒绝，带文件/行号）：
/// 错导出、坏 actor 引用、未知实例属性、重复局部绑定。
fn validate_closure(project: &Project) -> Result<(), ProjectError> {
    // 1. 错导出：具名 use 项必须是目标模块的真实导出。
    for module in project.modules.values() {
        for u in &module.uses {
            if u.items.is_empty() || u.module == SERVICES_NAMESPACE {
                continue;
            }
            let Some(target) = project.modules.get(&u.module) else {
                continue; // 缺模块已在闭包期拒绝
            };
            let mut exports: Vec<String> = Vec::new();
            for f in &target.fns {
                exports.push(f.0.clone());
            }
            for n in &target.nodes {
                exports.push(n.name.clone());
            }
            for r in &target.resources {
                exports.push(r.name.clone());
            }
            for e in target.enums.keys() {
                exports.push(e.clone());
            }
            for item in &u.items {
                if !exports.contains(item) {
                    let line = line_of(
                        &module.source,
                        module
                            .source
                            .find(&format!("use {}", u.module))
                            .unwrap_or(0),
                    );
                    return Err(ProjectError {
                        file: module.name.clone(),
                        line,
                        message: format!(
                            "module '{}' does not export '{}' (exports: {})",
                            u.module,
                            item,
                            exports.join(", ")
                        ),
                    });
                }
            }
        }
    }

    // 收集实例元素（全定义视图）与根 view 实例名（refs 引用面）。
    let mut all_instances: Vec<(String, InstanceRef)> = Vec::new();
    let mut root_names: Vec<String> = Vec::new();
    for module in project.modules.values() {
        for node in &module.nodes {
            let mut out: Vec<InstanceRef> = Vec::new();
            if let Some(view) = &node.view2d {
                walk_view(&node.name, &view.elements, &mut out);
            }
            // PLAN-024：view-3d 实例元素同走 prop/refs 校验。
            if let Some(view) = &node.view3d {
                walk_view3d(&view.elements, &mut out);
            }
            for inst in &out {
                if node.name == project.root {
                    root_names.push(inst.name.to_string());
                }
                all_instances.push((
                    module.name.clone(),
                    InstanceRef {
                        node: inst.node,
                        name: inst.name,
                        props: inst.props,
                    },
                ));
            }
        }
    }

    // 2. 未知实例属性；3. 坏 actor 引用（refs.<name>.<field> 必须指向根
    //    view 已声明的实例）。
    for (module_name, inst) in &all_instances {
        let module = &project.modules[module_name];
        let node = node_of(project, inst.node);
        for pr in inst.props {
            let known = node.params.iter().any(|(n, _)| n.as_str() == pr.name);
            if !known {
                return Err(ProjectError {
                    file: module.name.clone(),
                    line: line_of(&module.source, pr.span.0),
                    message: format!(
                        "unknown property '{}' on instance '{}' of node '{}' (params: {})",
                        pr.name,
                        inst.name,
                        inst.node,
                        node.params
                            .iter()
                            .map(|(n, _)| n.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                });
            }
            for e in pr.value.exprs() {
                let mut cursor = e;
                loop {
                    match cursor {
                        Expr::Dot(outer, field) => {
                            if let Expr::Dot(inner, name) = &**outer {
                                if let Expr::Ident(base) = &**inner {
                                    if base.as_str() == "refs"
                                        && !root_names.iter().any(|n| n == name.as_str())
                                    {
                                        return Err(ProjectError {
                                            file: module.name.clone(),
                                            line: line_of(&module.source, pr.span.0),
                                            message: format!(
                                                "refs.{} references unknown root instance '{}' (known: {})",
                                                field, name, root_names.join(", ")
                                            ),
                                        });
                                    }
                                }
                            }
                            cursor = outer;
                        }
                        _ => break,
                    }
                }
            }
        }
    }

    // 4. 重复局部绑定：node 参数与 model 字段、model 字段互间不得重名。
    for module in project.modules.values() {
        for node in &module.nodes {
            let mut seen: Vec<String> = Vec::new();
            for (pname, _) in &node.params {
                if seen.contains(pname) {
                    return Err(ProjectError {
                        file: module.name.clone(),
                        line: line_of(&module.source, node.span.0),
                        message: format!("duplicate parameter '{}' on node '{}'", pname, node.name),
                    });
                }
                seen.push(pname.clone());
            }
            if let Some(model) = &node.model {
                for f in &model.fields {
                    let fname = f.name.to_string();
                    if seen.contains(&fname) {
                        return Err(ProjectError {
                            file: module.name.clone(),
                            line: line_of(&module.source, node.span.0),
                            message: format!(
                                "duplicate binding '{}' on node '{}' (param/model field)",
                                fname, node.name
                            ),
                        });
                    }
                    seen.push(fname);
                }
            }
        }
    }
    Ok(())
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

/// PLAN-024：view-3d 实例名收集（group3d 子树递归）。
fn collect_instances3d(elements: &[auto_ui_dialect::node2d::View3dElement], out: &mut Vec<String>) {
    for el in elements {
        match el {
            auto_ui_dialect::node2d::View3dElement::Instance { node, .. } => {
                out.push(node.clone());
            }
            auto_ui_dialect::node2d::View3dElement::Group { children, .. } => {
                collect_instances3d(children, out);
            }
            _ => {}
        }
    }
}

/// PLAN-024：view-3d 实例元素引用收集（prop/refs 校验面与 view-2d 一致）。
fn walk_view3d<'a>(
    elements: &'a [auto_ui_dialect::node2d::View3dElement],
    out: &mut Vec<InstanceRef<'a>>,
) {
    for el in elements {
        match el {
            auto_ui_dialect::node2d::View3dElement::Instance { node, name, props } => {
                out.push(InstanceRef { node, name, props });
            }
            auto_ui_dialect::node2d::View3dElement::Group { children, .. } => {
                walk_view3d(children, out);
            }
            _ => {}
        }
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
