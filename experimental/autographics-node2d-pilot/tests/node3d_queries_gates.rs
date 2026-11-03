//! PLAN-039 T-03/T-04 lowering gates (V-S01): the queries-3d profile lowers
//! the frozen dual-exit lab fixtures and rejects every out-of-contract
//! construct with named module/line/symbol diagnostics — the limited dual
//! view (root-only view-2d markers), the query3d/query2d profile gates,
//! group/marker row typing (groups are not queryable geometry), arities,
//! literal-only flags, let-only results, and the marker z=0 rule. The old
//! profiles keep their parse-time dual-view rejection byte-identically.
//! Run:
//! cargo test --locked --test node3d_queries_gates
use autographics_node2d_pilot::{
    load_project, load_project_dual, lower_project_components3d, lower_project_queries,
    lower_project_queries3d,
};
use std::path::Path;

const DIR: &str = "fixtures/queries3d";

fn stage(tag: &str, subdir: &str) -> std::path::PathBuf {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("nq3-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    for name in ["resources.at", "main.at"] {
        std::fs::copy(Path::new(DIR).join(subdir).join(name), dir.join(name)).unwrap();
    }
    std::fs::copy(
        Path::new(DIR).join(subdir).join("assets/two-parts.glb"),
        dir.join("assets/two-parts.glb"),
    )
    .unwrap();
    dir
}

fn lower_q3(entry: &str, tag: &str) -> Result<String, String> {
    let dir = stage(tag, "q3-seg");
    let project = load_project_dual(&dir.join(entry), true).map_err(|e| format!("{e:?}"))?;
    let lowered = lower_project_queries3d(&project).map_err(|e| e.to_string())?;
    Ok(lowered.scene_json)
}

fn reject_q3_src(src: &str, tag: &str, needle: &str) {
    let dir = stage(tag, "q3-seg");
    std::fs::write(dir.join("main.at"), src).unwrap();
    let project = load_project_dual(&dir.join("main.at"), true)
        .map_err(|e| format!("{e:?}"))
        .expect("dual-view parse succeeds on the queries-3d path");
    let err = match lower_project_queries3d(&project) {
        Err(e) => format!("{e:?}"),
        Ok(_) => panic!("expected a named rejection containing '{needle}'"),
    };
    assert!(
        err.contains(needle),
        "expected rejection containing '{needle}', got: {err}"
    );
}

fn reject_q3(replace: (&str, &str), tag: &str, needle: &str) {
    let mut src = SEG_SRC.to_string();
    assert!(src.contains(replace.0), "fixture anchor missing: {}", replace.0);
    src = src.replace(replace.0, replace.1);
    reject_q3_src(&src, tag, needle);
}

const SEG_SRC: &str = include_str!("../fixtures/queries3d/q3-seg/main.at");
const SWEEP_SRC: &str = include_str!("../fixtures/queries3d/q3-sweep/main.at");

fn reject_sweep(replace: (&str, &str), tag: &str, needle: &str) {
    assert!(SWEEP_SRC.contains(replace.0), "fixture anchor missing: {}", replace.0);
    let src = SWEEP_SRC.replace(replace.0, replace.1);
    let dir = stage(tag, "q3-sweep");
    std::fs::write(dir.join("main.at"), src).unwrap();
    let project = load_project_dual(&dir.join("main.at"), true)
        .map_err(|e| format!("{e:?}"))
        .expect("dual-view parse succeeds on the queries-3d path");
    let err = match lower_project_queries3d(&project) {
        Err(e) => format!("{e:?}"),
        Ok(_) => panic!("expected a named rejection containing '{needle}'"),
    };
    assert!(
        err.contains(needle),
        "expected rejection containing '{needle}', got: {err}"
    );
}

#[test]
fn frozen_dual_exit_lab_lowers_with_content_derived_api() {
    let json = lower_q3("main.at", "frozen").expect("q3-seg lowers");
    assert!(json.contains("autographics.behavior/3"), "segment-only api");
    assert!(json.contains("\"kind\": \"quad\""), "2D marker rows present");
    assert!(json.contains("\"dimension\": \"3d\""), "3d group present");
}

#[test]
fn old_profiles_still_reject_dual_view_at_parse() {
    let dir = stage("olddialect", "q3-seg");
    let err = load_project(&dir.join("main.at"))
        .err()
        .expect("default dialect rejects dual view")
        .to_string();
    assert!(
        err.contains("declares both view-2d and view-3d"),
        "old rejection text: {err}"
    );
    // And the components-3d profile (a plain single-view project) still
    // lowers untouched.
    let dir = stage("c3d", "q3-seg");
    std::fs::write(
        dir.join("main.at"),
        "use resources: parts_glb\n\nnode Solo {\n    view-3d {\n        model3d m {\n            model: resources.parts_glb\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n}\n",
    )
    .unwrap();
    let project = load_project(&dir.join("main.at")).expect("solo project");
    lower_project_components3d(&project).expect("components-3d lowers");
}

