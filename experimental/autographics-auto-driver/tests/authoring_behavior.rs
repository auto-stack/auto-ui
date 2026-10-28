use autographics_auto_driver::authoring::{
    behavior::{SourceSnapshot, MAX_USER, VERSION},
    document::{Dimension, Document, Group, Kind, Object},
    play::{Action, Play, State},
    session::Editor,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn snapshot(s: &str) -> SourceSnapshot {
    SourceSnapshot::from_bytes(Path::new("D:/user/main.at"), s.as_bytes().to_vec()).unwrap()
}
fn fixture() -> Editor {
    let mut d = Document::empty("behavior".into());
    let mut g = Group::new(10, Dimension::Three);
    g.translation = [1.0, 0.0, 0.0];
    d.groups.push(g);
    let mut m = Object::new(11, Kind::Cylinder);
    m.parent = Some(10);
    m.translation = [-1.0, 0.0, 0.0];
    d.objects.push(m);
    let mut q = Object::new(12, Kind::Quad);
    q.translation = [-100.0, 0.0, 0.0];
    d.objects.push(q);
    Editor::new(d).unwrap()
}
fn ready(p: &mut Play) -> Result<(), String> {
    let end = Instant::now() + Duration::from_secs(31);
    while p.candidate().is_none() {
        if Instant::now() > end {
            return Err("test deadline".into());
        }
        p.pump_until(Instant::now() + Duration::from_millis(8))?;
    }
    p.accept()
}
#[test]
fn callbacks_globals_handle_zero_read_your_writes_and_new_run() {
    let e = fixture();
    let before = e.isolation_identity();
    let source="var initial=2.0\nvar starts=0\nfn app_start(){ starts=starts+1; if ag_valid(0)==1 {if ag_is_group(0)==1 {ag_set_local(ag_actor(2),0,initial)}}}\nfn app_update(dt float){let n=ag_actor(3); let parent=ag_parent(n); ag_move_local(parent,3.0*dt,0.0,0.0); ag_set_local(ag_actor(2),0,ag_world(n,0)+starts); ag_set_visible(parent,0); ag_set_visible(ag_actor(2),ag_visible(n));}\n";
    for _ in 0..2 {
        let mut p = Play::new_user(&e, [Some(12), Some(11)], snapshot(source)).unwrap();
        p.pause("starting cancellation");
        ready(&mut p).unwrap();
        assert_eq!(p.state, State::Paused);
        assert_eq!(
            p.last.as_ref().unwrap().document.objects[1].translation[0],
            2.0
        );
        let owner = p.owner_probe().unwrap();
        p.single_step().unwrap();
        ready(&mut p).unwrap();
        let f = p.last.as_ref().unwrap();
        assert!((f.document.objects[1].translation[0] - 1.05).abs() < 1e-5);
        assert!(!f.document.objects[1].visible);
        assert_eq!(p.tick(), 1);
        p.stop();
        assert!(!owner());
        assert_eq!(e.isolation_identity(), before);
    }
}
#[test]
fn first_api_error_sticks_and_rejects_whole_tick_closing_vm() {
    let e = fixture();
    let mut p=Play::new_user(&e,[Some(12),Some(11)],snapshot("fn app_start(){}\nfn app_update(dt float){ag_set_local(-1,0,5.0); ag_set_local(ag_actor(2),0,42.0)}\n")).unwrap();
    ready(&mut p).unwrap();
    let before = p.last.as_ref().unwrap().document.clone();
    let owner = p.owner_probe().unwrap();
    p.dispatch(Action::default()).unwrap();
    let err = ready(&mut p).unwrap_err();
    assert!(err.contains("code=1 node=-1"), "{err}");
    assert!(err.contains("app_update"));
    assert_eq!(p.state, State::Faulted);
    assert!(!owner());
    assert_eq!(p.last.as_ref().unwrap().document, before);
    assert_eq!(p.tick(), 0);
}
#[test]
fn invalid_axis_flag_dimension_and_missing_actor_controls() {
    let e = fixture();
    for (body, code) in [
        ("ag_set_local(0,3,1.0)", 2),
        ("ag_set_visible(0,2)", 5),
        ("ag_move_local(ag_actor(2),0.0,0.0,1.0)", 4),
        ("ag_actor(4)", 3),
    ] {
        let s = format!("fn app_start(){{{body}}}\nfn app_update(dt float){{}}\n");
        let mut p = Play::new_user(&e, [Some(12), Some(11)], snapshot(&s)).unwrap();
        assert!(ready(&mut p).unwrap_err().contains(&format!("code={code}")));
        assert!(p.last.is_none());
    }
    let mut p=Play::new_user(&e,[None,None],snapshot("fn app_start(){if ag_valid(ag_actor(2))==0 {ag_set_visible(0,1)}}\nfn app_update(dt float){}\n")).unwrap();
    ready(&mut p).unwrap();
}
#[test]
fn callback_duplicate_and_reserved_ast_admission() {
    for source in [
        "fn app_start(){}",
        "fn app_start(x int){}\nfn app_update(dt float){}",
        "fn app_start() int{return 1}\nfn app_update(dt float){}",
        "fn app_start(){}\nfn app_update(dt int){}",
        "var x=0\nvar x=1\nfn app_start(){}\nfn app_update(dt float){}",
        "fn profile(){}\nfn app_start(){}\nfn app_update(dt float){}",
        "fn app_start(){reset(0,0,0)}\nfn app_update(dt float){}",
        "fn app_start(){let x=_ag_error}\nfn app_update(dt float){}",
        "use unknown: value\nfn app_start(){}\nfn app_update(dt float){}",
        "var x=ag_tick()\nfn app_start(){}\nfn app_update(dt float){}",
        "fn app_start(){}\nfn app_update(dt float){}\napp_start()",
    ] {
        let e =
            SourceSnapshot::from_bytes(Path::new("D:/real/user.at"), source.as_bytes().to_vec())
                .unwrap_err();
        assert!(
            e.contains("D:/real/user.at") && e.contains("admission"),
            "{e}"
        );
    }
    let accepted="var text=\"reset _ag_error\"\n/*\nuse nonexistent\n*/\nfn helper(x float) float{return x+1.0}\nfn app_start(){let reset=3; let fields=reset+1; let words=\"_ag_error reset()\"; ag_set_local(0,0,helper(1.0))}\nfn app_update(dt float){}\n";
    let e = fixture();
    let mut p = Play::new_user(&e, [None, None], snapshot(accepted)).unwrap();
    ready(&mut p).unwrap();
    assert_eq!(
        p.last.as_ref().unwrap().document.groups[0].translation[0],
        2.0
    );
}
#[test]
fn source_bounds_raw_hash_regions_and_identity_rejection() {
    let source = "fn app_start(){}\nfn app_update(dt float){}\n";
    let a = snapshot(source);
    let b = snapshot(&source.replace('\n', "\r\n"));
    assert_ne!(a.user_hash, b.user_hash);
    assert_eq!(a.version, VERSION);
    let r = a
        .regions
        .iter()
        .find(|r| r.file == "D:/user/main.at")
        .unwrap();
    assert_eq!(&a.source.as_bytes()[r.start..r.end], source.as_bytes());
    for bytes in [
        vec![0xff],
        [vec![0xef, 0xbb, 0xbf], source.as_bytes().to_vec()].concat(),
        vec![b' '; MAX_USER + 1],
    ] {
        assert!(SourceSnapshot::from_bytes(Path::new("D:/bad.at"), bytes).is_err());
    }
    let mut maximum = source.to_string();
    maximum.push_str("//");
    maximum.extend(std::iter::repeat_n(' ', MAX_USER - maximum.len()));
    assert_eq!(snapshot(&maximum).bytes.len(), MAX_USER);
    let e = fixture();
    let mut wrong = a.clone();
    wrong.version = "autographics.behavior/2".into();
    assert!(Play::new_user(&e, [None, None], wrong).is_err());
    let mut wrong = a;
    wrong.source.push_str("\n");
    assert!(Play::new_user(&e, [None, None], wrong).is_err());
}
#[test]
fn source_snapshot_does_not_hot_reload_and_new_read_sees_change() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/behavior-read");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.at");
    let a = "fn app_start(){ag_set_local(0,0,2.0)}\nfn app_update(dt float){}";
    std::fs::write(&path, a).unwrap();
    let first = SourceSnapshot::read(&path).unwrap();
    std::fs::write(&path, a.replace("2.0", "7.0")).unwrap();
    let e = fixture();
    let mut p = Play::new_user(&e, [None, None], first).unwrap();
    ready(&mut p).unwrap();
    assert_eq!(
        p.last.as_ref().unwrap().document.groups[0].translation[0],
        2.0
    );
    p.stop();
    let mut p = Play::new_user(&e, [None, None], SourceSnapshot::read(&path).unwrap()).unwrap();
    ready(&mut p).unwrap();
    assert_eq!(
        p.last.as_ref().unwrap().document.groups[0].translation[0],
        7.0
    );
}
#[test]
fn callback_budget_fault_and_128_node_legal_control() {
    let e = fixture();
    let mut p = Play::new_user(
        &e,
        [None, None],
        snapshot("fn app_start(){}\nfn app_update(dt float){while true {}}\n"),
    )
    .unwrap();
    ready(&mut p).unwrap();
    let owner = p.owner_probe().unwrap();
    p.dispatch(Action::default()).unwrap();
    assert!(ready(&mut p).is_err());
    assert!(!owner());
    let mut d = Document::empty("128".into());
    for i in 1..=128 {
        d.objects.push(Object::new(i, Kind::Quad));
    }
    let e = Editor::new(d).unwrap();
    let mut p=Play::new_user(&e,[None,None],snapshot("fn app_start(){ag_set_local(0,0,1.0)}\nfn app_update(dt float){ag_set_local(0,0,ag_world(0,0)+dt)}\n")).unwrap();
    ready(&mut p).unwrap();
    p.dispatch(Action::default()).unwrap();
    ready(&mut p).unwrap();
    assert_eq!(p.last.as_ref().unwrap().objects.len(), 128);
}

