use autographics_auto_driver::squash::{source, Publication3d, SquashScript};
#[test]
fn bad_packets_are_transactional_and_profile_is_explicit() {
    let mut s = SquashScript::new(&source()).unwrap();
    s.settle().unwrap();
    let p = s.publication.last().unwrap().clone();
    let mut publication = Publication3d::new(p.session);
    publication.publish(p.clone()).unwrap();
    for mode in 0..8 {
        let mut bad = p.clone();
        bad.sequence += 1;
        match mode {
            0 => bad.profile = 1,
            1 => bad.session += 1,
            2 => bad.objects[0].resource = 1,
            3 => bad.objects[0].fields[3] = f32::NAN,
            4 => bad.objects[0].fields[6] = 0.0,
            5 => bad.events.push([0, 7, 1, 0]),
            6 => bad.events.push([1, 0, 1, 1]),
            _ => bad.sequence = 0,
        }
        assert!(publication.publish(bad).is_err(), "mode {mode}");
        assert_eq!(publication.last(), Some(&p));
    }
    let mut good = p.clone();
    good.sequence += 1;
    publication.publish(good).unwrap();
    assert!(SquashScript::new(&source().replace(
        "fn profile() int { return 2 }",
        "fn profile() int { return 1 }"
    ))
    .unwrap()
    .settle()
    .is_err());
}
#[test]
fn cancellation_budget_and_twenty_session_retries() {
    let mut s = SquashScript::new(&source()).unwrap();
    s.settle().unwrap();
    for _ in 0..20 {
        let id = s.id();
        let owner = s.session.owner_probe();
        s.close();
        assert!(!owner());
        s = SquashScript::new(&source()).unwrap();
        s.settle().unwrap();
        assert!(s.tick(id, 0, [0, 0, 0]).is_err());
    }
    let mut bad = SquashScript::new(&source().replace(
        "fn begin_tick(x int, z int, jump int) {",
        "fn begin_tick(x int, z int, jump int) { while true {} ",
    ))
    .unwrap();
    bad.settle().unwrap();
    let old = bad.publication.last().cloned();
    bad.outputs.clear();
    bad.tick(bad.id(), 0, [0, 0, 0]).unwrap();
    assert!(bad.settle().is_err());
    assert!(bad.session.closed());
    assert_eq!(bad.publication.last(), old.as_ref());
}

#[test]
fn pathological_spawn_load_and_queue_are_bounded() {
    let input = source().replace("step % 60 == 0", "step % 1 == 0");
    assert_ne!(input, source());
    let mut script = SquashScript::new(&input).unwrap();
    script.settle().unwrap();
    script.outputs.clear();
    let mut refused = None;
    for tick in 0..200 {
        let last = script.publication.last().cloned();
        script.tick(script.id(), tick, [0, 0, 0]).unwrap();
        match script.settle() {
            Ok(()) => script.outputs.clear(),
            Err(error) => {
                assert_eq!(script.publication.last(), last.as_ref());
                refused = Some(error);
                break;
            }
        }
    }
    let error = refused.expect("pathological spawn stream must meet the declared CPU/object bound");
    assert!(
        error.contains("capacity") || error.contains("budget"),
        "wrong refusal {error}"
    );
    println!("pathological load refused: {error}");
    let mut script = SquashScript::new(&source()).unwrap();
    script.settle().unwrap();
    script.outputs.clear();
    for tick in 0..64 {
        assert!(script.tick(script.id(), tick, [0, 0, 0]).unwrap());
    }
    assert!(script
        .tick(script.id(), 64, [0, 0, 0])
        .unwrap_err()
        .contains("backlog"));
    script.close();
}

#[test]
fn bad_spawn_packet_does_not_commit_random_stream() {
    let original = source();
    let old = "fn role(i int) int { if i == 0 { return 0 }; return 1 }";
    assert!(original.contains(old));
    let input=original.replace(old,"fn role(i int) int { if step > 60 && i > 0 { return 2 }; if i == 0 { return 0 }; return 1 }");
    let mut script = SquashScript::new(&input).unwrap();
    script.settle().unwrap();
    script.outputs.clear();
    for tick in 0..60 {
        script.tick(script.id(), tick, [0, 0, 0]).unwrap();
        script.settle().unwrap();
        script.outputs.clear();
    }
    let last = script.publication.last().cloned();
    assert_eq!(script.accepted_random_draws(), 0);
    script.tick(script.id(), 60, [0, 0, 0]).unwrap();
    assert!(script.settle().is_err());
    assert_eq!(script.publication.last(), last.as_ref());
    assert_eq!(script.accepted_random_draws(), 0);
}

#[test]
fn profile_cannot_change_after_the_initial_handshake() {
    let old = "fn profile() int { return 2 }";
    let original = source();
    assert!(original.contains(old));
    let input = original.replace(
        old,
        "fn profile() int { if step > 0 { return 3 }; return 2 }",
    );
    let mut script = SquashScript::new(&input).unwrap();
    script.settle().unwrap();
    let last = script.publication.last().cloned();
    script.outputs.clear();
    script.tick(script.id(), 0, [0, 0, 0]).unwrap();
    assert!(script.settle().unwrap_err().contains("profile"));
    assert_eq!(script.publication.last(), last.as_ref());
    assert!(script.session.closed());
}

#[test]
fn consumer_refuses_a_missing_completed_output() {
    let mut script = SquashScript::new(&source()).unwrap();
    script.settle().unwrap();
    let initial = script.publication.last().unwrap().clone();
    let mut consumer = Publication3d::new(initial.session);
    consumer.publish(initial.clone()).unwrap();
    let mut skipped = initial.clone();
    skipped.sequence += 2;
    assert!(consumer.publish(skipped).unwrap_err().contains("sequence"));
    assert_eq!(consumer.last(), Some(&initial));
}
