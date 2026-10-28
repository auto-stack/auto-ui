//! GLB loading via the `gltf` crate. The frozen squash GLBs are untextured
//! flat-color models (3 indexed primitives each, POSITION/NORMAL/TEXCOORD,
//! material baseColorFactor; see T-01 manifest §2.5 and evidence note E2) —
//! this loader extracts exactly that subset and bakes node transforms into
//! the vertices so model space matches the Godot-imported scene roots.

use glam::{Mat3, Mat4, Vec3, Vec4};
use std::path::{Path, PathBuf};

/// Classified asset failures (mirror the renderer taxonomy; the probe maps
/// them 1:1 into failure-injection reports).
#[derive(Debug)]
pub enum AssetError {
    MissingFile(PathBuf),
    Io(String),
    Parse(String),
    Unsupported(String),
}

impl AssetError {
    pub fn classify(&self) -> &'static str {
        match self {
            AssetError::MissingFile(_) => "missing_file",
            AssetError::Io(_) => "io",
            AssetError::Parse(_) => "bad_model_parse",
            AssetError::Unsupported(_) => "unsupported_extension",
        }
    }
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::MissingFile(p) => write!(f, "missing file {}", p.display()),
            AssetError::Io(m) => write!(f, "io: {m}"),
            AssetError::Parse(m) => write!(f, "glb parse: {m}"),
            AssetError::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for AssetError {}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialDef {
    pub name: String,
    /// linear RGBA base color factor (frozen GLBs: flat colors).
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshData {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub material: MaterialDef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelData {
    /// sha256 of the source bytes — the stable identity for resource binding.
    pub source_sha256: String,
    pub meshes: Vec<MeshData>,
}

pub fn load_glb_file(path: &Path) -> Result<ModelData, AssetError> {
    if !path.exists() {
        return Err(AssetError::MissingFile(path.to_path_buf()));
    }
    let bytes =
        std::fs::read(path).map_err(|e| AssetError::Io(format!("{}: {e}", path.display())))?;
    load_glb_bytes(&bytes)
}

pub fn load_glb_bytes(bytes: &[u8]) -> Result<ModelData, AssetError> {
    let (doc, buffers, _images) =
        gltf::import_slice(bytes).map_err(|e| AssetError::Parse(format!("{e}")))?;

    // The frozen samples are untextured; flag anything else rather than
    // silently dropping the texture requirement (AC-04: no silent omission).
    for material in doc.materials() {
        if let Some(info) = material.pbr_metallic_roughness().base_color_texture() {
            return Err(AssetError::Unsupported(format!(
                "material '{}' uses a base color texture ({}); untextured models only in this slice",
                material.name().unwrap_or("?"),
                info.texture().index()
            )));
        }
    }

    let mut meshes = Vec::new();
    let scene = doc
        .default_scene()
        .ok_or_else(|| AssetError::Parse("no default scene".into()))?;
    for node in scene.nodes() {
        collect_node(&doc, &buffers, &node, Mat4::IDENTITY, &mut meshes)?;
    }

    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);

    Ok(ModelData {
        source_sha256: format!("{:x}", h.finalize()),
        meshes,
    })
}

fn collect_node(
    doc: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    node: &gltf::Node,
    parent: Mat4,
    out: &mut Vec<MeshData>,
) -> Result<(), AssetError> {
    let global = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        let name = mesh.name().unwrap_or("mesh").to_string();
        for (i, primitive) in mesh.primitives().enumerate() {
            let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|d| &d.0[..]));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| AssetError::Parse(format!("{name}#{i}: no POSITION")))?
                .collect();
            let raw_normals: Vec<[f32; 3]> = reader
                .read_normals()
                .ok_or_else(|| AssetError::Parse(format!("{name}#{i}: no NORMAL")))?
                .collect();
            let indices: Vec<u32> = match reader.read_indices() {
                Some(gltf::mesh::util::ReadIndices::U16(it)) => it.map(|v| v as u32).collect(),
                Some(gltf::mesh::util::ReadIndices::U32(it)) => it.collect(),
                Some(gltf::mesh::util::ReadIndices::U8(it)) => it.map(|v| v as u32).collect(),
                None => (0..positions.len() as u32).collect(),
            };

            // Bake the node transform into the vertices (positions w=1,
            // normals through the inverse-transpose rotation part).
            let normal_matrix = Mat3::from_mat4(global).inverse().transpose();
            let positions: Vec<[f32; 3]> = positions
                .iter()
                .map(|p| {
                    let v = global * Vec4::new(p[0], p[1], p[2], 1.0);
                    [v.x, v.y, v.z]
                })
                .collect();
            let normals: Vec<[f32; 3]> = raw_normals
                .iter()
                .map(|n| {
                    let v = normal_matrix * Vec3::new(n[0], n[1], n[2]);
                    let v = v.normalize_or_zero();
                    [v.x, v.y, v.z]
                })
                .collect();

            let material = primitive.material();
            let pbr = material.pbr_metallic_roughness();
            out.push(MeshData {
                name: format!("{name}#{i}"),
                positions,
                normals,
                indices,
                material: MaterialDef {
                    name: material.name().unwrap_or("material").to_string(),
                    base_color: pbr.base_color_factor(),
                    metallic: pbr.metallic_factor(),
                    roughness: pbr.roughness_factor(),
                },
            });
        }
    }
    let _ = doc;
    for child in node.children() {
        collect_node(doc, buffers, &child, global, out)?;
    }
    Ok(())
}
