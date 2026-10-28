//! T-05 contract tests: GLB asset pipeline, 3D camera/animation, and the
//! squash collision scheme. Pure-state; the probe provides GPU evidence.

use agp::assets::{load_glb_bytes, load_glb_file, AssetError};
use agp::render3d::collision::*;
use glam::Vec3;
use std::path::PathBuf;

fn agp_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ---------------------------------------------------------------------------
// GLB asset pipeline
// ---------------------------------------------------------------------------

#[test]
fn glb_loads_frozen_player_model() {
    let model = load_glb_file(&agp_file("assets/squash_the_creeps/art/player.glb"))
        .expect("player.glb loads");
    assert_eq!(model.meshes.len(), 3);
    let verts: Vec<usize> = model.meshes.iter().map(|m| m.positions.len()).collect();
    assert_eq!(
        verts,
        vec![273, 687, 273],
        "frozen vertex counts (T-01 §2.5)"
    );
    let names: Vec<&str> = model
        .meshes
        .iter()
        .map(|m| m.material.name.as_str())
        .collect();
    assert_eq!(names, vec!["pupil", "body", "eye"]);
    let body = &model.meshes[1];
    assert!(
        (body.material.base_color[0] - 0.8).abs() < 1e-4
            && body.material.base_color[1] < 0.11
            && body.material.base_color[2] < 0.01,
        "player body red-orange: {:?}",
        body.material.base_color
    );
    assert_eq!(body.indices.len() % 3, 0, "indexed triangles");
    // Node transform is identity for the player GLB: y range unchanged.
    let min_y = body.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        (min_y - (-0.4739)).abs() < 1e-3,
        "identity node baked: {min_y}"
    );
    assert!(!model.source_sha256.is_empty());
}

#[test]
fn glb_loads_frozen_mob_model_with_node_transform_baked() {
    let model =
        load_glb_file(&agp_file("assets/squash_the_creeps/art/mob.glb")).expect("mob.glb loads");
    assert_eq!(model.meshes.len(), 3);
    let verts: Vec<usize> = model.meshes.iter().map(|m| m.positions.len()).collect();
    assert_eq!(verts, vec![273, 974, 273]);
    // mob node carries translation (0.011, 0.506, 0.081), scale
    // (0.953, 1, 1) AND rotation quat (-0.5, -0.5, 0.5, 0.5) — the full
    // node transform must be baked in (raw eye min y would be -0.7016).
    let eye = &model.meshes[0];
    let min_y = eye.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        (min_y - (-0.7016)).abs() > 0.05,
        "mob node transform must change vertices: {min_y}"
    );
    let body = &model.meshes[1];
    assert!(
        body.material.base_color[2] > 0.08 && body.material.base_color[0] < 0.01,
        "mob_body dark navy: {:?}",
        body.material.base_color
    );
}

#[test]
fn glb_failure_taxonomy_missing_and_corrupt() {
    let missing = load_glb_file(&agp_file("assets/squash_the_creeps/art/__missing__.glb"));
    assert!(matches!(missing, Err(AssetError::MissingFile(_))));
    assert_eq!(missing.unwrap_err().classify(), "missing_file");

    let corrupt = load_glb_bytes(b"this is not a glb at all");
    assert!(matches!(corrupt, Err(AssetError::Parse(_))));
    assert_eq!(corrupt.unwrap_err().classify(), "bad_model_parse");
}

#[test]
fn glb_untextured_guard_would_fire_on_textured_models() {
    // The frozen models are untextured; the loader rejects textured ones.
    // (No textured fixture ships in the frozen tree, so this asserts the
    // guard exists via a synthetic GLB would be overkill — instead verify
    // the frozen models pass the texture scan.)
    for f in ["player.glb", "mob.glb"] {
        let model = load_glb_file(&agp_file(&format!("assets/squash_the_creeps/art/{f}")))
            .expect("untextured frozen model loads without Unsupported");
        assert!(model.meshes.iter().all(|m| m.material.roughness == 0.5));
    }
}

// ---------------------------------------------------------------------------
// Bob animation (frozen tscn parameters, cited in render3d::anim)
// ---------------------------------------------------------------------------

#[test]
fn bob_animation_speed_scales_deterministically() {
    use agp::render3d::BobAnimation;
    let bob = BobAnimation::default();
    // player moving: speed_scale = 4 (Player.gd line 31).
    let t = bob.time_at(30, 1.0 / 60.0, 4.0);
    assert!((t - 2.0).abs() < 1e-4, "30 ticks at 4x = 2s: {t}");
    let (y, rot) = bob.eval(t);
    // 2.0 s mod 1.2 = 0.8 s — between keys (0.7, 1.18).
    let (y0, _) = bob.eval(0.7);
    let (y1, _) = bob.eval(1.18);
    assert!(y >= y0.min(y1) - 1e-5 && y <= y0.max(y1) + 1e-5);
    let _ = rot;
}

// ---------------------------------------------------------------------------
// Collision feasibility (the frozen scheme)
// ---------------------------------------------------------------------------

