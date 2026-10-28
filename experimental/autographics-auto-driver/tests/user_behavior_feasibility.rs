//! PLAN-840 T02: scratch assembly using the unchanged public SDK and Play.
//! Profile6 is deliberately retained for this mechanism probe, not the user ABI.
use autographics_auto_driver::authoring::{
    assets,
    document::{Dimension, Document, Group, Kind, Object},
    host::Host,
    play::{Action, SOURCE},
};
use std::time::{Duration, Instant};

fn assemble(user: &str) -> String {
    // Only the trusted managed bridge is adapted; user bytes are concatenated intact.
    let start = SOURCE.find("fn tick(").unwrap();
    let end = SOURCE.find("fn run_id(").unwrap();
    let bridge = format!(
        "{}{}{}",
        &SOURCE[..start],
        "fn tick(next int, dt float) { sequence=next; app_update(dt); scratch_derive() }\n",
        &SOURCE[end..]
    );
    let bridge = bridge
        .replace(
            "fn reset(token int, two int, three int) {",
            "fn reset(token int, two int, three int) { main();",
        )
        .replace("fn derive() {", "fn scratch_derive() {");
    // init callback after the complete configured graph, before the first query.
    format!(
        "{user}\n{bridge}\nfn derive() {{ app_start(); scratch_derive() }}\n\
fn ag_actor(space int) int {{ if space==2 {{return actor2}}; return actor3 }}\n\
fn ag_local(n int, axis int) float {{return fields[n*12+axis]}}\n\
fn ag_set_local(n int, axis int, value float) {{fields[n*12+axis]=value}}\n\
fn ag_world(n int, axis int) float {{scratch_derive(); return worlds[n*3+axis]}}\n\
fn ag_visible(n int) int {{scratch_derive(); return effective[n]}}\n\
fn ag_parent(n int) int {{return parents[n]}}\n"
    )
}
fn fixture() -> Host {
    let asset = assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap();
    let mut d = Document::empty("user-feasibility".into());
    let mut g = Group::new(10, Dimension::Three);
    g.translation = [1.0, 0.0, 0.0];
    d.groups.push(g);
    let mut m = Object::new(11, Kind::Model);
    m.parent = Some(10);
    m.translation = [-1.0, 0.0, 0.0];
    m.resource = Some(asset.identity.key());
    d.objects.push(m);
    let mut q = Object::new(12, Kind::Quad);
    q.translation = [-100.0, 0.0, 0.0];
    d.objects.push(q);
    d.assets.push(asset.identity.clone());
    Host::with_assets(d, assets::Registry::from([(asset.identity.key(), asset)])).unwrap()
}
fn settle(h: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        h.frame(None).unwrap();
        assert!(Instant::now() < deadline);
    }
}
#[test]
fn intact_user_callbacks_globals_group_read_after_write_gpu_stop_same_binary() {
    let mut h = fixture();
    h.frame(None).unwrap();
    // The panel now reports last-Run source metadata. Stop restores both
    // editing canvases; session diagnostics intentionally remain visible.
    let original = (
        h.rt.capture(h.two, "original2").unwrap().0,
        h.rt.capture(h.three, "original3").unwrap().0,
    );
    let identity = h.editor.isolation_identity();
    // app_start uses a nonzero initializer, rather than accidentally accepting Nil=0.
    let a="var initial=2.0\nvar starts=0\nfn app_start(){ starts=starts+1; ag_set_local(ag_actor(2),0,-100.0+initial) }\nfn app_update(dt float){ let n=ag_actor(3); let p=ag_parent(n); ag_set_local(p,0,ag_local(p,0)+3.0*dt); let world=ag_world(n,0); ag_set_local(ag_actor(2),0,-100.0+world) }\n";
    for (source, speed) in [(a.to_string(), 3.0), (a.replace("3.0*dt", "6.0*dt"), 6.0)] {
        h.start_play(&assemble(&source)).unwrap();
        h.pause_play("");
        settle(&mut h);
        assert_eq!(
            h.play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .unwrap()
                .document
                .objects[1]
                .translation[0],
            -98.0
        );
        let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
        for _ in 0..60 {
            h.play
                .as_mut()
                .unwrap()
                .dispatch(Action::default())
                .unwrap();
            settle(&mut h);
        }
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
        assert!((frame.objects[0].translation[0] - speed).abs() < 1e-4);
        assert!((frame.objects[1].translation[0] - (-100.0 + speed)).abs() < 1e-3);
        let x = frame.objects[0].translation[0];
        for (i, dx) in [-1.5, 1.5].into_iter().enumerate() {
            let p = autographics_auto_driver::authoring::interaction::project(
                h.camera(),
                glam::Vec3::new(x + dx, 0.5, 0.0),
            )
            .unwrap();
            let pixel = h.rt.probe_pixel(h.top, [p.x, p.y + 400.0]).unwrap();
            assert!(
                pixel[i] > 80 && pixel[1 - i] < 20,
                "all parts at user-driven world {pixel:?}"
            );
        }
        assert_ne!(
            (
                h.rt.capture(h.two, "user2").unwrap().0,
                h.rt.capture(h.three, "user3").unwrap().0
            ),
            original
        );
        h.stop_play();
        h.frame(None).unwrap();
        assert!(!owner());
        assert_eq!(h.editor.isolation_identity(), identity);
        assert_eq!(
            (
                h.rt.capture(h.two, "restored2").unwrap().0,
                h.rt.capture(h.three, "restored3").unwrap().0
            ),
            original
        );
    }
}

