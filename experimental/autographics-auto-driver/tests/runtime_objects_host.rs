//! auto-ui PLAN-001 runtime objects host tests (contract V-03/V-04): the real
//! Host/Play/GPU path. Spawned instances render through the non-persistent
//! adapter (2D draws, 3D mesh instances including the multi-mesh GLB
//! template), move/hide/despawn co-publish with the character and camera in
//! one transaction, preparation failure keeps the last-good pair, the frozen
//! slot/mesh envelopes reject at construction, and twenty start/stop churn
//! cycles close the VM owner and keep mesh uploads flat (no per-birth work).
//! Windowless GPU with public logical inputs only. Run single-threaded:
//! cargo test --locked --test runtime_objects_host -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::Action,
    runtime_objects::Dim,
};
use std::path::Path;
use std::time::Instant;

const GLB: &[u8] = include_bytes!("../research/asset-probe/two-parts.glb");

fn document2d() -> Document {
    let mut d = Document::empty("objects2d-host".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [2.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Quad);
    floor.translation = [15.0, -0.5, 0.0];
    floor.scale = [34.0, 1.0, 1.0];
    d.objects.push(floor);
    // Spawn templates stay INVISIBLE as static nodes: every visible circle in
    // the published stream is a runtime instance, never the template.
    let mut coin = Object::new(3, Kind::Circle);
    coin.translation = [8.0, 2.0, 0.0];
    coin.scale = [0.6, 0.6, 1.0];
    coin.rgba = [0.95, 0.8, 0.2, 1.0];
    coin.visible = false;
    d.objects.push(coin);
    let mut gem = Object::new(4, Kind::Quad);
    gem.translation = [20.0, 2.5, 0.0];
    gem.scale = [0.5, 0.5, 1.0];
    gem.rgba = [0.3, 0.9, 0.5, 1.0];
    gem.visible = false;
    d.objects.push(gem);
    d.validate().expect("document valid");
    d
}

fn document3d() -> Document {
    let mut d = Document::empty("objects3d-host".into());
    let mut character = Object::new(1, Kind::Cylinder);
    character.translation = [0.0, 0.5, 0.0];
    character.scale = [0.4, 0.5, 0.4];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Cylinder);
    floor.translation = [0.0, -0.5, 0.0];
    floor.scale = [10.0, 0.5, 12.0];
    d.objects.push(floor);
    let mut crystal = Object::new(3, Kind::Mob);
    crystal.translation = [3.0, 1.0, -3.0];
    crystal.scale = [0.5, 1.0, 0.5];
    crystal.visible = false;
    d.objects.push(crystal);
    let mut beacon = Object::new(4, Kind::Cylinder);
    beacon.translation = [-3.0, 1.5, -5.0];
    beacon.scale = [0.3, 1.5, 0.3];
    beacon.visible = false;
    d.objects.push(beacon);
    d.validate().expect("document valid");
    d
}

fn document3d_with_multi_mesh_template() -> (
    Document,
    autographics_auto_driver::authoring::assets::Registry,
) {
    let mut d = document3d();
    let asset = autographics_auto_driver::authoring::assets::decode(
        GLB.to_vec(),
        Some("two-parts.glb".into()),
    )
    .expect("glb decode");
    let mut model = Object::new(5, Kind::Model);
    model.translation = [6.0, 1.0, -6.0];
    model.scale = [1.0, 1.0, 1.0];
    model.visible = false;
    model.resource = Some(asset.identity.key());
    d.objects.push(model);
    d.assets.push(asset.identity.clone());
    let registry = autographics_auto_driver::authoring::assets::Registry::from([(
        asset.identity.key(),
        asset,
    )]);
    d.validate().expect("document valid");
    (d, registry)
}

/// 2D controller: bind, camera, spawn the coin at slot 0, and move it by
/// `per_tick` world units on every tick (the direct trajectory oracle).
fn source2d(per_tick: f32) -> String {
    format!(
        "var ok=0\nvar h=0\nvar x=8.0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n h=ag_object_spawn(0, 3, x, 2.0, 0.0)\n}}\nfn app_update(dt float) {{\n x=x+{per_tick:?}\n ok=ok+ag_object_move(h, x, 2.0, 0.0)\n}}\n"
    )
}

