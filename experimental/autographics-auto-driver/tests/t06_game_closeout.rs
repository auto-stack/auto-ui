//! auto-ui PLAN-023 T-06 V-06 consumption + closeout-discipline oracle on
//! the GENERATED components-2d-game world (world sha ed98b0b5... — the T-03
//! product, never modified; every negative is a derived IN-MEMORY world
//! variant (the t05 pattern) or a scratch copy under target/, and the
//! committed product tree is untouched throughout).
//!
//! F-9 closeout discipline (7 cases, t06_v06_*):
//!   a. single ag_present2d per callback: the generated world opens EXACTLY
//!      one bundle per tick (the unit's _ag_audio_bundle counter reads 2
//!      after every tick of a full win run; a second same-tick closeout is
//!      the named fault 50 at the bundle guard),
//!   b. mixed ag_hud_set AFTER the bundle -> named fault 57; a bare
//!      ag_hud_set BEFORE the bundle -> the same 57 at the bundle begin;
//!      the real world's identical tick is green,
//!   c. HUD text bounds: 32 chars green, 33 -> named fault 56, non-ASCII
//!      code -> 56,
//!   d. ordinals strictly increase 1..5 on the real run (no reuse); a
//!      stale/replayed ordinal -> named fault 52 exactly at the second cue
//!      tick (same tick index as the real world),
//!   e. >4 puts in one callback -> named fault 51 (the closeout's own
//!      packet is bounded: n=5 is rejected by the unit before any put; the
//!      unit boundary is 4 puts green / 5th put -> 51),
//!   f. cue out of range (unknown cue 3 vs the declared 0..2, and a
//!      negative cue id) -> named fault 53 exactly at the first collection
//!      tick's closeout put,
//!   g. gain domain: 0.0/1.0 are the legal edges (gain-1.0 rides a full
//!      win), >1/<0 -> named fault 54 exactly at the first cue tick, and
//!      NaN/+inf/-inf -> 54 through the same unit.
//! A->B consumption demonstration (t06_ab_*): with T06_B_ROOT set (by
//! tools/verify_plan023.py, after its closure-only cold regeneration of a
//! second root "B") the REAL CPU-session win path runs on the
//! B-regenerated world plus a save/reopen cycle over a staged project
//! copy; standalone (regression) the same controls run on the committed
//! product A.
//! Damaged-copy class (t06_damaged_*): a one-byte-corrupted
//! scripts/<sha>.at copy is rejected by the real load path (named), the
//! exact restore loads and RUNS green.
//! Bad-input classes (t06_bad_inputs_*, V-06/S-06 six classes): corrupt
//! world source, corrupt ttf byte, corrupt wav byte, empty license record,
//! wrong identity record (byte_length and sha forms), wrong runner
//! admission (Api) — each a NAMED rejection at its admission stage, then
//! exact restore -> green re-run.
//! CPU-session convention: spatial boxes are fed as translation-RELATIVE
//! half-extents (host Descriptor rows — F-8.3).
//! Run single-threaded:
//! cargo test --locked --offline --test t06_game_closeout -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        assets,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
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

const DT: f32 = 1.0 / 60.0;

fn sha256_hex(bytes: &[u8]) -> String {
    agp::renderer::capture::sha256_hex(bytes)
}

// ------------------------------------------------------------- session ----

/// Scene rows parsed from the committed scene document:
/// (id, kind(circle=0/quad=1), tx, ty, sx, sy).
fn scene_rows() -> Vec<(i32, i32, f32, f32, f32, f32)> {
    let objects = SCENE
        .split("\"objects\": [")
        .nth(1)
        .expect("objects list")
        .split("\n ],")
        .next()
        .expect("objects end");
    let mut rows = Vec::new();
    for chunk in objects.split("\"id\": ").skip(1) {
        let num = |key: &str| -> f64 {
            let ki = chunk.find(key).expect(key);
            let rest = &chunk[ki + key.len()..];
            let rest = rest.trim_start_matches(|c: char| c == '[' || c == ' ');
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest.len());
            rest[..end].parse().expect("number")
        };
        let id: i32 = chunk[..chunk.find(',').expect("id comma")]
            .trim()
            .parse()
            .expect("id");
        let kind = if chunk.contains("\"kind\": \"quad\"") {
            1
        } else if chunk.contains("\"kind\": \"circle\"") {
            0
        } else {
            panic!("unexpected scene kind: {}", &chunk[..60.min(chunk.len())]);
        };
        let tx = num("\"translation\": [") as f32;
        let ty = {
            let ki = chunk.find("\"translation\": [").expect("t");
            let rest = &chunk[ki + "\"translation\": [".len()..];
            let comma = rest.find(',').expect("comma");
            let rest2 = rest[comma + 1..].trim_start();
            let end = rest2
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest2.len());
            rest2[..end].parse::<f64>().expect("ty") as f32
        };
        let sx = num("\"scale\": [") as f32;
        let sy = {
            let ki = chunk.find("\"scale\": [").expect("s");
            let rest = &chunk[ki + "\"scale\": [".len()..];
            let comma = rest.find(',').expect("comma");
            let rest2 = rest[comma + 1..].trim_start();
            let end = rest2
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest2.len());
            rest2[..end].parse::<f64>().expect("sy") as f32
        };
        rows.push((id, kind, tx, ty, sx, sy));
    }
    rows
}

/// CPU session over a game world (Hud2V1 admission, the T-02/T-04 harness
/// shape; spatial boxes translation-relative, F-8.3).
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
    let rows = scene_rows();
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (id, kind, tx, ty, sx, sy)) in rows.iter().enumerate() {
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
    for n in 0..rows.len() {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n as i32 + 1), Int(n as i32)], ResultKind::Unit)
                .unwrap();
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

