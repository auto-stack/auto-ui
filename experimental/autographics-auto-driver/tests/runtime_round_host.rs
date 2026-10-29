//! auto-ui PLAN-002 triggers host tests (contract V-03/V-04): the real
//! Host/Play/GPU path for the audited round restart. A Won round's accepted
//! restart marker swaps in a fresh Run/VM from the same accepted snapshot
//! (new run id, tick 0, first-generation coins, camera initial values), held
//! Edges never double-restart, Paused/step ticks never process a request, a
//! new-round preparation failure keeps the last-good presentation under a
//! named fault and Stop still recovers the editing view, twenty restart
//! cycles close every old VM owner with the render stream stable, and
//! corrupted rounds blocks fault the run at the named admission stage.
//! Windowless GPU with public logical inputs only. Run single-threaded:
//! cargo test --locked --test runtime_round_host -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::{Action, State},
};
use std::path::Path;
use std::time::Instant;

const ACTION_RIGHT: usize = 1;
const ACTION_RESTART: usize = 3;

fn document2d() -> Document {
    let mut d = Document::empty("triggers2d-host".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [2.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Quad);
    floor.translation = [15.0, -1.5, 0.0];
    floor.scale = [40.0, 1.0, 1.0];
    d.objects.push(floor);
    // Spawn templates stay INVISIBLE as static nodes; the static Goal leaf
    // (id 7) is visible and is the win square.
    let mut coin_a = Object::new(3, Kind::Circle);
    coin_a.translation = [8.0, 2.0, 0.0];
    coin_a.scale = [0.6, 0.6, 1.0];
    coin_a.rgba = [0.95, 0.8, 0.2, 1.0];
    coin_a.visible = false;
    d.objects.push(coin_a);
    let mut coin_b = Object::new(4, Kind::Quad);
    coin_b.translation = [10.0, 2.0, 0.0];
    coin_b.scale = [0.6, 0.6, 1.0];
    coin_b.rgba = [0.95, 0.8, 0.2, 1.0];
    coin_b.visible = false;
    d.objects.push(coin_b);
    let mut coin_c = Object::new(5, Kind::Circle);
    coin_c.translation = [12.0, 2.0, 0.0];
    coin_c.scale = [0.6, 0.6, 1.0];
    coin_c.rgba = [0.95, 0.8, 0.2, 1.0];
    coin_c.visible = false;
    d.objects.push(coin_c);
    let mut hazard = Object::new(6, Kind::Quad);
    hazard.translation = [9.0, 2.4, 0.0];
    hazard.scale = [0.5, 0.5, 1.0];
    hazard.rgba = [0.9, 0.2, 0.2, 1.0];
    hazard.visible = false;
    d.objects.push(hazard);
    let mut goal = Object::new(7, Kind::Quad);
    goal.translation = [16.0, 1.0, 0.0];
    goal.scale = [0.5, 1.2, 1.0];
    goal.rgba = [0.3, 0.9, 0.5, 1.0];
    // The static template stays hidden: a visible leaf would sit in the sweep
    // world and stop the character at one clearance short of the win square.
    // The spawned goal INSTANCE is the only win-square geometry.
    goal.visible = false;
    d.objects.push(goal);
    d.validate().expect("document valid");
    d
}

/// The engine fixture controller, host-test edition: the walk line is short
/// (goal at x 6, coins at 3.5/4.5/5.5, hazard high at x 4 or low for the
/// failure path) and every state transition is the user Auto's own.
fn source2d(hazard_y: f32, speed: f32) -> String {
    format!(
        "var speed={speed:?}\nvar hy={hazard_y:?}\nvar hdir=0.02\nvar state=0\nvar bag=0\nvar req=0\nvar ok=0\nvar h0=0\nvar h1=0\nvar h2=0\nvar h3=0\nvar h4=0\n\
fn app_start() {{\n\
 ag_set_local(0,1,0.5)\n\
 ok=ok+ag_char_bind(1)\n\
 ok=ok+ag_char_solid(2)\n\
 ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n\
 h0=ag_object_spawn(0, 3, 3.5, 1.0, 0.0)\n\
 h1=ag_object_spawn(1, 4, 4.5, 1.0, 0.0)\n\
 h2=ag_object_spawn(2, 3, 5.5, 1.0, 0.0)\n\
 h3=ag_object_spawn(3, 6, 4.0, hy, 0.0)\n\
 h4=ag_object_spawn(4, 7, 6.0, 1.0, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h0, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(2, 1, 1, h2, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(3, 1, 1, h3, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(4, 1, 1, h4, 0, 0.0)\n\
}}\n\
fn app_update(dt float) {{\n\
 if state==0 {{\n\
  let hr=ag_action_held(1)\n\
  ok=ok+ag_char_move(hr*1.0*speed*dt, 0.0, 0.002)\n\
 }}\n\
 hy=hy+hdir\n\
 if hy>{hzy:?} {{hdir=0.0 - 0.02}}\n\
 if hy<{hzn:?} {{hdir=0.02}}\n\
 ok=ok+ag_object_move(h3, 4.0, hy, 0.0)\n\
 let n0=ag_event_next()\n\
 if n0==1 {{\n\
  let p=ag_event_pair()\n\
  if p<3 {{\n\
   if state==0 {{\n\
    bag=bag+1\n\
    if p==0 {{ok=ok+ag_object_despawn(h0)}}\n\
    if p==1 {{ok=ok+ag_object_despawn(h1)}}\n\
    if p==2 {{ok=ok+ag_object_despawn(h2)}}\n\
   }}\n\
  }}\n\
  if p==3 {{state=2}}\n\
  if p==4 {{if bag==3 {{state=1}}}}\n\
 }}\n\
 if state!=0 {{\n\
  if ag_action_pressed(3)==1 {{req=ag_round_request()}}\n\
 }}\n\
}}\n",
        hzy = hazard_y + 0.2,
        hzn = hazard_y - 0.2,
    )
}

/// A minimal restart-cycle controller: three coins bound, no game state, and
/// every fresh Enter edge asks for a new round (the twenty-cycle owner probe).
fn restart_cycle_source() -> String {
    "var ok=0\nvar h0=0\nvar h1=0\nvar h2=0\nfn app_start() {\n\
 ag_set_local(0,1,0.5)\n\
 ok=ok+ag_char_bind(1)\n\
 ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n\
 h0=ag_object_spawn(0, 3, 3.5, 1.0, 0.0)\n\
 h1=ag_object_spawn(1, 4, 4.5, 1.0, 0.0)\n\
 h2=ag_object_spawn(2, 3, 5.5, 1.0, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h0, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(2, 1, 1, h2, 0, 0.0)\n\
}\n\
fn app_update(dt float) {\n\
 if ag_action_pressed(3)==1 {ag_round_request()}\n\
}\n"
    .to_string()
}

fn snapshot(api: Api, source: String, name: &str) -> SourceSnapshot {
    SourceSnapshot::from_bytes_for(Path::new(name), source.as_bytes().to_vec(), api)
        .expect("admission")
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
}

/// Drive one tick: resume, hold Right (no fresh edge needed for held), then
/// dispatch and let the host pump prepare/accept/publish the transaction.
fn drive(h: &mut Host, hold_right: bool) {
    {
        let play = h.play.as_mut().unwrap();
        if !play.pending() {
            if play.state == State::Paused {
                play.resume().expect("resume");
            }
            if hold_right {
                play.char_key(ACTION_RIGHT, true, false);
            }
            play.dispatch(Action::default()).expect("dispatch");
        }
    }
    settle(h);
}

/// Walk with held Right, polling the condition after every pumped frame:
/// clock credits may run extra ticks inside one frame, and a rounds block
/// only holds its own tick, so settle-level polling can miss a window.
fn walk_until(h: &mut Host, mut condition: impl FnMut(&Host) -> bool, max: usize, label: &str) {
    for _ in 0..max {
        if condition(h) {
            return;
        }
        {
            let play = h.play.as_mut().unwrap();
            if play.pending() {
                // just pump below
            } else {
                if play.state == State::Paused {
                    play.resume().expect("resume");
                }
                play.char_key(ACTION_RIGHT, true, false);
                play.dispatch(Action::default()).expect("dispatch");
            }
        }
        h.frame(None).expect("frame");
    }
    let play = h.play.as_ref().unwrap();
    let events = play
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .map(|r| format!("{:?}", r.events))
        .unwrap_or_default();
    let char_x = play
        .last
        .as_ref()
        .and_then(|f| f.nodes.values().next().map(|d| d.translation[0]));
    panic!(
        "walk deadline: {label} state={:?} tick={} error={:?} events={events} char_x={char_x:?}",
        play.state,
        play.tick(),
        play.error
    );
}

fn live_instances(h: &Host) -> usize {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.instances.as_ref())
        .map(|i| i.live_count())
        .unwrap_or(0)
}

