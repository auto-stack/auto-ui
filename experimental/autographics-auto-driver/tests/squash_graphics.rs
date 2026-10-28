use agp::{host::surface_runtime::SurfaceContent, renderer::GpuDevice};
use autographics_auto_driver::{host::Host, squash::source};

fn upload_gate(host: &Host) -> bool {
    let mut content = agp::renderer::UploadStats::default();
    for id in [
        host.top,
        host.game_surfaces[0],
        host.game_surfaces[1],
        host.mesh_surface,
        host.panel,
    ] {
        match host.rt.content(id).unwrap() {
            SurfaceContent::Game2d(g) => {
                content.merge(&g.quad.uploads.snapshot());
                if let Some(t) = &g.text_overlay {
                    content.merge(&t.renderer.uploads.snapshot());
                }
            }
            SurfaceContent::Scene3d(c) => {
                content.merge(&c.mesh.uploads.snapshot());
                content.merge(&c.store.uploads.snapshot());
                if let Some(t) = &c.text_overlay {
                    content.merge(&t.renderer.uploads.snapshot());
                }
                if let Some(o) = &c.screen_overlay {
                    content.merge(&o.quad.uploads.snapshot());
                }
            }
            SurfaceContent::Iced(_) => {}
        }
    }
    let actual = host.rt.content_uploads();
    if actual.bytes_rewritten != content.bytes_rewritten
        || actual.buffer_rewrites != content.buffer_rewrites
    {
        return false;
    }
    let mut expected = host.rt.composition_upload_stats();
    expected.merge(&content);
    let actual = host.rt.upload_stats();
    actual.bytes_rewritten == expected.bytes_rewritten
        && actual.buffer_rewrites == expected.buffer_rewrites
}