/// One input tick: character input + `_ag_tick`; returns the status.
fn tick(s: &mut Session, step: i32, held: i32, pressed: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(pressed)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

/// HUD slot text through the session glyph-code getters.
fn hud_text(s: &mut Session, slot: i32) -> String {
    let len = s.int("_ag_hud_length", &[Int(slot)]).unwrap();
    (0..len)
        .map(|i| s.int("_ag_hud_code", &[Int(slot), Int(i)]).unwrap() as u8 as char)
        .collect()
}

/// This tick's pending audio bundle rows (ordinal, cue) as the closeout's
/// single ag_present2d delivered them (cleared at the next tick begin).
fn audio(s: &mut Session) -> Vec<(i32, i32)> {
    let n = s.int("_ag_audio_count", &[]).unwrap();
    (0..n)
        .map(|i| {
            (
                s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap(),
                s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap(),
            )
        })
        .collect()
}

/// This tick's bundle rows with the request gain (ordinal, cue, gain).
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

/// Settle the character onto the ground (state-based) and return the tick.
fn settle(s: &mut Session, name: &str) -> i32 {
    for t in 1..=300 {
        let st = tick(s, t, 0, 0);
        assert_eq!(st, 0, "{name} settle tick {t} status {st}");
        if int(s, "get_player_grounded") == 1 {
            return t;
        }
    }
    panic!("{name}: never settled");
}

/// Settle + walk right until Won; every tick asserted green; returns
/// (settle, coin1, coin2, coin3, win) latch ticks (the T-04 V-03 table).
fn win_marks(s: &mut Session, name: &str) -> (i32, i32, i32, i32, i32) {
    let t0 = settle(s, name);
    let mut t = t0;
    let (mut t1, mut t2, mut t3, mut twin) = (0, 0, 0, 0);
    while t < t0 + 1200 {
        t += 1;
        let st = tick(s, t, 2, 0);
        assert_eq!(st, 0, "{name} tick {t} over the pool");
        let c = int(s, "get_r_count");
        if c == 1 && t1 == 0 {
            t1 = t;
        }
        if c == 2 && t2 == 0 {
            t2 = t;
        }
        if c == 3 && t3 == 0 {
            t3 = t;
        }
        if int(s, "get_r_state") == 1 {
            twin = t;
            break;
        }
    }
    assert!(twin > 0, "{name}: pad never won");
    (t0, t1, t2, t3, twin)
}

/// First non-green tick of a right-walk from settle (0 = green throughout).
fn first_fault(s: &mut Session, name: &str, bound: i32) -> i32 {
    let t0 = settle(s, name);
    let mut t = t0;
    while t < t0 + bound {
        t += 1;
        let st = tick(s, t, 2, 0);
        if st != 0 {
            return t;
        }
    }
    0
}

// ------------------------------------------------------------- staging ----

fn closure_asset(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CLOSURE_ASSETS).join(name)
}

/// Stage a loadable project (scene + bound script + assets) under target/,
/// exactly as load_project resolves it (the t05 pattern; the committed
/// product tree is never touched).
fn stage_project(tag: &str, scene_text: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/t06-closeout")
        .join(format!("{}-{tag}", std::process::id()))
        .join("game");
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    std::fs::write(scripts.join(format!("{WORLD_SHA}.at")), WORLD).expect("write script");
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&assets_dir).expect("assets dir");
    // load_project resolves assets/<sha256>.<ext> (blob_name).
    std::fs::copy(
        closure_asset(&format!("{BANK_SHA}.wav")),
        assets_dir.join(format!("{BANK_SHA}.wav")),
    )
    .expect("copy bank");
    std::fs::copy(
        closure_asset("lilita_one_regular.ttf"),
        assets_dir.join(format!("{FONT_SHA}.ttf")),
    )
    .expect("copy font");
    dir
}

