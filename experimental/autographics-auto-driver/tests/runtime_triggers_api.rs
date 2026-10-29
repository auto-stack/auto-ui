//! auto-ui PLAN-002 triggers public API tests (contract V-01/V-02): the two
//! combined identities and the derived entry chain, the independent geometry
//! oracle (closed-interval AABB and closest-point circle/sphere with exact
//! touch and ±1ULP separation), the once-per-update evaluate that reads the
//! same final positions, enter/stay/exit in pair order with a once-only read
//! cursor and explicit absence, the binding-generation reset on explicit
//! rebind, destruction revoking eligibility without synthetic exits, the
//! post-evaluate geometry lock with despawn/visibility still allowed, every
//! named binding rejection, the tick-0 registry rules, the restart request
//! marker, and the whole-path budget probe. Frozen read timing: the user
//! cursor consumes the PREVIOUS evaluate's snapshot while the admitted block
//! emitters always show the current one. Frozen pricing drives every scenario:
//! one character move is ~26 credits (24 sweep + 2), object ops / binds /
//! event reads / request are 1 each, and the shared callback window is 32 —
//! so each callback pairs at most one character move with a bounded read.
//! The host Session caps direct calls at four arguments, so six-argument
//! binds are exercised through one-argument user wrappers, exactly like
//! production user code. Run single-threaded:
//! cargo test --locked --test runtime_triggers_api -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const DT: f32 = 1.0 / 60.0;
const RUN: i32 = 1;
const GEN_SPAN: i32 = 65536;
const RUN_STRIDE: i32 = GEN_SPAN * 8;
const PHASE_ENTER: i32 = 1;
const PHASE_STAY: i32 = 2;
const PHASE_EXIT: i32 = 3;

struct Row(i32, i32, [f32; 3], [f32; 3]);

fn row2(kind: i32, dim: i32) -> Row {
    Row(kind, dim, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0])
}

fn row3(kind: i32, dim: i32) -> Row {
    Row(kind, dim, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4])
}

/// 2D scene: character quad (id 1), floor quad (2), circle (3), quad (4);
/// id 5 is a 3D cylinder decoy, id 6 a group decoy (never a legal binding).
fn rows2d() -> Vec<Row> {
    vec![
        row2(1, 2),
        Row(1, 2, [-4.0, -1.0, 0.0], [34.0, 0.0, 0.0]),
        row2(0, 2),
        row2(1, 2),
        row2(2, 3),
        row2(5, 2),
    ]
}

/// 3D scene: character cylinder (1), floor cylinder (2), mob (3), model-kind
/// cylinder (4); id 5 is a 2D quad decoy, id 6 a group decoy.
fn rows3d() -> Vec<Row> {
    vec![
        row3(2, 3),
        Row(2, 3, [-8.0, -1.0, -8.0], [8.0, 0.0, 8.0]),
        row3(4, 3),
        row3(2, 3),
        row3(1, 2),
        row3(5, 3),
    ]
}

// No solid binding: the trigger geometry cases float the character at
// y 0..1, so a horizontal sweep can never be blocked by closed-interval
// contact with a solid floor (the trigger profiles do not test solidity).
const BIND2: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n";
const BIND3: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_camera_orbit_3d(1,8.0,45.0,0,0.0)\n";

/// Shared helpers: absolute instance/character placement and an event read
/// loop. All top-level names avoid the reserved transport roots.
const HELPERS: &str = "\
var gx=3.0\nvar gy=0.5\nvar gz=0.0\nvar cx=0.0\nvar state=0\nvar hits=0\nvar req=0\nvar ticks=0\nvar res=0
var canread=0\nvar h1=0\nvar h2=0\nvar h3=0\n\
fn setx(v float) {gx=v}\n\
fn sety(v float) {gy=v}\n\
fn setz(v float) {gz=v}\n\
fn get_state() int {return state}\n\
fn get_hits() int {return hits}\n\
fn get_req() int {return req}\n\
fn get_res() int {return res}\n\
fn reb(v int) int {return ag_trigger_bind(0, 1, 1, v, 0, 0.0)}\n\
fn read_events() int {\n\
 var n=0\n\
 for ag_event_next()==1 {\n\
  n=n+1\n\
 }\n\
 return n\n\
}\n";

fn source2d(start: &str, update: &str) -> String {
    format!(
        "var ok=0\n{HELPERS}fn sp() int {{h1=ag_object_spawn(0, 3, gx, gy, gz); return h1}}\nfn de() int {{return ag_object_despawn(h1)}}\nfn movetox(v float) {{ok=ok+ag_char_move(v - cx, 0.0, 0.002); cx=v}}\nfn app_start() {{\n{BIND2}{start}}}\nfn app_update(dt float) {{\n{update}}}\n"
    )
}

fn source3d(start: &str, update: &str) -> String {
    format!(
        "var ok=0\n{HELPERS}fn movetox(v float) {{ok=ok+ag_char3_move(v - cx, 0.0, 0.0, 0.002); cx=v}}\nfn app_start() {{\n{BIND3}{start}}}\nfn app_update(dt float) {{\n{update}}}\n"
    )
}