/// 3D controller: bind, camera, spawn the given template at slot 0 and keep
/// it still (the render oracle); `template` is the stable document id.
fn source3d(template: i32, x: f32) -> String {
    format!(
        "var ok=0\nvar h=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_camera_orbit_3d(1, 8.0, 45.0, 0, 0.0)\n h=ag_object_spawn(0, {template}, {x:?}, 1.0, -3.0)\n}}\nfn app_update(dt float) {{\n}}\n"
    )
}

fn snapshot(api: Api, source: String, name: &str) -> SourceSnapshot {
    SourceSnapshot::from_bytes_for(Path::new(name), source.as_bytes().to_vec(), api)
        .expect("admission")
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
}

fn start2d(h: &mut Host, per_tick: f32) {
    h.start_user_snapshot(snapshot(Api::Objects2V1, source2d(per_tick), "host-2d.at"))
        .expect("start");
    h.pause_play("");
    settle(h);
}

fn start3d(h: &mut Host, template: i32, x: f32) {
    h.start_user_snapshot(snapshot(
        Api::Objects3V1,
        source3d(template, x),
        "host-3d.at",
    ))
    .expect("start");
    h.pause_play("");
    settle(h);
}

fn step(h: &mut Host) {
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(h);
}

fn two_circles(h: &Host) -> Vec<agp::renderer::pipeline::DrawCmd> {
    match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g
            .commands
            .iter()
            .filter(|c| c.shape == agp::renderer::pipeline::Shape::Circle)
            .cloned()
            .collect(),
        _ => panic!("2D game content"),
    }
}

fn three_instances(h: &Host) -> Vec<agp::render3d::MeshInstance> {
    match h.rt.content(h.three) {
        Some(agp::host::surface_runtime::SurfaceContent::Scene3d(c)) => c.instances.clone(),
        _ => panic!("3D scene content"),
    }
}

// ------------------------------------------------------------- V-03 (2D) --

fn spawned_2d_instances_co_publish_and_move_with_the_frame_body() {
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    start2d(&mut h, 2.0);
    let play = h.play.as_ref().unwrap();
    let frame = play.last.as_ref().expect("tick 0 frame");
    let instances = frame.instances.as_ref().expect("objects block rides");
    assert_eq!(instances.dim, Dim::TwoD);
    assert_eq!(instances.live_count(), 1, "one spawned coin");
    let slot = instances.get(0).unwrap();
    assert_eq!((slot.generation, slot.template_id), (1, 3));
    assert_eq!(slot.translation, [8.0, 2.0, 0.0]);
    // The published stream renders the instance through the frozen template
    // shape; the hidden template contributes nothing.
    let circles = two_circles(&h);
    assert_eq!(circles.len(), 1, "exactly the runtime instance");
    // The objects profile carries the follow camera: instance draws project
    // through the live letterbox scale, not the editing zoom.
    let cam = h
        .play
        .as_ref()
        .unwrap()
        .camera_frame()
        .expect("follow camera")
        .clone();
    let (scale, _, _) = cam.letterbox_2d().expect("letterbox");
    let first = circles[0].center;
    assert!(
        (circles[0].size[0] - 0.6 * scale.x).abs() < 1e-3,
        "template scale {:?} letterbox scale {}",
        circles[0].size,
        scale.x
    );
    // One tick: the coin moves +2 world units; the published draw follows in
    // the SAME transaction (no stale pixel with a new object table).
    step(&mut h);
    let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
    assert_eq!(frame.tick, 1);
    let slot = frame.instances.as_ref().unwrap().get(0).unwrap();
    assert_eq!(slot.translation, [10.0, 2.0, 0.0]);
    let circles = two_circles(&h);
    assert_eq!(circles.len(), 1);
    let moved = circles[0].center;
    let delta = [moved[0] - first[0], moved[1] - first[1]];
    assert!(
        (delta[0] - 2.0 * scale.x).abs() < 1e-2 && delta[1].abs() < 1e-2,
        "screen delta must match the world move through the live camera: {delta:?}"
    );
    // Hide: identity stays in the admitted table, the draw disappears.
    let hide = SourceSnapshot::from_bytes_for(
        Path::new("host-2d-hide.at"),
        format!(
            "var ok=0\nvar h=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n h=ag_object_spawn(0, 3, 8.0, 2.0, 0.0)\n ok=ok+ag_object_visible(h, 0)\n}}\nfn app_update(dt float) {{\n}}\n"
        )
        .as_bytes()
        .to_vec(),
        Api::Objects2V1,
    )
    .expect("hide admission");
    h.stop_play();
    h.start_user_snapshot(hide).expect("restart");
    h.pause_play("");
    settle(&mut h);
    let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
    let slot = frame.instances.as_ref().unwrap().get(0).unwrap();
    assert!(
        slot.live() && !slot.visible,
        "hidden instance keeps identity"
    );
    assert_eq!(two_circles(&h).len(), 0, "hidden instance draws nothing");
}

