use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    behavior::Api,
    behavior_project::{Actors, Binding, CodePayload},
    document::Kind,
    host::Host,
    play::{Action, State},
    session::Edit,
    spatial::{self, Descriptor},
    storage::{Fault, Storage},
};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
fn settle(h: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        h.frame(None).unwrap();
        assert!(Instant::now() < deadline)
    }
}
#[test]
fn twenty_real_v1_v2_v3_v4_cancel_fault_history_open_owners_and_warm_geometry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (d, a, c, _) = Storage::load_project(
        &root.join("samples/spatial_projects/zone3d/scene.agscene"),
        &Default::default(),
    )
    .unwrap();
    let mut h = Host::with_project(d, a, c).unwrap();
    h.frame(None).unwrap();
    let content_before = [
        h.rt.capture(h.two, "warm-baseline2").unwrap().0,
        h.rt.capture(h.three, "warm-baseline3").unwrap().0,
    ];
    let asset = h.editor.assets.values().next().unwrap().clone();
    let asset_weak = Arc::downgrade(&asset);
    let sha = asset.identity.sha256.clone();
    let mesh_pin = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(s) => s.store.pin(&sha, "primitive-0").unwrap(),
        _ => unreachable!(),
    };
    let mesh_weak = Arc::downgrade(&mesh_pin);
    let resources = h.resource_snapshot();
    let builds = [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq);
    let mut weak_codes = vec![];
    let mut retained_codes = vec![];
    let mut legitimate_source_pin = None;
    let folder = root.join(format!("target/sweep-owner-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    for i in 0..20 {
        let api = if i % 4 == 3 {
            Api::SweepV4
        } else if i % 5 == 0 {
            Api::V1
        } else if i % 3 == 0 {
            Api::SpatialV2
        } else {
            Api::SegmentV3
        };
        let fault = i % 5 == 2;
        let init = if api == Api::V1 {
            ""
        } else if api == Api::SweepV4 {
            "let q=ag_sweep_aabb(ag_actor(3),0,1.0,0.0,0.0);let h=ag_sweep_hit(q)"
        } else if api == Api::SegmentV3 {
            "let t=ag_cast_segment(3,ag_actor(3),0,-4.0,0.0,0.0,4.0,0.0,0.0); let h=ag_cast_hit(t)"
        } else {
            "let v=ag_aabb_min(ag_actor(3),0)"
        };
        let update = if fault && api == Api::SweepV4 {
            "let q=ag_sweep_aabb(ag_actor(3),0,1.0,0.0,0.0);let h=ag_sweep_hit(q+1)"
        } else if fault && api == Api::SegmentV3 {
            "let t=ag_cast_segment(3,-1,0,-4.0,0.0,0.0,4.0,0.0,0.0); let h=ag_cast_hit(t+1)"
        } else if fault {
            match i % 3 {
                0 => "let v=ag_aabb_contains(ag_actor(3),(-1.0).sqrt(),0.0,0.0)",
                1 => "for n in 0..33 {let v=ag_aabb_min(ag_actor(3),0)}",
                _ => "for n in 0..32 {let v=ag_aabb_min(ag_actor(3),0)}; while true {}",
            }
        } else if api == Api::V1 {
            "let a=ag_actor(3); ag_move_local(ag_parent(a),dt,0.0,0.0)"
        } else if api == Api::SweepV4 {
            "let q=ag_sweep_aabb(ag_actor(3),0,1.0,0.0,0.0);let h=ag_sweep_hit(q)"
        } else if api == Api::SegmentV3 {
            "let t=ag_cast_segment(3,ag_actor(3),0,-4.0,0.0,0.0,4.0,0.0,0.0); let h=ag_cast_hit(t)"
        } else {
            "let v=ag_aabb_min(ag_actor(3),0)"
        };
        let raw = format!(
            "//{}\nfn app_start(){{{init}}}\nfn app_update(dt float){{{update}}}\n",
            if i % 5 == 4 { "undo" } else { "shared" }
        );
        let code =
            CodePayload::from_bytes_for(Path::new("owner.at"), raw.into_bytes(), api).unwrap();
        let binding = Binding {
            api: api.version().into(),
            source: code.identity().clone(),
            actors: Actors {
                two_d: None,
                three_d: Some(1),
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
        if i % 5 == 4 {
            h.editor.begin(h.editor.stamp, Edit::Undo).unwrap();
            h.settle().unwrap();
            h.editor.begin(h.editor.stamp, Edit::Redo).unwrap();
            h.settle().unwrap();
        }
        if i % 5 == 3 {
            let entry = folder.join(format!("{i}.agscene"));
            h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
            h.open(&entry).unwrap();
            h.settle().unwrap();
        }
        let accepted = h.editor.code.as_ref().unwrap();
        weak_codes.push(Arc::downgrade(accepted));
        if legitimate_source_pin.is_none() {
            legitimate_source_pin = Some(accepted.clone());
        }
        if std::env::var_os("AG_SPATIAL_RETAIN_CODE").is_some() && i == 2 {
            retained_codes.push(accepted.clone());
        }
        let before = h.editor.isolation_identity();
        let scans = spatial::vertex_scans();
        h.start_selected_play().unwrap();
        h.pause_play("owner/cancel control");
        let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
        let geometry = h.play.as_ref().unwrap().spatial_owner_probe();
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().tick(), 0);
        assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
        assert_eq!(
            h.play.as_ref().unwrap().spatial_scans(),
            if api == Api::V1 { 0 } else { 2 }
        );
        if i % 5 != 1 {
            h.play
                .as_mut()
                .unwrap()
                .dispatch(Action::default())
                .unwrap();
            settle(&mut h);
            assert_eq!(
                h.play.as_ref().unwrap().state,
                if fault { State::Faulted } else { State::Paused }
            );
            assert_eq!(h.play.as_ref().unwrap().tick(), if fault { 0 } else { 1 });
        }
        for _ in 0..4 {
            h.frame(None).unwrap();
        }
        if std::env::var_os("AG_SPATIAL_RESCAN_WARM").is_some() {
            let mut d = h.editor.document.clone();
            d.objects.retain(|o| o.kind == Kind::Model);
            Descriptor::build(&d, |o| {
                h.editor
                    .assets
                    .get(o.resource.as_ref().unwrap())
                    .map(|a| &a.model)
            })
            .unwrap();
        }
        assert_eq!(
            spatial::vertex_scans() - scans,
            if api == Api::V1 { 0 } else { 2 },
            "warm geometry must not rescan vertices"
        );
        let retained_geometry = if std::env::var_os("AG_SPATIAL_RETAIN_DESCRIPTOR").is_some() {
            geometry.as_ref().and_then(std::sync::Weak::upgrade)
        } else {
            None
        };
        let retained_vm = if std::env::var_os("AG_SPATIAL_RETAIN_VM").is_some() && !fault {
            h.play.take()
        } else {
            None
        };
        h.stop_play();
        assert!(!owner(), "runtime VM must release at Stop");
        assert!(
            geometry.as_ref().is_none_or(|w| w.upgrade().is_none()),
            "spatial descriptor must release at Stop"
        );
        assert_eq!(h.editor.isolation_identity(), before);
        if std::env::var_os("AG_SWEEP_WRONG_CONTENT").is_some() {
            h.asset_render_fault = Some("transparent".into());
        }
        h.frame(None).unwrap();
        let current_content = [
            h.rt.capture(h.two, "warm-current2").unwrap().0,
            h.rt.capture(h.three, "warm-current3").unwrap().0,
        ];
        assert_eq!(
            current_content, content_before,
            "warm canvas content must match accepted editing scene"
        );

        if std::env::var_os("AG_SPATIAL_FRESH_UPLOAD").is_some() {
            let mut stage = match h.rt.content(h.three).unwrap() {
                SurfaceContent::Scene3d(c) => c.store.staged(),
                _ => unreachable!(),
            };
            stage.retain_models(|s| s != sha);
            for part in 0..asset.model.meshes.len() {
                stage.get_or_upload(h.rt.gpu(), &asset.model, part).unwrap();
            }
            if let SurfaceContent::Scene3d(c) = h.rt.content_mut(h.three).unwrap() {
                c.store = stage;
            }
        }
        let current = match h.rt.content(h.three).unwrap() {
            SurfaceContent::Scene3d(c) => c.store.pin(&sha, "primitive-0").unwrap(),
            _ => unreachable!(),
        };
        assert!(
            Arc::ptr_eq(&mesh_pin, &current),
            "warm model must retain actual GPU Arc"
        );
        let after = h.resource_snapshot();
        for key in ["target_bytes", "target_count", "mesh_uploads"] {
            assert_eq!(resources[key], after[key], "stable resource {key}");
        }
        assert_eq!(
            builds,
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq)
        );
        eprintln!(
            "{}",
            serde_json::json!({"cycle":i,"api":api.version(),"fault":fault,"cancel_start":i%5==1,"open":i%5==3,"undo_redo":i%5==4,"vertex_scans":spatial::vertex_scans()-scans,"vm_released":!owner(),"geometry_released":geometry.as_ref().is_none_or(|w|w.upgrade().is_none()),"resources":after})
        );
        drop(retained_vm);
        drop(retained_geometry);
    }
    h.editor.close();
    drop(h);
    assert!(
        weak_codes
            .iter()
            .filter_map(std::sync::Weak::upgrade)
            .all(|c| Arc::ptr_eq(&c, legitimate_source_pin.as_ref().unwrap())),
        "only authorized external source pin may survive"
    );
    drop(legitimate_source_pin);
    assert!(
        weak_codes.iter().all(|w| w.upgrade().is_none()),
        "project source must release after last owner"
    );
    assert!(asset_weak.upgrade().is_some());
    drop(asset);
    assert!(
        asset_weak.upgrade().is_none(),
        "resource must release after last owner"
    );
    assert!(mesh_weak.upgrade().is_some());
    drop(mesh_pin);
    assert!(
        mesh_weak.upgrade().is_none(),
        "mesh must release after last owner"
    );
    drop(retained_codes);
}