fn triggers_session(
    api: Api,
    run: i32,
    user: &str,
    name: &str,
    rows: &[Row],
) -> Result<Session, String> {
    let snap = SourceSnapshot::from_bytes_for(Path::new(name), user.as_bytes().to_vec(), api)?;
    let mut s = Session::new(&snap.source)?;
    s.call("main", &[], ResultKind::Unit)?;
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)?;
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )?;
    for (n, Row(kind, dim, lo, hi)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )?;
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)?;
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)?;
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)?;
        }
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )?;
        }
    }
    let char_map = match api {
        Api::Triggers2V1 => "_ag_char_map",
        _ => "_ag_char3_map",
    };
    for n in 0..rows.len() {
        s.call(
            char_map,
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )?;
        s.call(
            "_ag_obj_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )?;
        s.call(
            "_ag_trg_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )?;
    }
    s.call("_ag_start", &[], ResultKind::Unit)?;
    Ok(s)
}

/// `_ag_status` mirrors the sticky error code; 0 means fault-free.
fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

fn fault_arg(s: &mut Session) -> i32 {
    s.int("_ag_error_argument", &[]).unwrap()
}

fn tick(s: &mut Session, step: i32) {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

fn handle(slot: i32, gen: i32) -> i32 {
    RUN * RUN_STRIDE + slot * GEN_SPAN + gen
}

fn set_float(s: &mut Session, name: &str, v: f32) {
    s.call(name, &[Float(v)], ResultKind::Unit).unwrap();
}

/// The evaluate emitters for one event row (canonical zeros past the count).
fn event_row(s: &mut Session, r: i32) -> [i32; 8] {
    [
        s.int("_ag_trg_e0", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e1", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e2", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e3", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e4", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e5", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e6", &[Int(r)]).unwrap(),
        s.int("_ag_trg_e7", &[Int(r)]).unwrap(),
    ]
}

fn event_count(s: &mut Session) -> i32 {
    s.int("_ag_trg_count_v", &[]).unwrap()
}

fn restart_flag(s: &mut Session) -> i32 {
    s.int("_ag_trg_req_v", &[]).unwrap()
}

// ------------------------------------------------- assembly/identity (V-01) --

#[test]
fn triggers_identities_parse_with_frozen_numbers_and_derived_chain() {
    for (api, version, number, profile) in [
        (Api::Triggers2V1, "autographics.triggers2d/1", 107, 18),
        (Api::Triggers3V1, "autographics.triggers3d/1", 108, 19),
    ] {
        assert_eq!(Api::parse(version).unwrap(), api);
        assert_eq!(api.number(), number, "{version}");
        assert_eq!(api.profile(), profile, "{version}");
        assert_eq!(api.version(), version);
        assert!(api.objects().is_some(), "triggers imply the objects unit");
        assert!(api.triggers().is_some());
    }
    assert!(Api::Objects2V1.triggers().is_none());
    // The derived entry chain: registry freeze before app_start, tick counters
    // before the callback and exactly one evaluate after the post-callback
    // derive (the unique `toggled=0}` tail belongs to tick alone; transport
    // root names arrive namespaced in the combined source).
    let bare = "var ok=0\nfn app_start() {}\nfn app_update(dt float) {}\n";
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("identity-trg2.at"),
        bare.as_bytes().to_vec(),
        Api::Triggers2V1,
    )
    .unwrap();
    assert!(
        snap.source
            .contains("if _ag_error==0 { _ag_obj_freeze(); _ag_trg_freeze(); app_start();"),
        "the trigger registry must freeze before app_start"
    );
    assert!(
        snap.source.contains(
            "_ag_char_tick_begin(); _ag_obj_tick_begin(); _ag_trg_tick_begin(); app_update(dt); _ag_cast_token=0"
        ),
        "tick begin must reset the lock and the request marker"
    );
    assert!(
        snap.source
            .contains("_ag_dirty=0; _ag_trg_evaluate(); _ag_toggled=0}"),
        "exactly one evaluate must run after the post-callback derive"
    );
    assert!(snap.source.contains("var _ag_trg_dim=2"));
    assert!(snap.source.contains("var _ag_obj_locked=0"));
    // The derived post-evaluate lock guards spawn/move and the character move
    // inside the combined unit, while the delivered bytes stay untouched.
    assert_eq!(
        snap.source
            .matches("if _ag_obj_locked==1 {_ag_fail(40,-1,0);return 0}")
            .count(),
        3,
        "spawn, object move and character move carry the lock guard"
    );
    let despawn = snap
        .source
        .split("fn ag_object_despawn(")
        .nth(1)
        .unwrap()
        .split("}")
        .next()
        .unwrap();
    assert!(
        !despawn.contains("_ag_obj_locked"),
        "despawn stays allowed after evaluate"
    );
    let regions: Vec<_> = snap.regions.iter().map(|r| r.file.as_str()).collect();
    assert!(regions.contains(&"<runtime_triggers_v1.at>"), "{regions:?}");
    let snap3 = SourceSnapshot::from_bytes_for(
        Path::new("identity-trg3.at"),
        bare.as_bytes().to_vec(),
        Api::Triggers3V1,
    )
    .unwrap();
    assert!(snap3.source.contains("var _ag_trg_dim=3"));
    assert!(snap3.source.contains(
        "_ag_char3_tick_begin(); _ag_obj_tick_begin(); _ag_trg_tick_begin(); app_update(dt); _ag_cast_token=0"
    ));
}

// ------------------------------------------------------ geometry (V-01/02) --

/// Walk an instance through touch/inside/separation against the character and
/// compare every phase with the independent Rust oracle of the same inputs.
/// The character never moves (object moves are 1 credit), so the whole walk
/// rides well inside the frozen 32-credit callback window.
fn aabb_walk(api: Api, rows: Vec<Row>, source: fn(&str, &str) -> String) {
    let start = "\
 h1=ag_object_spawn(0, 3, 3.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
";
    let update = "\
 ok=ok+ag_object_move(h1, gx, gy, gz)\n\
 state=0\n\
 var n=0\n\
 for ag_event_next()==1 {\n\
  n=n+1\n\
  if ag_event_pair()!=0 {state=91}\n\
  state=ag_event_phase()\n\
 }\n\
 if n>1 {state=90}\n\
";
    let mut s = triggers_session(api, RUN, &source(start, update), "aabb-walk.at", &rows)
        .expect("aabb walk session");
    assert_eq!(status(&mut s), 0, "bind in app_start must be clean");
    // Tick 0 registry: no events, seq 0; the binding row is live.
    assert_eq!(event_count(&mut s), 0, "app_start never emits events");
    assert_eq!(s.int("_ag_trg_seq_v", &[]).unwrap(), 0);
    assert_eq!(s.int("_ag_trg_b0", &[Int(0)]).unwrap(), 1);
    assert_eq!(s.int("_ag_trg_b1", &[Int(0)]).unwrap(), 1, "bind gen 1");
    // Separated: no event in either view.
    set_float(&mut s, "setx", 3.0);
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        0,
        "no event when separated"
    );
    assert_eq!(
        s.int("_ag_trg_seq_v", &[]).unwrap(),
        1,
        "one evaluate per update"
    );
    // Exact touch is a closed-interval hit: the block emits enter immediately.
    set_float(&mut s, "setx", 0.8);
    tick(&mut s, 2);
    assert_eq!(
        event_row(&mut s, 0),
        [0, PHASE_ENTER, 1, 1, handle(0, 1), 2, 1, 0]
    );
    // Inside: stay; the user read of the previous tick now reports the enter.
    set_float(&mut s, "setx", 0.5);
    tick(&mut s, 3);
    assert_eq!(event_row(&mut s, 0)[1], PHASE_STAY, "overlap stays");
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        PHASE_ENTER,
        "touch enters (user read)"
    );
    // One ULP past the touch line is a real separation: exit.
    let touch = 0.8f32;
    let ulp = f32::from_bits(touch.to_bits() + 1);
    set_float(&mut s, "setx", ulp);
    tick(&mut s, 4);
    assert_eq!(event_row(&mut s, 0)[1], PHASE_EXIT, "1ULP gap exits");
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        PHASE_STAY,
        "overlap stays (user read)"
    );
    assert_eq!(status(&mut s), 0);
    // Far separation: nothing new, and the pair stays bound.
    set_float(&mut s, "setx", 5.0);
    tick(&mut s, 5);
    assert_eq!(event_count(&mut s), 0);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        PHASE_EXIT,
        "1ULP gap exits (user read)"
    );
    assert_eq!(s.int("_ag_trg_b0", &[Int(0)]).unwrap(), 1);
    // Returning to the exact touch line enters again under the same binding.
    set_float(&mut s, "setx", touch);
    tick(&mut s, 6);
    assert_eq!(event_row(&mut s, 0)[1], PHASE_ENTER, "re-enter");
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        0,
        "the silent tick read nothing"
    );
}