#[test]
fn view2d_is_root_only_in_queries3d() {
    // A child component carrying view-2d is rejected by name (instantiated
    // in the root view so the root stays unique).
    let mut src = SEG_SRC.to_string();
    src = src.replace(
        "use resources: parts_glb\n",
        "use resources: parts_glb\n\nnode Child {\n    view-3d {\n        cylinder3d stub {\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n    view-2d {\n        shape2d rogue {\n            kind: quad\n        }\n    }\n}\n",
    );
    src = src.replace(
        "    view-3d {\n        group3d movers {",
        "    view-3d {\n        Child kid {\n        }\n        group3d movers {",
    );
    reject_q3_src(&src, "child2d", "view-2d is only allowed on the queries-3d root node");
}

#[test]
fn query3d_and_query2d_are_profile_gated_both_ways() {
    reject_q3(
        (
            "query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay, az)",
            "query2d.segment(refs.actor, true, ax, ay, ax + .reach, ay)",
        ),
        "q2inq3",
        "query2d is only available in the queries-2d profile",
    );

    // query3d under the queries-2d profile is rejected by name (existing
    // frozen 2D lab with the service name swapped).
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("nq3-q3inq2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "resources.at", "spectator.at", "runner.at", "target.at", "marker.at", "door.at",
        "point_marker.at", "far.at", "wall.at", "segment.at",
    ] {
        std::fs::copy(Path::new("fixtures/queries2d").join(name), dir.join(name)).unwrap();
    }
    let seg = std::fs::read_to_string(dir.join("segment.at")).unwrap();
    let swapped = seg
        .replace("query2d.segment(", "query3d.segment(")
        .replace(", ax, ay, ax + .reach, ay)", ", 0.0, ax, ay, ax + .reach, ay, 0.0)");
    assert!(swapped.contains("query3d.segment("), "swap applied");
    std::fs::write(dir.join("segment.at"), swapped).unwrap();
    let project = load_project(&dir.join("segment.at")).expect("parse (2D lab has no dual view)");
    let err = match lower_project_queries(&project) {
        Err(e) => format!("{e:?}"),
        Ok(_) => panic!("expected query3d under queries-2d to be rejected"),
    };
    assert!(
        err.contains("query3d is only available in the queries-3d profile"),
        "got: {err}"
    );
}

#[test]
fn groups_and_markers_never_enter_query_handles() {
    reject_q3(
        (
            "query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay, az)",
            "query3d.segment(refs.movers, true, ax, ay, az, ax + .reach, ay, az)",
        ),
        "groupignore",
        "groups are not queryable geometry",
    );
    reject_sweep(
        (
            "let q = query3d.sweep(refs.actor, true, dx, 0.0, 0.0)",
            "let q = query3d.sweep(refs.movers, true, dx, 0.0, 0.0)",
        ),
        "groupmover",
        "groups are not queryable geometry",
    );
    reject_q3(
        (
            "query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay, az)",
            "query3d.segment(refs.marker, true, ax, ay, az, ax + .reach, ay, az)",
        ),
        "markerhandle",
        "view-2d marker row",
    );
    reject_q3(
        (
            "query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay, az)",
            "query3d.segment(refs.ghost, true, ax, ay, az, ax + .reach, ay, az)",
        ),
        "ghost",
        "unknown query row reference",
    );
}

#[test]
fn arities_flags_and_result_typing_are_named_rejections() {
    reject_q3(
        (
            "let q = query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay, az)",
            "let q = query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay)",
        ),
        "arity",
        "query3d.segment arity 7",
    );
    reject_sweep(
        (
            "let q = query3d.sweep(refs.actor, true, dx, 0.0, 0.0)",
            "let q = query3d.sweep(-1, true, dx, 0.0, 0.0)",
        ),
        "moverminus",
        "sweep mover must be a named row",
    );
    reject_q3(
        (
            "let q = query3d.segment(refs.actor, true, ax, ay, az, ax + .reach, ay, az)",
            "let q = query3d.segment(refs.actor, ax, ax, ay, az, ax + .reach, ay, az)",
        ),
        "voliteral",
        "visible_only must be the literal",
    );
    reject_q3(
        (
            "let q = query3d.segment(refs.actor, true,",
            "var q = query3d.segment(refs.actor, true,",
        ),
        "varbind",
        "bind with 'let' only",
    );
    reject_q3(
        ("let px = q.point(0)", "let px = q.start_overlap()"),
        "wrongkind",
        "is a sweep result getter",
    );
}

#[test]
fn marker_z_stays_at_the_literal_zero() {
    reject_q3(
        (
            "refs.marker.place(-100.0 + node * 5.0, 50.0 + fraction * 10.0, 0.0)",
            "refs.marker.place(-100.0 + node * 5.0, 50.0 + fraction * 10.0, 2.0)",
        ),
        "markz",
        "keeps z at the literal 0",
    );
    // A 3D group shift may move z freely.
    let mut src = SEG_SRC.to_string();
    src = src.replace(
        "refs.movers.shift(input.x() * 3.0 * dt, 0.0, 0.0 - input.y() * 3.0 * dt)",
        "refs.movers.shift(input.x() * 3.0 * dt, 0.0, 0.0 - input.y() * 3.0 * dt + 1.0)",
    );
    let dir = stage("shiftz", "q3-seg");
    std::fs::write(dir.join("main.at"), &src).unwrap();
    let project = load_project_dual(&dir.join("main.at"), true).expect("parse");
    lower_project_queries3d(&project).expect("3D group shift z is free");
}
