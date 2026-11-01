//! auto-ui PLAN-023 T-04 real-media evidence on the GENERATED
//! components-2d-game chain (world sha ed98b0b5...):
//!   PCM: the audio service output captured while driving the generated
//!   world's callbacks — each closeout bundle row (ordinal/cue/gain read
//!   back through the session) is fed as the EXACT Packet into the real
//!   mixer with the cues resolved from the generated scene.agscene; every
//!   rendered cue matches its bank slice x request_gain(0.5) x cue_gain(1.0)
//!   at tolerance <= 2e-5 (measured bit-exact, 0 LSB), win chain and fail
//!   chain both. The gain check: the fixture declares cue gain 1.0; the
//!   generated closeout carries request gain 0.5 — the rendered amplitude
//!   ratio is exactly 0.5.
//!   Glyphs: one offscreen GPU host session of the generated game — HUD
//!   text pixel classes appear fresh at count=0, change with count ("0" ->
//!   "1" -> "3") and state ("Playing" -> "Won"), sit at the VIEWPORT anchor
//!   positions (clip+offset; a world-space layout would sit ~180px further
//!   right at the projected anchor), stay pixel-fixed while the walk plays
//!   out (the follow window [0,24]x view 24 pins the camera, so the win
//!   walk manifests as the character's projection sweeping ~370 px across
//!   the viewport — measured, not assumed), and the character/coins/pad/
//!   ground/pool world objects render. The host play frames also carry
//!   every closeout packet with its real mixer disposition — the four
//!   carrying ticks (5 rows) were all Played: the service consumed the
//!   generated cues.
//! Project staging: the generated product's scene/world/assets are staged
//! under target/ (load_project resolves assets/<blob> beside the scene; the
//! committed game dir keeps assets in closure/assets).
//! Run (single instance, GPU + audio):
//!   CARGO_TERM_COLOR=never RUST_MIN_STACK=16777216 cargo test --locked \
//!     --offline --test t04_game_media -- --test-threads=1 --nocapture
use autographics_auto_driver::{
    authoring::{
        assets,
        behavior::{Api, SourceSnapshot},
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
const WORLD_SHA: &str = "ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0";
const BANK_SHA: &str = "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d";
const FONT_SHA: &str = "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa";
/// Mission tolerance: max abs sample delta <= 2e-5 normalized.
const TOLERANCE: f32 = 2.0e-5;
/// Frozen cue table of the generated game (cue id = declaration index).
const SPANS: [(usize, usize); 3] = [(0, 7403), (7403, 20046), (20046, 22504)];

const DT: f32 = 1.0 / 60.0;

// ------------------------------------------------------------- staging ----

/// Stage the generated game as a loadable project (scene + bound script +
/// assets/) under target/, exactly as load_project resolves it.
fn stage_game(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/t04-media")
        .join(format!("{}-{tag}", std::process::id()))
        .join("game");
    let scripts = dir.join("scripts");
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::create_dir_all(&assets_dir).expect("assets dir");
    std::fs::write(dir.join("scene.agscene"), SCENE).expect("write scene");
    std::fs::write(scripts.join(format!("{WORLD_SHA}.at")), WORLD).expect("write script");
    let closure = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CLOSURE_ASSETS);
    // load_project resolves assets/<sha256>.<ext> (blob_name), so the staged
    // copies use the content-addressed names; sources are the closure files.
    for (src, blob) in [
        ("b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav", format!("{BANK_SHA}.wav")),
        ("lilita_one_regular.ttf", format!("{FONT_SHA}.ttf")),
    ] {
        std::fs::copy(closure.join(src), assets_dir.join(&blob))
            .unwrap_or_else(|e| panic!("copy asset {src}: {e}"));
    }
    dir
}

fn load_project(tag: &str) -> (
    PathBuf,
    autographics_auto_driver::authoring::document::Document,
    assets::Registry,
    std::sync::Arc<autographics_auto_driver::authoring::behavior_project::CodePayload>,
) {
    let dir = stage_game(tag);
    let (d, r, code, _) = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
        .unwrap_or_else(|e| panic!("load staged game project: {e:?}"));
    let code = code.expect("game world bound as behavior code");
    (dir, d, r, code)
}

// ---------------------------------------------------------- bank oracle ----

/// Independent PCM16 RIFF reader over the GENERATED product's asset bytes
/// (loaded registry), exactly as the service decodes them.
fn bank_oracle(registry: &assets::Registry) -> (Vec<i16>, Vec<[f32; 2]>) {
    let asset = registry.values().find(|a| a.audio.is_some()).unwrap();
    assert_eq!(asset.identity.format, "wav");
    assert_eq!(asset.identity.sha256, BANK_SHA);
    assert_eq!(asset.identity.byte_length, 90060);
    let meta = asset.identity.audio.as_ref().unwrap();
    assert_eq!((meta.rate, meta.channels, meta.frames), (44100, 2, 22504));
    let bytes = &asset.bytes[..];
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[36..40], b"data");
    let data_bytes = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
    assert_eq!(data_bytes, 90016);
    let raw: Vec<i16> = bytes[44..]
        .chunks_exact(2)
        .map(|p| i16::from_le_bytes([p[0], p[1]]))
        .collect();
    assert_eq!(raw.len(), 45008, "22504 frames x 2 channels");
    let frames: Vec<[f32; 2]> = raw
        .chunks_exact(2)
        .map(|p| [f32::from(p[0]) / 32768.0, f32::from(p[1]) / 32768.0])
        .collect();
    (raw, frames)
}

