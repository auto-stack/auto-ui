//! PLAN-058 T-05 (AV phase, GPU continuous frames): the real `on Life.Tick`
//! chain drives 36 ticks in ONE AV host, and ONE renderer host presents the
//! consecutive frames 31..=36 — after each AV tick the row position is applied
//! to the render world through the presentation data channel
//! (`Play::call_script("ag_biz_apply")` → `ag_set_local`, which only writes
//! the given numbers; the render path re-runs no business), the play steps
//! exactly one frame and is captured. Every frame's sprite colour-mask
//! centroid must match the camera projection of that AV row, and every
//! consecutive centroid shift must match the AV row delta (6.33 px per full
//! step, 3.17 px into the wall contact, 0.00 px stopped) — the continuous
//! same-host upgrade of the T-04 three-snapshot pre-proof.
use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::Value;
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use autographics_auto_driver::{
    Scalar,
    authoring::{
        assets::Registry,
        document::Layer,
        gfx_host_arms::{HostRow, NeutralGfxHost},
        host::Host,
        play::{Action, Frame, State},
        storage::Storage,
    },
};
use glam::Vec2;
use png::{BitDepth, ColorType, Decoder};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, fs, io::Cursor, path::PathBuf, rc::Rc, time::Instant};

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");
const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 8.0;
const PLAYER_ORDINAL: usize = 1;
const PLAYER_HALF: [f32; 2] = [0.4, 0.5];
const WALL_FACE: f32 = 9.0 - PLAYER_HALF[0]; // 8.6
/// The presented window: four full steps into the wall, the contact step, one
/// stopped step (1-based AV tick numbers).
const FIRST_FRAME_TICK: i32 = 31;
const LAST_FRAME_TICK: i32 = 36;
/// The render world: binds the parked spectator as the fixed camera target and
/// exposes the presentation data channel — `ag_biz_apply` writes the given
/// world coordinates into the player object's local transform (ordered row 2)
/// and nothing else. No queries, no motion service, no business.
const RENDER_WORLD: &str = "var ok=0.0\nfn app_start() {\n ok=ok+ag_char_bind(1)*1.0\n ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)\n}\nfn app_update(dt float) {\n}\nfn ag_biz_apply(x float, y float) int {\n ag_set_local(2, 0, x)\n ag_set_local(2, 1, y)\n return 0\n}\n";

const ATLAS_PNG: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/sprites2d/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png"
);
const ATLAS_SHA: &str = "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1";

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
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
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the T-05 source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

