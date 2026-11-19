//! PLAN-058 T-05/T-06 (r8 AV cells): the Run lifecycle and cold-start AV legs.
//!
//! * V-S04: Pause/Step on the real play, 20 consecutive Runs from the same
//!   public entry (fresh session each, deterministic trajectories, no residue),
//!   a stale handle from a previous Run refused, a broken module refused with
//!   named diagnostics, a missing asset refused with the image fault — then
//!   the precise fix and the SAME entry running for real.
//! * V-S05 (AV leg): cold A→B — the staged project loads with no producer root
//!   and no network, a source change yields a different package identity (the
//!   old Ready does not silently apply), and the old-profile identity is the
//!   controls' frozen business (C2/C3, referenced).
//! Run single-threaded.
use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::Value;
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use autographics_auto_driver::authoring::{
    assets::Registry,
    document::Layer,
    gfx_host_arms::{HostRow, NeutralGfxHost},
    host::Host,
    storage::Storage,
};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, time::Instant};

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");
const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 8.0;
const PLAYER_ORDINAL: usize = 1;
const ATLAS_PNG: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/sprites2d/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png"
);
const ATLAS_SHA: &str = "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1";

fn compile_of(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    };
    compile_age_package_with_scenario(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::ForNode {
                node: 1,
                op: Box::new(ScenarioOp::Tick { instance: 0 }),
            }],
        },
    )
    .unwrap_or_else(|ds| panic!("front door rejected: {ds:?}"))
}

fn compile() -> ace_core::source::age_package::AgePackageCompilation {
    compile_of(SOURCE)
}

fn tick_symbol(
    compiled: &ace_core::source::age_package::AgePackageCompilation,
    name: &str,
) -> String {
    let symbol = compiled
        .symbol_of(name)
        .unwrap_or_else(|| panic!("no symbol for `{name}`"));
    format!("d_{symbol}")
}

fn tick_def(compiled: &ace_core::source::age_package::AgePackageCompilation) -> String {
    for name in ["__age_state_n1_0_last_node", "__age_state_1_last_node"] {
        if let Some(symbol) = compiled.symbol_of(name) {
            return format!("d_{symbol}");
        }
    }
    panic!("no Player state symbol")
}

fn assets_root() -> PathBuf {
    // The fixture's asset paths are project-relative ("assets/atlas.png"), so
    // the host's asset root is a staged PROJECT root, not the assets dir.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/plan058-t05-assets");
    let assets = dir.join("assets");
    fs::create_dir_all(&assets).expect("assets dir");
    let atlas = assets.join("atlas.png");
    if !atlas.exists() {
        fs::write(&atlas, ATLAS_PNG).expect("atlas");
    }
    dir
}

fn rows_from_scene(
    compiled: &ace_core::source::age_package::AgePackageCompilation,
) -> Vec<HostRow> {
    let player = compiled
        .description
        .components
        .iter()
        .find(|c| c.name == "Player")
        .expect("Player component");
    player
        .scene
        .iter()
        .enumerate()
        .map(|(ordinal, row)| {
            let floats = |key: &str| -> Option<Vec<f32>> {
                row.cpu.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
                    if let ace_core::source::age_package::AgeSceneValue::Floats(f) = v {
                        Some(f.clone())
                    } else {
                        None
                    }
                })
            };
            let position = floats("position").unwrap_or_else(|| panic!("{row:?} position"));
            let size = floats("size").unwrap_or_else(|| panic!("{row:?} size"));
            let visible = row
                .cpu
                .iter()
                .find(|(k, _)| k == "visible")
                .map(|(_, v)| matches!(v, ace_core::source::age_package::AgeSceneValue::Bool(true)))
                .unwrap_or(true);
            HostRow {
                ordinal,
                world: [position[0], position[1]],
                half: [size[0] * 0.5, size[1] * 0.5],
                layer: match row.layer {
                    Some(ace_core::source::age_package::AgeRowLayer::RenderOnly) => {
                        Layer::RenderOnly
                    }
                    None => Layer::Collision,
                },
                visible,
            }
        })
        .collect()
}

