//! auto-ui PLAN-022 node_sprites_flow: the sprites-2d chain executes on the
//! pinned AG path as real project play. The consumed scene.agscene/world.at/
//! assets are frozen generator products
//! (experimental/autographics-node2d-pilot/generated-sprites2d/*). Every
//! oracle name below is the ORIGINAL G09 acceptance name, preserved verbatim
//! (ag-examples checks/g09/gallery_runtime.rs @ 9d7ce602); expectations are
//! computed from the fixed CC0 PNGs by independent decode + linear-light
//! compositing, never from the implementation under test.
//!
//! Coverage: atlas frame/flip/left-right pixels (A01), transparent underlay
//! layer order (A02), half-alpha CPU/GPU composite (A03), alt cadence at the
//! same tick (A04), fixed-flip negative control (A05), destroy/respawn
//! default sprite through the typed render service (A06), stale-handle
//! rejection with last-good (A07), Pause/Step/Stop/Save/reopen/fresh Run
//! (A08), opaque-alpha negative control (A09); character idle/run/left-right
//! (C01), jump/fall/land (C02), five-pose GPU pixels (C03), reused-capture
//! negative (C04), collision and follow camera (C05), pause/step/stop/fresh
//! (C06), main/alt same motion different cadence (C07), wrong-air negative
//! control (C08), 32-pass/33-refused budget (C09). Plus the sprites-2d
//! component-isolation pair (two same-def instances sharing one PNG with
//! independent local phase/pose/flip state).
//!
//! Run single-threaded:
//! cargo test --locked --offline --test node_sprites_flow -- --test-threads=1
use autographics_auto_driver::authoring::{
    host::Host,
    play::{Action, Frame, State},
    storage::{Fault, Storage},
};
use glam::Vec2;
use png::{BitDepth, ColorType, Decoder, Encoder};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Mutex,
};

static ORACLES: Mutex<Vec<Value>> = Mutex::new(Vec::new());

const SCENE_ATLAS: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-g09/scene.agscene");
const SCENE_ATLAS_ALT: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-alt/scene.agscene");
const SCENE_ATLAS_FLIP0: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/atlas-flip-control/scene.agscene"
);
const SCENE_ATLAS_SLOT: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-slot/scene.agscene");
const SCENE_CHARACTER: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/character-g09/scene.agscene");
const SCENE_CHARACTER_ALT: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/character-alt/scene.agscene");
const SCENE_WRONG_AIR: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/character-wrong-air/scene.agscene"
);
const SCENE_BUDGET_32: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/character-budget-32/scene.agscene"
);
const SCENE_BUDGET_33: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/character-budget-33/scene.agscene"
);
const SCENE_PAIR: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/pair-isolated/scene.agscene");

const WORLD_ATLAS: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-g09/world.at");
const WORLD_ATLAS_ALT: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-alt/world.at");
const WORLD_ATLAS_FLIP0: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-flip-control/world.at");
const WORLD_ATLAS_SLOT: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/atlas-slot/world.at");
const WORLD_CHARACTER: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/character-g09/world.at");
const WORLD_CHARACTER_ALT: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/character-alt/world.at");
const WORLD_WRONG_AIR: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/character-wrong-air/world.at"
);
const WORLD_BUDGET_32: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/character-budget-32/world.at"
);
const WORLD_BUDGET_33: &str = include_str!(
    "../../autographics-node2d-pilot/generated-sprites2d/character-budget-33/world.at"
);
const WORLD_PAIR: &str =
    include_str!("../../autographics-node2d-pilot/generated-sprites2d/pair-isolated/world.at");

const ATLAS_PNG: &[u8] =
    include_bytes!("../../autographics-node2d-pilot/fixtures/sprites2d/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png");
const CHARACTER_PNG: &[u8] =
    include_bytes!("../../autographics-node2d-pilot/fixtures/sprites2d/assets/8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df.png");

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn artifacts() -> PathBuf {
    let path = PathBuf::from(
        std::env::var("PLAN022_ARTIFACTS")
            .unwrap_or_else(|_| "D:/autostack/plan-artifacts/plan-022/auto-ui/flow".into()),
    );
    fs::create_dir_all(&path).expect("create plan022 artifacts");
    path
}

fn record(sid: &str, name: &str, passed: bool, detail: impl Into<String>) {
    ORACLES.lock().unwrap().push(json!({
        "subentry": sid, "name": name, "passed": passed, "detail": detail.into(),
    }));
}

fn oracle(sid: &str, name: &str, passed: bool, detail: impl Into<String>) -> Result<(), String> {
    let detail = detail.into();
    record(sid, name, passed, detail.clone());
    if passed {
        Ok(())
    } else {
        Err(format!("{sid}/{name}: {detail}"))
    }
}

// ---------------------------------------------------------------- harness --

/// Stage one frozen product under target/ and open a host on it. The asset
/// bytes are the fixed fixture PNGs; the file name comes from the scene
/// manifest, so the load path hash-verifies the full identity chain.
fn start_product(scene_text: &str, asset: &[u8], tag: &str) -> Result<Host, String> {
    let world_text = world_of_scene(scene_text, tag)?;
    let dir = staging_dir(scene_text, world_text, asset, tag)?;
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Default::default())?;
    let code = code.ok_or("behavior code missing")?;
    let mut host = Host::with_project(doc, registry, Some(code))?;
    host.start_selected_play()?;
    host.pause_play("");
    pump(&mut host)?;
    let play = host.play.as_ref().ok_or("Run missing after start")?;
    if play.state != State::Paused {
        return Err(format!("Run did not settle Paused: {:?}", play.state));
    }
    if let Some(error) = &play.error {
        return Err(format!("Run start faulted: {error}"));
    }
    Ok(host)
}

/// 场景文本 → 同产品的 world.at 文本（顺序表；tag 仅用于错误信息）。
fn world_of_scene(scene_text: &str, tag: &str) -> Result<&'static str, String> {
    let pairs: [(&str, &str); 10] = [
        (SCENE_ATLAS, WORLD_ATLAS),
        (SCENE_ATLAS_ALT, WORLD_ATLAS_ALT),
        (SCENE_ATLAS_FLIP0, WORLD_ATLAS_FLIP0),
        (SCENE_ATLAS_SLOT, WORLD_ATLAS_SLOT),
        (SCENE_CHARACTER, WORLD_CHARACTER),
        (SCENE_CHARACTER_ALT, WORLD_CHARACTER_ALT),
        (SCENE_WRONG_AIR, WORLD_WRONG_AIR),
        (SCENE_BUDGET_32, WORLD_BUDGET_32),
        (SCENE_BUDGET_33, WORLD_BUDGET_33),
        (SCENE_PAIR, WORLD_PAIR),
    ];
    pairs
        .iter()
        .find(|(sc, _)| *sc == scene_text)
        .map(|(_, w)| *w)
        .ok_or_else(|| format!("no frozen world for product tag {tag}"))
}

