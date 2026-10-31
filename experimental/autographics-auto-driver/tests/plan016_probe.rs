//! PLAN-016 T-02 whole-path prototype probe (temporary; superseded by the
//! named pilot tests). Proves on the real pinned AG execution path:
//! Start-once per instance, per-instance SoA state isolation for two
//! same-definition Hazards + a root sentinel, real input->character motion,
//! one real trigger event -> Auto-side Collect -> real despawn, idempotent
//! repeat, and the frozen credit envelope with a 33rd-lifecycle-write fault.
//! Run single-threaded:
//! cargo test --locked --offline --test plan016_probe -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const DT: f32 = 1.0 / 60.0;
const RUN: i32 = 1;

/// Lowering-shaped user source: per-node-definition SoA storage + per-instance
/// handlers, all gameplay rules (patrol wave, collect guard, despawn) in real
/// Auto. Player(1) walks right at 2 u/s; Hazard instances 0/1 patrol on y
/// layers 2.4/3.2 (never touching the walk line); Coin instance 0 sits on the
/// walk line at x 3.0; round root keeps a `phase` sentinel the Hazards must
/// not alias.
const USER: &str = r#"
var ok=0.0
var ok_in=0.0
var ok_mv=0.0
var ok_h0=0.0
var ok_h1=0.0
var ok_ev=0.0
var skip_mv=0
var skip_ev=0
var skip_hz=0
var overflow_arm=0
var p_axis=0
var p_starts=0
var p_ticks=0
var h_phase=[0.1,0.45]
var h_speed=[0.5,0.9]
var h_base=[2.4,3.2]
var h_dir=[1.0,1.0]
var h_ticks=[0,0]
var h_starts=[0,0]
var h_handle=[0,0]
var c_present=[1]
var c_handle=[0]
var c_starts=0
var c_alive=1
var c_events=0
var c_collects=0
var r_starts=0
var r_ticks=0
var r_phase=0.5
var r_coin_present=1
fn get_h0_phase() float {return h_phase[0]}
fn get_h1_phase() float {return h_phase[1]}
fn get_h0_y() float {return hazard_wave(0)}
fn get_h1_y() float {return hazard_wave(1)}
fn get_h0_ticks() int {return h_ticks[0]}
fn get_h1_ticks() int {return h_ticks[1]}
fn get_h0_starts() int {return h_starts[0]}
fn get_h1_starts() int {return h_starts[1]}
fn get_r_phase() float {return r_phase}
fn get_r_starts() int {return r_starts}
fn get_r_ticks() int {return r_ticks}
fn get_r_coin_present() int {return r_coin_present}
fn get_c_events() int {return c_events}
fn get_c_collects() int {return c_collects}
fn get_c_starts() int {return c_starts}
fn get_c_alive() int {return c_alive}
fn probe_extra_charged() int {return ag_action_held(1)}
fn get_ok() float {return ok}
fn get_ok_in() float {return ok_in}
fn get_ok_mv() float {return ok_mv}
fn get_ok_h0() float {return ok_h0}
fn get_ok_h1() float {return ok_h1}
fn get_ok_ev() float {return ok_ev}
fn set_skip_mv(v int) {skip_mv=v}
fn set_skip_ev(v int) {skip_ev=v}
fn set_skip_hz(v int) {skip_hz=v}
fn get_c_handle() int {return c_handle[0]}
fn get_axis() int {return p_axis}
fn get_char_x() float {return ag_local(0,0)}
fn get_char_y() float {return ag_local(0,1)}
fn get_obj0_x() float {return ag_local(3,0)}
fn get_zero() float {return 0.0}
fn get_local00() float {return ag_local(0,0)}
fn get_world_x() float {return ag_world(0,0)}
fn probe_mv(dx float) int {return ag_char_move(dx, 0.0, 0.002)}
fn hazard_wave(i int) float {
 let ph=h_phase[i]
 var tri=ph
 if ph>0.4 {tri=0.8-ph}
 return h_base[i]+(tri-0.4)
}
fn hazard_tick(i int, dt float) {
 h_phase[i]=h_phase[i]+h_speed[i]*dt
 if h_phase[i]>0.8 {h_phase[i]=h_phase[i]-0.8}
 h_ticks[i]=h_ticks[i]+1
 if i==0 {ok_h0=ok_h0+ag_object_move(h_handle[0], 12.0, hazard_wave(0), 0.0)*1.0}
 if i==1 {ok_h1=ok_h1+ag_object_move(h_handle[1], 16.0, hazard_wave(1), 0.0)*1.0}
 ok=ok+1.0
}
fn coin_collect(key int) {
 if r_coin_present==1 {
  r_coin_present=0
  c_alive=0
  c_collects=c_collects+1
  ok=ok+ag_object_despawn(c_handle[0])
 }
}
fn app_start() {
 r_starts=1
 p_starts=1
 h_starts[0]=1
 h_starts[1]=1
 c_starts=1
 ag_set_local(0,1,1.0)
 ag_set_local(0,0,2.0)
 ok=ok+ag_char_bind(1)
 ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)
 h_handle[0]=ag_object_spawn(0, 3, 12.0, 2.4, 0.0)
 h_handle[1]=ag_object_spawn(1, 4, 16.0, 3.2, 0.0)
 c_handle[0]=ag_object_spawn(2, 5, 4.0, 1.0, 0.0)
 ok=ok+ag_trigger_bind(0, 1, 1, c_handle[0], 0, 0.0)
 ok=ok+ag_trigger_bind(1, 1, 1, h_handle[0], 0, 0.0)
 ok=ok+ag_trigger_bind(2, 1, 1, h_handle[1], 0, 0.0)
}
fn app_update(dt float) {
 r_ticks=r_ticks+1
 p_ticks=p_ticks+1
 p_axis=ag_action_held(1)
 ok_in=ok_in+1.0
 var do_mv=p_axis
 if skip_mv==1 && r_coin_present==0 {do_mv=0}
 ok_mv=ok_mv+ag_char_move(do_mv*1.0*2.0*dt, 0.0, 0.002)*1.0
 ok=ok+ok_mv
 if overflow_arm==1 {app_overflow()}
 if skip_hz==0 || r_coin_present==1 {
  hazard_tick(0, dt)
  hazard_tick(1, dt)
 }
 var n=0
 var do_ev=1
 if skip_ev==1 && r_coin_present==0 {do_ev=0}
 if do_ev==1 {n=ag_event_next()}
 if n==1 {
  let p=ag_event_pair()
  if p==0 {
   c_events=c_events+1
   coin_collect(0)
  }
 }
}
// 33rd lifecycle write in one callback: the named objects-pool overflow,
// executed inside a dispatched tick (post-evaluate locks direct moves).
fn set_overflow(v int) {overflow_arm=v}
fn app_overflow() {
 var i=0
 for i in 0..40 {
  ok=ok+ag_object_move(h_handle[0], 12.0, 2.4, 0.0)
 }
}
// Stale-generation write: a run-1 handle must not answer in a later run.
fn stale_write(h int) int {
 return ag_object_despawn(h)
}
fn idempotent_second_collect() {
 coin_collect(0)
}
"#;