#[test]
fn aabb_closed_interval_matches_the_independent_oracle_2d() {
    aabb_walk(Api::Triggers2V1, rows2d(), source2d);
}

#[test]
fn aabb_closed_interval_matches_the_independent_oracle_3d() {
    aabb_walk(Api::Triggers3V1, rows3d(), source3d);
}

#[test]
fn disc_and_sphere_use_closest_point_and_explicit_radius_only() {
    // 2D: circle radius 0.5 against the character AABB. At the tangent center
    // distance the disc hits while the AABB is open (closed interval vs
    // closest point), and at the corner position the AABB hits while the disc
    // does not — the two shapes are told apart every step.
    let start = "\
 h1=ag_object_spawn(0, 3, 3.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h1, 1, 0.5)\n\
";
    let update = "\
 ok=ok+ag_object_move(h1, gx, gy, gz)\n\
 var aabb_p=0\n\
 var disc_p=0\n\
 for ag_event_next()==1 {\n\
  if ag_event_pair()==0 {aabb_p=ag_event_phase()}\n\
  if ag_event_pair()==1 {disc_p=ag_event_phase()}\n\
 }\n\
 state=aabb_p*10+disc_p\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, update),
        "disc-tangent.at",
        &rows2d(),
    )
    .expect("disc session");
    assert_eq!(status(&mut s), 0);
    // f32 arithmetic makes 0.9 - 0.4 round to exactly the radius: the disc
    // tangent-hits while the closed AABB is still 0.1 open — the shapes are
    // told apart at the same position. A 0.01 gap rejects the disc.
    set_float(&mut s, "setx", 0.9);
    tick(&mut s, 1);
    assert_eq!(event_count(&mut s), 1, "aabb silent, disc tangent-hits");
    assert_eq!(
        event_row(&mut s, 0),
        [1, PHASE_ENTER, 1, 1, handle(0, 1), 2, 1, 1]
    );
    // Binary-exact y tangent (dy = 1.5 - 1.0 = 0.5): both shapes enter — the
    // AABB by closed touch, the disc by closest-point distance == radius.
    set_float(&mut s, "setx", 0.0);
    set_float(&mut s, "sety", 1.5);
    tick(&mut s, 2);
    assert_eq!(
        event_count(&mut s),
        2,
        "tangent position: both shapes report"
    );
    assert_eq!(
        event_row(&mut s, 0),
        [0, PHASE_ENTER, 1, 1, handle(0, 1), 2, 1, 0]
    );
    assert_eq!(
        event_row(&mut s, 1),
        [1, PHASE_STAY, 1, 1, handle(0, 1), 2, 1, 1]
    );
    // A corner-in-box, sphere-out position: the AABB stays, the disc exits.
    set_float(&mut s, "sety", 1.4);
    set_float(&mut s, "setx", 0.75);
    tick(&mut s, 3);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        12,
        "user read: aabb entered (1), disc stayed (2) at the tangent spot"
    );
    tick(&mut s, 4);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        23,
        "aabb stays (2), disc exits (3): corner outside the circle"
    );
    assert_eq!(status(&mut s), 0);
    // A nonpositive or non-finite radius is a named binding rejection, and a
    // 0.01 gap past the tangent is a real separation counterexample.
    let mut bad = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(
            " h1=ag_object_spawn(0, 3, 0.91, 0.5, 0.0)\n ok=ok+ag_trigger_bind(0, 1, 1, h1, 1, 0.5)",
            "ok=ok+ag_object_move(h1, 0.91, 0.5, 0.0)\nvar c=0\nfor ag_event_next()==1 {c=c+1}\nstate=c\n",
        ),
        "disc-gap.at",
        &rows2d(),
    )
    .expect("gap session");
    tick(&mut bad, 1);
    assert_eq!(
        bad.int("get_state", &[]).unwrap(),
        0,
        "0.01 past the radius separates"
    );
    let mut bad = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(
            " h1=ag_object_spawn(0, 3, 3.0, 0.5, 0.0)\n ok=ok+ag_trigger_bind(0, 1, 1, h1, 1, 0.0)",
            "",
        ),
        "disc-radius0.at",
        &rows2d(),
    )
    .expect("radius session");
    assert_eq!(status(&mut bad), 41);
    let nan = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(
            " h1=ag_object_spawn(0, 3, 3.0, 0.5, 0.0)\n ok=ok+ag_trigger_bind(0, 1, 1, h1, 1, 0.0/0.0)",
            "",
        ),
        "disc-radius-nan.at",
        &rows2d(),
    );
    assert!(
        nan.is_err() && nan.err().unwrap().contains("DivisionByZero"),
        "0.0/0.0 traps at the VM arithmetic boundary"
    );

    // 3D: the same distinguishing corner for the sphere; the radius only ever
    // comes from the explicit bind configuration, never a GLB bound.
    let start3 = "\
 h1=ag_object_spawn(0, 3, 3.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h1, 1, 0.5)\n\
";
    let update3 = "\
 ok=ok+ag_object_move(h1, gx, gy, gz)\n\
 var aabb_p=0\n\
 var sphere_p=0\n\
 for ag_event_next()==1 {\n\
  if ag_event_pair()==0 {aabb_p=ag_event_phase()}\n\
  if ag_event_pair()==1 {sphere_p=ag_event_phase()}\n\
 }\n\
 state=aabb_p*10+sphere_p\n\
";
    let mut s3 = triggers_session(
        Api::Triggers3V1,
        RUN,
        &source3d(start3, update3),
        "sphere-corner.at",
        &rows3d(),
    )
    .expect("sphere session");
    assert_eq!(status(&mut s3), 0);
    set_float(&mut s3, "setx", 0.75);
    set_float(&mut s3, "sety", 1.4);
    tick(&mut s3, 1);
    assert_eq!(
        event_count(&mut s3),
        1,
        "aabb enters, sphere silent at the corner"
    );
    assert_eq!(
        event_row(&mut s3, 0),
        [0, PHASE_ENTER, 1, 1, handle(0, 1), 2, 1, 0]
    );
    tick(&mut s3, 2);
    assert_eq!(
        s3.int("get_state", &[]).unwrap(),
        10,
        "user read of the corner tick: aabb entered (1), sphere silent (0)"
    );
}

