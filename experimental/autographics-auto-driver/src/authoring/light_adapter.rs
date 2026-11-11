//! PLAN-080 T-03: stable Node→Light adapter contract ("只供准备").
//!
//! The neutral light service (agp::render3d::LightParams::admit) is frozen in
//! T-02. This module defines the STABLE adapter surface an author chain will
//! later implement against — the mapping from typed Node declarations /
//! bound instance state to `LightParams`, plus the classified rejection the
//! 979/981 A slices will consume. It deliberately does NOT depend on any
//! 979/981 crate: the contract is spelled in plain types here so the author
//! slices can bind against it without this plan re-implementing their
//! grammar/projection semantics.
//!
//! A second light is rejected as Unsupported before any GPU write; the single
//! legal light keeps its last-good published state (the neutral port already
//! enforces this at the GPU boundary — see prototype render3d::light).

use agp::render3d::LightParams;
use glam::Vec3;

/// One declared directional light inside a Node's view-3d block.
///
/// Fields mirror the frozen neutral port's support domain. Source span and
/// package/instance identity belong to the 979/981 author slices (T-04); this
/// contract only fixes the typed VALUES and the rejection classification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeclaredLight {
    /// From-surface-toward-light direction (finite, nonzero).
    pub dir_to_light: Vec3,
    /// RGB color, channels in [0,1].
    pub color: [f32; 3],
    /// Intensity in [0,1].
    pub energy: f32,
    /// Hard shadow enabled.
    pub shadow_enabled: bool,
    /// Shadow frustum center (finite).
    pub center: Vec3,
    /// Orthographic half-extent in world units, (0,32].
    pub radius: f32,
    /// Near plane [0, far).
    pub near: f32,
    /// Far plane (near, 160].
    pub far: f32,
    /// Shadow depth bias in [0,.01].
    pub bias: f32,
}

/// Classified adapter outcome: the single legal light, or a typed rejection.
/// The author chain (979/981) keeps its last-good published light on Err and
/// reports the diagnostic to the host; it never writes a bad light to GPU.
#[derive(Debug, Clone, Copy)]
pub enum AdmitLight {
    Accepted(LightParams),
    Rejected(agp::render3d::LightError),
}

/// The stable adapter entry. Node state (post-979 typed projection, pre-GPU)
/// is mapped onto `DeclaredLight` by the author slices; this plan's neutral
/// port performs numeric admission here so the GPU boundary is shared and the
/// domain rules live in ONE place (prototype render3d::light, unit-tested).
pub fn admit_declared(light: DeclaredLight) -> AdmitLight {
    let params = LightParams {
        dir_to_light: light.dir_to_light,
        color: light.color,
        energy: light.energy,
        shadow_enabled: light.shadow_enabled,
        center: light.center,
        radius: light.radius,
        near: light.near,
        far: light.far,
        bias: light.bias,
    };
    match params.admit() {
        Ok(_) => AdmitLight::Accepted(params),
        Err(e) => AdmitLight::Rejected(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agp::render3d::LightError;

    fn good() -> DeclaredLight {
        DeclaredLight {
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
    fn legal_declaration_admitted() {
        match admit_declared(good()) {
            AdmitLight::Accepted(p) => {
                assert!(p.dir_to_light.is_finite());
                assert_eq!(p.energy, 0.5);
                assert_eq!(p.radius, 32.0);
                assert!(p.admit().is_ok());
            }
            AdmitLight::Rejected(e) => panic!("legal light rejected: {e}"),
        }
    }

    #[test]
    fn second_light_and_out_of_domain_rejected_before_gpu() {
        // 第二灯语义：同一 host 只允许一盏合法灯；任何越域声明都以
        // Unsupported/Invalid 拒绝，GPU 前零写入，last-good 保持。
        let mut bad_energy = good();
        bad_energy.energy = 1.5;
        assert!(matches!(
            admit_declared(bad_energy),
            AdmitLight::Rejected(LightError::Unsupported("energy out of [0,1]"))
        ));
        let mut zero_dir = good();
        zero_dir.dir_to_light = Vec3::ZERO;
        assert!(matches!(
            admit_declared(zero_dir),
            AdmitLight::Rejected(LightError::Invalid("direction must be nonzero"))
        ));
        let mut bad_radius = good();
        bad_radius.radius = 0.0;
        assert!(matches!(
            admit_declared(bad_radius),
            AdmitLight::Rejected(LightError::Unsupported(
                "shadow radius out of (0, 32] world units"
            ))
        ));
    }
}