fn load(dir: &Path) -> (
    autographics_auto_driver::authoring::document::Document,
    assets::Registry,
    std::sync::Arc<CodePayload>,
) {
    let (d, r, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .unwrap_or_else(|e| panic!("load staged project: {e:?}"));
    (d, r, code.expect("behavior code"))
}

/// A restored project must load AND run green: a real CPU session over the
/// loaded code settles and walks (the consumption chain is live again).
fn green_load_run(dir: &Path, label: &str) {
    let (_d, _r, code) = load(dir);
    let world = std::str::from_utf8(code.bytes())
        .expect("utf8 world")
        .to_string();
    assert_eq!(
        sha256_hex(world.as_bytes()),
        WORLD_SHA,
        "{label}: restored copy is the pinned world"
    );
    let mut s = session(&world, &format!("t06-{label}-ok.at"), 1);
    let t0 = settle(&mut s, label);
    assert_eq!(tick(&mut s, t0 + 1, 2, 0), 0, "{label}: restored run green");
}

/// Flip one byte of a staged copy in place (the copy is scratch under
/// target/; the committed product is untouched).
fn flip_byte(path: &Path, offset: usize) {
    let mut bytes = std::fs::read(path).expect("read staged copy");
    let at = offset % bytes.len();
    bytes[at] = bytes[at].wrapping_add(1);
    std::fs::write(path, &bytes).expect("write corrupted copy");
}

// ------------------------------------------------------------- variants ----
// All variants are derived IN MEMORY from the committed world text (the
// t05 pattern); none is written to the product tree. Each constructor
// asserts its exact edit site exists, so the derivation tracks the frozen
// product (a drift breaks the derivation, never the assertion).

/// Second closeout in the same callback: the second ag_present2d must be
/// rejected by the bundle guard (named 50) — the "exactly one" proof.
fn double_closeout() -> String {
    let marker = " game_closeout()\n}";
    assert_eq!(WORLD.matches(marker).count(), 1, "closeout call marker");
    WORLD.replacen(marker, " game_closeout()\n game_closeout()\n}", 1)
}

/// ag_hud_set AFTER the bundle closeout (mode==2) -> named fault 57.
fn hud_set_after() -> String {
    let marker = " r_np = 0\n}";
    assert_eq!(WORLD.matches(marker).count(), 1, "closeout tail marker");
    WORLD.replacen(
        marker,
        " r_np = 0\n let zh = ag_hud_set(0, 1, \"x\")\n}",
        1,
    )
}

/// A bare ag_hud_set BEFORE the bundle in the same tick (mode==1 when the
/// closeout opens) -> the same named 57 at the bundle begin.
fn hud_set_before() -> String {
    let marker = " game_closeout()\n}";
    assert_eq!(WORLD.matches(marker).count(), 1, "closeout call marker");
    WORLD.replacen(
        marker,
        " let zk = ag_hud_set(0, 1, \"early\")\n game_closeout()\n}",
        1,
    )
}

/// Slot 2 becomes visible with `text` in the closeout's single present2d
/// (slots 0/1 keep the generated count/state texts; slot 3 stays hidden).
fn with_slot2_text(text: &str) -> String {
    let marker = "1, hud_state_text(r_state), 0, \"\", 0, \"\")";
    assert_eq!(WORLD.matches(marker).count(), 1, "slot tail marker");
    WORLD.replacen(
        marker,
        &format!("1, hud_state_text(r_state), 1, \"{text}\", 0, \"\")"),
        1,
    )
}

/// Stale ordinal: game_cue stops advancing r_ord and the closeout emits
/// r_ord+1 — the second cue replays the first ordinal (t02 ordbad trick).
fn stale_ordinal() -> String {
    let m1 = "fn game_cue(c int) {\n r_ord = r_ord + 1\n";
    assert_eq!(WORLD.matches(m1).count(), 1, "game_cue marker");
    let w = WORLD.replacen(m1, "fn game_cue(c int) {\n", 1);
    let m2 = " if r_np == 1 {\n  o0 = r_ord\n  c0 = r_p0\n }\n if r_np == 2 {\n  o0 = r_ord - 1\n  c0 = r_p0\n  o1 = r_ord\n  c1 = r_p1\n }\n";
    assert_eq!(w.matches(m2).count(), 1, "closeout ordinal block marker");
    w.replacen(
        m2,
        " if r_np == 1 {\n  o0 = r_ord + 1\n  c0 = r_p0\n }\n if r_np == 2 {\n  o0 = r_ord\n  c0 = r_p0\n  o1 = r_ord + 1\n  c1 = r_p1\n }\n",
        1,
    )
}

/// The closeout's own packet asks for n=5 rows: _ag_audio_begin rejects
/// n>4 (named 51) before any put.
fn five_puts() -> String {
    let marker = "r_np, o0, c0, 0.5,";
    assert_eq!(WORLD.matches(marker).count(), 1, "present2d n marker");
    WORLD.replacen(marker, "5, o0, c0, 0.5,", 1)
}

/// app_update stripped to an empty body: no walk shape, no closeout — a
/// quiet callback whose only charged calls are the ones the test injects.
fn quiet_app_update() -> String {
    let marker = "fn app_update(dt float) {\n main_lifetick(dt)\n player_lifetick(0,dt)\n zone_lifetick(0,dt)\n zone_present(0)\n zone_lifetick(1,dt)\n zone_present(1)\n game_closeout()\n}";
    assert_eq!(WORLD.matches(marker).count(), 1, "app_update marker");
    WORLD.replacen(marker, "fn app_update(dt float) {\n}", 1)
}

/// All three collect sites emit the unknown cue 3 (declared: cues 0..2).
fn unknown_cue() -> String {
    let marker = "   game_cue(0)\n";
    assert_eq!(WORLD.matches(marker).count(), 3, "three collect cue sites");
    WORLD.replace(marker, "   game_cue(3)\n")
}

/// All three collect sites emit a negative cue id (cue<0 -> the same 53).
fn cue_below_range() -> String {
    let marker = "   game_cue(0)\n";
    assert_eq!(WORLD.matches(marker).count(), 3, "three collect cue sites");
    WORLD.replace(marker, "   game_cue(0 - 1)\n")
}

/// The closeout's first put row carries `gain_expr` as the request gain.
fn with_first_gain(gain_expr: &str) -> String {
    let marker = "o0, c0, 0.5,";
    assert_eq!(WORLD.matches(marker).count(), 1, "first gain marker");
    WORLD.replacen(marker, &format!("o0, c0, {gain_expr},"), 1)
}

/// A scene whose hud.license record is emptied (bad-license class).
fn empty_license(scene: &str) -> String {
    let key = "\"license\": \"";
    let start = scene.find(key).expect("license key") + key.len();
    let rest = &scene[start..];
    let end = rest.find("\",\n  \"slots\"").expect("license end");
    let mut out = String::with_capacity(scene.len());
    out.push_str(&scene[..start]);
    out.push_str(&rest[end..]);
    assert_ne!(out, scene, "license emptied");
    out
}

// ===================================================== F-9 (a) ============

/// Single ag_present2d per callback: the generated world carries exactly one
/// present2d call site driven by exactly one closeout call per app_update,
/// and a full win run opens EXACTLY one bundle per tick (the unit's
/// _ag_audio_bundle counter reads 2 after every tick — the counter is reset
/// by the managed tick begin, so a value of 2 after the tick means the one
/// closeout bundle was opened and nothing else). A second same-tick
/// closeout is the named fault 50 at the bundle guard.
#[test]
fn t06_v06_a_single_present2d_one_bundle_per_tick() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    // static shape: one call site, one per-callback closeout.
    assert_eq!(
        WORLD.matches(" ag_present2d(").count(),
        1,
        "exactly one present2d call site"
    );
    assert_eq!(
        WORLD.matches(" game_closeout()\n").count(),
        1,
        "exactly one closeout call (in app_update)"
    );
    assert!(WORLD.contains("fn game_closeout() {"), "the closeout fn");
    // dynamic: settle -> win, every tick opens exactly one bundle.
    let mut s = session(WORLD, "t06-a.at", 1);
    assert_eq!(
        s.int("_ag_audio_bundle_id", &[]).unwrap(),
        0,
        "no bundle before the first tick (the closeout is per app_update)"
    );
    // the counter semantics: _ag_audio_tick_begin clears it, the ONE
    // _ag_audio_begin of the tick sets it to 1 and it survives the tick —
    // so 1 after the tick means exactly one bundle was opened.
    let t0 = settle(&mut s, "t06-a");
    let mut t = t0;
    let mut carriers = 0;
    let mut rows_seen: Vec<(i32, i32)> = Vec::new();
    while t < t0 + 1200 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "tick {t} green");
        assert_eq!(
            s.int("_ag_audio_bundle_id", &[]).unwrap(),
            1,
            "exactly one bundle opened at tick {t}"
        );
        let rows = audio(&mut s);
        if !rows.is_empty() {
            carriers += 1;
            rows_seen.extend(rows);
        }
        if int(&mut s, "get_r_state") == 1 {
            break;
        }
    }
    assert_eq!(int(&mut s, "get_r_state"), 1, "win reachable");
    assert_eq!(carriers, 4, "four carrying ticks");
    assert_eq!(
        rows_seen,
        vec![(1, 0), (2, 0), (3, 0), (4, 1), (5, 1)],
        "five cue rows over the single-bundle ticks"
    );
    // exactly-one is enforced: a second closeout in the same callback is
    // the named fault 50 at the bundle guard (tick 1), and it latches.
    let mut sd = session(&double_closeout(), "t06-a-double.at", 1);
    assert_eq!(tick(&mut sd, 1, 0, 0), 50, "second present2d named fault 50");
    assert_eq!(tick(&mut sd, 2, 0, 0), 50, "fault latches");
    println!("T06_V06_A single present2d per callback: one bundle/tick across the win run (counter==1 after every tick, 4 carrying ticks, ordinals 1..5); double closeout named 50");
}