#[test]
fn one_evaluate_reads_the_same_final_positions() {
    // The instance leaves and re-enters INSIDE one callback: only the final
    // position exists for the single evaluate, so the pair enters at the end
    // and the next callback's identical detour never produces an exit.
    let start = "\
 h1=ag_object_spawn(0, 3, 5.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
";
    let update = "\
 ok=ok+ag_object_move(h1, 5.0, 0.5, 0.0)\n\
 ok=ok+ag_object_move(h1, gx, 0.5, 0.0)\n\
 state=0\n\
 for ag_event_next()==1 {\n\
  state=ag_event_phase()\n\
 }\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, update),
        "double-move.at",
        &rows2d(),
    )
    .expect("double move session");
    assert_eq!(status(&mut s), 0);
    set_float(&mut s, "setx", 0.5);
    tick(&mut s, 1);
    assert_eq!(
        event_row(&mut s, 0),
        [0, PHASE_ENTER, 1, 1, handle(0, 1), 2, 1, 0]
    );
    tick(&mut s, 2);
    assert_eq!(event_row(&mut s, 0)[1], PHASE_STAY);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        PHASE_ENTER,
        "user read: entered at the final position"
    );
    tick(&mut s, 3);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        PHASE_STAY,
        "the mid-callback detour never produced an exit"
    );
    assert_eq!(s.int("_ag_trg_seq_v", &[]).unwrap(), 3);
}

// ------------------------------------------------- read cursor (V-02) --