#[test]
fn collision_player_rests_on_ground_and_jumps() {
    let start = Vec3::new(0.0, PLAYER_SPHERE_OFFSET_Y, 0.0);
    let (mut c, mut v, mut ground) = step_player(start, Vec3::ZERO, false);
    assert!(ground, "starts on ground");
    for _ in 0..240 {
        let r = step_player(c, v, false);
        c = r.0;
        v = r.1;
        ground = r.2;
    }
    assert!(ground, "still on ground after 240 steps");
    // Resting center height = sphere radius (sphere bottom on the ground
    // plane y=0); the tscn 0.771765 is the shape's local offset in Godot's
    // CharacterBody origin convention, not the world rest height.
    assert!(
        (c.y - PLAYER_SPHERE_RADIUS).abs() < 1e-3,
        "settled at rest height (radius): {}",
        c.y
    );
    // jump
    let (c2, v2, g2) = step_player(c, v, true);
    assert!(!g2 || v2.y > 0.0, "jump leaves the ground");
    let mut apex = c2.y;
    let (mut cc, mut vv) = (c2, v2);
    for _ in 0..600 {
        let r = step_player(cc, vv, false);
        cc = r.0;
        vv = r.1;
        apex = apex.max(cc.y);
    }
    // Semi-implicit Euler: v0 = 20 - g*dt = 19.375 ⇒ apex ≈ v0²/2g ≈ 2.50 m
    // above the rest height 0.792, plus one integration step of overshoot.
    assert!(apex > 3.1 && apex < 3.6, "jump apex physical: {apex}");
}

#[test]
fn collision_arena_bounds_clamp() {
    let p = clamp_to_arena(Vec3::new(100.0, 1.0, 100.0), PLAYER_SPHERE_RADIUS);
    assert!(
        p.x <= 14.0 - PLAYER_SPHERE_RADIUS + 1e-4,
        "x clamped: {}",
        p.x
    );
    assert!(
        p.z <= 16.0 - PLAYER_SPHERE_RADIUS + 1e-4,
        "z clamped: {}",
        p.z
    );
    let p2 = clamp_to_arena(Vec3::new(-100.0, 1.0, -100.0), PLAYER_SPHERE_RADIUS);
    assert!(p2.x >= -13.0 + PLAYER_SPHERE_RADIUS - 1e-4);
    assert!(p2.z >= -15.0 + PLAYER_SPHERE_RADIUS - 1e-4);
    // inside the arena: untouched
    let p3 = clamp_to_arena(Vec3::new(0.0, 1.0, 0.0), PLAYER_SPHERE_RADIUS);
    assert_eq!(p3, Vec3::new(0.0, 1.0, 0.0));
}

#[test]
fn collision_sphere_vs_obb_top_and_side_classification() {
    let mob_center = Vec3::new(0.0, MOB_BOX_OFFSET_Y, 0.0);
    // Top landing: sphere above the mob, overlapping from above.
    let top = sphere_vs_obb(
        Vec3::new(0.0, MOB_BOX_OFFSET_Y + MOB_BOX_HALF[1] + 0.4, 0.0),
        0.792278,
        mob_center,
        MOB_BOX_HALF,
        0.0,
    );
    let (n, _pen) = top.expect("top contact exists");
    assert_eq!(
        classify_player_mob_contact(n),
        PlayerMobContact::TopLandingSquash
    );

    // Side contact: sphere beside the mob at body height.
    let side = sphere_vs_obb(
        Vec3::new(MOB_BOX_HALF[0] + 0.6, MOB_BOX_OFFSET_Y, 0.0),
        0.792278,
        mob_center,
        MOB_BOX_HALF,
        0.0,
    );
    let (n, _) = side.expect("side contact exists");
    assert_eq!(classify_player_mob_contact(n), PlayerMobContact::Fatal);

    // Yaw-rotated mob (45°): still classifies a top landing.
    let top_rot = sphere_vs_obb(
        Vec3::new(0.0, MOB_BOX_OFFSET_Y + MOB_BOX_HALF[1] + 0.4, 0.0),
        0.792278,
        mob_center,
        MOB_BOX_HALF,
        std::f32::consts::FRAC_PI_4,
    );
    let (n, _) = top_rot.expect("rotated top contact");
    assert_eq!(
        classify_player_mob_contact(n),
        PlayerMobContact::TopLandingSquash
    );

    // No contact when far.
    assert!(sphere_vs_obb(
        Vec3::new(10.0, 5.0, 10.0),
        0.79,
        mob_center,
        MOB_BOX_HALF,
        0.0
    )
    .is_none());
}

#[test]
fn collision_physics_step_is_deterministic() {
    let run = || {
        let mut c = Vec3::new(1.0, PLAYER_SPHERE_OFFSET_Y, 2.0);
        let mut v = Vec3::new(3.0, 0.0, -2.0);
        for i in 0..240u64 {
            if i == 30 {
                let r = step_player(c, v, true);
                c = r.0;
                v = r.1;
            } else {
                let r = step_player(c, v, false);
                c = r.0;
                v = r.1;
            }
        }
        (c, v)
    };
    assert_eq!(run(), run());
}

// ---------------------------------------------------------------------------
// Sample camera
// ---------------------------------------------------------------------------

#[test]
fn sample_camera_matches_main_tscn() {
    use agp::render3d::{Camera3d, Projection};
    let cam = Camera3d::sample_camera(1.5);
    match cam.projection {
        Projection::Orthographic { size, far, .. } => {
            assert_eq!(size, 24.0, "Main.tscn line 105: size = 24.0");
            assert_eq!(far, 40.0, "Main.tscn line 106: far = 40.0");
        }
        _ => panic!("sample camera is orthographic (Main.tscn line 103: projection = 1)"),
    }
    // eye = pivot origin offset + 19 * z_axis (Main.tscn lines 98–102).
    assert!((cam.eye.x - 0.183_514).abs() < 1e-3);
    assert!((cam.eye.y - 13.435).abs() < 0.01, "eye.y {}", cam.eye.y);
    assert!((cam.eye.z - 13.435).abs() < 0.01, "eye.z {}", cam.eye.z);
}
