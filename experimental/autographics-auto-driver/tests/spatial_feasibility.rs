//! PLAN-846 T02 bounded assembly probe. Not a shipped v2 entrypoint.
use autographics_auto_driver::{
    authoring::behavior::SourceSnapshot, ResultKind, Scalar::*, Session,
};
use std::path::Path;

const QUERIES: &str = r#"
var _sq_boxes=[]
var _sq_calls=0
fn _sq_config(value float){_sq_boxes.push(value)}
fn _sq_attempt(node int) int {
 _sq_calls=_sq_calls+1
 if _sq_calls>32 {_ag_fail(10,node,0); return 0}
 if _ag_check(node)==0 {return 0}
 if kinds[node]==5 {_ag_fail(6,node,0); return 0}
 if _sq_boxes[node*7]==0.0 {_ag_fail(9,node,0); return 0}
 if _ag_dirty==1 {_ag_derive(); _ag_dirty=0}
 return 1
}
fn _sq_edge(node int, axis int, high int) float {return _ag_worlds[node*3+axis]+_sq_boxes[node*7+1+high*3+axis]}
fn ag_aabb_min(node int, axis int) float {
 if _sq_attempt(node)==0 {return 0.0}
 if _ag_axis(node,axis)==0 {return 0.0}
 return _sq_edge(node,axis,0)
}
fn ag_aabb_max(node int, axis int) float {
 if _sq_attempt(node)==0 {return 0.0}
 if _ag_axis(node,axis)==0 {return 0.0}
 return _sq_edge(node,axis,1)
}
fn ag_aabb_contains(node int, x float, y float, z float) int {
 if _sq_attempt(node)==0 {return 0}
 if _ag_dimensions[node]==2 {if z!=0.0 {_ag_fail(7,node,2); return 0}}
 if x!=x {_ag_fail(7,node,0); return 0}
 if y!=y {_ag_fail(7,node,1); return 0}
 if z!=z {_ag_fail(7,node,2); return 0}
 if x<_sq_edge(node,0,0) {return 0}; if x>_sq_edge(node,0,1) {return 0}
 if y<_sq_edge(node,1,0) {return 0}; if y>_sq_edge(node,1,1) {return 0}
 if _ag_dimensions[node]==3 {
 if z<_sq_edge(node,2,0) {return 0}; if z>_sq_edge(node,2,1) {return 0}
 }
 return 1
}
fn ag_aabb_overlaps(a int, b int) int {
 if _sq_attempt(a)==0 {return 0}
 if _ag_check(b)==0 {return 0}
 if _ag_kinds[b]==5 {_ag_fail(6,b,0); return 0}
 if _sq_boxes[b*7]==0.0 {_ag_fail(9,b,0); return 0}
 if _ag_dimensions[a]!=_ag_dimensions[b] {_ag_fail(8,b,0); return 0}
 let dims=_ag_dimensions[a]
 for axis in 0..dims {
 if _sq_edge(a,axis,1)<_sq_edge(b,axis,0) {return 0}
 if _sq_edge(b,axis,1)<_sq_edge(a,axis,0) {return 0}
 }
 return 1
}
"#;
fn source(update: &str) -> String {
    let user = format!(
        "var observed=0.0\nvar hits=0\nfn app_start(){{}}\nfn app_update(dt float){{{update}}}\n"
    );
    let s = SourceSnapshot::from_bytes(Path::new("probe.at"), user.into_bytes()).unwrap();
    // Only trusted assembly is changed. V1 original source and admission stay intact.
    let q = QUERIES.replace("kinds[node]", "_ag_kinds[node]");
    let entry = s.regions[2].start;
    let assembled = format!("{}\n{}\n{}", &s.source[..entry], q, &s.source[entry..]);
    format!("{}\n{}\nfn probe_tick(){{_sq_calls=0; _ag_tick(1,0.016666668)}}\nfn probe_observed() float {{return observed}}\nfn probe_hits() int {{return hits}}\nfn probe_calls() int {{return _sq_calls}}\n",assembled, "")
}
fn configured(update: &str, deep: bool) -> Session {
    let mut s = Session::new(&source(update)).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    for n in 0..128 {
        let group = if deep { n < 126 } else { n == 0 };
        let parent = if n == 0 {
            -1
        } else if deep {
            n.min(126) - 1
        } else {
            0
        };
        s.call(
            "_ag_identity",
            &[
                Int(n + 1),
                Int(if group { 5 } else { 2 }),
                Int(3),
                Int(parent),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 1.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for v in [if group { 0. } else { 1. }, -0.5, -0.5, -0.5, 0.5, 0.5, 0.5] {
            s.call("_sq_config", &[Float(v)], ResultKind::Unit).unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}
#[test]
fn wide_and_deep_128_nodes_32_mixed_queries_two_parent_writes_original_budget() {
    let update = r#"
 ag_set_local(0,0,2.0)
 observed=ag_aabb_min(126,0)
 for i in 0..8 {if ag_aabb_overlaps(126,127)==1 {hits=hits+1}}
 for i in 0..11 {observed=ag_aabb_max(126,0)}
 ag_set_local(0,0,4.0)
 for i in 0..12 {if ag_aabb_contains(126,4.0,0.0,0.0)==1 {hits=hits+1}}
 "#;
    for deep in [false, true] {
        let mut s = configured(update, deep);
        for name in ["ag_aabb_overlaps", "ag_aabb_contains"] {
            let args = if name == "ag_aabb_overlaps" {
                vec![Int(126), Int(127)]
            } else {
                vec![Int(126), Float(0.), Float(0.), Float(0.)]
            };
            eprintln!("direct {name}: {:?}", s.int(name, &args));
        }
        let result = s.call("probe_tick", &[], ResultKind::Unit);
        eprintln!("deep={deep}, tick={result:?}, yields={}", s.yielded);
        result.unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("probe_calls", &[]).unwrap(), 32);
        assert_eq!(s.int("probe_hits", &[]).unwrap(), 20);
        assert_eq!(s.float("probe_observed", &[]).unwrap(), 2.5);
    }
}

#[test]
fn explicit_v2_types_budget_and_v1_helper_control() {
    use autographics_auto_driver::authoring::behavior::{Api, API};
    let control=b"fn ag_aabb_min(n int, a int) float {return 17.0}\nfn app_start(){}\nfn app_update(dt float){}\n".to_vec();
    let old = SourceSnapshot::from_bytes(Path::new("old.at"), control.clone()).unwrap();
    let mut v1 = Session::new(&old.source).unwrap();
    v1.call("main", &[], ResultKind::Unit).unwrap();
    assert_eq!(v1.float("ag_aabb_min", &[Int(0), Int(0)]).unwrap(), 17.0);
    assert!(SourceSnapshot::from_bytes_for(Path::new("old.at"), control, Api::SpatialV2).is_err());
    assert_eq!(
        old.api_hash,
        agp::renderer::capture::sha256_hex(API.as_bytes())
    );
    let user=b"var hits=0\nfn app_start(){}\nfn app_update(dt float){ag_set_local(0,0,2.0); for i in 0..8 {if ag_aabb_overlaps(126,127)==1 {hits=hits+1}}; for i in 0..12 {let bound=ag_aabb_min(126,0)}; ag_set_local(0,0,4.0); for i in 0..12 {if ag_aabb_contains(126,4.0,0.0,0.0)==1 {hits=hits+1}}}\n";
    for deep in [false, true] {
        let snap =
            SourceSnapshot::from_bytes_for(Path::new("v2.at"), user.to_vec(), Api::SpatialV2)
                .unwrap();
        let mut s = Session::new(&snap.source).unwrap();
        s.call("main", &[], ResultKind::Unit).unwrap();
        s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_spatial_reset", &[Int(128)], ResultKind::Unit)
            .unwrap();
        for n in 0..128 {
            let group = if deep { n < 126 } else { n == 0 };
            let parent = if n == 0 {
                -1
            } else if deep {
                n.min(126) - 1
            } else {
                0
            };
            s.call(
                "_ag_identity",
                &[
                    Int(n + 1),
                    Int(if group { 5 } else { 2 }),
                    Int(3),
                    Int(parent),
                ],
                ResultKind::Unit,
            )
            .unwrap();
            s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
                .unwrap();
            s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
                .unwrap();
            for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 1.] {
                s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                    .unwrap();
            }
            for (f, v) in [if group { 0. } else { 1. }, -0.5, -0.5, -0.5, 0.5, 0.5, 0.5]
                .into_iter()
                .enumerate()
            {
                s.call(
                    "_ag_spatial_config",
                    &[Int(n * 7 + f as i32), Float(v)],
                    ResultKind::Unit,
                )
                .unwrap();
            }
        }
        s.call("_ag_start", &[], ResultKind::Unit).unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        let before = s.yielded;
        let tick = s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit);
        eprintln!(
            "production v2 deep={deep} tick={tick:?} callback_yields={}",
            s.yielded - before
        );
        tick.unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("_ag_version", &[]).unwrap(), 2);
        assert_eq!(s.int("_ag_profile", &[]).unwrap(), 8);
        // A 33rd direct public query before the next callback must retain QueryBudget.
        assert_eq!(s.int("ag_aabb_overlaps", &[Int(126), Int(127)]).unwrap(), 0);
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 10);
    }
}