#[test]
fn compile_span_maps_back_to_user_unit_and_runtime_does_not_fabricate_line() {
    let e = fixture();
    let s = snapshot(
        "fn app_start(){\n    missing_behavior_function()\n}\nfn app_update(dt float){}\n",
    );
    let err = Play::new_user(&e, [None, None], s).err().unwrap();
    assert!(err.contains("D:/user/main.at:2:"), "{err}");
    assert!(err.contains("compile"));
    let s = snapshot("fn app_start(){}\nfn app_update(dt float){while true {}}\n");
    let mut p = Play::new_user(&e, [None, None], s).unwrap();
    ready(&mut p).unwrap();
    p.dispatch(Action::default()).unwrap();
    let err = ready(&mut p).unwrap_err();
    assert!(err.contains("SDK supplied no mapped line"));
    assert!(err.contains("app_update"));
}

#[test]
fn multiline_diagnostic_and_snapshot_metadata_are_not_trusted() {
    let err = SourceSnapshot::from_bytes(
        Path::new("D:/user/main.at"),
        b"/* first\nsecond\nthird */\nfn app_start(x int){}\nfn app_update(dt float){}".to_vec(),
    )
    .unwrap_err();
    assert!(err.contains("user line 4"), "{err}");
    let e = fixture();
    let s = snapshot("fn app_start(){}\nfn app_update(dt float){}");
    let mut wrong = s.clone();
    wrong.regions[0].end += 1;
    assert!(Play::new_user(&e, [None, None], wrong).is_err());
    let mut wrong = s.clone();
    wrong.base_scene = Some(serde_json::json!({"document_hash":"forged"}));
    assert!(Play::new_user(&e, [None, None], wrong).is_err());
    let mut p = Play::new_user(&e, [None, None], s).unwrap();
    ready(&mut p).unwrap();
    assert_eq!(
        p.initial_document_hash.as_ref(),
        Some(&e.document.hash().unwrap())
    );
    assert!(p.user_source.as_ref().unwrap().base_scene.is_some());
}

