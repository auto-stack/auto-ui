use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::{Action, State},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn fixture() -> Host {
    let mut d = Document::empty("spatial-host".into());
    d.objects.push(Object::new(1, Kind::Circle));
    Host::new(d).unwrap()
}
fn settle(h: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        h.frame(None).unwrap();
        assert!(Instant::now() < deadline)
    }
}
#[test]
fn actual_host_128_wide_deep_sweep_initialization_and_query_budget() {
    use autographics_auto_driver::authoring::document::{Dimension, Group};
    for dim in [2, 3] {
        for deep in [false, true] {
            let mut d = Document::empty(format!("128-{deep}"));
            let groups = if deep { 126 } else { 3 };
            for i in 0..groups {
                let mut g = Group::new(
                    i + 1,
                    if dim == 2 {
                        Dimension::Two
                    } else {
                        Dimension::Three
                    },
                );
                if i > 0 {
                    g.parent = Some(if deep { i } else { 1 })
                };
                d.groups.push(g);
            }
            for i in groups..128 {
                let mut o = Object::new(
                    i + 1,
                    if dim == 2 {
                        Kind::Circle
                    } else {
                        Kind::Cylinder
                    },
                );
                o.parent = Some(if deep {
                    groups
                } else if i % 2 == 0 {
                    2
                } else {
                    3
                });
                d.objects.push(o);
            }
            let mut h = Host::new(d).unwrap();
            let raw=b"fn app_start(){}\nfn app_update(dt float){ag_set_local(0,0,2.0);let q=ag_move_and_slide(126,0,8.0,0.0,0.0,0.25);let s=ag_motion_status(q);let c=ag_motion_contacts(q);let x=ag_motion_delta(q,0);let y=ag_motion_delta(q,1);let z=ag_motion_delta(q,2);let nx=ag_motion_normal(q,0);let ny=ag_motion_normal(q,1);let nz=ag_motion_normal(q,2);ag_set_local(0,0,4.0);if s!=3 || c!=0 {ag_set_visible(127,0)}}\n";
            let start = Instant::now();
            h.start_user_snapshot(
                SourceSnapshot::from_bytes_for(Path::new("128.at"), raw.to_vec(), Api::MotionV5)
                    .unwrap(),
            )
            .unwrap();
            h.pause_play("");
            settle(&mut h);
            let p = h.play.as_ref().unwrap();
            assert_eq!(p.state, State::Paused, "{:?}", p.error);
            assert_eq!(p.spatial_configuration_calls, 897);
            assert!(start.elapsed() < Duration::from_secs(30));
            eprintln!(
                "128 deep={deep} init_us={} spatial_calls={} config_us={}",
                start.elapsed().as_micros(),
                p.spatial_configuration_calls,
                p.spatial_configuration_us
            );
            h.play
                .as_mut()
                .unwrap()
                .dispatch(Action::default())
                .unwrap();
            settle(&mut h);
            let p = h.play.as_ref().unwrap();
            assert_eq!(p.state, State::Paused, "{:?}", p.error);
            assert_eq!(p.tick(), 1);
            assert!(p.last.as_ref().unwrap().objects.iter().all(|o| o.visible));
            assert_eq!(p.last.as_ref().unwrap().objects[0].translation[0], 4.);
            h.stop_play();
        }
    }
}

#[test]
fn callback_fault_keeps_last_good_closes_vm_stop_and_new_run_recover() {
    let mut h = fixture();
    h.frame(None).unwrap();
    let original = h.editor.isolation_identity();
    let pix = h.rt.capture(h.two, "initial").unwrap().0;
    let raw=b"fn app_start(){}\nfn app_update(dt float){ag_move_local(0,1.0,0.0,0.0); if ag_tick()==2 {let x=ag_cast_hit(1)} else {let x=ag_aabb_min(0,0)}}\n";
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(Path::new("fault.at"), raw.to_vec(), Api::MotionV5).unwrap(),
    )
    .unwrap();
    h.pause_play("");
    let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    let spatial = h.play.as_ref().unwrap().spatial_owner_probe().unwrap();
    settle(&mut h);
    for tick in 1..3 {
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .unwrap();
        settle(&mut h);
        if tick == 1 {
            assert_eq!(h.play.as_ref().unwrap().tick(), 1);
        }
    }
    let p = h.play.as_ref().unwrap();
    assert_eq!(p.state, State::Faulted);
    assert_eq!(p.tick(), 1);
    assert_eq!(
        p.last.as_ref().unwrap().document.objects[0].translation[0],
        1.
    );
    assert!(p.error.as_ref().unwrap().contains("InvalidResult"));
    assert!(!owner());
    assert!(h.select_behavior_api(Api::V1).is_err());
    h.stop_play();
    h.frame(None).unwrap();
    assert!(spatial.upgrade().is_none());
    assert_eq!(h.editor.isolation_identity(), original);
    assert_eq!(h.rt.capture(h.two, "restored").unwrap().0, pix);
    let raw = b"fn app_start(){let x=ag_aabb_min(0,0)}\nfn app_update(dt float){}\n";
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(Path::new("fixed.at"), raw.to_vec(), Api::MotionV5).unwrap(),
    )
    .unwrap();
    h.pause_play("cancel starting");
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
    assert_eq!(h.play.as_ref().unwrap().tick(), 0);
    h.stop_play();
}

