//! PLAN-072 scratch probe: the real platformer-3d project through the full
//! window-path logic, no OS window. Dumps the composed entry identity, walks
//! the player off the z=0 edge via the Host router, and exercises the
//! end-state fresh-Enter round restart - isolating an engine defect from a
//! player-shell defect for the 3D chain.
use autographics_auto_driver::authoring::{game_input, host::Host, storage::Storage};
use std::time::Instant;

const SCENE: &str = "D:/autostack/ag-examples/games/platformer-3d/generated/platformer/scene.agscene";

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

fn tick(h: &mut Host) {
    h.play
        .as_mut()
        .expect("play")
        .dispatch(autographics_auto_driver::authoring::play::Action::default())
        .expect("dispatch");
    settle(h).expect("settle");
}

fn player(h: &Host) -> [f32; 3] {
    h.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .expect("published frame")
        .objects
        .iter()
        .find(|o| o.id == 1)
        .expect("player object")
        .translation
}

#[test]
fn platformer3d_real_game_end_state_enter_restarts_via_host_router() {
    let (doc, assets, code, _) =
        Storage::load_project(std::path::Path::new(SCENE), &Default::default())
            .expect("project loads");
    let mut h = Host::with_project(doc, assets, code).expect("host");
    h.start_selected_play().expect("run starts");
    settle(&mut h).expect("run settles");
    // Composition identity of the 3D assembly.
    let composed = h
        .play
        .as_ref()
        .unwrap()
        .user_source
        .as_ref()
        .map(|s| s.source.clone())
        .unwrap_or_default();
    let start_line = composed
        .lines()
        .find(|l| l.contains("app_start();"))
        .unwrap_or("<missing>");
    let entry_line = composed
        .lines()
        .find(|l| l.contains("app_update(dt)"))
        .unwrap_or("<missing>");
    println!("3D start: {start_line}");
    println!("3D entry: {entry_line}");
    for _ in 0..12 {
        tick(&mut h);
    }
    // The designed death: walk +x (camera-right at yaw 0), jump the x 8..10
    // gap (its under-gap boundary walls are solid), then walk the D+S
    // diagonal into the hazard at (15, 0.5, 4) - trigger r=1.2.
    let d = game_input::ACTION3_RIGHT;
    let s_key = game_input::ACTION3_BACK;
    let jump = game_input::ACTION3_JUMP;
    let n = game_input::ACTION3_RESTART;
    h.play.as_mut().unwrap().char_key(d, true, false);
    let run1 = h.play.as_ref().unwrap().run_id();
    let mut run2 = run1;
    let mut jumped = false;
    let mut diagonal = false;
    let mut died = false;
    for i in 0..800u64 {
        tick(&mut h);
        let p = player(&h);
        if i % 60 == 0 {
            println!("tick {i}: {p:?}");
        }
        if !jumped && p[0] > 6.8 {
            jumped = true;
            h.play.as_mut().unwrap().char_key(jump, true, false);
        }
        if jumped && !diagonal && p[0] > 10.5 && p[1] > 0.0 {
            diagonal = true;
            h.play.as_mut().unwrap().char_key(s_key, true, false);
        }
        // Aim at the hazard line z=4: release S on approach so the remaining
        // walk is straight +x through the trigger's r=1.2.
        if diagonal && p[2] > 3.55 {
            h.play.as_mut().unwrap().char_key(s_key, false, false);
        }
        if diagonal && !died && p[0] > 13.0 && p[2] > 2.5 {
            died = true;
            println!("hazard zone reached at tick {i}: {p:?}");
        }
        if died && i % 60 == 0 {
            // A fresh end-state press each window; presses while alive are
            // ignored by the live branch, so this is safe before the kill.
            h.play.as_mut().unwrap().char_key(n, true, false);
            let rounds = h
                .play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .and_then(|f| f.rounds.as_ref());
            println!(
                "tick {i}: rounds={:?} state={:?} err={:?}",
                rounds.map(|r| (r.restart_requested, r.restart_tick)),
                h.play.as_ref().unwrap().state,
                h.play.as_ref().unwrap().error
            );
        }
        if died && h.play.as_ref().unwrap().run_id() > run1 {
            run2 = h.play.as_ref().unwrap().run_id();
            break;
        }
    }
    println!("jumped {jumped} diagonal {diagonal} died {died}; run1 {run1} -> run2 {run2}");
    assert!(died, "the walk must reach the hazard");
    assert!(run2 > run1, "fresh Enter at the end state must open a new round");
    let p = player(&h);
    println!("respawn: {p:?}");
    assert!(
        (p[2] - 2.0).abs() < 0.7 && (p[0] - 2.0).abs() < 0.7,
        "the new round respawns at the original spawn"
    );
}
