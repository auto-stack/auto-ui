use autographics_auto_driver::{
    script::{Action, Script},
    ResultKind,
};
use std::time::{Duration, Instant};
fn settle(s: &mut Script) {
    let n = Instant::now();
    while !s.idle() {
        s.pump().unwrap();
        assert!(n.elapsed() < Duration::from_secs(5));
    }
    s.outputs.clear();
}
#[test]
fn runnable_timeout_preserves_last_good_close_reopen_and_bounded_queues() {
    let original = include_str!("../samples/scene-control/main.at");
    let source = original.replace(
        "x = x + direction * 2.0; ticks = ticks + 1",
        "while true { ticks = ticks + 1 }",
    );
    let mut s = Script::new(&source).unwrap();
    settle(&mut s);
    let prior = s.publication.last().unwrap().clone();
    let alive = s.session.owner_probe();
    s.tick(1, 1).unwrap();
    let n = Instant::now();
    loop {
        match s.pump() {
            Ok(_) => {}
            Err(e) => {
                assert!(e.contains("budget"));
                break;
            }
        }
        assert!(n.elapsed() < Duration::from_secs(2));
    }
    assert!(s.session.yielded > 0);
    assert!(s.session.closed());
    assert!(!alive());
    assert_eq!(s.publication.last(), Some(&prior));
    assert!(s.outputs.is_empty());
    assert!(s.enqueue(s.id(), Action::Destroy).is_err());
    let old = s.id();
    let mut fresh = Script::new(original).unwrap();
    settle(&mut fresh);
    assert!(fresh.enqueue(old, Action::Destroy).is_err());
    let mut late = prior;
    late.sequence += 100;
    assert!(fresh.publication.publish(late).is_err());
    for _ in 0..64 {
        fresh.enqueue(fresh.id(), Action::Key(1)).unwrap()
    }
    assert!(fresh
        .enqueue(fresh.id(), Action::Key(1))
        .unwrap_err()
        .contains("queue full"));
    fresh.close();
    assert!(fresh.session.closed());
    let mut session = autographics_auto_driver::Session::new("fn main() {}").unwrap();
    session.call("main", &[], ResultKind::Unit).unwrap();
}
