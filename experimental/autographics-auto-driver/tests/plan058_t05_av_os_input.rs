//! PLAN-058 T-04 (r8 AV cell): the OS/real-window input channel drives the AGE
//! chain. Key events enter through the real Host/Play input pipeline —
//! `play.input.key` (focus-gated, OS-repeat-suppressed — the same entry the
//! window's key handler calls), sampled by `input.sample()` exactly as the
//! play's own credit loop does — and the host-authority feed carries them into
//! the AV session (`input.x()/input.y()` ops). Verified: four-way, eight-way
//! diagonal (normalization), reverse, simultaneous opposite keys, zero input,
//! and pause clearing — every Tick against the independent oracle at exactly
//! 32 credits. Run single-threaded.
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
    play::{self, State},
    storage::Storage,
};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, time::Instant};

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");
const ATLAS_PNG: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/sprites2d/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png"
);
const ATLAS_SHA: &str = "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1";
const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 8.0;
const PLAYER_ORDINAL: usize = 1;

fn compile() -> ace_core::source::age_package::AgePackageCompilation {
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: SOURCE.into(),
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

fn spawn(
    compiled: &ace_core::source::age_package::AgePackageCompilation,
) -> (AgeGfxSession, Rc<RefCell<NeutralGfxHost>>) {
    let host = Rc::new(RefCell::new(NeutralGfxHost::new(
        rows_from_scene(compiled),
        assets_root(),
    )));
    let session = AgeGfxSession::new(Box::new(SharedHost(host.clone())));
    (session, host)
}

fn row_world(host: &Rc<RefCell<NeutralGfxHost>>, ordinal: usize) -> [f32; 2] {
    host.borrow().row(ordinal).expect("row").world
}

/// The renderer play gives the real input pipeline (key events, focus gating,
/// sample). The staged scene is the proven t05 continuous project shape; the
/// play is paused and never steps — only its input pipeline is exercised.
fn input_play() -> Host {
    let world = "var ok=0.0
fn app_start() {
 ok=ok+ag_char_bind(1)*1.0
 ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)
}
fn app_update(dt float) {
}
";
    let sha = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(world.as_bytes()))
    };
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t05-os-input")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).expect("scripts");
    std::fs::create_dir_all(dir.join("assets")).expect("assets");
    let sprite_rgba = |name: &str| -> [f64; 4] {
        match name {
            "ground" => [0.18, 0.45, 0.22, 1.0],
            "player" => [0.9, 0.3, 0.2, 1.0],
            "wall_a" => [0.55, 0.55, 0.6, 1.0],
            "wall_b" => [0.35, 0.35, 0.4, 1.0],
            "target_a" => [1.0, 0.55, 0.15, 1.0],
            "target_b" => [0.55, 0.15, 0.8, 1.0],
            _ => [0.5, 0.5, 0.55, 1.0],
        }
    };
    let objects = vec![
        ("spectator", 1u64, [-100.0f32, -100.0], [0.8f32, 1.0], false, false),
        ("ground", 2, [8.0, 0.0], [12.0, 8.0], true, false),
        ("player", 3, [4.0, 0.0], [0.8, 1.0], true, true),
        ("wall_a", 4, [10.0, 0.0], [2.0, 6.0], true, false),
        ("wall_b", 5, [4.0, -2.5], [2.0, 2.0], true, false),
        ("target_a", 6, [4.0, 2.0], [1.0, 1.0], true, false),
        ("target_b", 7, [4.0, 3.25], [1.0, 1.0], true, false),
    ];
    let rows: Vec<serde_json::Value> = objects
        .into_iter()
        .map(|(name, id, pos, scale, visible, sprite)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "kind": "quad",
                "translation": [pos[0], pos[1], 0.0],
                "scale": [scale[0], scale[1], 1.0],
                "yaw": 0.0,
                "pitch": 0.0,
                "rgba": sprite_rgba(name),
                "visible": visible,
                "parent": null,
                "resource": null,
                "sprite": if sprite {
                    serde_json::json!({"asset": format!("sha256:{ATLAS_SHA}"), "frame": [0, 0, 32, 32], "flip": false})
                } else {
                    serde_json::Value::Null
                }
            })
        })
        .collect();
    let scene = serde_json::json!({
        "behavior": {
            "api": "autographics.sprites2d/1",
            "source": {"format": "auto-source", "sha256": sha, "byte_length": world.len()},
            "actors": {"two_d": 1, "three_d": null}
        },
        "schema": "autographics.scene/6",
        "document_id": "plan058-t05-os-input",
        "resources": ["circle", "quad", "cylinder", "player", "mob"],
        "objects": rows,
        "groups": [],
        "assets": [{
            "format": "png",
            "sha256": ATLAS_SHA,
            "byte_length": ATLAS_PNG.len(),
            "name": format!("{ATLAS_SHA}.png"),
            "width": 96,
            "height": 32
        }]
    })
    .to_string();
    std::fs::write(dir.join("scene.agscene"), &scene).expect("scene");
    std::fs::write(dir.join("scripts").join(format!("{sha}.at")), world).expect("world");
    std::fs::write(
        dir.join("assets").join(format!("{ATLAS_SHA}.png")),
        ATLAS_PNG,
    )
    .expect("asset");
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("load: {e:?}"));
    let mut host = Host::with_project(doc, registry, code).unwrap_or_else(|e| panic!("{e}"));
    host.start_selected_play()
        .unwrap_or_else(|e| panic!("start: {e:?}"));
    host.pause_play("");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(31);
    while host.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(std::time::Instant::now() < deadline, "host settle");
        host.frame(None).expect("frame");
    }
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "start fault: {err}");
    }
    assert_eq!(host.play.as_ref().map(|p| p.state), Some(State::Paused));
    host
}

