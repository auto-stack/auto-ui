//! PLAN-861 character/1 action-input tests (contract V-02) through the real
//! Host/Play dispatch path: edge consume-once, OS-repeat suppression,
//! pause/blur clearing, neutral Step and cross-Run freshness. Run
//! single-threaded: cargo test --locked --test game_input -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    game_input,
    host::Host,
    play::Action,
};
use std::path::Path;
use std::time::Instant;

/// Character (stable 1) + floor (stable 2) + marker quad (stable 3, decor).
/// The user rule moves the marker +1 per consumed Jump edge and drives the
/// character x by the held MoveRight action, so both input paths are
/// observable through published object translations.
const USER_SOURCE: &str = r#"var ok=0
var grounded=0
var mx=0.0
fn app_start() {
 ok=ok+ag_char_bind(1)
 ok=ok+ag_char_solid(2)
}
fn app_update(dt float) {
 if ag_action_pressed(2)==1 {mx=mx+1.0;ag_set_local(2,0,mx)}
 if ag_action_pressed(3)==1 {mx=mx+1.0;ag_set_local(2,0,mx)}
 let hr=ag_action_held(1)
 let q=ag_char_move(hr*4.0*dt,0.0,0.002)
 grounded=ag_char_grounded(q)
}
"#;

fn document() -> Document {
    let mut d = Document::empty("character-input".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [0.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Quad);
    floor.translation = [0.0, -0.5, 0.0];
    floor.scale = [16.0, 1.0, 1.0];
    d.objects.push(floor);
    let mut marker = Object::new(3, Kind::Quad);
    marker.translation = [-6.0, 2.0, 0.0];
    marker.scale = [0.5, 0.5, 1.0];
    d.objects.push(marker);
    d.validate().expect("document valid");
    d
}

fn host_with_run() -> Result<Host, String> {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("game-input.at"),
        USER_SOURCE.as_bytes().to_vec(),
        Api::CharacterV1,
    )?;
    let mut h = Host::with_project(document(), Default::default(), None)?;
    h.start_user_snapshot(snapshot)?;
    Ok(h)
}

fn settle(h: &mut Host) -> Result<(), String> {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().ok_or("missing Play")?.pending() {
        if Instant::now() >= end {
            return Err("host initialization deadline 30s".into());
        }
        h.frame(None)?;
    }
    Ok(())
}

fn translation(h: &Host, object: usize) -> [f32; 3] {
    h.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .map(|f| f.objects[object].translation)
        .expect("published frame")
}

fn dispatch_tick(h: &mut Host, step: usize) {
    h.play
        .as_mut()
        .expect("play")
        .dispatch(Action::default())
        .expect("dispatch");
    settle(h).expect("settle");
    let _ = step;
}

const J: usize = game_input::ACTION_JUMP;
const R: usize = game_input::ACTION_RIGHT;
const N: usize = game_input::ACTION_RESTART;

#[test]
fn os_repeat_creates_no_edge_and_one_dispatch_consumes_the_fold_once() {
    let mut h = host_with_run().unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    // OS auto-repeat presses create nothing.
    h.play.as_mut().unwrap().char_key(J, true, true);
    h.play.as_mut().unwrap().char_key(J, true, true);
    dispatch_tick(&mut h, 1);
    assert_eq!(translation(&h, 2)[0], -6.0);
    // One real press: the counter moves to exactly 1 and never re-triggers.
    h.play.as_mut().unwrap().char_key(J, true, false);
    h.play
        .as_mut()
        .unwrap()
        .char_key(game_input::ACTION_LEFT, true, false);
    dispatch_tick(&mut h, 2);
    assert_eq!(translation(&h, 2)[0], 1.0);
    dispatch_tick(&mut h, 3);
    assert_eq!(translation(&h, 2)[0], 1.0);
    // Two presses folding into one dispatch still consume as one edge.
    h.play.as_mut().unwrap().char_key(J, false, false);
    h.play.as_mut().unwrap().char_key(J, true, false);
    h.play.as_mut().unwrap().char_key(J, false, false);
    h.play.as_mut().unwrap().char_key(J, true, false);
    dispatch_tick(&mut h, 4);
    assert_eq!(translation(&h, 2)[0], 2.0);
    dispatch_tick(&mut h, 5);
    assert_eq!(translation(&h, 2)[0], 2.0);
    // Restart edges reach user rules the same way.
    h.play.as_mut().unwrap().char_key(N, true, false);
    dispatch_tick(&mut h, 6);
    assert_eq!(translation(&h, 2)[0], 3.0);
}

