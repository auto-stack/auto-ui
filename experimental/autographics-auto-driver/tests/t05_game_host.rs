//! auto-ui PLAN-023 T-05 V-04 host/GPU lifecycle battery on the GENERATED
//! components-2d-game chain (world sha ed98b0b5... — the T-03 product,
//! never modified; derived variant projects are staged under target/ with
//! the scene's behavior sha re-pointed at the variant text, the committed
//! product tree untouched).
//!   a. GPU render of the full scene (pad/coins/char/ground/pool) with exact
//!      actor positions and the T-04 measured palette constants;
//!   b. walk+collect on GPU with exact position/visibility observations
//!      (coins despawn on collect; the character follows input both ways;
//!      closeout rows + real mixer dispositions per carrying tick);
//!   c. Pause/Step/Stop lifecycle: Start exactly once per Run (Resume xN
//!      never re-Starts — a repeat Start would double-spawn into the frozen
//!      8-slot pool -> named fault 31, or stale binds -> no collect), paused
//!      probe-dispatch = exactly one neutral tick (no input edges), render-
//!      only frames advance nothing, single_step = one neutral tick whose
//!      closeout packet is accepted SuppressedByStep (step ticks consume no
//!      ordinals — the next collect carries the next ordinal; the generated
//!      game cannot emit a cue on a neutral step, documented), and the
//!      delivered mixer step-suppression mechanism re-proven on the
//!      GENERATED cue table (cue-bearing step packet: suppressed, no voice,
//!      ordinal consumed, no replay on resume);
//!   d. 20 stop/fresh rounds: full model rebuild each round (count 0,
//!      ordinals restart at 1, coins back, document hash unchanged, audio
//!      service rebuilt for the Run — mixer.begin rejects stale Runs), every
//!      round's carrying packet disposition Played, shared legal resources
//!      alive throughout (same slot templates/generations, full palette
//!      renders after the last round).
//! Fault semantics on the generated chain:
//!   e. preaccept (a derived variant whose collect emits the unknown cue 3):
//!      the closeout's own audio unit raises the named cue-range fault 53
//!      before any mixer accept — the last-good visual/HUD is preserved
//!      (count still "0", previous tick's frame), no new voice; the exact
//!      restore (pristine project) runs green immediately after;
//!   f. postaccept device Fault via the delivered injection mechanism (the
//!      native stream error-callback path, Mixer::output_fault): named
//!      "audio_fault: ..." on the play run, Run voices stop, the mixer
//!      renders silence from then on (already-emitted PCM is irrevocable —
//!      the device consumed it; nothing replays), and the next start
//!      releases the failed stream, opens a replacement and runs green;
//!   g. native re-observation: the real default device consumes the
//!      generated chain's captured closeout packet — nonzero submitted
//!      frames, ring match at head for one cue vs the bank slice at the
//!      closeout's request gain, 0 deadline misses. The device is the ToDesk
//!      remote-desktop VIRTUAL speaker: observation covers samples submitted
//!      into the opened stream, not acoustic output — the acoustic/physical-
//!      device dimension remains the standing named gap (Plan D-04).
//! Project staging: the generated product's scene/world/assets are staged
//! under target/ (load_project resolves assets/<blob> beside the scene).
//! Run (single instance, GPU + audio):
//!   CARGO_TERM_COLOR=never RUST_MIN_STACK=16777216 cargo test --locked \
//!     --offline --test t05_game_host -- --test-threads=1 --nocapture
use autographics_auto_driver::{
    authoring::{
        assets,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        host::Host,
        play::Action,
        public_audio::{
            self,
            mixer::{Accepted, Mixer, Packet, Request},
        },
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-game/game/scene.agscene");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-game/game/world.at");
const CLOSURE_ASSETS: &str =
    "../autographics-node2d-pilot/generated-components2d-game/game/closure/assets";
/// T-03 pin: the generated world under test is the committed product.
const WORLD_SHA: &str = "ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0";
const BANK_SHA: &str = "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d";
const FONT_SHA: &str = "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa";
/// Coin cue span (generated scene cue table, id = declaration index).
const COIN_SPAN: (usize, usize) = (0, 7403);

const DT: f32 = 1.0 / 60.0;

fn sha256_hex(bytes: &[u8]) -> String {
    agp::renderer::capture::sha256_hex(bytes)
}

// ------------------------------------------------------------- staging ----

fn copy_assets(dir: &Path) {
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&assets_dir).expect("assets dir");
    let closure = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CLOSURE_ASSETS);
    for (src, blob) in [
        (
            "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav",
            format!("{BANK_SHA}.wav"),
        ),
        ("lilita_one_regular.ttf", format!("{FONT_SHA}.ttf")),
    ] {
        std::fs::copy(closure.join(src), assets_dir.join(blob))
            .unwrap_or_else(|e| panic!("copy asset {src}: {e}"));
    }
}

/// Stage the pristine generated project exactly as load_project resolves it.
fn stage_pristine(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/t05-host")
        .join(format!("{}-{tag}", std::process::id()))
        .join("game");
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), SCENE).expect("write scene");
    std::fs::write(scripts.join(format!("{WORLD_SHA}.at")), WORLD).expect("write script");
    copy_assets(&dir);
    dir
}

/// Stage a project whose world is a DERIVED variant: the scene's bound
/// behavior sha is re-pointed at sha256(variant_world). The staged copy is
/// hash-verified by the load path against its file name.
fn stage_variant(tag: &str, variant_world: &str) -> PathBuf {
    let vsha = sha256_hex(variant_world.as_bytes());
    assert_ne!(vsha, WORLD_SHA, "variant differs from the product");
    let scene = SCENE.replacen(WORLD_SHA, &vsha, 1);
    assert_ne!(scene, SCENE, "scene sha re-pointed");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/t05-host")
        .join(format!("{}-{tag}", std::process::id()))
        .join("game");
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), &scene).expect("write scene");
    std::fs::write(scripts.join(format!("{vsha}.at")), variant_world).expect("write script");
    copy_assets(&dir);
    dir
}