fn staging_dir(
    scene_text: &str,
    world_text: &str,
    asset: &[u8],
    tag: &str,
) -> Result<PathBuf, String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan022-sprites")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let assets = dir.join("assets");
    let scripts = dir.join("scripts");
    fs::create_dir_all(&assets).map_err(|e| e.to_string())?;
    fs::create_dir_all(&scripts).map_err(|e| e.to_string())?;
    fs::write(dir.join("scene.agscene"), scene_text).map_err(|e| e.to_string())?;
    let world_sha = scene_text
        .split("\"source\": {\"format\": \"auto-source\",\"sha256\": \"")
        .nth(1)
        .or_else(|| {
            scene_text.split("\"sha256\": \"").nth(1).and_then(|r| {
                let candidate = r.split('"').next()?;
                (candidate.len() == 64).then_some(candidate)
            })
        })
        .and_then(|r| r.split('"').next())
        .ok_or("scene source sha missing")?;
    fs::write(scripts.join(format!("{world_sha}.at")), world_text).map_err(|e| e.to_string())?;
    let assets_part = scene_text
        .split("\"assets\"")
        .nth(1)
        .ok_or("scene assets missing")?;
    let asset_name = assets_part
        .split("\"name\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .ok_or("scene asset name missing")?;
    fs::write(assets.join(asset_name), asset).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Reload a staged (or mutated) project directory without re-staging.
fn load_dir(scene: &Path) -> Result<Host, String> {
    let (doc, registry, code, _) = Storage::load_project(scene, &Default::default())?;
    let code = code.ok_or("behavior code missing")?;
    Host::with_project(doc, registry, Some(code))
}

fn pump(host: &mut Host) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(31);
    while host.play.as_ref().is_some_and(|play| play.pending()) {
        if std::time::Instant::now() >= deadline {
            return Err("PLAN-022 play tick exceeded 31 seconds".into());
        }
        host.frame(None)?;
    }
    Ok(())
}

fn step_raw(host: &mut Host, held: &[usize]) -> Result<(), String> {
    let play = host.play.as_mut().ok_or("Run missing")?;
    if play.state != State::Paused || play.pending() {
        return Err("deterministic input tick requires an idle Paused state".into());
    }
    play.resume()?;
    for action in 0..4usize {
        play.char_key(action, held.contains(&action), false);
    }
    play.dispatch(Action::default())?;
    host.pause_play("");
    pump(host)?;
    Ok(())
}

fn step(host: &mut Host, held: &[usize]) -> Result<(), String> {
    step_raw(host, held)?;
    let play = host.play.as_ref().ok_or("Run missing after tick")?;
    if let Some(error) = &play.error {
        return Err(error.clone());
    }
    if play.state != State::Paused {
        return Err(format!("tick did not return to Paused: {:?}", play.state));
    }
    Ok(())
}

fn frame(host: &Host) -> Result<&Frame, String> {
    host.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .ok_or_else(|| "accepted Play frame missing".into())
}

/// The one bound leaf of an atlas/character product (sprite + character row).
fn bound_leaf_id(host: &Host) -> Result<u64, String> {
    let leaves: Vec<u64> = host
        .editor
        .document
        .objects
        .iter()
        .filter(|o| o.sprite.is_some())
        .map(|o| o.id)
        .collect();
    if leaves.len() != 1 {
        return Err(format!("expected exactly one bound leaf, got {leaves:?}"));
    }
    Ok(leaves[0])
}

fn actor(host: &Host, id: u64) -> Result<[f32; 3], String> {
    frame(host)?
        .nodes
        .get(&id)
        .map(|node| node.translation)
        .ok_or_else(|| format!("node {id} missing"))
}

fn visual(host: &Host, id: u64) -> Result<([u32; 4], bool), String> {
    let sprites = frame(host)?
        .sprites
        .as_ref()
        .ok_or("sprite block missing")?;
    let visual = sprites
        .visual_for(id)
        .ok_or_else(|| format!("sprite target {id} missing"))?;
    Ok((visual.frame, visual.flip))
}

fn frame_and_position(host: &Host, id: u64) -> Result<([u32; 4], bool, [f32; 3], i32), String> {
    Ok((
        visual(host, id)?.0,
        visual(host, id)?.1,
        actor(host, id)?,
        frame(host)?.tick,
    ))
}

fn sprite_center_and_half(host: &Host, id: u64) -> Result<(Vec2, Vec2), String> {
    let f = frame(host)?;
    let camera = f.camera.as_ref().ok_or("camera frame missing")?;
    let position = f.nodes.get(&id).ok_or("sprite node missing")?.translation;
    let point = Vec2::new(position[0], position[1]);
    let follow = camera
        .follow_center_2d(point)
        .ok_or("camera follow projection missing")?;
    let center = camera
        .screen_2d(point, follow)
        .ok_or("camera screen projection missing")?;
    let scale = camera.letterbox_2d().ok_or("camera letterbox missing")?.0;
    let object = f
        .document
        .objects
        .iter()
        .find(|o| o.id == id)
        .ok_or("sprite object missing")?;
    Ok((
        center,
        Vec2::new(
            object.scale[0] * scale.x * 0.5,
            object.scale[1] * scale.y * 0.5,
        ),
    ))
}

fn probe(host: &mut Host, xy: [f32; 2]) -> Result<[u8; 4], String> {
    host.rt
        .probe_pixel(host.two, xy)
        .ok_or_else(|| format!("GPU pixel probe failed at {xy:?}"))
}

fn hero_side_pixels(host: &mut Host, id: u64) -> Result<([u8; 4], [u8; 4]), String> {
    let (center, half) = sprite_center_and_half(host, id)?;
    Ok((
        probe(host, [center.x - half.x * 0.55, center.y])?,
        probe(host, [center.x + half.x * 0.55, center.y])?,
    ))
}

