//! PLAN-005 explicit GLB/PNG closures. Only this module resolves image URIs;
//! the general GLB decoder receives geometry-only, admitted scratch, never paths.
use super::assets::{self, Asset, AssetRef, Registry};
use agp::assets::{
    textured::{MeshVisual, Sampling},
    ImageData,
};
use glam::Mat4;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageRef {
    pub source: String,
    pub sha256: String,
    pub byte_length: usize,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Closure {
    pub id: String,
    pub images: Vec<ImageRef>,
}
fn sha(b: &[u8]) -> String {
    agp::renderer::capture::sha256_hex(b)
}
fn valid_sha(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn closure_id(root: &str, len: usize, images: &[ImageRef]) -> String {
    sha(&serde_json::to_vec(&("autographics.textured/1", root, len, images)).unwrap())
}
pub fn validate_ref(a: &AssetRef) -> Result<(), String> {
    if let Some(c) = &a.closure {
        if a.format != "glb"
            || c.images.is_empty()
            || c.images.len() > 8
            || c.id != closure_id(&a.sha256, a.byte_length, &c.images)
        {
            return Err("invalid: model closure identity".into());
        }
        for i in &c.images {
            if !valid_sha(&i.sha256)
                || i.byte_length == 0
                || i.byte_length > assets::MAX_RAW
                || i.width == 0
                || i.height == 0
                || i.width > 2048
                || i.height > 2048
            {
                return Err("invalid: model image identity".into());
            }
        }
    }
    Ok(())
}
fn json(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.len() > assets::MAX_RAW {
        return Err("resource limit: GLB 8MiB".into());
    }
    if bytes.len() < 28
        || bytes[..4] != *b"glTF"
        || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
    {
        return Err("invalid: GLB header".into());
    }
    let n = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if n > assets::MAX_JSON {
        return Err("resource limit: GLB JSON 1MiB".into());
    }
    let end = 20 + n;
    if n % 4 != 0
        || end + 8 > bytes.len()
        || bytes[16..20] != *b"JSON"
        || bytes[end + 4..end + 8] != [b'B', b'I', b'N', 0]
        || u32::from_le_bytes(bytes[end..end + 4].try_into().unwrap()) as usize
            != bytes.len() - end - 8
    {
        return Err("invalid: GLB chunks".into());
    }
    Ok((super::document::unique_json(&bytes[20..end])?, end))
}
pub fn is_textured(bytes: &[u8]) -> Result<bool, String> {
    let (j, _) = json(bytes)?;
    Ok(j.get("images")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
        || j.get("textures")
            .and_then(Value::as_array)
            .is_some_and(|a| !a.is_empty()))
}
fn uri(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 256
        || !s.ends_with(".png")
        || !s.is_ascii()
        || s.bytes()
            .any(|b| b < 32 || matches!(b, b'\\' | b':' | b'%' | b'?' | b'#'))
        || s.split('/')
            .any(|p| p.is_empty() || p == "." || p == ".." || p.ends_with([' ', '.']))
    {
        return Err("unsupported: local relative PNG URI required".into());
    }
    Ok(())
}
fn uint(v: &Value, name: &str) -> Result<usize, String> {
    v.as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("invalid: {name}"))
}
fn array<'a>(j: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    j.get(k)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("invalid: {k} array"))
}
fn extensions(v: &Value, path: &str) -> Result<(), String> {
    match v {
        Value::Object(m) => {
            for (k, v) in m {
                if k == "extras" {
                    continue;
                }
                if k == "extensions" {
                    if !path.ends_with("/pbrMetallicRoughness/baseColorTexture") {
                        return Err("unsupported: extension location".into());
                    }
                    let m = v.as_object().ok_or("invalid: extensions object")?;
                    if m.keys().any(|s| s != "KHR_texture_transform") {
                        return Err("unsupported: texture extension".into());
                    }
                } else if k == "extensionsUsed" || k == "extensionsRequired" {
                    if v.as_array()
                        .is_none_or(|a| a.iter().any(|s| s != "KHR_texture_transform"))
                    {
                        return Err("unsupported: required/used extension".into());
                    }
                } else {
                    extensions(v, &format!("{path}/{k}"))?;
                }
            }
        }
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                extensions(v, &format!("{path}/{i}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn strip_extensions(v: &mut Value) {
    match v {
        Value::Object(m) => {
            m.remove("extensions");
            m.remove("extensionsUsed");
            m.remove("extensionsRequired");
            for (k, v) in m {
                if k != "extras" {
                    strip_extensions(v);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                strip_extensions(v);
            }
        }
        _ => {}
    }
}
fn scratch(j: &Value, bytes: &[u8], end: usize) -> Result<Vec<u8>, String> {
    let mut j = j.clone();
    strip_extensions(&mut j);
    let o = j.as_object_mut().ok_or("invalid: GLB JSON object")?;
    o.remove("images");
    o.remove("textures");
    o.remove("samplers");
    if let Some(meshes) = o.get_mut("meshes").and_then(Value::as_array_mut) {
        for mesh in meshes {
            if let Some(ps) = mesh.get_mut("primitives").and_then(Value::as_array_mut) {
                for p in ps {
                    if let Some(attrs) = p.get_mut("attributes").and_then(Value::as_object_mut) {
                        attrs.remove("TANGENT");
                    }
                }
            }
        }
    }
    if let Some(ms) = o.get_mut("materials").and_then(Value::as_array_mut) {
        for m in ms {
            if let Some(pbr) = m
                .get_mut("pbrMetallicRoughness")
                .and_then(Value::as_object_mut)
            {
                pbr.remove("baseColorTexture");
            }
            if let Some(m) = m.as_object_mut() {
                m.insert("doubleSided".into(), Value::Bool(false));
            }
        }
    }
    let mut b = serde_json::to_vec(&j).map_err(|e| e.to_string())?;
    while b.len() % 4 != 0 {
        b.push(b' ');
    }
    let total = 20 + b.len() + bytes.len() - end;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&bytes[..8]);
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend(b);
    out.extend_from_slice(&bytes[end..]);
    Ok(out)
}
#[derive(Clone)]
struct Material {
    image: Option<usize>,
    sampling: Sampling,
    double: bool,
    offset: [f32; 2],
    scale: [f32; 2],
    rotation: f32,
}
fn pair(v: Option<&Value>, default: [f32; 2]) -> Result<[f32; 2], String> {
    let Some(v) = v else {
        return Ok(default);
    };
    let a = v
        .as_array()
        .filter(|a| a.len() == 2)
        .ok_or("invalid: UV transform pair")?;
    let f = |v: &Value| {
        v.as_f64()
            .map(|n| n as f32)
            .filter(|n| n.is_finite())
            .ok_or("invalid: finite UV transform")
    };
    Ok([f(&a[0])?, f(&a[1])?])
}
fn sampling(v: &Value) -> Result<Sampling, String> {
    let enumv = |k: &str, d: u32| -> Result<u32, String> {
        v.get(k).map_or(Ok(d), |n| {
            uint(n, k).and_then(|n| u32::try_from(n).map_err(|_| "invalid: sampler enum".into()))
        })
    };
    let mut s = Sampling {
        mag: enumv("magFilter", 9729)?,
        min: enumv("minFilter", 9729)?,
        wrap_s: enumv("wrapS", 10497)?,
        wrap_t: enumv("wrapT", 10497)?,
    };
    s.min = match s.min {
        9728 | 9984 | 9986 => 9728,
        9729 | 9985 | 9987 => 9729,
        _ => return Err("unsupported: minFilter".into()),
    };
    if ![9728, 9729].contains(&s.mag)
        || ![33071, 33648, 10497].contains(&s.wrap_s)
        || ![33071, 33648, 10497].contains(&s.wrap_t)
    {
        return Err("unsupported: sampler".into());
    }
    Ok(s)
}
struct Plan {
    j: Value,
    scratch: Vec<u8>,
    cpu: usize,
    gpu: usize,
    materials: Vec<Material>,
}
fn inspect(bytes: &[u8]) -> Result<Plan, String> {
    let (j, end) = json(bytes)?;
    extensions(&j, "")?;
    let clean = scratch(&j, bytes, end)?;
    let (_, cpu, gpu, _) = assets::preflight(&clean)?;
    // Fixed Kenney meshes carry unused tangents. Check their semantic type
    // before typed reading, after the common all-accessor span validation.
    let g = gltf::Gltf::from_slice(bytes).map_err(|e| format!("invalid: glTF {e}"))?;
    let bin = g.blob.as_deref().ok_or("invalid: BIN")?;
    for mesh in g.meshes() {
        for p in mesh.primitives() {
            if let Some(t) = p.get(&gltf::Semantic::Tangents) {
                if t.data_type() != gltf::accessor::DataType::F32
                    || t.dimensions() != gltf::accessor::Dimensions::Vec4
                    || t.normalized()
                    || t.count()
                        != p.get(&gltf::Semantic::Positions)
                            .ok_or("invalid: POSITION")?
                            .count()
                {
                    return Err("invalid: tangent accessor type/count".into());
                }
                for t in p
                    .reader(|_| Some(bin))
                    .read_tangents()
                    .ok_or("invalid: tangent storage")?
                {
                    if t.iter().any(|v| !v.is_finite()) || (t[3] != 1. && t[3] != -1.) {
                        return Err("invalid: finite tangent/w".into());
                    }
                }
            }
        }
    }
    let images = array(&j, "images")?;
    if images.is_empty() || images.len() > 8 {
        return Err("resource limit: declared images".into());
    }
    for i in images {
        match (i.get("uri"), i.get("bufferView")) {
            (Some(u), None) => {
                uri(u.as_str().ok_or("invalid: image URI")?)?;
                if i.get("mimeType").is_some_and(|v| v != "image/png") {
                    return Err("unsupported: PNG mime".into());
                }
            }
            (None, Some(v)) => {
                uint(v, "image bufferView")?;
                if i["mimeType"] != "image/png" {
                    return Err("unsupported: embedded PNG mime".into());
                }
            }
            _ => return Err("invalid: exactly one image source".into()),
        }
    }
    let samplers = j
        .get("samplers")
        .map(|_| array(&j, "samplers"))
        .transpose()?
        .cloned()
        .unwrap_or_default();
    let samplers = samplers
        .iter()
        .map(sampling)
        .collect::<Result<Vec<_>, _>>()?;
    let mut textures = vec![];
    for t in array(&j, "textures")? {
        let source = uint(&t["source"], "texture source")?;
        if source >= images.len() {
            return Err("invalid: texture image index".into());
        }
        let s = if let Some(s) = t.get("sampler") {
            *samplers
                .get(uint(s, "sampler index")?)
                .ok_or("invalid: sampler index")?
        } else {
            Sampling::default()
        };
        textures.push((source, s));
    }
    let mut materials = vec![];
    for m in j
        .get("materials")
        .map(|_| array(&j, "materials"))
        .transpose()?
        .into_iter()
        .flatten()
    {
        let mut material = Material {
            image: None,
            sampling: Sampling::default(),
            double: false,
            offset: [0.; 2],
            scale: [1.; 2],
            rotation: 0.,
        };
        if let Some(d) = m.get("doubleSided") {
            material.double = d.as_bool().ok_or("invalid: doubleSided")?;
        }
        if let Some(t) = m
            .get("pbrMetallicRoughness")
            .and_then(|p| p.get("baseColorTexture"))
        {
            if t.get("texCoord").is_some_and(|v| v != 0) {
                return Err("unsupported: UV0 only".into());
            }
            let (image, s) = textures
                .get(uint(&t["index"], "texture index")?)
                .ok_or("invalid: texture index")?;
            material.image = Some(*image);
            material.sampling = *s;
            if let Some(x) = t
                .get("extensions")
                .and_then(|e| e.get("KHR_texture_transform"))
            {
                let obj = x.as_object().ok_or("invalid: texture transform")?;
                if obj
                    .keys()
                    .any(|k| !["offset", "scale", "rotation", "texCoord"].contains(&k.as_str()))
                {
                    return Err("unsupported: texture transform member".into());
                }
                if x.get("texCoord").is_some_and(|v| v != 0) {
                    return Err("unsupported: transform UV0 only".into());
                }
                material.offset = pair(x.get("offset"), [0.; 2])?;
                material.scale = pair(x.get("scale"), [1.; 2])?;
                material.rotation = x.get("rotation").map_or(Ok(0.), |v| {
                    v.as_f64()
                        .map(|n| n as f32)
                        .filter(|n| n.is_finite())
                        .ok_or("invalid: UV rotation")
                })?;
            }
        }
        materials.push(material);
    }
    Ok(Plan {
        j,
        scratch: clean,
        cpu,
        gpu,
        materials,
    })
}
type RawImages = (Vec<ImageRef>, Vec<Arc<[u8]>>, BTreeMap<String, Arc<[u8]>>);
fn image_inputs(
    plan: &Plan,
    bytes: &[u8],
    mut external: impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<RawImages, String> {
    let g = gltf::Gltf::from_slice(bytes).map_err(|e| format!("invalid: glTF {e}"))?;
    let bin = g.blob.as_deref().ok_or("invalid: BIN")?;
    let mut refs = vec![];
    let mut raws = vec![];
    let mut dependencies = BTreeMap::new();
    let mut sources = BTreeSet::new();
    for i in array(&plan.j, "images")? {
        let (source, b): (String, Vec<u8>) = if let Some(u) = i.get("uri") {
            let u = u.as_str().ok_or("invalid: image URI")?;
            uri(u)?;
            (format!("uri:{u}"), external(u)?)
        } else {
            let index = uint(&i["bufferView"], "image view")?;
            let view = g
                .views()
                .nth(index)
                .ok_or("invalid: image bufferView index")?;
            let end = view
                .offset()
                .checked_add(view.length())
                .ok_or("invalid: image span")?;
            if view.buffer().index() != 0 || view.stride().is_some() {
                return Err("invalid: image bufferView layout".into());
            }
            (
                format!("view:{index}"),
                bin.get(view.offset()..end)
                    .ok_or("invalid: image bufferView span")?
                    .to_vec(),
            )
        };
        if !sources.insert(source.clone()) {
            return Err("invalid: duplicate image source edge".into());
        }
        if b.len() > assets::MAX_RAW {
            return Err("resource limit: PNG raw".into());
        }
        let h = agp::assets::preflight_png(&b)?;
        let id = sha(&b);
        let b: Arc<[u8]> = b.into();
        if source.starts_with("uri:") {
            dependencies.insert(id.clone(), b.clone());
        }
        refs.push(ImageRef {
            source,
            sha256: id,
            byte_length: b.len(),
            width: h.width,
            height: h.height,
        });
        raws.push(b);
    }
    Ok((refs, raws, dependencies))
}
#[derive(Default)]
struct Usage {
    raw: BTreeMap<String, usize>,
    images: BTreeMap<String, usize>,
    geometry: BTreeMap<String, (usize, usize)>,
}
impl Usage {
    fn add(&mut self, a: &Asset) {
        self.raw.insert(a.identity.sha256.clone(), a.bytes.len());
        for (k, b) in &a.dependencies {
            self.raw.insert(k.clone(), b.len());
        }
        if let Some(i) = &a.image {
            self.images.insert(i.source_sha256.clone(), i.rgba.len());
        } else {
            self.geometry
                .insert(a.identity.key(), (a.cpu_bytes, a.gpu_bytes));
        }
        for i in &a.model_images {
            self.images.insert(i.source_sha256.clone(), i.rgba.len());
        }
    }
    fn check(&self) -> Result<(), String> {
        let pixels = self.images.values().sum::<usize>();
        let cpu = self.geometry.values().map(|n| n.0).sum::<usize>() + pixels;
        let gpu = self.geometry.values().map(|n| n.1).sum::<usize>() + pixels;
        if self.raw.len() > 8
            || self.raw.values().sum::<usize>() > assets::MAX_RAW_TOTAL
            || cpu > assets::MAX_GEOMETRY
            || gpu > assets::MAX_GEOMETRY
        {
            return Err("resource limit: active asset/texture union".into());
        }
        Ok(())
    }
}
pub fn check_union(r: &Registry) -> Result<(), String> {
    let mut u = Usage::default();
    for a in r.values() {
        u.add(a);
    }
    u.check()
}

pub struct Blob {
    pub identity: AssetRef,
    pub bytes: Arc<[u8]>,
}
pub fn blobs(r: &Registry) -> Result<BTreeMap<String, Blob>, String> {
    let mut out = BTreeMap::new();
    for a in r.values() {
        let mut identity = a.identity.clone();
        identity.closure = None;
        out.insert(
            format!("{}.{}", identity.sha256, identity.format),
            Blob {
                identity,
                bytes: a.bytes.clone(),
            },
        );
        for (id, b) in &a.dependencies {
            let h = agp::assets::preflight_png(b)?;
            let identity = AssetRef {
                closure: None,
                format: "png".into(),
                sha256: id.clone(),
                byte_length: b.len(),
                name: None,
                width: Some(h.width),
                height: Some(h.height),
            };
            out.insert(
                format!("{id}.png"),
                Blob {
                    identity,
                    bytes: b.clone(),
                },
            );
        }
    }
    Ok(out)
}

fn decode(
    bytes: Vec<u8>,
    name: Option<String>,
    existing: &Registry,
    external: impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<Arc<Asset>, String> {
    let start = std::time::Instant::now();
    let plan = inspect(&bytes)?;
    let (refs, raws, dependencies) = image_inputs(&plan, &bytes, external)?;
    let root_sha = sha(&bytes);
    let id = closure_id(&root_sha, bytes.len(), &refs);
    if let Some(old) = existing.get(&format!("closure:{id}")) {
        return Ok(old.clone());
    }
    let g = gltf::Gltf::from_slice(&bytes).map_err(|e| format!("invalid: glTF {e}"))?;
    let extra = g
        .nodes()
        .filter_map(|n| n.mesh())
        .map(|m| {
            m.primitives()
                .map(|p| {
                    p.get(&gltf::Semantic::Positions).unwrap().count() * 8
                        + std::mem::size_of::<MeshVisual>()
                })
                .sum::<usize>()
        })
        .sum::<usize>();
    let cpu = plan.cpu + extra;
    let gpu = plan.gpu
        + g.nodes()
            .filter_map(|n| n.mesh())
            .map(|m| {
                m.primitives()
                    .map(|p| p.get(&gltf::Semantic::Positions).unwrap().count() * 8)
                    .sum::<usize>()
            })
            .sum::<usize>();
    let mut usage = Usage::default();
    for a in existing.values() {
        usage.add(a);
    }
    usage.raw.insert(root_sha.clone(), bytes.len());
    for (k, b) in &dependencies {
        usage.raw.insert(k.clone(), b.len());
    }
    for i in &refs {
        usage
            .images
            .insert(i.sha256.clone(), i.width as usize * i.height as usize * 4);
    }
    usage.geometry.insert(format!("closure:{id}"), (cpu, gpu));
    usage.check()?;
    let mut images: Vec<Arc<ImageData>> = vec![];
    let mut decoded: BTreeMap<String, Arc<ImageData>> = BTreeMap::new();
    for a in existing.values() {
        if let Some(i) = &a.image {
            decoded.insert(i.source_sha256.clone(), i.clone());
        }
        for i in &a.model_images {
            decoded.insert(i.source_sha256.clone(), i.clone());
        }
    }
    for (r, b) in refs.iter().zip(&raws) {
        let image = if let Some(i) = decoded.get(&r.sha256) {
            i.clone()
        } else {
            let image = Arc::new(agp::assets::decode_png_bytes(b, &r.sha256)?);
            if image.rgba.chunks_exact(4).any(|p| p[3] != 255) {
                return Err("unsupported: opaque model PNG required".into());
            }
            decoded.insert(r.sha256.clone(), image.clone());
            image
        };
        if image.rgba.chunks_exact(4).any(|p| p[3] != 255) {
            return Err("unsupported: opaque model PNG required".into());
        }
        images.push(image);
    }
    let mut model = assets::decode(plan.scratch.clone(), None)?.model.clone();
    let bin = g.blob.as_deref().ok_or("invalid: BIN")?;
    fn visit(
        node: gltf::Node<'_>,
        parent: Mat4,
        bin: &[u8],
        plan: &Plan,
        images: &[Arc<ImageData>],
        visuals: &mut Vec<MeshVisual>,
    ) -> Result<(), String> {
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        if let Some(mesh) = node.mesh() {
            for (pi, p) in mesh.primitives().enumerate() {
                let material = p.material().index().and_then(|i| plan.materials.get(i));
                let count = p
                    .get(&gltf::Semantic::Positions)
                    .ok_or("invalid: POSITION")?
                    .count();
                let mut uv = p
                    .reader(|_| Some(bin))
                    .read_tex_coords(0)
                    .map(|a| a.into_f32().collect::<Vec<_>>())
                    .unwrap_or_else(|| vec![[0.; 2]; count]);
                if let Some(m) = material {
                    if m.image.is_some() && p.get(&gltf::Semantic::TexCoords(0)).is_none() {
                        return Err("invalid: textured primitive requires UV0".into());
                    }
                    let (s, c) = m.rotation.sin_cos();
                    for u in &mut uv {
                        let x = u[0] * m.scale[0];
                        let y = u[1] * m.scale[1];
                        *u = [m.offset[0] + c * x - s * y, m.offset[1] + s * x + c * y];
                        if u.iter().any(|v| !v.is_finite()) {
                            return Err("invalid: transformed UV".into());
                        }
                    }
                }
                visuals.push(MeshVisual {
                    uv,
                    image: material.and_then(|m| m.image.map(|i| images[i].clone())),
                    sampling: material.map_or(Sampling::default(), |m| m.sampling),
                    double_sided: material.is_some_and(|m| m.double),
                    node_index: node.index(),
                    mesh_index: mesh.index(),
                    primitive_index: pi,
                    world: world.to_cols_array_2d(),
                });
            }
        }
        for child in node.children() {
            visit(child, world, bin, plan, images, visuals)?;
        }
        Ok(())
    }
    let mut visuals = vec![];
    for n in g
        .default_scene()
        .unwrap_or_else(|| g.scenes().next().unwrap())
        .nodes()
    {
        visit(n, Mat4::IDENTITY, bin, &plan, &images, &mut visuals)?;
    }
    if visuals.len() != model.meshes.len() {
        return Err("invalid: source primitive association".into());
    }
    for (m, v) in model.meshes.iter_mut().zip(visuals) {
        m.visual = Some(v);
    }
    model.source_sha256 = id.clone();
    let identity = AssetRef {
        closure: Some(Closure { id, images: refs }),
        format: "glb".into(),
        sha256: root_sha,
        byte_length: bytes.len(),
        name,
        width: None,
        height: None,
    };
    identity.validate()?;
    let a = Arc::new(Asset {
        model_images: images,
        dependencies,
        identity,
        bytes: bytes.into(),
        model,
        image: None,
        cpu_bytes: cpu,
        gpu_bytes: gpu,
        decode_us: start.elapsed().as_micros(),
    });
    let mut r = existing.clone();
    assets::insert(&mut r, a.clone())?;
    Ok(a)
}
pub fn import(path: &Path, bytes: Vec<u8>, existing: &Registry) -> Result<Arc<Asset>, String> {
    assets::regular_path(path, false)?;
    let root = path.parent().ok_or("invalid: model root")?;
    decode(
        bytes,
        path.file_name().map(|s| s.to_string_lossy().into_owned()),
        existing,
        |u| assets::read_blob(&root.join(u)),
    )
}
pub fn load_declared(path: &Path, a: &AssetRef, existing: &Registry) -> Result<Arc<Asset>, String> {
    a.validate()?;
    let bytes = assets::read_blob(path)?;
    if bytes.len() != a.byte_length || sha(&bytes) != a.sha256 {
        return Err("conflict: model root identity".into());
    }
    let closure = a.closure.as_ref().ok_or("invalid: model closure missing")?;
    let root = path.parent().ok_or("invalid: assets root")?;
    let loaded = decode(bytes, a.name.clone(), existing, |u| {
        let edge = format!("uri:{u}");
        let image = closure
            .images
            .iter()
            .find(|i| i.source == edge)
            .ok_or("conflict: model dependency edge")?;
        let b = assets::read_blob(&root.join(format!("{}.png", image.sha256)))?;
        if b.len() != image.byte_length || sha(&b) != image.sha256 {
            return Err("conflict: model dependency identity".into());
        }
        Ok(b)
    })?;
    if loaded.identity != *a {
        return Err("conflict: model closure binding".into());
    }
    Ok(loaded)
}