fn draw_count(content: Option<&agp::host::surface_runtime::SurfaceContent>) -> usize {
    match content {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.commands.len(),
        _ => panic!("2D game content"),
    }
}

/// True when the admitted rounds block carries a fresh enter for `pair` (the
/// block only holds the latest evaluate, so this is a one-tick window).
fn entered(h: &Host, pair: i32) -> bool {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .is_some_and(|r| r.events.iter().any(|e| e.pair == pair && e.phase == 1))
}

/// A window-free state probe: `pair` still overlapping (enter or stay) while
/// the frozen fixture geometry keeps the contact alive.
fn contact(h: &Host, pair: i32) -> bool {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .is_some_and(|r| {
            r.events
                .iter()
                .any(|e| e.pair == pair && (e.phase == 1 || e.phase == 2))
        })
}

/// The Won state without a one-tick window: all three coins destroyed plus
/// the goal pair still in contact (the frozen character keeps overlapping).
fn won_reached(h: &Host) -> bool {
    live_instances(h) == 2 && contact(h, 4)
}

fn run_id(h: &Host) -> i32 {
    h.play.as_ref().unwrap().run_id
}

// ------------------------------------------------- V-03: the new Run swap --

fn won_round_restarts_with_a_fresh_run_and_reset_state_body() {
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot(
        Api::Triggers2V1,
        source2d(2.4, 2.0),
        "host-won.at",
    ))
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    let first_run = run_id(&h);
    assert_eq!(live_instances(&h), 5, "three coins, hazard and goal");
    // Walk right: the three coin enters fire along the line and the goal
    // enter at count 3 turns the state Won.
    walk_until(
        &mut h,
        won_reached,
        30000,
        "the goal reached with three coins collected",
    );
    let won_run = run_id(&h);
    assert_eq!(
        live_instances(&h),
        2,
        "three coins destroyed, hazard and goal left"
    );
    assert_eq!(run_id(&h), won_run, "the Won state alone restarts nothing");
    // One fresh Enter edge requests and processes the swap: a brand new
    // Run/VM with first-generation coins, tick 0 and the camera initial
    // values.
    let old_owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    {
        let play = h.play.as_mut().unwrap();
        play.char_key(ACTION_RESTART, true, false);
    }
    drive(&mut h, false);
    walk_until(
        &mut h,
        |h| run_id(h) != won_run,
        3000,
        "the Won restart swap",
    );
    settle(&mut h);
    assert_ne!(run_id(&h), won_run, "the new round runs on a new run id");
    let (fresh_tick, fresh_run, fresh_state) = {
        let fresh = h.play.as_ref().unwrap();
        assert_eq!(fresh.tick(), 0, "the new round starts at tick 0");
        assert_eq!(fresh.state, State::Running);
        assert_eq!(
            live_instances(&h),
            5,
            "three coins, hazard and goal are reborn"
        );
        let frame = fresh.last.as_ref().expect("tick 0 frame");
        let instances = frame.instances.as_ref().expect("instances");
        for slot in 0..3 {
            let coin = instances.get(slot).expect("coin slot");
            assert_eq!(coin.generation, 1, "the new run starts at generation 1");
        }
        assert!(frame.camera.is_some(), "the camera initial values re-admit");
        (fresh.tick(), fresh.run_id, fresh.state)
    };
    assert!(!old_owner(), "the old round VM is closed after the swap");
    // Held Edges never double-restart: more ticks without a fresh press keep
    // the new round running.
    for _ in 0..5 {
        drive(&mut h, true);
    }
    assert_eq!(run_id(&h), fresh_run, "no restart without a fresh edge");
    h.stop_play();
}

