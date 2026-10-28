//! PLAN-874 sample-project acceptance (contract V-05 work scope): both 3D
//! sample layouts import through the production loader, the SAME user source
//! binds them (no id/name hardcoding), the geometry changes the reachable set
//! (platform vs pit) and the 4 -> 5 speed source edit changes the run. When
//! CHARACTER3D_REPORT_DIR is set, the fresh layouts report is written for
//! tools/check_character3d.py.
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind},
    hierarchy,
    spatial::Descriptor,
};
use autographics_auto_driver::{
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const STEPS: usize = 120;

fn sample(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/character3d")
        .join(name)
}

fn source_of(name: &str) -> String {
    std::fs::read_to_string(sample(name)).expect("sample source")
}

/// Builtin cylinder bounds are the unit cube [-1,1]^3, so a node's AABB is its
/// scale; the sample character is the frozen 0.8/1.0/0.8 box.
fn cylinder_model() -> agp::assets::ModelData {
    agp::assets::ModelData {
        source_sha256: "character3d-sample-cylinder".into(),
        meshes: vec![agp::render3d::cylinder_mesh()],
    }
}

#[test]
fn sample_projects_import_and_the_same_source_sees_two_three_dimensional_geometries() {
    let (doc_a, _sa) =
        autographics_auto_driver::authoring::storage::Storage::load(&sample("layout_a.agscene"))
            .expect("layout a import");
    let (doc_b, _sb) =
        autographics_auto_driver::authoring::storage::Storage::load(&sample("layout_b.agscene"))
            .expect("layout b import");
    assert_eq!(doc_a.objects.len(), 3);
    assert_eq!(doc_b.objects.len(), 4);
    for doc in [&doc_a, &doc_b] {
        assert_eq!(doc.objects[0].kind, Kind::Cylinder);
        assert_eq!(doc.objects[0].scale, [0.4, 0.5, 0.4]);
    }

    // The frozen speed-4 source: forward held on both layouts. Layout A ground
    // is continuous through the walked path; layout B has a pit in front, so
    // the identical run must fall (geometry, not ids).
    let user = source_of("main.at");
    let path_a = run(&doc_a, &user, "run-a.at");
    let path_b = run(&doc_b, &user, "run-b.at");
    let a_min_y = path_a.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        a_min_y > -1.0,
        "layout A must stay on the ground (min {a_min_y})"
    );
    let b_min_y = path_b.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        b_min_y < -3.0,
        "layout B must fall into the pit (min {b_min_y})"
    );
    // Same source, two geometries: the paths really diverge.
    assert_ne!(
        path_a[STEPS - 1][1].to_bits(),
        path_b[STEPS - 1][1].to_bits()
    );

    // Speed 4 -> 5 on the SAME layout: the edited source really runs faster.
    let fast = source_of("main-fast.at");
    assert!(fast.contains("var speed=5.0") && !fast.contains("var speed=4.0"));
    let path_fast = run(&doc_a, &fast, "run-fast.at");
    let slow = path_a[59][2].abs();
    let quick = path_fast[59][2].abs();
    assert!(
        quick > slow * 1.2,
        "speed edit must move faster: {quick} vs {slow}"
    );

    if let Ok(dir) = std::env::var("CHARACTER3D_REPORT_DIR") {
        let exe = std::env::current_exe().unwrap();
        let runner_sha = agp::renderer::capture::sha256_hex(&std::fs::read(&exe).unwrap());
        let report = serde_json::json!({
            "schema": "autographics.character3d-layouts/1",
            "runner_sha256": runner_sha,
            "completed": true,
            "user_source": "samples/character3d/main.at",
            "speed_source": "samples/character3d/main-fast.at",
            "steps": STEPS,
            "layout_a": {"min_y": a_min_y, "fell_in_pit": false, "path_tail": tail(&path_a)},
            "layout_b": {"min_y": b_min_y, "fell_in_pit": true, "path_tail": tail(&path_b)},
            "speed": {"z60_slow": path_a[59][2], "z60_fast": path_fast[59][2]},
        });
        std::fs::write(
            Path::new(&dir).join("character3d-layouts-report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .expect("write character3d layouts report");
    }
}

fn tail(path: &[[f32; 3]]) -> Vec<[f32; 3]> {
    path[path.len() - 32..].to_vec()
}

fn run(doc: &Document, user: &str, name: &str) -> Vec<[f32; 3]> {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        user.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .expect("admission");
    let mut snap = snap;
    snap.source.push_str(
        "
fn probe_x() float {return _ag_worlds[0]}
fn probe_y() float {return _ag_worlds[1]}
fn probe_z() float {return _ag_worlds[2]}
",
    );
    let mut s = build_session(doc, &snap.source);
    let mut path = Vec::with_capacity(STEPS);
    for step in 1..=STEPS {
        // Forward held (vendor-neutral action code 2) for the whole run.
        s.call("_ag_char3_input", &[Int(4), Int(0)], ResultKind::Unit)
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
            s.float("probe_z", &[]).unwrap(),
        ]);
    }
    path
}

fn build_session(doc: &Document, combined: &str) -> Session {
    let model = cylinder_model();
    let descriptor = Descriptor::build(doc, |_o| Some(&model)).expect("descriptor");
    let order = hierarchy::ordered(doc).expect("order");
    let mut s = Session::new(combined).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
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
            "_ag_char3_map",
            &[Int(i32::try_from(n.id).unwrap()), Int(i as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "freeze");
    s
}
