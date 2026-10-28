//! Finite viewport geometry and bounded gesture state; nothing here is persistent.
use super::{
    document::{Document, Kind, Object},
    preview::{Building, Preview},
    session::Stamp,
};
use agp::{assets::ModelData, core::composition::SurfaceId};
use glam::{DMat4, DVec3, DVec4, Mat4, Vec2, Vec3};

pub fn transform(o: &Object) -> Mat4 {
    Mat4::from_translation(o.translation.into())
        * Mat4::from_rotation_y(o.yaw)
        * Mat4::from_rotation_x(o.pitch)
        * Mat4::from_scale(o.scale.into())
}
pub fn bounds(model: &ModelData) -> Option<(Vec3, Vec3)> {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for p in model.meshes.iter().flat_map(|m| &m.positions) {
        let p = Vec3::from(*p);
        if !p.is_finite() {
            return None;
        }
        lo = lo.min(p);
        hi = hi.max(p);
    }
    (lo.is_finite() && hi.is_finite()).then_some((lo, hi))
}
pub fn inside(p: [f32; 2], size: [f32; 2]) -> bool {
    p.iter().chain(size.iter()).all(|x| x.is_finite())
        && size[0] > 0.0
        && size[1] > 0.0
        && p[0] >= 0.0
        && p[1] >= 0.0
        && p[0] < size[0]
        && p[1] < size[1]
}
pub fn pick2(objects: &[Object], p: [f32; 2]) -> Option<u64> {
    pick2_view(objects, p, super::view2d::View2d::default())
}
pub fn pick2_view(objects: &[Object], p: [f32; 2], view: super::view2d::View2d) -> Option<u64> {
    if !inside(p, [800.0, 380.0]) {
        return None;
    }
    let world = view.world(Vec2::from(p))?;
    objects
        .iter()
        .rev()
        .find(|o| {
            if !o.visible || !o.kind.is_2d() || o.rgba[3] <= 0.0 {
                return false;
            }
            let d = world - Vec2::new(o.translation[0], o.translation[1]);
            let q = d / Vec2::new(o.scale[0] * 0.5, o.scale[1] * 0.5);
            if o.kind == Kind::Circle {
                q.length_squared() <= 1.0
            } else {
                q.abs().max_element() <= 1.0
            }
        })
        .map(|o| o.id)
}
pub fn project(vp: Mat4, p: Vec3) -> Option<Vec2> {
    let c = vp * p.extend(1.0);
    if !c.is_finite() || c.w <= 1e-6 {
        return None;
    }
    let n = c.truncate() / c.w;
    if n.z < 0.0 || n.z > 1.0 {
        return None;
    }
    Some(Vec2::new((n.x + 1.0) * 400.0, (1.0 - n.y) * 200.0))
}
/// Project a world OBB after clipping its twelve edges to the camera depth slab.
/// A box crossing a clip plane remains visible even if every original corner
/// lies outside it; fully clipped boxes contribute no group handles.
pub fn clipped_box_points(vp: Mat4, model: Mat4, lo: Vec3, hi: Vec3) -> Vec<Vec2> {
    let corners: Vec<_> = (0..8)
        .map(|i| {
            let p = Vec3::new(
                if i & 1 == 0 { lo.x } else { hi.x },
                if i & 2 == 0 { lo.y } else { hi.y },
                if i & 4 == 0 { lo.z } else { hi.z },
            );
            vp * model.transform_point3(p).extend(1.0)
        })
        .collect();
    if corners.iter().any(|c| !c.is_finite()) {
        return Vec::new();
    }
    let mut points = Vec::new();
    for i in 0..8 {
        for bit in [1, 2, 4] {
            let j = i ^ bit;
            if j <= i {
                continue;
            }
            let (a, b) = (corners[i], corners[j]);
            let mut first = 0.0f32;
            let mut last = 1.0f32;
            for (fa, fb) in [(a.z, b.z), (a.w - a.z, b.w - b.z), (a.w - 1e-6, b.w - 1e-6)] {
                if fa < 0.0 && fb < 0.0 {
                    first = 1.0;
                    last = 0.0;
                    break;
                }
                if (fa < 0.0) != (fb < 0.0) {
                    let t = fa / (fa - fb);
                    if fa < 0.0 {
                        first = first.max(t);
                    } else {
                        last = last.min(t);
                    }
                }
            }
            if first <= last {
                for t in [first, last] {
                    let c = a.lerp(b, t);
                    if c.is_finite() && c.w > 0.0 {
                        points.push(Vec2::new(
                            (c.x / c.w + 1.0) * 400.0,
                            (1.0 - c.y / c.w) * 200.0,
                        ));
                    }
                }
            }
        }
    }
    points
}
pub fn pick3(objects: &[Object], models: &[ModelData], vp: Mat4, p: [f32; 2]) -> Option<u64> {
    pick3_with(objects, vp, p, |o| {
        models.get(match o.kind {
            Kind::Cylinder => 0,
            Kind::Player => 1,
            _ => 2,
        })
    })
}
pub fn pick3_with<'a>(
    objects: &[Object],
    vp: Mat4,
    p: [f32; 2],
    model: impl Fn(&Object) -> Option<&'a ModelData>,
) -> Option<u64> {
    if !inside(p, [800.0, 400.0]) {
        return None;
    }
    let inv = stable_inverse64(vp)?;
    let (a, b) = ray(inv, Vec2::from(p))?;
    let mut hits = vec![];
    for o in objects.iter().filter(|o| o.visible && !o.kind.is_2d()) {
        let model = model(o)?;
        let (lo, hi) = bounds(model)?;
        let t = transform(o);
        if !t.is_finite() || t.determinant().abs() < 1e-10 {
            continue;
        }
        let inv = DMat4::from_cols_array(&t.to_cols_array().map(f64::from)).inverse();
        if !inv.is_finite() {
            continue;
        }
        let origin = inv.transform_point3(a);
        let d = inv.transform_point3(b) - origin;
        let lo = lo.as_dvec3();
        let hi = hi.as_dvec3();
        let mut near = 0.0f64;
        let mut far = 1.0f64;
        for axis in 0..3 {
            if d[axis].abs() < 1e-8 {
                if origin[axis] < lo[axis] || origin[axis] > hi[axis] {
                    far = -1.0;
                    break;
                }
            } else {
                let t0 = (lo[axis] - origin[axis]) / d[axis];
                let t1 = (hi[axis] - origin[axis]) / d[axis];
                near = near.max(t0.min(t1));
                far = far.min(t0.max(t1));
            }
        }
        if near <= far && near.is_finite() {
            hits.push((near, o.id));
        }
    }
    hits.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    hits.first().map(|x| x.1)
}
/// Finite inverse with a scale-independent residual, so legitimate wide views
/// are not rejected merely because their determinant is small.
pub fn stable_inverse(vp: Mat4) -> Option<Mat4> {
    // Preserve the existing public f32 return type. Internal ray consumers
    // retain the full-precision inverse instead of using this rounded copy.
    let inv = stable_inverse64(vp)?;
    let rounded = Mat4::from_cols_array(&inv.to_cols_array().map(|v| v as f32));
    rounded.is_finite().then_some(rounded)
}
fn stable_inverse64(vp: Mat4) -> Option<DMat4> {
    if !vp.is_finite() {
        return None;
    }
    // Invert the actual presented f32 matrix in f64; do not round its inverse
    // back to f32 before unprojecting a long perspective ray.
    let matrix = DMat4::from_cols_array(&vp.to_cols_array().map(f64::from));
    let inv = matrix.inverse();
    if !inv.is_finite() {
        return None;
    }
    let residual = (matrix * inv - DMat4::IDENTITY).to_cols_array();
    if residual.iter().any(|x| !x.is_finite() || x.abs() > 1e-7) {
        return None;
    }
    Some(inv)
}
fn ray(inv: DMat4, p: Vec2) -> Option<(DVec3, DVec3)> {
    if !inside(p.to_array(), [800.0, 400.0]) {
        return None;
    }
    let x = f64::from(p.x) / 400.0 - 1.0;
    let y = 1.0 - f64::from(p.y) / 200.0;
    let unproject = |z| {
        let c = inv * DVec4::new(x, y, z, 1.0);
        if !c.is_finite() || c.w.abs() < 1e-12 {
            None
        } else {
            Some(c.truncate() / c.w)
        }
    };
    let a = unproject(0.0)?;
    let b = unproject(1.0)?;
    if !a.is_finite() || !b.is_finite() || (b - a).length_squared() < 1e-12 {
        return None;
    }
    Some((a, b))
}
/// Frozen axis plane through the presented world origin; sample failures keep the last accepted preview.
#[derive(Clone, Copy)]
pub struct AxisDrag {
    inv: DMat4,
    origin: DVec3,
    axis: DVec3,
    normal: DVec3,
    start: f64,
}
impl AxisDrag {
    pub fn new(vp: Mat4, origin: Vec3, axis: usize, p: Vec2) -> Option<Self> {
        if axis >= 3 || !origin.is_finite() {
            return None;
        }
        let inv = stable_inverse64(vp)?;
        let (a, b) = ray(inv, Vec2::new(400.0, 200.0))?;
        let forward = (b - a).try_normalize()?;
        let mut direction = DVec3::ZERO;
        direction[axis] = 1.0;
        let normal = (forward - direction * forward.dot(direction)).try_normalize()?;
        if (forward - direction * forward.dot(direction)).length() < 1e-3 {
            return None;
        }
        let mut drag = Self {
            inv,
            origin: origin.as_dvec3(),
            axis: direction,
            normal,
            start: 0.0,
        };
        drag.start = drag.coordinate(p)?;
        Some(drag)
    }
    fn coordinate(self, p: Vec2) -> Option<f64> {
        let (a, b) = ray(self.inv, p)?;
        let d = b - a;
        let denom = d.dot(self.normal);
        if denom.abs() < 1e-6 * d.length() {
            return None;
        }
        let t = (self.origin - a).dot(self.normal) / denom;
        if !t.is_finite() || !(0.0..=1.0).contains(&t) {
            return None;
        }
        let value = (a + d * t - self.origin).dot(self.axis);
        value.is_finite().then_some(value)
    }
    pub fn delta(self, p: Vec2) -> Option<f32> {
        let delta = (self.coordinate(p)? - self.start) as f32;
        delta.is_finite().then_some(delta)
    }
}
// Handles have a 54 px visual length, 7 px hit radius, and deterministic X/Y/Z tie order.
pub fn axes(o: &Object, vp: Mat4) -> Vec<(usize, Vec2, Vec2)> {
    let origin = Vec3::from(o.translation);
    let Some(a) = project(vp, origin) else {
        return vec![];
    };
    if !inside(a.to_array(), [800.0, 400.0]) {
        return vec![];
    }
    (0..3)
        .filter_map(|axis| {
            let mut unit = Vec3::ZERO;
            unit[axis] = 1.0;
            // A world unit may be subpixel at a wide yet valid view. Extend
            // the same world axis until its screen direction is measurable;
            // retain the original one-unit result at ordinary scales.
            for distance in [1.0, 10.0, 100.0, 1000.0, 10000.0] {
                for sign in [1.0, -1.0] {
                    if let Some(b) = project(vp, origin + unit * distance * sign) {
                        let delta = (b - a) * sign;
                        if delta.length() >= 1.0 {
                            return Some((axis, a, delta));
                        }
                    }
                }
            }
            None
        })
        .collect()
}
pub fn handle(o: &Object, vp: Mat4, p: [f32; 2]) -> Option<(usize, Vec2)> {
    let p = Vec2::from(p);
    let mut hits = vec![];
    for (axis, a, d) in axes(o, vp) {
        let line = d.normalize() * 54.0;
        let t = ((p - a).dot(line) / line.length_squared()).clamp(0.2, 1.0);
        let dist = (p - a - line * t).length();
        if dist <= 7.0 {
            hits.push((dist, axis, d));
        }
    }
    hits.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    hits.first().map(|x| (x.1, x.2))
}
pub struct Gesture {
    pub view2d: super::view2d::View2d,
    pub world_origin: Vec2,
    pub world_origin3d: Vec3,
    pub view3d: super::view3d::View3d,
    pub axis_drag: Option<AxisDrag>,
    pub token: u64,
    pub stamp: Stamp,
    pub target: Object,
    pub surface: SurfaceId,
    pub start: [f32; 2],
    pub last_valid: [f32; 2],
    pub axis: Option<(usize, Vec2)>,
    pub vp: Mat4,
    pub latest: Option<(u64, [f32; 2])>,
    pub next_sample: u64,
    pub shown_sample: u64,
    pub inflight: Option<(u64, Document, Building)>,
    pub shown: Option<Preview>,
    pub released: bool,
    pub release_time: Option<std::time::Instant>,
    pub moved: bool,
}
impl Gesture {
    pub fn sample(&mut self, p: [f32; 2]) -> bool {
        if self
            .axis_drag
            .is_some_and(|drag| drag.delta(Vec2::from(p)).is_none())
        {
            return false;
        }
        self.last_valid = p;
        self.next_sample += 1;
        self.latest = Some((self.next_sample, p));
        if (Vec2::from(p) - Vec2::from(self.start)).length() >= 3.0 {
            self.moved = true
        }
        true
    }
    pub fn document(&self, accepted: &Document, p: [f32; 2]) -> Result<Document, String> {
        let mut d = accepted.clone();
        let mut translation = self.target.translation;
        let delta = Vec2::from(p) - Vec2::from(self.start);
        if let Some((axis, _)) = self.axis {
            let drag = self.axis_drag.ok_or("degenerate 3D drag plane")?;
            let world = self.world_origin3d[axis]
                + drag.delta(Vec2::from(p)).ok_or("invalid 3D drag sample")?;
            translation[axis] += self.view3d.snapped_axis(world) - self.world_origin3d[axis];
        } else {
            let world = self
                .view2d
                .snapped(self.world_origin + delta * Vec2::new(1.0, -1.0) / self.view2d.zoom);
            let delta = world - self.world_origin;
            translation[0] += delta.x;
            translation[1] += delta.y;
        }
        if let Some(o) = d.objects.iter_mut().find(|o| o.id == self.target.id) {
            o.translation = translation;
        } else if let Some(g) = d.groups.iter_mut().find(|g| g.id == self.target.id) {
            g.translation = translation;
        } else {
            return Err("gesture target".into());
        }
        d.validate()?;
        Ok(d)
    }
}