#[test]
fn three_items_same_tick_read_once_with_explicit_absence() {
    // Three items overlap in the same tick: three enter events in pair order;
    // each event is delivered exactly once (3 enters consumed, then 3 stays —
    // never the enters again), with an explicit absence and getters that
    // expire. Instance-only motion keeps every callback inside the budget.
    let start = "\
 h1=ag_object_spawn(0, 3, 0.0, 0.5, 0.0)\n\
 h2=ag_object_spawn(1, 4, 0.0, 0.5, 0.0)\n\
 h3=ag_object_spawn(2, 3, 0.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h2, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(2, 1, 1, h3, 0, 0.0)\n\
";
    let update = "\
 hits=hits+read_events()\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, update),
        "three-items.at",
        &rows2d(),
    )
    .expect("three items session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(event_count(&mut s), 3, "three pairs, three events");
    assert_eq!(
        event_row(&mut s, 0),
        [0, PHASE_ENTER, 1, 1, handle(0, 1), 2, 1, 0]
    );
    assert_eq!(
        event_row(&mut s, 1),
        [1, PHASE_ENTER, 1, 1, handle(1, 1), 3, 1, 0]
    );
    assert_eq!(
        event_row(&mut s, 2),
        [2, PHASE_ENTER, 1, 1, handle(2, 1), 2, 1, 0]
    );
    // Staying another tick: three stays; the user consumed the 3 enters once.
    tick(&mut s, 2);
    assert_eq!(event_count(&mut s), 3);
    assert_eq!(
        s.int("get_hits", &[]).unwrap(),
        3,
        "the 3 enters were delivered once"
    );
    assert_eq!(event_row(&mut s, 0)[1], PHASE_STAY);
    tick(&mut s, 3);
    assert_eq!(
        s.int("get_hits", &[]).unwrap(),
        6,
        "3 stays consumed; the enters were never re-delivered"
    );
    // The cursor is once-only: the getters answer the current token and the
    // walk past the end is an explicit absence.
    assert_eq!(s.int("ag_event_next", &[]).unwrap(), 1);
    assert_eq!(s.int("ag_event_pair", &[]).unwrap(), 0);
    assert_eq!(s.int("ag_event_phase", &[]).unwrap(), PHASE_STAY);
    assert_eq!(s.int("ag_event_generation", &[]).unwrap(), 1);
    assert_eq!(s.int("ag_event_source_kind", &[]).unwrap(), 1);
    assert_eq!(s.int("ag_event_source", &[]).unwrap(), handle(0, 1));
    assert_eq!(s.int("ag_event_geometry", &[]).unwrap(), 0);
    assert_eq!(s.int("ag_event_next", &[]).unwrap(), 1);
    assert_eq!(s.int("ag_event_next", &[]).unwrap(), 1);
    assert_eq!(s.int("ag_event_next", &[]).unwrap(), 0, "explicit absence");
    assert_eq!(s.int("ag_event_pair", &[]).unwrap(), 0, "getters expire");
    assert_eq!(status(&mut s), 0, "absence never sets a sticky error");
}

#[test]
fn destruction_revokes_eligibility_and_new_generations_never_inherit() {
    // The collection runs inside app_update right after the enter is read:
    // despawn plus a same-slot respawn (the lock allows both between ticks).
    let start = "\
 h1=ag_object_spawn(0, 3, 0.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
";
    let update = "\
 hits=hits+read_events()\n\
 ticks=ticks+1\n\
 if ticks==2 {\n\
  ok=ok+ag_object_despawn(h1)\n\
  h1=ag_object_spawn(0, 3, 9.0, 0.5, 0.0)\n\
 }\n\
 if ticks==4 {res=reb(524289)}\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, update),
        "revoke.at",
        &rows2d(),
    )
    .expect("revoke session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(event_count(&mut s), 1, "entered (block view)");
    assert_eq!(event_row(&mut s, 0)[1], PHASE_ENTER);
    // tick 2: the enter is read and the instance is destroyed in the same
    // callback; the evaluate that follows sees a dead identity.
    tick(&mut s, 2);
    assert_eq!(
        s.int("get_hits", &[]).unwrap(),
        1,
        "the enter was consumed once"
    );
    assert_eq!(
        event_count(&mut s),
        0,
        "no synthetic exit for the dead identity"
    );
    assert_eq!(
        s.int("_ag_trg_b0", &[Int(0)]).unwrap(),
        0,
        "the pair is unbound"
    );
    // The same slot respawns into a NEW generation: it must NOT be auto-bound.
    assert_eq!(
        s.float("_ag_obj_xv", &[Int(0)]).unwrap(),
        9.0,
        "the new generation sits at its own position"
    );
    tick(&mut s, 3);
    assert_eq!(
        event_count(&mut s),
        0,
        "no events for an unbound generation"
    );
    assert_eq!(s.int("_ag_trg_b0", &[Int(0)]).unwrap(), 0);
    // The stale generation-1 handle (524289) is dead for binding too; the
    // named stale rejection fires inside a callback (not under the lock).
    tick(&mut s, 4);
    assert_eq!(s.int("get_res", &[]).unwrap(), 0, "the stale bind failed");
    assert_eq!(
        status(&mut s),
        33,
        "stale handle binding is the named error"
    );
}