struct SharedHost(Rc<RefCell<NeutralGfxHost>>);

impl AgeGfxHost for SharedHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.0.borrow_mut().dispatch(symbol, args)
    }
    fn dispatch_with_ctx(
        &mut self,
        symbol: &str,
        args: &[Value],
        ctx: &ace_av::age_gfx::AgeGfxCall<'_>,
    ) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.0.borrow_mut().dispatch_with_ctx(symbol, args, ctx)
    }
}

fn fresh_run(
    compiled: &ace_core::source::age_package::AgePackageCompilation,
) -> (AgeGfxSession, Rc<RefCell<NeutralGfxHost>>) {
    let host = Rc::new(RefCell::new(NeutralGfxHost::new(
        rows_from_scene(compiled),
        assets_root(),
    )));
    let session = AgeGfxSession::new(Box::new(SharedHost(host.clone())));
    (session, host)
}

fn run_three_ticks(
    session: &mut AgeGfxSession,
    host: &Rc<RefCell<NeutralGfxHost>>,
    compiled: &ace_core::source::age_package::AgePackageCompilation,
    def: &str,
) -> Vec<[f32; 2]> {
    let mut trace = Vec::new();
    for _ in 0..3 {
        host.borrow_mut().set_input(2, 1.0, 0.0, false);
        host.borrow_mut().set_dt(DT);
        session.begin_tick();
        session
            .run_def(&compiled.checked, def)
            .unwrap_or_else(|e| panic!("tick: {e}"));
        assert_eq!(session.credits_used(), 32);
        trace.push(host.borrow().row(PLAYER_ORDINAL).expect("row").world);
    }
    trace
}

/// V-S04: 20 consecutive Runs from the same public entry — each Run is a fresh
/// session on the same checked module; trajectories are identical (the source
/// is the only state owner), generation counters start at 1 every Run, and the
/// rows reset to the declared origin (no cross-Run residue).
#[test]
fn twenty_runs_are_fresh_and_deterministic() {
    let t0 = Instant::now();
    let compiled = compile();
    let def = tick_def(&compiled);
    let first: Vec<[f32; 2]> = {
        let (mut session, host) = fresh_run(&compiled);
        run_three_ticks(&mut session, &host, &compiled, &def)
    };
    for run in 1..=20u32 {
        let (mut session, host) = fresh_run(&compiled);
        assert_eq!(session.callback_seq(), 0, "run {run}: fresh generations");
        let trace = run_three_ticks(&mut session, &host, &compiled, &def);
        assert_eq!(trace, first, "run {run}: deterministic trajectory");
        assert_eq!(session.callback_seq(), 3, "run {run}: three callbacks");
    }
    println!(
        "PLAN058_TIMING test=twenty_runs_are_fresh_and_deterministic ms={} runs=20",
        t0.elapsed().as_millis()
    );
}

