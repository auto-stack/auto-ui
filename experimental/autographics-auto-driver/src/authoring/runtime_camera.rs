//! Run-owned runtime game cameras (PLAN-875). One selected dimension per Run:
//! a 2D follow camera over the accepted character inside a user-configured
//! world rectangle and viewport, or a 3D orbit camera whose target is the
//! accepted character position. The camera state rides the admitted Play
//! `Frame`, so the frame and its camera are prepared, accepted and published
//! as one transaction; OS pointer/wheel math (orbit rates, basis components,
//! letterbox projection) is host-side system graphics math and never enters
//! the Auto world, Document, dirty tracking or Undo/Redo.
use agp::render3d::{Camera3d, Projection};
use glam::{Mat4, Vec2, Vec3};

/// 2D game child surface logical size (view2d.rs SIZE).
pub const SURFACE_2D: Vec2 = Vec2::new(800.0, 380.0);
/// 3D game child surface logical size; aspect 2.0.
pub const SURFACE_3D: Vec2 = Vec2::new(800.0, 400.0);
/// Frozen orbit/zoom policy (contract section 4): 0.25 degrees per logical
/// drag pixel, wheel distance factor per notch, clamped domains.
pub const ORBIT_DEG_PER_PX: f32 = 0.25;
pub const ZOOM_FACTOR_PER_NOTCH: f32 = 1.1;
pub const PITCH_MIN_DEG: f32 = 15.0;
pub const PITCH_MAX_DEG: f32 = 75.0;
pub const DIST_MIN: f32 = 4.0;
pub const DIST_MAX: f32 = 12.0;
pub const ORTHO_SIZE_MIN: f32 = 1.0;
pub const ORTHO_SIZE_MAX: f32 = 64.0;
/// Frozen perspective projection; orthographic shares near/far/aspect.
pub const FOV_Y_DEG: f32 = 60.0;
pub const NEAR: f32 = 0.05;
pub const FAR: f32 = 256.0;
/// Input accumulator saturation caps (fixed scalar state, never a queue).
pub const PENDING_YAW_CAP_DEG: f32 = 720.0;
pub const PENDING_PITCH_CAP_DEG: f32 = 720.0;
pub const PENDING_NOTCH_CAP: f32 = 64.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraDim {
    TwoD,
    ThreeD,
}

/// The camera outputs admitted with a Play frame. Created by `Play::admit`
/// on tick 0 (configuration) and cross-checked against the host-sent orbit
/// candidate on later ticks; `target_id` resolves the character run index to
/// the document node id so presentation reads the accepted frame only.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeCameraFrame {
    pub dim: CameraDim,
    pub target: i32,
    pub target_id: u64,
    pub run_id: i32,
    pub tick: i32,
    /// 2D: world rect (min_x, min_y, max_x, max_y). 3D: unused (zeros).
    pub rect: [f32; 4],
    /// 2D: visible world viewport (width, height). 3D: unused (zeros).
    pub view: [f32; 2],
    /// 3D orbit state echoed by the trusted unit (degrees; yaw in [0,360)).
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    /// 3D projection: 0 = perspective (frozen FOV), 1 = orthographic (size).
    pub projection: i32,
    pub ortho_size: f32,
}

/// Host-side horizontal basis for one candidate yaw (degrees): forward
/// F=(-sin,0,-cos), right R=(cos,0,-sin), matching the frozen yaw-0 rule
/// (eye above +z, horizontal forward -z, right +x).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Basis {
    pub forward_x: f32,
    pub forward_z: f32,
    pub right_x: f32,
    pub right_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}
