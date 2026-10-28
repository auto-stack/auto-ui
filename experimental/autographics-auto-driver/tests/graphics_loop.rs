use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::{host::Host, script::Action, Scalar::Int};
use std::time::Instant;
static GPU_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn source() -> &'static str {
    include_str!("../samples/scene-control/main.at")
}
fn pixel_gate(h: &mut Host) -> bool {
    h.click([80.0, 70.0]);
    h.settle().unwrap();
    let old =
        h.rt.probe_pixel(h.game_surfaces[0], [450.0, 300.0])
            .unwrap();
    let new =
        h.rt.probe_pixel(h.game_surfaces[0], [410.0, 290.0])
            .unwrap();
    let state = h.scripts[0].publication.last().unwrap().objects[0].values;
    println!(
        "pixel old={old:?} new={new:?} state={state:?} presses={:?}",
        h.scripts[0].session.int("integer", &[Int(2)])
    );
    old[0] < 100
        && new[0] > 180
        && (state[0] + 40.0).abs() < 0.01
        && h.scripts[0].session.int("integer", &[Int(2)]).unwrap() == 1
}
#[test]
fn pixels_shared_tick_input_controls_and_lifetimes() {
    let _serial = GPU_TEST.lock().unwrap();
    let mut h = Host::new(source()).unwrap();
    h.settle().unwrap();
    let initial3d = h.rt.capture(h.mesh_surface, "initial").unwrap().0;
    assert!(
        matches!(h.rt.content(h.mesh_surface),Some(SurfaceContent::Scene3d(s)) if !s.instances.is_empty())
    );
    assert!(pixel_gate(&mut h));
    assert_eq!(
        h.scripts[1].publication.last().unwrap().objects[0].values[0],
        0.0
    );
    let moved3d = h.rt.capture(h.mesh_surface, "moved").unwrap().0;
    assert_ne!(
        initial3d, moved3d,
        "real Auto mesh output did not change pixels"
    );
    let mesh_hit = h.click([600.0, 400.0]).unwrap();
    assert_eq!(mesh_hit.0, h.mesh_surface);
    assert_eq!(mesh_hit.1, [150.0, 80.0]);
    h.settle().unwrap();
    assert_eq!(
        h.consumers[0].object(1).unwrap().transform.translation.x,
        -50.0
    );
    assert_eq!(
        h.consumers[0].object(1).unwrap().transform.translation.y,
        70.0
    );
    h.tick(1);
    h.tick(1);
    h.settle().unwrap();
    for _ in 0..5 {
        h.frame(None).unwrap();
    }
    assert_eq!(h.scripts[0].session.int("integer", &[Int(0)]).unwrap(), 1);
    assert_eq!(h.scripts[0].completed_ticks, 1);
    let targets = h.rt.live_target_slots();
    let pipelines = agp::renderer::pipeline_creations();
    let mut times = vec![];
    for t in 2..62 {
        let n = Instant::now();
        h.tick(t);
        h.settle().unwrap();
        times.push(n.elapsed().as_micros());
        assert_eq!(h.rt.live_target_slots(), targets);
        assert_eq!(agp::renderer::pipeline_creations(), pipelines);
    }
    times.sort_unstable();
    println!("cold_compile_us={} p50_us={} p95_us={} targets={} owned_target_bytes={} upload_bytes={} gpu_timestamps=unavailable",h.scripts[0].session.compile_us,times[30],times[57],targets,h.rt.owned_target_bytes(),h.rt.upload_stats().bytes_rewritten);
    let portal =
        h.rt.graph
            .portals()
            .iter()
            .filter(|p| p.desc.child == h.game_surfaces[0])
            .last()
            .unwrap()
            .id;
    h.rt.remove_portal(portal).unwrap();
    let mut single = vec![];
    for t in 62..122 {
        let n = Instant::now();
        h.tick(t);
        h.settle().unwrap();
        single.push(n.elapsed().as_micros());
    }
    single.sort_unstable();
    println!(
        "single_view_p50_us={} p95_us={} pipelines_created={} warm_additional_pipelines=0",
        single[30], single[57], pipelines
    );
    let (_, png, _, _) = h.rt.capture(h.top, "auto-scenes").unwrap();
    std::fs::create_dir_all("research").unwrap();
    std::fs::write("research/auto-scenes.png", png).unwrap();
    for mode in 0..3 {
        h.reset();
        h.settle().unwrap();
        h.drop_output = mode == 0;
        h.corrupt_local = mode == 1;
        h.duplicate_delivery = mode == 2;
        assert!(
            !pixel_gate(&mut h),
            "mutation {mode} escaped actual pixel/input gate"
        );
        h.drop_output = false;
        h.corrupt_local = false;
        h.duplicate_delivery = false;
    }
    let resident = || {
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-Process -Id {}).WorkingSet64", std::process::id()),
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0)
    };
    let before_reset_memory = resident();
    for _ in 0..20 {
        let probes: Vec<_> = h.scripts.iter().map(|s| s.session.owner_probe()).collect();
        let old = h.scripts[0].id();
        h.reset();
        h.settle().unwrap();
        assert!(probes.iter().all(|p| !p()));
        assert!(h.scripts[0].enqueue(old, Action::Destroy).is_err());
        assert_eq!(h.rt.live_target_slots(), targets);
        assert_eq!(agp::renderer::pipeline_creations(), pipelines);
    }
    println!(
        "working_set_before_20_resets={} after={} bytes",
        before_reset_memory,
        resident()
    );
    let id = h.scripts[0].id();
    h.scripts[0].enqueue(id, Action::Destroy).unwrap();
    h.settle().unwrap();
    assert!(h.consumers[0].scene.objects().is_empty());
    assert!(
        matches!(h.rt.content(h.mesh_surface),Some(SurfaceContent::Scene3d(s)) if s.instances.is_empty()&&s.store.uploaded_count()==0)
    );
    assert!(h.errors.is_empty(), "{:?}", h.errors);
}

