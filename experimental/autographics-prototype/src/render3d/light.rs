//! Neutral directional-light admission port (PLAN-080 T-02).
//!
//! T-02 freezes the public Light port, numeric domain, resource accounting
//! and GPU policy without re-running the 059 feasibility study. The port is
//! the *only* legal way to turn typed author parameters into a `LightDef`:
//! every numeric field is admitted before any GPU write; an invalid or
//! unsupported light is rejected with a classified error and the caller keeps
//! its last-good state (no partial upload, no NaN/Inf reaches the shader).
//!
//! Projection construction (two candidates compared, one frozen):
//! - `shadow_vp_look_at` — the frozen port. Matches `sample_light()`'s
//!   orthographic look-at convention and supports arbitrary 3D directions
//!   with a deterministic up-axis fallback when the direction is parallel
//!   to the common Y up.
//! - `shadow_vp_slanted` — the 059 study's x/z-plane projection, kept only
//!   as the comparison candidate; NOT the frozen port (it cannot express
//!   arbitrary 3D directions).
//!
//! Frozen support domain (from the current sample + 059 study):
//! direction finite & nonzero; color channels in [0,1]; energy in [0,1];
//! center finite; radius in (0, 32] world units; 0 <= near < far <= 160;
//! bias in [0, .01]; every matrix element finite. A second light is a
//! resource/unsupported rejection, never a silent replacement.

use crate::render3d::pipeline::LightDef;
use glam::{camera::rh::view::look_at_mat4, Mat4, Vec3};

/// Frozen numeric-domain bounds (T-02). See module docs for rationale.
pub const SHADOW_RADIUS_MAX: f32 = 32.0;
pub const SHADOW_NEAR_MIN: f32 = 0.0;
pub const SHADOW_FAR_MAX: f32 = 160.0;
pub const SHADOW_BIAS_MAX: f32 = 0.01;
pub const AMBIENT: f32 = 0.25;
/// Shadow camera eye distance from the scene center (matches sample_light).
const SHADOW_CAMERA_DISTANCE: f32 = 60.0;

/// Classified admission outcome. `Invalid` = NaN / bad type / non-finite;
/// `Unsupported` = out of the frozen support domain (second light, oversized
/// frustum, etc.). The two are never conflated with language syntax errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightError {
    Invalid(&'static str),
    Unsupported(&'static str),
}

impl std::fmt::Display for LightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LightError::Invalid(d) => write!(f, "invalid light: {d}"),
            LightError::Unsupported(d) => write!(f, "unsupported light: {d}"),
        }
    }
}

/// Typed author-facing light parameters. This is the frozen port; the driver
/// maps checked Node state onto it and `admit()` guards the GPU boundary.
#[derive(Debug, Clone, Copy)]
pub struct LightParams {
    /// Unit direction FROM the surface TOWARD the light. Finite and nonzero.
    pub dir_to_light: Vec3,
    /// RGB color, each channel in [0, 1].
    pub color: [f32; 3],
    /// Intensity in [0, 1].
    pub energy: f32,
    /// Hard shadow enabled (single 2048² Depth32Float map).
    pub shadow_enabled: bool,
    /// Shadow frustum center (finite).
    pub center: Vec3,
    /// Orthographic half-extent in world units, in (0, 32].
    pub radius: f32,
    /// Near plane, in [0, far).
    pub near: f32,
    /// Far plane, in (near, 160].
    pub far: f32,
    /// Shadow depth bias in [0, .01].
    pub bias: f32,
}

impl LightParams {
    /// Admit this light. On `Err`, nothing was written to the GPU and the
    /// caller's last-good light stays in effect.
    pub fn admit(&self) -> Result<LightDef, LightError> {
        let dir = self.admit_direction()?;
        self.admit_color()?;
        self.admit_energy()?;
        self.admit_center()?;
        self.admit_frustum()?;
        let shadow_vp = shadow_vp_look_at(dir, self.center, self.radius, self.near, self.far)?;
        Ok(LightDef {
            dir_to_light: dir,
            color: self.color,
            energy: self.energy,
            shadow_vp,
            shadow_bias: self.bias,
        })
    }

    fn admit_direction(&self) -> Result<Vec3, LightError> {
        let d = self.dir_to_light;
        if !d.is_finite() {
            return Err(LightError::Invalid("direction must be finite"));
        }
        let len_sq = d.length_squared();
        if !len_sq.is_finite() {
            return Err(LightError::Invalid("direction length overflow"));
        }
        if len_sq < f32::EPSILON * f32::EPSILON {
            return Err(LightError::Invalid("direction must be nonzero"));
        }
        let n = d.normalize();
        if !n.is_finite() {
            return Err(LightError::Invalid("direction normalization overflow"));
        }
        Ok(n)
    }