#[test]
fn actual_host_128_two_contacts_parent_writes_final_world() {
    use autographics_auto_driver::authoring::document::{Dimension, Group};
    for dim in [2, 3] {
        for deep in [false, true] {
            let groups = if deep { 125 } else { 3 };
            let mut d = Document::empty(format!("motion-corner-{dim}-{deep}"));
            for i in 0..groups {
                let mut g = Group::new(
                    i + 1,
                    if dim == 2 {
                        Dimension::Two
                    } else {
                        Dimension::Three
                    },
                );
                if i > 0 {
                    g.parent = Some(if deep { i } else { 1 });
                }
                d.groups.push(g);
            }
            for i in groups..128 {
                let mut o = Object::new(i + 1, if dim == 2 { Kind::Quad } else { Kind::Cylinder });
                o.parent = Some(if deep {
                    groups
                } else {
                    if i % 2 == 0 {
                        2
                    } else {
                        3
                    }
                });
                o.scale = if dim == 2 { [2., 2., 1.] } else { [2., 2., 2.] };
                if i > groups {
                    o.translation[0] = 4.1;
                    o.scale = if dim == 2 {
                        [0.2, 40., 1.]
                    } else {
                        [0.2, 40., 40.]
                    };
                }
                if i == 127 {
                    o.translation = [0., 6.1, 0.];
                    o.scale = if dim == 2 {
                        [40., 0.2, 1.]
                    } else {
                        [40., 0.2, 40.]
                    };
                }
                d.objects.push(o);
            }
            let actor_ordinal = autographics_auto_driver::authoring::hierarchy::ordered(&d)
                .unwrap()
                .iter()
                .position(|n| n.id == groups + 1)
                .unwrap();
            let mut h = Host::new(d).unwrap();
            let raw=format!("fn app_start(){{}}\nfn app_update(dt float){{ag_set_local(0,0,2.0);let q=ag_move_and_slide({actor_ordinal},0,10.0,10.0,0.0,0.25);let s=ag_motion_status(q);let c=ag_motion_contacts(q);let x=ag_motion_delta(q,0);let y=ag_motion_delta(q,1);let z=ag_motion_delta(q,2);let nx=ag_motion_normal(q,0);let ny=ag_motion_normal(q,1);let nz=ag_motion_normal(q,2);ag_set_local(1,1,3.0);if s!=2 || c!=2 {{ag_set_visible({actor_ordinal},0)}}}}\n");
            h.start_user_snapshot(
                SourceSnapshot::from_bytes_for(
                    Path::new("corner.at"),
                    raw.into_bytes(),
                    Api::MotionV5,
                )
                .unwrap(),
            )
            .unwrap();
            h.pause_play("");
            settle(&mut h);
            h.play
                .as_mut()
                .unwrap()
                .dispatch(Action::default())
                .unwrap();
            settle(&mut h);
            let p = h.play.as_ref().unwrap();
            assert_eq!(p.state, State::Paused, "{:?}", p.error);
            let actor = &p.last.as_ref().unwrap().objects[0];
            assert!((p.last.as_ref().unwrap().objects[1].translation[1] - 3.).abs() < 0.001);
            assert!(actor.visible);
            let stop = if dim == 2 { 4.75 } else { 3.65 };
            assert!(
                (actor.translation[0] - stop).abs() < 0.001,
                "{:?}",
                actor.translation
            );
            let expected_y = stop + if deep || groups % 2 == 0 { 3.0 } else { 0.0 };
            assert!(
                (actor.translation[1] - expected_y).abs() < 0.001,
                "{:?}",
                actor.translation
            );
            h.stop_play();
        }
    }
}

#[test]
fn second_motion_budget_fault_rejects_first_motion_entire_frame() {
    let mut h = fixture();
    let original = h.editor.isolation_identity();
    h.start_user_snapshot(SourceSnapshot::from_bytes_for(Path::new("two.at"),b"fn app_start(){}\nfn app_update(dt float){let a=ag_move_and_slide(0,0,10.0,0.0,0.0,0.25);let b=ag_move_and_slide(0,0,10.0,0.0,0.0,0.25)}\n".to_vec(),Api::MotionV5).unwrap()).unwrap();
    h.pause_play("");
    settle(&mut h);
    let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    let baseline = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .unwrap()
        .document
        .clone();
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .unwrap();
    settle(&mut h);
    let p = h.play.as_ref().unwrap();
    assert_eq!(p.state, State::Faulted);
    assert!(p.error.as_ref().unwrap().contains("QueryBudget"));
    assert_eq!(p.last.as_ref().unwrap().document, baseline);
    assert!(!owner());
    h.stop_play();
    assert_eq!(h.editor.isolation_identity(), original);
}
