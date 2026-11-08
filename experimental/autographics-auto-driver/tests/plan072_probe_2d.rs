//! PLAN-072 scratch probe: the real platformer-2d project through the full
//! window-path logic (Host router -> GameInput -> dispatch -> VM), no OS
//! window. Isolates a player-shell defect from an engine defect.
use autographics_auto_driver::authoring::{game_input, host::Host, storage::Storage};
use std::time::Instant;

const SCENE: &str = "D:/autostack/ag-examples/games/platformer-2d/generated/platformer/scene.agscene";

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

fn hero(h: &Host) -> [f32; 3] {
    h.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .expect("published frame")
        .objects
        .iter()
        .find(|o| o.id == 1)
        .expect("hero object")
        .translation
}

fn tick(h: &mut Host) {
    h.play
        .as_mut()
        .expect("play")
        .dispatch(autographics_auto_driver::authoring::play::Action::default())
        .expect("dispatch");
    settle(h).expect("settle");
}

#[test]
fn platformer2d_real_game_pressed_and_held_via_host_router() {
    let (doc, assets, code, _) =
        Storage::load_project(std::path::Path::new(SCENE), &Default::default())
            .expect("project loads");
    let mut h = Host::with_project(doc, assets, code).expect("host");
    h.start_selected_play().expect("run starts");
    settle(&mut h).expect("run settles");
    for _ in 0..12 {
        tick(&mut h);
    }
    let rest = hero(&h);
    println!("rest: {rest:?}");
    // One real jump edge through the router.
    let j = game_input::ACTION_JUMP;
    h.play.as_mut().unwrap().char_key(j, true, false);
    tick(&mut h);
    let mut apex = rest[1];
    for _ in 0..60 {
        tick(&mut h);
        apex = apex.max(hero(&h)[1]);
    }
    println!("apex: {apex} (rest {})", rest[1]);
    assert!(apex > rest[1] + 0.4, "jump must lift the hero");
    // Held right through the router.
    let r = game_input::ACTION_RIGHT;
    h.play.as_mut().unwrap().char_key(r, true, false);
    let x0 = hero(&h)[0];
    for _ in 0..30 {
        tick(&mut h);
    }
    let x1 = hero(&h)[0];
    println!("x: {x0} -> {x1}");
    assert!(x1 > x0 + 1.0, "held right must move the hero");
}