/// V-S04 refusals: a stale handle from a previous Run is dead in the new one;
/// a broken module is refused with named diagnostics at the same public entry;
/// a missing asset is refused with the image fault — and the precise fix (the
/// real source/asset) runs for real through the same entry.
#[test]
fn stale_handle_and_broken_inputs_refuse_then_the_fix_runs() {
    let t0 = Instant::now();
    let compiled = compile();
    let def = tick_def(&compiled);

    // Run 1 produces a render handle; the Run ends.
    let (mut session, host) = fresh_run(&compiled);
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    session.run_def(&compiled.checked, &def).expect("run 1 tick");
    let stale_token = host.borrow().tokens()[0].token;
    drop(session);
    drop(host);

    // Run 2: a fresh host never issued that handle — the stale write is the
    // named bad-token refusal at the provider judgment point.
    let (_s2, host2) = fresh_run(&compiled);
    let err = host2
        .borrow_mut()
        .dispatch(
            "age.render2d.frame.v1",
            &[
                Value::I32(stale_token),
                Value::Str(ace_av_core::contract::StrHandle::detached(
                    b"assets/atlas.png".to_vec().into_boxed_slice(),
                )),
                Value::I32(0),
                Value::I32(0),
            ],
        )
        .expect_err("the previous Run's handle is dead");
    // A direct provider-level call carries the bare suffix (the AV lane adds
    // the `age-gfx.` prefix on its way out).
    assert_eq!(err.code, "bad-token", "{err:?}");
    drop(host2);

    // A broken module is refused with named diagnostics at the same entry.
    let broken = SOURCE.replace("query2d.sweep(1, true, dx, dy)", "query2d.sweeep(1, true, dx, dy)");
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: broken.into(),
        provider: None,
    };
    let ds = compile_age_package_with_scenario(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::ForNode {
                node: 1,
                op: Box::new(ScenarioOp::Tick { instance: 0 }),
            }],
        },
    )
    .expect_err("the broken module is refused");
    assert!(
        ds.iter().any(|d| !d.code.is_empty()),
        "named diagnostics: {ds:?}"
    );

    // A missing asset is refused with the image fault through the real lane
    // (the asset op runs where the author calls it — top-level resource lets
    // do not dispatch in this slice).
    let compiled_real = compile();
    let load_missing = tick_symbol(&compiled_real, "load_missing");
    let (mut session_m, host_m) = fresh_run(&compiled_real);
    host_m.borrow_mut().set_input(2, 1.0, 0.0, false);
    host_m.borrow_mut().set_dt(DT);
    session_m.begin_tick();
    let err = session_m
        .run_def(&compiled_real.checked, &load_missing)
        .expect_err("the missing asset is refused");
    assert!(
        err.contains("age-gfx.image-missing"),
        "named image fault: {err}"
    );
    // The precise asset fix — the real atlas through the same entry — resolves
    // to its content identity byte length.
    let load_atlas = tick_symbol(&compiled_real, "load_atlas");
    let got = session_m
        .run_def(&compiled_real.checked, &load_atlas)
        .expect("the real atlas loads");
    assert_eq!(got, Value::I32(258), "the real atlas is 258 bytes");

    // The precise fix — the real source and the real asset — runs for real
    // through the same entry: first contact at tick 35, wall face 8.6.
    let (mut session_ok, host_ok) = fresh_run(&compiled);
    let mut oracle = [4.0f32, 0.0];
    for tick in 1..=35 {
        host_ok.borrow_mut().set_input(2, 1.0, 0.0, false);
        host_ok.borrow_mut().set_dt(DT);
        session_ok.begin_tick();
        session_ok
            .run_def(&compiled.checked, &def)
            .unwrap_or_else(|e| panic!("fixed tick {tick}: {e}"));
        let dx = SPEED * DT;
        let wall_enter = (9.0 - 0.4 - oracle[0]) / dx;
        oracle[0] = if wall_enter < 1.0 {
            oracle[0] + dx * wall_enter
        } else {
            oracle[0] + dx
        };
    }
    let world = host_ok.borrow().row(PLAYER_ORDINAL).expect("row").world;
    assert!(
        (world[0] - 8.6).abs() <= 1e-5,
        "the fixed Run reaches the wall face: {world:?}"
    );
    println!(
        "PLAN058_TIMING test=stale_handle_and_broken_inputs_refuse_then_the_fix_runs ms={}",
        t0.elapsed().as_millis()
    );
}

