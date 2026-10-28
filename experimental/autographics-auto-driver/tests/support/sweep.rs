use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::*,
    Session,
};
use std::path::Path;
fn scene(update: &str, rows: &[(i32, i32, [f32; 7])]) -> Session {
    scene_guard(update, rows, false)
}
fn scene_guard(update: &str, rows: &[(i32, i32, [f32; 7])], deep: bool) -> Session {
    scene_custom(update, rows, deep, |s| s)
}
fn scene_custom(
    update: &str,
    rows: &[(i32, i32, [f32; 7])],
    deep: bool,
    change: impl FnOnce(String) -> String,
) -> Session {
    let src = format!("var observed=0\nfn app_start(){{}}\nfn app_update(dt float){{{update}}}\n");
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("spatial-api.at"),
        src.into_bytes(),
        Api::SweepV4,
    )
    .unwrap();
    let mut source = snap.source.clone();
    source.push_str("\nfn probe_observed() int {return observed}\nfn probe_credit() int {return _ag_spatial_calls}\nfn probe_token() int {return _ag_cast_token}\n");
    source.push_str("\nfn probe_exhaust() {_ag_cast_seq=2147483647}\n");
    source.push_str("\nfn probe_direct_x(n int) float {return _ag_worlds[n*3]}\nfn probe_direct_y(n int) float {return _ag_worlds[n*3+1]}\nfn probe_direct_z(n int) float {return _ag_worlds[n*3+2]}\n");
    let source = change(source);
    let folder = Path::new("target/sweep-feasibility-sources");
    std::fs::create_dir_all(folder).unwrap();
    let hash = agp::renderer::capture::sha256_hex(source.as_bytes());
    std::fs::write(folder.join(format!("{hash}.at")), &source).unwrap();
    let mut s = Session::new(&source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, dim, row)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[
                Int(n as i32 + 1),
                Int(*kind),
                Int(*dim),
                Int(if n == 0 {
                    -1
                } else if deep {
                    n as i32 - 1
                } else if rows[0].0 == 5 {
                    0
                } else {
                    -1
                }),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in row.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}