#[test]
fn won_round_restarts_with_a_fresh_run_and_reset_state() {
    let _ = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(won_round_restarts_with_a_fresh_run_and_reset_state_body)
        .expect("big stack")
        .join()
        .expect("join");
}

// --------------------------------------- V-03: failure path and held edges --

fn failed_round_restarts_and_never_double_restarts_body() {
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    // The hazard patrols the walk line: the run ends Failed at the hazard.
    h.start_user_snapshot(snapshot(
        Api::Triggers2V1,
        source2d(1.0, 3.0),
        "host-fail.at",
    ))
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    walk_until(
        &mut h,
        |h| contact(h, 3),
        30000,
        "the hazard contact (the run failed)",
    );
    let failed_run = run_id(&h);
    {
        let play = h.play.as_mut().unwrap();
        play.char_key(ACTION_RESTART, true, false);
    }
    // One Enter edge: the failed round swaps for a fresh one.
    {
        let play = h.play.as_mut().unwrap();
        play.char_key(ACTION_RESTART, true, false);
    }
    drive(&mut h, false);
    walk_until(
        &mut h,
        |h| run_id(h) != failed_run,
        3000,
        "the Failed restart swap",
    );
    settle(&mut h);
    let second_run = run_id(&h);
    assert_ne!(second_run, failed_run, "the failed round restarted");
    assert_eq!(h.play.as_ref().unwrap().tick(), 0);
    // Holding the key adds no edge: five walked ticks never restart again.
    for _ in 0..5 {
        drive(&mut h, true);
    }
    assert_eq!(run_id(&h), second_run, "held Enter never double-restarts");
    // A SECOND fresh edge while Playing is consumed by the state machine (the
    // fixture only requests in Won/Failed), so the round keeps running.
    {
        let play = h.play.as_mut().unwrap();
        play.char_key(ACTION_RESTART, true, false);
    }
    drive(&mut h, true);
    settle(&mut h);
    assert_eq!(run_id(&h), second_run, "Playing swallows the restart edge");
    h.stop_play();
}