fn load(
    dir: &Path,
) -> (
    autographics_auto_driver::authoring::document::Document,
    assets::Registry,
    std::sync::Arc<CodePayload>,
) {
    let (d, r, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .unwrap_or_else(|e| panic!("load staged project: {e:?}"));
    (d, r, code.expect("behavior code"))
}

// ---------------------------------------------------------- host harness --

fn host_start(h: &mut Host, tag: &str) {
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle_host(h);
}

fn host_for(tag: &str) -> Host {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "generated world pinned");
    let dir = stage_pristine(tag);
    let (doc, registry, code) = load(&dir);
    let mut h = Host::with_project(doc, registry, Some(code)).expect("host");
    host_start(&mut h, tag);
    h
}

fn settle_host(h: &mut Host) {
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

/// One live walk tick on the host: returns the tick's closeout audio rows
/// (ordinal, cue, gain) and the real mixer disposition.
fn step_play(
    h: &mut Host,
    held: u32,
) -> (Vec<(i32, usize, f32)>, Option<Accepted>) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        for action in 0..4usize {
            if held & (1 << action) != 0 {
                p.char_held_restore(action);
            } else {
                p.char_key(action, false, false);
            }
        }
        if p.dispatch(Action::default()).is_err() {
            return (vec![], None);
        }
    }
    h.pause_play("");
    settle_host(h);
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    let rows = f
        .audio
        .as_ref()
        .map(|p| {
            p.requests
                .iter()
                .map(|r| (r.ordinal, r.cue, r.gain))
                .collect()
        })
        .unwrap_or_default();
    (rows, f.audio_disposition)
}

fn play_error(h: &Host) -> String {
    h.play.as_ref().unwrap().error.clone().unwrap_or_default()
}

fn tick_of(h: &Host) -> i32 {
    h.play.as_ref().unwrap().tick()
}

fn last_frame_tick(h: &Host) -> i32 {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .tick
}