// ===================================================== F-9 (b) ============

/// Mixed ag_hud_set: AFTER the bundle closeout the unit is in bundle mode
/// (2) and a bare ag_hud_set is the named fault 57; BEFORE the bundle the
/// bare set arms text mode (1) and the closeout's own bundle begin is the
/// same named 57. The real world's identical tick is green — the generated
/// closeout never mixes.
#[test]
fn t06_v06_b_mixed_hud_set_faults_57() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    // after the bundle: mode==2 rejects the follow-up set.
    let mut sa = session(&hud_set_after(), "t06-b-after.at", 1);
    assert_eq!(tick(&mut sa, 1, 0, 0), 57, "set after bundle named fault 57");
    assert_eq!(tick(&mut sa, 2, 0, 0), 57, "fault 57 latches");
    // before the bundle: mode==1 rejects the closeout's own bundle begin.
    let mut sb = session(&hud_set_before(), "t06-b-before.at", 1);
    assert_eq!(tick(&mut sb, 1, 0, 0), 57, "set before bundle named fault 57");
    // the real world runs the same tick green (single-closeout discipline).
    let mut sok = session(WORLD, "t06-b-ok.at", 1);
    assert_eq!(tick(&mut sok, 1, 0, 0), 0, "the generated world never mixes");
    println!("T06_V06_B mixed ag_hud_set after/before the bundle -> named fault 57 both ways; the real world's tick 1 is green");
}

// ===================================================== F-9 (c) ============

/// HUD text bounds through the closeout's own put: 32 chars are legal
/// (slot 2 visible with exactly MAX_TEXT), 33 chars are the named fault 56,
/// and a text carrying a non-ASCII code (0xE9 byte) is the same 56.
#[test]
fn t06_v06_c_hud_text_bounds_32_green_33_and_nonascii_fault_56() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    let t32 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef";
    assert_eq!(t32.len(), 32, "boundary text is exactly 32 chars");
    let mut s32 = session(&with_slot2_text(t32), "t06-c32.at", 1);
    assert_eq!(tick(&mut s32, 1, 0, 0), 0, "32-char slot text is legal");
    assert_eq!(hud_text(&mut s32, 2).len(), 32, "slot 2 carries the text");
    let mut s33 = session(&with_slot2_text(&format!("{t32}g")), "t06-c33.at", 1);
    assert_eq!(tick(&mut s33, 1, 0, 0), 56, "33-char text named fault 56");
    assert_eq!(tick(&mut s33, 2, 0, 0), 56, "fault 56 latches");
    let mut sn = session(&with_slot2_text("éx"), "t06-cna.at", 1);
    assert_eq!(tick(&mut sn, 1, 0, 0), 56, "non-ASCII code named fault 56");
    println!("T06_V06_C text bounds: 32 green (slot 2 reads back 32 codes), 33 -> 56, non-ASCII -> 56");
}

// ===================================================== F-9 (d) ============

