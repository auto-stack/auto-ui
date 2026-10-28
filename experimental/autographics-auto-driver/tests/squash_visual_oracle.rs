use agp::{
    core::runtime::Runtime,
    games::squash::{mob_model_matrix, player_model_matrix, SquashAction, SquashDriver},
    host::surface_runtime::SurfaceContent,
    render3d::MeshInstance,
};
use autographics_auto_driver::{host::Host, squash::source};
use glam::{Mat4, Vec3};
fn image_gate(expected: &[u8], actual: &[u8], regions: &[[u32; 4]]) -> bool {
    let (mean, p99) = agp::renderer::capture::compare_png(expected, actual).unwrap();
    mean <= 3.0
        && p99 <= 12
        && regions.iter().all(|r| {
            let (mean, p99) =
                agp::renderer::capture::compare_png_region(expected, actual, *r).unwrap();
            mean <= 3.0 && p99 <= 12
        })
}
fn region(model: &agp::assets::ModelData, matrix: Mat4, vp: Mat4) -> [u32; 4] {
    let mut lo = [900.0f32, 600.0];
    let mut hi = [0.0f32, 0.0];
    for point in model.meshes.iter().flat_map(|mesh| &mesh.positions) {
        let p = (vp * matrix).project_point3(Vec3::from(*point));
        let xy = [(p.x + 1.0) * 450.0, (1.0 - p.y) * 300.0];
        for i in 0..2 {
            lo[i] = lo[i].min(xy[i]);
            hi[i] = hi[i].max(xy[i]);
        }
    }
    let x = (lo[0].floor() - 2.0).clamp(0.0, 899.0) as u32;
    let y = (lo[1].floor() - 2.0).clamp(0.0, 599.0) as u32;
    let right = (hi[0].ceil() + 2.0).clamp(x as f32 + 1.0, 900.0) as u32;
    let bottom = (hi[1].ceil() + 2.0).clamp(y as f32 + 1.0, 600.0) as u32;
    [x, y, right - x, bottom - y]
}
#[test]
fn frozen_visual_keyframes_and_real_fault_controls() {
    let doc: serde_json::Value = serde_json::from_str(include_str!(
        "../../autographics-prototype/testdata/squash/golden_squash_trace.json"
    ))
    .unwrap();
    let squash = doc["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "squash")
        .unwrap()["t"]
        .as_u64()
        .unwrap() as usize;
    let death = doc["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "death")
        .unwrap()["t"]
        .as_u64()
        .unwrap() as usize;
    let keys = std::collections::BTreeSet::from([0usize, 60, 200, squash, death]);
    let models = [
        agp::assets::load_glb_bytes(include_bytes!(
            "../../autographics-prototype/assets/squash_the_creeps/art/player.glb"
        ))
        .unwrap(),
        agp::assets::load_glb_bytes(include_bytes!(
            "../../autographics-prototype/assets/squash_the_creeps/art/mob.glb"
        ))
        .unwrap(),
    ];
    let mut host = Host::new_squash(&source()).unwrap();
    host.settle().unwrap();
    let surface = host.game_surfaces[0];
    let mut reference = SquashDriver::new(27);
    let mut rt = Runtime::new(std::time::Duration::from_nanos(8_333_333), 27);
    std::fs::create_dir_all("research/834-keyframes").unwrap();
    for (tick, a) in doc["actions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .take(death + 1)
    {
        let action = [
            a[0].as_i64().unwrap() as i32,
            a[1].as_i64().unwrap() as i32,
            a[2].as_i64().unwrap() as i32,
        ];
        rt.step(
            &mut reference,
            vec![SquashAction {
                move_x: action[0] as i8,
                move_z: action[1] as i8,
                jump: action[2] != 0,
                ..Default::default()
            }],
        );
        let mut q = host.squash.take().unwrap();
        let id = q.scripts[0].id();
        q.scripts[0].tick(id, tick as u64, action).unwrap();
        q.scripts[0].settle().unwrap();
        q.pump_outputs(&mut host).unwrap();
        host.squash = Some(q);
        if !keys.contains(&tick) {
            continue;
        }
        host.rt.frame(None, &Default::default()).unwrap();
        let actual = host.rt.capture(surface, "Auto keyframe").unwrap();
        let mut poses = vec![];
        if !reference.dead {
            poses.push((
                0,
                player_model_matrix(
                    reference.player,
                    reference.facing_yaw,
                    reference.vy,
                    reference.anim_time,
                ),
            ));
        }
        for mob in &reference.mobs {
            poses.push((
                1,
                mob_model_matrix(mob.pos, mob.vel, mob.age * (mob.speed / 10.0)),
            ));
        }
        let mut boxes = vec![];
        let (saved, saved_glyphs, saved_overlay) =
            if let Some(SurfaceContent::Scene3d(content)) = host.rt.content_mut(surface) {
                let saved = content.instances.clone();
                let mut expected = content.instances[..5].to_vec();
                for (kind, matrix) in &poses {
                    boxes.push(region(&models[*kind], *matrix, content.view_proj));
                    for mesh in &models[*kind].meshes {
                        expected.push(MeshInstance {
                            mesh: (models[*kind].source_sha256.clone(), mesh.name.clone()),
                            model: *matrix,
                        });
                    }
                }
                let saved_glyphs = content.text_overlay.as_ref().unwrap().glyphs.clone();
                let text = &content.text_overlay.as_ref().unwrap().renderer;
                let mut glyphs = text.layout(
                    &format!("Score: {}", reference.score),
                    16.0,
                    16.0,
                    agp::core::scene::Color::rgb(0.0, 0.0, 0.0),
                );
                if reference.dead {
                    let label = "Press Space or Enter to retry";
                    glyphs.extend(text.layout(
                        label,
                        (900.0 - text.measure(label)) * 0.5,
                        284.0,
                        agp::core::scene::Color::rgb(1.0, 1.0, 1.0),
                    ));
                }
                let saved_overlay = content.screen_overlay.as_ref().unwrap().commands.clone();
                assert_eq!(
                    saved_overlay.len(),
                    reference.dead as usize,
                    "Retry overlay presence"
                );
                content.instances = expected;
                content.text_overlay.as_mut().unwrap().glyphs = glyphs;
                (saved, saved_glyphs, saved_overlay)
            } else {
                panic!("3d content lost")
            };
        host.rt.frame(None, &Default::default()).unwrap();
        let expected = host.rt.capture(surface, "frozen reference").unwrap();
        assert!(
            image_gate(&expected.1, &actual.1, &boxes),
            "original image tolerance failed at {tick}"
        );
        std::fs::write(format!("research/834-keyframes/auto-{tick}.png"), &actual.1).unwrap();
        std::fs::write(
            format!("research/834-keyframes/reference-{tick}.png"),
            &expected.1,
        )
        .unwrap();
        if tick == 0 {
            let shadow = agp::renderer::capture::shadow_probe_position(&expected.1).unwrap();
            let correct = host
                .rt
                .probe_pixel(surface, [shadow[0] as f32, shadow[1] as f32])
                .unwrap();
            if let Some(SurfaceContent::Scene3d(c)) = host.rt.content_mut(surface) {
                c.mesh.set_shadow_sabotage(true);
            }
            host.rt.frame(None, &Default::default()).unwrap();
            let wrong = host
                .rt
                .probe_pixel(surface, [shadow[0] as f32, shadow[1] as f32])
                .unwrap();
            assert!(
                correct[0].abs_diff(wrong[0]) > 12,
                "shadowless render escaped original shadow oracle"
            );
            if let Some(SurfaceContent::Scene3d(c)) = host.rt.content_mut(surface) {
                c.mesh.set_shadow_sabotage(false);
                c.instances.truncate(5);
            }
            host.rt.frame(None, &Default::default()).unwrap();
            let missing = host.rt.capture(surface, "missing model").unwrap();
            assert!(
                !image_gate(&expected.1, &missing.1, &boxes),
                "missing original model escaped formal image gate"
            );
            for (from, to) in [
                ("v0 = -5.0326", "v0 = 25.0"),
                ("if f == 5 { return 0.0 }", "if f == 5 { return 0.5 }"),
                ("if f >= 6 { return 1.0 }", "if f >= 6 { return 1.5 }"),
            ] {
                let original = source();
                assert!(original.contains(from));
                let mutant = original.replace(from, to);
                let mut other = Host::new_squash(&mutant).unwrap();
                other.settle().unwrap();
                let changed = other
                    .rt
                    .capture(other.game_surfaces[0], "Auto transform mutation")
                    .unwrap();
                assert!(
                    !image_gate(&expected.1, &changed.1, &boxes),
                    "real Auto animation/roll/scale edit did not reach pixels: {from}"
                );
            }
        }
        if let Some(SurfaceContent::Scene3d(c)) = host.rt.content_mut(surface) {
            c.instances = saved;
            c.text_overlay.as_mut().unwrap().glyphs = saved_glyphs;
            c.screen_overlay.as_mut().unwrap().commands = saved_overlay;
        }
        println!(
            "original visual keyframe tick={tick} roles={} mean/p99={:?}",
            poses.len(),
            agp::renderer::capture::compare_png(&expected.1, &actual.1).unwrap()
        );
    }
}
