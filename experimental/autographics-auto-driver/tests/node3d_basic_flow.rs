//! auto-ui PLAN-024 node3d_basic_flow: the view-3d chain (components-3d
//! profile) executes on the pinned AG path as real project play. Three T-02
//! chains, rebuilt through the declarative author entry and consumed as
//! frozen generator products
//! (experimental/autographics-node2d-pilot/generated-components3d/*):
//! - g02 (V-02, B02-01..04): pure flat-color two-part GLB scene — run ok,
//!   red AND green part pixels > 20 on the 800-wide capture at 2px sampling
//!   (original thresholds), inspect document == run document.
//! - g03 (V-02, B03-01..06): explicit group3d hierarchy root(0.5,0,0) ->
//!   child(0.5,1,0) -> deep(0.8,0,0): deep world (1.8,1,0); v-parent moves
//!   deep to x 3.8; v-hidden strips effective visibility; base/hidden real
//!   runs both ok with different captures; Run leaves the document
//!   unchanged and Stop restores (pure-scene run path).
//! - twin (V-03/V-04): two same-def stateful Puck instances under one
//!   explicit group, each driving its OWN 3-axis position and visible
//!   through v1 ag_set_local/ag_set_visible at tick end; root receives both
//!   typed .Pinged payloads separately at Start (lo=1/hi=1); the right
//!   instance hides at its own hide_at 45 while the left persists; Stop/
//!   fresh 20 rounds rebuild state with zero GLB re-uploads after the first
//!   prepare (warm resource owner); old-Run state never leaks.
//! - v06: a single-byte damaged world.at / GLB copy is rejected at load,
//!   exact restore is accepted, and the same entry runs green again.
//! Run single-threaded:
//! cargo test --locked --offline --test node3d_basic_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        assets::Registry,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        hierarchy,
        host::Host,
        play::Action,
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const G02_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d/g02/scene.agscene");
const G02_GLB: &[u8] =
    include_bytes!("../../autographics-node2d-pilot/fixtures/node3d/g02/assets/two-parts.glb");
const G03_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d/g03/scene.agscene");
const G03_PARENT_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d/g03-parent/scene.agscene");
const G03_HIDDEN_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d/g03-hidden/scene.agscene");
const TWIN_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d/twin/scene.agscene");
const TWIN_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d/twin/world.at");

const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------- harness --

/// Materialize a product project under target and load it through the real
/// storage path (hash-verifies behavior source and every assets/<sha>.glb).
fn load_product(
    scene_text: &str,
    world_src: Option<&str>,
    glb: &[u8],
    tag: &str,
) -> (
    autographics_auto_driver::authoring::document::Document,
    Registry,
    Option<std::sync::Arc<CodePayload>>,
    PathBuf,
) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan024-components3d")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let assets_dir = dir.join("assets");
    let scripts_dir = dir.join("scripts");
    std::fs::create_dir_all(&assets_dir).expect("assets dir");
    std::fs::create_dir_all(&scripts_dir).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    let glb_sha = agp::renderer::capture::sha256_hex(glb);
    std::fs::write(assets_dir.join(format!("{glb_sha}.glb")), glb).expect("write glb");
    if let Some(world) = world_src {
        let sha = scene_text
            .split("\"sha256\": \"")
            .nth(1)
            .and_then(|r| r.split('"').next())
            .expect("scene binds a source sha")
            .to_string();
        std::fs::write(scripts_dir.join(format!("{sha}.at")), world).expect("write script");
    }
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    (doc, registry, code, dir)
}