#[test]
fn runnable_script_fault_does_not_stop_other_surface_or_ui_frames() {
    let _serial = GPU_TEST.lock().unwrap();
    let source = source().replace(
        "x = x + direction * 2.0; ticks = ticks + 1",
        "if direction == 1 { while true { ticks = ticks + 1 } }; ticks = ticks + 1",
    );
    let mut h = Host::new(&source).unwrap();
    h.settle().unwrap();
    h.click([80.0, 70.0]);
    h.settle().unwrap();
    let prior = h.scripts[0].publication.last().unwrap().clone();
    let frames = h.rt.stats.frames;
    h.direction = 1;
    for t in 0..30 {
        h.tick(t);
        h.frame(None).unwrap();
    }
    h.settle().unwrap();
    assert!(h.scripts[0].session.closed());
    assert!(h.scripts[0].session.yielded > 0);
    assert_eq!(h.scripts[0].publication.last(), Some(&prior));
    assert!(h.scripts[1].session.int("integer", &[Int(0)]).unwrap() > 0);
    assert!(h.rt.stats.frames >= frames + 30);
    assert!(h.errors.iter().any(|e| e.contains("budget")));
}
#[test]
fn changing_real_auto_color_changes_gpu_pixels_and_pipeline_oracle_detects_new_build() {
    let _serial = GPU_TEST.lock().unwrap();
    let mut h = Host::new(&source().replace("return 0.8", "return 0.1")).unwrap();
    h.settle().unwrap();
    assert_eq!(
        h.scripts[0].publication.last().unwrap().objects[0].values[6],
        0.1
    );
    let pixel =
        h.rt.probe_pixel(h.game_surfaces[0], [450.0, 300.0])
            .unwrap();
    assert!(pixel[0] < 120 && pixel[1] > 100, "{pixel:?}");
    let before = agp::renderer::pipeline_creations();
    let control = agp::renderer::pipeline::QuadPipeline::new(
        h.rt.gpu(),
        agp::renderer::target::OffscreenTarget::FORMAT,
    )
    .unwrap();
    assert_eq!(agp::renderer::pipeline_creations(), before + 1);
    drop(control);
}