fn preparation_failure_keeps_the_last_good_frame_and_instances_body() {
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    start2d(&mut h, 2.0);
    step(&mut h);
    let good = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    let good_circles = two_circles(&h);
    assert!(!good_circles.is_empty());
    // Injected preparation failure: the run faults and the LAST COMPLETE
    // frame (objects included) stays published.
    h.fail_resource = true;
    step(&mut h);
    h.fail_resource = false;
    let play = h.play.as_ref().unwrap();
    assert_eq!(
        play.state,
        autographics_auto_driver::authoring::play::State::Faulted
    );
    assert_eq!(play.last.as_ref().unwrap().tick, good.tick);
    let instances = play.last.as_ref().unwrap().instances.as_ref().unwrap();
    assert_eq!(instances.get(0).unwrap().translation, [10.0, 2.0, 0.0]);
    assert_eq!(
        two_circles(&h).iter().map(|c| c.center).collect::<Vec<_>>(),
        good_circles.iter().map(|c| c.center).collect::<Vec<_>>(),
        "last-good draw stays"
    );
    assert!(
        play.owner_probe().map_or(true, |f| !f()),
        "fault closes the VM"
    );
}

// ------------------------------------------------------------- V-03 (3D) --

fn spawned_3d_instances_render_all_template_meshes_body() {
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    start3d(&mut h, 3, -2.0);
    if let Some(error) = &h.play.as_ref().unwrap().error {
        panic!("3D run faulted: {error}");
    }
    let play = h.play.as_ref().unwrap();
    let frame = play.last.as_ref().unwrap();
    let slot = frame.instances.as_ref().unwrap().get(0).unwrap();
    assert_eq!(slot.translation, [-2.0, 1.0, -3.0]);
    // The spawned instance renders its full template mesh set at the
    // admitted world position.
    let instances = three_instances(&h);
    let hits = instances
        .iter()
        .filter(|i| {
            let t = i.model.w_axis.truncate();
            (t.x - (-2.0)).abs() < 1e-4 && (t.z - (-3.0)).abs() < 1e-4
        })
        .count();
    assert!(hits >= 1, "the spawned instance renders its meshes");
    // Move the instance one tick: the published transform follows in the SAME
    // transaction (the update moves the crystal by +1 world x).
    let mover = SourceSnapshot::from_bytes_for(
        Path::new("host-3d-move.at"),
        format!(
            "var ok=0
var h=0
fn app_start() {{
 ag_set_local(0,1,0.5)
 ok=ok+ag_char3_bind(1)
 ok=ok+ag_char3_solid(2)
 ok=ok+ag_camera_orbit_3d(1, 8.0, 45.0, 0, 0.0)
 h=ag_object_spawn(0, 3, -2.0, 1.0, -3.0)
}}
fn app_update(dt float) {{
 ok=ok+ag_object_move(h, -1.0, 1.0, -3.0)
}}
"
        )
        .as_bytes()
        .to_vec(),
        Api::Objects3V1,
    )
    .expect("3d move admission");
    h.stop_play();
    h.start_user_snapshot(mover).expect("restart");
    h.pause_play("");
    settle(&mut h);
    step(&mut h);
    let instances = three_instances(&h);
    let hits = instances
        .iter()
        .filter(|i| {
            let t = i.model.w_axis.truncate();
            (t.x - (-1.0)).abs() < 1e-4 && (t.z - (-3.0)).abs() < 1e-4
        })
        .count();
    assert!(
        hits >= 1,
        "the moved instance publishes at its new position"
    );
    let stale = instances
        .iter()
        .filter(|i| {
            let t = i.model.w_axis.truncate();
            (t.x - (-2.0)).abs() < 1e-4 && (t.z - (-3.0)).abs() < 1e-4
        })
        .count();
    assert_eq!(stale, 0, "no stale transform stays published");
    // The adapter never impersonates a selectable document object: selection
    // stays None.
    assert_eq!(h.editor.selection, None);
}