fn last_audio(h: &Host) -> (Option<(i32, usize)>, Option<Accepted>) {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    (
        f.audio
            .as_ref()
            .map(|p| (p.tick, p.requests.len())),
        f.audio_disposition,
    )
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn slot_text(h: &Host, i: usize) -> String {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.hud.as_ref().expect("hud frame").slots[i].text.clone()
}

/// Walk right until the closeout carries rows; returns (rows, disposition).
fn walk_until_cue(h: &mut Host, tag: &str, max: usize) -> (Vec<(i32, usize, f32)>, Accepted) {
    for _ in 0..max {
        let (rows, disp) = step_play(h, 1 << 1);
        if !rows.is_empty() {
            return (rows, disp.expect("carrying tick has a disposition"));
        }
        assert!(play_error(h).is_empty(), "{tag} walk fault: {}", play_error(h));
    }
    panic!("{tag}: no cue within {max} ticks");
}

// ------------------------------------------------------------ GPU pixels --

type Px = ([f32; 2], [u8; 4]);

fn pixel_classes(h: &mut Host) -> Vec<Px> {
    let mut points = Vec::new();
    for gy in 0..95 {
        for gx in 0..200 {
            points.push([(gx * 4 + 2) as f32, (gy * 4 + 2) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    points
        .iter()
        .zip(px.iter())
        .map(|(p, c)| (*p, *c))
        .collect()
}

fn rgb(c: [u8; 4]) -> (f32, f32, f32) {
    (
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
    )
}

fn is_white(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    r > 0.85 && g > 0.85 && b > 0.85
}

// The generated game's rendered palette, measured by the T-04 calibration
// probe (declared scene rgba -> gamma-lifted sRGB sample):
//   body   [0.9,0.5,0.25]  -> (0.953,0.737,0.537)
//   coins  [0.95,0.85,0.2] -> (0.976,0.929,0.486)
//   pad    [0.9,0.8,0.3]   -> (0.953,0.906,0.584)
//   ground [0.45 x3]       -> (0.702,0.702,0.702)
//   pool   [0.1 x3]        -> (0.349,0.349,0.349)
/// Character body quad.
fn is_body(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    r > 0.90 && g > 0.60 && g < 0.85 && b > 0.40 && b < 0.65
}
/// Coins and the pad (both declared golds).
fn is_gold(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    r > 0.90 && g > 0.85 && b > 0.35 && b < 0.70
}
/// Ground quad (neutral 0.702 gray).
fn is_ground(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    (r - g).abs() < 0.05 && (g - b).abs() < 0.05 && r > 0.63 && r < 0.78
}
/// Hazard pool band (neutral 0.349 gray).
fn is_pool(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    (r - g).abs() < 0.05 && (g - b).abs() < 0.05 && r > 0.28 && r < 0.45
}

fn in_box(
    px: &[Px],
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    pred: fn([u8; 4]) -> bool,
) -> Vec<Px> {
    px.iter()
        .filter(|(p, c)| p[0] >= x0 && p[0] <= x1 && p[1] >= y0 && p[1] <= y1 && pred(*c))
        .copied()
        .collect()
}

// ------------------------------------------------- derived variant worlds --

/// Unknown-cue variant: all three collect branches emit cue 3 (the generated
/// scene declares exactly cues 0..2). The closeout's own audio unit rejects
/// the unknown cue with the named range fault 53 BEFORE any mixer accept.
fn unknown_cue_world() -> String {
    let marker = "   r_count = r_count + 1\n   r_gate = 1\n   game_cue(0)\n";
    assert_eq!(WORLD.matches(marker).count(), 3, "three collect sites");
    let w = WORLD.replace(marker, "   r_count = r_count + 1\n   r_gate = 1\n   game_cue(3)\n");
    assert_ne!(w, WORLD, "unknown-cue derivation");
    w
}

// ============================================================ V-04 (a) ====

/// V-04 (a): the generated game renders the FULL scene on real GPU with
/// exact positions: the character rests at the discrete oracle (2.0, 0.502)
/// after 26 neutral ticks; the HUD glyph clusters sit at the viewport
/// anchors ("0" left, "Playing" right); coins 1..3, the pad, the ground and
/// the hazard pool render inside their calibrated windows with the T-04
/// measured palette constants; every scene object is at its declared world
/// position (the camera follow window [0,24]x[-8,8] at view 24x16 pins the
/// projection: screen ~= (397.5 + (wx-12)*23.6, 190 - wy*23.6)).
#[test]
fn v04_a_gpu_full_scene_exact_positions() {
    let mut h = host_for("v04a");
    assert!(play_error(&h).is_empty(), "fault: {}", play_error(&h));
    // discrete settle oracle: 26 neutral ticks land the character at rest.
    for _ in 0..26 {
        let _ = step_play(&mut h, 0);
    }
    let a0 = actor(&h, 1);
    assert!((a0[0] - 2.0).abs() < 1e-2, "rest x: {a0:?}");
    assert!((a0[1] - 0.502).abs() < 1e-2, "landed at rest: {a0:?}");
    assert_eq!(slot_text(&h, 0), "0", "count HUD at rest");
    assert_eq!(slot_text(&h, 1), "Playing", "state HUD at rest");

    let cap = pixel_classes(&mut h);
    // HUD glyph clusters at the viewport anchors (clip.x + 8 / right-aligned).
    let left = in_box(&cap, 110.0, 240.0, 2.0, 58.0, is_white);
    let right = in_box(&cap, 520.0, 686.0, 2.0, 58.0, is_white);
    assert!(left.len() >= 3, "count glyph pixels: {}", left.len());
    assert!(right.len() >= 3, "state glyph pixels: {}", right.len());
    let lx = left.iter().map(|(p, _)| p[0]).sum::<f32>() / left.len() as f32;
    let rx = right.iter().map(|(p, _)| p[0]).sum::<f32>() / right.len() as f32;
    assert!(lx > 110.0 && lx < 200.0, "left cluster at viewport anchor: {lx}");
    assert!(rx > 540.0 && rx < 686.0, "right cluster right-aligned: {rx}");
    // world objects in their calibrated windows (exact slots per palette).
    let body = in_box(&cap, 150.0, 176.0, 166.0, 196.0, is_body);
    let c1 = in_box(&cap, 293.0, 317.0, 149.0, 175.0, is_gold);
    let c2 = in_box(&cap, 388.0, 412.0, 149.0, 175.0, is_gold);
    let c3 = in_box(&cap, 483.0, 507.0, 149.0, 175.0, is_gold);
    let pad = in_box(&cap, 554.0, 580.0, 166.0, 192.0, is_gold);
    let ground = in_box(&cap, 395.0, 405.0, 195.0, 210.0, is_ground);
    let pool = in_box(&cap, 300.0, 500.0, 315.0, 350.0, is_pool);
    assert!(body.len() >= 3, "character renders: {}", body.len());
    assert!(c1.len() >= 3, "coin 1 renders: {}", c1.len());
    assert!(c2.len() >= 3, "coin 2 renders: {}", c2.len());
    assert!(c3.len() >= 3, "coin 3 renders: {}", c3.len());
    assert!(pad.len() >= 3, "pad renders: {}", pad.len());
    assert!(ground.len() >= 3, "ground renders: {}", ground.len());
    assert!(pool.len() >= 3, "hazard pool renders: {}", pool.len());
    // the white HUD text is the only white in the frame.
    let white_below = in_box(&cap, 0.0, 800.0, 58.0, 380.0, is_white);
    assert!(white_below.is_empty(), "no HUD leak below the band");
    println!(
        "V04_A full scene: actor=({:.3},{:.3}) body={}px coins={}/{}/{}px pad={}px ground={}px pool={}px hud_left={}px@{:.0} hud_right={}px@{:.0}",
        a0[0], a0[1], body.len(), c1.len(), c2.len(), c3.len(), pad.len(), ground.len(), pool.len(), left.len(), lx, right.len(), rx
    );
    h.stop_play();
}

// ============================================================ V-04 (b) ====

/// V-04 (b): walk+collect on GPU with exact position/visibility observations.
/// Walking right moves the character exactly 4 world units/s; coin 1
/// despawns on collect (its pixel window empties while coins 2/3 persist and
/// the pad stays); the closeout row (1, cue 0, gain 0.5) hits the real
/// service with disposition Played and the HUD latches "1". Walking LEFT
/// moves the character back by the same exact per-tick step (the character
/// follows input both ways); walking right again collects coin 2 (window
/// empties, ordinal 2, Played) while coin 3 and the pad persist.
#[test]
fn v04_b_gpu_walk_collect_positions_visibility() {
    let mut h = host_for("v04b");
    for _ in 0..26 {
        let _ = step_play(&mut h, 0);
    }
    let x0 = actor(&h, 1)[0];
    assert!((x0 - 2.0).abs() < 1e-2, "rest: {x0}");
    // walk right: exact per-tick displacement.
    for _ in 0..20 {
        let _ = step_play(&mut h, 1 << 1);
    }
    let x20 = actor(&h, 1)[0];
    assert!(
        (x20 - (x0 + 20.0 * 4.0 * DT)).abs() < 1e-2,
        "exact walk speed: {x20} vs {}",
        x0 + 20.0 * 4.0 * DT
    );
    // to the first collect.
    let (rows1, disp1) = walk_until_cue(&mut h, "v04b", 300);
    assert_eq!(rows1, vec![(1, 0, 0.5)], "first collect: ordinal 1, coin cue, gain 0.5");
    assert_eq!(disp1, Accepted::Played, "real service accepted the bundle");
    assert_eq!(slot_text(&h, 0), "1", "count latched on GPU");
    assert_eq!(slot_text(&h, 1), "Playing");
    let x1 = actor(&h, 1)[0];
    // coin 1 despawned; coins 2/3 and the pad persist.
    let cap1 = pixel_classes(&mut h);
    let c1 = in_box(&cap1, 293.0, 317.0, 149.0, 175.0, is_gold);
    let c2 = in_box(&cap1, 388.0, 412.0, 149.0, 175.0, is_gold);
    let c3 = in_box(&cap1, 483.0, 507.0, 149.0, 175.0, is_gold);
    let pad1 = in_box(&cap1, 554.0, 580.0, 166.0, 192.0, is_gold);
    assert!(c1.len() < 2, "coin 1 despawned on collect: {}", c1.len());
    assert!(c2.len() >= 3 && c3.len() >= 3 && pad1.len() >= 3, "rest persists");
    // walk LEFT: the character follows input back, exact per-tick step.
    for _ in 0..10 {
        let _ = step_play(&mut h, 1 << 0);
    }
    let xl = actor(&h, 1)[0];
    assert!(
        (xl - (x1 - 10.0 * 4.0 * DT)).abs() < 1e-2,
        "exact left step: {xl} vs {}",
        x1 - 10.0 * 4.0 * DT
    );
    // no spurious cues while re-crossing the dead coin 1 spot.
    for _ in 0..10 {
        let (rows, _) = step_play(&mut h, 1 << 1);
        assert!(rows.is_empty(), "despawned coin 1 never re-fires: {rows:?}");
    }
    // second collect: ordinal 2, coin 2 despawns, coin 3 + pad persist.
    let (rows2, disp2) = walk_until_cue(&mut h, "v04b", 400);
    assert_eq!(rows2, vec![(2, 0, 0.5)], "second collect: ordinal 2");
    assert_eq!(disp2, Accepted::Played);
    assert_eq!(slot_text(&h, 0), "2");
    let cap2 = pixel_classes(&mut h);
    let c2b = in_box(&cap2, 388.0, 412.0, 149.0, 175.0, is_gold);
    let c3b = in_box(&cap2, 483.0, 507.0, 149.0, 175.0, is_gold);
    assert!(c2b.len() < 2, "coin 2 despawned: {}", c2b.len());
    assert!(c3b.len() >= 3, "coin 3 persists: {}", c3b.len());
    println!(
        "V04_B walk+collect: x0={x0:.3} x20={x20:.3} collect1@x={x1:.3} left->x={xl:.3} coin1 gone, coin2 gone at ord 2, dispositions Played"
    );
    h.stop_play();
}

// ============================================================ V-04 (c) ====

/// V-04 (c): Pause/Step/Stop lifecycle on the GPU host. Start ran exactly
/// once per Run — after 12 explicit Resume/dispatch cycles AND the whole
/// walk to the first collect (~70 more Resume/dispatch cycles), the game is
/// fault-free, the collect fires with ordinal 1 (a repeated Start would
/// double-spawn 5 more objects into the frozen 8-slot pool -> named fault
/// 31, or leave the trigger binds on stale generations -> no collect) and
/// the HUD latches "1". Paused probe-dispatch advances exactly one neutral
/// tick (no input edges: x frozen, gravity only); render-only frames advance
/// nothing; a single Step advances exactly one neutral tick whose closeout
/// packet the real mixer accepts SuppressedByStep; steps consume no
/// ordinals (the next collect after 3 steps carries ordinal 2 — no sound
/// replay on Resume). The generated game cannot emit a cue on a neutral
/// step (step input is fully neutral and events require entering a source
/// this tick), so the cue-bearing suppression half is proven against the
/// GENERATED cue table with the delivered mixer mechanism: a cue-bearing
/// packet accepted with step=true comes back SuppressedByStep, inserts no
/// voice, consumes its ordinal (a later live attempt at that ordinal is
/// rejected) and renders nothing after resume.
#[test]
fn v04_c_gpu_pause_step_stop_lifecycle() {
    let mut h = host_for("v04c");
    for _ in 0..26 {
        let _ = step_play(&mut h, 0);
    }
    assert!((actor(&h, 1)[1] - 0.502).abs() < 1e-2, "settled");
    let x_settled = actor(&h, 1)[0];

    // Start exactly once: 12 explicit Resume x dispatch neutral cycles.
    for _ in 0..12 {
        {
            let p = h.play.as_mut().unwrap();
            let _ = p.resume();
            p.dispatch(Action::default()).expect("dispatch");
        }
        h.pause_play("");
        settle_host(&mut h);
        assert!(play_error(&h).is_empty(), "resume cycle fault");
    }
    assert_eq!(tick_of(&h), 26 + 12, "each Resume cycle advanced exactly one tick");
    assert_eq!(actor(&h, 1)[0], x_settled, "neutral cycles never moved the character");

    // paused probe-dispatch: exactly one neutral tick, no input edges.
    let before = tick_of(&h);
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.dispatch(Action::default());
    }
    settle_host(&mut h);
    assert_eq!(tick_of(&h), before + 1, "probe dispatch = one tick");
    let (audio_probe, _disp_probe) = last_audio(&h);
    assert_eq!(audio_probe, Some((before + 1, 0)), "probe tick's closeout packet");
    assert_eq!(actor(&h, 1)[0], x_settled, "probe dispatch applied no input edges");

    // render-only frames advance nothing.
    for _ in 0..3 {
        h.frame(None).expect("render-only frame");
    }
    assert_eq!(tick_of(&h), before + 1, "render-only frames advance nothing");
    assert_eq!(actor(&h, 1)[0], x_settled);

    // single Step = one neutral tick, packet accepted SuppressedByStep.
    {
        let p = h.play.as_mut().unwrap();
        p.single_step().expect("single step");
    }
    settle_host(&mut h);
    assert_eq!(tick_of(&h), before + 2, "single step = one tick");
    let (_, disp_step) = last_audio(&h);
    assert_eq!(disp_step, Some(Accepted::SuppressedByStep), "step packet suppressed");
    assert_eq!(actor(&h, 1)[0], x_settled, "step input is neutral");

    // Resume xN never re-Starts: walk to the first collect (each step_play
    // is one more Resume x dispatch cycle) — no fault 31, binds live, the
    // FIRST ordinal of the Run is 1.
    let (rows1, disp1) = walk_until_cue(&mut h, "v04c", 300);
    assert_eq!(rows1, vec![(1, 0, 0.5)], "Start effects intact exactly once: collect fires, ordinal 1");
    assert_eq!(disp1, Accepted::Played);
    assert_eq!(slot_text(&h, 0), "1");
    assert!(play_error(&h).is_empty());
    // 3 steps (still neutral): suppressed, consume no ordinals...
    for _ in 0..3 {
        {
            let p = h.play.as_mut().unwrap();
            p.single_step().expect("step");
        }
        settle_host(&mut h);
        assert_eq!(last_audio(&h).1, Some(Accepted::SuppressedByStep));
    }
    // ...and Resume never replays: the next carrying tick is coin 2 at
    // ordinal 2, Played — the step-consumed ticks left no backlog.
    let (rows2, disp2) = walk_until_cue(&mut h, "v04c", 400);
    assert_eq!(rows2, vec![(2, 0, 0.5)], "next ordinal after steps is 2 (no replay/backlog)");
    assert_eq!(disp2, Accepted::Played);
    h.stop_play();
    assert!(h.play.is_none(), "stop tears the run down");

    // Delivered suppression mechanism on the GENERATED cue table: a
    // cue-bearing packet accepted with step=true (single Step semantics).
    let dir = stage_pristine("v04c-mixer");
    let (doc, registry, _code) = load(&dir);
    let cues = public_audio::resolve_cues(&doc, &registry).unwrap();
    assert_eq!(cues.len(), 3);
    let mut m = Mixer::default();
    m.begin(7).unwrap();
    let step_packet = Packet {
        run: 7,
        tick: 3,
        bundle: 0,
        requests: vec![Request { ordinal: 1, cue: 0, gain: 0.5 }],
    };
    assert_eq!(
        m.accept(step_packet.clone(), &cues, true, || Ok(())).unwrap(),
        Accepted::SuppressedByStep,
        "step-consumed cue ordinal: suppressed, no voice"
    );
    assert_eq!(m.voices(), 0, "no voice inserted for a step tick");
    m.pause(7, true).unwrap();
    let mut out = vec![[0.0f32; 2]; 512];
    m.render(&mut out, 44100);
    assert!(out.iter().all(|f| *f == [0.0; 2]), "pause renders silence");
    m.pause(7, false).unwrap();
    m.render(&mut out, 44100);
    assert!(out.iter().all(|f| *f == [0.0; 2]), "resume never replays the suppressed cue");
    // the step-consumed ordinal is spent: a later live packet at ordinal 1
    // is rejected; the next live ordinal 2 plays.
    let live_stale = Packet { run: 7, tick: 4, bundle: 0, requests: step_packet.requests.clone() };
    assert!(m.accept(live_stale, &cues, false, || Ok(()))
        .is_err(), "step-consumed ordinal cannot be reused");
    let live = Packet {
        run: 7,
        tick: 4,
        bundle: 0,
        requests: vec![Request { ordinal: 2, cue: 0, gain: 0.5 }],
    };
    assert_eq!(m.accept(live, &cues, false, || Ok(())).unwrap(), Accepted::Played);
    println!("V04_C lifecycle: start-once proven across ~85 Resume cycles (collect ordinal 1, no fault 31), probe-dispatch/render-only/step exact, step packet SuppressedByStep, no replay, generated-cue mixer suppression proven");
}

// ============================================================ V-04 (d) ====

/// V-04 (d): 20 host-driven stop/fresh rounds fully rebuild the model.
/// Every round starts count "0"/"Playing" with the character at spawn; the
/// audio service is rebuilt for the Run (mixer.begin rejects stale/exhausted
/// Runs, old voices cannot survive); round 0 mutates (walk to count 2,
/// ordinals 1..2, coins 1+2 consumed); every fresh round walks to its FIRST
/// collect and it carries ordinal 1 again with disposition Played and the
/// packet's Run identity equal to the live Run (no old-Run cue leakage, no
/// ordinal carry-over); the replayed document hashes identically every
/// round; the shared legal resources (bank + font assets, slot templates)
/// stay alive: the last round still renders the full palette.
#[test]
fn v04_d_gpu_20_stop_fresh_rounds_rebuild() {
    let dir = stage_pristine("v04d");
    let (doc, registry, code) = load(&dir);
    let mut h = Host::with_project(doc, registry, Some(code)).expect("host");
    let mut doc_hash = String::new();
    for round in 0..20 {
        h.stop_play();
        h.start_selected_play()
            .unwrap_or_else(|e| panic!("round {round} start: {e:?}"));
        h.pause_play("");
        settle_host(&mut h);
        assert!(play_error(&h).is_empty(), "round {round} fault: {}", play_error(&h));
        let p = h.play.as_ref().unwrap();
        // audio service rebuilt for THIS Run; no old-Run voices survive.
        {
            let m = h.audio.as_ref().expect("audio manager").mixer.lock().unwrap();
            assert_eq!(m.run(), p.run_id, "round {round}: mixer Run rebuilt");
            assert_eq!(m.voices(), 0, "round {round}: no old-Run voices");
            assert!(m.fault().is_none());
        }
        // fresh model: count 0, Playing, character at spawn, document replayed.
        assert_eq!(slot_text(&h, 0), "0", "round {round}: count rebuilt");
        assert_eq!(slot_text(&h, 1), "Playing", "round {round}: state rebuilt");
        let a = actor(&h, 1);
        assert!((a[0] - 2.0).abs() < 1e-2, "round {round}: player respawned: {a:?}");
        let hash = p
            .initial_document_hash
            .clone()
            .expect("round {round} document hash");
        if round == 0 {
            doc_hash = hash;
        } else {
            assert_eq!(hash, doc_hash, "round {round}: document unchanged across runs");
        }
        let run_id = h.play.as_ref().unwrap().run_id();
        if round == 0 {
            // mutate: walk to count 2 (ordinals 1..2, coins 1+2 consumed).
            let (r1, d1) = walk_until_cue(&mut h, "v04d-r0", 300);
            assert_eq!(r1, vec![(1, 0, 0.5)]);
            assert_eq!(d1, Accepted::Played);
            let (r2, d2) = walk_until_cue(&mut h, "v04d-r0", 400);
            assert_eq!(r2, vec![(2, 0, 0.5)]);
            assert_eq!(d2, Accepted::Played);
            assert_eq!(slot_text(&h, 0), "2", "round 0 mutated to count 2");
        } else {
            // fresh Run: the FIRST collect restarts the ordinals at 1.
            let (rows, disp) = walk_until_cue(&mut h, &format!("v04d-r{round}"), 300);
            assert_eq!(rows, vec![(1, 0, 0.5)], "round {round}: ordinals restart per Run");
            assert_eq!(disp, Accepted::Played, "round {round}: disposition Played");
            assert_eq!(slot_text(&h, 0), "1", "round {round}: collect latched");
            // the carrying packet belongs to THIS Run.
            let f = h.play.as_ref().unwrap().last.as_ref().unwrap();
            assert_eq!(f.audio.as_ref().unwrap().run, run_id, "round {round}: packet Run identity");
        }
    }
    // shared legal resources alive after 20 rounds: one more fresh Run (the
    // 21st start) rests the character at spawn and the full palette renders.
    h.stop_play();
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("post-rounds start: {e:?}"));
    h.pause_play("");
    settle_host(&mut h);
    assert!(play_error(&h).is_empty(), "post-rounds fault");
    assert_eq!(slot_text(&h, 0), "0", "post-rounds count rebuilt");
    for _ in 0..26 {
        let _ = step_play(&mut h, 0);
    }
    let cap = pixel_classes(&mut h);
    assert!(in_box(&cap, 150.0, 176.0, 166.0, 196.0, is_body).len() >= 3, "body renders");
    assert!(in_box(&cap, 293.0, 317.0, 149.0, 175.0, is_gold).len() >= 3, "coin 1 renders after 20 rounds");
    assert!(in_box(&cap, 388.0, 412.0, 149.0, 175.0, is_gold).len() >= 3, "coin 2 renders");
    assert!(in_box(&cap, 483.0, 507.0, 149.0, 175.0, is_gold).len() >= 3, "coin 3 renders");
    assert!(in_box(&cap, 554.0, 580.0, 166.0, 192.0, is_gold).len() >= 3, "pad renders");
    assert!(in_box(&cap, 395.0, 405.0, 195.0, 210.0, is_ground).len() >= 3, "ground renders");
    assert!(in_box(&cap, 300.0, 500.0, 315.0, 350.0, is_pool).len() >= 3, "pool renders");
    let left = in_box(&cap, 110.0, 240.0, 2.0, 58.0, is_white);
    let right = in_box(&cap, 520.0, 686.0, 2.0, 58.0, is_white);
    assert!(left.len() >= 3 && right.len() >= 3, "HUD glyphs render after 20 rounds");
    h.stop_play();
    println!("V04_D 20 stop/fresh rounds: model rebuilt (count 0, ordinals restart at 1, coins back, dispositions Played, packet Run identity per round, document hash constant, resources alive)");
}

// ====================================================== fault: preaccept ====

/// Preaccept on the generated chain: a derived variant whose collect emits
/// the UNKNOWN cue 3 (the generated scene declares exactly cues 0..2). The
/// closeout's own audio unit rejects it with the NAMED cue-range fault 53
/// before any packet reaches the mixer: the play run surfaces the named
/// fault, the last-good visual/HUD is preserved (previous tick's frame,
/// count still "0", no new voice), and the exact restore (the pristine
/// project loaded fresh) runs green immediately after — its first collect is
/// ordinal 1, Played, HUD "1".
#[test]
fn v04_e_preaccept_unknown_cue_last_good_then_restore() {
    // the pristine control ticks green past the first collect first.
    let dir = stage_variant("v04e-preaccept", &unknown_cue_world());
    let (doc, registry, code) = load(&dir);
    let mut h = Host::with_project(doc, registry, Some(code)).expect("host");
    host_start(&mut h, "v04e");
    // walk right toward coin 1; the collect tick's closeout carries cue 3.
    let mut faulted_at = 0;
    for _ in 0..300 {
        let _ = step_play(&mut h, 1 << 1);
        let err = play_error(&h);
        if !err.is_empty() {
            faulted_at = last_frame_tick(&h);
            println!("V04_E preaccept fault: {err}");
            assert!(err.contains("53"), "named cue-range fault 53, got: {err}");
            break;
        }
    }
    assert!(faulted_at > 0, "the unknown-cue collect must fault");
    // last-good visual/HUD preserved: the previous tick's frame, count 0.
    assert_eq!(slot_text(&h, 0), "0", "last-good HUD preserved (count never showed 1)");
    assert_eq!(slot_text(&h, 1), "Playing", "last-good HUD preserved");
    let a = actor(&h, 1);
    assert!(a[0] < 7.0, "last-good position is pre-collect: {a:?}");
    assert_eq!(last_audio(&h).1, Some(Accepted::Played), "last good frame was its own accepted tick");
    // no new voice: the mixer never received the bad packet and the device
    // stream itself stayed healthy (the fault is the world's cue identity).
    {
        let m = h.audio.as_ref().expect("audio").mixer.lock().unwrap();
        assert!(m.fault().is_none(), "device stream healthy");
    }
    h.stop_play();
    let voices_after_stop = h.audio.as_ref().map(|a| a.mixer.lock().unwrap().voices());
    assert!(voices_after_stop.unwrap_or(0) == 0, "no voice survived");

    // exact restore: the pristine project runs green right after.
    let mut h2 = host_for("v04e-restore");
    let (rows, disp) = walk_until_cue(&mut h2, "v04e-restore", 300);
    assert_eq!(rows, vec![(1, 0, 0.5)], "restored first collect: ordinal 1, known cue");
    assert_eq!(disp, Accepted::Played);
    assert_eq!(slot_text(&h2, 0), "1");
    h2.stop_play();
    println!("V04_E preaccept: unknown cue -> named fault 53 at tick {faulted_at}, last-good HUD '0'/'Playing' preserved, no new voice, exact restore green (ordinal 1 Played)");
}

// ===================================================== fault: device Fault ====

/// Postaccept device Fault via the DELIVERED injection mechanism (the native
/// stream error-callback path — Mixer::output_fault, the exact hook the
/// cpal error callback drives; a real device fault cannot be induced safely
/// in this environment). The generated game collects coin 1, the Run keeps
/// playing so the coin cue's PCM is really emitted (nonzero device frames),
/// then the fault is injected mid-voice: the named "audio_fault: ..." latches
/// on the play run, the Run voices stop (the frozen/live coin voice is gone),
/// the mixer renders silence from then on (the already-emitted PCM is
/// irrevocable — the device consumed it; nothing is replayed — the nonzero
/// count stops growing), and the next start releases the failed stream,
/// opens a replacement and runs green (first collect ordinal 1, Played).
#[test]
fn v04_f_postaccept_device_fault_named_voices_stop_no_replay() {
    let mut h = host_for("v04f");
    let (rows, disp) = walk_until_cue(&mut h, "v04f", 300);
    assert_eq!(rows, vec![(1, 0, 0.5)]);
    assert_eq!(disp, Accepted::Played, "coin voice accepted (postaccept)");
    // resume the Run (held state cleared by pause) so the coin voice really
    // renders; wait for real nonzero device frames.
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
    }
    let (mixer, observation) = {
        let a = h.audio.as_ref().expect("audio manager");
        (a.mixer.clone(), a.observation.clone())
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let n = observation.lock().unwrap().nonzero;
        if n >= 64 {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "device never received the coin PCM");
        h.frame(None).expect("frame while playing");
    }
    {
        let m = mixer.lock().unwrap();
        assert_eq!(m.voices(), 1, "coin voice live mid-render");
    }
    let nonzero_before = observation.lock().unwrap().nonzero;
    // inject the delivered device-fault mechanism.
    mixer
        .lock()
        .unwrap()
        .output_fault("t05 injected device fault".into());
    {
        let mut m = mixer.lock().unwrap();
        assert_eq!(
            m.fault(),
            Some("audio_fault: t05 injected device fault"),
            "named audio Fault latched"
        );
        assert_eq!(m.voices(), 0, "Run voices stopped");
        let mut silence = vec![[1.0f32; 2]; 256];
        m.render(&mut silence, 44100);
        assert!(silence.iter().all(|f| *f == [0.0; 2]), "faulted mixer renders silence");
    }
    // the play run surfaces the named fault on its next pump.
    let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let err = play_error(&h);
        if !err.is_empty() {
            assert!(err.contains("audio_fault"), "named fault on the run: {err}");
            println!("V04_F play run error: {err}");
            break;
        }
        assert!(std::time::Instant::now() < end, "fault surface deadline");
        h.frame(None).expect("frame");
    }
    // no replay: the device receives silence from now on (already-emitted
    // PCM was irrevocable; the remaining voice was discarded, not muted).
    h.frame(None).expect("frame");
    std::thread::sleep(std::time::Duration::from_millis(120));
    h.frame(None).expect("frame");
    let nonzero_after = observation.lock().unwrap().nonzero;
    assert_eq!(
        nonzero_after, nonzero_before,
        "no PCM after the fault (no replay of the discarded voice)"
    );
    h.stop_play();
    // restore: the next start releases the failed stream and opens a
    // replacement; the fresh Run is green and its first collect is Played.
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("fresh start after device fault: {e:?}"));
    h.pause_play("");
    settle_host(&mut h);
    assert!(play_error(&h).is_empty(), "fresh run after the device fault");
    {
        let m = h.audio.as_ref().unwrap().mixer.lock().unwrap();
        assert!(m.fault().is_none(), "replacement stream healthy");
        assert_eq!(m.run(), h.play.as_ref().unwrap().run_id());
    }
    let (rows2, disp2) = walk_until_cue(&mut h, "v04f-restore", 300);
    assert_eq!(rows2, vec![(1, 0, 0.5)], "fresh Run ordinals restart");
    assert_eq!(disp2, Accepted::Played);
    h.stop_play();
    println!("V04_F postaccept device Fault: named audio_fault latched, Run voices stopped, silence after (no replay), replacement stream green with ordinals restarting at 1");
}