#[test]
fn actual_v2_host_multimesh_query_drives_gpu_and_stop() {
    use autographics_auto_driver::authoring::{
        assets,
        behavior::Api,
        document::{Dimension, Document, Group, Kind, Object},
        host::Host,
    };
    use std::time::{Duration, Instant};
    let asset = assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap();
    assert_eq!(asset.model.meshes.len(), 2);
    let mut d = Document::empty("spatial-feasibility".into());
    let mut g = Group::new(10, Dimension::Three);
    g.translation = [1., 0., 0.];
    d.groups.push(g);
    let mut m = Object::new(11, Kind::Model);
    m.parent = Some(10);
    m.translation = [-1., 0., 0.];
    m.resource = Some(asset.identity.key());
    d.objects.push(m);
    let mut q = Object::new(12, Kind::Quad);
    q.translation = [-100., 0., 0.];
    d.objects.push(q);
    d.assets.push(asset.identity.clone());
    let mut h =
        Host::with_assets(d, assets::Registry::from([(asset.identity.key(), asset)])).unwrap();
    h.frame(None).unwrap();
    let original = (
        h.rt.capture(h.two, "spatial-original2").unwrap().0,
        h.rt.capture(h.three, "spatial-original3").unwrap().0,
    );
    // Parent-first ordinal0=group, 1=model, 2=indicator. Geometry derives from
    // the same Registry payload rendered by Host; no box is supplied by this script.
    let user=b"fn app_start(){}\nfn app_update(dt float){ag_set_local(0,0,4.0); let bound=ag_aabb_max(1,0); ag_set_local(2,0,-100.0+bound); if ag_aabb_contains(1,3.0,0.5,0.0)==0 {ag_set_visible(1,0)}}\n";
    let source =
        SourceSnapshot::from_bytes_for(Path::new("zone3d.at"), user.to_vec(), Api::SpatialV2)
            .unwrap();
    // Accepted v2 project, save and reopen before executing the same whole path.
    use autographics_auto_driver::authoring::{
        behavior_project::{Actors, Binding, CodePayload},
        session::Edit,
        storage::{Fault, Storage},
    };
    let code =
        CodePayload::from_bytes_for(Path::new("zone3d.at"), source.bytes.clone(), Api::SpatialV2)
            .unwrap();
    let binding = Binding {
        api: Api::SpatialV2.version().into(),
        source: code.identity().clone(),
        actors: Actors {
            two_d: Some(12),
            three_d: Some(11),
        },
    };
    h.editor
        .begin(
            h.editor.stamp,
            Edit::Behavior {
                binding: Some(binding),
                code: Some(code),
            },
        )
        .unwrap();
    h.settle().unwrap();
    let root = std::env::temp_dir().join(format!("ag846-feasibility-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let entry = root.join("scene.agscene");
    h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
    let (doc, assets, code, _) = Storage::load_project(&entry, &Default::default()).unwrap();
    let mut h = Host::with_project(doc, assets, code).unwrap();
    h.frame(None).unwrap();
    let identity = h.editor.isolation_identity();
    h.start_selected_play().unwrap();
    h.pause_play("");
    let settle = |h: &mut Host| {
        let deadline = Instant::now() + Duration::from_secs(31);
        while h.play.as_ref().unwrap().pending() {
            h.frame(None).unwrap();
            assert!(Instant::now() < deadline);
        }
    };
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().tick(), 0);
    let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Default::default())
        .unwrap();
    settle(&mut h);
    let f = h.play.as_ref().unwrap().last.as_ref().unwrap();
    assert_eq!(f.tick, 1);
    assert_eq!(f.objects[0].translation[0], 3.0);
    assert!(f.objects[0].visible);
    // Known all-mesh fixture occupies [-2,2] before translation (inspect vertices).
    let expected = h
        .editor
        .assets
        .values()
        .next()
        .unwrap()
        .model
        .meshes
        .iter()
        .flat_map(|m| m.positions.iter())
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max)
        + 3.0;
    assert!((f.document.objects[1].translation[0] - (-100.0 + expected)).abs() < 1e-5);
    for (i, dx) in [-1.5, 1.5].into_iter().enumerate() {
        let p = autographics_auto_driver::authoring::interaction::project(
            h.camera(),
            glam::Vec3::new(3.0 + dx, 0.5, 0.),
        )
        .unwrap();
        let pixel = h.rt.probe_pixel(h.top, [p.x, p.y + 400.]).unwrap();
        assert!(pixel[i] > 80 && pixel[1 - i] < 20, "part{i} {pixel:?}");
    }
    assert_ne!(
        h.rt.capture(h.two, "spatial-runtime2").unwrap().0,
        original.0
    );
    h.stop_play();
    h.frame(None).unwrap();
    assert!(!owner());
    assert_eq!(h.editor.isolation_identity(), identity);
    assert_eq!(
        (
            h.rt.capture(h.two, "spatial-restored2").unwrap().0,
            h.rt.capture(h.three, "spatial-restored3").unwrap().0
        ),
        original
    );
}