fn multi_mesh_template_spawns_every_mesh_not_just_the_first_body() {
    let (doc, assets) = document3d_with_multi_mesh_template();
    let mut h = Host::with_project(doc, assets, None).expect("host");
    start3d(&mut h, 5, 6.0);
    if let Some(error) = &h.play.as_ref().unwrap().error {
        panic!("multi-mesh run faulted: {error}");
    }
    let play = h.play.as_ref().unwrap();
    let slot = play
        .last
        .as_ref()
        .unwrap()
        .instances
        .as_ref()
        .unwrap()
        .get(0)
        .unwrap()
        .clone();
    assert_eq!(slot.template_id, 5, "the Model template spawned");
    let instances = three_instances(&h);
    let at_spawn: Vec<_> = instances
        .iter()
        .filter(|i| {
            let t = i.model.w_axis.truncate();
            (t.x - 6.0).abs() < 1e-3 && (t.z - (-3.0)).abs() < 1e-3
        })
        .collect();
    assert!(
        at_spawn.len() >= 2,
        "the multi-mesh GLB template must render ALL its meshes, got {}",
        at_spawn.len()
    );
    let names: std::collections::BTreeSet<_> = at_spawn.iter().map(|i| i.mesh.1.as_str()).collect();
    assert!(names.len() >= 2, "distinct mesh parts, got {names:?}");
}