    fn admit_color(&self) -> Result<(), LightError> {
        for c in self.color {
            if !c.is_finite() {
                return Err(LightError::Invalid("color channel must be finite"));
            }
            if !(0.0..=1.0).contains(&c) {
                return Err(LightError::Unsupported("color channel out of [0,1]"));
            }
        }
        Ok(())
    }

    fn admit_energy(&self) -> Result<(), LightError> {
        if !self.energy.is_finite() {
            return Err(LightError::Invalid("energy must be finite"));
        }
        if !(0.0..=1.0).contains(&self.energy) {
            return Err(LightError::Unsupported("energy out of [0,1]"));
        }
        Ok(())
    }

    fn admit_center(&self) -> Result<(), LightError> {
        if !self.center.is_finite() {
            return Err(LightError::Invalid("shadow center must be finite"));
        }
        Ok(())
    }

    fn admit_frustum(&self) -> Result<(), LightError> {
        for x in [self.radius, self.near, self.far, self.bias] {
            if !x.is_finite() {
                return Err(LightError::Invalid("frustum/bias must be finite"));
            }
        }
        if !(0.0..=SHADOW_RADIUS_MAX).contains(&self.radius) || self.radius == 0.0 {
            return Err(LightError::Unsupported(
                "shadow radius out of (0, 32] world units",
            ));
        }
        if self.near < SHADOW_NEAR_MIN {
            return Err(LightError::Unsupported("shadow near < 0"));
        }
        if self.far > SHADOW_FAR_MAX {
            return Err(LightError::Unsupported("shadow far > 160"));
        }
        if self.far <= self.near {
            return Err(LightError::Unsupported("shadow far must exceed near"));
        }
        if self.bias < 0.0 || self.bias > SHADOW_BIAS_MAX {
            return Err(LightError::Unsupported("shadow bias out of [0, .01]"));
        }
        Ok(())
    }
}

/// Frozen projection: orthographic look-at along `dir` covering `radius`
/// around `center`, with near/far depth and a deterministic up fallback when
/// the direction is (anti-)parallel to the common Y up.
pub fn shadow_vp_look_at(
    dir: Vec3,
    center: Vec3,
    radius: f32,
    near: f32,
    far: f32,
) -> Result<Mat4, LightError> {
    let up = if dir.y.abs() > 0.999 {
        // Deterministic replacement up when the look direction is parallel to
        // the common up: Z keeps the frustum well-defined (no NaN cross).
        Vec3::Z
    } else {
        Vec3::Y
    };
    let eye = center + dir * SHADOW_CAMERA_DISTANCE;
    let view = look_at_mat4(eye, center, up);
    let proj = orthographic_rh(radius, near, far);
    let vp = proj * view;
    if !vp.is_finite() {
        return Err(LightError::Invalid("shadow view-projection overflow"));
    }
    Ok(vp)
}

/// Comparison candidate only (059 study x/z-plane convention); NOT frozen.
/// Kept so the T-02 decision is auditable against the study's numbers.
#[allow(dead_code)]
pub fn shadow_vp_slanted(
    dir: Vec3,
    center: Vec3,
    radius: f32,
    near: f32,
    far: f32,
) -> Result<Mat4, LightError> {
    if dir.y.abs() > f32::EPSILON {
        return Err(LightError::Unsupported(
            "slanted projection is x/z-plane only",
        ));
    }
    let slope = dir.x / dir.z.max(f32::EPSILON);
    let (az, bz) = if dir.z > 0.0 {
        (-1.0 / (far - near), far / (far - near))
    } else {
        (1.0 / (far - near), -near / (far - near))
    };
    let vp = Mat4::from_cols_array(&[
        1.0 / radius,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / radius,
        0.0,
        0.0,
        -slope / radius,
        0.0,
        az,
        0.0,
        center.x * slope / radius,
        0.0,
        bz,
        1.0,
    ]);
    if !vp.is_finite() {
        return Err(LightError::Invalid("slanted shadow view-projection overflow"));
    }
    Ok(vp)
}

