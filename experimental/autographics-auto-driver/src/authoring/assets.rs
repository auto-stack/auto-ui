//! Immutable GLB and static-PNG payloads. No decoder or GPU sees input before
//! no-IO admission. PLAN-004 adds the finite PNG domain beside the unchanged
//! model domain: identity is the complete original SHA/length/format, the
//! resource name never identifies a payload, and decoded pixels join the same
//! 8-identity / 32MiB raw / 64MiB CPU-and-GPU budget as model geometry.
use agp::assets::{load_glb_bytes, ImageData, ModelData};
use glam::{Mat3, Mat4, Vec3};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::Path,
    sync::Arc,
};
pub const MAX_RAW: usize = 8 * 1024 * 1024;
pub const MAX_JSON: usize = 1024 * 1024;
pub const MAX_ASSETS: usize = 8;
pub const MAX_RAW_TOTAL: usize = 32 * 1024 * 1024;
pub const MAX_GEOMETRY: usize = 64 * 1024 * 1024;
/// Largest accepted PNG side and decoded RGBA payload (PLAN-004 5.1).
pub const MAX_IMAGE_SIDE: u32 = 2048;
pub const MAX_IMAGE_DECODED: usize = 16 * 1024 * 1024;
pub const FORMAT_GLB: &str = "glb";
pub const FORMAT_PNG: &str = "png";
static DECODE_ATTEMPTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub fn decode_attempts() -> u64 {
    DECODE_ATTEMPTS.load(std::sync::atomic::Ordering::Relaxed)
}
pub(super) fn record_decode() {
    DECODE_ATTEMPTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure: Option<super::textured_models::Closure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<super::public_audio::Metadata>,
    pub format: String,
    pub sha256: String,
    pub byte_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// PNG-only decoded dimensions: part of the persisted identity because the
    /// frame domain of a bound visual is defined against them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}
impl AssetRef {
    /// Immutable payload binding. A document may display another name while
    /// reusing the same accepted Arc; names still participate in document Eq.
    pub fn same_content(&self, other: &Self) -> bool {
        self.sha256 == other.sha256
            && self.byte_length == other.byte_length
            && self.format == other.format
            && self.width == other.width
            && self.height == other.height
            && self.closure == other.closure
            && self.audio == other.audio
    }
    pub fn key(&self) -> String {
        self.closure.as_ref().map_or_else(
            || format!("sha256:{}", self.sha256),
            |c| format!("closure:{}", c.id),
        )
    }
    pub fn is_audio(&self) -> bool {
        matches!(self.format.as_str(), "ogg" | "wav")
    }
    pub fn is_image(&self) -> bool {
        self.format == FORMAT_PNG
    }
    pub fn validate(&self) -> Result<(), String> {
        super::textured_models::validate_ref(self)?;
        if (!matches!(self.format.as_str(), "glb" | "png" | "ogg" | "wav"))
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("invalid: asset identity/format".into());
        }
        if self.byte_length == 0
            || self.byte_length > MAX_RAW
            || self.name.as_ref().is_some_and(|s| s.chars().count() > 128)
        {
            return Err("resource limit: asset metadata".into());
        }
        match (self.is_audio(), &self.audio) {
            (true, Some(m)) => m.validate()?,
            (false, None) => {}
            _ => return Err("audio_invalid: metadata/format".into()),
        }
        match (self.is_image(), self.width, self.height) {
            (false, None, None) => {}
            (true, Some(w), Some(h)) => {
                if w == 0
                    || h == 0
                    || w > MAX_IMAGE_SIDE
                    || h > MAX_IMAGE_SIDE
                    || (w as usize)
                        .checked_mul(h as usize)
                        .and_then(|p| p.checked_mul(4))
                        .is_none_or(|n| n > MAX_IMAGE_DECODED)
                {
                    return Err("resource limit: image dimensions".into());
                }
            }
            _ => return Err("invalid: asset dimensions/format".into()),
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct Asset {
    pub audio: Option<Arc<super::public_audio::mixer::Pcm>>,
    pub model_images: Vec<Arc<ImageData>>,
    /// Immutable external PNG blobs; embedded image raw bytes stay in the GLB.
    pub dependencies: BTreeMap<String, Arc<[u8]>>,
    pub identity: AssetRef,
    pub bytes: Arc<[u8]>,
    /// GLB payload. Empty for an image payload, which is never a model and is
    /// never uploaded as geometry; `model_data` refuses it by identity.
    pub model: ModelData,
    pub animation: Option<Arc<agp::assets::node_animation::Animation>>,
    /// PNG payload (PLAN-004). `Some` exactly when the identity format is png.
    pub image: Option<Arc<ImageData>>,
    /// Decoded CPU bytes: model geometry or RGBA pixels, one or the other.
    pub cpu_bytes: usize,
    pub gpu_bytes: usize,
    pub decode_us: u128,
}
impl Asset {
    /// The model payload, or a named rejection for an image asset. Every
    /// geometry/instance consumer goes through here so a PNG can never be
    /// mistaken for a zero-mesh model.
    pub fn model_data(&self) -> Result<&ModelData, String> {
        if self.image.is_some() || self.audio.is_some() {
            return Err("wrong asset kind: geometry required".into());
        }
        Ok(&self.model)
    }
    pub fn image_data(&self) -> Result<&ImageData, String> {
        self.image
            .as_deref()
            .ok_or_else(|| "wrong asset kind: image required".to_string())
    }
}
/// Resource identities plus transient admission context for a Host-owned service.
/// The requirement is not an asset/blob, never consumes a raw identity and is
/// not persisted. Cloning a budget view keeps its predecode reservation.
#[derive(Clone, Debug, Default)]
pub struct Registry {
    items: BTreeMap<String, Arc<Asset>>,
    pub(super) audio_manager_required: bool,
}
impl Registry {
    pub fn new() -> Self {
        Self::default()
    }
}
impl std::ops::Deref for Registry {
    type Target = BTreeMap<String, Arc<Asset>>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
impl std::ops::DerefMut for Registry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
impl FromIterator<(String, Arc<Asset>)> for Registry {
    fn from_iter<T: IntoIterator<Item = (String, Arc<Asset>)>>(items: T) -> Self {
        Self {
            items: items.into_iter().collect(),
            audio_manager_required: false,
        }
    }
}
impl<const N: usize> From<[(String, Arc<Asset>); N]> for Registry {
    fn from(items: [(String, Arc<Asset>); N]) -> Self {
        items.into_iter().collect()
    }
}
impl From<BTreeMap<String, Arc<Asset>>> for Registry {
    fn from(items: BTreeMap<String, Arc<Asset>>) -> Self {
        Self {
            items,
            audio_manager_required: false,
        }
    }
}
impl<'a> IntoIterator for &'a Registry {
    type Item = (&'a String, &'a Arc<Asset>);
    type IntoIter = std::collections::btree_map::Iter<'a, String, Arc<Asset>>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
impl IntoIterator for Registry {
    type Item = (String, Arc<Asset>);
    type IntoIter = std::collections::btree_map::IntoIter<String, Arc<Asset>>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
pub fn check_registry(r: &Registry) -> Result<(), String> {
    super::textured_models::check_union(r)
}
pub fn insert(r: &mut Registry, asset: Arc<Asset>) -> Result<(), String> {
    let key = asset.identity.key();
    if let Some(old) = r.get(&key) {
        if old.bytes != asset.bytes || !old.identity.same_content(&asset.identity) {
            return Err("conflict: SHA identity bytes".into());
        }
        return Ok(());
    }
    let mut candidate = r.clone();
    candidate.insert(key, asset);
    check_registry(&candidate)?;
    *r = candidate;
    Ok(())
}
/// Check every existing ancestor, including Windows junctions and other reparse tags.
pub fn regular_path(path: &Path, allow_missing: bool) -> Result<(), String> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::Prefix(p) if !matches!(p.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_))))
    {
        return Err("unsupported: local disk path required".into());
    }
    for p in path.ancestors() {
        match fs::symlink_metadata(p) {
            Ok(m) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err("unsupported: reparse path".into());
                    }
                }
                if m.file_type().is_symlink() {
                    return Err("unsupported: linked path".into());
                }
                if p == path && !m.is_file() && !m.is_dir() {
                    return Err("unsupported: nonregular path".into());
                }
            }
            Err(e) if allow_missing && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("io: {}: {e}", p.display())),
        }
    }
    Ok(())
}
pub fn read_blob(path: &Path) -> Result<Vec<u8>, String> {
    regular_path(path, false)?;
    let f = fs::File::open(path).map_err(|e| format!("io: {e}"))?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("unsupported: regular GLB file required".into());
    }
    let mut bytes = vec![];
    f.take(MAX_RAW as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("io: {e}"))?;
    if bytes.len() > MAX_RAW {
        return Err("resource limit: asset 8MiB".into());
    }
    Ok(bytes)
}
/// Explicit animated GLB admission; legacy import keeps rejecting clips.
pub fn import_animated(path: &Path, existing: &Registry) -> Result<Arc<Asset>, String> {
    let bytes = super::textured_models::RawPool::new(existing).read(path)?;
    super::textured_models::import_animated(path, bytes, existing)
}
pub fn import(path: &Path, existing: &Registry) -> Result<Arc<Asset>, String> {
    let original = super::textured_models::RawPool::new(existing).read(path)?;
    if super::textured_models::is_textured(&original)? {
        return super::textured_models::import(path, original, existing);
    }
    if let Some(old) = existing.values().find(|a| a.bytes == original) {
        return Ok(old.clone());
    }
    let bytes = original.to_vec();
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().chars().take(128).collect());
    let asset = decode_in(bytes, name, existing)?;
    let mut r = existing.clone();
    insert(&mut r, asset.clone())?;
    Ok(asset)
}
enum Input {
    Cached,
    Fresh(Vec<u8>),
}
/// One bounded read. When slots/raw bytes are exhausted, compare against cached
/// immutable payloads while streaming; duplicate import still works without
/// allocating a ninth payload or a second full copy beyond the raw budget.
fn read_for_import(path: &Path, r: &Registry) -> Result<Input, String> {
    let bytes = super::textured_models::RawPool::new(r).read(path)?;
    if r.values().any(|a| a.bytes == bytes) {
        Ok(Input::Cached)
    } else {
        Ok(Input::Fresh(bytes.to_vec()))
    }
}
pub fn load_declared(path: &Path, a: &AssetRef, r: &Registry) -> Result<Arc<Asset>, String> {
    a.validate()?;
    if a.closure.is_some() {
        return super::textured_models::load_declared(path, a, r);
    }
    if a.is_audio() {
        let bytes = super::textured_models::RawPool::new(r).read(path)?;
        if bytes.len() != a.byte_length || agp::renderer::capture::sha256_hex(&bytes) != a.sha256 {
            return Err("audio_invalid: declared hash/length".into());
        }
        let loaded = super::public_audio::import_raw(bytes, r, a.name.clone())?;
        if !loaded.identity.same_content(a) {
            return Err("audio_invalid: declared sample identity".into());
        }
        return Ok(loaded);
    }
    if a.is_image() {
        let bytes = super::textured_models::RawPool::new(r).read(path)?;
        if bytes.len() != a.byte_length || agp::renderer::capture::sha256_hex(&bytes) != a.sha256 {
            return Err("conflict: asset hash/length".into());
        }
        let loaded = import_png_raw(bytes, r, a.name.clone())?;
        if !loaded.identity.same_content(a) {
            return Err("conflict: image identity/dimensions".into());
        }
        return Ok(loaded);
    }
    regular_path(path, false)?;
    if fs::metadata(path).map_err(|e| format!("io: {e}"))?.len() != a.byte_length as u64 {
        return Err(format!("conflict: asset length {}", path.display()));
    }
    if let Some(old) = r.get(&a.key()) {
        // Verify the complete file without creating another retained payload.
        let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut offset = 0;
        let mut chunk = [0u8; 65536];
        loop {
            let n = f.read(&mut chunk).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            if old.bytes.get(offset..offset + n) != Some(&chunk[..n]) {
                return Err(format!("conflict: asset hash {}", path.display()));
            }
            offset += n;
        }
        if offset != old.bytes.len() {
            return Err("conflict: asset length".into());
        }
        return Ok(old.clone());
    }
    let bytes = match read_for_import(path, r)? {
        Input::Cached => {
            return Err("conflict: declared asset SHA differs from actual cached content".into())
        }
        Input::Fresh(b) => b,
    };
    if bytes.len() != a.byte_length || agp::renderer::capture::sha256_hex(&bytes) != a.sha256 {
        return Err(format!("conflict: asset hash/length {}", path.display()));
    }
    decode_declared(bytes, a, r)
}
/// Decode an already identity-verified payload according to its declared
/// format; an unknown format never reaches a decoder.
fn decode_declared(bytes: Vec<u8>, a: &AssetRef, r: &Registry) -> Result<Arc<Asset>, String> {
    if a.is_image() {
        let asset = decode_png_in(bytes, a.name.clone(), r)?;
        if asset.identity.width != a.width || asset.identity.height != a.height {
            return Err("conflict: image dimensions".into());
        }
        return Ok(asset);
    }
    if a.width.is_some() || a.height.is_some() {
        return Err("conflict: model dimensions".into());
    }
    decode_in(bytes, a.name.clone(), r)
}
fn u32_at(b: &[u8], at: usize) -> Result<usize, String> {
    Ok(u32::from_le_bytes(
        b.get(at..at + 4)
            .ok_or("invalid: GLB header/chunk")?
            .try_into()
            .unwrap(),
    ) as usize)
}
fn reject_features(v: &Value) -> Result<(), String> {
    match v {
        Value::Object(m) => {
            for (k, v) in m {
                if k == "extensions" && v.as_object().is_none_or(|m| !m.is_empty()) {
                    return Err("unsupported: extensions".into());
                }
                if (k == "extensionsUsed" || k == "extensionsRequired")
                    && v.as_array().is_none_or(|a| !a.is_empty())
                {
                    return Err(format!("unsupported: {k}"));
                }
                if k != "extras" {
                    reject_features(v)?;
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                reject_features(v)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn span(
    view: gltf::buffer::View<'_>,
    offset: usize,
    count: usize,
    size: usize,
    stride: usize,
    bin: &[u8],
) -> Result<(), String> {
    if count == 0
        || stride < size
        || offset
            .checked_add(
                (count - 1)
                    .checked_mul(stride)
                    .ok_or("invalid: accessor overflow")?,
            )
            .and_then(|x| x.checked_add(size))
            .is_none_or(|end| end > view.length())
        || view
            .offset()
            .checked_add(view.length())
            .is_none_or(|end| end > bin.len())
    {
        return Err("invalid: accessor/buffer span".into());
    }
    Ok(())
}
/// Predicted parts/CPU/GPU arrays and whether the sole scene needs selection.
pub(super) fn preflight(bytes: &[u8]) -> Result<(usize, usize, usize, bool), String> {
    if bytes.len() > MAX_RAW {
        return Err("resource limit: asset 8MiB".into());
    }
    if bytes.len() < 28
        || u32_at(bytes, 0)? != 0x46546c67
        || u32_at(bytes, 4)? != 2
        || u32_at(bytes, 8)? != bytes.len()
    {
        return Err("invalid: GLB v2 header/length".into());
    }
    let json_len = u32_at(bytes, 12)?;
    if json_len > MAX_JSON {
        return Err("resource limit: GLB JSON 1MiB".into());
    }
    let jend = 20usize
        .checked_add(json_len)
        .ok_or("invalid: JSON length")?;
    if json_len % 4 != 0
        || u32_at(bytes, 16)? != 0x4e4f534a
        || jend + 8 > bytes.len()
        || u32_at(bytes, jend + 4)? != 0x004e4942
        || u32_at(bytes, jend)? % 4 != 0
        || jend + 8 + u32_at(bytes, jend)? != bytes.len()
    {
        return Err("invalid: exactly JSON+BIN chunks required".into());
    }
    let raw = super::document::unique_json(&bytes[20..jend])?;
    if raw.get("scene").is_some_and(Value::is_null) {
        return Err("invalid: scene reference".into());
    }
    reject_features(&raw)?;
    if raw["asset"]["version"] != "2.0"
        || raw["asset"].get("minVersion").is_some_and(|v| v != "2.0")
    {
        return Err("unsupported: glTF version".into());
    }
    for key in ["images", "textures", "animations", "skins"] {
        if raw
            .get(key)
            .is_some_and(|a| a.as_array().is_none_or(|a| !a.is_empty()))
        {
            return Err(format!("unsupported: {key}"));
        }
    }
    // Check declarations before gltf's convenience accessors select defaults
    // or choose one transform representation and hide the other declaration.
    if let Some(meshes) = raw.get("meshes").and_then(Value::as_array) {
        for mesh in meshes {
            if mesh.get("weights").is_some()
                || mesh
                    .get("primitives")
                    .and_then(Value::as_array)
                    .is_some_and(|ps| ps.iter().any(|p| p.get("targets").is_some()))
            {
                return Err("unsupported: mesh morph weights/targets declaration".into());
            }
        }
    }
    if let Some(nodes) = raw.get("nodes").and_then(Value::as_array) {
        for node in nodes {
            if node.get("weights").is_some() {
                return Err("unsupported: morph node weights declaration".into());
            }
            if node.get("matrix").is_some()
                && ["translation", "rotation", "scale"]
                    .iter()
                    .any(|key| node.get(key).is_some())
            {
                return Err("invalid: node matrix and TRS are mutually exclusive".into());
            }
        }
    }
    let buffers = raw
        .get("buffers")
        .and_then(Value::as_array)
        .ok_or("invalid: embedded buffer required")?;
    if buffers.len() != 1 || buffers[0].get("uri").is_some() {
        return Err("unsupported: external/data URI or multiple buffers".into());
    }
    // from_slice validates JSON types/references without fetching buffers/images;
    // semantic accessor layouts still require admission before typed readers.
    let g = gltf::Gltf::from_slice(bytes).map_err(|e| {
        format!(
            "invalid: glTF {}",
            e.to_string().chars().take(512).collect::<String>()
        )
    })?;
    let bin = g.blob.as_deref().ok_or("invalid: BIN missing")?;
    if g.buffers().len() != 1
        || g.buffers().next().unwrap().length() > bin.len()
        || bin.len() - g.buffers().next().unwrap().length() > 3
        || g.scenes().len() != 1
    {
        return Err("invalid: single scene/embedded buffer".into());
    }
    let data = &bin[..g.buffers().next().unwrap().length()];
    // Admit every declared primitive, including unreferenced meshes, before
    // any typed reader interprets bytes. Sparse values inherit this accessor
    // type; the storage checks below validate their separate index/value spans.
    use gltf::accessor::{DataType, Dimensions};
    for mesh in g.meshes() {
        for p in mesh.primitives() {
            for (semantic, a) in p.attributes() {
                let valid = match semantic {
                    gltf::Semantic::Positions | gltf::Semantic::Normals => {
                        a.data_type() == DataType::F32
                            && a.dimensions() == Dimensions::Vec3
                            && !a.normalized()
                    }
                    gltf::Semantic::TexCoords(0) => {
                        a.dimensions() == Dimensions::Vec2
                            && match a.data_type() {
                                DataType::F32 => !a.normalized(),
                                DataType::U8 | DataType::U16 => a.normalized(),
                                _ => false,
                            }
                    }
                    _ => return Err(format!("unsupported: vertex semantic {semantic:?}")),
                };
                if !valid {
                    return Err(format!(
                        "invalid: {semantic:?} accessor type/dimensions/normalized"
                    ));
                }
            }
            if let Some(a) = p.indices() {
                if !matches!(a.data_type(), DataType::U8 | DataType::U16 | DataType::U32)
                    || a.dimensions() != Dimensions::Scalar
                    || a.normalized()
                {
                    return Err("invalid: indices accessor type/dimensions/normalized".into());
                }
            }
        }
    }
    for view in g.views() {
        if view
            .offset()
            .checked_add(view.length())
            .is_none_or(|n| n > data.len())
        {
            return Err("invalid: bufferView span".into());
        }
    }
    for a in g.accessors() {
        if let Some(v) = a.view() {
            span(
                v.clone(),
                a.offset(),
                a.count(),
                a.size(),
                v.stride().unwrap_or(a.size()),
                data,
            )?;
        } else if a.sparse().is_none() {
            return Err("invalid: accessor missing storage".into());
        }
        if let Some(s) = a.sparse() {
            if s.count() > a.count() {
                return Err("invalid: sparse count".into());
            }
            let i = s.indices();
            let size = i.index_type().size();
            span(i.view(), i.offset(), s.count(), size, size, data)?;
            let v = s.values();
            span(v.view(), v.offset(), s.count(), a.size(), a.size(), data)?;
            let at = i.view().offset() + i.offset();
            let mut last = None;
            for n in 0..s.count() {
                let b = &data[at + n * size..at + (n + 1) * size];
                let index = match size {
                    1 => b[0] as usize,
                    2 => u16::from_le_bytes(b.try_into().unwrap()) as usize,
                    _ => u32::from_le_bytes(b.try_into().unwrap()) as usize,
                };
                if index >= a.count() || last.is_some_and(|old| old >= index) {
                    return Err("invalid: sparse indices".into());
                }
                last = Some(index);
            }
        }
    }
    for material in g.materials() {
        let p = material.pbr_metallic_roughness();
        if material.alpha_mode() != gltf::material::AlphaMode::Opaque
            || material.double_sided()
            || p.base_color_factor()
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || p.base_color_factor()[3] != 1.0
            || p.metallic_factor() != 0.0
            || p.roughness_factor() != 1.0
            || material.emissive_factor() != [0.0; 3]
            || p.base_color_texture().is_some()
            || p.metallic_roughness_texture().is_some()
            || material.normal_texture().is_some()
            || material.occlusion_texture().is_some()
            || material.emissive_texture().is_some()
        {
            return Err("unsupported: opaque flat-color material required".into());
        }
    }
    let mut mesh_parts = vec![];
    let mut mesh_sizes = vec![];
    let mut mesh_gpu = vec![];
    let mut declared = (0usize, 0usize, 0usize);
    for mesh in g.meshes() {
        let mut parts = 0;
        let (mut vertices, mut indices, mut gpu_bytes) = (0usize, 0usize, 0usize);
        for p in mesh.primitives() {
            if p.mode() != gltf::mesh::Mode::Triangles || p.morph_targets().next().is_some() {
                return Err("unsupported: static triangles without morph targets".into());
            }
            for (s, _) in p.attributes() {
                if !matches!(
                    s,
                    gltf::Semantic::Positions
                        | gltf::Semantic::Normals
                        | gltf::Semantic::TexCoords(0)
                ) {
                    return Err(format!("unsupported: vertex semantic {s:?}"));
                }
            }
            let pos = p
                .get(&gltf::Semantic::Positions)
                .ok_or("invalid: POSITION required")?;
            let norm = p
                .get(&gltf::Semantic::Normals)
                .ok_or("invalid: NORMAL required")?;
            let count = pos.count();
            if count == 0 || count > 65536 || norm.count() != count {
                return Err("resource limit: primitive vertices/normals".into());
            }
            if let Some(uv) = p.get(&gltf::Semantic::TexCoords(0)) {
                if uv.count() != count {
                    return Err("invalid: texcoord count".into());
                }
            }
            let ic = p.indices().map_or(count, |a| a.count());
            if ic == 0 || ic % 3 != 0 || ic > 786432 {
                return Err("invalid: triangle index count/budget".into());
            }
            let reader = p.reader(|_| Some(data));
            for v in reader.read_positions().ok_or("invalid: position storage")? {
                if v.iter().any(|v| !v.is_finite()) {
                    return Err("invalid: nonfinite position".into());
                }
            }
            for n in reader.read_normals().ok_or("invalid: normal storage")? {
                let n = Vec3::from(n);
                if !n.is_finite() || !n.length_squared().is_finite() || n.length_squared() < 1e-12 {
                    return Err("invalid: normal".into());
                }
            }
            if let Some(idx) = reader.read_indices() {
                for i in idx.into_u32() {
                    if i > 65535 || i as usize >= count {
                        return Err("invalid: u16/vertex index range".into());
                    }
                }
            }
            if let Some(uv) = reader.read_tex_coords(0) {
                if uv.into_f32().flatten().any(|v| !v.is_finite()) {
                    return Err("invalid: texcoord".into());
                }
            }
            vertices += count;
            indices += ic;
            gpu_bytes += count * 40 + ((ic * 2 + 3) & !3);
            parts += 1;
            if parts > 16 || vertices > 262144 || indices > 786432 {
                return Err("resource limit: declared mesh geometry".into());
            }
        }
        declared.0 += parts;
        declared.1 += vertices;
        declared.2 += indices;
        if declared.0 > 16 || declared.1 > 262144 || declared.2 > 786432 {
            return Err("resource limit: declared file geometry".into());
        }
        mesh_parts.push(parts);
        mesh_sizes.push((vertices, indices));
        mesh_gpu.push(gpu_bytes);
    }
    if g.nodes().len() > 128 {
        return Err("resource limit: GLB nodes 128".into());
    }
    let nodes: Vec<_> = g.nodes().collect();
    let mut parent = vec![None; nodes.len()];
    for node in &nodes {
        if node.skin().is_some() || node.weights().is_some() {
            return Err("unsupported: skin/morph node".into());
        }
        for child in node.children() {
            if parent[child.index()].replace(node.index()).is_some() {
                return Err("invalid: node has multiple parents".into());
            }
        }
    }
    let mut total = (0usize, 0usize, 0usize);
    let mut total_gpu = 0;
    for node in &nodes {
        let mut chain = BTreeSet::new();
        let mut at = Some(node.index());
        let mut world = Mat4::IDENTITY;
        while let Some(i) = at {
            if !chain.insert(i) {
                return Err("invalid: node cycle".into());
            }
            let m = Mat4::from_cols_array_2d(&nodes[i].transform().matrix());
            if !m.is_finite()
                || m.x_axis.w != 0.0
                || m.y_axis.w != 0.0
                || m.z_axis.w != 0.0
                || m.w_axis.w != 1.0
            {
                return Err("invalid: finite affine transform required".into());
            }
            let det = Mat3::from_mat4(m).determinant();
            if !det.is_finite() || det <= 0.0 {
                return Err("unsupported: singular/mirrored node transform".into());
            }
            world = m * world;
            at = parent[i];
        }
        let det = Mat3::from_mat4(world).determinant();
        if !world.is_finite()
            || !det.is_finite()
            || det <= 0.0
            || !Mat3::from_mat4(world).inverse().is_finite()
        {
            return Err("invalid: transformed geometry matrix".into());
        }
        if let Some(m) = node.mesh() {
            let i = m.index();
            total.0 += mesh_parts[i];
            total.1 += mesh_sizes[i].0;
            total.2 += mesh_sizes[i].1;
            total_gpu += mesh_gpu[i];
        }
    }
    if total.0 > 16 || total.1 > 262144 || total.2 > 786432 {
        return Err("resource limit: expanded geometry".into());
    }
    let mut roots = BTreeSet::new();
    for n in g
        .default_scene()
        .unwrap_or_else(|| g.scenes().next().unwrap())
        .nodes()
    {
        if parent[n.index()].is_some() || !roots.insert(n.index()) {
            return Err("invalid: scene roots".into());
        }
    }
    Ok((
        total.0,
        total.1 * 24 + total.2 * 4,
        total_gpu,
        g.default_scene().is_none(),
    ))
}
/// The frozen decoder requires an explicit default. For an admitted single-scene
/// file, make that selection explicit in bounded parser scratch only. Original
/// bytes, byte length and persistent SHA remain unchanged; BIN is copied exactly.
fn select_sole_scene(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let end = 20 + u32_at(bytes, 12)?;
    let mut json = super::document::unique_json(&bytes[20..end])?;
    json["scene"] = Value::from(0);
    let mut json = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    let total = 20 + json.len() + bytes.len() - end;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&bytes[..8]);
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4e4f534au32.to_le_bytes());
    out.extend(json);
    out.extend_from_slice(&bytes[end..]);
    Ok(out)
}
pub fn decode(bytes: Vec<u8>, name: Option<String>) -> Result<Arc<Asset>, String> {
    decode_in(bytes, name, &Registry::new())
}
pub fn decode_in(
    bytes: Vec<u8>,
    name: Option<String>,
    existing: &Registry,
) -> Result<Arc<Asset>, String> {
    let start = std::time::Instant::now();
    let (predicted, cpu, gpu, implicit_scene) = preflight(&bytes)?; // no import_slice / external IO before this point
    let original_sha = agp::renderer::capture::sha256_hex(&bytes);
    super::textured_models::candidate_budget(
        existing,
        &original_sha,
        bytes.len(),
        cpu,
        gpu,
        false,
    )?;
    let selected = if implicit_scene {
        Some(select_sole_scene(&bytes)?)
    } else {
        None
    };
    let input = selected.as_deref().unwrap_or(&bytes);
    let decoder_sha = if implicit_scene {
        agp::renderer::capture::sha256_hex(input)
    } else {
        original_sha.clone()
    };
    DECODE_ATTEMPTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut model = load_glb_bytes(input).map_err(|e| format!("invalid: decoder {e}"))?;
    if model.source_sha256 != decoder_sha {
        return Err("invalid: decoder input identity".into());
    }
    model.source_sha256 = original_sha.clone();
    if model.meshes.is_empty() || model.meshes.len() > predicted {
        return Err("invalid: decoded mesh expansion".into());
    }
    let (mut vertices, mut indices, mut cpu, mut gpu) = (0usize, 0usize, 0usize, 0usize);
    for (i, m) in model.meshes.iter_mut().enumerate() {
        if m.positions.is_empty()
            || m.positions.len() != m.normals.len()
            || m.indices.is_empty()
            || m.indices.len() % 3 != 0
            || m.positions.len() > 65536
            || m.indices
                .iter()
                .any(|n| *n > 65535 || *n as usize >= m.positions.len())
            || m.positions.iter().flatten().any(|v| !v.is_finite())
            || m.normals.iter().any(|n| {
                let n = Vec3::from(*n);
                !n.is_finite() || n.length_squared() < 0.99 || n.length_squared() > 1.01
            })
        {
            return Err("invalid: decoded geometry".into());
        }
        // Stable expanded identity: duplicate names and repeated mesh nodes stay distinct.
        m.name = format!("primitive-{i}");
        m.material.name = format!("material-{i}");
        // A missing glTF material has metallic default1; this renderer's documented
        // missing-material rule is fixed white flat-color, metallic0/roughness1.
        m.material.metallic = 0.0;
        m.material.roughness = 1.0;
        vertices += m.positions.len();
        indices += m.indices.len();
        cpu += m.positions.len() * 24 + m.indices.len() * 4;
        gpu += m.positions.len() * 40 + ((m.indices.len() * 2 + 3) & !3);
    }
    if vertices > 262144 || indices > 786432 || cpu > MAX_GEOMETRY || gpu > MAX_GEOMETRY {
        return Err("resource limit: decoded geometry".into());
    }
    let identity = AssetRef {
        audio: None,
        closure: None,
        format: FORMAT_GLB.into(),
        sha256: original_sha,
        byte_length: bytes.len(),
        name,
        width: None,
        height: None,
    };
    identity.validate()?;
    if model.source_sha256 != identity.sha256 {
        return Err("invalid: decoder source identity".into());
    }
    Ok(Arc::new(Asset {
        audio: None,
        animation: None,
        model_images: vec![],
        dependencies: BTreeMap::new(),
        identity,
        bytes: bytes.into(),
        model,
        image: None,
        cpu_bytes: cpu,
        gpu_bytes: gpu,
        decode_us: start.elapsed().as_micros(),
    }))
}

/// The PNG candidate/union budget is checked after the bounded preflight and
/// before the decoder allocates pixels; the decoded payload then has to match
/// the predicted size exactly.
pub fn decode_png_in(
    bytes: Vec<u8>,
    name: Option<String>,
    existing: &Registry,
) -> Result<Arc<Asset>, String> {
    let start = std::time::Instant::now();
    if bytes.len() > MAX_RAW {
        return Err("resource limit: image 8MiB".into());
    }
    let header = agp::assets::preflight_png(&bytes)?;
    let decoded = (header.width as usize)
        .checked_mul(header.height as usize)
        .and_then(|p| p.checked_mul(4))
        .ok_or("invalid: image dimension overflow")?;
    if decoded > MAX_IMAGE_DECODED {
        return Err("resource limit: decoded RGBA 16MiB".into());
    }
    let sha = agp::renderer::capture::sha256_hex(&bytes);
    super::textured_models::candidate_budget(existing, &sha, bytes.len(), decoded, decoded, true)?;
    let cached = existing.values().find_map(|a| {
        a.image
            .as_ref()
            .filter(|i| i.source_sha256 == sha)
            .cloned()
            .or_else(|| {
                a.model_images
                    .iter()
                    .find(|i| i.source_sha256 == sha)
                    .cloned()
            })
    });
    let image = if let Some(i) = cached {
        i
    } else {
        record_decode();
        Arc::new(agp::assets::decode_png_bytes(&bytes, &sha)?)
    };
    if image.width != header.width
        || image.height != header.height
        || image.byte_length() != decoded
        || image.source_sha256 != sha
    {
        return Err("invalid: decoded image identity".into());
    }
    let identity = AssetRef {
        audio: None,
        closure: None,
        format: FORMAT_PNG.into(),
        sha256: sha.clone(),
        byte_length: bytes.len(),
        name,
        width: Some(image.width),
        height: Some(image.height),
    };
    identity.validate()?;
    let raw = existing.values().find_map(|a| {
        if a.identity.sha256 == sha {
            Some(a.bytes.clone())
        } else {
            a.dependencies.get(&sha).cloned()
        }
    });
    Ok(Arc::new(Asset {
        audio: None,
        animation: None,
        model_images: vec![],
        dependencies: BTreeMap::new(),
        identity,
        bytes: raw.unwrap_or_else(|| bytes.into()),
        model: ModelData {
            source_sha256: String::new(),
            meshes: vec![],
        },
        image: Some(image),
        cpu_bytes: decoded,
        gpu_bytes: decoded,
        decode_us: start.elapsed().as_micros(),
    }))
}

/// Import one local static PNG under the same no-overwrite/cached-identity
/// rules as model import. A duplicate payload is reused without re-decoding.
pub fn import_png(path: &Path, existing: &Registry) -> Result<Arc<Asset>, String> {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().chars().take(128).collect());
    import_png_named(path, existing, name)
}
fn import_png_named(
    path: &Path,
    existing: &Registry,
    name: Option<String>,
) -> Result<Arc<Asset>, String> {
    let bytes = super::textured_models::RawPool::new(existing).read(path)?;
    import_png_raw(bytes, existing, name)
}
fn import_png_raw(
    bytes: Arc<[u8]>,
    existing: &Registry,
    name: Option<String>,
) -> Result<Arc<Asset>, String> {
    let hash = agp::renderer::capture::sha256_hex(&bytes);
    if let Some(old) = existing.get(&format!("sha256:{hash}")) {
        old.image_data()?;
        return Ok(old.clone());
    }
    let shared = existing.values().find_map(|a| {
        a.model_images
            .iter()
            .find(|i| i.source_sha256 == hash)
            .cloned()
    });
    let asset = if let Some(image) = shared {
        let n = image.rgba.len();
        super::textured_models::candidate_budget(existing, &hash, bytes.len(), n, n, true)?;
        Arc::new(Asset {
            audio: None,
            animation: None,
            identity: AssetRef {
                audio: None,
                closure: None,
                format: FORMAT_PNG.into(),
                sha256: hash,
                byte_length: bytes.len(),
                name,
                width: Some(image.width),
                height: Some(image.height),
            },
            bytes,
            model: ModelData {
                source_sha256: String::new(),
                meshes: vec![],
            },
            image: Some(image),
            model_images: vec![],
            dependencies: BTreeMap::new(),
            cpu_bytes: n,
            gpu_bytes: n,
            decode_us: 0,
        })
    } else {
        decode_png_in(bytes.to_vec(), name, existing)?
    };
    let mut r = existing.clone();
    insert(&mut r, asset.clone())?;
    Ok(asset)
}