fn tick_def(compiled: &ace_core::source::age_package::AgePackageCompilation) -> String {
    for name in [
        "__age_state_n1_0_last_node",
        "__age_state_1_last_node",
        "__age_state_0_last_node",
    ] {
        if let Some(symbol) = compiled.symbol_of(name) {
            return format!("d_{symbol}");
        }
    }
    panic!("no Player last_node state symbol")
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
    assert_eq!(compiled.description.components.len(), 2, "Target + Player");
    let player = compiled
        .description
        .components
        .iter()
        .find(|c| c.name == "Player")
        .expect("Player component");
    let scene = &player.scene;
    assert_eq!(scene.len(), 6, "six scene rows in the fixture");
    scene
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

fn spawn_session(
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

/// Independent sweep oracle (Minkowski shrink by the player half, render-only
/// rows and the mover skipped, nearest t, ties by lower ordinal).
fn oracle_sweep(px: f32, py: f32, dx: f32, dy: f32) -> (bool, i32, f32, bool) {
    let boxes = [
        (0usize, [2.0f32, -4.0], [14.0, 4.0]),
        (2, [9.0, -3.0], [11.0, 3.0]),
        (3, [3.0, -3.5], [5.0, -1.5]),
        (4, [3.5, 1.5], [4.5, 2.5]),
        (5, [3.5, 2.75], [4.5, 3.75]),
    ];
    let mut target = -1i32;
    let mut fraction = 2.0f32;
    for (ordinal, lo, hi) in boxes {
        if ordinal == PLAYER_ORDINAL || ordinal == 0 {
            continue;
        }
        let l = lo[0] - PLAYER_HALF[0];
        let h = hi[0] + PLAYER_HALF[0];
        let yl = lo[1] - PLAYER_HALF[1];
        let yh = hi[1] + PLAYER_HALF[1];
        if py < yl || py > yh {
            continue;
        }
        let (mut enter, mut leave) = (0.0f32, 1.0f32);
        if dx != 0.0 {
            let near = (l - px) / dx;
            let far = (h - px) / dx;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if px < l || px > h {
            continue;
        }
        if dy != 0.0 {
            let near = (yl - py) / dy;
            let far = (yh - py) / dy;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if py < yl || py > yh {
            continue;
        }
        if enter >= fraction {
            continue;
        }
        target = ordinal as i32;
        fraction = enter;
    }
    if target < 0 {
        return (false, -1, 1.0, false);
    }
    let (_, lo, hi) = boxes
        .iter()
        .find(|(o, _, _)| *o as i32 == target)
        .expect("target");
    let l = lo[0] - PLAYER_HALF[0];
    let h = hi[0] + PLAYER_HALF[0];
    let yl = lo[1] - PLAYER_HALF[1];
    let yh = hi[1] + PLAYER_HALF[1];
    let overlap = px >= l && px <= h && py >= yl && py <= yh;
    (true, target, fraction, overlap)
}

// ---------------------------------------------------------------- renderer --

fn artifacts() -> PathBuf {
    let path = PathBuf::from(
        std::env::var("PLAN058_ARTIFACTS")
            .unwrap_or_else(|_| "D:/autostack/plan-artifacts/plan-058/auto-ui".into()),
    );
    fs::create_dir_all(&path).expect("artifacts dir");
    path
}

fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn gpu_scene(player_x: f32) -> String {
    let world_sha = sha_hex(RENDER_WORLD.as_bytes());
    let objects = vec![
        ("spectator", 1u64, [-100.0f32, -100.0], [0.8f32, 1.0], false, false),
        ("ground", 2, [8.0, 0.0], [12.0, 8.0], true, false),
        ("player", 3, [player_x, 0.0], [0.8, 1.0], true, true),
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
    serde_json::json!({
        "behavior": {
            "api": "autographics.sprites2d/1",
            "source": {"format": "auto-source", "sha256": world_sha, "byte_length": RENDER_WORLD.len()},
            "actors": {"two_d": 1, "three_d": null}
        },
        "schema": "autographics.scene/6",
        "document_id": "plan058-t05-gpu-continuous",
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
    .to_string()
}

fn sprite_rgba(name: &str) -> [f64; 4] {
    match name {
        "ground" => [0.18, 0.45, 0.22, 1.0],
        "player" => [0.9, 0.3, 0.2, 1.0],
        "wall_a" => [0.55, 0.55, 0.6, 1.0],
        "wall_b" => [0.35, 0.35, 0.4, 1.0],
        "target_a" => [1.0, 0.55, 0.15, 1.0],
        "target_b" => [0.55, 0.15, 0.8, 1.0],
        _ => [0.5, 0.5, 0.55, 1.0],
    }
}

fn stage_gpu(player_x: f32) -> PathBuf {
    let scene = gpu_scene(player_x);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t05-gpu")
        .join(format!("{}-continuous", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("scripts")).expect("scripts");
    fs::create_dir_all(dir.join("assets")).expect("assets");
    fs::write(dir.join("scene.agscene"), &scene).expect("scene");
    fs::write(
        dir.join("scripts").join(format!("{}.at", sha_hex(RENDER_WORLD.as_bytes()))),
        RENDER_WORLD,
    )
    .expect("world");
    fs::write(dir.join("assets").join(format!("{ATLAS_SHA}.png")), ATLAS_PNG).expect("asset");
    dir
}

fn pump(host: &mut Host) {
    let deadline = Instant::now() + std::time::Duration::from_secs(31);
    while host.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < deadline, "host settle");
        host.frame(None).expect("frame");
    }
}

fn gpu_host(player_x: f32) -> Host {
    let dir = stage_gpu(player_x);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("load: {e:?}"));
    let mut host = Host::with_project(doc, registry, code).unwrap_or_else(|e| panic!("{e}"));
    host.start_selected_play()
        .unwrap_or_else(|e| panic!("start: {e:?}"));
    host.pause_play("");
    pump(&mut host);
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "start fault: {err}");
    }
    host
}

fn step_host(host: &mut Host) {
    {
        let play = host.play.as_mut().expect("run");
        play.resume().expect("resume");
        play.dispatch(Action {
            space: 0,
            x: 0,
            y: 0,
            toggle: false,
        })
        .expect("dispatch");
    }
    host.pause_play("");
    pump(host);
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "tick fault: {err}");
    }
    assert_eq!(
        host.play.as_ref().map(|p| p.state),
        Some(State::Paused),
        "tick settled"
    );
}

fn frame_of(host: &Host) -> &Frame {
    host.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .expect("frame")
}

fn screen_of(host: &Host, world: Vec2) -> Vec2 {
    let f = frame_of(host);
    let cam = f.camera.as_ref().expect("camera");
    let target = f.nodes.get(&cam.target_id).expect("camera target");
    let centre = cam
        .follow_center_2d(Vec2::new(target.translation[0], target.translation[1]))
        .expect("centre");
    cam.screen_2d(world, centre).expect("projection")
}

fn px_per_world(host: &Host) -> Vec2 {
    frame_of(host)
        .camera
        .as_ref()
        .expect("camera")
        .letterbox_2d()
        .expect("letterbox")
        .0
}

struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode_png(bytes: &[u8]) -> Image {
    let decoder = Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("png info");
    let mut buffer = vec![0; reader.output_buffer_size().expect("png size")];
    let info = reader.next_frame(&mut buffer).expect("png frame");
    assert_eq!(info.color_type, ColorType::Rgba);
    assert_eq!(info.bit_depth, BitDepth::Eight);
    buffer.truncate(info.buffer_size());
    Image {
        width: info.width,
        height: info.height,
        rgba: buffer,
    }
}

fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * image.width + x) * 4) as usize;
    image.rgba[at..at + 4].try_into().unwrap()
}

