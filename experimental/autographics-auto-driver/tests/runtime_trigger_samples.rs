//! auto-ui PLAN-002 triggers sample acceptance (contract V-05 engine scope):
//! both dimensions' engine fixtures import through the production loader,
//! their public sources admit on the triggers profiles, the real Auto plays
//! the whole loop through the Host/GPU path (walk, three single-count coin
//! collections, the goal win, one fresh Enter edge, and the audited restart
//! with first-generation coins), the parameter-edited twin fails at the
//! hazard on a different trajectory (parameter sensitivity), and removing a
//! spawn changes the actual admitted template identity and the published
//! content (frozen-input mutation). Windowless GPU with public logical
//! inputs only. Run single-threaded:
//! cargo test --locked --test runtime_trigger_samples -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::Kind,
    host::Host,
    play::{Action, State},
};
use std::path::{Path, PathBuf};
use std::time::Instant;

const ACTION_RESTART: usize = 3;

fn sample(dim: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/runtime-triggers")
        .join(dim)
        .join(name)
}

fn source_of(dim: &str, name: &str) -> String {
    std::fs::read_to_string(sample(dim, name)).expect("sample source")
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
}

/// One walked tick: dispatch when idle, then pump exactly one frame so the
/// caller polls the per-tick rounds block (clock credits may run extra ticks
/// inside a frame, and a block only holds its own tick).
fn walked_frame(h: &mut Host, entered: &mut dyn FnMut(&Host) -> bool) -> bool {
    if entered(h) {
        return true;
    }
    {
        let play = h.play.as_mut().unwrap();
        if !play.pending() {
            if play.state == State::Paused {
                play.resume().expect("resume");
            }
            play.char_key(1, true, false);
            play.dispatch(Action::default()).expect("dispatch");
        }
    }
    h.frame(None).expect("frame");
    entered(h)
}

fn entered_pair(pair: i32) -> impl FnMut(&Host) -> bool {
    move |h: &Host| {
        h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .and_then(|f| f.rounds.as_ref())
            .is_some_and(|r| r.events.iter().any(|e| e.pair == pair && e.phase == 1))
    }
}

fn run_id(h: &Host) -> i32 {
    h.play.as_ref().unwrap().run_id
}

fn char_x(h: &Host) -> f32 {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.nodes.values().next().map(|d| d.translation[0]))
        .expect("character node")
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

fn draw_count(h: &Host) -> usize {
    match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.commands.len(),
        _ => panic!("2D game content"),
    }
}

fn circle_count(h: &Host) -> usize {
    match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g
            .commands
            .iter()
            .filter(|c| c.shape == agp::renderer::pipeline::Shape::Circle)
            .count(),
        _ => panic!("2D game content"),
    }
}

fn start_sample(h: &mut Host, dim: &str, name: &str, api: Api, source: &str) {
    let scene = autographics_auto_driver::authoring::storage::Storage::load(&sample(
        dim,
        "layout_a.agscene",
    ))
    .expect("layout import")
    .0;
    *h = Host::with_project(scene, Default::default(), None).expect("host");
    let snap = SourceSnapshot::from_bytes_for(Path::new(name), source.as_bytes().to_vec(), api)
        .expect("sample admits");
    h.start_user_snapshot(snap).expect("start");
    h.pause_play("");
    settle(h);
}

fn big_stack(body: impl FnOnce() + Send + 'static) {
    let _ = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(body)
        .expect("big stack")
        .join()
        .expect("join");
}