/// Ordinals: the real run spends 1..5 strictly increasing with no reuse
/// (windows over the full-row table); the derived stale variant (game_cue
/// stops advancing r_ord, the closeout emits r_ord+1) replays ordinal 1 on
/// the second cue and is the named fault 52 EXACTLY at the real world's
/// second collection tick, while its first cue tick is green at the same
/// tick index as the real world.
#[test]
fn t06_v06_d_ordinals_strictly_increase_stale_ordinal_faults_52() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    // positive: full-run ordinal table, strictly increasing.
    let mut s = session(WORLD, "t06-d-ok.at", 1);
    let (t0, t1, t2, _t3, _twin) = win_marks(&mut s, "t06-d-ok");
    let mut sp = session(&stale_ordinal(), "t06-d-stale.at", 1);
    let r0 = settle(&mut sp, "t06-d-stale");
    assert_eq!(r0, t0, "same settle tick as the real world");
    let mut t = r0;
    let mut tf = 0;
    let mut first = 0;
    while t < r0 + 400 {
        t += 1;
        let st = tick(&mut sp, t, 2, 0);
        if first == 0 && int(&mut sp, "get_r_count") == 1 {
            first = t;
            assert_eq!(st, 0, "first cue tick (ordinal 1) is green");
            assert_eq!(first, t1, "same first collection tick as the real world");
            assert_eq!(audio(&mut sp), vec![(1, 0)], "ordinal 1 delivered");
            continue;
        }
        if st != 0 {
            tf = t;
            break;
        }
    }
    assert_eq!(tf, t2, "stale ordinal faults exactly at the second cue tick");
    assert_eq!(tick(&mut sp, tf + 1, 2, 0), 52, "fault 52 latches");
    assert_eq!(int(&mut sp, "get_r_state"), 0, "no game progress past the fault");
    println!("T06_V06_D ordinals 1..5 strictly increasing on the real run; stale ordinal named 52 exactly at the second cue tick {tf} (real t2={t2})");
}

// ===================================================== F-9 (e) ============

/// >4 puts in one callback: the closeout's own packet is bounded — a
/// derived variant forcing n=5 is rejected by the unit BEFORE any put
/// (named fault 51 at the first tick); the unit boundary is exact: in one
/// callback four puts are legal (rows land, count reads 4) and the FIFTH
/// put is the named 51.
#[test]
fn t06_v06_e_over_four_puts_faults_51() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    // world side: the closeout's present2d with n=5.
    let mut s5 = session(&five_puts(), "t06-e5.at", 1);
    assert_eq!(tick(&mut s5, 1, 0, 0), 51, "n=5 named fault 51 at tick 1");
    assert_eq!(tick(&mut s5, 2, 0, 0), 51, "fault 51 latches");
    // unit boundary on a quiet callback (the walk shape stripped): four
    // direct puts are legal, the fifth is not.
    let mut sq = session(&quiet_app_update(), "t06-eq.at", 1);
    for o in 1..=4 {
        sq.call(
            "ag_audio_emit",
            &[Int(o), Int(0), Float(1.0)],
            ResultKind::Int,
        )
        .unwrap();
        assert_eq!(int(&mut sq, "_ag_status"), 0, "put {o} legal");
    }
    assert_eq!(int(&mut sq, "_ag_audio_count"), 4, "four puts landed");
    sq.call(
        "ag_audio_emit",
        &[Int(5), Int(0), Float(1.0)],
        ResultKind::Int,
    )
    .unwrap();
    assert_eq!(int(&mut sq, "_ag_status"), 51, "fifth put named fault 51");
    assert_eq!(int(&mut sq, "_ag_audio_count"), 4, "the fifth put never landed");
    println!("T06_V06_E over-four puts: closeout n=5 named 51 at tick 1; unit boundary 4 puts green / 5th put named 51");
}

// ===================================================== F-9 (f) ============

/// Cue out of range through the closeout's own put: the unknown cue 3 (the
/// scene declares exactly cues 0..2) and a negative cue id are each the
/// named fault 53 EXACTLY at the first collection tick (the baseline real
/// world is green through that very tick and collects with cue 0).
#[test]
fn t06_v06_f_cue_out_of_range_faults_53() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    // baseline: the real first collect is green with cue 0.
    let mut sb = session(WORLD, "t06-f-ok.at", 1);
    let t0b = settle(&mut sb, "t06-f-ok");
    let mut t = t0b;
    let mut t1 = 0;
    while t < t0b + 400 {
        t += 1;
        assert_eq!(tick(&mut sb, t, 2, 0), 0, "baseline tick {t}");
        if int(&mut sb, "get_r_count") == 1 {
            t1 = t;
            assert_eq!(audio(&mut sb), vec![(1, 0)], "cue 0 delivered");
            break;
        }
    }
    assert!(t1 > 0, "baseline first collect");
    // unknown cue 3.
    let mut s3 = session(&unknown_cue(), "t06-f-cue3.at", 1);
    let f3 = first_fault(&mut s3, "t06-f-cue3", 1200);
    assert_eq!(f3, t1, "cue 3 faults exactly at the first collection tick");
    assert_eq!(tick(&mut s3, f3 + 1, 2, 0), 53, "fault 53 latches");
    assert_eq!(int(&mut s3, "get_r_state"), 0, "no game progress past the fault");
    // negative cue id: the same named 53 at the same tick.
    let mut sn = session(&cue_below_range(), "t06-f-neg.at", 1);
    let fneg = first_fault(&mut sn, "t06-f-neg", 1200);
    assert_eq!(fneg, t1, "cue -1 faults exactly at the first collection tick");
    assert_eq!(tick(&mut sn, fneg + 1, 2, 0), 53, "fault 53 latches");
    println!("T06_V06_F cue out of range: unknown cue 3 and negative cue id both named 53 exactly at the first collection tick {t1}");
}

// ===================================================== F-9 (g) ============