fn rows() -> Vec<(i32, i32, [f32; 3], [f32; 3])> {
    vec![
        (1, 2, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0]),       // char quad id 1
        (1, 2, [-4.0, -1.0, 0.0], [34.0, 0.0, 0.0]),      // floor id 2
        (0, 2, [-0.3, -0.3, 0.0], [0.3, 0.3, 0.0]),       // coin circle id 3
        (1, 2, [-0.5, -0.3, 0.0], [0.5, 0.3, 0.0]),       // hazard quad id 4
        (1, 2, [-0.5, -0.3, 0.0], [0.5, 0.3, 0.0]),       // hazard quad id 5
    ]
}

fn session(user: &str, name: &str, run: i32) -> Session {
    let snap =
        SourceSnapshot::from_bytes_for(Path::new(name), user.as_bytes().to_vec(), Api::Triggers2V1)
            .expect("admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let scene = rows();
    s.call(
        "_ag_spatial_reset",
        &[Int(scene.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, dim, lo, hi)) in scene.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        // Template rows park far off the play area; the character row is
        // placed at the Player node's spawn (2,1). Instances spawn from the
        // parked templates, so unborn templates can never overlap the char.
        let (tx, ty) = if n == 0 { (2.0, 1.0) } else { (-100.0, -100.0) };
        for v in [tx, ty, 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
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
    for n in 0..scene.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_trg_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}
fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}
fn float(s: &mut Session, name: &str) -> f32 {
    s.float(name, &[]).unwrap()
}
fn tick(s: &mut Session, step: i32) {
    // Host-side input injection: hold Right (bit 1 of the held mask), one
    // sample consumed per world tick.
    s.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

/// Independent triangle-wave expectation for hazard i at tick n.
fn wave(phase0: f32, speed: f32, base: f32, n: i32) -> f32 {
    let mut ph = phase0 + speed * DT * n as f32;
    ph %= 0.8;
    let tri = if ph > 0.4 { 0.8 - ph } else { ph };
    base + (tri - 0.4)
}

#[test]
fn probe_start_two_instances_one_event_and_budget() {
    let mut s = session(USER, "probe016.at", RUN);
    assert_eq!(status(&mut s), 0, "start fault-free");

    // Start-once semantics: every instance starts exactly 1.
    assert_eq!(int(&mut s, "get_r_starts"), 1);
    assert_eq!(int(&mut s, "get_h0_starts"), 1);
    assert_eq!(int(&mut s, "get_h1_starts"), 1);
    assert_eq!(int(&mut s, "get_c_starts"), 1);

    // Drive to the coin: player x(t) = 0.5 + 2*t; coin circle at x 3.0 with
    // |dx| <= 0.7 overlaps from tick ~66 (0.5+2*t >= 2.3); event is consumed
    // on the tick AFTER the enter evaluate.
    let mut collect_tick = 0;
    for t in 1..=120 {
        tick(&mut s, t);
        assert_eq!(status(&mut s), 0, "tick {t} fault-free");
        if collect_tick == 0 && int(&mut s, "get_r_coin_present") == 0 {
            collect_tick = t;
            break;
        }
    }
    assert!(
        collect_tick > 0 && collect_tick <= 120,
        "coin collected within 120 ticks, got {collect_tick}"
    );

    // Instance isolation: both hazards ticked the same count, phases match the
    // independent triangle-wave oracle, and the root sentinel phase never moved.
    assert_eq!(int(&mut s, "get_h0_ticks"), collect_tick);
    assert_eq!(int(&mut s, "get_h1_ticks"), collect_tick);
    let h0 = float(&mut s, "get_h0_phase");
    let h1 = float(&mut s, "get_h1_phase");
    assert!((h0 - (0.1 + 0.5 * DT * collect_tick as f32) % 0.8).abs() < 1e-4, "h0 {h0}");
    assert!((h1 - (0.45 + 0.9 * DT * collect_tick as f32) % 0.8).abs() < 1e-4, "h1 {h1}");
    assert!((float(&mut s, "get_h0_y") - wave(0.1, 0.5, 2.4, collect_tick)).abs() < 1e-4, "h0y");
    assert!((float(&mut s, "get_h1_y") - wave(0.45, 0.9, 3.2, collect_tick)).abs() < 1e-4, "h1y");
    assert!((float(&mut s, "get_r_phase") - 0.5).abs() < 1e-6, "sentinel");

    // Real despawn: the coin object answers dead; exactly one Collect ran.
    assert_eq!(int(&mut s, "get_c_alive"), 0);
    assert_eq!(int(&mut s, "get_c_collects"), 1);
    assert!(int(&mut s, "get_c_events") >= 1);

    // Idempotence: a repeat Collect (and continued overlap ticks) cannot
    // resurrect or re-count.
    s.call("idempotent_second_collect", &[], ResultKind::Unit).unwrap();
    for t in collect_tick + 1..=120 {
        tick(&mut s, t);
        if t % 5 == 0 {
            println!(
                "i{t}: stat={:?} in={:?} mv={:?} h0={:?} ev={:?} x={:?}",
                s.int("_ag_status", &[]),
                s.float("get_ok_in", &[]),
                s.float("get_ok_mv", &[]),
                s.float("get_ok_h0", &[]),
                s.float("get_ok_ev", &[]),
                s.float("get_local00", &[])
            );
        }
        if status(&mut s) != 0 {
            println!("fault at tick {t}");
            break;
        }
    }
    println!(
        "IDEM: stat={:?} collects={:?} alive={:?} ev={:?} pres={:?} x={:?}",
        s.int("_ag_status", &[]),
        s.int("get_c_collects", &[]),
        s.int("get_c_alive", &[]),
        s.int("get_c_events", &[]),
        s.int("get_r_coin_present", &[]),
        s.float("get_local00", &[])
    );
    // Isolate the post-collect fault: (a) no char_move after collect,
    // (b) no event read after collect, (c) neither.
    for (name, s_mv, s_ev, s_hz) in [("a", 1, 0, 0), ("b", 0, 1, 0), ("c", 1, 1, 0), ("d", 0, 0, 1), ("e", 0, 0, 0)] {
        let mut sv = session(USER, "probe016_iso.at", 1);
        sv.call("set_skip_mv", &[Int(s_mv)], ResultKind::Unit).unwrap();
        sv.call("set_skip_ev", &[Int(s_ev)], ResultKind::Unit).unwrap();
        sv.call("set_skip_hz", &[Int(s_hz)], ResultKind::Unit).unwrap();
        let mut ct = 0;
        for t in 1..=120 {
            tick(&mut sv, t);
            if int(&mut sv, "get_r_coin_present") == 0 {
                ct = t;
                break;
            }
        }
        for t in ct + 1..=60 {
            tick(&mut sv, t);
            if status(&mut sv) != 0 {
                println!("iso{name}: fault 10 at tick {t} (collected at {ct})");
                break;
            }
        }
        println!("iso{name}: final stat={:?} ct={ct}", sv.int("_ag_status", &[]));
    }
    assert_eq!(status(&mut s), 0);
    assert_eq!(int(&mut s, "get_c_collects"), 1, "repeat collect idempotent");
    assert_eq!(int(&mut s, "get_c_alive"), 0, "coin stays destroyed");

    // The budget envelope: the whole 120-tick window ran fault-free on the
    // shared pool (readbacks were pure Auto fns, zero charge).
    assert_eq!(int(&mut s, "get_r_ticks"), 120);

    // Full-pool negative control: the collect tick consumes exactly the shared
    // 32-credit pool; one more charged call right after it is the named
    // spatial/cast meter fault (code 10).
    {
        let mut sf = session(USER, "probe016_full.at", 4);
        let mut ctf = 0;
        for t in 1..=120 {
            tick(&mut sf, t);
            if int(&mut sf, "get_r_coin_present") == 0 {
                ctf = t;
                break;
            }
        }
        assert!(ctf > 0, "control session collects");
        sf.call("probe_extra_charged", &[], ResultKind::Int).unwrap();
        assert_eq!(status(&mut sf), 10, "33rd charged call faults: full pool");
    }


    // Old-generation isolation: a run-1 handle is foreign to a run-2 session
    // (the handle base grows with the run id); a stale despawn must be the
    // named sticky error, and the fresh run starts neutral.
    let stale_handle = int(&mut s, "get_c_handle");
    let mut s2 = session(USER, "probe016_fresh.at", 2);
    assert_eq!(status(&mut s2), 0);
    s2.call("stale_write", &[Int(stale_handle)], ResultKind::Int)
        .unwrap();
    assert_eq!(status(&mut s2), 33, "stale handle is the named fault");
    assert_eq!(int(&mut s2, "get_r_starts"), 1, "fresh run starts once");
    assert_eq!(int(&mut s2, "get_r_coin_present"), 1, "fresh run neutral");
}

#[test]
fn probe_float_mod_rejected_at_runtime() {
    // The pinned SDK parses float `%` at admission but faults at VM execution
    // (sticky fault during _ag_start): the patrol lowering must use the
    // conditional triangle-wave form, not `%`.
    let src = "var x=0.0\nfn app_start() {x=(1.5)%(0.8)}\nfn app_update(dt float) {}\nfn get_x() float {return x}\n";
    let mut s = session(src, "probe016_mod.at", RUN);
    assert_ne!(status(&mut s), 0, "float % must not execute cleanly");
}
