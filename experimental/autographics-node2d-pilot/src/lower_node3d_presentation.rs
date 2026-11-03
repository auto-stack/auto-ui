//! PLAN-034 components-3d-presentation：具名纹理/动画资源与呈现接线。
//!
//! 候选A（T-02 冻结）：闭包元数据由既有公开导入产出——pilot 先写
//! `resources-request.json`（声明面），driver `resource-meta` 工具对每条
//! 声明走同一 `assets::import`/`import_animated` 独立准入面，输出
//! `resources.json`；本模块读取后者并把受检 `AssetRef`（closure id/图像
//! 边/原 clip 序数）原样绑定进 scene.agscene。生成器不解析 GLB 结构、
//! 不引入第二套解码器；几何/闭包准入最终仍由 driver 原预检/`load_declared`
//! 独立复验。作者面只有具名符号（资源名/clip 名/元素名），无数字
//! scene/slot/pair/clip 索引。

use crate::load::{Project, ResourceKind};
use crate::translate::LowerError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// pass B 消费的受检元数据（resources.json，`resource-meta` 产出）。
#[derive(Debug, Clone)]
pub struct PresentationMeta3 {
    /// 资源名 → 受检条目（与声明面精确对账）。
    pub resources: BTreeMap<String, ResourceMeta3>,
}

