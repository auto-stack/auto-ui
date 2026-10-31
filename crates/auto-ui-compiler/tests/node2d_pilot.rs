//! PLAN-016 T-03 (V-01, compiler side): the real minimal pilot project
//! binds through the existing production module-resolution surface —
//! `scan_use_statements` + `resolve_use_module` + `collect_use_module_fns`
//! — with no new parser and no production-code changes. The research
//! services namespace (`auto.graphics`) resolves to nothing by design and
//! is owned by the pilot validator; missing modules resolve to `None` so
//! the pilot can raise positioned diagnostics before execution.
//! Run: cargo test -p auto-ui-compiler --test node2d_pilot
use auto_lang_core::use_scanner::scan_use_statements;
use auto_ui_compiler::ui_ast::{resolve_use_module, UseModuleResolution};
use std::path::Path;

fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experimental/autographics-node2d-pilot/fixtures/minimal")
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_dir().join(name)).expect("fixture readable")
}

#[test]
fn file_module_uses_resolve_to_existing_fixture_files() {
    // main.at imports five real file modules.
    let main = read_fixture("main.at");
    let uses = scan_use_statements(&main);
    let modules: Vec<&str> = uses.iter().map(|u| u.module.as_str()).collect();
    assert_eq!(modules, vec!["types", "player", "hazard", "coin", "patrol"]);
    for u in &uses {
        match resolve_use_module(&fixture_dir(), u) {
            UseModuleResolution::Module(path) => {
                assert!(path.is_file(), "resolved module must exist: {path:?}");
            }
            UseModuleResolution::None => {
                panic!("use {} must resolve to a module file, got None", u.module)
            }
            UseModuleResolution::StoreFiles(_) => panic!(
                "use {} must resolve to a module file, got StoreFiles",
                u.module
            ),
        }
    }

    // player.at imports the whole resources module and the services
    // namespace; hazard.at imports a named pure function from patrol.
    let player = read_fixture("player.at");
    let uses = scan_use_statements(&player);
    let modules: Vec<&str> = uses.iter().map(|u| u.module.as_str()).collect();
    assert_eq!(modules, vec!["resources", "auto.graphics"]);
    assert!(matches!(
        resolve_use_module(&fixture_dir(), &uses[0]),
        UseModuleResolution::Module(_)
    ));
}

#[test]
fn research_services_namespace_is_not_a_file_module() {
    // `use auto.graphics: ...` is the frozen research services namespace
    // (contract S2/S5): it must NOT resolve to a file — the pilot validator
    // owns binding it to AG service symbols.
    let resources = read_fixture("resources.at");
    let uses = scan_use_statements(&resources);
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].module, "auto.graphics");
    assert!(matches!(
        resolve_use_module(&fixture_dir(), &uses[0]),
        UseModuleResolution::None
    ));
}

#[test]
fn missing_module_resolves_none_for_positioned_diagnostics() {
    let temp = tempfile::tempdir().expect("tempdir");
    let uses = scan_use_statements("use ghost_module: Thing\n");
    assert_eq!(uses.len(), 1);
    assert!(matches!(
        resolve_use_module(temp.path(), &uses[0]),
        UseModuleResolution::None
    ));
}

#[test]
fn cross_module_fn_import_pools_the_patrol_function() {
    // The production import path pools real fns from imported modules:
    // main.at's `use patrol: patrol_y` brings the pure patrol function in.
    let main = read_fixture("main.at");
    let app = fixture_dir().join("main.at");
    let (functions, imported) = auto_ui_compiler::ui_gen::api::collect_use_module_fns(&app, &main);
    // `imported` lists pooled item names across all visited modules; the
    // patrol pure function must be among them with a real body.
    assert!(
        imported.contains(&"patrol_y".to_string()),
        "patrol_y pooled, got {imported:?}"
    );
    let pooled = functions.iter().find(|f| f.name.to_string() == "patrol_y");
    assert!(pooled.is_some(), "patrol_y fn present in pool");
    assert!(
        !pooled.unwrap().body.is_empty(),
        "patrol_y pooled with a real body"
    );
}

#[test]
fn unparseable_imported_module_fails_validation_with_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("brokenmod.at"), "fn broken( {\n").expect("write");

    // No import: no failure.
    let app = temp.path().join("app.at");
    std::fs::write(&app, "widget App { view { text \"x\" {} } }\n").expect("write");
    let result = auto_ui_compiler::ui_gen::api::validate_ui_file_with_modules(&app);
    assert!(result.is_ok(), "no import means no failure: {result:?}");

    // Broken imported module: the production diagnostic names the path.
    let app2 = temp.path().join("app2.at");
    std::fs::write(
        &app2,
        "widget App { view { text \"x\" {} } }\nuse brokenmod: f\n",
    )
    .expect("write");
    let result2 = auto_ui_compiler::ui_gen::api::validate_ui_file_with_modules(&app2);
    let err = result2.expect_err("broken module must fail validation");
    assert!(err.contains("brokenmod"), "path in diagnostic: {err}");
}

#[test]
fn fixture_files_are_nonempty_utf8_lf() {
    // The frozen contract binds exact fixture bytes (gen_plan016 identity,
    // T-06). This check pins the read-shape assumptions: non-empty UTF-8,
    // LF endings, no BOM.
    for name in [
        "main.at",
        "resources.at",
        "types.at",
        "player.at",
        "hazard.at",
        "coin.at",
        "patrol.at",
    ] {
        let bytes = std::fs::read(fixture_dir().join(name)).expect("fixture");
        assert!(!bytes.is_empty(), "{name} must be non-empty");
        assert!(!bytes.starts_with(&[0xef, 0xbb, 0xbf]), "{name} no BOM");
        assert_eq!(bytes.last(), Some(&b'\n'), "{name} must end with LF");
        std::str::from_utf8(&bytes).expect("{name} must be UTF-8");
    }
}