pub fn normalize_yaw_deg(yaw: f32) -> f32 {
    if !yaw.is_finite() {
        return 0.0;
    }
    yaw.rem_euclid(360.0)
}
pub fn basis_from_yaw(yaw_deg: f32) -> Basis {
    let (sy, cy) = yaw_deg.to_radians().sin_cos();
    Basis {
        forward_x: -sy,
        forward_z: -cy,
        right_x: cy,
        right_z: -sy,
        yaw: normalize_yaw_deg(yaw_deg),
        pitch: 0.0,
        distance: 0.0,
    }
}
impl RuntimeCameraFrame {
    pub fn view_proj_3d(&self, target: Vec3) -> Option<Mat4> {
        if self.dim != CameraDim::ThreeD || self.pitch.is_finite() == false {
            return None;
        }
        let (yaw_ok, pitch_ok, dist_ok) = (
            (0.0..360.0).contains(&self.yaw),
            (PITCH_MIN_DEG..=PITCH_MAX_DEG).contains(&self.pitch),
            (DIST_MIN..=DIST_MAX).contains(&self.distance),
        );
        if !yaw_ok || !pitch_ok || !dist_ok {
            return None;
        }
        let (sy, cy) = self.yaw.to_radians().sin_cos();
        let (sp, cp) = self.pitch.to_radians().sin_cos();
        let offset = Vec3::new(sy * cp, sp, cy * cp) * self.distance;
        let projection = match self.projection {
            0 => Projection::Perspective {
                fov_y_deg: FOV_Y_DEG,
                aspect: SURFACE_3D.x / SURFACE_3D.y,
                near: NEAR,
                far: FAR,
            },
            1 if (ORTHO_SIZE_MIN..=ORTHO_SIZE_MAX).contains(&self.ortho_size) => {
                Projection::Orthographic {
                    size: self.ortho_size,
                    aspect: SURFACE_3D.x / SURFACE_3D.y,
                    near: NEAR,
                    far: FAR,
                }
            }
            _ => return None,
        };
        Some(
            Camera3d {
                eye: target + offset,
                target,
                up: Vec3::Y,
                projection,
            }
            .view_proj(),
        )
    }
    /// Equal-ratio centered letterbox for the 2D viewport inside the game
    /// surface: scale = min(surface/view) per axis, bars on the other axis.
    pub fn letterbox_2d(&self) -> Option<(Vec2, Vec2, Vec2)> {
        if self.dim != CameraDim::TwoD || self.view[0] <= 0.0 || self.view[1] <= 0.0 {
            return None;
        }
        let view = Vec2::from(self.view);
        let ratio = SURFACE_2D / view;
        let scale = ratio.x.min(ratio.y);
        let content = view * Vec2::splat(scale);
        if !(content.x.is_finite() && content.y.is_finite()) || content.x > SURFACE_2D.x {
            return None;
        }
        let bars = (SURFACE_2D - content) * 0.5;
        Some((Vec2::splat(scale), content, bars))
    }
    /// World -> surface pixel for the 2D follow camera (y up in world, y down
    /// on the surface); returns None outside the finite surface range.
    pub fn screen_2d(&self, world: Vec2, center: Vec2) -> Option<Vec2> {
        let (scale, content, bars) = self.letterbox_2d()?;
        let rect_min = Vec2::new(self.rect[0], self.rect[1]);
        let rect_max = Vec2::new(self.rect[2], self.rect[3]);
        if rect_min.x >= rect_max.x || rect_min.y >= rect_max.y {
            return None;
        }
        // The follow center lands on the content-area center, not on the
        // bar corner: bars + content/2 + (world - center) * scale.
        let p = bars
            + content * 0.5
            + Vec2::new(
                (world.x - center.x) * scale.x,
                (center.y - world.y) * scale.y,
            );
        p.is_finite().then_some(p)
    }
    /// Follow center: the accepted character position clamped so the viewport
    /// stays inside the configured rectangle (no interpolation).
    pub fn follow_center_2d(&self, actor: Vec2) -> Option<Vec2> {
        if !actor.is_finite() {
            return None;
        }
        let half = Vec2::from(self.view) * 0.5;
        let min = Vec2::new(self.rect[0], self.rect[1]) + half;
        let max = Vec2::new(self.rect[2], self.rect[3]) - half;
        if min.x > max.x || min.y > max.y {
            return None;
        }
        Some(actor.clamp(min, max))
    }
}
/// Fixed-size per-Run input accumulator for orbit/zoom (Play-owned). Events
/// saturate into scalar pending state; every dispatch folds at most one
/// candidate and clears the pendings, so rendering alone never consumes input.
#[derive(Default)]
pub struct CameraInput {
    pub pending_yaw: f32,
    pub pending_pitch: f32,
    pub pending_notches: f32,
    pub saturated: u32,
    pub active: bool,
}
impl CameraInput {
    pub fn drag(&mut self, dyaw: f32, dpitch: f32) {
        if !self.active {
            return;
        }
        if !dyaw.is_finite() || !dpitch.is_finite() {
            self.saturated = self.saturated.saturating_add(1);
            return;
        }
        let before = (self.pending_yaw, self.pending_pitch);
        self.pending_yaw =
            (self.pending_yaw + dyaw).clamp(-PENDING_YAW_CAP_DEG, PENDING_YAW_CAP_DEG);
        self.pending_pitch =
            (self.pending_pitch + dpitch).clamp(-PENDING_PITCH_CAP_DEG, PENDING_PITCH_CAP_DEG);
        if self.pending_yaw != before.0 + dyaw || self.pending_pitch != before.1 + dpitch {
            self.saturated = self.saturated.saturating_add(1);
        }
    }
    pub fn zoom(&mut self, notches: f32) {
        if !self.active {
            return;
        }
        if !notches.is_finite() {
            self.saturated = self.saturated.saturating_add(1);
            return;
        }
        let before = self.pending_notches;
        self.pending_notches =
            (self.pending_notches + notches).clamp(-PENDING_NOTCH_CAP, PENDING_NOTCH_CAP);
        if self.pending_notches != before + notches {
            self.saturated = self.saturated.saturating_add(1);
        }
    }
    pub fn clear(&mut self) {
        // Pendings are clock state; the saturation counter is a diagnostic
        // that survives Pause/Resume so the host can still report it.
        self.pending_yaw = 0.0;
        self.pending_pitch = 0.0;
        self.pending_notches = 0.0;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame3(yaw: f32, pitch: f32, dist: f32) -> RuntimeCameraFrame {
        RuntimeCameraFrame {
            dim: CameraDim::ThreeD,
            target: 0,
            target_id: 1,
            run_id: 1,
            tick: 1,
            rect: [0.0; 4],
            view: [0.0; 2],
            yaw,
            pitch,
            distance: dist,
            projection: 0,
            ortho_size: 0.0,
        }
    }
    #[test]
    fn basis_and_yaw_zero_rule_match_the_frozen_axes() {
        let b = basis_from_yaw(0.0);
        assert_eq!((b.forward_x, b.forward_z), (0.0, -1.0));
        assert_eq!((b.right_x, b.right_z), (1.0, 0.0));
        let b90 = basis_from_yaw(90.0);
        assert!((b90.forward_x - (-1.0)).abs() < 1e-6, "{b90:?}");
        assert!(b90.forward_z.abs() < 1e-6);
        assert!(b90.right_x.abs() < 1e-6);
        assert!((b90.right_z - (-1.0)).abs() < 1e-6);
        assert_eq!(normalize_yaw_deg(-90.0), 270.0);
        assert_eq!(normalize_yaw_deg(450.0), 90.0);
        let b = basis_from_yaw(30.0);
        assert_eq!((b.yaw, b.pitch, b.distance), (30.0, 0.0, 0.0));
    }
    #[test]
    fn orbit_matrix_sits_above_plus_z_at_yaw_zero_and_rejects_out_of_domain() {
        let pos = Vec3::new(1.0, 0.5, -2.0);
        let vp = frame3(0.0, 45.0, 8.0).view_proj_3d(pos).expect("matrix");
        assert!(vp != Mat4::ZERO);
        for (yaw, pitch, dist) in [(400.0, 45.0, 8.0), (0.0, 10.0, 8.0), (0.0, 45.0, 20.0)] {
            assert!(frame3(yaw, pitch, dist).view_proj_3d(pos).is_none());
        }
        let mut f = frame3(0.0, 45.0, 8.0);
        f.pitch = f32::NAN;
        assert!(f.view_proj_3d(pos).is_none());
        let mut f = frame3(0.0, 45.0, 8.0);
        f.projection = 1;
        f.ortho_size = 24.0;
        assert!(f.view_proj_3d(pos).is_some());
        f.ortho_size = 0.5;
        assert!(f.view_proj_3d(pos).is_none());
    }
    #[test]
    fn follow_clamp_letterbox_and_screen_match_the_frozen_sample_rule() {
        let f = RuntimeCameraFrame {
            dim: CameraDim::TwoD,
            target: 0,
            target_id: 1,
            run_id: 1,
            tick: 0,
            rect: [0.0, -4.0, 30.0, 12.0],
            view: [12.0, 8.0],
            yaw: 0.0,
            pitch: 0.0,
            distance: 0.0,
            projection: 0,
            ortho_size: 0.0,
        };
        assert_eq!(
            f.follow_center_2d(Vec2::new(2.0, 4.0)),
            Some(Vec2::new(6.0, 4.0))
        );
        assert_eq!(
            f.follow_center_2d(Vec2::new(15.0, 4.0)),
            Some(Vec2::new(15.0, 4.0))
        );
        assert_eq!(
            f.follow_center_2d(Vec2::new(28.0, 11.0)),
            Some(Vec2::new(24.0, 8.0))
        );
        let (scale, content, bars) = f.letterbox_2d().unwrap();
        assert!((scale.x - 47.5).abs() < 1e-4 && scale.x == scale.y);
        assert!((content.x - 570.0).abs() < 1e-3 && (content.y - 380.0).abs() < 1e-3);
        assert!((bars.x - 115.0).abs() < 1e-3 && bars.y == 0.0);
        // The visible viewport window at center (6,4) is x[0,12] y[0,8];
        // its corners land exactly on the content area edges.
        let c = f.follow_center_2d(Vec2::new(6.0, 4.0)).unwrap();
        let bottom_left = f.screen_2d(Vec2::new(0.0, 0.0), c).unwrap();
        assert!((bottom_left.x - 115.0).abs() < 1e-3, "{bottom_left:?}");
        assert!((bottom_left.y - 380.0).abs() < 1e-3);
        let top_right = f.screen_2d(Vec2::new(12.0, 8.0), c).unwrap();
        assert!((top_right.x - 685.0).abs() < 1e-3, "{top_right:?}");
        assert!(top_right.y.abs() < 1e-3);
        // The follow center is the content-area center.
        let mid = f.screen_2d(c, c).unwrap();
        assert!(
            (mid.x - 400.0).abs() < 1e-3 && (mid.y - 190.0).abs() < 1e-3,
            "{mid:?}"
        );
        // Movement along +x moves the projection right by scale per world unit.
        let a = f.screen_2d(Vec2::new(3.0, 4.0), c).unwrap();
        let b = f.screen_2d(Vec2::new(4.0, 4.0), c).unwrap();
        assert!((b.x - a.x - 47.5).abs() < 1e-3);
        assert!(f.screen_2d(Vec2::new(f32::NAN, 0.0), c).is_none());
    }
    #[test]
    fn accumulator_saturates_clears_and_gates() {
        let mut acc = CameraInput::default();
        assert!(!acc.active);
        acc.drag(90.0, 90.0);
        acc.zoom(100.0);
        assert_eq!(
            (acc.pending_yaw, acc.pending_pitch, acc.pending_notches),
            (0.0, 0.0, 0.0)
        );
        acc.active = true;
        acc.drag(2000.0, -2000.0);
        assert_eq!(acc.pending_yaw, PENDING_YAW_CAP_DEG);
        assert_eq!(acc.pending_pitch, -PENDING_PITCH_CAP_DEG);
        assert_eq!(acc.saturated, 1, "per-event saturation is diagnosed");
        acc.zoom(3.0);
        acc.zoom(-1.5);
        assert_eq!(acc.pending_notches, 1.5);
        acc.drag(f32::NAN, 1.0);
        assert_eq!(acc.saturated, 2, "the nonfinite event is diagnosed");
        acc.clear();
        assert!(acc.active);
        assert_eq!(acc.saturated, 2, "the diagnostic survives clear");
        assert_eq!(
            (acc.pending_yaw, acc.pending_pitch, acc.pending_notches),
            (0.0, 0.0, 0.0)
        );
    }
}