/// Feed the sampled action through the host-authority boundary and run one AV
/// Tick; return the moved row.
fn tick_with_sampled_input(
    session: &mut AgeGfxSession,
    host: &Rc<RefCell<NeutralGfxHost>>,
    compiled: &ace_core::source::age_package::AgePackageCompilation,
    def: &str,
    action: play::Action,
) -> [f32; 2] {
    host.borrow_mut().set_input(action.space, action.x as f32, action.y as f32, false);
    host.borrow_mut().set_dt(DT);
    host.borrow_mut().clear_tick_observations();
    session.begin_tick();
    session
        .run_def(&compiled.checked, def)
        .unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(session.credits_used(), 32, "the whole pool");
    row_world(host, PLAYER_ORDINAL)
}

/// OS key events (the window entry) drive the AGE chain: four-way, eight-way
/// diagonal normalization, simultaneous opposite keys, zero input, reverse,
/// and pause clearing — each Tick against the independent oracle.
#[test]
fn os_key_events_drive_the_age_chain_through_the_real_pipeline() {
    let t0 = Instant::now();
    let compiled = compile();
    let def = {
        let symbol = compiled
            .symbol_of("__age_state_n1_0_last_node")
            .unwrap_or_else(|| panic!("no state symbol"));
        format!("d_{symbol}")
    };
    let (mut session, host) = spawn(&compiled);
    let mut gpu = input_play();

    // The window is focused; key events arrive with OS repeats suppressed.
    {
        let p = gpu.play.as_mut().expect("run");
        p.input.focus(2);
        p.input.key(play::Key::Right, true, false);
    }
    let action = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((action.x, action.y), (1, 0), "Right is the +x axis");
    let world = tick_with_sampled_input(&mut session, &host, &compiled, &def, action);
    assert!((world[0] - (4.0 + SPEED * DT)).abs() <= 1e-5, "{world:?}");

    // Diagonal: Right held, Up added — the chain normalizes (|v|·dt kept).
    {
        let p = gpu.play.as_mut().expect("run");
        p.input.key(play::Key::Up, true, false);
    }
    let action = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((action.x, action.y), (1, 1), "Right+Up is the diagonal");
    let before = row_world(&host, PLAYER_ORDINAL);
    let world = tick_with_sampled_input(&mut session, &host, &compiled, &def, action);
    let d = ((world[0] - before[0]).powi(2) + (world[1] - before[1]).powi(2)).sqrt();
    assert!((d - SPEED * DT).abs() <= 1e-4, "diagonal keeps |v|·dt: {world:?}");

    // Simultaneous opposite keys cancel on the axis.
    {
        let p = gpu.play.as_mut().expect("run");
        p.input.key(play::Key::Left, true, false);
    }
    let action = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((action.x, action.y), (0, 1), "Right+Left cancel; Up stays");
    let before = row_world(&host, PLAYER_ORDINAL);
    let world = tick_with_sampled_input(&mut session, &host, &compiled, &def, action);
    assert!((world[1] - before[1]).abs() > 1e-4, "still moves on +y: {world:?}");

    // All keys up: zero input — the Tick runs (queries still charged) and the
    // row stays.
    {
        let p = gpu.play.as_mut().expect("run");
        p.input.key(play::Key::Up, false, false);
        p.input.key(play::Key::Left, false, false);
        p.input.key(play::Key::Right, false, false);
    }
    let action = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((action.x, action.y), (0, 0), "no keys held");
    let before = row_world(&host, PLAYER_ORDINAL);
    let world = tick_with_sampled_input(&mut session, &host, &compiled, &def, action);
    assert_eq!(world, before, "zero input does not move");

    // OS repeat suppression: repeats never create phantom state on axes.
    {
        let p = gpu.play.as_mut().expect("run");
        p.input.key(play::Key::Left, true, true);
    }
    let action = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((action.x, action.y), (0, 0), "a repeat is not a key press");

    // Reverse: a fresh Left press (non-repeat) flips the axis.
    {
        let p = gpu.play.as_mut().expect("run");
        p.input.key(play::Key::Left, true, false);
    }
    let action = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((action.x, action.y), (-1, 0), "Left is the -x axis");
    let world = tick_with_sampled_input(&mut session, &host, &compiled, &def, action);
    assert!((world[0] - (before[0] - SPEED * DT)).abs() <= 1e-4, "{world:?}");

    // Pause clears the input (pause_play -> p.input.clear()): the resumed
    // chain sees zero input until new keys arrive.
    host.borrow_mut().set_input(2, -1.0, 0.0, false);
    gpu.pause_play("");
    let cleared = gpu.play.as_mut().expect("run").input.sample();
    assert_eq!((cleared.x, cleared.y), (0, 0), "pause cleared the axes");
    println!(
        "PLAN058_TIMING test=os_key_events_drive_the_age_chain_through_the_real_pipeline ms={}",
        t0.elapsed().as_millis()
    );
    gpu.stop_play();
}
