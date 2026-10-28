//! PLAN-857 T-02: distinguish camera inversion and perspective drag approaches.
use agp::render3d::{Camera3d, Projection};
use autographics_auto_driver::authoring::interaction::project;
use glam::{Mat4, Vec2, Vec3, Vec4};

fn unproject(vp: Mat4, pixel: Vec2, depth: f32) -> Vec3 {
    let clip = Vec4::new(pixel.x / 400.0 - 1.0, 1.0 - pixel.y / 200.0, depth, 1.0);
    let p = vp.inverse() * clip;
    p.truncate() / p.w
}
fn point_on_axis_plane(vp: Mat4, pixel: Vec2, origin: Vec3, axis: Vec3, forward: Vec3) -> Vec3 {
    let normal = (forward - axis * forward.dot(axis)).normalize();
    let near = unproject(vp, pixel, 0.0);
    let far = unproject(vp, pixel, 1.0);
    let ray = far - near;
    near + ray * ((origin - near).dot(normal) / ray.dot(normal))
}
#[test]
fn finite_large_orthographic_view_is_invertible_below_old_absolute_determinant() {
    let camera = Camera3d {
        eye: Vec3::new(0.0, 200.0, 200.0),
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: Projection::Orthographic {
            size: 8192.0,
            aspect: 2.0,
            near: 0.05,
            far: 16384.0,
        },
    };
    let vp = camera.view_proj();
    assert!(vp.is_finite() && vp.determinant().abs() < 1e-10);
    assert!(autographics_auto_driver::authoring::interaction::stable_inverse(vp).is_some());
    assert!(autographics_auto_driver::authoring::interaction::stable_inverse(Mat4::ZERO).is_none());
    let point = Vec3::new(35.0, 0.0, -60.0);
    let clip = vp * point.extend(1.0);
    let roundtrip = vp.inverse() * (clip / clip.w);
    let recovered = roundtrip.truncate() / roundtrip.w;
    assert!(
        (recovered - point).length() < 0.02,
        "valid large view: {recovered:?}"
    );
}
#[test]
fn perspective_ray_plane_recovers_axis_while_old_screen_delta_does_not() {
    let camera = Camera3d {
        eye: Vec3::new(8.0, 6.0, 12.0),
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_deg: 60.0,
            aspect: 2.0,
            near: 0.05,
            far: 16384.0,
        },
    };
    let vp = camera.view_proj();
    let origin = Vec3::ZERO;
    let axis = Vec3::X;
    let forward = (camera.target - camera.eye).normalize();
    let start = project(vp, origin).unwrap();
    let end = project(vp, origin + axis * 3.0).unwrap();
    let unit = project(vp, origin + axis).unwrap() - start;
    let old = (end - start).dot(unit) / unit.length_squared();
    let a = point_on_axis_plane(vp, start, origin, axis, forward);
    let b = point_on_axis_plane(vp, end, origin, axis, forward);
    let exact = (b - a).dot(axis);
    let actual =
        autographics_auto_driver::authoring::interaction::AxisDrag::new(vp, origin, 0, start)
            .unwrap()
            .delta(end)
            .unwrap();
    assert!(
        (actual - exact).abs() < 0.005,
        "actual drag must match independent ray-plane oracle: {actual} vs {exact}"
    );
    assert!((exact - 3.0).abs() < 0.005, "ray-plane exact={exact}");
    assert!(
        (old - 3.0).abs() > 0.1,
        "old approximation must be distinguishable: {old}"
    );
}