fn consumer_breakout_gate(source: &str) -> Result<(), String> {
    let trace: serde_json::Value = serde_json::from_str(include_str!(
        "../../autographics-prototype/testdata/breakout/golden_trace.json"
    ))
    .unwrap();
    let mut h = Host::new(source)?;
    h.settle()?;
    h.click([80.0, 70.0]);
    h.settle()?;
    for t in 0..240 {
        h.direction = trace["actions"][t].as_i64().unwrap() as i32;
        h.tick(t as u64);
        h.settle()?;
        let g = &trace["ticks"][t];
        let ball = h.consumers[0].object(5).ok_or("missing ball")?;
        let paddle = h.consumers[0].object(4).ok_or("missing paddle")?;
        for (actual, key) in [
            (ball.transform.translation.x, "bx"),
            (ball.transform.translation.y, "by"),
            (paddle.transform.translation.x, "px"),
        ] {
            if (actual - g[key].as_f64().unwrap() as f32).abs() > 1e-3 {
                return Err(format!("consumer tick {t} {key}"));
            }
        }
        let p = h.scripts[0].publication.last().unwrap();
        if p.score != g["score"].as_i64().unwrap() as i32
            || p.sounds != g["sounds"].as_i64().unwrap() as i32
        {
            return Err(format!("consumer event tick {t}"));
        }
        // Replay input immediately after a collision: events belong to one
        // callback, not the next key/pointer callback's output.
        if !p.events.is_empty() {
            h.click([80.0, 70.0]);
            let id = h.scripts[0].id();
            h.scripts[0].enqueue(id, Action::Key(1))?;
            h.settle()?;
            if !h.errors.is_empty() {
                return Err(format!("post-contact input: {:?}", h.errors));
            }
        }
    }
    let expected: Vec<_> = trace["contacts_detail"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["t"].as_u64().unwrap() < 240)
        .collect();
    let actual: Vec<_> = h
        .events
        .iter()
        .filter(|(session, _, e)| *session == h.scripts[0].id() && e[0] == 0)
        .collect();
    if actual.len() != expected.len() {
        return Err("consumer contact count".into());
    }
    for ((_, _, event), expected) in actual.iter().zip(expected) {
        let k = match expected["kind"].as_str().unwrap() {
            "wall_left" => 0,
            "wall_right" => 1,
            "wall_bottom" => 2,
            "wall_top" => 3,
            "paddle" => 4,
            _ => 5 + expected["brick"].as_i64().unwrap() as i32,
        };
        let side = match expected["side"].as_str().unwrap() {
            "L" => 0,
            "R" => 1,
            "T" => 2,
            _ => 3,
        };
        if event[1] != k || event[2] != side {
            return Err("consumer contact identity".into());
        }
    }
    if !h.errors.is_empty() {
        return Err(format!("{:?}", h.errors));
    }
    Ok(())
}
#[test]
fn actual_graphics_consumer_breakout_and_rust_substitution_controls() {
    let _serial = GPU_TEST.lock().unwrap();
    let source = include_str!("../samples/breakout/main.at");
    consumer_breakout_gate(source).unwrap();
    let speed = consumer_breakout_gate(&source.replace("400.0", "401.0")).unwrap_err();
    assert!(
        speed.starts_with("consumer tick ") || speed.starts_with("consumer event tick "),
        "{speed}"
    );
    let collision =
        consumer_breakout_gate(&source.replace("if side >= 0", "if side > 99")).unwrap_err();
    assert!(
        collision.starts_with("consumer tick ") || collision.starts_with("consumer event tick "),
        "{collision}"
    );
}

