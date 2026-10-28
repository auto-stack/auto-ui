use autographics_auto_driver::squash::{source, SquashScript};
fn replay(input: &str) {
    let doc: serde_json::Value = serde_json::from_str(include_str!(
        "../../autographics-prototype/testdata/squash/golden_squash_trace.json"
    ))
    .unwrap();
    let mut s = SquashScript::new(input).unwrap();
    s.settle().unwrap();
    s.outputs.clear();
    let mut events = vec![];
    for (t, a) in doc["actions"].as_array().unwrap().iter().enumerate() {
        let action = [
            a[0].as_i64().unwrap() as i32,
            a[1].as_i64().unwrap() as i32,
            a[2].as_i64().unwrap() as i32,
        ];
        s.tick(s.id(), t as u64, action).unwrap();
        s.settle().unwrap();
        let p = s.outputs.pop_front().unwrap();
        assert!(s.outputs.is_empty());
        assert_eq!(
            p.score,
            doc["ticks_trace"][t]["score"].as_i64().unwrap() as i32,
            "score {t}"
        );
        assert_eq!(
            p.dead,
            doc["ticks_trace"][t]["alive"].as_i64().unwrap() == 0
        );
        for o in &p.objects {
            assert!(o.matrix().is_finite());
        }
        events.extend(p.events.into_iter().map(|e| (t, e)));
    }
    assert_eq!(events.len(), 91);
    for ((t, e), want) in events.iter().zip(doc["events"].as_array().unwrap()) {
        assert_eq!(*t, want["t"].as_u64().unwrap() as usize);
        assert_eq!(
            ["spawn", "squash", "despawn", "death"][e[0] as usize],
            want["kind"].as_str().unwrap()
        );
        if !want["index"].is_null() {
            assert_eq!(e[1], want["index"].as_i64().unwrap() as i32);
        }
    }
    println!(
        "staged profile2: ticks={} events=91 pumps={} max_pump_us={}",
        s.completed_ticks,
        s.pump_us.len(),
        s.pump_us.iter().max().unwrap()
    );
}

#[test]
fn staged_profile_original_events_and_transforms() {
    replay(&source());
}
#[test]
fn auto_gravity_and_collision_mutations_fail_the_same_oracle() {
    for (from, to) in [
        ("vy = vy - 75.0 * dt", "vy = vy - 60.0 * dt"),
        (
            "if result == 1 { hit = slot; break }",
            "if result == 2 { hit = slot; break }",
        ),
    ] {
        let original = source();
        assert!(original.contains(from), "mutation not applied");
        let mutant = original.replace(from, to);
        assert_ne!(mutant, original);
        assert!(
            std::panic::catch_unwind(|| replay(&mutant)).is_err(),
            "mutation escaped actual producer/publication replay"
        );
    }
}