fn max_abs_delta(actual: &[[f32; 2]], expected: &[[f32; 2]]) -> f32 {
    actual
        .iter()
        .zip(expected)
        .map(|(a, e)| (a[0] - e[0]).abs().max((a[1] - e[1]).abs()))
        .fold(0.0f32, f32::max)
}

/// Max abs delta in PCM16 LSB between the rendered output (re-quantized by
/// round(x*32768)) and the raw bank samples scaled by `scale`.
fn max_abs_delta_i16(actual: &[[f32; 2]], raw: &[i16], start_frame: usize, scale: f32) -> i64 {
    actual
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let base = (start_frame + i) * 2;
            let l = i64::from((a[0] * 32768.0).round() as i32)
                - i64::from((f32::from(raw[base]) * scale).round() as i32);
            let r = i64::from((a[1] * 32768.0).round() as i32)
                - i64::from((f32::from(raw[base + 1]) * scale).round() as i32);
            l.abs().max(r.abs())
        })
        .max()
        .unwrap()
}

fn peak_of(out: &[[f32; 2]]) -> f32 {
    out.iter()
        .map(|f| f[0].abs().max(f[1].abs()))
        .fold(0.0f32, f32::max)
}

/// Render one cue alone at `gain` in a fresh mixer (the t02 capture shape).
fn render_one(cues: &[public_audio::mixer::Cue], index: usize, gain: f32) -> Vec<[f32; 2]> {
    let len = cues[index].end - cues[index].start;
    let mut m = Mixer::default();
    m.begin(1).unwrap();
    let packet = Packet {
        run: 1,
        tick: 0,
        bundle: 0,
        requests: vec![Request {
            ordinal: 1,
            cue: index,
            gain,
        }],
    };
    assert_eq!(
        m.accept(packet, cues, false, || Ok(())).unwrap(),
        Accepted::Played
    );
    let mut out = vec![[0.0; 2]; len];
    m.render(&mut out, 44100);
    assert_eq!(m.voices(), 0, "voice ends exactly at the half-open span");
    out
}

// ------------------------------------------------------- CPU session ------