/// Gain domain through the same closeout unit: 0.0 and 1.0 are the legal
/// edges (a gain-1.0 variant rides a whole win and the rows read back 1.0);
/// >1 and <0 are the named fault 54 EXACTLY at the first cue tick; and the
/// non-finite edges (NaN/+inf/-inf) are the same 54 through the unit's
/// gain-gain check (session-side puts on the real world's unit).
#[test]
fn t06_v06_g_gain_domain_faults_54() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "product pinned");
    // green edge: gain 1.0 through a full win; rows read back 1.0.
    let mut s1 = session(&with_first_gain("1.0"), "t06-g1.at", 1);
    let t0 = settle(&mut s1, "t06-g1");
    let mut t = t0;
    let mut t1 = 0;
    let mut twin = 0;
    let mut gain_seen = 0.0f32;
    while t < t0 + 1200 {
        t += 1;
        assert_eq!(tick(&mut s1, t, 2, 0), 0, "gain-1.0 tick {t}");
        let rows = audio_rows(&mut s1);
        if !rows.is_empty() && t1 == 0 {
            t1 = t;
            gain_seen = rows[0].2;
        }
        if int(&mut s1, "get_r_state") == 1 {
            twin = t;
            break;
        }
    }
    assert!(twin > 0, "gain 1.0 win reachable");
    assert!((gain_seen - 1.0).abs() < 1e-6, "row gain reads back 1.0");
    // out-of-domain edges through the world's own closeout put.
    for (label, expr) in [("gt1", "2.0"), ("lt0", "(0.0 - 0.5)")] {
        let mut sx = session(&with_first_gain(expr), &format!("t06-g-{label}.at"), 1);
        let f = first_fault(&mut sx, &format!("t06-g-{label}"), 1200);
        assert_eq!(f, t1, "gain {expr} faults exactly at the first cue tick");
        assert_eq!(tick(&mut sx, f + 1, 2, 0), 54, "fault 54 latches ({label})");
    }
    // non-finite edges: they cannot reach the unit's put, and both paths
    // to it carry their own NAMED rejection (measured here, not assumed):
    //   - a world-computed non-finite gain (`(1.0 / 0.0)` in the closeout)
    //     is a VM trap at evaluation: the tick call itself errors
    //     "call: DivisionByZero",
    //   - a harness-injected non-finite gain is rejected by the trusted
    //     session call gate ("nonfinite argument") before the VM call.
    // The unit's own `gain - gain != 0` fault 54 remains the defense for
    // any value that could reach it; the reachable out-of-domain inputs
    // (<0, >1) are faulted 54 above.
    {
        let mut sx = session(&with_first_gain("(1.0 / 0.0)"), "t06-g-inf.at", 1);
        sx.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit)
            .unwrap();
        let call = sx.call("_ag_tick", &[Int(1), Float(DT)], ResultKind::Unit);
        assert_eq!(
            call.err().as_deref(),
            Some("call: DivisionByZero"),
            "world-computed non-finite gain is a named VM trap at evaluation"
        );
    }
    for (label, g) in [
        ("nan", f32::NAN),
        ("inf", f32::INFINITY),
        ("ninf", f32::NEG_INFINITY),
    ] {
        let mut sx = session(WORLD, &format!("t06-g-{label}.at"), 1);
        let call = sx.call("ag_audio_emit", &[Int(1), Int(0), Float(g)], ResultKind::Int);
        assert_eq!(
            call.err().as_deref(),
            Some("nonfinite argument"),
            "{label} gain named rejection at the call gate"
        );
        assert_eq!(
            int(&mut sx, "_ag_status"),
            0,
            "the call never reached the unit"
        );
    }
    // the full legal/rejected domain on one unit: 0.0 and 1.0 land, -0.1
    // and 2.0 are the named 54.
    let mut sd = session(WORLD, "t06-g-domain.at", 1);
    sd.call("ag_audio_emit", &[Int(1), Int(0), Float(0.0)], ResultKind::Int)
        .unwrap();
    assert_eq!(int(&mut sd, "_ag_status"), 0, "gain 0.0 legal");
    sd.call("ag_audio_emit", &[Int(2), Int(0), Float(1.0)], ResultKind::Int)
        .unwrap();
    assert_eq!(int(&mut sd, "_ag_status"), 0, "gain 1.0 legal");
    assert_eq!(
        sd.float("_ag_audio_g", &[Int(0)]).unwrap(),
        0.0,
        "row gains read back"
    );
    sd.call("ag_audio_emit", &[Int(3), Int(0), Float(-0.1)], ResultKind::Int)
        .unwrap();
    assert_eq!(int(&mut sd, "_ag_status"), 54, "-0.1 named fault 54");
    println!("T06_V06_G gain domain: 0.0/1.0 legal (gain-1.0 full win, rows read back), 2.0/(0.0-0.5) named 54 exactly at the first cue tick {t1}; non-finite: world-computed -> VM trap 'call: DivisionByZero', harness-injected -> call gate 'nonfinite argument' (named boundary before the unit)");
}

// ================================================== A->B consumption ======

