//! 3D camera. Projection matrices are written explicitly (z in [0,1] NDC for
//! wgpu, right-handed, y-up world) to stay free of projection-API drift —
//! same rationale as the 2D ortho matrix.

use glam::{camera::rh::view::look_at_mat4, Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    /// Perspective with vertical field of view (degrees).
    Perspective {
        fov_y_deg: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    /// Orthographic with vertical world-size (the frozen sample's camera:
    /// `Main.tscn` lines 101–106, `projection = 1`, `size = 24.0`, `far = 40.0`).
    Orthographic {
        size: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera3d {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub projection: Projection,
}

impl Camera3d {
    /// The frozen sample camera, derived from `Main.tscn`:
    /// CameraPivot basis rows (0,0.707,0.707 / 0,-0.707,0.707 ⇒ z-axis
    /// (0, 0.707, 0.707)), camera child at local z=19 → eye
    /// (0.183514, 13.435, 13.435), looking down-forward 45°.
    pub fn sample_camera(aspect: f32) -> Self {
        let z_axis = Vec3::new(0.0, 0.707_106_8, 0.707_106_8);
        let eye = Vec3::new(0.183_514, 0.0, 0.0) + z_axis * 19.0;
        Camera3d {
            eye,
            target: eye - z_axis, // forward = -z_axis
            up: Vec3::new(0.0, 0.707_106_8, -0.707_106_8),
            projection: Projection::Orthographic {
                size: 24.0,
                aspect,
                near: 0.05,
                far: 40.0, // frozen Main.tscn: far = 40.0
            },
        }
    }

    pub fn view(&self) -> Mat4 {
        look_at_mat4(self.eye, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        match self.projection {
            Projection::Perspective {
                fov_y_deg,
                aspect,
                near,
                far,
            } => {
                let f = 1.0 / (fov_y_deg.to_radians() * 0.5).tan();
                let a = f / aspect;
                Mat4::from_cols_array(&[
                    a,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    f,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    far / (near - far),
                    -1.0,
                    0.0,
                    0.0,
                    (near * far) / (near - far),
                    0.0,
                ])
            }
            Projection::Orthographic {
                size,
                aspect,
                near,
                far,
            } => {
                let t = size * 0.5;
                let b = -t;
                let r = t * aspect;
                let l = -r;
                Mat4::from_cols_array(&[
                    2.0 / (r - l),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    2.0 / (t - b),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    -1.0 / (far - near),
                    0.0,
                    -(r + l) / (r - l),
                    -(t + b) / (t - b),
                    -near / (far - near),
                    1.0,
                ])
            }
        }
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection_matrix() * self.view()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4Swizzles;

    #[test]
    fn ortho_maps_center_and_corners() {
        let cam = Camera3d {
            eye: Vec3::new(0.0, 10.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::Orthographic {
                size: 24.0,
                aspect: 1.5,
                near: 0.1,
                far: 40.0,
            },
        };
        let vp = cam.view_proj();
        // A point straight ahead at the ground center projects near NDC 0,0.
        let clip = vp * Vec3::ZERO.extend(1.0);
        let ndc = clip.xyz() / clip.w;
        assert!(
            ndc.x.abs() < 1e-3 && ndc.y.abs() < 1e-3,
            "center at NDC origin: {ndc:?}"
        );
    }

    #[test]
    fn perspective_maps_z_range() {
        let cam = Camera3d {
            eye: Vec3::new(0.0, 0.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::Perspective {
                fov_y_deg: 48.6,
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 100.0,
            },
        };
        let vp = cam.view_proj();
        let near = vp * Vec3::new(0.0, 0.0, 10.0 - 0.1).extend(1.0);
        let far = vp * Vec3::new(0.0, 0.0, 10.0 - 100.0).extend(1.0);
        let zn = near.z / near.w;
        let zf = far.z / far.w;
        assert!((zn - 0.0).abs() < 1e-3, "near plane at NDC z=0: {zn}");
        assert!((zf - 1.0).abs() < 1e-3, "far plane at NDC z=1: {zf}");
    }
}