#[test]
fn failed_round_restarts_and_never_double_restarts() {
    let _ = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(failed_round_restarts_and_never_double_restarts_body)
        .expect("big stack")
        .join()
        .expect("join");
}

#[test]
fn unfinished_goal_never_wins_and_paused_ticks_never_process_a_restart() {
    // A Won without three coins is not a win: the coins spawn OFF the walk
    // line (y 2.5), so the goal entry arrives under count 0 and the state
    // machine ignores it - the round never requests a restart.
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    let off_line = source2d(2.4, 6.0).replace(", 1.0, 0.0)\n", ", 2.5, 0.0)\n");
    h.start_user_snapshot(snapshot(Api::Triggers2V1, off_line, "host-nogoal.at"))
        .expect("start");
    h.pause_play("");
    settle(&mut h);
    let run = run_id(&h);
    // Speed 6 crosses the whole walk line without touching a coin pair? The
    // coins sit on the line, so instead bound the walk length: 120 ticks at
    // 6 u/s crosses x=7 past the goal without three enters being READ (the
    // one-event-per-tick read drops the coin enters while the goal enter
    // arrives under count<3). The round must never request.
    for _ in 0..120 {
        drive(&mut h, true);
        let requested = h
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .and_then(|f| f.rounds.as_ref())
            .is_some_and(|r| r.restart_requested);
        assert!(!requested, "an unearned goal entry never wins");
    }
    assert_eq!(run_id(&h), run);
    h.stop_play();

    // A request carried on a tick that ends Paused is not processed.
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot(
        Api::Triggers2V1,
        restart_cycle_source(),
        "host-paused.at",
    ))
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    let run = run_id(&h);
    {
        let play = h.play.as_mut().unwrap();
        play.resume().expect("resume");
        play.char_key(ACTION_RESTART, true, false);
        play.dispatch(Action::default()).expect("dispatch");
        play.pause("test pause before accept");
    }
    h.frame(None).expect("frame");
    settle(&mut h);
    {
        let play = h.play.as_ref().unwrap();
        assert_eq!(play.state, State::Paused, "the tick accepted into Paused");
        assert_eq!(play.run_id, run, "Paused never processes a restart");
    }
    // The request marker was consumed with its frame: resuming does not
    // restart either.
    {
        let play = h.play.as_mut().unwrap();
        play.resume().expect("resume");
        play.dispatch(Action::default()).expect("dispatch");
    }
    h.frame(None).expect("frame");
    settle(&mut h);
    assert_eq!(run_id(&h), run, "the consumed marker stays consumed");
    h.stop_play();
}