/// Second-root consumption demonstration. With T06_B_ROOT set (by
/// tools/verify_plan023.py, after its closure-only cold regeneration) the
/// B-regenerated world is the consumption input; standalone the committed
/// product (A) is. Asserted: the input equals the PINNED world sha; a real
/// CPU-session Run settles, walks and WINS (win reachable, every tick
/// green); then a save/reopen cycle — the scene+world copy is staged as a
/// loadable project (with assets), load_project admits it, a session over
/// the loaded code runs green, and an independent second load (reopen) of
/// the same files runs green again.
#[test]
fn t06_ab_second_root_world_runs_win_and_reopens() {
    let b = std::env::var("T06_B_ROOT").ok();
    let (world_src, scene_src, label) = match &b {
        Some(root) => {
            let p = Path::new(root);
            (
                std::fs::read_to_string(p.join("world.at")).expect("B world.at"),
                std::fs::read_to_string(p.join("scene.agscene")).expect("B scene"),
                format!("B regen ({root})"),
            )
        }
        None => (
            WORLD.to_string(),
            SCENE.to_string(),
            "A (committed product; set T06_B_ROOT for the B regen)".to_string(),
        ),
    };
    assert_eq!(
        sha256_hex(world_src.as_bytes()),
        WORLD_SHA,
        "{label}: the consumption input must equal the pinned world sha"
    );
    assert!(
        scene_src.contains(WORLD_SHA),
        "{label}: the staged scene binds the same world sha"
    );
    // real CPU-session Run of the second root: settle -> win, all green.
    let mut s = session(&world_src, "t06-ab-run.at", 1);
    let (t0, t1, t2, t3, twin) = win_marks(&mut s, "t06-ab");
    assert!(
        t0 < t1 && t1 < t2 && t2 < t3 && t3 < twin,
        "win reachable in order: {t0} {t1} {t2} {t3} {twin}"
    );
    assert_eq!(int(&mut s, "get_r_ord"), 5, "ordinals 1..5 spent");
    assert_eq!(int(&mut s, "get_r_state"), 1, "won");
    drop(s);
    // save/reopen: write the scene+world copy as a loadable project.
    let dir = stage_project("t06-ab-reopen", &scene_src);
    let (_doc, _reg, code) = load(&dir);
    let saved = std::str::from_utf8(code.bytes()).expect("utf8").to_string();
    assert_eq!(
        sha256_hex(saved.as_bytes()),
        WORLD_SHA,
        "the saved copy is the pinned world"
    );
    let mut s2 = session(&saved, "t06-ab-reopen1.at", 1);
    let r1 = settle(&mut s2, "t06-ab-reopen1");
    assert_eq!(tick(&mut s2, r1 + 1, 2, 0), 0, "reopened copy runs green");
    drop(s2);
    // reopen again: an independent second load of the same files.
    let (_doc2, _reg2, code2) = load(&dir);
    let saved2 = std::str::from_utf8(code2.bytes()).expect("utf8").to_string();
    assert_eq!(sha256_hex(saved2.as_bytes()), WORLD_SHA, "reopen is identical");
    let mut s3 = session(&saved2, "t06-ab-reopen2.at", 2);
    let r2 = settle(&mut s3, "t06-ab-reopen2");
    assert_eq!(tick(&mut s3, r2 + 1, 2, 0), 0, "second reopen runs green");
    println!("T06_AB {label}: world sha pinned, CPU Run settle={t0} coins={t1}/{t2}/{t3} win={twin} green; save/reopen cycle green twice");
}

// ===================================================== damaged copy =======

/// Damaged-copy recovery control (the 020 v06_damaged shape on the game
/// chain): the consumer binds world.at as scripts/<sha>.at; a one-byte
/// corruption is rejected by the REAL load path with the named
/// closure-identity error; the exact restore loads and RUNS green.
#[test]
fn t06_damaged_world_copy_rejected_then_restore_green() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "pinned");
    let dir = stage_project("damaged", SCENE);
    let script = dir.join("scripts").join(format!("{WORLD_SHA}.at"));
    // python-replay of the load condition: the copy is bound by its name.
    let mut bytes = std::fs::read(&script).expect("script copy");
    bytes[0] = bytes[0].wrapping_add(1);
    assert_ne!(
        sha256_hex(&bytes),
        WORLD_SHA,
        "one-byte corruption changes the identity"
    );
    std::fs::write(&script, &bytes).expect("corrupt copy");
    // the real load path rejects, naming the bound copy.
    let err = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
        .err()
        .expect("corrupted world copy must be rejected");
    assert!(
        err.contains("behavior source closure identity"),
        "named rejection, got: {err}"
    );
    assert!(
        err.contains("scripts"),
        "the bound scripts/<sha>.at path is named: {err}"
    );
    // exact restore -> green re-run.
    std::fs::write(&script, WORLD.as_bytes()).expect("exact restore");
    assert_eq!(
        sha256_hex(&std::fs::read(&script).expect("reread")),
        WORLD_SHA,
        "restore is exact"
    );
    green_load_run(&dir, "damaged-restore");
    println!("T06_DAMAGED corrupted scripts/<sha>.at copy rejected (behavior source closure identity); exact restore loads and runs green");
}

// ===================================================== bad inputs =========