fn frozen_envelopes_reject_at_construction_body() {
    // Slot capacity: 121 base nodes + 8 reserved slots exceed 128.
    let mut d = Document::empty("objects2d-wide".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [2.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    for i in 1..121 {
        let mut node = Object::new(i as u64 + 1, Kind::Quad);
        node.translation = [i as f32 * 3.0, 5.0, 0.0];
        node.visible = false;
        d.objects.push(node);
    }
    let mut h = Host::with_project(d, Default::default(), None).expect("host");
    let error = h
        .start_user_snapshot(snapshot(Api::Objects2V1, source2d(2.0), "host-wide.at"))
        .expect_err("wide document must be rejected");
    assert!(error.contains("objects slot capacity"), "{error}");
    assert!(h.play.is_none(), "no run exists after the rejection");
    // A 120-node document stays inside the envelope and starts.
    let mut d = Document::empty("objects2d-fit".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [2.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    for i in 1..120 {
        let mut node = Object::new(i as u64 + 1, Kind::Quad);
        node.translation = [i as f32 * 3.0, 5.0, 0.0];
        node.visible = false;
        d.objects.push(node);
    }
    let mut h = Host::with_project(d, Default::default(), None).expect("host");
    start2d(&mut h, 2.0);
    if let Some(error) = &h.play.as_ref().unwrap().error {
        panic!("fit document faulted: {error}");
    }
    assert_eq!(
        h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .instances
            .as_ref()
            .unwrap()
            .live_count(),
        1
    );
}

/// Twenty object start/stop cycles per dimension over move/fault/Stop: the VM
/// weak owner closes on every Stop, the editing presentation restores, and
/// after the first warm cycle the per-cycle upload deltas stay exactly flat
/// (shared meshes are never re-uploaded for a new birth). Writes the
/// env-gated host report for the checker.
fn twenty_object_lifecycle_cycles_release_owners_and_keep_uploads_flat_body() {
    let report_dir = std::env::var("RUNTIME_OBJECTS_REPORT_DIR").ok();
    let snapshot2d = snapshot(Api::Objects2V1, source2d(2.0), "cycle-2d.at");
    let snapshot3d = snapshot(Api::Objects3V1, source3d(3, 3.0), "cycle-3d.at");
    let mut hosts = [
        Host::with_project(document3d(), Default::default(), None).expect("host3d"),
        Host::with_project(document2d(), Default::default(), None).expect("host2d"),
    ];
    let mut deltas: [Vec<(u64, bool)>; 2] = [Vec::new(), Vec::new()];
    let mut births = 0u64;
    for cycle in 0..20 {
        let which = cycle % 2;
        let h = &mut hosts[which];
        let before = h.rt.upload_stats().buffer_rewrites;
        h.start_user_snapshot(if which == 0 {
            snapshot3d.clone()
        } else {
            snapshot2d.clone()
        })
        .expect("start");
        h.pause_play("");
        settle(h);
        let alive = h.play.as_ref().unwrap().owner_probe().expect("VM owner");
        assert!(alive(), "cycle {cycle}: owner alive after init");
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
        let instances = frame.instances.as_ref().expect("objects block");
        assert_eq!(instances.live_count(), 1, "cycle {cycle}");
        births += instances.live_count() as u64;
        // A move keeps the object table and the pixels in one transaction.
        if which == 1 {
            step(h);
            assert_eq!(
                h.play
                    .as_ref()
                    .unwrap()
                    .last
                    .as_ref()
                    .unwrap()
                    .instances
                    .as_ref()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .translation,
                [10.0, 2.0, 0.0]
            );
        }
        // A fault every fifth cycle keeps the last-good pair.
        if cycle % 5 == 4 {
            h.play.as_mut().unwrap().resume().expect("resume");
            h.play
                .as_mut()
                .unwrap()
                .dispatch(Action::default())
                .expect("dispatch");
            h.fail_resource = true;
            settle(h);
            h.fail_resource = false;
            assert_eq!(
                h.play.as_ref().unwrap().state,
                autographics_auto_driver::authoring::play::State::Faulted,
                "cycle {cycle}"
            );
        }
        h.stop_play();
        h.frame(None).ok();
        assert!(
            h.play.is_none() || !h.play.as_ref().unwrap().owner_probe().map_or(true, |f| f()),
            "cycle {cycle}: Stop closes the VM owner"
        );
        let faulted = cycle % 5 == 4;
        deltas[which].push((h.rt.upload_stats().buffer_rewrites - before, faulted));
    }
    for (which, per_host) in deltas.iter().enumerate() {
        let warm = per_host.iter().find(|(_, f)| !f).map(|(v, _)| *v).unwrap();
        let outliers: Vec<u64> = per_host[1..]
            .iter()
            .filter(|(v, f)| !f && *v != warm)
            .map(|(v, _)| *v)
            .collect();
        assert!(
            outliers.is_empty(),
            "host {which} upload outliers (warm {warm}): {per_host:?}"
        );
    }
    assert!(births >= 20, "every cycle birthed one instance");
    if let Some(dir) = report_dir {
        let report = serde_json::json!({
            "schema": "autographics.runtime-objects-host/1",
            "completed": true,
            "cycles": 20,
            "births": births,
            "upload_deltas_3d": deltas[0].iter().map(|(v, f)| serde_json::json!({"delta": v, "faulted": f})).collect::<Vec<_>>(),
            "upload_deltas_2d": deltas[1].iter().map(|(v, f)| serde_json::json!({"delta": v, "faulted": f})).collect::<Vec<_>>(),
        });
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(
            std::path::Path::new(&dir).join("runtime-objects-host.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .expect("write host report");
        println!("host report written to {dir}");
    }
}

/// The assemblies compile four full trusted units plus the user source; the
/// combined parse depth overflows the default libtest stack, so every test
/// body runs on a dedicated big-stack thread.
fn big_stack(body: fn()) {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, body)
            .expect("test thread")
            .join()
            .expect("test body panicked");
    });
}

#[test]
fn spawned_2d_instances_co_publish_and_move_with_the_frame() {
    big_stack(spawned_2d_instances_co_publish_and_move_with_the_frame_body);
}

#[test]
fn preparation_failure_keeps_the_last_good_frame_and_instances() {
    big_stack(preparation_failure_keeps_the_last_good_frame_and_instances_body);
}

#[test]
fn spawned_3d_instances_render_all_template_meshes() {
    big_stack(spawned_3d_instances_render_all_template_meshes_body);
}

#[test]
fn multi_mesh_template_spawns_every_mesh_not_just_the_first() {
    big_stack(multi_mesh_template_spawns_every_mesh_not_just_the_first_body);
}

#[test]
fn frozen_envelopes_reject_at_construction() {
    big_stack(frozen_envelopes_reject_at_construction_body);
}

#[test]
fn twenty_object_lifecycle_cycles_release_owners_and_keep_uploads_flat() {
    big_stack(twenty_object_lifecycle_cycles_release_owners_and_keep_uploads_flat_body);
}

// --------------------------------------------------- fresh-process sampling --

fn sampler_document(shape: &str) -> Document {
    if shape.contains("wide120") {
        let two_d = shape.contains("2d");
        let mut d = Document::empty(format!("objects-{shape}"));
        let mut character = Object::new(1, Kind::Quad);
        character.translation = [2.0, 1.0, 0.0];
        character.scale = [0.8, 1.0, 1.0];
        d.objects.push(character);
        for i in 1..120 {
            let mut node = Object::new(
                i as u64 + 1,
                if two_d { Kind::Quad } else { Kind::Cylinder },
            );
            node.translation = [i as f32 * 3.0, if two_d { 5.0 } else { 0.5 }, 0.0];
            node.scale = if two_d {
                [0.8, 1.0, 1.0]
            } else {
                [0.4, 0.5, 0.4]
            };
            node.visible = false;
            d.objects.push(node);
        }
        d.validate().expect("document valid");
        return d;
    }
    if shape.contains("dense64") {
        let mut d = Document::empty(format!("objects-{shape}"));
        let mut character = Object::new(1, Kind::Cylinder);
        character.translation = [0.0, 0.5, 0.0];
        character.scale = [0.4, 0.5, 0.4];
        d.objects.push(character);
        let mut floor = Object::new(2, Kind::Cylinder);
        floor.translation = [0.0, -0.5, 0.0];
        floor.scale = [10.0, 0.5, 10.0];
        d.objects.push(floor);
        for i in 2..64 {
            let mut node = Object::new(i as u64 + 1, Kind::Mob);
            node.translation = [i as f32 * 0.4, 1.0, -2.0];
            node.scale = [0.3, 0.6, 0.3];
            node.visible = false;
            d.objects.push(node);
        }
        d.validate().expect("document valid");
        return d;
    }
    if shape.contains("2d") {
        document2d()
    } else {
        document3d()
    }
}

fn sampler_source(shape: &str) -> String {
    // Spawn into every reserved slot during app_start, then move them all on
    // every tick: the steady state is the full 8-instance envelope.
    let (bind, camera) = if shape.contains("2d") {
        (
            "ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n",
            "ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n",
        )
    } else {
        (
            "ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
            "ok=ok+ag_camera_orbit_3d(1, 8.0, 45.0, 0, 0.0)\n",
        )
    };
    let mut spawns = String::new();
    let template = if shape.contains("2d") { 3 } else { 4 };
    for slot in 0..8i32 {
        spawns.push_str(&format!(
            " h{slot}=ag_object_spawn({slot}, {template}, {slot}.0 + 1.0, 2.0, 0.0)\n"
        ));
    }
    let mut moves = String::new();
    for slot in 0..8i32 {
        moves.push_str(&format!(
            " ok=ok+ag_object_move(h{slot}, {slot}.0 + step * 0.05, 2.0, 0.0)\n"
        ));
    }
    format!(
        "var ok=0\nvar step=0\nvar h0=0\nvar h1=0\nvar h2=0\nvar h3=0\nvar h4=0\nvar h5=0\nvar h6=0\nvar h7=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n {bind}{camera}{spawns}}}\nfn app_update(dt float) {{\n step=step+1\n{moves}}}\n"
    )
}

/// One fresh process per shape: the full Host path (8 live instances, camera
/// block, objects block, prepare/publish) for warm steps, reporting per-tick
/// whole-transaction wall time. Shape grammar:
/// objects{2d|3d}-{regular|wide120|dense64}.
#[test]
fn runtime_objects_sampler_child() {
    let Ok(shape) = std::env::var("RUNTIME_OBJECTS_SAMPLER") else {
        return;
    };
    let out = std::env::var("RUNTIME_OBJECTS_SAMPLER_REPORT").expect("sampler report path");
    let api = if shape.contains("2d") {
        Api::Objects2V1
    } else {
        Api::Objects3V1
    };
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("sampler-objects.at"),
        sampler_source(&shape).as_bytes().to_vec(),
        api,
    )
    .expect("admission");
    let mut h =
        Host::with_project(sampler_document(&shape), Default::default(), None).expect("host");
    h.start_user_snapshot(snap).expect("start");
    h.pause_play("");
    settle(&mut h);
    if let Some(error) = &h.play.as_ref().unwrap().error {
        panic!("sampler {shape} faulted: {error}");
    }
    let mut tick_ns = Vec::new();
    for step_index in 1..=60i32 {
        h.play.as_mut().unwrap().resume().expect("resume");
        let begin = Instant::now();
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .expect("dispatch");
        h.pause_play("");
        settle(&mut h);
        // The first ten steps pay one-time GPU/first-frame warm-up and stay
        // out of the steady-state sample (874 sampler methodology).
        if step_index > 10 {
            tick_ns.push(begin.elapsed().as_nanos() as u64);
        }
        let play = h.play.as_ref().unwrap();
        assert!(
            play.error.is_none(),
            "sampler step {step_index}: {:?}",
            play.error
        );
        assert_eq!(
            play.last
                .as_ref()
                .unwrap()
                .instances
                .as_ref()
                .unwrap()
                .live_count(),
            8,
            "the full slot envelope stays live"
        );
    }
    tick_ns.sort();
    let p50 = tick_ns[tick_ns.len() / 2];
    let p95 = tick_ns[tick_ns.len() * 95 / 100];
    let report = serde_json::json!({
        "shape": shape,
        "steps": 50,
        "objects_block": true,
        "live_instances": 8,
        "tick_ns_p50": p50,
        "tick_ns_p95": p95,
        "max_ns": tick_ns.last().copied().unwrap_or(0),
    });
    std::fs::write(&out, serde_json::to_vec(&report).unwrap()).expect("write sample");
}

/// Five fresh processes per shape across objects2d/3d x regular/wide120/dense64;
/// aggregates the frozen sampling report for tools/check_runtime_objects.py.
/// Gate: RUNTIME_OBJECTS_REPORT_DIR.
#[test]
fn runtime_objects_sampling_matrix_five_fresh_processes_per_shape() {
    let Ok(dir) = std::env::var("RUNTIME_OBJECTS_REPORT_DIR") else {
        return;
    };
    let exe = std::env::current_exe().unwrap();
    let mut shapes = Vec::new();
    for shape in [
        "objects2d-regular",
        "objects2d-wide120",
        "objects3d-regular",
        "objects3d-dense64",
    ] {
        let mut samples = Vec::new();
        for run in 0..5 {
            let out = std::env::temp_dir().join(format!(
                "runtime-objects-sampler-{}-{shape}-{run}.json",
                std::process::id()
            ));
            let status = std::process::Command::new(&exe)
                .args(["--test-threads=1", "runtime_objects_sampler_child"])
                .env("RUNTIME_OBJECTS_SAMPLER", shape)
                .env("RUNTIME_OBJECTS_SAMPLER_REPORT", &out)
                .status()
                .expect("spawn sampler child");
            assert!(status.success(), "sampler child {shape}/{run} failed");
            samples.push(
                serde_json::from_slice::<serde_json::Value>(&std::fs::read(&out).unwrap()).unwrap(),
            );
            let _ = std::fs::remove_file(&out);
        }
        let mut p50s: Vec<u64> = samples
            .iter()
            .map(|s| s["tick_ns_p50"].as_u64().unwrap())
            .collect();
        let mut p95s: Vec<u64> = samples
            .iter()
            .map(|s| s["tick_ns_p95"].as_u64().unwrap())
            .collect();
        p50s.sort_unstable();
        p95s.sort_unstable();
        shapes.push(serde_json::json!({
            "shape": shape,
            "processes": 5,
            "tick_ns_p50_median": p50s[2],
            "tick_ns_p95_median": p95s[2],
            "all_under_limit": p95s.iter().all(|v| *v < 250_000_000u64),
            "samples": samples,
        }));
    }
    let report = serde_json::json!({
        "schema": "autographics.runtime-objects-sampling/1",
        "completed": true,
        "shapes": shapes,
    });
    std::fs::create_dir_all(&dir).ok();
    std::fs::write(
        std::path::Path::new(&dir).join("runtime-objects-sampling.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .expect("write sampling report");
    println!("sampling report written to {dir}");
}
