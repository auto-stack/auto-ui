//! PLAN-109: synchronous re-emission through mounted component owners.
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_val::Value;
use std::{fs, time::Instant};

const LEAF: &str = r#"
widget Leaf (payload: str = "") {
    msg { Send, "sent"(str) }
    view { button (text: .payload, onclick: .Send) {} }
    on { .Send -> { try { ."sent"(.payload) } catch {} } }
}
"#;

const MIDDLE: &str = r#"
use leaf: Leaf

widget Middle (payload: str = "") {
    msg { Forward(str), "middleSent"(str) }
    model { var observed str = "" }
    view { Leaf (payload: .payload, onsent: .Forward($event)) {} }
    on {
        .Forward(value str) -> {
            .observed = value
            try { ."middleSent"(value) } catch {}
        }
    }
}
"#;

const OUTER: &str = r#"
use middle: Middle

widget Outer (payload: str = "") {
    msg { Forward(str), "outerSent"(str) }
    view { Middle (payload: .payload, onmiddleSent: .Forward($event)) {} }
    on { .Forward(value str) -> { try { ."outerSent"(value) } catch {} } }
}
"#;

const APP: &str = r#"
widget App {
    use { package: emit_probe from "<PACKAGE>" }
    msg { Collect(str), Close }
    model {
        var show_a bool = true
        var observed str = ""
        var count int = 0
    }
    view {
        col {
            if .show_a { Outer (key: "a", payload: "A", onouterSent: .Collect($event)) {} }
            Outer (key: "b", payload: "B", onouterSent: .Collect($event)) {}
            Outer (key: "unbound", payload: "none") {}
            Leaf (key: "direct", payload: "direct", onsent: .Collect($event)) {}
            button (text: "Close A", onclick: .Close) {}
        }
    }
    on {
        .Collect(s str) -> { .observed = s
            .count = .count + 1 }
        .Close -> { .show_a = false }
    }
}
"#;

fn build(dir: &tempfile::TempDir) -> DynamicComponent {
    let package = dir.path().join("emit_probe");
    fs::create_dir(&package).unwrap();
    for (name, source) in [
        ("package.at", "name: \"emit-probe\"\nversion: \"0.1.0\"\n"),
        ("leaf.at", LEAF),
        ("middle.at", MIDDLE),
        ("outer.at", OUTER),
    ] {
        fs::write(package.join(name), source).unwrap();
    }
    let app = dir.path().join("app.at");
    let source = APP.replace("<PACKAGE>", &package.to_string_lossy().replace('\\', "/"));
    fs::write(&app, &source).unwrap();
    let app = auto_ui_runtime::build_dynamic_component(&source, app.to_str()).unwrap();
    app.view_with_debug_gated(false);
    app
}

fn root_state(app: &DynamicComponent, observed: &str, count: i32) {
    assert_eq!(
        app.bridge().read_state("observed").unwrap(),
        Value::str(observed)
    );
    assert_eq!(app.bridge().read_state("count").unwrap(), Value::Int(count));
}

#[test]
fn nested_explicit_emit_follows_the_live_owner_chain() {
    let start = Instant::now();
    let mut scratch = tempfile::Builder::new();
    scratch.prefix("nested-collection-emit-");
    let dir = match std::env::var_os("PLAN109_ARTIFACT_ROOT") {
        Some(root) => scratch.tempdir_in(root),
        None => scratch.tempdir(),
    }
    .unwrap();
    let mut app = build(&dir);
    let leaf = |app: &DynamicComponent, parent| {
        app.bridge()
            .child_instance_identities_of_type("Leaf")
            .into_iter()
            .find(|id| id.starts_with(parent))
            .unwrap()
    };
    let leaf_a = leaf(&app, "/Outer#a/");
    let leaf_b = leaf(&app, "/Outer#b/");
    let leaf_unbound = leaf(&app, "/Outer#unbound/");

    app.on_with_input_for_instance("Leaf", "Send", None, Some(&leaf_a));
    root_state(&app, "A", 1);
    let middles = app.bridge().child_instance_identities_of_type("Middle");
    for (owner, expected) in [("/Outer#a/", "A"), ("/Outer#b/", "")] {
        let id = middles.iter().find(|id| id.starts_with(owner)).unwrap();
        assert_eq!(
            app.bridge().read_instance_state(id, "observed").unwrap(),
            Value::str(expected)
        );
    }

    app.on_with_input_for_instance("Leaf", "Send", None, Some(&leaf_unbound));
    root_state(&app, "A", 1);
    app.on_with_input_for_instance("App", "Close", None, None);
    app.view_with_debug_gated(false);
    app.on_with_input_for_instance("Leaf", "Send", None, Some(&leaf_a));
    root_state(&app, "A", 1);

    app.on_with_input_for_instance("Leaf", "Send", None, Some(&leaf_b));
    root_state(&app, "B", 2);
    app.on_with_input_for_instance("Leaf", "Send", None, Some("/Leaf#direct"));
    root_state(&app, "direct", 3);
    drop(app);
    dir.close().unwrap();
    eprintln!(
        "nested_explicit_emit_follows_the_live_owner_chain {:?}",
        start.elapsed()
    );
}