// ================================================== native re-observation ====

/// Native re-observation on the generated chain: the CPU session drives the
/// GENERATED world's callbacks and captures the closeout rows at their real
/// tick identities; the REAL default output device is opened and consumes
/// the captured packet exactly as the closeout delivered it (Run/tick/bundle
/// identity, ordinal 1, cue 0, request gain 0.5). Asserted: nonzero
/// submitted frames, ring match at head for one cue (the coin slice
/// resampled at the device rate x 0.5, tolerance 1e-6 on F32), 0 deadline
/// misses.
/// STANDING NAMED GAP (Plan D-04, verbatim): the observed device is the
/// ToDesk remote-desktop VIRTUAL speaker — the observation covers samples
/// submitted into the opened stream (post-routing, post-quantization), not
/// acoustic output; the acoustic/physical-device dimension remains a named
/// gap and no hardware output claim is made.
#[test]
fn v04_g_native_stream_observation_generated_chain() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "generated world pinned");
    // CPU session over the generated world: settle, walk, capture the first
    // carrying closeout (ordinal 1, cue 0, gain 0.5) at its tick identity.
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("t05g-native.at"),
        WORLD.as_bytes().to_vec(),
        Api::Hud2V1,
    )
    .expect("admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let rows7: [(i32, i32, f32, f32, f32, f32); 7] = [
        (1, 1, 2.0, 1.0, 0.8, 1.0),
        (2, 1, 8.0, -0.5, 24.0, 1.0),
        (3, 0, -30.0, -40.0, 0.5, 0.5),
        (4, 0, -30.0, -42.0, 0.5, 0.5),
        (5, 0, -30.0, -44.0, 0.5, 0.5),
        (6, 1, -30.0, -40.0, 2.0, 1.0),
        (7, 1, -30.0, -42.0, 40.0, 2.0),
    ];
    s.call("_ag_spatial_reset", &[Int(7)], ResultKind::Unit).unwrap();
    for (n, (id, kind, tx, ty, sx, sy)) in rows7.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(*id), Int(*kind), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [*tx, *ty, 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, -sx / 2., -sy / 2., 0., sx / 2., sy / 2., 0.]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..7 {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit).unwrap();
        }
    }
    s.call("_ag_audio_config", &[Int(3)], ResultKind::Unit).unwrap();
    for slot in 0..4 {
        s.call(
            "_ag_hud_config",
            &[Int(slot), Int(1), Int(0)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "app_start green");
    let mut t = 0;
    loop {
        t += 1;
        s.call("_ag_char_input", &[Int(0), Int(0)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_tick", &[Int(t), Float(DT)], ResultKind::Unit).unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "settle tick {t}");
        if s.int("get_player_grounded", &[]).unwrap() == 1 {
            break;
        }
    }
    let mut captured = None;
    while t < 1000 {
        t += 1;
        s.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_tick", &[Int(t), Float(DT)], ResultKind::Unit).unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "walk tick {t}");
        let n = s.int("_ag_audio_count", &[]).unwrap();
        if n > 0 {
            let rows: Vec<(i32, i32, f32)> = (0..n)
                .map(|i| {
                    (
                        s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap(),
                        s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap(),
                        s.float("_ag_audio_g", &[Int(i)]).unwrap(),
                    )
                })
                .collect();
            assert_eq!(rows, vec![(1, 0, 0.5)], "first carrying closeout of the generated chain");
            captured = Some(t);
            break;
        }
    }
    let carry_tick = captured.expect("the generated chain emitted its first cue");
    drop(s);

    // the generated cue table + the real default device.
    let dir = stage_pristine("v04g-native");
    let (doc, registry, _code) = load(&dir);
    let cues = public_audio::resolve_cues(&doc, &registry).unwrap();
    assert_eq!(cues.len(), 3);
    assert_eq!((cues[0].start, cues[0].end), COIN_SPAN);
    let native = match public_audio::native::Native::open() {
        Ok(n) => n,
        Err(e) => panic!("native device required for this gate: {e:?}"),
    };
    println!(
        "V04_G device='{}' rate={} channels={} format={} (VIRTUAL remote-desktop speaker — standing named gap D-04: observation covers submitted samples, not acoustic output)",
        native.device, native.rate, native.channels, native.sample_format
    );
    let frames_before = native.observation.lock().unwrap().frames;
    {
        let mut m = native.mixer.lock().unwrap();
        m.begin(1).unwrap();
        let packet = Packet {
            run: 1,
            tick: carry_tick,
            bundle: 0,
            requests: vec![Request { ordinal: 1, cue: 0, gain: 0.5 }],
        };
        assert_eq!(
            m.accept(packet, &cues, false, || Ok(())).unwrap(),
            Accepted::Played,
            "the generated closeout packet accepted by the real device stream"
        );
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if native.observation.lock().unwrap().nonzero >= 8 {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "native PCM timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let report = {
        let o = native.observation.lock().unwrap();
        let written = (o.frames - frames_before) as usize;
        let trace = public_audio::native::TRACE_FRAMES;
        let m_frames = written.min(1024);
        let rate = native.rate;
        let mono = native.channels == 1;
        let len = cues[0].end - cues[0].start;
        // Mirror mixer.render cursor arithmetic at the device rate exactly,
        // with the closeout's request gain 0.5 x declared cue gain 1.0.
        let mut expected = Vec::with_capacity(m_frames);
        let mut cursor = 0.0f64;
        for _ in 0..m_frames {
            let position = (cursor.floor() as usize).min(len - 1);
            let fraction = (cursor - position as f64) as f32;
            let a = cues[0].pcm.frames[cues[0].start + position];
            let b = cues[0].pcm.frames[cues[0].start + (position + 1).min(len - 1)];
            let mut e = [
                (a[0] * (1.0 - fraction) + b[0] * fraction) * 0.5,
                (a[1] * (1.0 - fraction) + b[1] * fraction) * 0.5,
            ];
            if mono {
                let mono_value = (e[0] + e[1]) * 0.5;
                e = [mono_value, mono_value];
            }
            expected.push(e);
            cursor += f64::from(44100u32) / f64::from(rate);
        }
        let tolerance = if native.sample_format == "F32" { 1.0e-6 } else { 2.0 / 32768.0 };
        let head_start = (o.cursor + trace - written.min(trace)) % trace;
        let tail_start = (o.cursor + trace - m_frames) % trace;
        let windows = if written <= trace {
            vec![("head", head_start)]
        } else {
            vec![("head", head_start), ("tail", tail_start)]
        };
        let mut matched = None;
        for (label, start) in windows {
            let ok = (0..m_frames).all(|i| {
                let ring = o.ring[(start + i) % trace];
                (ring[0] - expected[i][0]).abs() <= tolerance
                    && (ring[1] - expected[i][1]).abs() <= tolerance
            });
            if ok {
                matched = Some((label, start, m_frames));
                break;
            }
        }
        (
            written,
            (
                o.callbacks,
                o.frames,
                o.nonzero,
                o.peak,
                o.max_us,
                o.deadline_misses,
                o.last_quantum,
            ),
            matched,
        )
    };
    native.mixer.lock().unwrap().stop();
    let (written, stats, matched) = report;
    println!(
        "V04_G native: written_frames={written} callbacks={} total_frames={} nonzero={} peak={:.6} max_us={} deadline_misses={} quantum={} ring_match={matched:?} carry_tick={carry_tick}",
        stats.0, stats.1, stats.2, stats.3, stats.4, stats.5, stats.6
    );
    assert!(stats.2 > 0, "nonzero submitted frames");
    assert_eq!(stats.5, 0, "0 deadline misses");
    assert!(matched.is_some(), "device ring must contain the generated chain's coin cue at head");
}