fn srgb_encode(v: f64) -> u8 {
    let c = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (c.mul_add(255.0, 0.5).floor() as i32).clamp(0, 255) as u8
}

fn colour_class(rgba: [f64; 4]) -> [u8; 3] {
    [
        srgb_encode(rgba[0]),
        srgb_encode(rgba[1]),
        srgb_encode(rgba[2]),
    ]
}

/// The four opaque frame-0 colours and their alpha-255 texel centroid.
fn atlas_mask() -> (Vec<[u8; 3]>, f32, f32, usize) {
    let image = decode_png(ATLAS_PNG);
    let mut colours: Vec<[u8; 3]> = Vec::new();
    let mut sum = (0.0f64, 0.0f64);
    let mut count = 0usize;
    for y in 0..32u32 {
        for x in 0..32u32 {
            let p = pixel(&image, x, y);
            if p[3] != 255 {
                continue;
            }
            let c = [p[0], p[1], p[2]];
            if !colours.contains(&c) {
                colours.push(c);
            }
            sum.0 += f64::from(x) + 0.5;
            sum.1 += f64::from(y) + 0.5;
            count += 1;
        }
    }
    assert_eq!(colours.len(), 4, "four opaque atlas colours");
    (
        colours,
        (sum.0 / count as f64) as f32,
        (sum.1 / count as f64) as f32,
        count,
    )
}

fn mask_matches(p: [u8; 4], colours: &[[u8; 3]]) -> bool {
    colours.iter().any(|c| {
        (i32::from(p[0]) - i32::from(c[0])).abs() <= 4
            && (i32::from(p[1]) - i32::from(c[1])).abs() <= 4
            && (i32::from(p[2]) - i32::from(c[2])).abs() <= 4
    })
}

fn sprite_centroid(image: &Image, colours: &[[u8; 3]]) -> (f64, f64, usize) {
    let mut sum = (0.0f64, 0.0f64);
    let mut count = 0usize;
    for y in 0..image.height {
        for x in 0..image.width {
            if mask_matches(pixel(image, x, y), colours) {
                sum.0 += f64::from(x);
                sum.1 += f64::from(y);
                count += 1;
            }
        }
    }
    assert!(count > 0, "sprite mask present");
    (sum.0 / count as f64, sum.1 / count as f64, count)
}

fn class_centroid(image: &Image, wanted: [u8; 3]) -> Option<(f64, f64, usize)> {
    let mut sum = (0.0f64, 0.0f64);
    let mut count = 0usize;
    for y in 0..image.height {
        for x in 0..image.width {
            let p = pixel(image, x, y);
            if (i32::from(p[0]) - i32::from(wanted[0])).abs() <= 6
                && (i32::from(p[1]) - i32::from(wanted[1])).abs() <= 6
                && (i32::from(p[2]) - i32::from(wanted[2])).abs() <= 6
            {
                sum.0 += f64::from(x);
                sum.1 += f64::from(y);
                count += 1;
            }
        }
    }
    (count > 0).then(|| (sum.0 / count as f64, sum.1 / count as f64, count))
}