#[test]
fn explicit_rebind_resets_prev_and_old_events_never_cross_generations() {
    let start = "\
 h1=ag_object_spawn(0, 3, 0.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
";
    // Rebind every tick to the still-overlapping instance: each binding
    // generation starts from a false previous state, so every tick enters.
    let update = "\
 hits=hits+read_events()\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, update),
        "rebind-enter.at",
        &rows2d(),
    )
    .expect("rebind session");
    assert_eq!(status(&mut s), 0);
    for step in 1..=3 {
        tick(&mut s, step);
        // The user counter lags one tick (it reads the previous snapshot);
        // the block view shows this tick's enter under the new generation.
        assert_eq!(
            s.int("get_hits", &[]).unwrap(),
            step - 1,
            "tick {step}: the previous rebind enter was counted"
        );
        assert_eq!(
            event_row(&mut s, 0)[1],
            PHASE_ENTER,
            "tick {step}: rebind enters again"
        );
        assert_eq!(
            event_row(&mut s, 0)[2],
            step as i32 + 1,
            "the event carries the generation bound in the same callback"
        );
        assert_eq!(s.int("_ag_trg_b1", &[Int(0)]).unwrap(), step as i32 + 1);
    }
    // A binding change after evaluate is the named lock rejection, never a
    // silent rebind (routed through the one-argument user wrapper).
    assert_eq!(s.int("reb", &[Int(handle(0, 1))]).unwrap(), 0);
    assert_eq!(status(&mut s), 40, "bind after evaluate is locked");
}

#[test]
fn static_leaf_source_and_pair_registry_domains() {
    // Two static leaf sources under the stationary character: the wide floor
    // (closed touch in y, never separates) and the circle leaf (overlapping).
    // A leaf-overlapping character cannot sweep away (the motion profile
    // rejects started-in-contact movement), so the exit phase for a moving
    // subject is covered by the instance walk above; this test pins the
    // static-source registry rows, the enter/stay phases and the user read.
    let start = "\
 ok=ok+ag_trigger_bind(0, 1, 0, 2, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 0, 3, 0, 0.0)\n\
";
    let update = "\
 var p0=0\n\
 var p1=0\n\
 for ag_event_next()==1 {\n\
  if ag_event_pair()==0 {p0=ag_event_phase()}\n\
  if ag_event_pair()==1 {p1=ag_event_phase()}\n\
 }\n\
 state=p0*10+p1\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, update),
        "static-source.at",
        &rows2d(),
    )
    .expect("static session");
    assert_eq!(status(&mut s), 0);
    // Both static pairs enter at the first update: the floor touches in y
    // and the circle overlaps at the origin (closed intervals).
    tick(&mut s, 1);
    assert_eq!(event_row(&mut s, 0), [0, PHASE_ENTER, 1, 0, 2, 1, 0, 0]);
    assert_eq!(event_row(&mut s, 1), [1, PHASE_ENTER, 1, 0, 3, 2, 0, 0]);
    assert_eq!(s.int("_ag_trg_b0", &[Int(0)]).unwrap(), 1);
    assert_eq!(
        s.int("_ag_trg_b4", &[Int(0)]).unwrap(),
        1,
        "floor source run index"
    );
    assert_eq!(
        s.int("_ag_trg_b4", &[Int(1)]).unwrap(),
        2,
        "circle source run index"
    );
    tick(&mut s, 2);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        11,
        "user read of the enter tick"
    );
    // Both stay while nothing moves; the registry rows are frozen within the
    // same binding generation.
    tick(&mut s, 3);
    assert_eq!(event_row(&mut s, 0), [0, PHASE_STAY, 1, 0, 2, 1, 0, 0]);
    assert_eq!(event_row(&mut s, 1), [1, PHASE_STAY, 1, 0, 3, 2, 0, 0]);
    tick(&mut s, 4);
    assert_eq!(
        s.int("get_state", &[]).unwrap(),
        22,
        "stays read as 2, never re-enter"
    );
    assert_eq!(status(&mut s), 0);
}

#[test]
fn every_named_binding_rejection_fires_at_its_own_stage() {
    let h = handle(0, 1);
    let spawn = " h1=ag_object_spawn(0, 3, 0.0, 0.5, 0.0)\n";
    let cases: Vec<(&str, String, i32, i32)> = vec![
        (
            "pair-high",
            format!("{spawn} ok=ok+ag_trigger_bind(8, 1, 1, {h}, 0, 0.0)"),
            41,
            8,
        ),
        (
            "pair-negative",
            format!("{spawn} ok=ok+ag_trigger_bind(-1, 1, 1, {h}, 0, 0.0)"),
            41,
            0 - 1,
        ),
        (
            "subject-unknown",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 99, 1, {h}, 0, 0.0)"),
            41,
            99,
        ),
        (
            "subject-group",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 6, 1, {h}, 0, 0.0)"),
            41,
            6,
        ),
        (
            "subject-3d-decoy",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 5, 1, {h}, 0, 0.0)"),
            41,
            5,
        ),
        (
            "source-unknown",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 1, 0, 99, 0, 0.0)"),
            41,
            99,
        ),
        (
            "source-group",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 1, 0, 6, 0, 0.0)"),
            41,
            6,
        ),
        (
            "source-kind",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 1, 2, {h}, 0, 0.0)"),
            41,
            2,
        ),
        (
            "shape-unknown",
            format!("{spawn} ok=ok+ag_trigger_bind(0, 1, 1, {h}, 2, 0.0)"),
            41,
            2,
        ),
        (
            "stale-handle",
            "ok=ok+ag_trigger_bind(0, 1, 1, 12345, 0, 0.0)".into(),
            33,
            12345,
        ),
        (
            "foreign-run-handle",
            format!(
                "{spawn} ok=ok+ag_trigger_bind(0, 1, 1, {}, 0, 0.0)",
                handle(0, 1) - RUN * RUN_STRIDE
            ),
            33,
            1,
        ),
    ];
    for (name, body, code, arg) in cases {
        let mut s = triggers_session(
            Api::Triggers2V1,
            RUN,
            &source2d(&body, ""),
            &format!("case-{name}.at"),
            &rows2d(),
        )
        .unwrap_or_else(|e| panic!("{name}: session {e}"));
        assert_eq!(status(&mut s), code, "{name}: wrong fault code");
        assert_eq!(fault_arg(&mut s), arg, "{name}: wrong fault argument");
    }
    // A despawned instance handle is dead for binding (its slot generation
    // is gone).
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(
            &format!(
                "{spawn} ok=ok+ag_object_despawn(h1)\n ok=ok+ag_trigger_bind(0, 1, 1, {h}, 0, 0.0)"
            ),
            "",
        ),
        "case-despawned.at",
        &rows2d(),
    )
    .expect("despawned session");
    assert_eq!(status(&mut s), 33);
}