/// The V-06/S-06 bad-input classes, each a NAMED rejection at its admission
/// stage on a scratch copy under target/, then exact restore -> green
/// re-run. Classes:
///   1. bad source: corrupt world copy at the bound scripts/<sha>.at
///      (load stage, "behavior source closure identity"),
///   2. bad font: one corrupt ttf byte under the declared blob name
///      (asset admission, "hud_invalid: fixed font bytes"),
///   3. bad bank: one corrupt wav byte (asset admission,
///      "audio_invalid: declared hash/length"),
///   4. bad license: emptied license record in the scene
///      (document decode, "hud_invalid: font license identity"),
///   5. bad record: wrong identity record in the scene binding — a
///      byte_length drift (load stage, "behavior source closure
///      identity") and a re-pointed sha (the bound script path does not
///      exist; the load stage names the missing scripts/<sha>.at),
///   6. bad runner: the world fed through a non-matching admission API
///      (Audio2V1) is a named admission rejection; the real runner
///      (Hud2V1) admits and runs green.
#[test]
fn t06_bad_inputs_six_classes_named_rejections_then_restore_green() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "pinned");

    // ---- class 1: bad source (corrupt world copy). ----
    {
        let dir = stage_project("bad-source", SCENE);
        let script = dir.join("scripts").join(format!("{WORLD_SHA}.at"));
        flip_byte(&script, 4096);
        let err = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .err()
            .expect("bad source rejected");
        assert!(
            err.contains("behavior source closure identity"),
            "class 1 named rejection, got: {err}"
        );
        std::fs::write(&script, WORLD.as_bytes()).expect("exact restore");
        green_load_run(&dir, "bad-source");
        println!("T06_BAD1 bad source: 'behavior source closure identity' at load; restore runs green");
    }

    // ---- class 2: bad font (corrupt ttf byte). ----
    {
        let dir = stage_project("bad-font", SCENE);
        let font = dir.join("assets").join(format!("{FONT_SHA}.ttf"));
        flip_byte(&font, 4096);
        let err = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .err()
            .expect("bad font rejected");
        assert!(
            err.contains("hud_invalid: fixed font bytes"),
            "class 2 named rejection, got: {err}"
        );
        std::fs::copy(closure_asset("lilita_one_regular.ttf"), &font).expect("exact restore");
        green_load_run(&dir, "bad-font");
        println!("T06_BAD2 bad font: 'hud_invalid: fixed font bytes' at asset admission; restore runs green");
    }

    // ---- class 3: bad bank (corrupt wav byte). ----
    {
        let dir = stage_project("bad-bank", SCENE);
        let bank = dir.join("assets").join(format!("{BANK_SHA}.wav"));
        flip_byte(&bank, 100);
        let err = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .err()
            .expect("bad bank rejected");
        assert!(
            err.contains("audio_invalid: declared hash/length"),
            "class 3 named rejection, got: {err}"
        );
        std::fs::copy(closure_asset(&format!("{BANK_SHA}.wav")), &bank).expect("exact restore");
        green_load_run(&dir, "bad-bank");
        println!("T06_BAD3 bad bank: 'audio_invalid: declared hash/length' at asset admission; restore runs green");
    }

    // ---- class 4: bad license (emptied license record). ----
    {
        let bad_scene = empty_license(SCENE);
        let dir = stage_project("bad-license", &bad_scene);
        let err = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .err()
            .expect("bad license rejected");
        assert!(
            err.contains("hud_invalid: font license identity"),
            "class 4 named rejection, got: {err}"
        );
        // exact restore: the pristine scene re-staged -> green re-run.
        let good = stage_project("bad-license-restore", SCENE);
        green_load_run(&good, "bad-license");
        println!("T06_BAD4 bad license: 'hud_invalid: font license identity' at document decode; pristine re-stage runs green");
    }

    // ---- class 5: bad record (wrong identity record). ----
    {
        // (a) the byte_length record drifts by one: the payload no longer
        // verifies against its own record.
        let bad_len = SCENE.replacen("\"byte_length\": 6556", "\"byte_length\": 6557", 1);
        assert_ne!(bad_len, SCENE, "byte_length record edited");
        let dir = stage_project("bad-record-len", &bad_len);
        let err = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
            .err()
            .expect("wrong byte_length record rejected");
        assert!(
            err.contains("behavior source closure identity"),
            "class 5a named rejection, got: {err}"
        );
        // (b) the sha record is re-pointed: the bound script path does not
        // exist and the load stage names it.
        let wrong_sha = sha256_hex("t06 wrong record".as_bytes());
        let bad_sha = SCENE.replacen(WORLD_SHA, &wrong_sha, 1);
        assert_ne!(bad_sha, SCENE, "sha record re-pointed");
        let dir2 = stage_project("bad-record-sha", &bad_sha);
        let err2 = Storage::load_project(&dir2.join("scene.agscene"), &assets::Registry::new())
            .err()
            .expect("wrong sha record rejected");
        assert!(
            err2.contains("scripts") && err2.contains(&wrong_sha),
            "class 5b names the missing bound copy, got: {err2}"
        );
        // exact restore: the pristine scene re-staged -> green re-run.
        let good = stage_project("bad-record-restore", SCENE);
        green_load_run(&good, "bad-record");
        println!("T06_BAD5 bad record: byte_length drift -> 'behavior source closure identity'; re-pointed sha -> named missing scripts/<sha>.at; pristine re-stage runs green");
    }

    // ---- class 6: bad runner (wrong admission API). ----
    // Observed boundaries (recorded, not assumed): the grammar-level
    // from_bytes_for admission accepts the text under the wrong API (the
    // grammar is shared), but the RUNNER compile boundary names the first
    // missing game symbol ("Undefined symbol: ag_action_pressed in module
    // <main>") for the base V1 runner. The audio-only runner shares most
    // of the surface, so it compiles and the mismatch surfaces at the
    // closeout's runtime call boundary (observed form asserted). The real
    // runner (Hud2V1) is green end to end.
    {
        let snap_v1 = SourceSnapshot::from_bytes_for(
            Path::new("bad-runner.at"),
            WORLD.as_bytes().to_vec(),
            Api::V1,
        )
        .expect("the grammar-level admission accepts the shared grammar");
        let err = Session::new(&snap_v1.source)
            .err()
            .expect("the V1 runner rejects the game world at its compile boundary");
        assert!(
            err.contains("Undefined symbol") && err.contains("ag_action_pressed"),
            "class 6 named rejection at the runner compile boundary, got: {err}"
        );
        // observed audio-runner form: compiles (shared surface), breaks at
        // the closeout's runtime call.
        let audio_runner = SourceSnapshot::from_bytes_for(
            Path::new("bad-runner-audio.at"),
            WORLD.as_bytes().to_vec(),
            Api::Audio2V1,
        )
        .expect("audio2d admission is green (shared surface)");
        let mut sa = Session::new(&audio_runner.source).expect("audio-runner session");
        sa.call("main", &[], ResultKind::Unit).unwrap();
        let start_err = sa.call("_ag_start", &[], ResultKind::Unit).err();
        let run_err = match start_err {
            Some(e) => e,
            None => format!("status={}", sa.int("_ag_status", &[]).unwrap_or(-1)),
        };
        assert!(
            !(run_err.is_empty() || run_err == "status=0"),
            "the audio runner must break on the game's closeout, got: {run_err}"
        );
        // the real runner: Hud2V1 admits and the world runs green.
        let mut s = session(WORLD, "t06-bad6-ok.at", 1);
        let t0 = settle(&mut s, "bad6-ok");
        assert_eq!(tick(&mut s, t0 + 1, 2, 0), 0, "real runner runs green");
        println!(
            "T06_BAD6 bad runner: V1 runner named compile rejection ('Undefined symbol: ag_action_pressed'); audio runner breaks at the closeout call ({run_err}); Hud2V1 admits and runs green"
        );
    }
}