#[derive(Debug, Clone)]
pub struct ResourceMeta3 {
    pub kind: String,
    pub sha256: String,
    pub byte_length: usize,
    pub animated: bool,
    /// 受检 AssetRef 的 closure 子对象（嵌入 scene.agscene assets 行）。
    pub closure: Option<ClosureJson>,
    /// 原始文件 clip 表（ordinal = 文件原序，名称逐字）。
    pub clips: Vec<(usize, String)>,
    /// 外部图像边（相对 --entry 的源路径；pass B 复制进 out/assets/）。
    pub images: Vec<ImageCopy3>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageCopy3 {
    pub source: String,
    pub sha256: String,
    /// 相对 entry 的原始文件路径（正斜杠）。
    pub path: String,
}

/// scene.agscene assets 行的 closure 子对象（受检原文的确定性重排）。
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ClosureJson {
    #[serde(default)]
    pub node_animation: bool,
    pub id: String,
    pub images: Vec<ClosureImageJson>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ClosureImageJson {
    pub source: String,
    pub sha256: String,
    pub byte_length: usize,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetaDoc {
    schema: String,
    entries: Vec<MetaEntryDoc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetaEntryDoc {
    name: String,
    kind: String,
    file: String,
    animated: bool,
    identity: IdentityDoc,
    #[serde(default)]
    clips: Vec<ClipDoc>,
    #[serde(default)]
    images: Vec<ImageDoc>,
    #[serde(default)]
    bounds_min: [f64; 3],
    #[serde(default)]
    bounds_max: [f64; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityDoc {
    #[serde(default)]
    closure: Option<ClosureJson>,
    format: String,
    sha256: String,
    byte_length: usize,
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClipDoc {
    ordinal: usize,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImageDoc {
    source: String,
    sha256: String,
    #[serde(default)]
    byte_length: usize,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    path: String,
}

fn at(module: &str, line: usize, symbol: &str, message: impl Into<String>) -> LowerError {
    crate::lower_components::at(module, line, symbol, &message.into())
}

/// pass A（声明面）：收集全部具名纹理/动画资源声明，写
/// resources-request.json。身份 = 声明期已核的原始字节 SHA/长度。
pub fn resources_request_json(project: &Project) -> String {
    let mut rows = String::new();
    let mut names: Vec<_> = project.modules.keys().collect();
    names.sort();
    for name in &names {
        for r in &project.modules[*name].resources {
            // 平色声明也进请求：既有公开导入揭示其真实内容面，pass B 对
            // "旧构造器携纹理/动画文件"具名拒绝（不静默升档）。
            let (file, sha256, byte_length, animated, kind) = match &r.kind {
                ResourceKind::Model3D {
                    file,
                    sha256,
                    byte_length,
                } => (file, sha256, byte_length, false, "model3d"),
                ResourceKind::Model3DTextured {
                    file,
                    sha256,
                    byte_length,
                } => (file, sha256, byte_length, false, "textured3d"),
                ResourceKind::Model3DAnimated {
                    file,
                    sha256,
                    byte_length,
                } => (file, sha256, byte_length, true, "animated3d"),
                _ => continue,
            };
            rows.push_str(&format!(
                "   {{\"name\": \"{}\", \"module\": \"{}\", \"kind\": \"{}\", \"file\": \"{}\", \"sha256\": \"{}\", \"byte_length\": {}, \"animated\": {}}},
",
                r.name, name, kind, file, sha256, byte_length, animated
            ));
        }
    }
    let rows = rows.trim_end_matches(",\n").to_string();
    format!(
        "{{\n \"schema\": \"autographics.resources-request/1\",\n \"resources\": [\n{} ]\n}}\n",
        rows
    )
}

/// 项目是否存在动画资源（behavior_api 选择：动画档 → node_animation3d/1）。
pub fn has_animated_resource(project: &Project) -> bool {
    project.modules.values().any(|m| {
        m.resources
            .iter()
            .any(|r| matches!(r.kind, ResourceKind::Model3DAnimated { .. }))
    })
}

/// PLAN-034 降低入口：bindings 载体 + 受检元数据；返回（产品，装配身份）。
/// 动画资源在案 → autographics.node_animation3d/1（ag_visual_pose 等受信
/// 面），否则沿 bindings 冻结身份 autographics.triggers3d/1。
pub fn lower_project_components3d_presentation(
    project: &Project,
    meta: &PresentationMeta3,
) -> Result<(crate::lower_basic2d::LowerOutputBasic2D, String), LowerError> {
    let api = if has_animated_resource(project) {
        crate::lower_components3d_bindings::BINDINGS_API_FOR_ANIMATION
    } else {
        crate::lower_components3d_bindings::BINDINGS_API
    };
    let lowered = crate::lower_components3d_bindings::lower_project_components3d_presentation_entry(
        project,
        Some(meta),
    )?;
    Ok((lowered, api.to_string()))
}

/// pass B：读取受检元数据并按声明面对账（每条声明恰一覆盖；身份一致）。
pub fn load_resources_meta(
    path: &Path,
    project: &Project,
) -> Result<PresentationMeta3, LowerError> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        at(
            "resources",
            0,
            "resources.json",
            format!("cannot read {}: {e}", path.display()),
        )
    })?;
    let doc: MetaDoc = serde_json::from_str(&text).map_err(|e| {
        at(
            "resources",
            0,
            "resources.json",
            format!("bad metadata {}: {e}", path.display()),
        )
    })?;
    if doc.schema != "autographics.resources-meta/1" {
        return Err(at(
            "resources",
            0,
            "resources.json",
            format!("unsupported metadata schema {}", doc.schema),
        ));
    }
    let mut meta = PresentationMeta3 {
        resources: BTreeMap::new(),
    };
    for entry in doc.entries {
        if meta.resources.contains_key(&entry.name) {
            return Err(at(
                "resources",
                0,
                &entry.name,
                format!("duplicate metadata entry for resource '{}'", entry.name),
            ));
        }
        // 声明面交叉核对：恰一覆盖 + 身份一致（防陈旧/错文件元数据）。
        let declared = project
            .modules
            .values()
            .flat_map(|m| &m.resources)
            .find(|r| r.name == entry.name);
        let (d_sha, d_len, d_animated, d_kind) = match declared.map(|r| &r.kind) {
            Some(ResourceKind::Model3D {
                sha256,
                byte_length,
                ..
            }) => (sha256, *byte_length, false, "model3d"),
            Some(ResourceKind::Model3DTextured {
                sha256,
                byte_length,
                ..
            }) => (sha256, *byte_length, false, "textured3d"),
            Some(ResourceKind::Model3DAnimated {
                sha256,
                byte_length,
                ..
            }) => (sha256, *byte_length, true, "animated3d"),
            _ => {
                return Err(at(
                    "resources",
                    0,
                    &entry.name,
                    format!(
                        "metadata entry '{}' has no matching model3d/textured3d/animated3d declaration",
                        entry.name
                    ),
                ))
            }
        };
        if entry.kind != d_kind {
            return Err(at(
                "resources",
                0,
                &entry.name,
                format!(
                    "metadata kind '{}' does not match declared constructor kind '{}'",
                    entry.kind, d_kind
                ),
            ));
        }
        // 旧构造器不静默接受纹理/动画：平色声明的文件若被公共导入揭示为
        // 携图像/动画内容，pass B 具名拒绝。
        if d_kind == "model3d" && (entry.identity.closure.is_some() || !entry.clips.is_empty()) {
            return Err(at(
                "resources",
                0,
                &entry.name,
                "plain model3d.glb declaration carries textured/animated content (the old constructor must not silently accept it; declare textured3d.glb/animated3d.glb)".to_string(),
            ));
        }
        if &entry.identity.sha256 != d_sha || entry.identity.byte_length != d_len {
            return Err(at(
                "resources",
                0,
                &entry.name,
                format!(
                    "metadata identity {}/{}B does not match declaration {}/{}B (stale resources.json: rerun resource-meta)",
                    entry.identity.sha256, entry.identity.byte_length, d_sha, d_len
                ),
            ));
        }
        if entry.animated != d_animated {
            return Err(at(
                "resources",
                0,
                &entry.name,
                format!(
                    "metadata animated={} does not match declaration animated={}",
                    entry.animated, d_animated
                ),
            ));
        }
        if entry.identity.format != "glb" {
            return Err(at(
                "resources",
                0,
                &entry.name,
                format!("metadata identity format '{}' is not glb", entry.identity.format),
            ));
        }
        let animated = entry.animated;
        let clips: Vec<(usize, String)> = entry
            .clips
            .iter()
            .map(|c| (c.ordinal, c.name.clone()))
            .collect();
        if animated && clips.is_empty() {
            return Err(at(
                "resources",
                0,
                &entry.name,
                "animation resource metadata carries no clips".to_string(),
            ));
        }
        if !animated && !clips.is_empty() {
            return Err(at(
                "resources",
                0,
                &entry.name,
                "static texture resource metadata unexpectedly carries clips".to_string(),
            ));
        }
        let images = entry
            .images
            .iter()
            .map(|i| ImageCopy3 {
                source: i.source.clone(),
                sha256: i.sha256.clone(),
                path: i.path.clone(),
            })
            .collect();
        meta.resources.insert(
            entry.name.clone(),
            ResourceMeta3 {
                kind: entry.kind.clone(),
                sha256: entry.identity.sha256.clone(),
                byte_length: entry.identity.byte_length,
                animated,
                closure: entry.identity.closure.clone(),
                clips,
                images,
            },
        );
    }
    Ok(meta)
}

/// clip 具名解析：`clips.<resource>.<name|base>` → 原 ordinal（base=-1）。
/// 未知资源/未知名/重名歧义/跨模型错配全部具名拒绝。
pub(crate) fn resolve_clip_ref(
    meta: &PresentationMeta3,
    elem: &str,
    resource: &str,
    path: &[String],
    module: &str,
    line: usize,
) -> Result<i64, LowerError> {
    let bad = |message: String| at(module, line, elem, message);
    let [res, name] = path else {
        return Err(bad(
            "pose clip must be clips.<resource>.<name|base>".to_string(),
        ));
    };
    if res != resource {
        return Err(bad(format!(
            "pose clip 'clips.{res}.{name}' does not belong to element '{elem}' resource '{resource}'"
        )));
    }
    let Some(entry) = meta.resources.get(resource) else {
        return Err(bad(format!(
            "pose clip references unknown resource '{resource}'"
        )));
    };
    if name == "base" {
        return Ok(-1);
    }
    let mut hits = entry.clips.iter().filter(|(_, n)| n == name);
    let Some((ordinal, _)) = hits.next() else {
        return Err(bad(format!(
            "resource '{resource}' has no clip named '{name}'"
        )));
    };
    if hits.next().is_some() {
        return Err(bad(format!(
            "resource '{resource}' has duplicate clip names; '{name}' is ambiguous (anonymous selection is not part of this profile)"
        )));
    }
    i64::try_from(*ordinal).map_err(|_| bad("clip ordinal exceeds i64".to_string()))
}

/// scene.agscene assets 行：具名资源 → 受检 AssetRef 确定性重排（closure
/// 面原样；嵌入图像仅存在于 closure.images，外部 PNG 由 pass B 复制）。
pub(crate) fn asset_ref_row(sha256: &str, byte_length: usize, name: &str, meta: Option<&ResourceMeta3>) -> String {
    let Some(meta) = meta else {
        // 平色 GLB：024 冻结行面（字节不变）。
        return format!(
            "  {{\n   \"format\": \"glb\",\n   \"sha256\": \"{sha256}\",\n   \"byte_length\": {byte_length},\n   \"name\": \"{name}\"\n  }}"
        );
    };
    let closure = match &meta.closure {
        Some(c) => {
            let mut images = String::new();
            for image in &c.images {
                images.push_str(&format!(
                    "     {{\n      \"source\": \"{}\",\n      \"sha256\": \"{}\",\n      \"byte_length\": {},\n      \"width\": {},\n      \"height\": {}\n     }},\n",
                    image.source, image.sha256, image.byte_length, image.width, image.height
                ));
            }
            let images = images.trim_end_matches(",\n").to_string();
            let anim = if c.node_animation { "true" } else { "false" };
            format!(
                "   \"closure\": {{\n    \"node_animation\": {anim},\n    \"id\": \"{}\",\n    \"images\": [\n{}\n    ]\n   }},\n",
                c.id, images
            )
        }
        None => String::new(),
    };
    format!(
        "  {{\n{closure}   \"format\": \"glb\",\n   \"sha256\": \"{sha256}\",\n   \"byte_length\": {byte_length},\n   \"name\": \"{name}\"\n  }}"
    )
}