#[test]
fn callback_unit_returns_cover_expression_operands_in_both_callbacks() {
    // Each is valid Core syntax. The return exits the callback, even when it
    // appears in an operand rather than a top-level callback statement.
    for body in [
        "let x=if true {return 1} else {return 2}",
        "var x=0; x=if true {return 1} else {return 2}",
        "sink(if true {return 1} else {return 2})",
        "let xs=[if true {return 1} else {return 2}]",
        "if (if true {return 1} else {return 2})==1 {}",
        "for i in 0..1 {let x=if true {return 1} else {return 2}}",
        "for let fields=if true {return 1} else {return 2}; fields<1 {}",
        "while true {let x=if true {return 1} else {return 2}}",
        "let text=f\"${if true {return 1} else {return 2}}\"",
    ] {
        for callback in ["app_start", "app_update"] {
            let source = format!(
                "fn sink(x int){{}}\nfn app_start() void {{{}}}\nfn app_update(dt float) void {{{}}}",
                if callback == "app_start" { body } else { "" },
                if callback == "app_update" { body } else { "" },
            );
            let err =
                SourceSnapshot::from_bytes(Path::new("D:/user/returns.at"), source.into_bytes())
                    .unwrap_err();
            assert!(
                err.contains("callback must return Unit"),
                "{callback}: {body}: {err}"
            );
        }
    }
}

