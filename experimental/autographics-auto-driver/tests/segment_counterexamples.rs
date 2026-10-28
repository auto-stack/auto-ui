// Exact R1 input replay, changed only from diagnostic failure assertions to acceptance.
include!("support/segment.rs");
#[test]
fn review_128_wide_late_axis_and_progressively_nearer() {
    let update="ag_set_local(0,0,2.0); let a=ag_cast_segment(3,-1,0,0.0,0.0,0.0,800.0,0.0,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,4.0); let b=ag_cast_segment(3,-1,0,0.0,0.0,0.0,800.0,0.0,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(127,0)}";
    for mode in ["equal", "late-z-miss", "decreasing-t"] {
        let mut rows = vec![(2, 3, [1., -1., -1., -1., 1., 1., 1.]); 128];
        rows[0] = (5, 3, [0.; 7]);
        for i in 1..128 {
            if mode == "late-z-miss" {
                rows[i].2 = [1., 1., -1., 1., 2., 1., 2.]
            };
            if mode == "decreasing-t" {
                let x = 500. - i as f32;
                rows[i].2 = [1., x, -1., -1., x + 0.5, 1., 1.]
            }
        }
        let mut s = scene(update, &rows);
        let before = s.yielded;
        let result = s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
        eprintln!(
            "REVIEW mode={mode} result={result:?} slices={}",
            s.yielded - before
        );
        assert!(result.is_ok(), "{mode}")
    }
}
#[test]
fn review_real_host_128_geometry_distribution() {
    use autographics_auto_driver::authoring::{
        document::{Dimension, Document, Group, Kind, Object},
        host::Host,
        play::{Action, State},
    };
    use std::time::{Duration, Instant};
    fn settle(h: &mut Host) {
        let end = Instant::now() + Duration::from_secs(31);
        while h.play.as_ref().unwrap().pending() {
            h.frame(None).unwrap();
            assert!(Instant::now() < end)
        }
    }
    for mode in ["equal", "late-z-miss", "decreasing-t"] {
        let mut d = Document::empty(format!("review-128-{mode}"));
        d.groups.push(Group::new(1, Dimension::Three));
        for i in 1..128 {
            let mut o = Object::new(i + 1, Kind::Cylinder);
            o.parent = Some(1);
            o.translation = match mode {
                "late-z-miss" => [10., 0., 10.],
                "decreasing-t" => [500. - i as f32, 0., 0.],
                _ => [10., 0., 0.],
            };
            d.objects.push(o)
        }
        let update="ag_set_local(0,0,2.0); let a=ag_cast_segment(3,-1,0,0.0,0.5,0.0,800.0,0.5,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,4.0); let b=ag_cast_segment(3,-1,0,0.0,0.5,0.0,800.0,0.5,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(127,0)}";
        let source = format!("fn app_start(){{}}\nfn app_update(dt float){{{update}}}");
        let mut h = Host::new(d).unwrap();
        h.start_user_snapshot(
            SourceSnapshot::from_bytes_for(
                Path::new("review128.at"),
                source.into_bytes(),
                Api::SegmentV3,
            )
            .unwrap(),
        )
        .unwrap();
        h.pause_play("");
        settle(&mut h);
        assert_eq!(
            h.play.as_ref().unwrap().state,
            State::Paused,
            "initialization"
        );
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .unwrap();
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        eprintln!(
            "HOST mode={mode} state={:?} tick={} error={:?}",
            p.state,
            p.tick(),
            p.error
        );
        assert_eq!(p.state, State::Paused, "{mode}");
        assert_eq!(p.tick(), 1);
        h.stop_play();
    }
}