fn inert_text(kind: &str) -> String {
    let control = "fn app_start(){}\nfn app_update(dt float){}\n";
    match kind {
        "block-comment" => format!("/*\nuse plan840_missing_dependency: value\n*/\n{control}"),
        "string" => {
            format!("var note=\"text\nuse plan840_missing_dependency: value\nend\"\n{control}")
        }
        _ => panic!("unknown fixture"),
    }
}
fn check_inert_text(kind: &str) {
    use auto_lang_core::{ast::Stmt, parser::Parser};
    use autographics_auto_driver::Session;
    let control = "fn app_start(){}\nfn app_update(dt float){}\n";
    Session::new(&assemble(control)).unwrap();
    let user = inert_text(kind);
    let ast = Parser::from(user.as_str()).parse().unwrap();
    assert!(!ast
        .stmts
        .iter()
        .any(|s| matches!(s, Stmt::Use(_) | Stmt::Dep(_))));
    let result = Session::new(&assemble(&user));
    assert!(
        result.is_ok(),
        "{kind}: AST has no dependency, but SDK source preprocessing rejected it: {:?}",
        result.err()
    );
}
#[test]
fn sdk_preprocessing_must_not_treat_block_comment_as_dependency() {
    check_inert_text("block-comment");
}
#[test]
fn sdk_preprocessing_must_not_treat_string_as_dependency() {
    check_inert_text("string");
}

#[test]
fn public_module_path_must_not_treat_inert_text_as_dependency() {
    use auto_lang_core::compile::CompileSession;
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/plan840-module-probe");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("plan840_probe.at");
    // Existing public module API. Distinct sessions avoid masking via caches.
    std::fs::write(&file, "fn app_start(){}\nfn app_update(dt float){}\n").unwrap();
    let mut control = CompileSession::new();
    control.add_source_dir(dir.clone());
    assert!(control.resolve_uses("use plan840_probe.*\n").is_ok());
    let user = inert_text("block-comment");
    let ast = auto_lang_core::parser::Parser::from(user.as_str())
        .parse()
        .unwrap();
    assert!(!ast.stmts.iter().any(|s| matches!(
        s,
        auto_lang_core::ast::Stmt::Use(_) | auto_lang_core::ast::Stmt::Dep(_)
    )));
    std::fs::write(&file, &user).unwrap();
    let mut session = CompileSession::new();
    session.add_source_dir(dir);
    let result = session.resolve_uses("use plan840_probe.*\n");
    assert!(
        result.is_ok(),
        "public module path also scans inert text: {result:?}"
    );
}

#[test]
fn sdk_scanners_and_fresh_cached_modules_preserve_real_dependencies() {
    use auto_lang_core::{
        compile::CompileSession, dep_scanner::scan_dep_statements, use_scanner::scan_use_statements,
    };
    let inert="/*\nuse missing_block\ndep missing_block\n*/\nvar text=\"\nuse missing_string\ndep missing_string\n\"\n";
    assert!(scan_use_statements(inert).is_empty());
    assert!(scan_dep_statements(inert).is_empty());
    let real = format!(
        "{inert}use real.module: value\nuse real.module: value\ndep real(path: \"folder//data\")\n"
    );
    assert_eq!(scan_use_statements(&real).len(), 1);
    assert_eq!(
        scan_dep_statements(&real)[0].path.as_deref(),
        Some("folder//data")
    );
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/plan840-cache-probe");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("p840_base.at"), "fn value() int {return 42}\n").unwrap();
    std::fs::write(
        dir.join("p840_entry.at"),
        format!("{inert}use p840_base: value\nfn result() int {{return value()}}\n"),
    )
    .unwrap();
    let mut fresh = CompileSession::new();
    fresh.add_source_dir(dir.clone());
    fresh.resolve_uses("use p840_entry.*\n").unwrap();
    assert_eq!(fresh.cached_module_count(), 2);
    assert_eq!(fresh.compiled_modules.len(), 2);
    let mut cached = fresh.clone();
    assert_eq!(cached.cache_stats().valid, 2);
    cached.resolve_uses("use p840_entry.*\n").unwrap();
    assert_eq!(
        cached.compiled_modules.len(),
        2,
        "cache hit must still compile the real dependency closure"
    );
    // A real unresolved import still fails; inert imports above must stay absent.
    std::fs::write(
        dir.join("p840_bad.at"),
        "use p840_missing_real.*\nfn result(){}\n",
    )
    .unwrap();
    let mut bad = CompileSession::new();
    bad.add_source_dir(dir);
    assert!(bad.resolve_uses("use p840_bad.*\n").is_err());
}