fn overlay(gpu: &agp::renderer::GpuDevice, label: &str) -> agp::host::surface_runtime::TextOverlay {
    use agp::{core::scene::Color, render2d::TextRenderer, renderer::target::OffscreenTarget};
    let renderer = TextRenderer::new(
        gpu,
        OffscreenTarget::FORMAT,
        include_bytes!("../../autographics-prototype/assets/breakout/fonts/FiraMono-Medium.ttf"),
        33.0,
        "Score",
    )
    .unwrap();
    let glyphs = renderer.layout(label, 5.0, 5.0, Color::rgb(0.8, 0.2, 0.4));
    agp::host::surface_runtime::TextOverlay { renderer, glyphs }
}

#[test]
fn overlay_provenance_matrix_admission_and_replacement_preserves_target() {
    let _serial = GPU_TEST.lock().unwrap();
    use agp::{
        core::{composition::CompositionError, scene::Color},
        host::surface_runtime::{Game2dContent, SurfaceContent, SurfaceRuntime},
        renderer::{target::OffscreenTarget, GpuDevice, QuadPipeline},
    };
    let mut rt = SurfaceRuntime::new(GpuDevice::new().unwrap()).unwrap();
    let foreign = GpuDevice::new().unwrap();
    // Both ownership directions, homogeneous foreign, and raw provenance.
    for replacement in [false, true] {
        for (q, t) in [
            (0, 0),
            (0, 1),
            (1, 0),
            (1, 1),
            (2, 0),
            (0, 2),
            (1, 2),
            (2, 2),
        ] {
            let mut game = Game2dContent::simple(
                if q == 1 { &foreign } else { rt.gpu() },
                Color::rgb(0.1, 0.1, 0.1),
            );
            if q == 2 {
                game.quad = QuadPipeline::new_with(
                    rt.gpu().device(),
                    rt.gpu().queue(),
                    OffscreenTarget::FORMAT,
                )
                .unwrap();
            }
            if t != 2 {
                game.text_overlay =
                    Some(overlay(if t == 1 { &foreign } else { rt.gpu() }, "Score"));
            }
            let content = SurfaceContent::Game2d(Box::new(game));
            let valid = q == 0 && t != 1;
            let targets = rt.live_target_slots();
            let frames = rt.stats.frames;
            let result = if replacement {
                let mut good = Game2dContent::simple(rt.gpu(), Color::rgb(0.1, 0.1, 0.1));
                good.text_overlay = Some(overlay(rt.gpu(), "Score"));
                let id = rt
                    .add_surface([200.0, 100.0], SurfaceContent::Game2d(Box::new(good)))
                    .unwrap();
                rt.frame(None, &Default::default()).unwrap();
                let hash = rt.capture(id, "before").unwrap().0;
                *rt.content_mut(id).unwrap() = content;
                rt.graph.mark_content_dirty(id);
                let before = rt.stats.frames;
                let uploads = rt.upload_stats();
                let result = rt.frame(None, &Default::default());
                if !valid {
                    assert_eq!(rt.stats.frames, before);
                    assert_eq!(rt.upload_stats().delta_since(&uploads).bytes_rewritten, 0);
                    assert_eq!(rt.capture(id, "rejected").unwrap().0, hash);
                }
                // Restore valid ownership so subsequent cases exercise a healthy runtime.
                let mut good = Game2dContent::simple(rt.gpu(), Color::rgb(0.1, 0.1, 0.1));
                good.text_overlay = Some(overlay(rt.gpu(), "Score"));
                *rt.content_mut(id).unwrap() = SurfaceContent::Game2d(Box::new(good));
                result
            } else {
                rt.add_surface([200.0, 100.0], content).map(|_| ())
            };
            if valid {
                result.unwrap();
            } else if q == 2 {
                assert!(matches!(
                    result,
                    Err(CompositionError::DeviceUnverifiable { .. })
                ));
            } else {
                assert!(matches!(
                    result,
                    Err(CompositionError::DeviceMismatch { .. })
                ));
            }
            if !replacement && !valid {
                assert_eq!(rt.live_target_slots(), targets);
                assert_eq!(rt.stats.frames, frames);
            }
        }
    }
}