#[test]
fn callback_return_policy_stops_at_nested_function_and_closure_boundaries() {
    for body in [
        "fn helper() float {return 3.0}; ag_set_local(0,0,helper())",
        "let helper=(x float)=>{return x+1.0}; ag_set_local(0,0,helper(2.0))",
        "for fields in 0..1 {ag_set_local(0,0,3.0); break}",
        "if true {return}",
    ] {
        let source = format!("fn app_start(){{{body}}}\nfn app_update(dt float){{{body}}}");
        let e = fixture();
        let before = e.isolation_identity();
        let mut p = Play::new_user(&e, [None, None], snapshot(&source)).unwrap();
        ready(&mut p).unwrap();
        let owner = p.owner_probe().unwrap();
        p.dispatch(Action::default()).unwrap();
        ready(&mut p).unwrap();
        assert_eq!(p.tick(), 1);
        if body.contains("ag_set_local") {
            assert_eq!(
                p.last.as_ref().unwrap().document.groups[0].translation[0],
                3.0
            );
        }
        p.stop();
        assert!(!owner());
        assert_eq!(e.isolation_identity(), before);
    }
}

#[test]
fn immutable_constant_initializers_execute_without_weakening_dynamic_rejection() {
    let e = fixture();
    for source in [
        "const BASE=1.0\nconst NEXT=BASE+1.0\nvar x=NEXT+1.0\nfn app_start(){ag_set_local(0,0,x)}\nfn app_update(dt float){}",
        "let BASE=1.0\nvar x=BASE+2.0\nfn app_start(){ag_set_local(0,0,x)}\nfn app_update(dt float){}",
        "const N=3.0\nvar text=f\"literal${N}\"\nvar raw=`literal\nuse nonexistent\n`\nfn app_start(){if raw.len()==24 {if text.len()>7 {ag_set_local(0,0,N)}}}\nfn app_update(dt float){}",
        "const VALUES=[1.0,3.0]\nvar x=VALUES[1]\nfn app_start(){ag_set_local(0,0,x)}\nfn app_update(dt float){}",
    ] {
        let mut p = Play::new_user(&e, [None, None], snapshot(source)).unwrap();
        ready(&mut p).unwrap();
        assert_eq!(p.last.as_ref().unwrap().document.groups[0].translation[0], 3.0);
        p.stop();
    }
    for declarations in [
        "var BASE=1.0\nvar x=BASE+2.0",
        "const A=B\nconst B=A",
        "var x=unknown_constant+1.0",
        "var text=f\"${ag_tick()}\"",
        "const A=1\nvar x=(A+=2)",
        "const A=1\nvar x=_ag_run+A",
        "const A=ag_tick()",
    ] {
        let source = format!("{declarations}\nfn app_start(){{}}\nfn app_update(dt float){{}}");
        let err =
            SourceSnapshot::from_bytes(Path::new("D:/user/constants.at"), source.into_bytes())
                .unwrap_err();
        assert!(err.contains("admission"), "{err}");
    }
}
