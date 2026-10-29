use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    host::Host,
    play::{Action, State},
    storage::Storage,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn out() -> PathBuf {
    let p = PathBuf::from(std::env::var("G2_ARTIFACTS").expect("G2_ARTIFACTS"));
    fs::create_dir_all(&p).unwrap();
    p
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn load(scene: &Path) -> Host {
    let (doc, assets, code, _) = Storage::load_project(scene, &Default::default()).unwrap();
    Host::with_project(doc, assets, code).unwrap()
}
fn pump(h: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < deadline, "pump timeout");
        h.frame(None).unwrap();
    }
}
fn start(h: &mut Host, s: &str) {
    let sources = out().join("sources");
    fs::create_dir_all(&sources).unwrap();
    fs::write(sources.join(format!("{}.at", hash(s.as_bytes()))), s).unwrap();
    let path = root().join("fixtures/candidate-a.at");
    let snap =
        SourceSnapshot::from_bytes_for(&path, s.as_bytes().to_vec(), Api::Sprites2V1).unwrap();
    h.start_user_snapshot(snap).unwrap();
    h.pause_play("");
    pump(h);
    assert_eq!(
        h.play.as_ref().unwrap().state,
        State::Paused,
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}
fn step(h: &mut Host, held: &[usize]) -> u128 {
    let t = Instant::now();
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    for a in 0..4 {
        p.char_key(a, held.contains(&a), false)
    }
    p.dispatch(Action::default()).unwrap();
    h.pause_play("");
    pump(h);
    t.elapsed().as_micros()
}
fn fact(h: &Host) -> Value {
    let p = h.play.as_ref().unwrap();
    let f = p.last.as_ref().unwrap();
    json!({"run":f.run_id,"tick":f.tick,"state":format!("{:?}",p.state),"error":p.error,"actor":f.nodes[&1].translation,"diagnostic":f.nodes.get(&12).map(|n|n.translation),"source_sizes":p.user_source.as_ref().map(|s|json!({"user_bytes":s.bytes.len(),"combined_bytes":s.source.len(),"managed_plus_separators_bytes":s.source.len()-s.bytes.len(),"combined_sha256":s.combined_hash})),"enemy_x":f.instances.as_ref().and_then(|i|i.get(3)).map(|i|i.translation[0]),"live_count":f.instances.as_ref().map(|i|i.live_count()),"slots":format!("{:?}",f.instances),"sprite":format!("{:?}",f.sprites),"camera":format!("{:?}",f.camera)})
}
fn capture(h: &mut Host, name: &str) -> String {
    let (_, b, _, _) = h.rt.capture(h.two, name).expect("GPU capture");
    fs::write(out().join(name), &b).unwrap();
    hash(&b)
}
#[test]
fn full_loop_research() {
    let mut rows = Vec::new();
    for candidate in ["a", "b"] {
        let s =
            fs::read_to_string(root().join(format!("fixtures/candidate-{candidate}.at"))).unwrap();
        let mut h = load(&root().join("fixtures/scene.agscene"));
        let t = Instant::now();
        start(&mut h, &s);
        let init_us = t.elapsed().as_micros();
        let before = format!("{:?}", h.play.as_ref().unwrap().last);
        let image_before = capture(&mut h, &format!("{candidate}-initial.png"));
        let mut trace = Vec::new();
        for tick in 1..=2400 {
            let held: Vec<usize> = if (21..=400).contains(&tick) {
                if [21, 100, 140, 200, 260, 320, 380].contains(&tick) {
                    vec![1, 2]
                } else {
                    vec![1]
                }
            } else if (421..=460).contains(&tick) {
                if [430, 440].contains(&tick) {
                    vec![0, 2]
                } else {
                    vec![0]
                }
            } else if tick == 481 {
                vec![3]
            } else {
                vec![]
            };
            let us = step(&mut h, &held);
            trace.push(json!({"logical_tick":tick,"held":held,"host_pump_us":us,"frame":fact(&h)}));
            if h.play.as_ref().unwrap().error.is_some() {
                break;
            }
        }
        let error = h.play.as_ref().unwrap().error.clone();
        assert!(
            error
                .as_ref()
                .is_some_and(|e| e.contains("10") || e.contains("QueryBudget")),
            "{candidate}: {error:?}"
        );
        assert_eq!(
            before,
            format!("{:?}", h.play.as_ref().unwrap().last),
            "entire last-good frame"
        );
        let image_after = capture(&mut h, &format!("{candidate}-fault.png"));
        assert_eq!(
            image_before, image_after,
            "fault GPU must retain full last-good"
        );
        rows.push(json!({"candidate":candidate,"source_sha256":hash(s.as_bytes()),"init_us":init_us,"trace":trace,"initial_capture_sha256":image_before,"fault_capture_sha256":image_after,"outcome":"no_go_current_profile"}));
        h.stop_play();
    }
    fs::write(out().join("core.json"),serde_json::to_vec_pretty(&json!({"schema":"ui011-core/1","profile":if cfg!(debug_assertions){"debug"}else{"release"},"rows":rows})).unwrap()).unwrap();
}

fn source_a() -> String {
    fs::read_to_string(root().join("fixtures/candidate-a.at"))
        .unwrap()
        .replace("\r\n", "\n")
}
fn character_only(s: &str) -> String {
    let from = s.find(" ex=ex+direction*dt").unwrap();
    let to = s[from..].find(" if ag_world").unwrap() + from;
    format!("{}{}", &s[..from], &s[to..])
}
fn event_only(s: &str) -> String {
    assert!(s.contains("let q=ag_char_move(dx,dy,0.002)\n grounded=ag_char_grounded(q)"));
    s.replace("let q=ag_char_move(dx,dy,0.002)\n grounded=ag_char_grounded(q)","grounded=ag_char_grounded(0)").replace(" if ag_world(0,1)<0.0 - 3.0 {state=2;hud_intent=hud_intent+1}"," if ag_world(0,1)<0.0 - 3.0 {state=2;hud_intent=hud_intent+1}\n ag_set_local(11,0,bag*1.0)\n ag_set_local(11,1,state*1.0)")
}
fn variant(name: &str, xy: [f32; 2]) -> PathBuf {
    let mut d: Value =
        serde_json::from_slice(&fs::read(root().join("fixtures/scene.agscene")).unwrap()).unwrap();
    d["objects"][0]["translation"] = json!([xy[0], xy[1], 0]);
    let mut marker = d["objects"][10].clone();
    marker["id"] = json!(12);
    marker["name"] = json!("isolated-diagnostic-only");
    marker["visible"] = json!(false);
    marker["translation"] = json!([-44., -40., 0.]);
    d["objects"].as_array_mut().unwrap().push(marker);
    let dir = out().join("scenes");
    fs::create_dir_all(dir.join("assets")).unwrap();
    for e in fs::read_dir(root().join("fixtures/assets")).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join("assets").join(e.file_name())).unwrap();
    }
    let p = dir.join(format!("{name}.agscene"));
    fs::write(&p, serde_json::to_vec(&d).unwrap()).unwrap();
    p
}
#[track_caller]
fn healthy(h: &Host) {
    assert!(
        h.play.as_ref().unwrap().error.is_none(),
        "{:?}",
        h.play.as_ref().unwrap().error
    )
}
fn pos(h: &Host) -> [f32; 3] {
    h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation
}
fn live(h: &Host) -> usize {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .unwrap()
        .instances
        .as_ref()
        .unwrap()
        .live_count()
}
fn visual(h: &Host) -> ([u32; 4], bool) {
    let f = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .unwrap()
        .sprites
        .as_ref()
        .unwrap()
        .visual_for(1)
        .unwrap();
    (f.frame, f.flip)
}
fn record(rows: &mut Vec<Value>, name: &str, h: &Host, us: u128, source: &str) {
    rows.push(json!({"case":name,"source_sha256":hash(source.as_bytes()),"host_pump_us":us,"frame":fact(h),"resources":h.resource_snapshot()}))
}
fn pixel_oracle(h: &mut Host, frame_x: u32, flip: bool) -> Value {
    use glam::Vec2;
    let bytes = fs::read(root().join(
        "fixtures/assets/8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df.png",
    ))
    .unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut rgba = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut rgba).unwrap();
    assert_eq!(info.width, 160);
    let f = h.play.as_ref().unwrap().last.as_ref().unwrap();
    let camera = f.camera.as_ref().unwrap();
    let point = Vec2::new(pos(h)[0], pos(h)[1]);
    let follow = camera.follow_center_2d(point).unwrap();
    let center = camera.screen_2d(point, follow).unwrap();
    let scale = camera.letterbox_2d().unwrap().0;
    let half = Vec2::new(0.4 * scale.x, 0.5 * scale.y);
    let points: &[(u32, u32)] = match frame_x {
        32 => &[(16, 9), (19, 9), (16, 18), (25, 18), (8, 20)],
        64 => &[(16, 9), (19, 9), (16, 18), (4, 19), (23, 20)],
        96 => &[(16, 9), (19, 9), (16, 18), (3, 8), (28, 8)],
        128 => &[(16, 9), (19, 9), (16, 18), (4, 23), (27, 23)],
        _ => &[(16, 9), (19, 9), (16, 18), (8, 20), (23, 20)],
    };
    let background = h.rt.probe_pixel(h.two, [10., 10.]).unwrap();
    let mut samples = Vec::new();
    let mut wrong_diff = false;
    for &(x, y) in points {
        let shown = if flip { 31 - x } else { x };
        let xy = [
            center.x + ((shown as f32 + 0.5) / 32. * 2. - 1.) * half.x,
            center.y + ((y as f32 + 0.5) / 32. * 2. - 1.) * half.y,
        ];
        let actual = h.rt.probe_pixel(h.two, xy).unwrap();
        let at = ((y * 160 + frame_x + x) * 4) as usize;
        let expected = composite(rgba[at..at + 4].try_into().unwrap(), background);
        assert!(
            actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 8),
            "frame {frame_x} actual{actual:?} expected{expected:?}"
        );
        let wrong = ((y * 160 + ((frame_x + 32) % 160) + x) * 4) as usize;
        if actual
            .iter()
            .zip(composite(
                rgba[wrong..wrong + 4].try_into().unwrap(),
                background,
            ))
            .any(|(a, b)| a.abs_diff(b) > 8)
        {
            wrong_diff = true
        }
        samples.push(json!({"xy":xy,"actual":actual,"expected":expected}));
    }
    assert!(wrong_diff, "wrong-frame control must be distinguished");
    json!({"frame_x":frame_x,"flip":flip,"samples":samples,"wrong_frame_rejected":wrong_diff})
}
#[test]
fn scoped_branches_and_controls() {
    let a = source_a();
    let character = character_only(&a);
    let events = event_only(&a);
    let ground = variant("ground", [2., 0.502]);
    let mut h = load(&ground);
    let mut rows = Vec::new();
    // Fresh 32/33 threshold, same normal movement and grounded source.
    for extra in [1, 2] {
        let s = character.replace(
            " // SPRITE_WRITE",
            &format!(" for b in 0..{extra} {{ok=ok+ag_action_held(0)}}\n // SPRITE_WRITE"),
        );
        start(&mut h, &s);
        let old = format!("{:?}", h.play.as_ref().unwrap().last);
        let us = step(&mut h, &[]);
        if extra == 1 {
            healthy(&h)
        } else {
            assert!(h
                .play
                .as_ref()
                .unwrap()
                .error
                .as_ref()
                .is_some_and(|e| e.contains("QueryBudget")));
            assert_eq!(old, format!("{:?}", h.play.as_ref().unwrap().last));
        }
        record(&mut rows, &format!("boundary-{}", 31 + extra), &h, us, &s);
        h.stop_play();
    }
    // Grounded B's unchanged sprite is exactly32; first changed run sprite faults.
    let b = fs::read_to_string(root().join("fixtures/candidate-b.at")).unwrap();
    start(&mut h, &b);
    let us = step(&mut h, &[]);
    healthy(&h);
    record(&mut rows, "b-cached-idle32", &h, us, &b);
    let us = step(&mut h, &[1]);
    assert!(h
        .play
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .is_some_and(|e| e.contains("QueryBudget")));
    record(&mut rows, "b-first-run-frame33", &h, us, &b);
    h.stop_play();
    // Isolated character path: true input, release, facing, accepted ROI and air phases.
    start(&mut h, &character);
    let x = pos(&h)[0];
    let us = step(&mut h, &[1]);
    healthy(&h);
    assert!(pos(&h)[0] > x);
    assert_eq!(visual(&h).0[0], 64);
    let pixel = pixel_oracle(&mut h, 64, false);
    capture(&mut h, "isolated-run.png");
    record(&mut rows, "move-right-run", &h, us, &character);
    rows.last_mut().unwrap()["pixels"] = pixel;
    step(&mut h, &[]);
    let x = pos(&h)[0];
    step(&mut h, &[]);
    assert!((pos(&h)[0] - x).abs() < 0.001);
    let us = step(&mut h, &[0]);
    assert!(pos(&h)[0] < x);
    let frame_x = visual_frame(&h);
    let pixel = pixel_oracle(&mut h, frame_x, true);
    record(&mut rows, "move-left-flip", &h, us, &character);
    rows.last_mut().unwrap()["pixels"] = pixel;
    let y = pos(&h)[1];
    let us = step(&mut h, &[2]);
    healthy(&h);
    assert!(pos(&h)[1] > y);
    assert_eq!(visual(&h).0[0], 96);
    let pixel = pixel_oracle(&mut h, 96, true);
    capture(&mut h, "isolated-rise.png");
    record(&mut rows, "jump-rise", &h, us, &character);
    rows.last_mut().unwrap()["pixels"] = pixel;
    let mut saw_fall = false;
    let mut landed = false;
    let mut trajectory = Vec::new();
    for t in 0..100 {
        let us = step(&mut h, if t == 2 { &[2] } else { &[] });
        healthy(&h);
        trajectory.push(json!({"t":t,"y":pos(&h)[1],"frame":visual(&h).0[0],"us":us}));
        if visual(&h).0[0] == 128 && !saw_fall {
            let pixel = pixel_oracle(&mut h, 128, true);
            capture(&mut h, "isolated-fall.png");
            record(&mut rows, "fall", &h, us, &character);
            rows.last_mut().unwrap()["pixels"] = pixel;
            saw_fall = true;
        }
        if saw_fall && visual(&h).0[0] == 0 {
            landed = true;
            break;
        }
    }
    for pair in trajectory.windows(3) {
        let y0 = pair[0]["y"].as_f64().unwrap();
        let y1 = pair[1]["y"].as_f64().unwrap();
        let y2 = pair[2]["y"].as_f64().unwrap();
        if pair[2]["frame"] != 0 {
            assert!(
                y2 - y1 <= y1 - y0 + 0.001,
                "air jump must not reset vertical velocity"
            );
        }
    }
    assert!(saw_fall && landed);
    assert!((pos(&h)[1] - 0.502).abs() < 0.015);
    rows.push(json!({"case":"air-rejump-and-land","trajectory":trajectory}));
    let paused = fact(&h);
    for _ in 0..3 {
        h.frame(None).unwrap()
    }
    assert_eq!(paused, fact(&h));
    let before = pos(&h);
    h.play.as_mut().unwrap().single_step().unwrap();
    pump(&mut h);
    healthy(&h);
    assert!((pos(&h)[0] - before[0]).abs() < 0.001);
    record(&mut rows, "pause-neutral-step", &h, 0, &character);
    h.stop_play();
    // Controlled geometry and peaks: full candidates attempt each, never count isolated as core.
    for (name, xy, held) in [
        ("wall", [0.402, 0.502], vec![0]),
        ("ceiling", [13., 9.498], vec![2]),
        ("edge", [8.39, 0.502], vec![1]),
        ("pit", [9., -3.01], vec![]),
        ("single", [4., 1.], vec![]),
        ("upper-coin", [13., 2.25], vec![]),
        ("goal-without-coins", [28., 1.], vec![]),
        ("enemy-contact", [11., 0.5], vec![]),
    ] {
        println!("branch {name}");
        let scene = variant(name, xy);
        let mut ph = load(&scene);
        for (c, s) in [("a", a.as_str()), ("b", b.as_str())] {
            start(&mut ph, s);
            let mut times = Vec::new();
            for _ in 0..3 {
                let us = step(&mut ph, &held);
                times.push(us);
                if ph.play.as_ref().unwrap().error.is_some() {
                    break;
                }
            }
            record(
                &mut rows,
                &format!("full-{c}-{name}"),
                &ph,
                *times.last().unwrap(),
                s,
            );
            rows.last_mut().unwrap()["times"] = json!(times);
            ph.stop_play();
        }
        let ceiling_source = character.replace("var vy=0.0", "var vy=8.0");
        let s = if name == "ceiling" {
            ceiling_source.as_str()
        } else if name == "pit"
            || name == "single"
            || name == "upper-coin"
            || name == "goal-without-coins"
            || name == "enemy-contact"
        {
            events.as_str()
        } else {
            character.as_str()
        };
        start(&mut ph, s);
        let initial_run = ph.play.as_ref().unwrap().run_id;
        let mut states = Vec::new();
        for _ in 0..3 {
            let us = step(&mut ph, &held);
            healthy(&ph);
            states.push(json!({"us":us,"frame":fact(&ph)}));
        }
        if name == "single" || name == "upper-coin" {
            assert_eq!(live(&ph), 3);
            assert_eq!(diagnostic(&ph), [1., 0.]);
        }
        if name == "wall" {
            assert!(pos(&ph)[0] >= 0.401)
        }
        if name == "ceiling" {
            assert!(pos(&ph)[1] <= 9.5)
        }
        if name == "edge" {
            assert!(pos(&ph)[1] < 0.502)
        }
        if name == "goal-without-coins" {
            assert_eq!(diagnostic(&ph), [0., 0.]);
        }
        if name == "enemy-contact" || name == "pit" {
            assert_eq!(diagnostic(&ph)[1], 2.);
            enter_restart(&mut ph);
            healthy(&ph);
            assert_ne!(initial_run, ph.play.as_ref().unwrap().run_id);
            assert_eq!(live(&ph), 4)
        }
        if name == "goal-without-coins" {
            step(&mut ph, &[3]);
            healthy(&ph);
            assert_eq!(initial_run, ph.play.as_ref().unwrap().run_id)
        }
        record(&mut rows, &format!("isolated-{name}"), &ph, 0, s);
        rows.last_mut().unwrap()["states"] = json!(states);
        ph.stop_play();
    }
    // Real three-ID enter/despawn at N+1 and coin/goal adjacency, no Rust collection.
    let triple = events
        .replace("(0,9,4.0,1.0,0.0)", "(0,9,28.0,1.0,0.0)")
        .replace("(1,9,13.0,2.25,0.0)", "(1,9,28.0,1.0,0.0)")
        .replace("(2,9,24.0,1.0,0.0)", "(2,9,28.0,1.0,0.0)");
    let triple_scene = variant("triple", [28., 1.]);
    let mut th = load(&triple_scene);
    start(&mut th, &triple);
    let run = th.play.as_ref().unwrap().run_id;
    step(&mut th, &[]);
    healthy(&th);
    assert_eq!(live(&th), 4);
    let us = step(&mut th, &[]);
    healthy(&th);
    assert_eq!(live(&th), 1);
    assert_eq!(diagnostic(&th), [3., 1.]);
    record(&mut rows, "three-coins-goal-Nplus1", &th, us, &triple);
    step(&mut th, &[]);
    assert_eq!(live(&th), 1);
    enter_restart(&mut th);
    healthy(&th);
    assert_ne!(run, th.play.as_ref().unwrap().run_id);
    assert_eq!(live(&th), 4);
    record(&mut rows, "triple-win-enter-fresh-run", &th, 0, &triple);
    th.stop_play();
    // Deliberately drop third event: real live-slot observation catches the mutant.
    let dropped = triple.replace("for i in 0..6", "for i in 0..2");
    start(&mut th, &dropped);
    step(&mut th, &[]);
    step(&mut th, &[]);
    healthy(&th);
    assert_eq!(live(&th), 2);
    record(&mut rows, "dropped-third-event-control", &th, 0, &dropped);
    th.stop_play();
    start(&mut th, &triple);
    step(&mut th, &[]);
    step(&mut th, &[]);
    healthy(&th);
    assert_eq!(live(&th), 1);
    record(&mut rows, "restored-triple-control", &th, 0, &triple);
    th.stop_play();

    // Full triple peak attempts preserve original motion costs; isolated reports never establish core.
    for (c, base) in [("a", a.as_str()), ("b", b.as_str())] {
        let source = base
            .replace("(0,9,4.0,1.0,0.0)", "(0,9,28.0,1.0,0.0)")
            .replace("(1,9,13.0,2.25,0.0)", "(1,9,28.0,1.0,0.0)")
            .replace("(2,9,24.0,1.0,0.0)", "(2,9,28.0,1.0,0.0)");
        start(&mut th, &source);
        let mut peak_trace = Vec::new();
        let mut us = 0;
        for _ in 0..3 {
            us = step(&mut th, &[]);
            peak_trace.push(fact(&th));
            if th.play.as_ref().unwrap().error.is_some() {
                break;
            }
        }
        assert!(
            th.play
                .as_ref()
                .unwrap()
                .error
                .as_ref()
                .is_some_and(|e| e.contains("QueryBudget")),
            "full peak {c} {:?}",
            th.play.as_ref().unwrap().error
        );
        record(
            &mut rows,
            &format!("full-{c}-three-coins-peak-budget-blocked"),
            &th,
            us,
            &source,
        );
        rows.last_mut().unwrap()["trace"] = json!(peak_trace);
        th.stop_play();
    }
    let danger_scene = variant("danger", [11., 0.5]);
    let mut dd: Value = serde_json::from_slice(&fs::read(&danger_scene).unwrap()).unwrap();
    dd["objects"][10]["translation"] = json!([11., 0.5, 0.]);
    fs::write(&danger_scene, serde_json::to_vec(&dd).unwrap()).unwrap();
    let mut dh = load(&danger_scene);
    let danger = events
        .replace("(0,9,4.0,1.0,0.0)", "(0,9,11.0,0.5,0.0)")
        .replace("(1,9,13.0,2.25,0.0)", "(1,9,11.0,0.5,0.0)")
        .replace("(2,9,24.0,1.0,0.0)", "(2,9,11.0,0.5,0.0)");
    start(&mut dh, &danger);
    step(&mut dh, &[]);
    let us = step(&mut dh, &[]);
    healthy(&dh);
    assert_eq!(diagnostic(&dh), [3., 2.]);
    assert_eq!(live(&dh), 1);
    record(
        &mut rows,
        "isolated-three-coins-enemy-goal-priority",
        &dh,
        us,
        &danger,
    );
    dh.stop_play();
    // Bad user sprite output rejects atomically; exact source restored for legal control.
    let bad = character.replace(" // SPRITE_WRITE", " x=160\n // SPRITE_WRITE");
    start(&mut h, &bad);
    let before = format!("{:?}", h.play.as_ref().unwrap().last);
    let img = capture(&mut h, "bad-output-before.png");
    let us = step(&mut h, &[]);
    assert!(h.play.as_ref().unwrap().error.is_some());
    assert_eq!(before, format!("{:?}", h.play.as_ref().unwrap().last));
    assert_eq!(img, capture(&mut h, "bad-output-after.png"));
    record(&mut rows, "bad-sprite-last-good", &h, us, &bad);
    h.stop_play();
    start(&mut h, &character);
    let old_token = h.play.as_ref().unwrap().run_id * 524288 + 1;
    h.stop_play();
    let stale = format!(
        "{}\nfn app_update(dt float) {{ok=ag_object_despawn({old_token})}}",
        character.split("fn app_update").next().unwrap()
    );
    start(&mut h, &stale);
    let before = format!("{:?}", h.play.as_ref().unwrap().last);
    let us = step(&mut h, &[]);
    assert!(h.play.as_ref().unwrap().error.is_some());
    assert_eq!(before, format!("{:?}", h.play.as_ref().unwrap().last));
    record(&mut rows, "old-run-handle-rejected", &h, us, &stale);
    h.stop_play();
    start(&mut h, &character);
    step(&mut h, &[]);
    healthy(&h);
    record(&mut rows, "exact-source-restored", &h, 0, &character);
    h.stop_play();
    // Actual patrol mutation and endpoint controls, labelled isolated public Auto.
    let frozen = events.replace("ok=ok+ag_object_move(enemy,ex,0.5,0.0)", "ok=ok+0");
    assert_ne!(events, frozen);
    start(&mut h, &events);
    let initial = enemy_x(&h);
    step(&mut h, &[]);
    healthy(&h);
    assert!(enemy_x(&h) > initial);
    record(&mut rows, "isolated-patrol-moving-control", &h, 0, &events);
    h.stop_play();
    start(&mut h, &frozen);
    let initial = enemy_x(&h);
    step(&mut h, &[]);
    healthy(&h);
    assert_eq!(enemy_x(&h), initial);
    record(&mut rows, "frozen-enemy-mutant-detected", &h, 0, &frozen);
    h.stop_play();
    for (name, x, dir) in [("patrol-east", 16.99, 1.), ("patrol-west", 11.001, -1.)] {
        let source = events
            .replace("var ex=11.0", &format!("var ex={x}"))
            .replace("var direction=1.0", &format!("var direction={dir:.1}"));
        start(&mut h, &source);
        let mut positions = Vec::new();
        for _ in 0..4 {
            step(&mut h, &[]);
            healthy(&h);
            let x = enemy_x(&h);
            assert!((11.0..=17.0).contains(&x));
            positions.push(x);
        }
        if name == "patrol-east" {
            assert!(positions[1] < positions[0])
        } else {
            assert!(positions[1] > positions[0])
        }
        record(&mut rows, name, &h, 0, &source);
        rows.last_mut().unwrap()["positions"] = json!(positions);
        h.stop_play();
    }
    // Warm resources and final owner release over20 short Runs, valid cached editor image stays pinned.
    h.frame(None).unwrap();
    let editor_image = capture(&mut h, "editor-before-cycles.png");
    let before = h.resource_snapshot();
    let mut cycles = Vec::new();
    for n in 0..20 {
        start(&mut h, &character);
        let owner = h
            .play
            .as_ref()
            .unwrap()
            .owner_probe()
            .expect("VM owner probe");
        assert!(owner());
        let us = step(&mut h, &[]);
        healthy(&h);
        let first = fact(&h);
        assert_eq!(live(&h), 4);
        h.stop_play();
        h.frame(None).unwrap();
        assert!(!owner(), "last VM owner released");
        assert!(h.play.is_none());
        assert_eq!(
            editor_image,
            capture(&mut h, &format!("editor-cycle-{n}.png")),
            "Stop restores actual editor pixels"
        );
        cycles.push(json!({"n":n,"us":us,"first":first,"owner_released":true}));
    }
    let after = h.resource_snapshot();
    assert_eq!(
        before["asset_union"]["decode_attempts"],
        after["asset_union"]["decode_attempts"]
    );
    rows.push(json!({"case":"twenty-short-runs","cycles":cycles,"before":before,"after":after}));
    start(&mut h, &character);
    let pin = h
        .play
        .as_ref()
        .unwrap()
        .assets
        .values()
        .next()
        .unwrap()
        .clone();
    let weak = std::sync::Arc::downgrade(&pin);
    h.stop_play();
    assert!(
        weak.upgrade().is_some(),
        "editor/external pin legitimately retains image"
    );
    drop(h);
    assert!(weak.upgrade().is_some(), "external pin still owns image");
    drop(pin);
    assert!(weak.upgrade().is_none(), "last image owner released");
    rows.push(json!({"case":"image-owner-editor-external-final","editor_pin_retained":true,"external_pin_retained":true,"last_owner_released":true}));
    fs::write(
        out().join("branches.json"),
        serde_json::to_vec_pretty(&json!({"schema":"ui011-branches/1","rows":rows})).unwrap(),
    )
    .unwrap();
}
fn visual_frame(h: &Host) -> u32 {
    visual(h).0[0]
}