#[test]
fn overlay_upload_payloads_are_counted_once_in_all_aggregates() {
    let _serial = GPU_TEST.lock().unwrap();
    use agp::{
        core::scene::Color,
        host::surface_runtime::{Game2dContent, SurfaceContent, SurfaceRuntime},
        renderer::GpuDevice,
    };
    let mut hashes = Vec::new();
    for label in [None, Some("Score"), Some("")] {
        let mut rt = SurfaceRuntime::new(GpuDevice::new().unwrap()).unwrap();
        let mut game = Game2dContent::simple(rt.gpu(), Color::rgb(0.1, 0.1, 0.1));
        if let Some(label) = label {
            game.text_overlay = Some(overlay(rt.gpu(), label));
        }
        let id = rt
            .add_surface([200.0, 100.0], SurfaceContent::Game2d(Box::new(game)))
            .unwrap();
        let expected = match label {
            None => (1, 64),
            Some("") => (2, 128),
            _ => (3, 368),
        };
        for _ in 0..2 {
            let before = rt.upload_stats();
            let part = rt.content_uploads_of(id).unwrap();
            rt.graph.mark_content_dirty(id);
            rt.frame(None, &Default::default()).unwrap();
            let delta = rt.content_uploads_of(id).unwrap().delta_since(&part);
            assert_eq!(
                (delta.buffer_rewrites, delta.bytes_rewritten),
                expected,
                "{label:?}"
            );
            let all = rt.content_uploads();
            let single = rt.content_uploads_of(id).unwrap();
            assert_eq!(
                (all.buffer_rewrites, all.bytes_rewritten),
                (single.buffer_rewrites, single.bytes_rewritten)
            );
            let total = rt.upload_stats().delta_since(&before);
            assert_eq!((total.buffer_rewrites, total.bytes_rewritten), expected);
            let stable = rt.upload_stats();
            rt.frame(None, &Default::default()).unwrap();
            assert_eq!(
                rt.upload_stats().delta_since(&stable).bytes_rewritten,
                0,
                "clean warm frame"
            );
        }
        hashes.push(rt.capture(id, "upload").unwrap().0);
    }
    // The real Host mounts this child twice. Content uploads belong to the
    // child render, never to each view. Removing one mount keeps the same cost.
    let mut host = Host::new(source()).unwrap();
    host.settle().unwrap();
    let id = host.game_surfaces[0];
    let portal = host
        .rt
        .graph
        .portals()
        .iter()
        .find(|p| p.desc.child == id)
        .unwrap()
        .id;
    for shared in [true, false] {
        if !shared {
            host.rt.remove_portal(portal).unwrap();
        }
        let text = overlay(host.rt.gpu(), "Score");
        if let Some(SurfaceContent::Game2d(game)) = host.rt.content_mut(id) {
            game.text_overlay = Some(text);
        }
        host.rt.graph.mark_content_dirty(id);
        let before = host.rt.content_uploads_of(id).unwrap();
        let quad_before = match host.rt.content(id).unwrap() {
            SurfaceContent::Game2d(g) => g.quad.uploads.snapshot(),
            _ => unreachable!(),
        };
        host.rt.frame(None, &Default::default()).unwrap();
        let delta = host.rt.content_uploads_of(id).unwrap().delta_since(&before);
        let quad = match host.rt.content(id).unwrap() {
            SurfaceContent::Game2d(g) => g.quad.uploads.snapshot().delta_since(&quad_before),
            _ => unreachable!(),
        };
        assert_eq!(
            delta.buffer_rewrites - quad.buffer_rewrites,
            2,
            "shared={shared}"
        );
        assert_eq!(
            delta.bytes_rewritten - quad.bytes_rewritten,
            304,
            "shared={shared}"
        );
    }
    assert_ne!(hashes[0], hashes[1], "glyphs actually rendered");
    assert_eq!(hashes[0], hashes[2], "empty overlay draws no glyphs");
}