fn session(world_src: &str, name: &str, run: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Hud2V1,
    )
    .unwrap_or_else(|e| panic!("{name} admission: {e}"));
    let mut s = Session::new(&snap.source).unwrap_or_else(|e| panic!("{name} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    // Scene rows of the generated game (7 rows, translation-relative boxes).
    let scene = 7;
    s.call("_ag_spatial_reset", &[Int(scene)], ResultKind::Unit).unwrap();
    for (n, (id, kind, tx, ty, sx, sy)) in [
        (1, 1, 2.0, 1.0, 0.8, 1.0),      // player.body
        (2, 1, 8.0, -0.5, 24.0, 1.0),    // ground
        (3, 0, -30.0, -40.0, 0.5, 0.5),  // c1 template
        (4, 0, -30.0, -42.0, 0.5, 0.5),  // c2 template
        (5, 0, -30.0, -44.0, 0.5, 0.5),  // c3 template
        (6, 1, -30.0, -40.0, 2.0, 1.0),  // pad template
        (7, 1, -30.0, -42.0, 40.0, 2.0), // hazard template
    ]
    .iter()
    .enumerate()
    {
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
    for n in 0..scene {
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
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "{name} app_start fault {} arg {}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

fn tick(s: &mut Session, step: i32, held: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

/// This tick's closeout bundle rows (ordinal, cue, request gain) as the
/// generated ag_present2d delivered them.
fn audio_rows(s: &mut Session) -> Vec<(i32, i32, f32)> {
    let n = s.int("_ag_audio_count", &[]).unwrap();
    (0..n)
        .map(|i| {
            (
                s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap(),
                s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap(),
                s.float("_ag_audio_g", &[Int(i)]).unwrap(),
            )
        })
        .collect()
}

fn settle(s: &mut Session, name: &str) -> i32 {
    for t in 1..=300 {
        let st = tick(s, t, 0);
        assert_eq!(st, 0, "{name} settle tick {t} status {st}");
        if int(s, "get_player_grounded") == 1 {
            return t;
        }
    }
    panic!("{name}: never settled");
}

/// Drive the win path (walk right from settle until state Won) and capture
/// every emitted closeout bundle with its real tick identity.
fn capture_win_path(s: &mut Session, name: &str) -> (i32, Vec<(i32, Vec<(i32, i32, f32)>)>) {
    let t0 = settle(s, name);
    let mut t = t0;
    let mut captures = Vec::new();
    let mut win = 0;
    while t < t0 + 1200 {
        t += 1;
        let st = tick(s, t, 2);
        assert_eq!(st, 0, "{name} tick {t} over the pool");
        let rows = audio_rows(s);
        if !rows.is_empty() {
            captures.push((t, rows));
        }
        if int(s, "get_r_state") == 1 {
            win = t;
            break;
        }
    }
    assert!(win > 0, "{name}: pad never won");
    (t0, captures)
}

// -------------------------------------------------------------- PCM ----

/// Real PCM through the generated chain: the session-captured closeout
/// rows (with the closeout's request gain 0.5) are fed as exact Packets at
/// their real tick identities into the mixer built from the GENERATED
/// scene's cue table; rendered PCM must equal bank slice x 0.5 bit-exact.
#[test]
fn t04_pcm_generated_chain_matches_bank_slices() {
    let (_dir, doc, registry, _code) = load_project("pcm");
    // The generated scene declares the frozen cue table (id = index).
    let declared = doc.cues.as_deref().expect("scene cues");
    assert_eq!(declared.len(), 3);
    for (i, (name, span)) in [("coin", SPANS[0]), ("win", SPANS[1]), ("fail", SPANS[2])]
        .into_iter()
        .enumerate()
    {
        assert_eq!(declared[i].name, name);
        assert_eq!(
            (declared[i].start_frame as usize, declared[i].end_frame as usize),
            span
        );
        assert_eq!(declared[i].gain, 1.0, "fixture cue gain is 1.0");
    }
    let cues = public_audio::resolve_cues(&doc, &registry).unwrap();
    assert_eq!(cues.len(), 3);
    for (i, span) in SPANS.iter().enumerate() {
        assert_eq!((cues[i].start, cues[i].end), *span);
        assert_eq!(cues[i].gain, 1.0);
    }
    assert!(std::sync::Arc::ptr_eq(&cues[0].pcm, &cues[1].pcm));
    assert_eq!(cues[0].pcm.rate, 44100);
    assert_eq!(cues[0].pcm.frames.len(), 22504);
    let (raw, frames) = bank_oracle(&registry);

    // Drive the GENERATED world's callbacks: win path captures.
    let mut s = session(WORLD, "t04-pcm-win.at", 1);
    let (t0, win_caps) = capture_win_path(&mut s, "t04-pcm-win");
    let rows: Vec<(i32, i32, f32)> = win_caps.iter().flat_map(|(_, r)| r.iter().copied()).collect();
    assert_eq!(
        rows,
        vec![
            (1, 0, 0.5),
            (2, 0, 0.5),
            (3, 0, 0.5),
            (4, 1, 0.5),
            (5, 1, 0.5)
        ],
        "the generated closeout emits ordinals 1..5, cues coin/coin/coin+win/win, request gain 0.5"
    );
    // Fail path captures (cue 2 through the same chain).
    let mut sf = session(WORLD, "t04-pcm-fail.at", 1);
    let tf0 = settle(&mut sf, "t04-pcm-fail");
    let mut t = tf0;
    let mut fail_caps = Vec::new();
    while t < tf0 + 500 {
        t += 1;
        let st = tick(&mut sf, t, 1);
        assert_eq!(st, 0, "fail tick {t}");
        let r = audio_rows(&mut sf);
        if !r.is_empty() {
            fail_caps.push((t, r));
            break;
        }
    }
    assert_eq!(
        fail_caps,
        vec![(fail_caps[0].0, vec![(1, 2, 0.5)])],
        "fail buzz: ordinal 1 cue 2 gain 0.5"
    );

    // Per-cue isolation at the captured request gain: slice x 0.5, bit-exact.
    for (cue, gain) in [(0usize, 0.5f32), (1, 0.5), (2, 0.5)] {
        let rendered = render_one(&cues, cue, gain);
        let start = SPANS[cue].0;
        let expected: Vec<[f32; 2]> = frames[start..SPANS[cue].1]
            .iter()
            .map(|f| [f[0] * gain, f[1] * gain])
            .collect();
        let delta = max_abs_delta(&rendered, &expected);
        let lsb = max_abs_delta_i16(&rendered, &raw, start, gain);
        println!(
            "T04_PCM cue={cue} gain={gain} frames={} max_abs_f32_delta={delta:.3e} max_abs_i16_lsb={lsb} peak={:.6} tol_ok={}",
            rendered.len(),
            peak_of(&rendered),
            delta <= TOLERANCE
        );
        assert!(delta <= TOLERANCE, "cue {cue} delta {delta}");
        assert_eq!(lsb, 0, "cue {cue} must be bit-exact in PCM16");
    }

    // The real win chain: all five rows accepted at their captured ticks in
    // ONE mixer of Run 1, then rendered together. Each voice renders its own
    // slice from output frame 0, so the mix is the sample-wise sum of the
    // scaled slices.
    let mut m = Mixer::default();
    m.begin(1).unwrap();
    for (tick_id, rows) in &win_caps {
        let packet = Packet {
            run: 1,
            tick: *tick_id,
            bundle: 0,
            requests: rows
                .iter()
                .map(|(o, c, g)| Request {
                    ordinal: *o,
                    cue: *c as usize,
                    gain: *g,
                })
                .collect(),
        };
        assert_eq!(
            m.accept(packet, &cues, false, || Ok(())).unwrap(),
            Accepted::Played,
            "win chain packet at tick {tick_id}"
        );
    }
    let mut mixed = vec![[0.0f32; 2]; 22504];
    m.render(&mut mixed, 44100);
    assert_eq!(m.voices(), 0, "all voices finite (half-open spans)");
    let mut expected = vec![[0.0f32; 2]; 22504];
    for (_, rows) in &win_caps {
        for (_, c, g) in rows {
            for i in 0..(SPANS[*c as usize].1 - SPANS[*c as usize].0) {
                expected[i][0] += frames[SPANS[*c as usize].0 + i][0] * g;
                expected[i][1] += frames[SPANS[*c as usize].0 + i][1] * g;
            }
        }
    }
    let mix_delta = max_abs_delta(&mixed, &expected);
    println!(
        "T04_PCM win_chain ticks={:?} max_abs_f32_delta={mix_delta:.3e} peak={:.6} tol_ok={}",
        win_caps.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
        peak_of(&mixed),
        mix_delta <= TOLERANCE
    );
    assert!(mix_delta <= TOLERANCE, "win chain mix delta {mix_delta}");

    // The fail chain: one packet (ordinal 1, cue 2, gain 0.5) at its tick.
    let mut mf = Mixer::default();
    mf.begin(1).unwrap();
    let (ftick, frows) = &fail_caps[0];
    let fpacket = Packet {
        run: 1,
        tick: *ftick,
        bundle: 0,
        requests: frows
            .iter()
            .map(|(o, c, g)| Request {
                ordinal: *o,
                cue: *c as usize,
                gain: *g,
            })
            .collect(),
    };
    assert_eq!(
        mf.accept(fpacket, &cues, false, || Ok(())).unwrap(),
        Accepted::Played
    );
    let flen = SPANS[2].1 - SPANS[2].0;
    let mut fout = vec![[0.0f32; 2]; flen];
    mf.render(&mut fout, 44100);
    let fexpected: Vec<[f32; 2]> = frames[SPANS[2].0..SPANS[2].1]
        .iter()
        .map(|f| [f[0] * 0.5, f[1] * 0.5])
        .collect();
    let fdelta = max_abs_delta(&fout, &fexpected);
    let flsb = max_abs_delta_i16(&fout, &raw, SPANS[2].0, 0.5);
    println!(
        "T04_PCM fail_chain tick={ftick} max_abs_f32_delta={fdelta:.3e} max_abs_i16_lsb={flsb} tol_ok={}",
        fdelta <= TOLERANCE
    );
    assert!(fdelta <= TOLERANCE);
    assert_eq!(flsb, 0);

    // Gain semantics on the generated chain: rendered = slice x 0.5 exactly
    // (request gain 0.5 x declared cue gain 1.0); full gain differs.
    let full = render_one(&cues, 0, 1.0);
    let full_peak = peak_of(&full);
    let half_peak = peak_of(&render_one(&cues, 0, 0.5));
    let ratio = half_peak / full_peak;
    println!(
        "T04_PCM gain: cue_gain=1.0(request=0.5) full_peak={full_peak:.6} half_peak={half_peak:.6} ratio={ratio:.6}"
    );
    assert!((ratio - 0.5).abs() < 1.0e-6, "amplitude ratio must be 0.5");
    println!("T04_PCM captured ticks: settle={t0} win={:?}", win_caps.iter().map(|(t, _)| *t).collect::<Vec<_>>());
}

// -------------------------------------------------------- GPU session ----

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
    (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0)
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

fn in_box(px: &[Px], x0: f32, x1: f32, y0: f32, y1: f32, pred: fn([u8; 4]) -> bool) -> Vec<Px> {
    px.iter()
        .filter(|(p, c)| p[0] >= x0 && p[0] <= x1 && p[1] >= y0 && p[1] <= y1 && pred(*c))
        .copied()
        .collect()
}

fn centroid(px: &[Px]) -> (f32, f32) {
    let (mut sx, mut sy) = (0.0f32, 0.0f32);
    for (p, _) in px {
        sx += p[0];
        sy += p[1];
    }
    let n = px.len() as f32;
    (sx / n, sy / n)
}

/// Bounding box of a classified cluster: (min_x, max_x, min_y, max_y).
fn extent(px: &[Px]) -> (f32, f32, f32, f32) {
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for (p, _) in px {
        x0 = x0.min(p[0]);
        x1 = x1.max(p[0]);
        y0 = y0.min(p[1]);
        y1 = y1.max(p[1]);
    }
    (x0, x1, y0, y1)
}

fn sig(px: &[Px]) -> (usize, f32, f32) {
    let n = px.len();
    if n == 0 {
        return (0, 0.0, 0.0);
    }
    let (x, y) = centroid(px);
    (n, x, y)
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
fn step_play(h: &mut Host, held: u32) -> (Vec<(i32, usize, f32)>, Option<Accepted>) {
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

/// Real glyph evidence: HUD text renders as viewport-anchored glyph pixels
/// that change with count/state and stay fixed while the camera target pans;
/// world objects (character/coins/pad) render as before; every closeout
/// bundle the callbacks emit is accepted by the real audio service (Played).
#[test]
fn t04_gpu_game_hud_glyphs_viewport_overlay_and_world() {
    let (_dir, _doc, registry, code) = load_project("gpu");
    let mut h = Host::with_project(_doc, registry, Some(code)).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("start run: {e:?}"));
    h.pause_play("");
    settle_host(&mut h);
    assert!(h.play.as_ref().unwrap().error.clone().unwrap_or_default().is_empty());

    // Settled: character on the ground at rest (T=26 discrete oracle).
    for _ in 0..26 {
        let _ = step_play(&mut h, 0);
    }
    let a0 = actor(&h, 1);
    assert!((a0[0] - 2.0).abs() < 1e-2, "rest x: {a0:?}");
    assert!((a0[1] - 0.502).abs() < 1e-2, "landed at rest: {a0:?}");
    assert_eq!(slot_text(&h, 0), "0", "count HUD at rest");
    assert_eq!(slot_text(&h, 1), "Playing", "state HUD at rest");

    // Capture 1 (count=0): the fresh HUD glyph classes + the world objects.
    let cap0 = pixel_classes(&mut h);
    let left0 = in_box(&cap0, 110.0, 240.0, 2.0, 58.0, is_white);
    let right0 = in_box(&cap0, 520.0, 686.0, 2.0, 58.0, is_white);
    let (l0n, l0x, l0y) = sig(&left0);
    let (r0n, r0x, r0y) = sig(&right0);
    assert!(l0n >= 3, "count glyph pixels at top-left: {l0n}");
    assert!(r0n >= 6, "state glyph pixels at top-right: {r0n}");
    // Viewport-anchored, NOT world-transformed: the HUD anchor math puts
    // slot 0 at clip.x+offset = 115+8 (world-space (8,8) would project to
    // x~305), slot 1 right-aligned to clip right - offset (~685-8).
    assert!(l0x > 110.0 && l0x < 200.0, "left glyph cluster at viewport anchor: {l0x}");
    assert!(r0x > 540.0 && r0x < 686.0, "right glyph cluster right-aligned: {r0x}");
    assert!(l0y < 60.0 && r0y < 60.0, "glyphs in the top HUD band: {l0y} {r0y}");
    // No white anywhere else: the HUD text is the only white in the frame.
    let white_below = in_box(&cap0, 0.0, 800.0, 58.0, 380.0, is_white);
    assert!(
        white_below.is_empty(),
        "HUD glyphs must not leak below the HUD band: {} samples",
        white_below.len()
    );
    // World objects visible as before (camera pinned by the follow window
    // [0,24]x[-8,8] at view 24x16 -> screen ~= (397.5 + (wx-12)*23.6,
    // 190 - wy*23.6); measured by the calibration probe).
    let player0 = in_box(&cap0, 150.0, 176.0, 166.0, 196.0, is_body);
    assert!(player0.len() >= 3, "character renders: {}", player0.len());
    let c1_0 = in_box(&cap0, 293.0, 317.0, 149.0, 175.0, is_gold);
    let c2_0 = in_box(&cap0, 388.0, 412.0, 149.0, 175.0, is_gold);
    let c3_0 = in_box(&cap0, 483.0, 507.0, 149.0, 175.0, is_gold);
    let pad0 = in_box(&cap0, 554.0, 580.0, 166.0, 192.0, is_gold);
    assert!(c1_0.len() >= 3, "coin 1 renders: {}", c1_0.len());
    assert!(c2_0.len() >= 3, "coin 2 renders: {}", c2_0.len());
    assert!(c3_0.len() >= 3, "coin 3 renders: {}", c3_0.len());
    assert!(pad0.len() >= 3, "pad renders: {}", pad0.len());
    let ground0 = in_box(&cap0, 395.0, 405.0, 195.0, 210.0, is_ground);
    assert!(ground0.len() >= 3, "ground renders: {}", ground0.len());
    let pool0 = in_box(&cap0, 300.0, 500.0, 315.0, 350.0, is_pool);
    assert!(pool0.len() >= 3, "hazard pool renders: {}", pool0.len());

    // Walk right in 5-tick chunks until the run is Won; record every
    // carrying tick's closeout rows + the real mixer disposition of that
    // bundle. Capture pixel classes at the first count change ("0" -> "1").
    let mut seen_rows: Vec<(i32, usize, f32)> = Vec::new();
    let mut carried: Vec<Accepted> = Vec::new();
    let mut cap1: Option<Vec<Px>> = None;
    let ax0 = a0[0];
    let mut chunks = 0;
    let (mut hud0, mut hud1) = (slot_text(&h, 0), slot_text(&h, 1));
    while hud1 != "Won" {
        chunks += 1;
        assert!(chunks <= 120, "walk to win never finished: {hud0}/{hud1}");
        for _ in 0..5 {
            let (rows, disp) = step_play(&mut h, 1 << 1);
            let carried_here = !rows.is_empty();
            seen_rows.extend(rows);
            if carried_here {
                carried.push(disp.expect("carrying tick has a disposition"));
            }
        }
        let (n0, n1) = (slot_text(&h, 0), slot_text(&h, 1));
        if n0 != hud0 {
            hud0 = n0;
            if hud0 == "1" && cap1.is_none() {
                cap1 = Some(pixel_classes(&mut h));
            }
        }
        hud1 = n1;
    }
    // The generated callbacks emitted exactly the five expected rows and the
    // real audio service accepted every bundle (Played, not suppressed). The
    // closeout emits a present2d every tick, so zero-row packets ride along;
    // only the four carrying ticks (5 rows) hit the mixer with requests.
    assert_eq!(
        seen_rows,
        vec![
            (1, 0, 0.5),
            (2, 0, 0.5),
            (3, 0, 0.5),
            (4, 1, 0.5),
            (5, 1, 0.5)
        ],
        "host closeout rows across the walk"
    );
    assert!(
        carried.iter().all(|d| *d == Accepted::Played),
        "every bundle accepted by the service: {carried:?}"
    );
    assert_eq!(carried.len(), 4, "four carrying ticks (5 rows)");
    assert_eq!(slot_text(&h, 0), "3");
    let a1 = actor(&h, 1);
    let pan_world = a1[0] - ax0;
    assert!(pan_world > 14.0, "camera target panned across the level: {pan_world}");

    // Capture 2 (Won): HUD changed with count AND state; coins consumed; the
    // character swept to the pad.
    let capw = pixel_classes(&mut h);
    let leftw = in_box(&capw, 110.0, 240.0, 2.0, 58.0, is_white);
    let rightw = in_box(&capw, 520.0, 686.0, 2.0, 58.0, is_white);
    let (lw, lwx, lwy) = sig(&leftw);
    let (rw, rwx, _rwy) = sig(&rightw);
    // The character's own screen sweep over the same two captures (the
    // follow window pins the camera, so the swept pixels ARE the walk).
    let bodyw = in_box(&capw, 500.0, 553.0, 160.0, 196.0, is_body);
    let (bwn, bwx, _bwy) = sig(&bodyw);
    let (_, bx0, _by0) = sig(&player0);
    let sweep_x = (bwx - bx0).abs();
    assert!(bwn >= 3, "character renders at the pad: {bwn}");
    assert!(sweep_x > 100.0, "character swept the viewport: {sweep_x}");
    assert!(lw >= 3 && rw >= 3, "HUD still rendered after the latch");
    assert!(
        (lw as i32 - l0n as i32).abs() > 2 || (lwx - l0x).abs() > 2.0,
        "count glyphs changed with count: {l0n}/{l0x} -> {lw}/{lwx}"
    );
    assert!(
        (rw as i32 - r0n as i32).abs() > 2 || (rwx - r0x).abs() > 2.0,
        "state glyphs changed with state: {r0n}/{r0x} -> {rw}/{rwx}"
    );
    // Coins 1..3 consumed along the walk (coin 3 window empty at Won; the
    // pad remains). Also measured above: the character's screen sweep.
    let c3_w = in_box(&capw, 483.0, 507.0, 149.0, 175.0, is_gold);
    assert!(c3_w.len() < 2, "coin 3 consumed: {}", c3_w.len());
    let pad_w = in_box(&capw, 554.0, 580.0, 166.0, 192.0, is_gold);
    assert!(pad_w.len() >= 3, "pad still renders: {}", pad_w.len());

    // HUD is viewport-fixed: across the whole win walk (the follow target
    // crossed pan_world world units; the character's projection swept
    // sweep_x px on screen), each cluster's anchored edge stayed within 2
    // samples (4 px) — slot 0 is left-anchored (its left edge is the
    // anchor), slot 1 right-anchored (its right edge is the anchor). The
    // right cluster's CENTROID moves 15 px because "Playing" -> "Won"
    // reflows a right-aligned run; the anchor edge does not move. A
    // character-anchored HUD would have swept with the body; a world-space
    // layout would sit at the projected anchor (~305 px), not at the
    // measured viewport anchor (clip.x + 8).
    let (lx0e, _, _, _) = extent(&left0);
    let (lxwe, _, _, _) = extent(&leftw);
    let (_, rx0e, _, _) = extent(&right0);
    let (_, rxwe, _, _) = extent(&rightw);
    let dlx = (lwx - l0x).abs();
    let dl_edge = (lxwe - lx0e).abs();
    let dr_edge = (rxwe - rx0e).abs();
    let dly = (lwy - l0y).abs();
    println!(
        "T04_GPU hud_fix: pan_world={pan_world:.3} body_sweep_px={sweep_x:.1} left_dx={dlx} left_edge_dx={dl_edge} right_edge_dx={dr_edge} right_centroid_dx={:.1} left_dy={dly}",
        (rwx - r0x).abs()
    );
    assert!(dlx <= 4.0, "left HUD fixed across the walk: {dlx}");
    assert!(dl_edge <= 4.0, "left HUD anchor edge fixed: {lx0e} -> {lxwe}");
    assert!(dr_edge <= 4.0, "right HUD anchor edge fixed: {rx0e} -> {rxwe}");
    assert!(dly <= 4.0, "left HUD vertically fixed: {dly}");

    // Capture 1 checks (taken right after the first collect): the count
    // glyphs changed, the state glyphs did not ("Playing" still), coin 1 is
    // consumed while coins 2/3 persist.
    let cap1 = cap1.expect("first-collect capture");
    let left1 = in_box(&cap1, 110.0, 240.0, 2.0, 58.0, is_white);
    let right1 = in_box(&cap1, 520.0, 686.0, 2.0, 58.0, is_white);
    let (l1n, l1x, _) = sig(&left1);
    let (r1n, r1x, _) = sig(&right1);
    assert!(
        (l1n as i32 - l0n as i32).abs() > 2 || (l1x - l0x).abs() > 2.0,
        "count glyphs changed at collect 1: {l0n}/{l0x} -> {l1n}/{l1x}"
    );
    assert!(
        (r1n as i32 - r0n as i32).abs() <= 2,
        "state glyphs unchanged at collect 1: {r0n} -> {r1n}"
    );
    assert!((r1x - r0x).abs() <= 2.0, "state cluster fixed: {r0x} -> {r1x}");
    let c1_1 = in_box(&cap1, 293.0, 317.0, 149.0, 175.0, is_gold);
    let c2_1 = in_box(&cap1, 388.0, 412.0, 149.0, 175.0, is_gold);
    let c3_1 = in_box(&cap1, 483.0, 507.0, 149.0, 175.0, is_gold);
    assert!(c1_1.len() < 2, "coin 1 consumed at collect 1: {}", c1_1.len());
    assert!(c2_1.len() >= 3, "coin 2 persists: {}", c2_1.len());
    assert!(c3_1.len() >= 3, "coin 3 persists: {}", c3_1.len());
    println!(
        "T04_GPU glyphs: count0={l0n}px@({l0x:.1},{l0y:.1}) state0={r0n}px@({r0x:.1},{r0y:.1}) count1={l1n}px state_won={rw}px chunks={chunks}"
    );
}
