//! Bounded, immutable node TRS clips. No gameplay, IO, render clock or GPU state.
//! Callers admit the enclosing GLB/geometry/texture closure separately and charge
//! `cpu_bytes` to their existing asset union before retaining the payload.
use glam::{Mat4, Quat, Vec3, Vec4};
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};

static CHANNEL_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
/// Admission diagnostic: actual pairs of key/output vector allocations.
/// Metadata preflight, rejected union gates and sampling never increment this.
pub fn channel_allocation_count() -> usize {
    CHANNEL_ALLOCATIONS.load(Ordering::Relaxed)
}

pub const MAX_NODES: usize = 128;
pub const MAX_CLIPS: usize = 16;
pub const MAX_CHANNELS: usize = 128;
pub const MAX_SAMPLES: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Step,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Translation,
    Rotation,
    Scale,
}
#[derive(Clone, Debug)]
pub struct Node {
    pub parent: Option<usize>,
    pub matrix: Option<Mat4>,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
#[derive(Clone, Debug)]
pub struct Channel {
    pub node: usize,
    pub path: Path,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    // VEC3 outputs have canonical w=0; quaternions retain all four components.
    pub values: Vec<Vec4>,
}
#[derive(Clone, Debug)]
pub struct Clip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<Channel>,
}
#[derive(Clone, Debug)]
pub struct Animation {
    pub nodes: Vec<Node>,
    pub clips: Vec<Clip>,
    order: Vec<usize>,
    /// Conservative retained payload + per-sample pose scratch for one instance.
    /// Counts vector element storage, names, topology and all TRS/matrix scratch.
    pub cpu_bytes: usize,
}
fn invalid(s: &str) -> String {
    format!("invalid: node animation {s}")
}
fn limit(s: &str) -> String {
    format!("resource limit: node animation {s}")
}
fn unsupported(s: &str) -> String {
    format!("unsupported: node animation {s}")
}
fn interpolation(s: &Value) -> Result<Interpolation, String> {
    match s
        .get("interpolation")
        .map(|v| v.as_str().ok_or_else(|| invalid("interpolation type")))
        .transpose()?
        .unwrap_or("LINEAR")
    {
        "LINEAR" => Ok(Interpolation::Linear),
        "STEP" => Ok(Interpolation::Step),
        _ => Err(unsupported("interpolation")),
    }
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a [Value], String> {
    match v.get(key) {
        None => Ok(&[]),
        Some(Value::Array(a)) => Ok(a),
        _ => Err(invalid(key)),
    }
}
fn index(v: &Value) -> Result<usize, String> {
    v.as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("index/count"))
}
fn number(v: &Value) -> Result<f32, String> {
    v.as_f64()
        .map(|v| v as f32)
        .filter(|v| v.is_finite())
        .ok_or_else(|| invalid("finite float"))
}
fn components<const N: usize>(v: Option<&Value>, default: [f32; N]) -> Result<[f32; N], String> {
    let Some(v) = v else { return Ok(default) };
    let a = v
        .as_array()
        .filter(|a| a.len() == N)
        .ok_or_else(|| invalid("TRS shape"))?;
    let mut result = default;
    for (o, v) in result.iter_mut().zip(a) {
        *o = number(v)?;
    }
    Ok(result)
}
fn unit(q: Quat) -> Result<Quat, String> {
    if !q.is_finite() || (q.length_squared() - 1.).abs() > 1e-4 {
        return Err(invalid("unit quaternion"));
    }
    Ok(q.normalize())
}
fn positive(s: Vec3) -> Result<Vec3, String> {
    if !s.is_finite() || s.min_element() <= 0. {
        return Err(invalid("positive scale"));
    }
    Ok(s)
}
#[derive(Clone, Copy)]
struct Access<'a> {
    bytes: &'a [u8],
    offset: usize,
    stride: usize,
    count: usize,
    width: usize,
}
impl Access<'_> {
    fn row(&self, i: usize) -> Result<Vec4, String> {
        let mut result = [0.; 4];
        for (k, v) in result.iter_mut().enumerate().take(self.width) {
            let at = self.offset + i * self.stride + k * 4;
            *v = f32::from_le_bytes(self.bytes[at..at + 4].try_into().unwrap());
            if !v.is_finite() {
                return Err(invalid("finite key/output"));
            }
        }
        Ok(Vec4::from_array(result))
    }
}
// Span, type and count validation precedes any key/output vector allocation.
fn access<'a>(
    j: &Value,
    bin: &'a [u8],
    id: &Value,
    ty: &str,
    width: usize,
) -> Result<Access<'a>, String> {
    let a = array(j, "accessors")?
        .get(index(id)?)
        .ok_or_else(|| invalid("accessor index"))?;
    if a.get("sparse").is_some() {
        return Err(unsupported("sparse accessor"));
    }
    if a["componentType"] != 5126
        || a["type"] != ty
        || a.get("normalized").is_some_and(|v| v != false)
    {
        return Err(invalid("accessor type/normalization"));
    }
    let count = index(&a["count"])?;
    if count == 0 {
        return Err(invalid("empty accessor"));
    }
    let view = array(j, "bufferViews")?
        .get(index(&a["bufferView"])?)
        .ok_or_else(|| invalid("bufferView index"))?;
    if index(&view["buffer"])? != 0 {
        return Err(unsupported("external animation buffer"));
    }
    let start = view.get("byteOffset").map(index).transpose()?.unwrap_or(0);
    let length = index(&view["byteLength"])?;
    let offset = a.get("byteOffset").map(index).transpose()?.unwrap_or(0);
    let stride = view
        .get("byteStride")
        .map(index)
        .transpose()?
        .unwrap_or(width * 4);
    if start % 4 != 0 || offset % 4 != 0 || stride % 4 != 0 || stride < width * 4 || stride > 252 {
        return Err(invalid("accessor alignment/stride"));
    }
    let span = (count - 1)
        .checked_mul(stride)
        .and_then(|n| n.checked_add(width * 4))
        .and_then(|n| n.checked_add(offset))
        .ok_or_else(|| invalid("accessor span overflow"))?;
    if span > length || start.checked_add(length).is_none_or(|n| n > bin.len()) {
        return Err(invalid("accessor span"));
    }
    Ok(Access {
        bytes: bin,
        offset: start + offset,
        stride,
        count,
        width,
    })
}
struct Track<'a> {
    node: usize,
    path: Path,
    interpolation: Interpolation,
    input: Access<'a>,
    output: Access<'a>,
}
fn sampler_key(
    input: &Access<'_>,
    output: &Access<'_>,
    i: usize,
    previous: f32,
) -> Result<(f32, Vec4), String> {
    let time = input.row(i)?.x;
    if time < 0. || time <= previous {
        return Err(invalid("strict nonnegative key times"));
    }
    Ok((time, output.row(i)?))
}
fn key(t: &Track<'_>, i: usize, previous: f32) -> Result<(f32, Vec4), String> {
    let (time, mut value) = sampler_key(&t.input, &t.output, i, previous)?;
    match t.path {
        Path::Rotation => {
            value = Vec4::from_array(unit(Quat::from_array(value.to_array()))?.to_array())
        }
        Path::Scale => {
            positive(value.truncate())?;
        }
        Path::Translation => {}
    }
    Ok((time, value))
}
impl Animation {
    /// No path access or decoding of images/geometry. `j` is the original GLB
    /// JSON and `bin` its bounded original BIN. Every declared clip is inspected.
    pub fn from_parts(j: &Value, bin: &[u8]) -> Result<Self, String> {
        Self::build(j, bin, true)?
            .0
            .ok_or_else(|| invalid("retained payload"))
    }
    /// All declared track types/spans/counts/values, before allocating any key
    /// or output payload. The enclosing asset gate must charge this exact size.
    pub fn preflight(j: &Value, bin: &[u8]) -> Result<usize, String> {
        Ok(Self::build(j, bin, false)?.1)
    }
    fn build(j: &Value, bin: &[u8], retain: bool) -> Result<(Option<Self>, usize), String> {
        let declared = array(j, "nodes")?;
        if declared.len() > MAX_NODES {
            return Err(limit("nodes128"));
        }
        if !array(j, "skins")?.is_empty() || declared.iter().any(|n| n.get("skin").is_some()) {
            return Err(unsupported("skin"));
        }
        if array(j, "meshes")?.iter().any(|m| {
            m.get("weights").is_some()
                || array(m, "primitives")
                    .is_ok_and(|p| p.iter().any(|p| p.get("targets").is_some()))
        }) || declared.iter().any(|n| n.get("weights").is_some())
        {
            return Err(unsupported("morph/weights"));
        }
        let clips = array(j, "animations")?;
        if clips.len() > MAX_CLIPS {
            return Err(limit("clips16"));
        }
        // First validate all metadata, buffer spans and aggregate counts. The
        // only allocation before this finishes is bounded topology/track state.
        let mut parents = vec![None; declared.len()];
        for (i, n) in declared.iter().enumerate() {
            for c in array(n, "children")? {
                let c = index(c)?;
                if c >= declared.len() || c == i || parents[c].replace(i).is_some() {
                    return Err(invalid("unique parent"));
                }
            }
        }
        let mut order = Vec::with_capacity(declared.len());
        let mut done = vec![false; declared.len()];
        while order.len() < declared.len() {
            let before = order.len();
            for (i, p) in parents.iter().enumerate() {
                if !done[i] && p.is_none_or(|p| done[p]) {
                    done[i] = true;
                    order.push(i);
                }
            }
            if order.len() == before {
                return Err(invalid("cyclic hierarchy"));
            }
        }
        let mut plans = Vec::with_capacity(clips.len());
        // Metadata views only: unused declarations must pass the same common
        // admission, but are never retained as tracks or charged as channels.
        let mut declared_samplers = Vec::new();
        let (mut total_channels, mut total_samples) = (0usize, 0usize);
        for clip in clips {
            let channels = array(clip, "channels")?;
            if channels.is_empty() {
                return Err(invalid("empty clip channels"));
            }
            total_channels = total_channels
                .checked_add(channels.len())
                .ok_or_else(|| limit("channels128"))?;
            if total_channels > MAX_CHANNELS {
                return Err(limit("channels128"));
            }
            let name = clip
                .get("name")
                .map(|n| n.as_str().ok_or_else(|| invalid("clip name")))
                .transpose()?
                .unwrap_or("");
            if name.len() > 256 {
                return Err(limit("clip name256"));
            }
            let samplers = array(clip, "samplers")?
                .iter()
                .map(|s| {
                    let mode = interpolation(s)?;
                    let input = access(j, bin, &s["input"], "SCALAR", 1)?;
                    let output_id = index(&s["output"])?;
                    let output_decl = array(j, "accessors")?
                        .get(output_id)
                        .ok_or_else(|| invalid("accessor index"))?;
                    let (ty, width) = match output_decl["type"].as_str() {
                        Some("VEC3") => ("VEC3", 3),
                        Some("VEC4") => ("VEC4", 4),
                        _ => return Err(invalid("sampler TRS output type")),
                    };
                    let output = access(j, bin, &s["output"], ty, width)?;
                    if input.count != output.count {
                        return Err(invalid("input/output count"));
                    }
                    Ok((mode, input, output))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let mut tracks = Vec::with_capacity(channels.len());
            let mut seen = BTreeSet::new();
            for c in channels {
                let node = index(&c["target"]["node"])?;
                if node >= declared.len() {
                    return Err(invalid("target node"));
                }
                if declared[node].get("matrix").is_some() {
                    return Err(unsupported("animated matrix node"));
                }
                let (path, width) = match c["target"]["path"].as_str() {
                    Some("translation") => (Path::Translation, 3),
                    Some("rotation") => (Path::Rotation, 4),
                    Some("scale") => (Path::Scale, 3),
                    _ => return Err(unsupported("target path")),
                };
                if !seen.insert((node, c["target"]["path"].as_str().unwrap())) {
                    return Err(invalid("duplicate node/path"));
                }
                let &(interpolation, input, output) = samplers
                    .get(index(&c["sampler"])?)
                    .ok_or_else(|| invalid("sampler index"))?;
                if output.width != width {
                    return Err(invalid("target output dimension"));
                }
                total_samples = total_samples
                    .checked_add(input.count)
                    .ok_or_else(|| limit("samples8192"))?;
                if total_samples > MAX_SAMPLES {
                    return Err(limit("samples8192"));
                }
                tracks.push(Track {
                    node,
                    path,
                    interpolation,
                    input,
                    output,
                });
            }
            declared_samplers.extend(samplers);
            plans.push((name, tracks));
        }
        let mut nodes = Vec::with_capacity(declared.len());
        for (i, n) in declared.iter().enumerate() {
            // Preserve unanimated parent matrices exactly. glTF forbids matrix
            // targets and combining matrix with TRS; no lossy decomposition.
            let matrix = if n.get("matrix").is_some() {
                if ["translation", "rotation", "scale"]
                    .iter()
                    .any(|k| n.get(k).is_some())
                {
                    return Err(invalid("matrix and TRS"));
                }
                let m = Mat4::from_cols_array(&components(n.get("matrix"), [0.; 16])?);
                if m.x_axis.w != 0. || m.y_axis.w != 0. || m.z_axis.w != 0. || m.w_axis.w != 1. {
                    return Err(invalid("affine matrix"));
                }
                Some(m)
            } else {
                None
            };
            nodes.push(Node {
                parent: parents[i],
                matrix,
                translation: Vec3::from_array(components(n.get("translation"), [0.; 3])?),
                rotation: unit(Quat::from_array(components(
                    n.get("rotation"),
                    [0., 0., 0., 1.],
                )?))?,
                scale: positive(Vec3::from_array(components(n.get("scale"), [1.; 3])?))?,
            });
        }
        let cpu_bytes = std::mem::size_of::<Self>()
            + nodes.len() * (std::mem::size_of::<Node>() + std::mem::size_of::<usize>() + 48 + 64)
            + plans.len() * std::mem::size_of::<Clip>()
            + plans
                .iter()
                .map(|(name, tracks)| name.len() + tracks.len() * std::mem::size_of::<Channel>())
                .sum::<usize>()
            + total_samples * (4 + std::mem::size_of::<Vec4>());
        // Validate every declared sampler before any key/output allocation,
        // including unused samplers. Target-specific rotation/scale semantics
        // belong to channels, not unreferenced VEC3/VEC4 declarations.
        for (_, input, output) in &declared_samplers {
            let mut previous = -1.;
            for i in 0..input.count {
                previous = sampler_key(input, output, i, previous)?.0;
            }
        }
        for (_, tracks) in &plans {
            for track in tracks {
                let mut previous = -1.;
                for i in 0..track.input.count {
                    previous = key(track, i, previous)?.0;
                }
            }
        }
        if !retain {
            let base = Self {
                nodes,
                clips: Vec::new(),
                order,
                cpu_bytes,
            };
            base.sample(None, 0., false)?;
            return Ok((None, cpu_bytes));
        }
        let mut result = Vec::with_capacity(plans.len());
        for (name, tracks) in plans {
            let mut channels = Vec::with_capacity(tracks.len());
            let mut duration = 0.0f32;
            for t in tracks {
                CHANNEL_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
                let mut times = Vec::with_capacity(t.input.count);
                let mut values = Vec::with_capacity(t.output.count);
                let mut previous = -1.;
                for i in 0..t.input.count {
                    let (time, value) = key(&t, i, previous)?;
                    previous = time;
                    times.push(time);
                    values.push(value);
                }
                duration = duration.max(previous);
                channels.push(Channel {
                    node: t.node,
                    path: t.path,
                    interpolation: t.interpolation,
                    times,
                    values,
                });
            }
            result.push(Clip {
                name: name.to_owned(),
                duration,
                channels,
            });
        }
        let out = Self {
            nodes,
            clips: result,
            order,
            cpu_bytes,
        };
        out.sample(None, 0., false)?;
        Ok((Some(out), cpu_bytes))
    }
    /// Seconds supplied by Auto. None is the unanimated base pose; clip ordinal
    /// switches sample immediately. Exact loop end wraps to zero; one-shot holds.
    pub fn sample(
        &self,
        clip: Option<usize>,
        seconds: f32,
        looping: bool,
    ) -> Result<Vec<Mat4>, String> {
        if !seconds.is_finite() || seconds < 0. {
            return Err(invalid("phase"));
        }
        let mut local: Vec<_> = self
            .nodes
            .iter()
            .map(|n| (n.translation, n.rotation, n.scale))
            .collect();
        if let Some(id) = clip {
            let c = self.clips.get(id).ok_or_else(|| invalid("clip ordinal"))?;
            let time = if c.duration == 0. {
                0.
            } else if looping {
                seconds % c.duration
            } else {
                seconds.min(c.duration)
            };
            for ch in &c.channels {
                let next = ch.times.partition_point(|v| *v <= time);
                let lo = next.saturating_sub(1);
                let hi = next.min(ch.times.len() - 1);
                let a = ch.values[lo];
                let b = ch.values[hi];
                let factor =
                    if hi == lo || ch.interpolation == Interpolation::Step || time <= ch.times[0] {
                        0.
                    } else {
                        (time - ch.times[lo]) / (ch.times[hi] - ch.times[lo])
                    };
                let target = &mut local[ch.node];
                match ch.path {
                    // Convex form avoids overflowing b-a for opposite finite
                    // endpoints even when their correct interpolated pose fits.
                    Path::Translation => {
                        target.0 = a.truncate() * (1. - factor) + b.truncate() * factor
                    }
                    Path::Scale => target.2 = a.truncate() * (1. - factor) + b.truncate() * factor,
                    Path::Rotation => {
                        target.1 = Quat::from_array(a.to_array())
                            .slerp(Quat::from_array(b.to_array()), factor)
                            .normalize()
                    }
                }
            }
        }
        let mut worlds = vec![Mat4::IDENTITY; local.len()];
        for &i in &self.order {
            let (t, r, s) = local[i];
            let matrix = self.nodes[i]
                .matrix
                .unwrap_or_else(|| Mat4::from_scale_rotation_translation(s, r, t));
            positive(s)?;
            unit(r)?;
            let world = self.nodes[i].parent.map_or(matrix, |p| worlds[p] * matrix);
            if !world.is_finite()
                || !world.determinant().is_finite()
                || world.determinant() <= 0.
                || !world.inverse().is_finite()
            {
                return Err(invalid("finite invertible pose"));
            }
            worlds[i] = world;
        }
        Ok(worlds)
    }
}

/// Visual yaw has no collision side effect. The bound prevents accepting
/// arbitrary huge angles whose f32 remainder loses useful direction precision.
pub fn yaw(radians: f32) -> Result<f32, String> {
    if !radians.is_finite() || radians.abs() > std::f32::consts::TAU * 1024. {
        return Err(invalid("yaw"));
    }
    Ok((radians + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI)
}