#[test]
fn original_mesh_rotation_overlay_ownership_and_music_lifetime() {
    let mut h = Host::new_squash(&source()).unwrap();
    let audio_gate = |host: &Host| {
        host.squash
            .as_ref()
            .unwrap()
            .pin_music()
            .is_some_and(|m| m.is_playing() && m.loop_enabled)
    };
    assert!(
        !audio_gate(&h),
        "muted canonical sample must fail the audio oracle"
    );
    h.enable_audio().unwrap();
    h.enable_audio().unwrap();
    h.settle().unwrap();
    assert_eq!(h.squash.as_ref().unwrap().music_plays, 1);
    assert!(audio_gate(&h));
    assert!(h.squash.as_ref().unwrap().pin_music().unwrap().is_playing());
    let surface = h.game_surfaces[0];
    let initial = h.rt.capture(surface, "initial").unwrap();
    let target_count = h.rt.live_target_slots();
    {
        let q = h.squash.as_mut().unwrap();
        let id = q.scripts[0].id();
        q.scripts[0].tick(id, 0, [0, 0, 1]).unwrap();
        q.scripts[0].settle().unwrap();
    }
    h.settle().unwrap();
    let jumped = h.rt.capture(surface, "jumped").unwrap();
    assert_ne!(initial.0, jumped.0);
    // A separate reference simulation is an oracle, never an Auto-mode producer.
    use agp::{
        core::runtime::Runtime,
        games::squash::{player_model_matrix, SquashAction, SquashDriver},
    };
    let mut reference = SquashDriver::new(27);
    let mut rt = Runtime::new(std::time::Duration::from_nanos(8_333_333), 27);
    rt.step(
        &mut reference,
        vec![SquashAction {
            jump: true,
            ..Default::default()
        }],
    );
    let packet = h.squash.as_ref().unwrap().accepted[0].as_ref().unwrap();
    let object = packet.objects.iter().find(|o| o.resource == 0).unwrap();
    let expected = player_model_matrix(
        reference.player,
        reference.facing_yaw,
        reference.vy,
        reference.anim_time,
    );
    for (a, b) in object
        .matrix()
        .to_cols_array()
        .iter()
        .zip(expected.to_cols_array())
    {
        assert!(
            (a - b).abs() < 0.00001,
            "Auto animation/pitch matrix {a}/{b}"
        );
    }
    let correct = object.matrix();
    if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
        for instance in &mut c.instances[5..] {
            instance.model = glam::Mat4::from_translation(correct.w_axis.truncate());
        }
    }
    h.rt.frame(None, &Default::default()).unwrap();
    assert_ne!(
        h.rt.capture(surface, "rotation omitted").unwrap().0,
        jumped.0,
        "real rotation omission must change pixels"
    );
    // Replaying a complete accepted output is rejected by the independent real consumer.
    let prior = h.squash.as_ref().unwrap().accepted[0].clone().unwrap();
    h.squash.as_mut().unwrap().scripts[0]
        .outputs
        .push_back(prior.clone());
    assert!(h.frame(None).unwrap_err().contains("sequence"));
    assert_eq!(
        h.squash.as_ref().unwrap().accepted[0].as_ref(),
        Some(&prior)
    );
    // Foreign individual resources must reject before any surface is rendered.
    let foreign = GpuDevice::new().unwrap();
    for mode in 0..6 {
        let before = h.rt.capture(surface, "before rejection").unwrap().0;
        let uploads_before = h.rt.upload_stats().bytes_rewritten;
        match mode {
            0 => {
                let new=agp::render2d::TextRenderer::new(&foreign,agp::renderer::target::OffscreenTarget::FORMAT,include_bytes!("../../autographics-prototype/assets/squash_the_creeps/fonts/Montserrat-Medium.ttf"),32.0,"Score: 0").unwrap();
                let old = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    Some(std::mem::replace(
                        &mut c.text_overlay.as_mut().unwrap().renderer,
                        new,
                    ))
                } else {
                    None
                };
                let accepted = h.squash.as_ref().unwrap().accepted[0].clone().unwrap();
                let mut candidate = accepted.clone();
                candidate.sequence += 1;
                candidate.objects[0].fields[0] += 1.0;
                h.squash.as_mut().unwrap().scripts[0]
                    .outputs
                    .push_back(candidate);
                assert!(h.frame(None).unwrap_err().contains("resource admission"));
                assert_eq!(
                    h.squash.as_ref().unwrap().accepted[0].as_ref(),
                    Some(&accepted)
                );
                assert_eq!(
                    h.squash.as_ref().unwrap().consumer_publications[0].last(),
                    Some(&accepted)
                );
                if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    c.text_overlay.as_mut().unwrap().renderer = old.unwrap();
                }
            }
            1 => {
                let mut store = agp::render3d::MeshStore::default();
                let model = agp::assets::load_glb_bytes(include_bytes!(
                    "../../autographics-prototype/assets/squash_the_creeps/art/player.glb"
                ))
                .unwrap();
                store.get_or_upload(&foreign, &model, 0).unwrap();
                let old = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    Some(std::mem::replace(&mut c.store, store))
                } else {
                    None
                };
                assert!(h.rt.frame(None, &Default::default()).is_err());
                if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    c.store = old.unwrap();
                }
            }
            2 => {
                let old = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    let old = c.instances[0].mesh.clone();
                    c.instances[0].mesh.0 = "missing".into();
                    Some(old)
                } else {
                    None
                };
                assert!(h.rt.frame(None, &Default::default()).is_err());
                if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    c.instances[0].mesh = old.unwrap();
                }
            }
            3 => {
                let replacement = agp::renderer::QuadPipeline::new(
                    &foreign,
                    agp::renderer::target::OffscreenTarget::FORMAT,
                )
                .unwrap();
                let old = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    Some(std::mem::replace(
                        &mut c.screen_overlay.as_mut().unwrap().quad,
                        replacement,
                    ))
                } else {
                    None
                };
                assert!(h.rt.frame(None, &Default::default()).is_err());
                if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    c.screen_overlay.as_mut().unwrap().quad = old.unwrap();
                }
            }
            _ => {
                let replacement = if mode == 4 {
                    agp::render3d::MeshPipeline::new(
                        &foreign,
                        agp::renderer::target::OffscreenTarget::FORMAT,
                    )
                    .unwrap()
                } else {
                    agp::render3d::MeshPipeline::new_with(
                        h.rt.gpu().device(),
                        h.rt.gpu().queue(),
                        agp::renderer::target::OffscreenTarget::FORMAT,
                    )
                    .unwrap()
                };
                let old = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    Some(std::mem::replace(&mut c.mesh, replacement))
                } else {
                    None
                };
                assert!(h.rt.frame(None, &Default::default()).is_err());
                if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(surface) {
                    c.mesh = old.unwrap();
                }
            }
        }
        assert_eq!(
            h.rt.upload_stats().bytes_rewritten,
            uploads_before,
            "resource rejection left real writes"
        );
        assert_eq!(h.rt.capture(surface, "after rejection").unwrap().0, before);
    }
    let exact_model_bytes = [
        agp::assets::load_glb_bytes(include_bytes!(
            "../../autographics-prototype/assets/squash_the_creeps/art/player.glb"
        ))
        .unwrap(),
        agp::assets::load_glb_bytes(include_bytes!(
            "../../autographics-prototype/assets/squash_the_creeps/art/mob.glb"
        ))
        .unwrap(),
    ]
    .iter()
    .flat_map(|m| &m.meshes)
    .map(|m| m.positions.len() * 40 + m.indices.len() * 2)
    .sum::<usize>()
        + [agp::render3d::ground_mesh(), agp::render3d::cylinder_mesh()]
            .iter()
            .map(|m| m.positions.len() * 40 + m.indices.len() * 2)
            .sum::<usize>();
    let store_before = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content(surface) {
        c.store.uploads.snapshot().bytes_rewritten
    } else {
        0
    };
    assert_eq!(store_before, exact_model_bytes as u64);
    assert!(upload_gate(&h));
    h.rt.set_content_upload_aggregation(false);
    assert!(!upload_gate(&h), "omitted uploads escaped formal sum gate");
    h.rt.set_content_upload_aggregation(true);
    h.rt.frame(None, &Default::default()).unwrap();
    let full = h.rt.upload_stats();
    let content = h.rt.content_uploads();
    assert!(
        full.bytes_rewritten >= content.bytes_rewritten && content.bytes_rewritten >= store_before
    );
    let pipelines_before = agp::renderer::pipeline_creations();
    let cache_pin = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content(surface) {
        let key = &c.instances[0].mesh;
        c.store.pin(&key.0, &key.1).unwrap()
    } else {
        panic!("scene missing")
    };
    for _ in 0..20 {
        let owners: Vec<_> = h
            .squash
            .as_ref()
            .unwrap()
            .scripts
            .iter()
            .map(|s| s.session.owner_probe())
            .collect();
        h.reset();
        h.settle().unwrap();
        assert!(owners.iter().all(|f| !f()));
        assert_eq!(h.rt.live_target_slots(), target_count);
        assert_eq!(h.squash.as_ref().unwrap().music_plays, 1);
        assert_eq!(agp::renderer::pipeline_creations(), pipelines_before);
        if let Some(SurfaceContent::Scene3d(c)) = h.rt.content(surface) {
            let key = &c.instances[0].mesh;
            assert!(std::sync::Arc::ptr_eq(
                &cache_pin,
                &c.store.pin(&key.0, &key.1).unwrap()
            ));
            assert_eq!(c.store.uploads.snapshot().bytes_rewritten, store_before);
        }
    }
    drop(cache_pin);
    println!("original GLB/HUD/rotation pixels; 6 device/missing-resource refusals; mesh upload={store_before} bytes; 20 retry owners released; 1 music play");
    let mesh_pin = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content(surface) {
        let key = &c.instances[0].mesh;
        c.store.pin(&key.0, &key.1).unwrap()
    } else {
        panic!("scene missing")
    };
    let mesh_owner = std::sync::Arc::downgrade(&mesh_pin);
    let pin = h.squash.as_ref().unwrap().pin_music().unwrap();
    let began = std::time::Instant::now();
    let mut prior = pin.position_secs();
    let mut wrapped = false;
    while began.elapsed().as_secs_f64() < pin.duration_secs + 2.0 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let position = pin.position_secs();
        if position + 1.0 < prior {
            wrapped = true;
            break;
        }
        prior = position;
    }
    assert!(
        wrapped && pin.is_playing(),
        "real original OGG did not loop through Kira"
    );
    println!(
        "actual original OGG wrapped, duration={} seconds",
        pin.duration_secs
    );
    let owner = h.squash.as_ref().unwrap().music_owner_probe().unwrap();
    h.squash.as_mut().unwrap().close();
    assert!(
        pin.is_playing(),
        "external pin retains actual looping stream"
    );
    assert!(owner(), "external legal music owner must remain live");
    drop(pin);
    assert!(!owner(), "last actual manager/sound owner leaked");
    drop(h);
    assert!(mesh_owner.upgrade().is_some());
    drop(mesh_pin);
    assert!(
        mesh_owner.upgrade().is_none(),
        "actual mesh-buffer owner leaked"
    );
}