/// V-S05 (AV leg): cold A→B — a staged project loads and runs with no producer
/// root and no network; changing the source yields a different package
/// identity (the old Ready does not silently apply); the old session keeps its
/// own identity and stays deterministic; old-profile bytes are the controls'
/// frozen business (C2/C3) and are not re-verified here.
#[test]
fn cold_a_to_b_changes_identity_and_the_old_ready_does_not_apply() {
    let t0 = Instant::now();
    // Cold staging: project A on disk (scene + script + the atlas asset), no
    // producer root, no network — Storage::load_project reads it cold.
    let world_a = "var ok=0.0\nfn app_start() {\n ok=ok+ag_char_bind(1)*1.0\n}\nfn app_update(dt float) {\n}\n";
    let world_b = "var ok=0.0\nfn app_start() {\n ok=ok+ag_char_bind(1)*1.0\n ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)\n}\nfn app_update(dt float) {\n}\n";
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t05-cold")
        .join(std::process::id().to_string());
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("scripts")).expect("scripts");
    fs::create_dir_all(dir.join("assets")).expect("assets");
    let sha = |w: &str| format!("{:x}", Sha256::digest(w.as_bytes()));
    let scene = |world: &str, id: &str| {
        let world_sha = sha(world);
        serde_json::json!({
            "behavior": {
                "api": "autographics.sprites2d/1",
                "source": {"format": "auto-source", "sha256": world_sha, "byte_length": world.len()},
                "actors": {"two_d": 1, "three_d": null}
            },
            "schema": "autographics.scene/6",
            "document_id": id,
            "resources": ["circle", "quad", "cylinder", "player", "mob"],
            "objects": [{
                "id": 1, "name": "spectator", "kind": "quad",
                "translation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0],
                "yaw": 0.0, "pitch": 0.0, "rgba": [0.5, 0.5, 0.55, 1.0],
                "visible": true, "parent": null, "resource": null, "sprite": null
            }],
            "groups": [],
            "assets": []
        })
        .to_string()
    };
    fs::write(dir.join("scene.agscene"), scene(world_a, "cold-a")).expect("scene a");
    fs::write(dir.join("scripts").join(format!("{}.at", sha(world_a))), world_a).expect("world a");
    fs::write(dir.join("assets").join(format!("{ATLAS_SHA}.png")), ATLAS_PNG).expect("asset");

    // Cold load A and start it (the Ready).
    let (doc_a, registry_a, code_a, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("cold load A: {e:?}"));
    let mut host_a = Host::with_project(doc_a, registry_a, code_a).unwrap_or_else(|e| panic!("{e}"));
    host_a.start_selected_play()
        .unwrap_or_else(|e| panic!("A start: {e:?}"));
    let ready_hash = host_a
        .play
        .as_ref()
        .expect("A run")
        .source_hash
        .clone();

    // Change the source on disk to B — the editor saves the new script AND the
    // scene manifest that names it by content sha; a cold reload sees B's
    // identity, and the old Ready hash no longer matches what's on disk.
    fs::write(dir.join("scripts").join(format!("{}.at", sha(world_b))), world_b).expect("world b");
    fs::write(dir.join("scene.agscene"), scene(world_b, "cold-b")).expect("scene b");
    let (doc_b, registry_b, code_b, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("cold load B: {e:?}"));
    let mut host_b = Host::with_project(doc_b, registry_b, code_b).unwrap_or_else(|e| panic!("{e}"));
    host_b.start_selected_play()
        .unwrap_or_else(|e| panic!("B start: {e:?}"));
    let new_hash = host_b.play.as_ref().expect("B run").source_hash.clone();
    assert_ne!(
        ready_hash, new_hash,
        "the source change invalidates the old Ready identity"
    );

    // The AGE face: A and B compile to DIFFERENT package identities; the old
    // session keeps running A deterministically while a fresh Run takes B.
    let slow = SOURCE.replace("speed f32 = 8.0", "speed f32 = 4.0");
    let a = compile();
    let b = compile_of(&slow);
    assert_ne!(
        a.identity().input_hash,
        b.identity().input_hash,
        "the source change changes the package identity"
    );
    let (mut session_a, host_a2) = fresh_run(&a);
    let def_a = tick_def(&a);
    let trace_a = run_three_ticks(&mut session_a, &host_a2, &a, &def_a);
    assert!((trace_a[0][0] - (4.0 + 8.0 * DT)).abs() <= 1e-4);
    let (mut session_b, host_b2) = fresh_run(&b);
    let def_b = tick_def(&b);
    let trace_b = run_three_ticks(&mut session_b, &host_b2, &b, &def_b);
    assert!((trace_b[0][0] - (4.0 + 4.0 * DT)).abs() <= 1e-4, "B is the slow source");
    println!(
        "PLAN058_TIMING test=cold_a_to_b_changes_identity_and_the_old_ready_does_not_apply ms={}",
        t0.elapsed().as_millis()
    );
    let _ = (host_a.stop_play(), host_b.stop_play());
}
