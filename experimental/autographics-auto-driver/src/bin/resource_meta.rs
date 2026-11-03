//! PLAN-034 resource metadata exporter（候选A"既有公开导入输出受检闭包元数据"
//! 的公共入口）。对 `resources-request.json` 里每条具名纹理/动画 GLB 声明走
//! 既有公开 `assets::import` / `assets::import_animated`（同一独立准入：
//! preflight/闭包重算/图像边/预算），把受检 `AssetRef`（closure id、图像边、
//! 原始 clip 名与序数、模型局部 bounds）原样写给 pilot 绑定。本工具只产
//! 元数据：不解码、不复制字节、不建 Host；严格几何/闭包准入仍由 Host 载入
//! 时 `load_declared` 复验（生成器签名不豁免它）。
//!
//! 用法：
//! `resource-meta --entry <dir> --request <resources-request.json> --out <resources.json>`
use autographics_auto_driver::authoring::assets::{self, Registry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    resources: Vec<RequestEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestEntry {
    name: String,
    #[serde(default)]
    module: String,
    #[serde(default)]
    kind: String,
    file: String,
    sha256: String,
    byte_length: usize,
    animated: bool,
}

#[derive(Serialize)]
struct Meta {
    schema: &'static str,
    entries: Vec<MetaEntry>,
}

#[derive(Serialize)]
struct MetaEntry {
    name: String,
    kind: String,
    file: String,
    animated: bool,
    /// 既有公开导入产出的受检身份，pilot 原样嵌入 scene.agscene assets。
    identity: assets::AssetRef,
    /// 原始文件 clip 表（ordinal=文件原序）。匿名选择不提供；具名必须精确
    /// 命中一条，重名歧义由 pilot 具名拒绝。
    clips: Vec<ClipMeta>,
    /// 外部图像边（uri:）与相对声明 GLB 目录的源路径（pilot 复制用）。
    images: Vec<ImagePath>,
    /// 模型局部 bounds（全部 mesh 顶点 f32 min/max，与 driver spatial
    /// local_bounds 同源数据）。仅供参考；world AABB 门仍在 driver bind。
    bounds_min: [f64; 3],
    bounds_max: [f64; 3],
}

#[derive(Serialize)]
struct ClipMeta {
    ordinal: usize,
    name: String,
}

#[derive(Serialize)]
struct ImagePath {
    source: String,
    sha256: String,
    byte_length: usize,
    width: u32,
    height: u32,
    /// 相对 --entry 的原始图像文件路径（正斜杠）。
    path: String,
}

fn fail(message: String) -> ! {
    eprintln!("resource-meta: {message}");
    std::process::exit(2);
}

fn arg(name: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == name) {
        Some(i) => args.get(i + 1).cloned().unwrap_or_else(|| fail(format!("{name} requires a value"))),
        None => fail(format!("usage: resource-meta --entry <dir> --request <request.json> --out <resources.json> (missing {name})")),
    }
}

fn rel_path(entry: &Path, path: &Path) -> String {
    path.strip_prefix(entry)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn bounds_of(model: &agp::assets::ModelData) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for m in &model.meshes {
        for p in &m.positions {
            for axis in 0..3 {
                let v = f64::from(p[axis]);
                lo[axis] = lo[axis].min(v);
                hi[axis] = hi[axis].max(v);
            }
        }
    }
    (lo, hi)
}

fn main() {
    let entry = PathBuf::from(&arg("--entry"));
    let request_path = PathBuf::from(&arg("--request"));
    let out_path = PathBuf::from(&arg("--out"));
    if entry.as_os_str().is_empty() {
        fail("--entry required".into());
    }
    let request_text = std::fs::read_to_string(&request_path)
        .unwrap_or_else(|e| fail(format!("cannot read {}: {e}", request_path.display())));
    let request: Request = serde_json::from_str(&request_text)
        .unwrap_or_else(|e| fail(format!("bad request {}: {e}", request_path.display())));
    if request.schema != "autographics.resources-request/1" {
        fail(format!("unsupported request schema {}", request.schema));
    }
    let mut seen = BTreeMap::new();
    let mut entries = Vec::new();
    for r in &request.resources {
        if seen.insert(r.name.clone(), ()).is_some() {
            fail(format!("duplicate resource name '{}'", r.name));
        }
        let path = entry.join(&r.file);
        // 既有公开导入：同一独立准入面（preflight/闭包/图像边/预算）。
        let loaded = if r.animated {
            assets::import_animated(&path, &Registry::new())
        } else {
            assets::import(&path, &Registry::new())
        }
        .unwrap_or_else(|e| fail(format!("resource '{}' ({}): {e}", r.name, path.display())));
        if loaded.identity.sha256 != r.sha256 || loaded.identity.byte_length != r.byte_length {
            fail(format!(
                "resource '{}': declared identity {}/{}B but import saw {}/{}B",
                r.name, r.sha256, r.byte_length, loaded.identity.sha256, loaded.identity.byte_length
            ));
        }
        let node_animation = loaded
            .identity
            .closure
            .as_ref()
            .map(|c| c.node_animation)
            .unwrap_or(false);
        if node_animation != r.animated {
            fail(format!(
                "resource '{}': declared animated={} but closure says node_animation={}",
                r.name, r.animated, node_animation
            ));
        }
        let clips = loaded
            .animation
            .as_ref()
            .map(|a| {
                a.clips
                    .iter()
                    .enumerate()
                    .map(|(ordinal, c)| ClipMeta {
                        ordinal,
                        name: c.name.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut images = Vec::new();
        if let Some(closure) = &loaded.identity.closure {
            for image in &closure.images {
                let Some(rel) = image.source.strip_prefix("uri:") else {
                    continue; // 嵌入图像无外部文件，pilot 无需复制。
                };
                let glb_dir = Path::new(&r.file).parent().unwrap_or(Path::new("."));
                let source = entry.join(glb_dir).join(rel);
                if !source.is_file() {
                    fail(format!(
                        "resource '{}': external image {} missing at {}",
                        r.name,
                        image.source,
                        source.display()
                    ));
                }
                images.push(ImagePath {
                    source: image.source.clone(),
                    sha256: image.sha256.clone(),
                    byte_length: image.byte_length,
                    width: image.width,
                    height: image.height,
                    path: rel_path(&entry, &source),
                });
            }
        }
        let (bounds_min, bounds_max) = bounds_of(&loaded.model);
        entries.push(MetaEntry {
            name: r.name.clone(),
            kind: r.kind.clone(),
            file: r.file.clone(),
            animated: r.animated,
            identity: loaded.identity.clone(),
            clips,
            images,
            bounds_min,
            bounds_max,
        });
    }
    let meta = Meta {
        schema: "autographics.resources-meta/1",
        entries,
    };
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| fail(format!("cannot create {}: {e}", parent.display())));
    }
    let text = serde_json::to_string_pretty(&meta)
        .unwrap_or_else(|e| fail(format!("serialize meta: {e}")));
    std::fs::write(&out_path, &text)
        .unwrap_or_else(|e| fail(format!("cannot write {}: {e}", out_path.display())));
    println!(
        "resource-meta: {} entries -> {}",
        meta.entries.len(),
        out_path.display()
    );
}