/// Right-handed orthographic projection, z in [0,1] NDC (wgpu convention),
/// matching `sample_light()`'s matrix shape.
fn orthographic_rh(radius: f32, near: f32, far: f32) -> Mat4 {
    Mat4::from_cols_array(&[
        1.0 / radius,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / radius,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0 / (far - near),
        0.0,
        0.0,
        0.0,
        -near / (far - near),
        1.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> LightParams {
        LightParams {
            dir_to_light: Vec3::new(0.382_355, 0.897_258, 0.220_753),
            color: [1.0, 1.0, 1.0],
            energy: 0.5,
            shadow_enabled: true,
            center: Vec3::new(0.5, 0.0, 0.5),
            radius: 32.0,
            near: 1.0,
            far: 160.0,
            bias: 0.0005,
        }
    }

    #[test]
    fn frozen_domain_boundaries_admit() {
        let l = good().admit().unwrap();
        assert!(l.dir_to_light.is_finite());
        assert!(l.shadow_vp.is_finite());
        // radius lower bound (exclusive 0) admits the smallest positive value.
        let mut small = good();
        small.radius = f32::MIN_POSITIVE;
        assert!(small.admit().is_ok());
        // radius upper bound admits exactly 32.
        let mut max = good();
        max.radius = SHADOW_RADIUS_MAX;
        assert!(max.admit().is_ok());
        // far upper bound admits exactly 160; near 0 with far>0 admits.
        let mut deep = good();
        deep.near = 0.0;
        deep.far = SHADOW_FAR_MAX;
        assert!(deep.admit().is_ok());
        // bias upper bound admits exactly .01.
        let mut bias = good();
        bias.bias = SHADOW_BIAS_MAX;
        assert!(bias.admit().is_ok());
    }

    #[test]
    fn invalid_domain_rejected_before_gpu() {
        // Zero / NaN / Inf directions.
        let mut zero = good();
        zero.dir_to_light = Vec3::ZERO;
        assert!(matches!(zero.admit(), Err(LightError::Invalid("direction must be nonzero"))));
        let mut nan = good();
        nan.dir_to_light = Vec3::new(f32::NAN, 0.0, 1.0);
        assert!(matches!(nan.admit(), Err(LightError::Invalid("direction must be finite"))));
        let mut inf = good();
        inf.dir_to_light = Vec3::new(f32::INFINITY, 0.0, 1.0);
        assert!(matches!(inf.admit(), Err(LightError::Invalid("direction must be finite"))));
        // Non-finite energy, negative/oversized energy.
        let mut e = good();
        e.energy = f32::NAN;
        assert!(matches!(e.admit(), Err(LightError::Invalid("energy must be finite"))));
        e.energy = -0.1;
        assert!(matches!(e.admit(), Err(LightError::Unsupported("energy out of [0,1]"))));
        e.energy = 1.5;
        assert!(matches!(e.admit(), Err(LightError::Unsupported("energy out of [0,1]"))));
        // Color channel out of range / NaN.
        let mut c = good();
        c.color = [1.0, 1.0, 2.0];
        assert!(matches!(
            c.admit(),
            Err(LightError::Unsupported("color channel out of [0,1]"))
        ));
        c.color = [1.0, f32::NAN, 1.0];
        assert!(matches!(c.admit(), Err(LightError::Invalid("color channel must be finite"))));
        // Non-finite center.
        let mut center = good();
        center.center = Vec3::new(f32::NAN, 0.0, 0.0);
        assert!(matches!(center.admit(), Err(LightError::Invalid("shadow center must be finite"))));
    }

    #[test]
    fn frustum_domain_rejected() {
        let mut r = good();
        r.radius = 0.0;
        assert!(matches!(
            r.admit(),
            Err(LightError::Unsupported("shadow radius out of (0, 32] world units"))
        ));
        r.radius = SHADOW_RADIUS_MAX + 1.0;
        assert!(matches!(
            r.admit(),
            Err(LightError::Unsupported("shadow radius out of (0, 32] world units"))
        ));
        let mut nf = good();
        nf.near = 5.0;
        nf.far = 5.0;
        assert!(matches!(
            nf.admit(),
            Err(LightError::Unsupported("shadow far must exceed near"))
        ));
        let mut far = good();
        far.far = SHADOW_FAR_MAX + 1.0;
        assert!(matches!(far.admit(), Err(LightError::Unsupported("shadow far > 160"))));
        let mut b = good();
        b.bias = 0.02;
        assert!(matches!(b.admit(), Err(LightError::Unsupported("shadow bias out of [0, .01]"))));
    }

    #[test]
    fn look_at_up_fallback_is_deterministic() {
        // Direction parallel to Y must not produce a NaN cross product.
        let mut l = good();
        l.dir_to_light = Vec3::Y;
        let vp = l.admit().unwrap().shadow_vp;
        assert!(vp.is_finite());
        l.dir_to_light = -Vec3::Y;
        let vp2 = l.admit().unwrap().shadow_vp;
        assert!(vp2.is_finite());
    }

    #[test]
    fn slanted_candidate_is_xy_plane_only() {
        let mut l = good();
        l.dir_to_light = Vec3::Y;
        assert!(matches!(
            shadow_vp_slanted(l.dir_to_light, l.center, l.radius, l.near, l.far),
            Err(LightError::Unsupported("slanted projection is x/z-plane only"))
        ));
    }
}
