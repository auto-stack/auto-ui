//! Finite, editing-only 3D view. Document and runtime game state never own it.
use agp::render3d::{editor_grid::EditorGridVertex, Camera3d, Projection};
use glam::{Vec2, Vec3};

const FOV: f32 = 60.0;
const HALF_FOV_TAN: f32 = 0.577_350_26;
const SIZE: Vec2 = Vec2::new(800.0, 400.0);
const LIMIT: f32 = 65536.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Orthographic,
    Perspective,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View3d {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub ortho_height: f32,
    pub mode: Mode,
    pub grid: bool,
    pub snap: bool,
    pub step: f32,
    pub revision: u64,
    /// Exact original sample camera until a navigation/projection change.
    pub sample_default: bool,
}
impl Default for View3d {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.183_514, 0.0, 0.0),
            yaw: 0.0,
            pitch: std::f32::consts::FRAC_PI_4,
            distance: 19.0,
            ortho_height: 24.0,
            mode: Mode::Orthographic,
            grid: false,
            snap: false,
            step: 10.0,
            revision: 0,
            sample_default: true,
        }
    }
}
impl View3d {
    pub fn valid(self) -> bool {
        self.target.is_finite()
            && self.target.abs().max_element() <= LIMIT
            && self.yaw.is_finite()
            && self.pitch.is_finite()
            && self.pitch.abs() <= 85f32.to_radians()
            && self.distance.is_finite()
            && (0.5..=4096.0).contains(&self.distance)
            && self.ortho_height.is_finite()
            && (0.125..=8192.0).contains(&self.ortho_height)
            && super::view2d::STEPS.contains(&self.step)
            && (!self.sample_default
                || (self.target == Vec3::new(0.183_514, 0.0, 0.0)
                    && self.yaw == 0.0
                    && self.pitch == std::f32::consts::FRAC_PI_4
                    && self.distance == 19.0
                    && self.ortho_height == 24.0
                    && self.mode == Mode::Orthographic))
    }
    pub fn camera(self) -> Result<Camera3d, String> {
        if !self.valid() {
            return Err("invalid 3D view".into());
        }
        if self.sample_default {
            return Ok(Camera3d::sample_camera(2.0));
        }
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let offset = Vec3::new(sy * cp, sp, cy * cp) * self.distance;
        let projection = match self.mode {
            Mode::Orthographic => Projection::Orthographic {
                size: self.ortho_height,
                aspect: 2.0,
                near: 0.05,
                far: 16384.0,
            },
            Mode::Perspective => Projection::Perspective {
                fov_y_deg: FOV,
                aspect: 2.0,
                near: 0.05,
                far: 16384.0,
            },
        };
        Ok(Camera3d {
            eye: self.target + offset,
            target: self.target,
            up: Vec3::Y,
            projection,
        })
    }
    pub fn orbit(self, delta: Vec2) -> Result<Self, String> {
        if !self.valid() || !delta.is_finite() {
            return Err("invalid 3D orbit".into());
        }
        if delta == Vec2::ZERO {
            return Ok(self);
        }
        let yaw = (self.yaw - delta.x / SIZE.x * std::f32::consts::TAU + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let pitch = (self.pitch + delta.y / SIZE.y * std::f32::consts::PI)
            .clamp(-85f32.to_radians(), 85f32.to_radians());
        Ok(Self {
            yaw,
            pitch,
            sample_default: false,
            ..self
        })
    }
    pub fn visible_height(self) -> f32 {
        match self.mode {
            Mode::Orthographic => self.ortho_height,
            Mode::Perspective => 2.0 * self.distance * HALF_FOV_TAN,
        }
    }
    pub fn pan(self, delta: Vec2) -> Result<Self, String> {
        if !self.valid() || !delta.is_finite() {
            return Err("invalid 3D pan".into());
        }
        if delta == Vec2::ZERO {
            return Ok(self);
        }
        let camera = self.camera()?;
        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward).normalize();
        let target = (self.target
            + (-right * delta.x + up * delta.y) * (self.visible_height() / SIZE.y))
            .clamp(Vec3::splat(-LIMIT), Vec3::splat(LIMIT));
        if !target.is_finite() {
            return Err("invalid 3D pan result".into());
        }
        Ok(Self {
            target,
            sample_default: false,
            ..self
        })
    }
    pub fn wheel(self, notches: f32) -> Result<Self, String> {
        if !self.valid() || !notches.is_finite() {
            return Err("invalid 3D wheel".into());
        }
        if notches == 0.0 {
            return Ok(self);
        }
        let factor = 1.1f32.powf(-notches.clamp(-8.0, 8.0));
        Ok(match self.mode {
            Mode::Orthographic => Self {
                ortho_height: (self.ortho_height * factor).clamp(0.125, 8192.0),
                sample_default: false,
                ..self
            },
            Mode::Perspective => Self {
                distance: (self.distance * factor).clamp(0.5, 4096.0),
                sample_default: false,
                ..self
            },
        })
    }
    pub fn toggle_projection(self) -> Result<(Self, bool), String> {
        if !self.valid() {
            return Err("invalid 3D projection".into());
        }
        match self.mode {
            Mode::Orthographic => {
                let desired = self.ortho_height / (2.0 * HALF_FOV_TAN);
                let d = desired.clamp(0.5, 4096.0);
                Ok((
                    Self {
                        mode: Mode::Perspective,
                        distance: d,
                        sample_default: false,
                        ..self
                    },
                    d != desired,
                ))
            }
            Mode::Perspective => {
                let desired = 2.0 * self.distance * HALF_FOV_TAN;
                let h = desired.clamp(0.125, 8192.0);
                Ok((
                    Self {
                        mode: Mode::Orthographic,
                        ortho_height: h,
                        sample_default: false,
                        ..self
                    },
                    h != desired,
                ))
            }
        }
    }
    /// Frame all corners using actual camera axes and at least 20% screen margin.
    pub fn focus(self, lo: Vec3, hi: Vec3) -> Result<Self, String> {
        if !self.valid() || !lo.is_finite() || !hi.is_finite() || (hi - lo).min_element() < 0.0 {
            return Err("no finite visible 3D bounds".into());
        }
        let center = (lo.as_dvec3() * 0.5 + hi.as_dvec3() * 0.5).as_vec3();
        if !center.is_finite() || center.abs().max_element() > LIMIT {
            return Err("selection exceeds supported 3D target range".into());
        }
        let camera = self.camera()?;
        let backward = (camera.eye - camera.target).normalize();
        let right = Vec3::Y.cross(backward).normalize();
        let up = backward.cross(right).normalize();
        let mut height = 0.125f32;
        let mut distance = 0.5f32;
        let mut near_distance = 0.5f32;
        let mut farthest = 0.0f32;
        for i in 0..8 {
            let point = Vec3::new(
                if i & 1 == 0 { lo.x } else { hi.x },
                if i & 2 == 0 { lo.y } else { hi.y },
                if i & 4 == 0 { lo.z } else { hi.z },
            );
            let d = point - center;
            let x = d.dot(right).abs();
            let y = d.dot(up).abs();
            let z = d.dot(backward);
            height = height.max((2.0 * y / 0.8).max(x / 0.8));
            distance = distance
                .max(z + 0.05 + y / (0.8 * HALF_FOV_TAN))
                .max(z + 0.05 + x / (1.6 * HALF_FOV_TAN));
            near_distance = near_distance.max(z + 0.05);
            farthest = farthest.max(-z);
        }
        let distance = match self.mode {
            Mode::Perspective => distance.max(0.5),
            Mode::Orthographic => self.distance.max(near_distance),
        };
        if !height.is_finite()
            || !distance.is_finite()
            || height > 8192.0
            || distance > 4096.0
            || distance + farthest > 16384.0
        {
            return Err("selection cannot be framed within 3D view limits".into());
        }
        Ok(Self {
            target: center,
            distance,
            ortho_height: height.max(0.125),
            sample_default: false,
            ..self
        })
    }
    pub fn snapped_axis(self, world: f32) -> f32 {
        if self.snap {
            (world / self.step).round() * self.step
        } else {
            world
        }
    }
    /// World-aligned finite XZ lines; never consumes scene instances or shadow capacity.
    pub fn grid_vertices(self) -> Result<(f32, Vec<EditorGridVertex>), String> {
        if !self.valid() {
            return Err("invalid 3D grid view".into());
        }
        if !self.grid {
            return Ok((0.0, vec![]));
        }
        let units_per_pixel = self.visible_height() / SIZE.y;
        let radius = (self.visible_height() * 1.5).clamp(1.0, 4096.0);
        let mut spacing = 0.001f32;
        'choice: for exponent in -3..=8 {
            for factor in [1.0, 2.0, 5.0] {
                let s = factor * 10f32.powi(exponent);
                if s / units_per_pixel >= 16.0 {
                    spacing = s;
                    break 'choice;
                }
            }
        }
        loop {
            let nx = ((2.0 * radius / spacing).ceil() as usize + 2) * 2 + 2;
            if nx <= 128 {
                break;
            }
            let mag = 10f32.powf(spacing.log10().floor());
            let f = spacing / mag;
            spacing = if f < 1.5 {
                2.0 * mag
            } else if f < 3.5 {
                5.0 * mag
            } else {
                10.0 * mag
            };
        }
        let lo = Vec2::new(self.target.x - radius, self.target.z - radius);
        let hi = Vec2::new(self.target.x + radius, self.target.z + radius);
        let mut out = Vec::with_capacity(256);
        for axis in 0..2 {
            let first = (lo[axis] as f64 / spacing as f64).ceil() as i64;
            let last = (hi[axis] as f64 / spacing as f64).floor() as i64;
            for i in first..=last {
                let v = i as f32 * spacing;
                let color = if i == 0 {
                    if axis == 0 {
                        [0.2, 0.45, 0.95, 1.0]
                    } else {
                        [0.95, 0.25, 0.2, 1.0]
                    }
                } else if i % 5 == 0 {
                    [0.35, 0.38, 0.42, 1.0]
                } else {
                    [0.20, 0.22, 0.25, 1.0]
                };
                let (a, b) = if axis == 0 {
                    ([v, 0.0, lo.y], [v, 0.0, hi.y])
                } else {
                    ([lo.x, 0.0, v], [hi.x, 0.0, v])
                };
                out.push(EditorGridVertex { position: a, color });
                out.push(EditorGridVertex { position: b, color });
            }
        }
        if out.len() > 256 {
            return Err("3D grid line capacity".into());
        }
        Ok((spacing, out))
    }
}