fn sprite_cell_pixel(
    host: &mut Host,
    id: u64,
    source_x: u32,
    source_y: u32,
    flipped: bool,
) -> Result<[u8; 4], String> {
    let (center, half) = sprite_center_and_half(host, id)?;
    let shown_x = if flipped { 31 - source_x } else { source_x };
    let xy = [
        center.x + ((shown_x as f32 + 0.5) / 32.0 * 2.0 - 1.0) * half.x,
        center.y + ((source_y as f32 + 0.5) / 32.0 * 2.0 - 1.0) * half.y,
    ];
    probe(host, xy)
}

fn save_capture(host: &mut Host, name: &str) -> Result<(), String> {
    let (_, bytes, width, height) = host
        .rt
        .capture(host.two, name)
        .ok_or("GPU capture missing")?;
    fs::write(artifacts().join(name), &bytes).map_err(|e| e.to_string())?;
    println!("capture {name}: {width}x{height}, sha256={}", sha(&bytes));
    Ok(())
}

// ------------------------------------------------------------ pixel math --

struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode_png(bytes: &[u8]) -> Result<Image, String> {
    let decoder = Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size().expect("png output buffer size")];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    if info.color_type != ColorType::Rgba || info.bit_depth != BitDepth::Eight {
        return Err("PLAN-022 acceptance decoder expected RGBA8".into());
    }
    buffer.truncate(info.buffer_size());
    Ok(Image {
        width: info.width,
        height: info.height,
        rgba: buffer,
    })
}

fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * image.width + x) * 4) as usize;
    image.rgba[at..at + 4].try_into().unwrap()
}

fn encode_png(image: &Image) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    {
        let mut encoder = Encoder::new(&mut bytes, image.width, image.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&image.rgba)
            .map_err(|e| e.to_string())?;
    }
    Ok(bytes)
}

fn srgb_decode(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_encode(v: f64) -> u8 {
    let c = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (c.mul_add(255.0, 0.5).floor() as i32).clamp(0, 255) as u8
}

fn composite(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let alpha = f64::from(src[3]) / 255.0;
    [
        srgb_encode(srgb_decode(src[0]) * alpha + srgb_decode(dst[0]) * (1.0 - alpha)),
        srgb_encode(srgb_decode(src[1]) * alpha + srgb_decode(dst[1]) * (1.0 - alpha)),
        srgb_encode(srgb_decode(src[2]) * alpha + srgb_decode(dst[2]) * (1.0 - alpha)),
        255,
    ]
}

fn near(actual: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, e)| a.abs_diff(e) <= tolerance)
}

#[derive(Debug)]
struct PixelObservation {
    background: [u8; 4],
    samples: Vec<[u8; 4]>,
}

const IDLE_PIXELS: &[(u32, u32)] = &[(16, 9), (19, 9), (16, 18), (8, 20), (23, 20)];
const RUN_A_PIXELS: &[(u32, u32)] = &[(16, 9), (19, 9), (16, 18), (25, 18), (8, 20)];
const RUN_B_PIXELS: &[(u32, u32)] = &[(16, 9), (19, 9), (16, 18), (4, 19), (23, 20)];
const JUMP_PIXELS: &[(u32, u32)] = &[(16, 9), (19, 9), (16, 18), (3, 8), (28, 8)];
const FALL_PIXELS: &[(u32, u32)] = &[(16, 9), (19, 9), (16, 18), (4, 23), (27, 23)];

fn character_pixel_observation(
    host: &mut Host,
    id: u64,
    points: &[(u32, u32)],
) -> Result<PixelObservation, String> {
    let flip = visual(host, id)?.1;
    let (center, half) = sprite_center_and_half(host, id)?;
    let background = probe(host, [10.0, 10.0])?;
    let mut samples = Vec::with_capacity(points.len());
    for &(x, y) in points {
        let shown_x = if flip { 31 - x } else { x };
        let xy = [
            center.x + ((shown_x as f32 + 0.5) / 32.0 * 2.0 - 1.0) * half.x,
            center.y + ((y as f32 + 0.5) / 32.0 * 2.0 - 1.0) * half.y,
        ];
        samples.push(probe(host, xy)?);
    }
    Ok(PixelObservation {
        background,
        samples,
    })
}

fn expected_character_pixels(
    image: &Image,
    frame_x: u32,
    points: &[(u32, u32)],
    background: [u8; 4],
) -> Vec<[u8; 4]> {
    points
        .iter()
        .map(|&(x, y)| composite(pixel(image, frame_x + x, y), background))
        .collect()
}

fn character_pixels_match(
    observation: &PixelObservation,
    image: &Image,
    frame_x: u32,
    points: &[(u32, u32)],
) -> bool {
    let expected = expected_character_pixels(image, frame_x, points, observation.background);
    observation.samples.len() == expected.len()
        && observation
            .samples
            .iter()
            .zip(expected)
            .all(|(actual, expected)| near(*actual, expected, 3))
}

