use autographics_auto_driver::{ResultKind, Scalar::Int, Session};
fn replay(source: &str) -> Result<(), String> {
    let trace: serde_json::Value = serde_json::from_str(include_str!(
        "../../autographics-prototype/testdata/breakout/golden_trace.json"
    ))
    .unwrap();
    let mut s = Session::new(source)?;
    s.call("init", &[], ResultKind::Unit)?;
    let mut contacts = Vec::new();
    for (t, a) in trace["actions"].as_array().unwrap().iter().enumerate() {
        s.call(
            "update",
            &[Int(a.as_i64().unwrap() as i32)],
            ResultKind::Unit,
        )?;
        let g = &trace["ticks"][t];
        for (i, key) in [(0, "bx"), (1, "by"), (4, "px")] {
            let actual = s.float("value", &[Int(i)])?;
            let expected = g[key].as_f64().unwrap() as f32;
            if (actual - expected).abs() > 1e-3 {
                return Err(format!("tick {t} {key}: {actual} != {expected}"));
            }
        }
        for (i, key) in [(0, "score"), (1, "sounds"), (2, "contacts")] {
            if s.int("integer", &[Int(i)])? != g[key].as_i64().unwrap() as i32 {
                return Err(format!("tick {t} {key}"));
            }
        }
        let destroyed: Vec<_> = trace["contacts_detail"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|g| g["t"].as_u64() == Some(t as u64) && g["kind"] == "brick")
            .map(|g| g["brick"].as_i64().unwrap() as i32)
            .collect();
        if s.int("integer", &[Int(3)])? != destroyed.len() as i32 {
            return Err(format!("tick {t} brick event count"));
        }
        for (i, expected) in destroyed.iter().enumerate() {
            if s.int("destroyed", &[Int(i as i32)])? != *expected {
                return Err(format!("tick {t} brick event identity"));
            }
        }
        let n = s.int("integer", &[Int(2)])?;
        if s.int("event_count", &[])? != n * 2 + destroyed.len() as i32 {
            return Err(format!("tick {t} output event count"));
        }
        let total = s.int("integer", &[Int(1)])?;
        for j in 0..n {
            let raw = s.int("contact", &[Int(j)])?;
            for (idx, k, a, b) in [
                (j * 2, 0, raw / 4, raw % 4),
                (j * 2 + 1, 1, total - n + j + 1, 0),
            ] {
                if s.int("event_kind", &[Int(idx)])? != k
                    || s.int("event_a", &[Int(idx)])? != a
                    || s.int("event_b", &[Int(idx)])? != b
                {
                    return Err(format!("tick {t} output event {idx}"));
                }
            }
        }
        for (j, brick) in destroyed.iter().enumerate() {
            let idx = n * 2 + j as i32;
            let score = s.int("integer", &[Int(0)])? - destroyed.len() as i32 + j as i32 + 1;
            if s.int("event_kind", &[Int(idx)])? != 2
                || s.int("event_a", &[Int(idx)])? != *brick
                || s.int("event_b", &[Int(idx)])? != score
            {
                return Err(format!("tick {t} destroyed output event"));
            }
        }

        for i in 0..n {
            let c = s.int("contact", &[Int(i)])?;
            let k = c / 4;
            let side = ["L", "R", "T", "B"][(c % 4) as usize];
            let kind = match k {
                0 => "wall_left",
                1 => "wall_right",
                2 => "wall_bottom",
                3 => "wall_top",
                4 => "paddle",
                _ => "brick",
            };
            contacts.push((t, kind, side, if k >= 5 { Some(k - 5) } else { None }));
        }
    }
    for (i, (t, kind, side, brick)) in contacts.iter().enumerate() {
        let g = &trace["contacts_detail"][i];
        if g["t"].as_u64() != Some(*t as u64)
            || g["kind"] != *kind
            || g["side"] != *side
            || g["brick"].as_i64() != brick.map(|v| v as i64)
        {
            return Err(format!("contact {i}: {t} {kind} {side} {brick:?} != {g}"));
        }
    }
    if contacts.len() != trace["contacts_detail"].as_array().unwrap().len() {
        return Err("contact count".into());
    }
    println!(
        "{} ticks, {} contacts match frozen golden",
        trace["actions"].as_array().unwrap().len(),
        contacts.len()
    );
    Ok(())
}
#[test]
fn frozen_auto_breakout() {
    replay(include_str!("../samples/breakout/main.at")).unwrap()
}
#[test]
fn speed_and_collision_controls() {
    let src = include_str!("../samples/breakout/main.at");
    assert!(replay(&src.replace("400.0", "401.0"))
        .unwrap_err()
        .starts_with("tick "));
    assert!(replay(&src.replace("if side >= 0", "if side > 99"))
        .unwrap_err()
        .starts_with("tick "));
}