/// The continuous same-host acceptance: 36 AV ticks in one session, frames
/// 31..=36 presented consecutively in ONE renderer host, every frame's sprite
/// centroid at that tick's camera projection, every consecutive shift matching
/// the AV row delta.
#[test]
fn t05_gpu_continuous_frames_track_the_av_rows_in_one_host() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);
    let def = tick_def(&compiled);
    let (mut session, host) = spawn_session(&compiled);
    let (colours, texel_cx, texel_cy, opaque) = atlas_mask();

    // 36 continuous AV ticks (one host): per-tick oracle + 32-credit budget.
    let mut oracle = row_world(&host, PLAYER_ORDINAL);
    let mut rows: Vec<(i32, [f32; 2])> = Vec::new();
    for tick in 1..=36 {
        host.borrow_mut().set_input(2, 1.0, 0.0, false);
        host.borrow_mut().set_dt(DT);
        host.borrow_mut().clear_tick_observations();
        session.begin_tick();
        session
            .run_def(&compiled.checked, &def)
            .unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        assert_eq!(session.credits_used(), 32, "tick {tick} budget");
        assert!(!session.tick_faulted(), "tick {tick} inside budget");
        let (px, py) = (oracle[0], oracle[1]);
        let dx = SPEED * DT;
        let (hit, _node, t, _overlap) = oracle_sweep(px, py, dx, 0.0);
        oracle = if hit { [px + dx * t, py] } else { [px + dx, py] };
        let row = row_world(&host, PLAYER_ORDINAL);
        assert!(
            (row[0] - oracle[0]).abs() <= 1e-5 && (row[1] - oracle[1]).abs() <= 1e-5,
            "tick {tick}: AV row {row:?} vs oracle {oracle:?}"
        );
        if (FIRST_FRAME_TICK..=LAST_FRAME_TICK).contains(&tick) {
            rows.push((tick, row));
        }
    }
    assert_eq!(rows.len(), 6);
    // First contact at tick 35, then the row never penetrates.
    assert!(
        (rows[4].1[0] - WALL_FACE).abs() <= 1e-5,
        "tick 35 stops at the wall face: {:?}",
        rows[4].1
    );
    assert_eq!(rows[5].1, rows[4].1, "tick 36 does not penetrate");

    // ONE renderer host presents the consecutive frames.
    let mut gpu = gpu_host(rows[0].1[0]);
    let mut evidence = Vec::new();
    let mut previous_centroid: Option<(f64, f64)> = None;
    for (frame_index, (tick, row)) in rows.iter().enumerate() {
        let t1 = Instant::now();
        // The presentation data channel: apply THIS tick's AV row to the render
        // world (no business in the render path — the script fn only writes the
        // given numbers), then present exactly one frame.
        let applied = gpu
            .play
            .as_mut()
            .expect("run")
            .call_script("ag_biz_apply", &[Scalar::Float(row[0]), Scalar::Float(row[1])])
            .expect("apply");
        assert_eq!(applied, 0, "ag_biz_apply acknowledged frame for tick {tick}");
        step_host(&mut gpu);
        // The continuous host accumulates its step counter across frames —
        // exactly one rendered step per presentation.
        assert_eq!(
            frame_of(&gpu).tick,
            frame_index as i32 + 1,
            "one rendered step after tick {tick}"
        );
        let (sha, bytes, w, h) = gpu
            .rt
            .capture(gpu.two, &format!("t05-gpu-f{tick}"))
            .expect("capture");
        let name = format!("t05-gpu-continuous-f{tick}.png");
        fs::write(artifacts().join(&name), &bytes).expect("write png");
        let image = decode_png(&bytes);
        assert_eq!((image.width, image.height), (w, h));

        let scale = px_per_world(&gpu);
        let half = Vec2::new(PLAYER_HALF[0] * scale.x, PLAYER_HALF[1] * scale.y);
        let projected = screen_of(&gpu, Vec2::new(row[0], row[1]));
        let expected = Vec2::new(
            projected.x + (texel_cx / 32.0 * 2.0 - 1.0) * half.x,
            projected.y + (texel_cy / 32.0 * 2.0 - 1.0) * half.y,
        );
        let (cx, cy, mask_px) = sprite_centroid(&image, &colours);
        let dx_px = cx - f64::from(expected.x);
        let dy_px = cy - f64::from(expected.y);
        assert!(
            dx_px.abs() <= 1.0 && dy_px.abs() <= 1.0,
            "tick {tick}: centroid ({cx:.2}, {cy:.2}) vs projection ({:.2}, {:.2})",
            expected.x,
            expected.y
        );
        let shift = previous_centroid
            .map(|(px_, py_)| ((cx - px_).powi(2) + (cy - py_).powi(2)).sqrt())
            .unwrap_or(0.0);
        if frame_index > 0 {
            let row_delta =
                (row[0] - rows[frame_index - 1].1[0]).abs() as f64 * f64::from(scale.x);
            assert!(
                (shift - row_delta).abs() <= 1.0,
                "tick {} -> {tick}: centroid shift {shift:.2}px vs AV row delta {row_delta:.2}px",
                rows[frame_index - 1].0
            );
        }
        let ground_px = class_centroid(&image, colour_class(sprite_rgba("ground")))
            .map(|(_, _, n)| n)
            .unwrap_or(0);
        assert!(
            ground_px > (800 * 380) / 2,
            "tick {tick}: ground visible pixels {ground_px}"
        );
        println!(
            "frame tick {tick}: row {:.6} centroid ({cx:.2}, {cy:.2}) dx {dx_px:+.2} dy {dy_px:+.2} shift {shift:.2}px mask {mask_px}px ground {ground_px}px step {}ms",
            row[0],
            t1.elapsed().as_millis()
        );
        previous_centroid = Some((cx, cy));
        evidence.push(serde_json::json!({
            "av_tick": tick,
            "row_world": [row[0], row[1]],
            "projection": [f64::from(projected.x), f64::from(projected.y)],
            "expected_mask_centroid": [f64::from(expected.x), f64::from(expected.y)],
            "observed_mask_centroid": [cx, cy],
            "delta_px": [dx_px, dy_px],
            "shift_from_previous_px": shift,
            "mask_pixels": mask_px,
            "ground_pixels": ground_px,
            "credits": 32,
            "frame_tick": frame_of(&gpu).tick,
            "png": name,
            "png_sha256": sha,
        }));
    }
    // Same-host discrimination at the wall: the contact step moves ~3.17 px and
    // the stopped step does not move at all.
    let (c34, _) = (previous_centroid, ());
    let _ = c34;
    let shift_contact = f64::sqrt(
        (serde_json::from_value::<f64>(evidence[4]["observed_mask_centroid"][0].clone())
            .unwrap()
            - serde_json::from_value::<f64>(evidence[3]["observed_mask_centroid"][0].clone())
                .unwrap())
        .powi(2),
    );
    assert!(
        shift_contact > 1.0,
        "the contact step is distinguishable: {shift_contact:.2}px"
    );
    let shift_stopped = (serde_json::from_value::<f64>(
        evidence[5]["observed_mask_centroid"][0].clone(),
    )
    .unwrap()
        - serde_json::from_value::<f64>(evidence[4]["observed_mask_centroid"][0].clone())
            .unwrap())
    .abs();
    assert!(
        shift_stopped <= 0.5,
        "the stopped step does not move: {shift_stopped:.2}px"
    );
    let json = serde_json::json!({
        "case": "t05_gpu_continuous_frames_track_the_av_rows_in_one_host",
        "atlas_sha256": ATLAS_SHA,
        "opaque_texels": opaque,
        "frames": evidence,
        "first_frame_tick": FIRST_FRAME_TICK,
        "last_frame_tick": LAST_FRAME_TICK,
        "wall_face": WALL_FACE,
        "dt": DT,
        "speed": SPEED,
    });
    fs::write(
        artifacts().join("t05-gpu-continuous.json"),
        serde_json::to_vec_pretty(&json).expect("json"),
    )
    .expect("write json");
    gpu.stop_play();
    println!(
        "PLAN058_TIMING test=t05_gpu_continuous_frames_track_the_av_rows_in_one_host ms={} frames=6 contact_shift={shift_contact:.2}px stopped_shift={shift_stopped:.2}px",
        t0.elapsed().as_millis()
    );
}