fn srgb_decode(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb_encode(v: f64) -> u8 {
    let c = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (c.mul_add(255.0, 0.5).floor() as i32).clamp(0, 255) as u8
}
fn composite(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let alpha = f64::from(src[3]) / 255.0;
    [
        srgb_encode(srgb_decode(src[0]) * alpha + srgb_decode(dst[0]) * (1.0 - alpha)),
        srgb_encode(srgb_decode(src[1]) * alpha + srgb_decode(dst[1]) * (1.0 - alpha)),
        srgb_encode(srgb_decode(src[2]) * alpha + srgb_decode(dst[2]) * (1.0 - alpha)),
        255,
    ]
}

fn enter_restart(h: &mut Host) {
    let old = h.play.as_ref().unwrap().run_id;
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    for a in 0..4 {
        p.char_key(a, a == 3, false)
    }
    p.dispatch(Action::default()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while h.play.as_ref().unwrap().run_id == old {
        assert!(Instant::now() < deadline, "Enter restart did not publish");
        healthy(h);
        h.frame(None).unwrap();
    }
    h.pause_play("");
    pump(h);
    healthy(h);
}

fn diagnostic(h: &Host) -> [f32; 2] {
    let p = h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&12].translation;
    [p[0], p[1]]
}

#[test]
fn performance_prefix() {
    let mut rows = Vec::new();
    for c in ["a", "b"] {
        let source = fs::read_to_string(root().join(format!("fixtures/candidate-{c}.at"))).unwrap();
        let mut h = load(&root().join("fixtures/scene.agscene"));
        h.prepare_measurement.enable(true);
        let t = Instant::now();
        start(&mut h, &source);
        let init_us = t.elapsed().as_micros();
        let us = step(&mut h, &[]);
        assert!(h
            .play
            .as_ref()
            .unwrap()
            .error
            .as_ref()
            .is_some_and(|e| e.contains("QueryBudget")));
        let p = h.play.as_ref().unwrap();
        rows.push(json!({"candidate":c,"case":"original-failing-prefix","source_sha256":hash(source.as_bytes()),"init_us":init_us,"host_whole_tick_us":us,"vm_output_ns":p.vm_output_measurement.samples_ns,"admission_ns":p.admission_measurement.samples_ns,"host_prepare_ns":h.prepare_measurement.samples_ns,"resources":h.resource_snapshot(),"published_tick":p.last.as_ref().unwrap().tick}));
        h.stop_play();
    }
    let scene = variant("perf-ground", [2., 0.502]);
    let s = character_only(&source_a());
    let mut h = load(&scene);
    h.prepare_measurement.enable(true);
    start(&mut h, &s);
    let mut times = Vec::new();
    let mut readback = Vec::new();
    for t in 0..60 {
        let us = step(
            &mut h,
            if t % 4 == 0 {
                &[1]
            } else if t % 4 == 2 {
                &[0]
            } else {
                &[]
            },
        );
        healthy(&h);
        times.push(us);
        if t % 20 == 0 {
            let begin = Instant::now();
            let fx = visual_frame_value(&h);
            let flip = visual_flip_value(&h);
            pixel_oracle(&mut h, fx, flip);
            readback.push(begin.elapsed().as_micros());
        }
    }
    let p = h.play.as_ref().unwrap();
    rows.push(json!({"case":"isolated-legal-character60","source_sha256":hash(s.as_bytes()),"host_whole_tick_us":times,"readback_assert_us":readback,"vm_output_ns":p.vm_output_measurement.samples_ns,"admission_ns":p.admission_measurement.samples_ns,"host_prepare_ns":h.prepare_measurement.samples_ns,"resources":h.resource_snapshot(),"note":"isolated character only; not a successful core route"}));
    h.stop_play();
    fs::write(out().join("performance.json"),serde_json::to_vec_pretty(&json!({"schema":"ui011-performance/1","profile":if cfg!(debug_assertions){"debug"}else{"release"},"rows":rows,"step_resume_meter":"not_exposed_by_public_Play; enforced frozen SDK bounds only","gpu_timestamps":"unavailable"})).unwrap()).unwrap();
}
fn visual_frame_value(h: &Host) -> u32 {
    visual(h).0[0]
}
fn visual_flip_value(h: &Host) -> bool {
    visual(h).1
}

fn enemy_x(h: &Host) -> f32 {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .unwrap()
        .instances
        .as_ref()
        .unwrap()
        .get(3)
        .unwrap()
        .translation[0]
}
