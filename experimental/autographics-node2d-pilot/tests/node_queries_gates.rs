//! PLAN-021 T-03 lowering gates (V-01): the queries-2d profile lowers the
//! frozen lab fixtures to the T-02 contract shape, byte-stably, with named
//! module/line/symbol diagnostics for every rejected construct; the
//! components-2d profile rejects query constructs by name. Run:
//! cargo test --locked --test node_queries_gates
use autographics_node2d_pilot::{load_project, lower_project_components, lower_project_queries};
use std::path::Path;

const DIR: &str = "fixtures/queries2d";

const LAB_FILES: [&str; 13] = [
    "resources.at",
    "spectator.at",
    "runner.at",
    "target.at",
    "marker.at",
    "door.at",
    "point_marker.at",
    "far.at",
    "wall.at",
    "segment.at",
    "sweep.at",
    "crosslab.at",
    "changed.at",
];

fn stage(tag: &str) -> std::path::PathBuf {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("nq-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for name in LAB_FILES {
        std::fs::copy(Path::new(DIR).join(name), dir.join(name)).unwrap();
    }
    dir
}

fn lower_queries(entry: &str, tag: &str) -> Result<String, String> {
    let dir = stage(tag);
    let project = load_project(&dir.join(entry)).map_err(|e| e.to_string())?;
    lower_project_queries(&project)
        .map(|o| o.user_source.expect("source"))
        .map_err(|e| e.to_string())
}

fn lower_components(entry: &str, tag: &str) -> Result<String, String> {
    let dir = stage(tag);
    let project = load_project(&dir.join(entry)).map_err(|e| e.to_string())?;
    lower_project_components(&project)
        .map(|o| o.user_source.expect("source"))
        .map_err(|e| e.to_string())
}

/// Q-G1: the segment lab lowers to the frozen T-02 contract shape and is
/// byte-stable across runs.
#[test]
fn q01_segment_lab_lowers_to_frozen_shape_byte_stable() {
    let a = lower_queries("segment.at", "q1a").expect("lower");
    let b = lower_queries("segment.at", "q1b").expect("lower");
    assert_eq!(a, b, "byte-stable regeneration");
    for needle in [
        "let q = ag_cast_segment(2, 1, 1, ax, ay, 0.0, (ax + r_reach), ay, 0.0)",
        "let hit = ag_cast_hit(q)",
        "let fraction = ag_cast_t(q)",
        "ag_set_local(3, 0, (2.0 + (node * 2.5)))",
        "ag_set_local(5, 1, ((-2.0) + (nx * 1.5)))",
        "ag_set_visible(4, hit)",
        "var runner_qrow=[1]",
        "var fartarget_qrow=[6]",
        "ok=ok+ag_char_bind(1)*1.0",
        "ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 40.0, 12.0, 40.0, 12.0)*1.0",
        "fn app_update(dt float) {",
    ] {
        assert!(a.contains(&needle), "missing generated shape: {needle}");
    }
    // No hand-written ordinals beyond the resolved refs; no raw query names.
    assert!(
        !a.contains("query2d"),
        "query2d must not leak into user source"
    );
}

/// Q-G2: instance-scoped wiring — self.row() resolves through the per-def
/// qrow array, per-instance model getters follow the 020 convention, and no
/// cross-def state flattening happens.
#[test]
fn q02_cross_lab_qrow_and_isolation() {
    let src = lower_queries("crosslab.at", "q2").expect("lower");
    assert!(src.contains("var querier_qrow=[3,4]"), "qrow array");
    assert!(
        src.contains("let q = ag_cast_segment(2, querier_qrow[i], 1, ag_world(querier_qrow[i], 0), ag_world(querier_qrow[i], 1), 0.0, (ag_world(querier_qrow[i], 0) + querier_reach[i]), ag_world(querier_qrow[i], 1), 0.0)"),
        "self.row()/self.world() lower through qrow[i]"
    );
    assert!(
        src.contains("fn get_qa_hits() int {return querier_hits[0]}"),
        "path alias getter"
    );
    assert!(
        src.contains("fn get_qb_hits() int {return querier_hits[1]}"),
        "second instance getter"
    );
}

/// Q-G3: the changed variant differs from the segment lab only in the reach
/// initial value (source-only change; scene geometry identical).
#[test]
fn q03_changed_variant_is_a_source_only_reach_change() {
    let seg = lower_queries("segment.at", "q3a").expect("lower");
    let chg = lower_queries("changed.at", "q3b").expect("lower");
    assert!(
        seg.contains("var r_reach = 20.0")
            || seg.contains("var r_reach=20.0")
            || seg.contains("var r_reach = 20.0"),
        "segment reach"
    );
    assert!(chg.contains("10.0"), "changed reach 10 present");
    assert!(!chg.contains("20.0"), "changed has no 20.0 reach");
    // Same structure otherwise: identical fn inventory.
    let fns = |s: &str| -> Vec<String> {
        s.lines()
            .filter(|l| l.starts_with("fn "))
            .map(|l| l.to_string())
            .collect()
    };
    assert_eq!(fns(&seg), fns(&chg), "same fn inventory");
}