fn start_with(scene_text: &str, world_src: Option<&str>, glb: &[u8], tag: &str) -> (Host, PathBuf) {
    let (_, _, _, dir) = load_product(scene_text, world_src, glb, tag);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("{tag} reload: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    (h, dir)
}

fn settle(h: &mut Host) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(std::time::Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

/// Advance exactly `n` world ticks through real host dispatch.
fn tick_n(h: &mut Host, n: usize) {
    for _ in 0..n {
        {
            let p = h.play.as_mut().unwrap();
            let _ = p.resume();
            if p.dispatch(Action::default()).is_err() {
                return;
            }
        }
        h.pause_play("");
        settle(h);
    }
}

fn node_world(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn row_id(doc: &autographics_auto_driver::authoring::document::Document, name: &str) -> u64 {
    doc.objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("row {name}"))
        .id
}

// ---------------------------------------------------- v1 CPU session harness --

/// CPU assertion surface: replay the frozen host config order
/// (main -> reset -> identity/visibility/bind_resource/append_field per
/// ordered node -> _ag_start) into a trusted Session built from the same
/// frozen world.at through the real admission path.
fn probe_session(scene_text: &str, world_src: &str, name: &str, run: i32) -> Session {
    probe_session_src(scene_text, world_src, name, run)
}

fn probe_session_src(scene_text: &str, world_src: &str, name: &str, run: i32) -> Session {
    let snap =
        SourceSnapshot::from_bytes_for(Path::new(name), world_src.as_bytes().to_vec(), Api::V1)
            .expect("v1 admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    // 节点序来自冻结场景（载入层真实路径）；行为源直接用受检快照——
    // 变异负控特意不匹配场景绑定哈希（生产载入路径对那种漂移的拒绝
    // 由 v06 实测覆盖）。
    let (doc, _, _, _) = {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/plan024-cpu");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("assets")).expect("cpu assets");
        let glb_sha = agp::renderer::capture::sha256_hex(G02_GLB);
        std::fs::write(dir.join("assets").join(format!("{glb_sha}.glb")), G02_GLB)
            .expect("cpu glb");
        std::fs::write(dir.join("scene.agscene"), scene_text).expect("cpu scene");
        if scene_text.contains("\"behavior\"") && !scene_text.contains("\"behavior\": null") {
            let sha = scene_text
                .split("\"sha256\": \"")
                .nth(1)
                .and_then(|r| r.split('"').next())
                .expect("cpu scene binds sha")
                .to_string();
            std::fs::create_dir_all(dir.join("scripts")).expect("cpu scripts");
            // scripts/<sha>.at 持场景绑定的冻结源；TWIN_WORLD 与绑定一致，
            // 变异源只进 VM 快照、不落该文件（保持载入层哈希合同）。
            std::fs::write(dir.join("scripts").join(format!("{sha}.at")), TWIN_WORLD)
                .expect("cpu world");
        }
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("cpu load")
    };
    let order = hierarchy::ordered(&doc).expect("ordered");
    for (i, n) in order.iter().enumerate() {
        let parent = n
            .parent
            .and_then(|p| order.iter().position(|q| q.id == p))
            .map(|v| v as i32)
            .unwrap_or(-1);
        s.call(
            "_ag_identity",
            &[
                Int(i as i32 + 1),
                Int(n.kind as i32),
                Int(n.dimension.code()),
                Int(parent),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        s.call(
            "_ag_visibility",
            &[Int(i32::from(n.visible))],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        let statics: Vec<f32> = if n.kind == 5 {
            vec![1.0, 1.0, 1.0, 0.0, 0.0, 0.9, 0.3, 0.2, 1.0]
        } else {
            let o = doc.objects.iter().find(|o| o.id == n.id).expect("object");
            o.scale
                .iter()
                .copied()
                .chain([o.yaw, o.pitch])
                .chain(o.rgba)
                .collect()
        };
        for v in n.local.iter().copied().chain(statics) {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

fn flt(s: &mut Session, name: &str) -> f32 {
    s.float(name, &[]).unwrap()
}

fn cpu_tick(s: &mut Session, next: usize) {
    s.call("_ag_tick", &[Int(next as i32), Float(DT)], ResultKind::Unit)
        .unwrap();
}

// ------------------------------------------------------------------ V-02 --

#[test]
fn v02_g02_pure_glb_scene_original_oracles() {
    // B02-01 run ok + B02-02/03 pixel classes + B02-04 document identity.
    let (mut h, dir) = start_with(G02_SCENE, None, G02_GLB, "g02");
    settle(&mut h);
    // Original oracle samples every 2px on an 800-wide capture (400 columns);
    // the native surface here is 1200x800, so step by w/400 horizontally to
    // keep the same column density and thresholds.
    let cap = h.rt.capture(h.top, "g02-grid").expect("capture");
    let (cw, ch) = ((cap.2 as usize), (cap.3 as usize));
    let step_x = (cw / 400).max(1);
    let mut points = Vec::new();
    for y in (0..ch).step_by(step_x) {
        for x in (0..cw).step_by(step_x) {
            points.push([x as f32, y as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    let (mut red, mut green) = (0usize, 0usize);
    for (_, c) in points.iter().zip(px.iter()) {
        let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
        if r > 150 && r > g + 60 && r > b + 60 {
            red += 1;
        }
        if g > 150 && g > r + 60 && g > b + 60 {
            green += 1;
        }
    }
    assert!(red > 20, "B02-02 red part pixels {red} <= 20");
    assert!(green > 20, "B02-03 green part pixels {green} <= 20");
    // B02-04: inspect reopen document == run document (same entry, twice).
    let (doc_a, _, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("inspect a");
    let (doc_b, _, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("inspect b");
    assert_eq!(doc_a, doc_b, "inspect reopen document identity");
    let run_doc = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .document
        .clone();
    assert_eq!(run_doc, doc_a, "run document == inspect document");
    h.stop_play();
    assert_eq!(h.editor.document, doc_a, "Stop restores document defaults");

    // Derived-oracle cross-check: CPU hierarchy derive agrees with the scene.
    let derived = hierarchy::derive(&doc_a).expect("derive");
    let badge_id = row_id(&doc_a, "badge");
    assert_eq!(derived[&badge_id].translation, [0.0, 0.6, 0.0]);
}

fn g03_cpu_asserts(scene: &str, tag: &str, deep_world: [f32; 3], root_visible: bool) {
    let (doc, _, _, _) = load_product(scene, None, G02_GLB, tag);
    // B03-01/02/03 on the CPU admission oracle (hierarchy::derive).
    let derived = hierarchy::derive(&doc).expect("derive");
    let deep = row_id(&doc, "deep");
    let leaf = row_id(&doc, "leaf");
    assert!(
        (derived[&deep].translation[0] - deep_world[0]).abs() < 1e-3,
        "{tag} deep world x {} != {}",
        derived[&deep].translation[0],
        deep_world[0]
    );
    assert!((derived[&deep].translation[1] - deep_world[1]).abs() < 1e-3);
    assert_eq!(
        derived[&leaf].visible, root_visible,
        "{tag} leaf visibility"
    );
    assert_eq!(
        derived[&deep].visible, root_visible,
        "{tag} ancestor visibility AND"
    );
}

#[test]
fn v02_g03_hierarchy_world_parent_and_hidden() {
    g03_cpu_asserts(G03_SCENE, "g03", [1.8, 1.0, 0.0], true);
    g03_cpu_asserts(G03_PARENT_SCENE, "g03-parent", [3.8, 1.0, 0.0], true);
    g03_cpu_asserts(G03_HIDDEN_SCENE, "g03-hidden", [1.8, 1.0, 0.0], false);

    // B03-04 base + hidden both run for real; B03-05 captures differ.
    let (mut hb, dirb) = start_with(G03_SCENE, None, G02_GLB, "g03-run");
    settle(&mut hb);
    let cap_base = hb.rt.capture(hb.top, "g03-base").expect("capture base").0;
    hb.stop_play();
    let (mut hh, _) = start_with(G03_HIDDEN_SCENE, None, G02_GLB, "g03-hidden-run");
    settle(&mut hh);
    let cap_hidden = hh
        .rt
        .capture(hh.top, "g03-hidden")
        .expect("capture hidden")
        .0;
    assert_ne!(cap_base, cap_hidden, "B03-05 hidden capture differs");
    hh.stop_play();
    let (mut hp, _) = start_with(G03_PARENT_SCENE, None, G02_GLB, "g03-parent-run");
    settle(&mut hp);
    let cap_parent = hp
        .rt
        .capture(hp.top, "g03-parent")
        .expect("capture parent")
        .0;
    assert_ne!(cap_base, cap_parent, "B03-02 moved-root capture differs");
    assert_ne!(cap_hidden, cap_parent, "hidden vs moved renders differ");
    hp.stop_play();

    // B03-06: a real run (pure-scene play) leaves the document unchanged and
    // Stop restores; the run frame carries the derived world positions.
    let (doc0, _, _, _) =
        Storage::load_project(&dirb.join("scene.agscene"), &Registry::new()).expect("reload");
    let (mut h, _) = start_with(G03_SCENE, None, G02_GLB, "g03-run2");
    tick_n(&mut h, 5);
    let run_doc = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .document
        .clone();
    assert_eq!(run_doc, doc0, "B03-06 Run does not change the document");
    let deep_id = row_id(&doc0, "deep");
    assert!(
        (node_world(&h, deep_id)[0] - 1.8).abs() < 1e-3,
        "frame world x = {}",
        node_world(&h, deep_id)[0]
    );
    assert!(
        (node_world(&h, deep_id)[1] - 1.0).abs() < 1e-3,
        "frame world y = {}",
        node_world(&h, deep_id)[1]
    );
    h.stop_play();
    assert_eq!(h.editor.document, doc0, "Stop restores document defaults");
}

// ------------------------------------------------------------------ V-03 --

#[test]
fn v03_twin_stateful_local_phase_events_visibility() {
    // CPU session: Start delivered BOTH typed payloads through the root
    // receiver; local model phases advance per instance through real ticks.
    let mut s = probe_session(TWIN_SCENE, TWIN_WORLD, "twin-world.at", 1);
    assert_eq!(int(&mut s, "get_r_lo"), 1, "left payload 0.0 -> lo");
    assert_eq!(int(&mut s, "get_r_hi"), 1, "right payload 3.0 -> hi");
    for n in 1..=30usize {
        cpu_tick(&mut s, n);
    }
    let left = flt(&mut s, "get_Main_left_phase");
    let right = flt(&mut s, "get_Main_right_phase");
    assert!((left - 0.5).abs() < 1e-3, "left phase {left}");
    assert!((right - 3.5).abs() < 1e-3, "right phase {right}");
    for n in 31..=46usize {
        cpu_tick(&mut s, n);
    }
    assert_eq!(
        int(&mut s, "get_Main_right_on"),
        0,
        "right hidden at tick 45"
    );
    assert_eq!(int(&mut s, "get_Main_left_on"), 1, "left unaffected");
    for n in 47..=76usize {
        cpu_tick(&mut s, n);
    }
    let right2 = flt(&mut s, "get_Main_right_phase");
    // Visible=false keeps identity and never stops Life.Tick: at tick 76 the
    // hidden right instance still owns phase 3.0 + 76/60 = 4.2667.
    assert!(
        (right2 - 4.2667).abs() < 1e-3,
        "right model keeps ticking {right2}"
    );
    s.close();

    // GPU: the same frozen products on the real host path — world positions
    // follow the per-instance model, and the visible flip shows in captures.
    let (mut h, dir) = start_with(TWIN_SCENE, Some(TWIN_WORLD), G02_GLB, "twin-gpu");
    settle(&mut h);
    let (doc, _, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("reload doc");
    tick_n(&mut h, 30);
    let left_row = row_id(&doc, "Main/left/puck");
    let right_row = row_id(&doc, "Main/right/puck");
    assert!(
        (node_world(&h, left_row)[0] - 0.5).abs() < 2e-3,
        "left GPU world x = {}",
        node_world(&h, left_row)[0]
    );
    assert!(
        (node_world(&h, right_row)[0] - 3.5).abs() < 2e-3,
        "right GPU world x = {}",
        node_world(&h, right_row)[0]
    );
    tick_n(&mut h, 15);
    let cap_45 = h.rt.capture(h.top, "twin-45").expect("capture 45").0;
    tick_n(&mut h, 30);
    let cap_75 = h.rt.capture(h.top, "twin-75").expect("capture 75").0;
    assert_ne!(cap_45, cap_75, "GPU differs after the right instance hides");

    // V-04 lifecycle on the same products: fresh Start replays payloads
    // exactly once; Stop restores the document; 20 stop->fresh rounds show
    // no old-Run leakage and a warm resource owner (no GLB re-upload).
    h.stop_play();
    assert_eq!(h.editor.document, doc, "stop restore");
    let (mut h2, _) = start_with(TWIN_SCENE, Some(TWIN_WORLD), G02_GLB, "twin-fresh");
    settle(&mut h2);
    let baseline_uploads = h2.resource_snapshot()["mesh_uploads"].clone();
    for round in 0..20usize {
        h2.stop_play();
        h2.start_selected_play().expect("start round");
        h2.pause_play("");
        settle(&mut h2);
        assert_eq!(
            h2.resource_snapshot()["mesh_uploads"].clone(),
            baseline_uploads,
            "round {round}: warm resource owner — no GLB re-upload"
        );
        let run_doc = h2
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .expect("frame")
            .document
            .clone();
        assert_eq!(run_doc, doc, "round {round}: document untouched");
    }
    h2.stop_play();
}

#[test]
fn v04_twin_cpu_lifecycle_counts() {
    // Life.Start fires exactly once per Run on the CPU path; a fresh Session
    // rebuilds from zero (no old-Run leakage); Resume-less Step (single
    // tick) advances exactly one tick.
    let mut s = probe_session(TWIN_SCENE, TWIN_WORLD, "twin-lc.at", 7);
    assert_eq!(int(&mut s, "get_r_lo"), 1);
    assert_eq!(int(&mut s, "get_r_hi"), 1);
    assert_eq!(int(&mut s, "get_Main_left_ticks"), 0, "Start does not tick");
    cpu_tick(&mut s, 1);
    assert_eq!(
        int(&mut s, "get_Main_left_ticks"),
        1,
        "one tick, one dispatch"
    );
    cpu_tick(&mut s, 2);
    assert_eq!(int(&mut s, "get_Main_left_ticks"), 2);
    assert_eq!(int(&mut s, "get_r_lo"), 1, "ticks never re-run Start");
    s.close();
    let mut fresh = probe_session(TWIN_SCENE, TWIN_WORLD, "twin-lc2.at", 8);
    assert_eq!(int(&mut fresh, "get_Main_left_ticks"), 0, "fresh rebuild");
    assert_eq!(int(&mut fresh, "get_r_lo"), 1, "fresh Start replay");
    fresh.close();

    // 错误绑定负控（V-03）：投递路径必须承重——把生成源的投递实参从
    // puck_p0[i]（每实例自己的初值）换成常量 9.0 后，两个实例都计进 hi
    // （lo=0/hi=2），运行本身仍绿：接收计数来自真实投递，不是常量回声。
    let mutant_world = TWIN_WORLD.replace("main_pinged(puck_p0[i])", "main_pinged(9.0)");
    assert!(mutant_world != TWIN_WORLD, "mutation anchor present");
    let mut bad = probe_session_src(TWIN_SCENE, &mutant_world, "twin-mut.at", 9);
    assert_eq!(
        int(&mut bad, "get_r_lo"),
        0,
        "mutated payload never lands in lo"
    );
    assert_eq!(
        int(&mut bad, "get_r_hi"),
        2,
        "mutated payload lands in hi twice"
    );
    bad.close();
}

// ------------------------------------------------------------------ V-06 --

#[test]
fn v06_damaged_copies_rejected_then_restore_green() {
    let (_, _, _, dir) = load_product(TWIN_SCENE, Some(TWIN_WORLD), G02_GLB, "v06-base");
    let scripts = dir.join("scripts");
    let world_name = {
        let mut it = std::fs::read_dir(&scripts).expect("scripts");
        let p = it.next().expect("one script").expect("entry").path();
        p.file_name().unwrap().to_str().unwrap().to_string()
    };
    // Damage one byte of the world copy: the load path must reject it.
    let world_path = scripts.join(&world_name);
    let mut bytes = std::fs::read(&world_path).expect("world bytes");
    let at = bytes.len() / 2;
    bytes[at] = bytes[at].wrapping_sub(1);
    std::fs::write(&world_path, &bytes).expect("damage world");
    let damaged = Storage::load_project(&dir.join("scene.agscene"), &Registry::new());
    assert!(
        damaged.is_err(),
        "damaged world.at must be rejected at load"
    );
    // Exact restore: same entry runs green again.
    std::fs::write(&world_path, TWIN_WORLD).expect("restore world");
    let (mut h, _) = start_with(TWIN_SCENE, Some(TWIN_WORLD), G02_GLB, "v06-restore");
    settle(&mut h);
    h.stop_play();

    // B02-05 analog: single corrupted GLB byte is rejected by the resource
    // admission at load; exact restore is accepted.
    let (_, _, _, gdir) = load_product(G02_SCENE, None, G02_GLB, "v06-glb");
    let glb = glb_path_of(&gdir);
    let mut gbytes = std::fs::read(&glb).expect("glb bytes");
    let mid = gbytes.len() / 2;
    gbytes[mid] = gbytes[mid].wrapping_add(1);
    std::fs::write(&glb, &gbytes).expect("damage glb");
    let rejected = Storage::load_project(&gdir.join("scene.agscene"), &Registry::new());
    assert!(rejected.is_err(), "corrupted GLB must be rejected at load");
    std::fs::write(&glb, G02_GLB).expect("restore glb");
    let (mut hg, _) = start_with(G02_SCENE, None, G02_GLB, "v06-glb-restore");
    settle(&mut hg);
    let cap = hg.rt.capture(hg.top, "g02-restore").expect("capture");
    let (cw, ch) = ((cap.2 as usize), (cap.3 as usize));
    let step_x = (cw / 400).max(1);
    let mut points = Vec::new();
    for y in (0..ch).step_by(step_x) {
        for x in (0..cw).step_by(step_x) {
            points.push([x as f32, y as f32]);
        }
    }
    let px = hg.rt.probe_pixels(hg.top, &points).expect("probe");
    let colored = px
        .iter()
        .filter(|c| {
            let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
            (r > 150 && r > g + 60 && r > b + 60) || (g > 150 && g > r + 60 && g > b + 60)
        })
        .count();
    assert!(colored > 10, "restored g02 renders both parts ({colored})");
    hg.stop_play();
}

fn glb_path_of(dir: &Path) -> PathBuf {
    let assets = dir.join("assets");
    let mut it = std::fs::read_dir(assets).expect("assets listing");
    let p = it.next().expect("one glb").expect("entry").path();
    assert_eq!(p.extension().and_then(|e| e.to_str()), Some("glb"));
    p
}

// ------------------------------------------------------------------ V-04/05 --

#[test]
fn v05_driver_envelopes_rows512_and_step_warm() {
    use autographics_auto_driver::authoring::{
        assets,
        document::{Dimension, Document, Group, Kind, Object},
    };

    // 129 行（组+对象）在载入层具名拒绝（128 包络的 driver 侧重复防线）。
    {
        let mut d = Document::empty("rows129".into());
        for i in 0..129u64 {
            let mut o = Object::new(i + 1, Kind::Cylinder);
            o.translation = [(i as f32) * 0.1, 0.0, 0.0];
            d.objects.push(o);
        }
        let host = Host::with_project(d, Registry::new(), None);
        assert!(host.is_err(), "129 rows must be rejected at the host entry");
    }

    // 512 潜在 mesh 实例上限：原实现未被 PLAN-024 触碰（host/viewport.rs
    // prepare_frame + host/play_adapter.rs 均为交付代码），本计划无多图元
    // 新资产（原 plain-GLB 域最大 2 primitives/件），128 节点包络下文档级
    // potential 上界 = 128×2 = 256 < 512，故该上限沿原证据复用（观察：
    // plain 链上 128 包络先于 512 上限生效；tests/asset_feasibility 的
    // 单元级检查目标在 master 上因 PLAN-013 hud_overlay 字段早已失编译，
    // 属先在状态，非本期破坏）。

    // V-04 Pause/Step：暂停下单次 Step 恰推进一个中性 tick；渲染帧不推进。
    let (mut h2, _) = start_with(TWIN_SCENE, Some(TWIN_WORLD), G02_GLB, "twin-step");
    settle(&mut h2);
    let left_row = {
        let f = h2.play.as_ref().unwrap().last.as_ref().expect("frame");
        f.document
            .objects
            .iter()
            .find(|o| o.name == "Main/left/puck")
            .expect("left row")
            .id
    };
    tick_n(&mut h2, 10);
    let x10 = node_world(&h2, left_row)[0];
    {
        let p = h2.play.as_mut().unwrap();
        p.single_step().expect("single step");
    }
    h2.pause_play("");
    settle(&mut h2);
    let x11 = node_world(&h2, left_row)[0];
    assert!(
        (x11 - x10 - DT).abs() < 2e-3,
        "single step advances exactly one tick: {x10} -> {x11}"
    );
    // render-only 帧（暂停下 frame）不推进状态。
    h2.frame(None).expect("render frame");
    let x11b = node_world(&h2, left_row)[0];
    assert_eq!(x11, x11b, "render-only frame advances nothing");
    h2.stop_play();

    // V-05 tick 记账观察：受信 CPU 会话 120 tick 墙钟（独立口径报告，
    // 不冒充宿主 8ms 帧硬保证）。
    let mut s = probe_session(TWIN_SCENE, TWIN_WORLD, "twin-budget.at", 11);
    let t0 = std::time::Instant::now();
    let mut worst = std::time::Duration::ZERO;
    for n in 1..=120usize {
        let a = std::time::Instant::now();
        cpu_tick(&mut s, n);
        worst = worst.max(a.elapsed());
    }
    let avg = t0.elapsed() / 120;
    println!(
        "plan024 twin CPU whole-tick wall clock: avg {:?} / max {worst:?} over 120 ticks (observation, separate from the host 8ms frame cap)",
        avg
    );
    assert!(
        avg < std::time::Duration::from_millis(8),
        "CPU whole-tick avg {:?} exceeds the 8ms frame cap reference",
        avg
    );
    s.close();
}
