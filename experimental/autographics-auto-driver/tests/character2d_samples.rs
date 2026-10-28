//! PLAN-861 sample-project acceptance (contract V-05 work scope): both sample
//! layouts import through the production loader, the SAME user source binds
//! them (no id/name hardcoding), geometry changes the reachable set, and the
//! 4 -> 5 speed source edit changes the run. When CHARACTER2D_REPORT_DIR is
//! set, the fresh layouts report is written for tools/check_character2d.py.
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    hierarchy,
    spatial::Descriptor,
};
use autographics_auto_driver::{
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const STEPS: usize = 300;

fn sample(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/character2d")
        .join(name)
}

fn source_of(name: &str) -> String {
    std::fs::read_to_string(sample(name)).expect("sample source")
}

#[test]
fn sample_projects_import_and_the_same_source_sees_two_geometries() {
    // Production import path for both layouts (scene/2 strict import).
    let (doc_a, _sa) =
        autographics_auto_driver::authoring::storage::Storage::load(&sample("layout_a.agscene"))
            .expect("layout a import");
    let (doc_b, _sb) =
        autographics_auto_driver::authoring::storage::Storage::load(&sample("layout_b.agscene"))
            .expect("layout b import");
    assert_eq!(doc_a.objects.len(), 5);
    assert_eq!(doc_b.objects.len(), 5);

    // The frozen speed-4 source: right-held run on both layouts. Layout A
    // ground is continuous to the wall; layout B has a pit between the two
    // ground segments, so the identical run must fall (geometry, not ids).
    let user = source_of("main.at");
    // The source must be re-admitted per run (fresh VM): rebuild per layout.
    let path_a = {
        let mut s = {
            let snap = SourceSnapshot::from_bytes_for(
                Path::new("run-a.at"),
                user.as_bytes().to_vec(),
                Api::CharacterV1,
            )
            .unwrap();
            let mut snap = snap;
            snap.source.push_str("\nfn probe_x() float {return _ag_worlds[0]}\nfn probe_y() float {return _ag_worlds[1]}\n");
            build_session(&doc_a, &snap.source)
        };
        run_right(&mut s)
    };
    let path_b = {
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("run-b.at"),
            user.as_bytes().to_vec(),
            Api::CharacterV1,
        )
        .unwrap();
        let mut snap = snap;
        snap.source.push_str("\nfn probe_x() float {return _ag_worlds[0]}\nfn probe_y() float {return _ag_worlds[1]}\n");
        let mut s = build_session(&doc_b, &snap.source);
        run_right(&mut s)
    };
    // Layout A: stays on the ground the whole way (no pit on the path).
    let a_min_y = path_a.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        a_min_y > -1.0,
        "layout A min {a_min_y} head={:?}",
        &path_a[..5]
    );
    // Layout B: the identical input falls into the pit (y far below ground).
    let b_min_y = path_b.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        b_min_y < -3.0,
        "layout B must fall into the pit (min {b_min_y}) tail={:?} end_x={}",
        &path_b[path_b.len() - 8..],
        path_b[path_b.len() - 1][0]
    );

    // Speed 4 -> 5 on the SAME layout: the edited source really runs faster.
    let fast = source_of("main-fast.at");
    assert!(fast.contains("var speed=5.0") && !fast.contains("var speed=4.0"));
    let path_fast = {
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("run-fast.at"),
            fast.as_bytes().to_vec(),
            Api::CharacterV1,
        )
        .unwrap();
        let mut snap = snap;
        snap.source.push_str(
            "
fn probe_x() float {return _ag_worlds[0]}
fn probe_y() float {return _ag_worlds[1]}
",
        );
        let mut s = build_session(&doc_a, &snap.source);
        run_right(&mut s)
    };
    assert!(
        path_fast[59][0] > path_a[59][0] * 1.2,
        "speed edit must move faster: {} vs {}",
        path_fast[59][0],
        path_a[59][0]
    );

    // Fresh report for the checker when requested.
    if let Ok(dir) = std::env::var("CHARACTER2D_REPORT_DIR") {
        let exe = std::env::current_exe().unwrap();
        let runner_sha = agp::renderer::capture::sha256_hex(&std::fs::read(&exe).unwrap());
        let report = serde_json::json!({
            "schema": "autographics.character-layouts/1",
            "runner_sha256": runner_sha,
            "completed": true,
            "user_source": "samples/character2d/main.at",
            "speed_source": "samples/character2d/main-fast.at",
            "steps": STEPS,
            "layout_a": {"min_y": a_min_y, "fell_in_pit": false, "path_tail": path_a[path_a.len()-32..]},
            "layout_b": {"min_y": b_min_y, "fell_in_pit": true, "path_tail": path_b[path_b.len()-32..]},
            "speed": {"x60_slow": path_a[59][0], "x60_fast": path_fast[59][0]},
        });
        std::fs::write(
            Path::new(&dir).join("character-layouts-report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .expect("write layouts report");
    }
}

fn build_session(
    doc: &autographics_auto_driver::authoring::document::Document,
    combined: &str,
) -> Session {
    let mut s = Session::new(combined).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let descriptor = Descriptor::build(doc, |_o| None).expect("descriptor");
    let order = hierarchy::ordered(doc).expect("order");
    s.call(
        "_ag_spatial_reset",
        &[Int(descriptor.rows().len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (index, value) in descriptor.rows().iter().flatten().enumerate() {
        s.call(
            "_ag_spatial_config",
            &[Int(index as i32), Float(*value)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    for (i, n) in order.iter().enumerate() {
        let parent = n
            .parent
            .and_then(|p| order.iter().position(|o| o.id == p))
            .map(|v| v as i32)
            .unwrap_or(-1);
        s.call(
            "_ag_identity",
            &[
                Int(i as i32 + 1),
                Int(n.kind),
                Int(n.dimension.code()),
                Int(parent),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        s.call(
            "_ag_visibility",
            &[Int(i32::from(n.visible))],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        // Field layout per play.rs: local xyz + scale xyz + yaw + pitch + rgba.
        let o = doc.objects.iter().find(|o| o.id == n.id).expect("object");
        let fields: Vec<f32> = n
            .local
            .iter()
            .copied()
            .chain(o.scale.iter().copied())
            .chain([o.yaw, o.pitch])
            .chain(o.rgba.iter().copied())
            .collect();
        for v in fields {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        s.call(
            "_ag_char_map",
            &[Int(i32::try_from(n.id).unwrap()), Int(i as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "freeze");
    s
}

fn run_right(s: &mut Session) -> Vec<[f32; 2]> {
    let mut path = Vec::with_capacity(STEPS);
    for step in 1..=STEPS {
        s.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit)
            .unwrap();
        s.call(
            "_ag_tick",
            &[Int(step as i32), Float(1.0 / 60.0)],
            ResultKind::Unit,
        )
        .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "tick {step}");
        path.push([
            s.float("probe_x", &[]).unwrap(),
            s.float("probe_y", &[]).unwrap(),
        ]);
    }
    path
}