#[test]
fn post_evaluate_lock_rejects_geometry_writes_but_not_collection() {
    let start = "\
 h1=ag_object_spawn(0, 3, 0.0, 0.5, 0.0)\n\
 h2=ag_object_spawn(1, 4, 9.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, "ok=ok+movetox(0.0)\n"),
        "lock.at",
        &rows2d(),
    )
    .expect("lock session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(s.int("_ag_trg_seq_v", &[]).unwrap(), 1, "evaluated");
    // After evaluate: collection stays legal (visibility and despawn).
    assert_eq!(
        s.int("ag_object_visible", &[Int(handle(1, 1)), Int(0)])
            .unwrap(),
        1
    );
    assert_eq!(s.int("de", &[]).unwrap(), 1, "despawn after evaluate");
    assert_eq!(status(&mut s), 0, "the lock allows collection");
    // A fresh session: object move and spawn are locked after evaluate.
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, "ok=ok+movetox(0.0)\n"),
        "lock-move.at",
        &rows2d(),
    )
    .expect("lock move session");
    tick(&mut s, 1);
    assert_eq!(
        s.int(
            "ag_object_move",
            &[Int(handle(0, 1)), Float(1.0), Float(0.5), Float(0.0)]
        )
        .unwrap(),
        0
    );
    assert_eq!(status(&mut s), 40, "move after evaluate");
    assert_eq!(
        s.int("sp", &[]).unwrap(),
        0,
        "spawn after evaluate is locked"
    );
    // The character move is locked too (routed through the 1-arg wrapper).
    let mut s2 = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, "ok=ok+movetox(0.0)\n"),
        "lock-char.at",
        &rows2d(),
    )
    .expect("lock char session");
    tick(&mut s2, 1);
    s2.call("movetox", &[Float(1.0)], ResultKind::Unit).unwrap();
    assert_eq!(status(&mut s2), 40, "the character move is locked too");
    // The sticky error persists for the VM lifetime, so the unlock check runs
    // on a clean session: the next tick begin clears the lock and the object
    // move inside that callback succeeds.
    let unlock = "\
 ticks=ticks+1\n\
 if ticks==2 {res=ag_object_move(h2, 1.0, 0.5, 0.0)}\n\
";
    let mut s3 = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start, unlock),
        "lock-unlock.at",
        &rows2d(),
    )
    .expect("lock unlock session");
    tick(&mut s3, 1);
    tick(&mut s3, 2);
    assert_eq!(status(&mut s3), 0);
    assert_eq!(
        s3.int("get_res", &[]).unwrap(),
        1,
        "unlocked at the next tick begin"
    );
}

#[test]
fn second_evaluate_is_a_named_rejection() {
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(" ok=ok+ag_trigger_bind(0, 1, 0, 2, 0, 0.0)\n", ""),
        "double-evaluate.at",
        &rows2d(),
    )
    .expect("evaluate session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(s.int("_ag_trg_seq_v", &[]).unwrap(), 1);
    s.call("_ag_trg_evaluate", &[], ResultKind::Unit).unwrap();
    assert_eq!(status(&mut s), 40, "the second evaluate is the named lock");
}

#[test]
fn restart_request_marker_is_bounded_idempotent_and_tick0_rejected() {
    // tick 0 (app_start): the request is a named rejection.
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(
            " ok=ok+ag_trigger_bind(0, 1, 0, 2, 0, 0.0)\n req=ag_round_request()",
            "",
        ),
        "request-tick0.at",
        &rows2d(),
    )
    .expect("request session");
    assert_eq!(status(&mut s), 41, "app_start cannot request a restart");
    assert_eq!(restart_flag(&mut s), 0);
    // During an update: one pending marker, idempotent merges, and the marker
    // clears at the next tick begin.
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(
            " ok=ok+ag_trigger_bind(0, 1, 0, 2, 0, 0.0)",
            "ticks=ticks+1\nif ticks==1 {req=req+ag_round_request()\nreq=req+ag_round_request()}\n",
        ),
        "request-idem.at",
        &rows2d(),
    )
    .expect("request idem session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(s.int("get_req", &[]).unwrap(), 2, "both calls answered 1");
    assert_eq!(restart_flag(&mut s), 1, "one pending request");
    assert_eq!(s.int("_ag_trg_req_tick_v", &[]).unwrap(), 1);
    tick(&mut s, 2);
    assert_eq!(restart_flag(&mut s), 0, "the marker is per tick");
    assert_eq!(s.int("_ag_trg_req_tick_v", &[]).unwrap(), 0);
}

