use autographics_auto_driver::authoring::{
    document::{Dimension, Document, Group, Kind, Object},
    play::{Action, Input, Key, Play, State, SOURCE},
    session::{Edit, Editor},
};
use std::time::{Duration, Instant};
fn fixture() -> Editor {
    let mut d = Document::empty("play-test".into());
    let mut g = Group::new(100, Dimension::Two);
    g.translation = [7.0, 11.0, 0.0];
    d.groups.push(g);
    let mut a = Object::new(1, Kind::Quad);
    a.translation = [-20.0, 5.0, 0.0];
    a.parent = Some(100);
    d.objects.push(a);
    let mut b = Object::new(2, Kind::Cylinder);
    b.translation = [1.0, 2.0, 3.0];
    d.objects.push(b);
    Editor::new(d).unwrap()
}
fn ready(p: &mut Play) -> Result<(), String> {
    let limit = Instant::now() + Duration::from_secs(31);
    while p.candidate().is_none() {
        if Instant::now() > limit {
            return Err("test deadline".into());
        }
        p.pump_until(Instant::now() + Duration::from_millis(8))?;
    }
    p.accept()
}
fn started(e: &Editor, source: &str) -> Play {
    let mut p = Play::new(e, [Some(1), Some(2)], source).unwrap();
    ready(&mut p).unwrap();
    p
}
#[test]
fn retained_600_tick_trace_f32_reference_and_replay() {
    let mut e = fixture();
    let mut o = e.document.objects[0].clone();
    o.translation[0] += 2.0;
    e.select(Some(1)).unwrap();
    e.begin(e.stamp, Edit::Replace(o)).unwrap();
    while !e.pump().unwrap() {}
    e.commit().unwrap();
    e.begin(e.stamp, Edit::Undo).unwrap();
    while !e.pump().unwrap() {}
    e.commit().unwrap();
    let before = e.isolation_identity();
    assert_eq!(e.history().1, 1);
    let mut expected = e.document.clone();
    let mut first = vec![];
    for replay in 0..2 {
        let mut p = started(&e, SOURCE);
        let alive = p.owner_probe().unwrap();
        if replay == 1 {
            expected = e.document.clone();
        }
        for i in 1..=600 {
            let a = Action {
                space: if i % 3 == 0 { 3 } else { 2 },
                x: if i % 4 < 2 { 1 } else { -1 },
                y: if i % 5 == 0 { 1 } else { 0 },
                toggle: i % 37 == 0,
            };
            let o = &mut expected.objects[if a.space == 2 { 0 } else { 1 }];
            let speed = if a.space == 2 { 60.0 } else { 3.0 };
            o.translation[0] += a.x as f32 * speed * (1.0 / 60.0);
            if a.space == 2 {
                o.translation[1] += a.y as f32 * speed * (1.0 / 60.0);
            } else {
                o.translation[2] -= a.y as f32 * speed * (1.0 / 60.0);
            }
            if a.toggle {
                o.visible = !o.visible;
            }
            p.dispatch(a).unwrap();
            ready(&mut p).unwrap();
            assert_eq!(p.tick(), i);
            let f = p.last.as_ref().unwrap();
            for (a, b) in f.document.objects.iter().zip(&expected.objects) {
                for axis in 0..3 {
                    assert!(
                        (a.translation[axis] - b.translation[axis]).abs() < 1e-3,
                        "tick{i} {a:?} != {b:?}"
                    );
                }
                assert_eq!(a.visible, b.visible);
            }
            let derived =
                autographics_auto_driver::authoring::hierarchy::derive(&expected).unwrap();
            assert_eq!(f.nodes, derived);
            let hash = f.document.hash().unwrap();
            if replay == 0 {
                first.push(hash);
            } else {
                assert_eq!(first[i as usize - 1], hash);
            }
        }
        assert_eq!(p.reports.len(), 601);
        assert_eq!(e.isolation_identity(), before);
        p.stop();
        assert!(!alive());
    }
}
#[test]
fn input_axes_parity_repeat_focus_and_reset() {
    let mut i = Input::default();
    i.key(Key::Left, true, false);
    assert_eq!(i.sample(), Action::default());
    i.focus(2);
    i.key(Key::Right, true, false);
    i.key(Key::Up, true, false);
    i.key(Key::Toggle, true, false);
    i.key(Key::Toggle, true, true);
    assert_eq!(
        i.sample(),
        Action {
            space: 2,
            x: 1,
            y: 1,
            toggle: true
        }
    );
    i.key(Key::Toggle, true, false);
    i.key(Key::Toggle, true, false);
    assert!(!i.sample().toggle);
    i.focus(3);
    assert_eq!(i.sample().x, 0);
    i.key(Key::Right, true, true);
    assert_eq!(i.sample().x, 0);
    i.key(Key::Right, true, false);
    i.clear();
    i.key(Key::Right, true, true);
    assert_eq!(i.sample().x, 0);
}
#[test]
fn pause_step_pending_stop_restart_gap_and_backlog() {
    let e = fixture();
    let mut p = started(&e, SOURCE);
    p.dispatch(Action {
        space: 2,
        x: 1,
        ..Default::default()
    })
    .unwrap();
    p.pause("");
    assert_eq!(p.state, State::Pausing);
    ready(&mut p).unwrap();
    assert_eq!(p.state, State::Paused);
    assert_eq!(p.tick(), 1);
    p.single_step().unwrap();
    assert!(p.single_step().is_err());
    ready(&mut p).unwrap();
    assert_eq!(p.tick(), 2);
    assert_eq!(p.state, State::Paused);
    let doc = p.last.as_ref().unwrap().document.clone();
    p.clock(Instant::now() + Duration::from_secs(100));
    assert_eq!(p.tick(), 2);
    p.resume().unwrap();
    let now = Instant::now();
    p.clock(now);
    p.clock(now + Duration::from_millis(300));
    assert_eq!(p.state, State::Paused);
    assert!(p.error.as_ref().unwrap().contains("gap"));
    p.resume().unwrap();
    p.clock(now);
    p.clock(now + Duration::from_millis(100));
    assert_eq!(p.state, State::Paused);
    assert!(p.error.as_ref().unwrap().contains("backlog"));
    assert_eq!(p.last.as_ref().unwrap().document, doc);
    p.resume().unwrap();
    p.dispatch(Action::default()).unwrap();
    let alive = p.owner_probe().unwrap();
    p.stop();
    assert!(!alive());
    assert!(p.candidate().is_none());
    assert!(p.dispatch(Action::default()).is_err());
    let fresh = started(&e, SOURCE);
    assert_ne!(fresh.run_id, p.run_id);
    assert_eq!(fresh.tick(), 0);
    assert_eq!(fresh.last.unwrap().document, e.document);
}
#[test]
fn malformed_entire_output_faults_no_partial_and_releases_owner() {
    let e = fixture();
    for (from, to, why) in [
        (
            "return run",
            "if sequence>0 {return run-1}; return run",
            "profile/run/tick/count",
        ),
        (
            "return sequence",
            "if sequence>0 {return sequence+1}; return sequence",
            "profile/run/tick/count",
        ),
        (
            "return identities.len()",
            "if sequence>0 {return 1}; return identities.len()",
            "profile/run/tick/count",
        ),
        (
            "return resources[i]",
            "if sequence>0 {return -50}; return resources[i]",
            "resource",
        ),
        (
            "return parents[i]",
            "if sequence>0 {return 0}; return parents[i]",
            "parent",
        ),
        (
            "return kinds[i]",
            "if sequence>0 {return 100}; return kinds[i]",
            "kind",
        ),
        (
            "return identities[i]",
            "if sequence>0 {return 99}; return identities[i]",
            "identity",
        ),
        (
            "return dimensions[i]",
            "if sequence>0 {return 0}; return dimensions[i]",
            "dimension",
        ),
        (
            "return flags[i]",
            "if sequence>0 {return 2}; return flags[i]",
            "visibility",
        ),
        (
            "return effective[i]",
            "if sequence>0 {return 1-effective[i]}; return effective[i]",
            "effective",
        ),
        (
            "return worlds[i*3+f]",
            "if sequence>0 {return worlds[i*3+f]+1.0}; return worlds[i*3+f]",
            "world",
        ),
        (
            "return fields[i*12+f]",
            "if sequence>0 {return 100001.0}; return fields[i*12+f]",
            "fields",
        ),
    ] {
        assert!(SOURCE.contains(from));
        let source = SOURCE.replace(from, to);
        let mut p = started(&e, &source);
        let before = p.last.as_ref().unwrap().document.clone();
        let alive = p.owner_probe().unwrap();
        p.dispatch(Action::default()).unwrap();
        let error = ready(&mut p).unwrap_err();
        assert!(error.contains(why), "{why}: {error}");
        assert_eq!(p.state, State::Faulted);
        assert_eq!(p.tick(), 0);
        assert_eq!(p.last.as_ref().unwrap().document, before);
        assert!(!alive());
        p.stop();
    }
}
#[test]
fn initializing_cancel_type_budget_numeric_and_128_nodes() {
    let empty = Editor::new(Document::empty("empty-play".into())).unwrap();
    let mut empty_play = Play::new(&empty, [None, None], SOURCE).unwrap();
    ready(&mut empty_play).unwrap();
    empty_play.dispatch(Action::default()).unwrap();
    ready(&mut empty_play).unwrap();
    assert_eq!(empty_play.tick(), 1);
    assert!(empty_play.last.as_ref().unwrap().objects.is_empty());
    let mut e = fixture();
    for id in 200..325 {
        e.document.objects.push(Object::new(id, Kind::Quad));
    }
    let mut p = started(&e, SOURCE);
    assert_eq!(p.last.as_ref().unwrap().nodes.len(), 128);
    p.dispatch(Action::default()).unwrap();
    ready(&mut p).unwrap();
    assert_eq!(p.tick(), 1);
    println!(
        "128-node real Auto init_us={} tick_us={} compile_us={}",
        p.reports[0].elapsed_us, p.reports[1].elapsed_us, p.compile_us
    );
    let mut canceled = Play::new(&e, [Some(1), Some(2)], SOURCE).unwrap();
    let alive = canceled.owner_probe().unwrap();
    canceled.stop();
    assert!(!alive());
    for source in [
        SOURCE.replace(
            "fn tick(next int, dt float)",
            "fn tick(next float, dt float)",
        ),
        SOURCE.replace(
            "let space=input_space",
            "while true {}\n    let space=input_space",
        ),
    ] {
        let mut p = started(&e, &source);
        p.dispatch(Action::default()).unwrap();
        assert!(ready(&mut p).is_err());
        assert_eq!(p.state, State::Faulted);
    }
    let mut e = fixture();
    e.document.objects[0].translation[0] = 99999.0;
    e.document.groups[0].translation[0] = 0.0;
    let mut p = started(&e, SOURCE);
    for _ in 0..2 {
        p.dispatch(Action {
            space: 2,
            x: 1,
            ..Default::default()
        })
        .unwrap();
        if ready(&mut p).is_err() {
            break;
        }
    }
    assert_eq!(p.state, State::Faulted);
    assert_eq!(p.tick(), 1);
}

#[test]
fn display_batches_dispatch_same_600_complete_ticks_without_coalescing() {
    let e = fixture();
    let mut reference = vec![];
    for (run, gap) in [Duration::from_nanos(16_666_667), Duration::from_millis(33)]
        .into_iter()
        .enumerate()
    {
        let mut p = started(&e, SOURCE);
        p.input.focus(2);
        p.input.key(Key::Right, true, false);
        let start = Instant::now();
        p.clock(start);
        let mut rounds = 0;
        while p.tick() < 600 {
            rounds += 1;
            assert!(rounds < 1000);
            p.clock(start + gap * rounds);
            assert_eq!(p.state, State::Running);
            for _ in 0..4 {
                if p.tick() == 600 || !p.dispatch_credit().unwrap() {
                    break;
                }
                ready(&mut p).unwrap();
                let tick = p.tick() as usize;
                let bytes = p.last.as_ref().unwrap().document.encode().unwrap();
                if run == 0 {
                    reference.push(bytes);
                } else {
                    assert_eq!(bytes, reference[tick - 1]);
                }
            }
        }
        assert_eq!(p.reports.len(), 601);
        for (i, r) in p.reports.iter().enumerate() {
            assert_eq!(r.tick, i as i32);
        }
    }
}