#[test]
fn held_moves_the_character_and_release_stops_it() {
    let mut h = host_with_run().unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    let x0 = translation(&h, 0)[0];
    h.play.as_mut().unwrap().char_key(R, true, false);
    for step in 1..=5 {
        dispatch_tick(&mut h, step);
    }
    let x_moved = translation(&h, 0)[0];
    // Held dispatches move the character (wall-clock credit ticks between
    // manual dispatches make the exact count nondeterministic).
    assert!(
        x_moved > x0 + 3.0 * 4.0 / 60.0,
        "held moved x {x0} -> {x_moved}"
    );
    h.play.as_mut().unwrap().char_key(R, false, false);
    dispatch_tick(&mut h, 6);
    assert_eq!(translation(&h, 0)[0], x_moved);
}

#[test]
fn pause_clears_pending_edges_and_blur_clears_held() {
    let mut h = host_with_run().unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    h.play.as_mut().unwrap().char_key(J, true, false);
    // Pause consumes the whole input state (edge included); resume needs a
    // fresh press.
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    let marker_before = translation(&h, 2)[0];
    dispatch_tick(&mut h, 1);
    assert!((translation(&h, 2)[0] - marker_before).abs() < 1e-6);
    // Held keys survive... no: focus loss (blur) clears held too. Re-press
    // after blur, then resume: the held state was cleared with the pause, so
    // a press during Paused tracks held without an edge, and the character
    // only moves once dispatches observe the tracked held key.
    h.play.as_mut().unwrap().char_key(R, true, false);
    h.blur();
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    let x0 = translation(&h, 0)[0];
    dispatch_tick(&mut h, 2);
    assert_eq!(translation(&h, 0)[0], x0, "blur cleared held right");
}

#[test]
fn neutral_step_is_all_neutral_while_keys_are_tracked() {
    let mut h = host_with_run().unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    // A press during Paused tracks held without an edge.
    h.play.as_mut().unwrap().char_key(R, true, false);
    h.play.as_mut().unwrap().single_step().unwrap();
    settle(&mut h).unwrap();
    let x0 = translation(&h, 0)[0];
    let marker0 = translation(&h, 2)[0];
    // The neutral Step itself must not move anything.
    h.play.as_mut().unwrap().single_step().unwrap();
    settle(&mut h).unwrap();
    assert_eq!(translation(&h, 0)[0], x0);
    assert_eq!(translation(&h, 2)[0], marker0);
    // Resume clears the whole input state (contract section 3); a fresh
    // press after resume is what drives the next dispatch.
    h.play.as_mut().unwrap().resume().unwrap();
    dispatch_tick(&mut h, 1);
    assert_eq!(translation(&h, 0)[0], x0, "resume cleared tracked held");
    h.play.as_mut().unwrap().char_key(R, true, false);
    dispatch_tick(&mut h, 2);
    assert!(
        translation(&h, 0)[0] > x0,
        "fresh press drives the character"
    );
}

#[test]
fn non_character_runs_ignore_char_keys_and_stay_on_the_legacy_path() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("legacy.at"),
        b"fn app_start(){}\nfn app_update(dt float){}".to_vec(),
        Api::MotionV5,
    )
    .unwrap();
    let mut h = Host::with_project(document(), Default::default(), None).unwrap();
    h.start_user_snapshot(snapshot).unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    assert!(!h.play.as_ref().unwrap().character_run());
    // char_key is a no-op on legacy runs; the tick still dispatches cleanly.
    h.play.as_mut().unwrap().char_key(J, true, false);
    dispatch_tick(&mut h, 1);
    assert!(h.play.as_ref().unwrap().error.is_none());
}

#[test]
fn fresh_run_starts_from_the_document_and_old_input_is_not_inherited() {
    let mut h = host_with_run().unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    h.play.as_mut().unwrap().char_key(J, true, false);
    dispatch_tick(&mut h, 1);
    assert_eq!(translation(&h, 2)[0], 1.0);
    h.stop_play();
    h.frame(None).unwrap();
    // A new Run rebuilds VM/user state: the marker returns to the document
    // position and no edge survives.
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("game-input.at"),
        USER_SOURCE.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .unwrap();
    h.start_user_snapshot(snapshot).unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.play.as_mut().unwrap().resume().unwrap();
    dispatch_tick(&mut h, 1);
    assert!(
        (translation(&h, 2)[0] - (-6.0)).abs() < 1e-6,
        "fresh marker"
    );
    assert!((translation(&h, 0)[0] - 0.0).abs() < 1e-6, "fresh spawn");
}