#[test]
fn fixtures_import_and_play_the_full_round_loop_in_both_dimensions() {
    big_stack(|| {
        // The layouts import with their frozen template sets.
        let two = autographics_auto_driver::authoring::storage::Storage::load(&sample(
            "two",
            "layout_a.agscene",
        ))
        .expect("2D layout import")
        .0;
        assert_eq!(two.objects.len(), 7);
        assert_eq!(two.objects[2].kind, Kind::Circle, "the CoinA template");
        assert_eq!(two.objects[4].kind, Kind::Circle, "the CoinC template");
        assert_eq!(two.objects[6].kind, Kind::Quad, "the Goal template");
        let three = autographics_auto_driver::authoring::storage::Storage::load(&sample(
            "three",
            "layout_a.agscene",
        ))
        .expect("3D layout import")
        .0;
        assert_eq!(three.objects.len(), 7);
        assert_eq!(three.objects[2].kind, Kind::Mob, "the ShardA template");

        // 2D: the public sample admits and plays the whole loop.
        let mut h = Host::with_project(two, Default::default(), None).expect("host");
        start_sample(
            &mut h,
            "two",
            "two-main.at",
            Api::Triggers2V1,
            &source_of("two", "main.at"),
        );
        assert_eq!(live_instances(&h), 5, "three coins, hazard and goal");
        let first_run = run_id(&h);
        let mut won = false;
        for _ in 0..40000 {
            if won_reached(&h) {
                won = true;
                break;
            }
            walked_frame(&mut h, &mut |_h| false);
        }
        if !won {
            let play = h.play.as_ref().unwrap();
            panic!(
                "2D sample deadline: state={:?} tick={} run={} error={:?} char_x={:?} live={}",
                play.state,
                play.tick(),
                play.run_id,
                play.error,
                char_x(&h),
                live_instances(&h)
            );
        }
        assert_eq!(run_id(&h), first_run, "the win alone restarts nothing");
        assert_eq!(live_instances(&h), 2, "three coins destroyed");
        // Fresh Enter edges until the swap lands (the fixture reads one edge
        // per tick in the Won state; a credit tick may consume one).
        let mut swapped = false;
        for _ in 0..3 {
            {
                let play = h.play.as_mut().unwrap();
                play.char_key(ACTION_RESTART, true, false);
            }
            drive_one(&mut h);
            for _ in 0..400 {
                if run_id(&h) != first_run {
                    swapped = true;
                    break;
                }
                settle(&mut h);
            }
            if swapped {
                break;
            }
        }
        if !swapped {
            let play = h.play.as_ref().unwrap();
            panic!(
                "2D restart deadline: state={:?} tick={} run={} first={first_run} error={:?}",
                play.state,
                play.tick(),
                play.run_id,
                play.error
            );
        }
        settle(&mut h);
        settle(&mut h);
        assert_eq!(
            h.play.as_ref().unwrap().tick(),
            0,
            "the new round at tick 0"
        );
        assert_eq!(live_instances(&h), 5, "the coins and goal are reborn");
        h.stop_play();

        // 3D: the same loop on the triggers3d profile.
        let three = autographics_auto_driver::authoring::storage::Storage::load(&sample(
            "three",
            "layout_a.agscene",
        ))
        .expect("3D layout import")
        .0;
        let mut h = Host::with_project(three, Default::default(), None).expect("host");
        start_sample(
            &mut h,
            "three",
            "three-main.at",
            Api::Triggers3V1,
            &source_of("three", "main.at"),
        );
        for _ in 0..50 {
            if live_instances(&h) == 5 {
                break;
            }
            settle(&mut h);
        }
        if live_instances(&h) != 5 {
            let play = h.play.as_ref().unwrap();
            panic!(
                "3D start: state={:?} tick={} error={:?}",
                play.state,
                play.tick(),
                play.error
            );
        }
        assert_eq!(live_instances(&h), 5, "the 3D tick 0 spawns");
        let first_run = run_id(&h);
        let mut won = false;
        for _ in 0..40000 {
            if won_reached(&h) {
                won = true;
                break;
            }
            walked_frame(&mut h, &mut |_h| false);
        }
        if !won {
            let play = h.play.as_ref().unwrap();
            let cx = play
                .last
                .as_ref()
                .and_then(|f| f.nodes.values().next().map(|d| d.translation));
            panic!(
                "3D sample deadline: state={:?} tick={} run={} error={:?} char={cx:?} live={}",
                play.state,
                play.tick(),
                play.run_id,
                play.error,
                live_instances(&h)
            );
        }
        let mut swapped = false;
        for _ in 0..3 {
            {
                let play = h.play.as_mut().unwrap();
                play.char_key(5, true, false);
            }
            drive_one(&mut h);
            for _ in 0..400 {
                if run_id(&h) != first_run {
                    swapped = true;
                    break;
                }
                settle(&mut h);
            }
            if swapped {
                break;
            }
        }
        if !swapped {
            let play = h.play.as_ref().unwrap();
            panic!(
                "3D restart deadline: state={:?} tick={} run={} first={first_run} error={:?}",
                play.state,
                play.tick(),
                play.run_id,
                play.error
            );
        }
        settle(&mut h);
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().tick(), 0, "the 3D round at tick 0");
        assert_eq!(live_instances(&h), 5, "the 3D round is reborn");
        h.stop_play();
    });
}

/// The Won state is observable without a one-tick window: all three coins
/// destroyed plus the goal pair staying in the admitted block (the frozen
/// character keeps overlapping the goal, so the stay persists).
fn won_reached(h: &Host) -> bool {
    live_instances(h) == 2
        && h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .and_then(|f| f.rounds.as_ref())
            .is_some_and(|r| r.events.iter().any(|e| e.pair == 4 && e.phase == 2))
}

fn drive_one(h: &mut Host) {
    {
        let play = h.play.as_mut().unwrap();
        if !play.pending() {
            if play.state == State::Paused {
                play.resume().expect("resume");
            }
            play.dispatch(Action::default()).expect("dispatch");
        }
    }
    settle(h);
}

