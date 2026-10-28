use autographics_auto_driver::{ResultKind, Scalar::*, Session};
fn source() -> String {
    format!(
        "{}\n{}",
        include_str!("../samples/squash/math.at"),
        include_str!("../samples/squash/main.at")
    )
}
#[test]
fn original_numeric_trace() {
    let doc: serde_json::Value = serde_json::from_str(include_str!(
        "../../autographics-prototype/testdata/squash/golden_squash_trace.json"
    ))
    .unwrap();
    let mut auto = Session::new(&source()).unwrap();
    auto.call("init", &[], ResultKind::Unit).unwrap();
    for (i, v) in agp::render3d::Camera3d::sample_camera(1.5)
        .view_proj()
        .to_cols_array()
        .iter()
        .enumerate()
    {
        auto.call(
            "setup_camera",
            &[Int(i as i32), Float(*v)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    let mut rng = agp::core::DeterministicRng::new(27);
    let mut sequence = 0;
    let mut events = vec![];
    let mut spawns = vec![];
    let mut maxpos = 0.0f32;
    for (t, action) in doc["actions"].as_array().unwrap().iter().enumerate() {
        auto.call(
            "begin_tick",
            &[
                Int(action[0].as_i64().unwrap() as i32),
                Int(action[1].as_i64().unwrap() as i32),
                Int(action[2].as_i64().unwrap() as i32),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        while auto.int("random_request", &[]).unwrap() == 1 {
            sequence += 1;
            assert_eq!(
                auto.int("random_value", &[Int(sequence), Float(rng.next_f32())])
                    .unwrap(),
                1
            );
        }
        for _ in 0..8 {
            auto.call("move_chunk", &[], ResultKind::Unit).unwrap();
        }
        auto.call("finish_tick", &[], ResultKind::Unit).unwrap();
        let want = &doc["ticks_trace"][t];
        for (f, name) in ["px", "py", "pz", "vy"].iter().enumerate() {
            let got = auto.float("state", &[Int(f as i32)]).unwrap();
            let error = (got - want[*name].as_f64().unwrap() as f32).abs();
            maxpos = maxpos.max(error);
            assert!(error <= 0.01, "tick {t} {name}: {got} vs {}", want[*name]);
        }
        assert_eq!(
            auto.int("status", &[Int(0)]).unwrap(),
            want["score"].as_i64().unwrap() as i32,
            "score tick {t}"
        );
        assert_eq!(
            auto.int("status", &[Int(1)]).unwrap(),
            1 - want["alive"].as_i64().unwrap() as i32,
            "alive tick {t}"
        );
        let count = (1..=64)
            .filter(|i| auto.int("object_alive", &[Int(*i)]).unwrap() == 1)
            .count();
        assert_eq!(
            count,
            want["mobs_alive"].as_u64().unwrap() as usize,
            "mob count tick {t}"
        );
        for i in 0..auto.int("event_count", &[]).unwrap() {
            let e: Vec<_> = (0..4)
                .map(|f| auto.int("event_field", &[Int(i), Int(f)]).unwrap())
                .collect();
            if e[0] == 0 {
                let slot = e[2] - 1;
                let data: Vec<_> = (0..6)
                    .map(|f| auto.float("mob_value", &[Int(slot), Int(f)]).unwrap())
                    .collect();
                spawns.push((t, e[1], data));
            }
            events.push((t, e));
        }
    }
    assert_eq!(events.len(), doc["events"].as_array().unwrap().len());
    for ((t, e), w) in events.iter().zip(doc["events"].as_array().unwrap()) {
        assert_eq!(*t, w["t"].as_u64().unwrap() as usize);
        assert_eq!(
            ["spawn", "squash", "despawn", "death"][e[0] as usize],
            w["kind"].as_str().unwrap()
        );
        if !w["index"].is_null() {
            assert_eq!(e[1], w["index"].as_i64().unwrap() as i32);
        }
        if e[0] == 1 {
            assert_eq!(e[3], w["score"].as_i64().unwrap() as i32);
        }
    }
    assert_eq!(spawns.len(), 45);
    for ((tick, index, data), want) in spawns.iter().zip(doc["spawns"].as_array().unwrap()) {
        assert_eq!(*tick, want["tick"].as_u64().unwrap() as usize);
        assert_eq!(*index, want["index"].as_i64().unwrap() as i32);
        for (got, key, f, tol) in [
            (data[0] - data[2] * (1.0 / 120.0), "pos", 0, 0.0001),
            (0.0, "pos", 1, 0.0001),
            (data[1] - data[3] * (1.0 / 120.0), "pos", 2, 0.0001),
            (data[2], "vel", 0, 0.001),
            (data[3], "vel", 1, 0.001),
        ] {
            assert!(
                (got - want[key][f].as_f64().unwrap() as f32).abs() <= tol,
                "spawn {index} {key}[{f}]"
            );
        }
        assert!((data[4] - want["speed"].as_f64().unwrap() as f32).abs() < 0.0001);
        assert!((data[5] - want["yaw"].as_f64().unwrap() as f32).abs() < 0.0001);
    }
    println!("original3600tick/91event/45spawn maxpos={maxpos} random_draws={sequence}");
}