#[test]
fn whole_path_combination_stays_inside_the_frozen_budget() {
    // The frozen whole-path probe across ticks: a real character move rides
    // one callback with a bounded read (26+2+1 charges), then the remaining
    // events drain, then the collection (despawn+respawn+rebind) and the
    // restart request complete — every tick clean, both dimensions.
    let start2d = "\
 h1=ag_object_spawn(0, 3, 0.0, 0.5, 0.0)\n\
 h2=ag_object_spawn(1, 4, 0.0, 0.5, 0.0)\n\
 h3=ag_object_spawn(2, 3, 0.0, 0.5, 0.0)\n\
 ok=ok+ag_object_spawn(3, 3, 9.0, 0.5, 0.0)\n\
 ok=ok+ag_object_spawn(4, 4, -9.0, 0.5, 0.0)\n\
 ok=ok+ag_object_spawn(5, 3, 12.0, 0.5, 0.0)\n\
 ok=ok+ag_object_spawn(6, 4, -12.0, 0.5, 0.0)\n\
 ok=ok+ag_object_spawn(7, 3, 15.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h2, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(2, 1, 1, h3, 1, 0.5)\n\
 ok=ok+ag_trigger_bind(3, 1, 0, 2, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(4, 1, 0, 3, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(5, 1, 0, 4, 1, 0.5)\n\
 ok=ok+ag_trigger_bind(6, 1, 0, 2, 1, 0.5)\n\
 ok=ok+ag_trigger_bind(7, 1, 0, 3, 0, 0.0)\n\
";
    let update2d = "\
 ticks=ticks+1\n\
 if ticks==1 {\n\
  ok=ok+ag_char_move(0.001,0.0,0.002)\n\
  ok=ok+ag_object_alive(h2)\n\
  hits=hits+read_events()\n\
 }\n\
 if ticks==2 {\n\
  hits=hits+read_events()\n\
 }\n\
 if ticks==3 {\n\
  ok=ok+ag_object_despawn(h1)\n\
  h1=ag_object_spawn(0, 3, 4.0, 0.5, 0.0)\n\
  ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
  req=req+ag_round_request()\n\
 }\n\
";
    let mut s = triggers_session(
        Api::Triggers2V1,
        RUN,
        &source2d(start2d, update2d),
        "whole-path-2d.at",
        &rows2d(),
    )
    .expect("whole path 2d");
    assert_eq!(
        status(&mut s),
        0,
        "app_start: 8 spawns + 8 binds inside budget"
    );
    tick(&mut s, 1);
    assert_eq!(
        status(&mut s),
        0,
        "move + camera + bounded read inside budget"
    );
    assert!(event_count(&mut s) >= 3, "the overlapping pairs fired");
    assert!(event_count(&mut s) <= 8, "bounded");
    tick(&mut s, 2);
    assert_eq!(status(&mut s), 0, "the event drain stays inside budget");
    assert!(
        s.int("get_hits", &[]).unwrap() >= 3,
        "the user counted the events once each"
    );
    tick(&mut s, 3);
    assert_eq!(
        status(&mut s),
        0,
        "collection + rebind + request inside budget"
    );
    assert_eq!(
        s.int("get_req", &[]).unwrap(),
        1,
        "the restart request answered"
    );

    // The 3D mirror with the character3 chain.
    let start3d = "\
 h1=ag_object_spawn(0, 4, 0.0, 0.5, 0.0)\n\
 h2=ag_object_spawn(1, 4, 0.0, 0.5, 0.0)\n\
 h3=ag_object_spawn(2, 4, 0.0, 0.5, 0.0)\n\
 ok=ok+ag_object_spawn(3, 4, 9.0, 0.5, 0.0)\n\
 ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(1, 1, 1, h2, 1, 0.5)\n\
 ok=ok+ag_trigger_bind(2, 1, 1, h3, 0, 0.0)\n\
 ok=ok+ag_trigger_bind(3, 1, 0, 2, 0, 0.0)\n\
";
    let update3d = "\
 ticks=ticks+1\n\
 if ticks==1 {\n\
  let q=ag_char3_move(0.001,0.0,0.0,0.002)\n\
  ok=ok+ag_char3_grounded(q)\n\
  ok=ok+ag_cam3_forward_x()\n\
  hits=hits+read_events()\n\
 }\n\
 if ticks==2 {\n\
  hits=hits+read_events()\n\
 }\n\
 if ticks==3 {\n\
  ok=ok+ag_object_despawn(h1)\n\
  h1=ag_object_spawn(0, 4, 4.0, 0.5, 0.0)\n\
  ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)\n\
  req=req+ag_round_request()\n\
 }\n\
";
    let mut s3 = triggers_session(
        Api::Triggers3V1,
        RUN,
        &source3d(start3d, update3d),
        "whole-path-3d.at",
        &rows3d(),
    )
    .expect("whole path 3d");
    assert_eq!(status(&mut s3), 0);
    tick(&mut s3, 1);
    assert_eq!(
        status(&mut s3),
        0,
        "3D move + grounded + camera + bounded read"
    );
    assert!(event_count(&mut s3) >= 3, "the overlapping pairs fired");
    tick(&mut s3, 2);
    assert_eq!(status(&mut s3), 0);
    assert!(s3.int("get_hits", &[]).unwrap() >= 3);
    tick(&mut s3, 3);
    assert_eq!(status(&mut s3), 0);
    assert_eq!(
        s3.int("get_req", &[]).unwrap(),
        1,
        "the restart request answered"
    );
}