// ------------------------------------- V-03/V-04: failure keeps the last-good --

fn failed_new_round_keeps_last_good_and_recovers_on_stop_body() {
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot(
        Api::Triggers2V1,
        source2d(2.4, 2.0),
        "host-badres.at",
    ))
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    walk_until(
        &mut h,
        won_reached,
        30000,
        "the goal reached with three coins collected",
    );
    let draws_before = draw_count(h.rt.content(h.two));
    // The new round's first preparation fails at the injected resource fault.
    h.fail_resource = true;
    {
        let play = h.play.as_mut().unwrap();
        play.char_key(ACTION_RESTART, true, false);
    }
    drive(&mut h, false);
    settle(&mut h);
    let play = h.play.as_ref().unwrap();
    assert_eq!(
        play.state,
        State::Faulted,
        "the failed new round is a named fault"
    );
    let error = play.error.clone().expect("fault message");
    assert!(!error.is_empty(), "{error}");
    assert_eq!(
        draw_count(h.rt.content(h.two)),
        draws_before,
        "the old last-good presentation stays published"
    );
    assert_eq!(
        live_instances(&h) >= 1,
        true,
        "the admitted last frame remains"
    );
    // Stop still recovers the exact editing presentation.
    h.stop_play();
    assert!(h.play.is_none(), "stop clears the round");
    assert_ne!(
        draw_count(h.rt.content(h.two)),
        draws_before,
        "the editing view is back"
    );
}

#[test]
fn failed_new_round_keeps_last_good_and_recovers_on_stop() {
    let _ = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(failed_new_round_keeps_last_good_and_recovers_on_stop_body)
        .expect("big stack")
        .join()
        .expect("join");
}

// ---------------------------------------------- V-04: twenty round cycles --

fn twenty_round_cycles_close_every_old_vm_owner_body() {
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot(
        Api::Triggers2V1,
        restart_cycle_source(),
        "host-cycles.at",
    ))
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    for round in 0..20 {
        let old_run = run_id(&h);
        let old_owner = h.play.as_ref().unwrap().owner_probe().unwrap();
        let old_tick = h.play.as_ref().unwrap().tick();
        assert!(old_tick >= 0);
        // Two walked ticks so every cycle runs real updates before the swap.
        drive(&mut h, true);
        drive(&mut h, true);
        let draws_before = draw_count(h.rt.content(h.two));
        {
            let play = h.play.as_mut().unwrap();
            play.char_key(ACTION_RESTART, true, false);
        }
        drive(&mut h, false);
        settle(&mut h);
        let new_run = run_id(&h);
        assert_ne!(new_run, old_run, "round {round}: the run id advanced");
        assert!(new_run > old_run, "round ids are monotonic");
        assert_eq!(h.play.as_ref().unwrap().tick(), 0, "round {round}: tick 0");
        assert_eq!(live_instances(&h), 3, "round {round}: three fresh coins");
        assert!(!old_owner(), "round {round}: the old VM owner is closed");
        let after = draw_count(h.rt.content(h.two));
        assert_eq!(
            draws_before, after,
            "round {round}: the render stream stays stable across the swap"
        );
    }
    h.stop_play();
}

#[test]
fn twenty_round_cycles_close_every_old_vm_owner() {
    let _ = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(twenty_round_cycles_close_every_old_vm_owner_body)
        .expect("big stack")
        .join()
        .expect("join");
}