fn character_pixels_detail(
    observation: &PixelObservation,
    image: &Image,
    frame_x: u32,
    points: &[(u32, u32)],
) -> String {
    format!(
        "points={points:?}; actual={:?}; expected={:?}; background={:?}",
        observation.samples,
        expected_character_pixels(image, frame_x, points, observation.background),
        observation.background
    )
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- atlas --

fn run_atlas() -> Result<(), String> {
    const SID: &str = "g09/atlas";
    let mut host = start_product(SCENE_ATLAS, ATLAS_PNG, "atlas")?;
    let id = bound_leaf_id(&host)?;

    let initial = visual(&host, id)?;
    step(&mut host, &[]).map_err(|e| format!("atlas main neutral tick: {e}"))?;
    step(&mut host, &[0]).map_err(|e| format!("atlas main flip tick: {e}"))?;
    let (main_frame, main_flip) = visual(&host, id)?;
    let (left, right) = hero_side_pixels(&mut host, id)?;
    let border = {
        let (center, half) = sprite_center_and_half(&host, id)?;
        probe(&mut host, [center.x - half.x * 0.90, center.y])?
    };
    let image = decode_png(ATLAS_PNG)?;
    let alpha_source = pixel(&image, 32 + 21, 21);
    let alpha_expected = composite(alpha_source, [0, 0, 0, 255]);
    let alpha_gpu = sprite_cell_pixel(&mut host, id, 21, 21, true)?;
    save_capture(&mut host, "atlas-main-left.png")?;
    oracle(
        SID,
        "atlas_main_frame_flip_left_right",
        initial == ([0, 0, 32, 32], false)
            && main_frame == [32, 0, 32, 32]
            && main_flip
            && left == [28, 176, 204, 255]
            && right == [246, 104, 32, 255],
        format!(
            "initial={initial:?}; frame={main_frame:?} flip={main_flip}; left={left:?}; right={right:?}"
        ),
    )?;
    oracle(
        SID,
        "atlas_transparent_underlay_layer_order",
        border == [0, 0, 0, 255],
        format!("transparent source border reveals the black underlay, sample={border:?}"),
    )?;
    oracle(
        SID,
        "atlas_half_alpha_cpu_gpu_composite",
        alpha_source == [255, 255, 255, 128] && near(alpha_gpu, alpha_expected, 3),
        format!(
            "source={alpha_source:?}; independent linear-light composite={alpha_expected:?}; GPU={alpha_gpu:?}"
        ),
    )?;
    host.stop_play();

    let mut alt = start_product(SCENE_ATLAS_ALT, ATLAS_PNG, "atlas-alt")?;
    let alt_id = bound_leaf_id(&alt)?;
    step(&mut alt, &[]).map_err(|e| format!("atlas alt neutral tick: {e}"))?;
    step(&mut alt, &[0]).map_err(|e| format!("atlas alt flip tick: {e}"))?;
    let (alt_frame, alt_flip) = visual(&alt, alt_id)?;
    let (alt_left, alt_right) = hero_side_pixels(&mut alt, alt_id)?;
    save_capture(&mut alt, "atlas-alt-left.png")?;
    oracle(
        SID,
        "atlas_alt_cadence_same_tick",
        alt_frame == [64, 0, 32, 32]
            && alt_flip
            && alt_left == [132, 56, 214, 255]
            && alt_right == [224, 52, 126, 255],
        format!(
            "same tick={} alt frame={alt_frame:?} flip={alt_flip}; left={alt_left:?}; right={alt_right:?}",
            frame(&alt)?.tick
        ),
    )?;
    alt.stop_play();

    let mut control = start_product(SCENE_ATLAS_FLIP0, ATLAS_PNG, "atlas-flip-control")?;
    let ctrl_id = bound_leaf_id(&control)?;
    step(&mut control, &[])?;
    step(&mut control, &[0])?;
    let (control_frame, control_flip) = visual(&control, ctrl_id)?;
    let (control_left, control_right) = hero_side_pixels(&mut control, ctrl_id)?;
    oracle(
        SID,
        "atlas_flip_negative_control",
        control_frame == [32, 0, 32, 32]
            && !control_flip
            && control_left == [246, 104, 32, 255]
            && control_right == [28, 176, 204, 255],
        format!(
            "fixed flip=0 control must reverse the accepted left/right color oracle: {control_left:?}/{control_right:?}"
        ),
    )?;
    control.stop_play();

    let mut slot = start_product(SCENE_ATLAS_SLOT, ATLAS_PNG, "atlas-slot")?;
    step(&mut slot, &[])?;
    let f1 = frame(&slot)?;
    let slot1 = f1
        .instances
        .as_ref()
        .and_then(|x| x.get(0))
        .ok_or("spawned sprite slot missing")?;
    let visual1 = f1
        .sprites
        .as_ref()
        .and_then(|x| x.slot_visual(0))
        .ok_or("spawned sprite visual missing")?;
    let first_generation = slot1.generation;
    let first_slot_visual = visual1.visual.clone();
    step(&mut slot, &[])?;
    let f2 = frame(&slot)?;
    let slot2 = f2
        .instances
        .as_ref()
        .and_then(|x| x.get(0))
        .ok_or("respawned sprite slot missing")?;
    let visual2 = f2
        .sprites
        .as_ref()
        .and_then(|x| x.slot_visual(0))
        .ok_or("respawned sprite visual missing")?;
    let generation2 = slot2.generation;
    let default_visual = visual2.visual.clone();
    oracle(
        SID,
        "atlas_destroy_respawn_default_sprite",
        first_generation == 1
            && first_slot_visual.frame == [64, 0, 32, 32]
            && first_slot_visual.flip
            && generation2 == 2
            && default_visual.frame == [0, 0, 32, 32]
            && !default_visual.flip,
        format!(
            "slot generation1 visual={first_slot_visual:?}; generation2={generation2} default={default_visual:?}"
        ),
    )?;
    step_raw(&mut slot, &[])?;
    let stale = host_play_error(&slot);
    let accepted_tick = frame(&slot)?.tick;
    let kept = frame(&slot)?
        .sprites
        .as_ref()
        .and_then(|s| s.slot_visual(0))
        .map(|v| v.visual.clone());
    oracle(
        SID,
        "atlas_stale_handle_rejected",
        stale.contains("33") && accepted_tick == 2 && kept.as_ref() == Some(&default_visual),
        format!(
            "old-generation write error={stale:?}; last accepted tick={accepted_tick}; slot={kept:?}"
        ),
    )?;
    slot.stop_play();

    // Run/Pause/neutral Step/Stop/Save/reopen/fresh Run on an isolated copy.
    let product_dir = staging_dir(SCENE_ATLAS, WORLD_ATLAS, ATLAS_PNG, "atlas-lifecycle")?;
    let saved_project = artifacts().join("atlas-save-reopen");
    let _ = fs::remove_dir_all(&saved_project);
    copy_tree(&product_dir, &saved_project)?;
    let saved_scene = saved_project.join("scene.agscene");
    let mut lifecycle = load_dir(&saved_scene)?;
    start_run(&mut lifecycle)?;
    let before = frame(&lifecycle)?.tick;
    for _ in 0..5 {
        lifecycle.frame(None)?;
    }
    let paused_still = frame(&lifecycle)?.tick;
    lifecycle
        .play
        .as_mut()
        .ok_or("Run missing before Step")?
        .single_step()?;
    pump(&mut lifecycle)?;
    let after_step = frame(&lifecycle)?.tick;
    let paused_after_step = lifecycle
        .play
        .as_ref()
        .is_some_and(|p| p.state == State::Paused);
    lifecycle.stop_play();
    let leaf = bound_leaf_id(&lifecycle)?;
    let scene_default = lifecycle
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.id == leaf)
        .and_then(|o| o.sprite.as_ref())
        .map(|s| (s.frame, s.flip));
    let saved = lifecycle
        .storage
        .save(&mut lifecycle.editor, &saved_scene, Fault::None)?;
    let _ = saved;
    drop(lifecycle);
    let mut reopened = load_dir(&saved_scene)?;
    start_run(&mut reopened)?;
    let reopen_leaf = bound_leaf_id(&reopened)?;
    let restart =
        frame(&reopened)?.tick == 0 && visual(&reopened, reopen_leaf)? == ([0, 0, 32, 32], false);
    oracle(
        SID,
        "atlas_pause_step_stop_fresh_run",
        before == 0
            && paused_still == before
            && after_step == before + 1
            && paused_after_step
            && scene_default == Some(([0, 0, 32, 32], false))
            && restart,
        format!(
            "paused tick {before}->{paused_still}; neutral Step={after_step}; Stop scene default={scene_default:?}; reopened new Run tick0/default={restart}"
        ),
    )?;
    reopened.stop_play();

    // Re-encode only the translucent highlight as opaque. The same pixel/Auto
    // oracle must detect the intentionally wrong GPU result.
    let control_project = artifacts().join("atlas-alpha-opaque-control");
    let _ = fs::remove_dir_all(&control_project);
    copy_tree(&product_dir, &control_project)?;
    let scene_path = control_project.join("scene.agscene");
    let mut doc: Value = serde_json::from_slice(&fs::read(&scene_path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let old_name = doc["assets"][0]["name"]
        .as_str()
        .ok_or("asset filename missing")?
        .to_string();
    let old_path = control_project.join("assets").join(&old_name);
    let mut control_image = decode_png(&fs::read(&old_path).map_err(|e| e.to_string())?)?;
    for y in 20..24 {
        for x in 0..control_image.width {
            if (4..28).contains(&(x % 32)) {
                let at = ((y * control_image.width + x) * 4 + 3) as usize;
                if control_image.rgba[at] == 128 {
                    control_image.rgba[at] = 255;
                }
            }
        }
    }
    let new_bytes = encode_png(&control_image)?;
    let new_hash = sha(&new_bytes);
    let new_name = format!("{new_hash}.png");
    fs::write(control_project.join("assets").join(&new_name), &new_bytes)
        .map_err(|e| e.to_string())?;
    fs::remove_file(&old_path).map_err(|e| e.to_string())?;
    doc["assets"][0]["sha256"] = json!(new_hash);
    doc["assets"][0]["name"] = json!(new_name);
    doc["assets"][0]["byte_length"] = json!(new_bytes.len());
    for object in doc["objects"].as_array_mut().ok_or("objects missing")? {
        if object["sprite"].is_object() {
            object["sprite"]["asset"] = json!(format!("sha256:{new_hash}"));
        }
    }
    fs::write(
        &scene_path,
        serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let control_alpha = pixel(&decode_png(&new_bytes)?, 32 + 21, 21)[3];
    let mut alpha_control = load_dir(&scene_path)?;
    start_run(&mut alpha_control)?;
    let alpha_leaf = bound_leaf_id(&alpha_control)?;
    step(&mut alpha_control, &[])?;
    step(&mut alpha_control, &[0])?;
    let opaque_gpu = sprite_cell_pixel(&mut alpha_control, alpha_leaf, 21, 21, true)?;
    oracle(
        SID,
        "atlas_alpha_negative_control",
        control_alpha == 255
            && !near(opaque_gpu, alpha_expected, 3)
            && near(opaque_gpu, [255, 255, 255, 255], 3),
        format!(
            "mutated source alpha={control_alpha}; GPU={opaque_gpu:?}; original CPU expectation={alpha_expected:?}"
        ),
    )?;
    alpha_control.stop_play();
    Ok(())
}

fn host_play_error(host: &Host) -> String {
    host.play
        .as_ref()
        .and_then(|p| p.error.clone())
        .unwrap_or_default()
}

fn start_run(host: &mut Host) -> Result<(), String> {
    host.start_selected_play()?;
    host.pause_play("");
    pump(host)
}

// ------------------------------------------------------------- character --

fn run_character() -> Result<(), String> {
    const SID: &str = "g09/character";
    let character_image = decode_png(CHARACTER_PNG)?;
    let mut host = start_product(SCENE_CHARACTER, CHARACTER_PNG, "character")?;
    let id = bound_leaf_id(&host)?;
    let initial = visual(&host, id)?;
    host.play
        .as_mut()
        .ok_or("Run missing before first Step")?
        .single_step()?;
    pump(&mut host)?;
    let idle = frame_and_position(&host, id)?;
    let idle_pixels = character_pixel_observation(&mut host, id, IDLE_PIXELS)?;
    let stale_idle_for_run_a = character_pixel_observation(&mut host, id, RUN_A_PIXELS)?;
    save_capture(&mut host, "character-idle.png")?;
    for _ in 0..5 {
        host.frame(None)?;
    }
    let paused_tick = frame(&host)?.tick;
    step(&mut host, &[1])?;
    let right1 = frame_and_position(&host, id)?;
    let run_b_pixels = character_pixel_observation(&mut host, id, RUN_B_PIXELS)?;
    save_capture(&mut host, "character-run-b.png")?;
    step(&mut host, &[1])?;
    let right2 = frame_and_position(&host, id)?;
    let run_a_pixels = character_pixel_observation(&mut host, id, RUN_A_PIXELS)?;
    save_capture(&mut host, "character-run-a.png")?;
    step(&mut host, &[0])?;
    let left = frame_and_position(&host, id)?;
    let left_points = if left.0[0] == 64 {
        RUN_B_PIXELS
    } else {
        RUN_A_PIXELS
    };
    let left_pixels = character_pixel_observation(&mut host, id, left_points)?;
    save_capture(&mut host, "character-left-run.png")?;
    let x_before_idle = left.2[0];
    step(&mut host, &[])?;
    let idling = frame_and_position(&host, id)?;
    let left_idle_pixels = character_pixel_observation(&mut host, id, IDLE_PIXELS)?;
    save_capture(&mut host, "character-left-idle.png")?;
    let motion_ok = initial == ([0, 0, 32, 32], false)
        && idle.0 == [0, 0, 32, 32]
        && (idle.2[1] - 0.0).abs() < 0.01
        && right1.2[0] > idle.2[0]
        && right1.0 == [64, 0, 32, 32]
        && !right1.1
        && right2.2[0] > right1.2[0]
        && right2.0 == [32, 0, 32, 32]
        && !right2.1
        && left.2[0] < right2.2[0]
        && left.1
        && [32, 64].contains(&left.0[0])
        && (idling.2[0] - x_before_idle).abs() < 0.0001
        && idling.0 == [0, 0, 32, 32]
        && idling.1;
    oracle(
        SID,
        "character_idle_run_left_right",
        motion_ok,
        format!(
            "idle={idle:?}; right1={right1:?}; right2={right2:?}; left={left:?}; idle-again={idling:?}; paused tick={paused_tick}"
        ),
    )?;

    step(&mut host, &[1, 2])?;
    let jump = frame_and_position(&host, id)?;
    let jump_pixels = character_pixel_observation(&mut host, id, JUMP_PIXELS)?;
    save_capture(&mut host, "character-jump.png")?;
    let mut falling = None;
    for _ in 0..80 {
        step(&mut host, &[])?;
        let facts = frame_and_position(&host, id)?;
        if facts.0 == [128, 0, 32, 32] {
            falling = Some(facts);
            break;
        }
    }
    let fall = falling.ok_or("real Auto never selected the falling pose")?;
    let fall_pixels = character_pixel_observation(&mut host, id, FALL_PIXELS)?;
    save_capture(&mut host, "character-fall.png")?;
    let mut landed = None;
    for _ in 0..100 {
        step(&mut host, &[])?;
        let facts = frame_and_position(&host, id)?;
        if facts.0 == [0, 0, 32, 32] && (facts.2[1] - 0.0).abs() < 0.02 {
            landed = Some(facts);
            break;
        }
    }
    let landing = landed.ok_or("real Auto did not return to grounded idle")?;
    let landing_pixels = character_pixel_observation(&mut host, id, IDLE_PIXELS)?;
    save_capture(&mut host, "character-land.png")?;
    oracle(
        SID,
        "character_jump_fall_land",
        jump.0 == [96, 0, 32, 32]
            && jump.2[1] > idling.2[1]
            && fall.0 == [128, 0, 32, 32]
            && landing.0 == [0, 0, 32, 32]
            && (landing.2[1] - 0.0).abs() < 0.02,
        format!("jump={jump:?}; fall={fall:?}; landed={landing:?}"),
    )?;

    let pose_pixels_ok = character_pixels_match(&idle_pixels, &character_image, 0, IDLE_PIXELS)
        && character_pixels_match(&run_b_pixels, &character_image, 64, RUN_B_PIXELS)
        && character_pixels_match(&run_a_pixels, &character_image, 32, RUN_A_PIXELS)
        && left.1
        && ((left.0[0] == 64
            && character_pixels_match(&left_pixels, &character_image, 64, RUN_B_PIXELS))
            || (left.0[0] == 32
                && character_pixels_match(&left_pixels, &character_image, 32, RUN_A_PIXELS)))
        && idling.1
        && character_pixels_match(&left_idle_pixels, &character_image, 0, IDLE_PIXELS)
        && character_pixels_match(&jump_pixels, &character_image, 96, JUMP_PIXELS)
        && character_pixels_match(&fall_pixels, &character_image, 128, FALL_PIXELS)
        && character_pixels_match(&landing_pixels, &character_image, 0, IDLE_PIXELS);
    oracle(
        SID,
        "character_pose_gpu_pixels",
        pose_pixels_ok,
        format!(
            "idle: {}; run-b: {}; run-a: {}; left-run(frame={}): {}; left-idle: {}; jump: {}; fall: {}; landing: {}",
            character_pixels_detail(&idle_pixels, &character_image, 0, IDLE_PIXELS),
            character_pixels_detail(&run_b_pixels, &character_image, 64, RUN_B_PIXELS),
            character_pixels_detail(&run_a_pixels, &character_image, 32, RUN_A_PIXELS),
            left.0[0],
            character_pixels_detail(&left_pixels, &character_image, left.0[0], left_points),
            character_pixels_detail(&left_idle_pixels, &character_image, 0, IDLE_PIXELS),
            character_pixels_detail(&jump_pixels, &character_image, 96, JUMP_PIXELS),
            character_pixels_detail(&fall_pixels, &character_image, 128, FALL_PIXELS),
            character_pixels_detail(&landing_pixels, &character_image, 0, IDLE_PIXELS)
        ),
    )?;
    let reused_idle_matches_run_a =
        character_pixels_match(&stale_idle_for_run_a, &character_image, 32, RUN_A_PIXELS);
    oracle(
        SID,
        "character_reused_capture_negative_control",
        !reused_idle_matches_run_a,
        format!(
            "previous idle GPU sample reused as run-a must mismatch: {}; matches={reused_idle_matches_run_a}",
            character_pixels_detail(&stale_idle_for_run_a, &character_image, 32, RUN_A_PIXELS)
        ),
    )?;

    let camera = frame(&host)?
        .camera
        .as_ref()
        .ok_or("follow camera output missing")?;
    let actor_pos = actor(&host, id)?;
    let follow = camera
        .follow_center_2d(Vec2::new(actor_pos[0], actor_pos[1]))
        .ok_or("follow center missing")?;
    let screen = camera
        .screen_2d(Vec2::new(actor_pos[0], actor_pos[1]), follow)
        .ok_or("camera projection missing")?;
    let camera_ok = camera.target_id == id
        && camera.view == [12.0, 8.0]
        && camera.rect == [0.0, -4.0, 30.0, 12.0]
        && screen.x.is_finite()
        && screen.y.is_finite()
        && (115.0..=685.0).contains(&screen.x)
        && (0.0..=380.0).contains(&screen.y);
    for _ in 0..120 {
        step(&mut host, &[1])?;
    }
    let blocked_x = actor(&host, id)?[0];
    let blocked_ok = blocked_x < 4.0;
    oracle(
        SID,
        "character_collision_and_camera",
        camera_ok && blocked_ok,
        format!(
            "follow screen={screen:?}; rightward movement stopped at x={blocked_x:.5}, platform begins at 3.9"
        ),
    )?;

    host.stop_play();
    let product_dir = staging_dir(
        SCENE_CHARACTER,
        WORLD_CHARACTER,
        CHARACTER_PNG,
        "character-lifecycle",
    )?;
    let saved_project = artifacts().join("character-save-reopen");
    let _ = fs::remove_dir_all(&saved_project);
    copy_tree(&product_dir, &saved_project)?;
    let saved_scene = saved_project.join("scene.agscene");
    let mut lifecycle = load_dir(&saved_scene)?;
    start_run(&mut lifecycle)?;
    let lifecycle_leaf = bound_leaf_id(&lifecycle)?;
    let tick0 = frame(&lifecycle)?.tick;
    let default = visual(&lifecycle, lifecycle_leaf)?;
    lifecycle.stop_play();
    let saved = lifecycle
        .storage
        .save(&mut lifecycle.editor, &saved_scene, Fault::None)?;
    let _ = saved;
    drop(lifecycle);
    let mut reopened = load_dir(&saved_scene)?;
    start_run(&mut reopened)?;
    let reopen_leaf = bound_leaf_id(&reopened)?;
    let fresh =
        frame(&reopened)?.tick == 0 && visual(&reopened, reopen_leaf)? == ([0, 0, 32, 32], false);
    oracle(
        SID,
        "character_pause_step_stop_fresh_run",
        paused_tick == idle.3 && tick0 == 0 && default == ([0, 0, 32, 32], false) && fresh,
        format!(
            "neutral Step/paused tick={idle:?}/{paused_tick}; saved default={default:?}; reopened tick0/default={fresh}"
        ),
    )?;
    reopened.stop_play();

    // A second run of each cadence receives identical input; only the frame
    // cadence may differ, never the fixed-tick character path.
    let compare_cadence =
        |scene: &str, asset: &[u8], tag: &str| -> Result<Vec<([u32; 4], bool, [f32; 3])>, String> {
            let mut h = start_product(scene, asset, tag)?;
            let cid = bound_leaf_id(&h)?;
            step(&mut h, &[])?;
            let mut rows = vec![];
            for _ in 0..3 {
                step(&mut h, &[1])?;
                let (f, flip, p, _) = frame_and_position(&h, cid)?;
                rows.push((f, flip, p));
            }
            h.stop_play();
            Ok(rows)
        };
    let main_rows = compare_cadence(SCENE_CHARACTER, CHARACTER_PNG, "character-cadence-main")?;
    let alt_rows = compare_cadence(SCENE_CHARACTER_ALT, CHARACTER_PNG, "character-cadence-alt")?;
    let same_motion = main_rows
        .iter()
        .zip(&alt_rows)
        .all(|(a, b)| (a.2[0] - b.2[0]).abs() < 0.00001 && (a.2[1] - b.2[1]).abs() < 0.00001)
        && main_rows[1].0 == [32, 0, 32, 32]
        && alt_rows[1].0 == [64, 0, 32, 32];
    oracle(
        SID,
        "character_main_alt_same_motion",
        same_motion,
        format!("main={main_rows:?}; alt={alt_rows:?}"),
    )?;

    let mut control = start_product(SCENE_WRONG_AIR, CHARACTER_PNG, "character-wrong-air")?;
    let wrong_leaf = bound_leaf_id(&control)?;
    step(&mut control, &[])?;
    step(&mut control, &[2])?;
    let wrong_rise = visual(&control, wrong_leaf)?;
    let wrong_rise_pixels = character_pixel_observation(&mut control, wrong_leaf, JUMP_PIXELS)?;
    let wrong_rise_matches_jump =
        character_pixels_match(&wrong_rise_pixels, &character_image, 96, JUMP_PIXELS);
    oracle(
        SID,
        "character_wrong_air_run_negative_control",
        wrong_rise.0 == [32, 0, 32, 32] && !wrong_rise_matches_jump,
        format!(
            "intentional run-pose-on-jump mutation is rejected by the GPU pixel oracle: frame={wrong_rise:?}; {}; matches-jump={wrong_rise_matches_jump}",
            character_pixels_detail(&wrong_rise_pixels, &character_image, 96, JUMP_PIXELS)
        ),
    )?;
    control.stop_play();

    let mut budget = start_product(SCENE_BUDGET_32, CHARACTER_PNG, "character-budget-32")?;
    step(&mut budget, &[])?;
    let pass32 = host_play_error(&budget);
    budget.stop_play();
    let mut budget33 = start_product(SCENE_BUDGET_33, CHARACTER_PNG, "character-budget-33")?;
    step_raw(&mut budget33, &[])?;
    let fail33 = host_play_error(&budget33);
    let query_budget = fail33.contains("10") || fail33.contains("QueryBudget");
    oracle(
        SID,
        "character_budget_32_pass_33_refused",
        pass32.is_empty() && query_budget,
        format!("32-credit error={pass32:?}; 33-credit error={fail33:?}"),
    )?;
    budget33.stop_play();
    Ok(())
}

// ------------------------------------------------------------------ pair --

/// Component isolation on the sprites-2d chain: two same-definition
/// AnimatedSprite instances share one PNG/frame table but keep independent
/// local phase/pose/flip state, each publishing its own slot visual per tick.
fn run_pair() -> Result<(), String> {
    let mut host = start_product(SCENE_PAIR, ATLAS_PNG, "pair")?;
    step(&mut host, &[])?;
    let f1 = frame(&host)?;
    let sprites = f1.sprites.as_ref().ok_or("sprite block missing")?;
    let slot_a = sprites
        .slot_visual(0)
        .ok_or("slot a missing")?
        .visual
        .clone();
    let slot_b = sprites
        .slot_visual(1)
        .ok_or("slot b missing")?
        .visual
        .clone();
    step(&mut host, &[])?;
    let f2 = frame(&host)?;
    let sprites2 = f2.sprites.as_ref().ok_or("sprite block missing")?;
    let slot_a2 = sprites2
        .slot_visual(0)
        .ok_or("slot a missing")?
        .visual
        .clone();
    let slot_b2 = sprites2
        .slot_visual(1)
        .ok_or("slot b missing")?
        .visual
        .clone();
    // tick1: a phase1 -> pose0 (ruby), b phase3 -> pose1 (amber) + flip
    // tick2: a phase2 -> pose0, b phase4 -> pose2 (magenta) + flip
    let independent = slot_a.frame == [0, 0, 32, 32]
        && !slot_a.flip
        && slot_b.frame == [32, 0, 32, 32]
        && slot_b.flip
        && slot_a2.frame == [32, 0, 32, 32]
        && !slot_a2.flip
        && slot_b2.frame == [64, 0, 32, 32]
        && slot_b2.flip;
    record(
        "plan022/pair",
        "pair_component_state_isolation",
        independent,
        format!("tick1 a={slot_a:?} b={slot_b:?}; tick2 a={slot_a2:?} b={slot_b2:?}"),
    );
    if !independent {
        return Err("pair isolation failed".into());
    }
    host.stop_play();
    Ok(())
}

/// V-06 consumption control (mirrors node_components_flow's damaged-world
/// recovery): a one-byte corruption of the frozen world copy is rejected at
/// load by the closure-identity check; restoring the exact bytes recovers a
/// green run on the same entry. The same control replays for the staged PNG
/// asset (identity = complete original bytes).
#[test]
fn v06_damaged_copies_rejected_then_recovery_green() {
    let dir = staging_dir(SCENE_ATLAS, WORLD_ATLAS, ATLAS_PNG, "v06-damaged").expect("stage");
    let scene = dir.join("scene.agscene");
    let world_sha = SCENE_ATLAS
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene source sha")
        .to_string();
    let world = dir.join("scripts").join(format!("{world_sha}.at"));
    let original_world = fs::read(&world).expect("world bytes");
    let asset_name = SCENE_ATLAS
        .split("\"assets\"")
        .nth(1)
        .and_then(|r| r.split("\"name\": \"").nth(1))
        .and_then(|r| r.split('"').next())
        .expect("asset name")
        .to_string();
    let asset = dir.join("assets").join(&asset_name);
    let original_asset = fs::read(&asset).expect("asset bytes");

    // damaged world: load must reject with the named closure-identity error
    let mut damaged = original_world.clone();
    damaged[0] ^= 0xFF;
    fs::write(&world, &damaged).expect("write damaged world");
    let load_error = Storage::load_project(&scene, &Default::default())
        .err()
        .expect("damaged world must be rejected at load");
    assert!(
        load_error.contains("identity") || load_error.contains("behavior source"),
        "unexpected load error for damaged world: {load_error}"
    );
    fs::write(&world, &original_world).expect("restore world");
    let mut host = load_dir(&scene).expect("reload after restore");
    start_run(&mut host).expect("green run after world restore");
    let leaf = bound_leaf_id(&host).expect("leaf");
    assert_eq!(visual(&host, leaf), Ok(([0, 0, 32, 32], false)));
    host.stop_play();

    // damaged asset: load must reject, restore the exact bytes, run green
    let mut damaged = original_asset.clone();
    damaged[0] ^= 0xFF;
    fs::write(&asset, &damaged).expect("write damaged asset");
    let load_error = Storage::load_project(&scene, &Default::default())
        .err()
        .expect("damaged asset must be rejected at load");
    assert!(
        load_error.contains("asset hash/length")
            || load_error.contains(&asset_name)
            || load_error.contains("identity"),
        "unexpected load error for damaged asset: {load_error}"
    );
    fs::write(&asset, &original_asset).expect("restore asset");
    let mut host = load_dir(&scene).expect("reload after asset restore");
    start_run(&mut host).expect("green run after asset restore");
    let leaf = bound_leaf_id(&host).expect("leaf");
    assert_eq!(visual(&host, leaf), Ok(([0, 0, 32, 32], false)));
    host.stop_play();
    record(
        "plan022/v06",
        "v06_damaged_copies_rejected_then_recovery_green",
        true,
        format!(
            "world+asset one-byte corruption rejected at load; exact restore recovered green runs"
        ),
    );
}

/// V-04 owner control (020's 20-round pattern on the sprites chain): 20
/// host-driven Stop/fresh rounds on the slot product rebuild the Run every
/// time (tick 0, empty slot, document default sprite), the scene document
/// (including every sprite binding) stays byte-stable across all rounds, and
/// the shared PNG resource keeps rendering (texture cache owner lives).
#[test]
fn v04_twenty_round_owner_fresh_runs() {
    let mut host = start_product(SCENE_ATLAS_SLOT, ATLAS_PNG, "v04-owner").expect("start");
    let scene_before = host.editor.document.objects.len();
    let sprite_bindings_before: Vec<_> = host
        .editor
        .document
        .objects
        .iter()
        .map(|o| o.sprite.clone())
        .collect();
    host.stop_play();
    for round in 0..20 {
        let mut h = start_product(SCENE_ATLAS_SLOT, ATLAS_PNG, "v04-owner").expect("fresh start");
        let f = frame(&h).expect("frame");
        assert_eq!(f.tick, 0, "round {round}: fresh run starts at tick 0");
        let sprites = f.sprites.as_ref().expect("sprite block");
        assert!(
            sprites.slot_visual(0).is_none(),
            "round {round}: slot must start empty"
        );
        let leaf = bound_leaf_id(&h).expect("leaf");
        assert_eq!(
            visual(&h, leaf).expect("leaf visual"),
            ([0, 0, 32, 32], false),
            "round {round}: document default sprite restored"
        );
        h.stop_play();
    }
    let mut final_host =
        start_product(SCENE_ATLAS_SLOT, ATLAS_PNG, "v04-owner").expect("final start");
    step(&mut final_host, &[]).expect("spawn tick");
    let f = frame(&final_host).expect("frame");
    let sprites = f.sprites.as_ref().expect("sprite block");
    let slot = sprites.slot_visual(0).expect("slot after spawn");
    assert_eq!(slot.generation, 1, "generation restarts per Run");
    assert_eq!(slot.visual.frame, [64, 0, 32, 32]);
    assert_eq!(slot.visual.flip, true);
    final_host.stop_play();
    let bindings_after: Vec<_> = final_host
        .editor
        .document
        .objects
        .iter()
        .map(|o| o.sprite.clone())
        .collect();
    assert_eq!(scene_before, final_host.editor.document.objects.len());
    assert_eq!(
        sprite_bindings_before, bindings_after,
        "document sprite bindings unchanged across 20 fresh rounds"
    );
    record(
        "plan022/v04",
        "v04_twenty_round_owner_fresh_runs",
        true,
        "20 stop/fresh rounds: tick0 + empty slot + document default sprite each round; scene bindings byte-stable; shared PNG renders after respawn (generation 1, [64,0] flip)",
    );
}

fn write_receipt() -> Result<(), String> {
    let receipt = json!({
        "schema": "auto-ui.plan022.node-sprites-receipt/1",
        "oracles": ORACLES.lock().unwrap().clone(),
    });
    let path = artifacts().join("node-sprites-receipt.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[test]
fn node_sprites_real_auto_gpu_acceptance() {
    let atlas = run_atlas();
    if let Err(error) = &atlas {
        record("g09/atlas", "adapter_fatal", false, error.clone());
    }
    let character = run_character();
    if let Err(error) = &character {
        record("g09/character", "adapter_fatal", false, error.clone());
    }
    let pair = run_pair();
    if let Err(error) = &pair {
        record("plan022/pair", "adapter_fatal", false, error.clone());
    }
    write_receipt().expect("write PLAN-022 sprites receipt");
    assert!(
        atlas.is_ok() && character.is_ok() && pair.is_ok(),
        "atlas={atlas:?}; character={character:?}; pair={pair:?}"
    );
    let rows = ORACLES.lock().unwrap();
    let failed: Vec<_> = rows.iter().filter(|row| row["passed"] != true).collect();
    assert!(failed.is_empty(), "failed PLAN-022 oracles: {failed:?}");
}