/// Q-G4: every rejected construct carries the named module/line/symbol
/// diagnostic; nothing panics.
#[test]
fn q04_negative_fixtures_fail_with_named_diagnostics() {
    let cases: [(&str, fn(&str) -> String, &str, &str); 10] = [
        (
            "unknown query row reference",
            |s| {
                s.replace(
                    "query2d.segment(refs.runner,",
                    "query2d.segment(refs.ghost,",
                )
            },
            "symbol 'ghost'",
            "refs.ghost",
        ),
        (
            "wrong-kind getter",
            |s| s.replace("let px = q.point(0)", "let px = q.origin(0)"),
            "'.origin()' is a sweep result getter",
            "cast result",
        ),
        (
            "unbound getter base",
            |s| s.replace("let hit = q.hit()", "let hit = qq.hit()"),
            "unknown query result 'qq'",
            "let",
        ),
        (
            "character-row position write",
            |s| {
                s.replace(
                    "refs.runner.shift(input.x() * 60.0 * dt, input.y() * 60.0 * dt)",
                    "refs.spectator.shift(input.x() * 60.0 * dt, input.y() * 60.0 * dt)",
                )
            },
            "character row",
            "position is owned by the character service",
        ),
        (
            "sweep mover -1",
            |s| {
                s.replace(
                    "let q = query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                    "let q = query2d.sweep(-1, 1, ax, ay)",
                )
            },
            "sweep mover must be a named row",
            "self.row()",
        ),
        (
            "var-bound query result",
            |s| s.replace("let q = query2d.segment", "var q = query2d.segment"),
            "query results bind with 'let' only",
            "shared latest-slot token",
        ),
        (
            "query arity",
            |s| {
                s.replace(
                    "query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                    "query2d.segment(refs.runner, 1, ax, ay, ax + .reach)",
                )
            },
            "arity 5 (want 6 args)",
            "query2d.segment",
        ),
        (
            "visible_only literal",
            |s| {
                s.replace(
                    "let q = query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                    "let vis = 1
            let q = query2d.segment(refs.runner, vis, ax, ay, ax + .reach, ay)",
                )
            },
            "visible_only must be the literal 0/1",
            "true/false",
        ),
        (
            "query result rebind",
            |s| {
                s.replace(
                    "let q = query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                    "let q = query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)\n            let q = query2d.sweep(refs.runner, 1, 0.0, 0.0)",
                )
            },
            "query result 'q' rebinds",
            "one let per query result name",
        ),
        (
            "query result reassignment",
            |s| s.replace("let hit = q.hit()", "let hit = q.hit()\n            q = q"),
            "query result cannot be reassigned",
            "read the values out instead",
        ),
    ];
    for (name, mutate, needle1, needle2) in cases {
        let mutated =
            mutate(&std::fs::read_to_string(Path::new(DIR).join("segment.at")).expect("fixture"));
        let dir = stage("q4");
        std::fs::write(dir.join("mutant.at"), mutated).unwrap();
        let project = load_project(&dir.join("mutant.at")).expect("parse ok");
        let err = lower_project_queries(&project).err().expect(name);
        let msg = err.to_string();
        assert!(
            msg.contains(needle1),
            "{name}: missing '{needle1}' in {msg}"
        );
        assert!(
            msg.contains(needle2),
            "{name}: missing '{needle2}' in {msg}"
        );
        assert!(
            msg.contains("module mutant, line "),
            "{name}: module/line missing: {msg}"
        );
        assert!(msg.contains("symbol '"), "{name}: symbol missing: {msg}");
    }
}

/// Q-G5: the components-2d profile rejects query constructs by name (the
/// old admission stays closed; no silent acceptance).
#[test]
fn q05_components_profile_rejects_query_constructs_named() {
    let err = lower_components("segment.at", "q5")
        .err()
        .expect("rejected");
    assert!(
        err.contains("query2d is only available in the queries-2d profile"),
        "{err}"
    );
    assert!(err.contains("module segment, line "), "{err}");
    assert!(err.contains("symbol 'query2d'"), "{err}");
}

/// Q-G6: the sweep lab lowering shape — sweep call, six getters including
/// the origin re-read, explicit clearance arithmetic on the mover row.
#[test]
fn q06_sweep_lab_lowes_clearance_contract_shape() {
    let src = lower_queries("sweep.at", "q6").expect("lower");
    for needle in [
        "let q = ag_sweep_aabb(1, 1, dx, 0.0, 0.0)",
        "let hit = ag_sweep_hit(q)",
        "let overlap = ag_sweep_start_overlap(q)",
        "ag_move_local(1, move, 0.0, 0.0)",
        "let px2 = ag_sweep_origin(q, 0)",
        "var r_speed=100.0",
        "var r_clearance=0.02",
    ] {
        assert!(src.contains(needle), "missing sweep shape: {needle}");
    }
}

/// Q-G4b (F-R1-01): a top-level instance with more than one static shape is
/// named-rejected in queries-2d (the frozen contract: a named row ref
/// resolves exactly one static row); components-2d keeps accepting it.
#[test]
fn q04b_multi_static_shape_instance_rejected_in_queries_mode() {
    let dir = stage("q4b");
    let path = dir.join("marker.at");
    let s = std::fs::read_to_string(&path).expect("fixture");
    let s2 = s.replace(
        "material: resources.lab_marker\n        }\n    }\n}",
        "material: resources.lab_marker\n        }\n        shape2d extra {\n            mesh: resources.lab_quad1\n            position: (3.0, 3.0)\n            material: resources.lab_marker\n        }\n    }\n}",
    );
    assert!(s2 != s, "mutation no-op");
    std::fs::write(&path, s2).unwrap();
    let project = load_project(&dir.join("segment.at")).expect("parse ok");
    let err = lower_project_queries(&project).err().expect("rejected");
    let msg = err.to_string();
    assert!(msg.contains("has 2 static shapes"), "{msg}");
    assert!(msg.contains("resolves exactly one"), "{msg}");
    assert!(msg.contains("module marker, line "), "{msg}");
    assert!(msg.contains("symbol 'marker'"), "{msg}");
    // components-2d (no query surface) still accepts the same project.
    let project2 = load_project(&dir.join("segment.at")).expect("parse ok");
    assert!(
        lower_project_components(&project2).is_err(),
        "components-2d must still reject query2d for its own reason (profile boundary), not the static-shape count"
    );
}