#[test]
fn parameter_twin_fails_at_the_hazard_on_a_different_trajectory() {
    big_stack(|| {
        let mut h = Host::with_project(
            autographics_auto_driver::authoring::storage::Storage::load(&sample(
                "two",
                "layout_a.agscene",
            ))
            .expect("layout")
            .0,
            Default::default(),
            None,
        )
        .expect("host");
        start_sample(
            &mut h,
            "two",
            "two-alt.at",
            Api::Triggers2V1,
            &source_of("two", "main-alt.at"),
        );
        // The alt run fails at the hazard sitting on the walk line: fewer
        // The alt run fails at the hazard sitting on the walk line: fewer
        // ticks and a different character position than the winning main run.
        // The contact latch is window-free: the patrolling hazard keeps the
        // pair overlapping (enter or stay) while the failed round sits frozen.
        let mut ticks = 0usize;
        let mut hazard_contact = |h: &Host| {
            h.play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .and_then(|f| f.rounds.as_ref())
                .is_some_and(|r| {
                    r.events
                        .iter()
                        .any(|e| e.pair == 3 && (e.phase == 1 || e.phase == 2))
                })
        };
        let mut saw_hazard = false;
        for _ in 0..40000 {
            ticks += 1;
            if hazard_contact(&h) {
                saw_hazard = true;
                break;
            }
            walked_frame(&mut h, &mut hazard_contact);
        }
        if !saw_hazard {
            let play = h.play.as_ref().unwrap();
            panic!(
                "alt deadline: state={:?} tick={} run={} error={:?} char_x={:?}",
                play.state,
                play.tick(),
                play.run_id,
                play.error,
                char_x(&h)
            );
        }
        let failed_x = char_x(&h);
        assert!(
            (7.0..9.5).contains(&failed_x),
            "the failure happens at the x=9 hazard, not at the x=16 goal: {failed_x}"
        );
        // The Failed state reads one fresh Enter edge and restarts.
        let failed_run = run_id(&h);
        let mut swapped = false;
        for _ in 0..3 {
            {
                let play = h.play.as_mut().unwrap();
                play.char_key(ACTION_RESTART, true, false);
            }
            drive_one(&mut h);
            for _ in 0..400 {
                if run_id(&h) != failed_run {
                    swapped = true;
                    break;
                }
                settle(&mut h);
            }
            if swapped {
                break;
            }
        }
        if !swapped {
            let play = h.play.as_ref().unwrap();
            panic!(
                "alt restart deadline: state={:?} tick={} run={} first={failed_run} error={:?}",
                play.state,
                play.tick(),
                play.run_id,
                play.error
            );
        }
        // The winning main run reaches the goal at a strictly different
        // character position: the parameter edit changed the real trajectory.
        let mut h_main = Host::with_project(
            autographics_auto_driver::authoring::storage::Storage::load(&sample(
                "two",
                "layout_a.agscene",
            ))
            .expect("layout")
            .0,
            Default::default(),
            None,
        )
        .expect("host");
        start_sample(
            &mut h_main,
            "two",
            "two-main.at",
            Api::Triggers2V1,
            &source_of("two", "main.at"),
        );
        let mut goal = entered_pair(4);
        let mut saw_goal = false;
        let mut goal_ticks = 0usize;
        for _ in 0..40000 {
            goal_ticks += 1;
            if walked_frame(&mut h_main, &mut goal) {
                saw_goal = true;
                break;
            }
        }
        assert!(saw_goal);
        let goal_x = char_x(&h_main);
        assert!(
            goal_ticks > ticks && goal_x > failed_x + 1.0,
            "the trajectories diverge: alt failed at tick {ticks} x={failed_x}, main won at tick {goal_ticks} x={goal_x}"
        );
    });
}

#[test]
fn frozen_input_mutation_changes_admitted_identity_and_content() {
    big_stack(|| {
        // Removing the CoinB spawn (template 4) from a copy of the sample and
        // respawning it from the CoinA template changes the admitted template
        // identity and the published draw stream.
        let mutated = source_of("two", "main.at").replace(
            "h1=ag_object_spawn(1, 4, 10.0, 1.0, 0.0)",
            "h1=ag_object_spawn(1, 3, 10.0, 1.0, 0.0)",
        );
        assert_ne!(mutated, source_of("two", "main.at"), "the mutation applied");
        let mut h = Host::with_project(
            autographics_auto_driver::authoring::storage::Storage::load(&sample(
                "two",
                "layout_a.agscene",
            ))
            .expect("layout")
            .0,
            Default::default(),
            None,
        )
        .expect("host");
        start_sample(&mut h, "two", "mutated.at", Api::Triggers2V1, &mutated);
        let frame = h
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .expect("tick 0 frame")
            .instances
            .as_ref()
            .expect("instances")
            .clone();
        let coin_b = frame.get(1).expect("slot 1 live");
        assert_eq!(
            coin_b.template_id, 3,
            "the mutation changed the template identity"
        );
        let draws = draw_count(&h);
        assert!(draws > 0);
        // The unmuted sample publishes a different stream for the same slot.
        let mut h2 = Host::with_project(
            autographics_auto_driver::authoring::storage::Storage::load(&sample(
                "two",
                "layout_a.agscene",
            ))
            .expect("layout")
            .0,
            Default::default(),
            None,
        )
        .expect("host");
        start_sample(
            &mut h2,
            "two",
            "two-main.at",
            Api::Triggers2V1,
            &source_of("two", "main.at"),
        );
        let frame2 = h2
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .expect("tick 0 frame")
            .instances
            .as_ref()
            .expect("instances")
            .clone();
        assert_eq!(frame2.get(1).expect("slot 1").template_id, 4);
        assert_ne!(
            circle_count(&h),
            circle_count(&h2),
            "the template mutation changed the published content (quad to circle)"
        );
    });
}
