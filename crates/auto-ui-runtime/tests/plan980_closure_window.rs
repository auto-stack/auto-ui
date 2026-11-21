//! PLAN-980 ④-A 首片 #2b：**AGE 包闭包的桌面桥**——CPU 面与真窗面。
//!
//! 输入是 ④-A 派生真实例（`ace-node-counter` 派生 + 点分 `use util.steps: step`
//! + 惰性 `use themes.themes: Auto, Solarized`）；两实例场景 + 实例 0 的 `Inc`。
//! 判据：状态行的 AV 值随 helper 源从 `[4, 2]` 变为 `[20, 10]`（CPU 与真窗同源）。
//!
//! 包场景桥观察标量状态和文本，非空 props/initials 具名拒绝。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_node_desktop::{
    ensure_ui980_provider, push_event, NodeDesktopBridge, StepOutcome,
};

#[path = "support/ui980.rs"]
mod ui980;

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 派生真实例（与 `plan980_closure_av` 同源）：初始化 `step(0)`、`Inc` 走
/// `step(.count)`；按钮声明序 = Reset(0) / +(1)。
fn derived_counter() -> String {
    "use util.steps: step\n\
use themes.themes: Auto, Solarized\n\
\n\
node Counter {\n\
    msg { Inc, Reset }\n\
\n\
    model {\n\
        var count int = step(0)\n\
    }\n\
\n\
    view {\n\
        col {\n\
            text f\"Counter ${.count}\"\n\
            row {\n\
                button \"Reset\" { onclick: .Reset }\n\
                button \"+\" { onclick: .Inc }\n\
            }\n\
        }\n\
    }\n\
\n\
    on {\n\
        .Inc -> { .count = step(.count) }\n\
        .Reset -> { .count = 0 }\n\
    }\n\
}\n"
    .to_string()
}

const HELPER: &str = "pub fn step(n int) int {\n    return n + 2\n}\n";
const HELPER_EDITED: &str = "pub fn step(n int) int {\n    return n + 10\n}\n";

const TARGET_INC: usize = 1;

fn load(helper: &str) -> NodeDesktopBridge {
    ui980::require_prepared();
    ensure_ui980_provider().expect("ui980 provider");
    NodeDesktopBridge::load_package(
        vec![
            unit("app", "main", &derived_counter()),
            unit("util", "steps", helper),
        ],
        "app::main",
        2,
    )
    .expect("package bridge loads")
}

fn count_of(bridge: &NodeDesktopBridge, instance: usize) -> i32 {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
    bridge
        .states(instance)
        .unwrap_or_else(|| panic!("instance {instance} live"))
        .into_iter()
        .find(|(n, _)| n == "count")
        .map(|(_, v)| match v {
            AceStateValue::Int(i) => i,
            other => panic!("count is not int: {other:?}"),
        })
        .unwrap_or_else(|| panic!("no count field"))
}

/// CPU 面：包闭包桥装载 → open → 实例 0 `Inc` → 状态 `[4, 2]`；改 helper 源
/// → `[20, 10]`（同一场景、同一 AV 观察链）。
#[test]
fn p980_closure_package_bridge_states_follow_the_helper() {
    let mut base = load(HELPER);
    base.open().expect("open");
    push_event(0, TARGET_INC);
    match base.step() {
        StepOutcome::Dispatched { instance, msg, .. } => {
            assert_eq!((instance, msg.as_str()), (0, "Inc"));
        }
        other => panic!("expected a dispatched Inc, got {other:?}"),
    }
    assert_eq!(count_of(&base, 0), 4, "step(0) then step(2)");
    assert_eq!(
        count_of(&base, 1),
        2,
        "instance 1 keeps the initial step(0)"
    );
    let assert_texts = |bridge: &NodeDesktopBridge| {
        let evidence = bridge.evidence();
        let session = evidence["sessions"].as_array().unwrap().last().unwrap();
        assert_eq!(session["texts"], serde_json::json!([["Counter 4"], ["Counter 2"]]));
    };
    assert_texts(&base);
    base.close_session().expect("close");
    base.reopen().expect("reopen");
    assert_texts(&base);
    base.close_session().expect("close after reopen");

    let mut edited = load(HELPER_EDITED);
    edited.open().expect("open");
    push_event(0, TARGET_INC);
    edited.step();
    assert_eq!(count_of(&edited, 0), 20, "step(0)=10 then step(10)=20");
    assert_eq!(count_of(&edited, 1), 10, "initial 10");
}

/// 负控：包面场景无 props/initials 面——非空种子具名拒（不静默丢种子）。
#[test]
fn p980_closure_package_bridge_refuses_initials() {
    ui980::require_prepared();
    ensure_ui980_provider().expect("ui980 provider");
    let err = NodeDesktopBridge::load_package(
        vec![
            unit("app", "main", &derived_counter()),
            unit("util", "steps", HELPER),
        ],
        "app::main",
        1,
    )
    .err();
    assert!(err.is_none(), "load itself does not take initials");

    let mut engine = auto_ui_runtime::ui::ace_package_bridge::PackageStateEngine::new(
        vec![
            unit("app", "main", &derived_counter()),
            unit("util", "steps", HELPER),
        ],
        "app::main",
        ace_av::age_component::LegacyWidgetAlias::Accept,
    )
    .expect("engine");
    let err = engine
        .mount(
            "i0",
            vec![("count".into(), ace_av::age_component::InitialValue::Int(9))],
        )
        .expect_err("package initials are a named pending surface");
    assert!(err.contains("no props/initials surface"), "{err}");
}

#[test]
fn p107_package_bridge_replays_pointer_floats_in_the_entry_instance() {
    use auto_ui_runtime::ui::ace_package_bridge::PackageStateEngine;
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue::{F32, Int};

    let source = r#"
node Plot {
    msg { PointerMove(float, f32), Clear }
    model { var side int = 0 }
    view { text "Plot" }
    on {
        .PointerMove(x, y) -> {
            if x > y {
                .side = 1
            } else {
                .side = 2
            }
        }
        .Clear -> { .side = 0 }
    }
}
"#;
    let mut engine = PackageStateEngine::new(
        vec![
            unit("aux", "first", r#"node Notice { view { text "Other component" } }"#),
            unit("app", "main", source),
        ],
        "app::main",
        ace_av::age_component::LegacyWidgetAlias::Reject,
    )
    .expect("checked pointer message");
    engine.mount("a", vec![]).unwrap();
    engine.mount("b", vec![]).unwrap();
    let read = |engine: &PackageStateEngine, identity: &str| {
        engine.read_all(identity).expect("live entry state")
    };
    let side = |value| vec![("side".to_owned(), Int(value))];
    assert!(engine.dispatch("a", "PointerMove").is_err());
    assert!(engine.dispatch_with_args("a", "PointerMove", &[Int(2), F32(2.25)]).is_err());
    assert!(engine.dispatch_with_args("a", "PointerMove", &[F32(f32::NAN), F32(2.25)]).is_err());
    assert_eq!(read(&engine, "a"), side(0));
    engine.dispatch_with_args("a", "PointerMove", &[F32(2.5), F32(2.25)]).unwrap();
    engine.dispatch_with_args("b", "PointerMove", &[F32(2.25), F32(2.5)]).unwrap();
    assert_eq!(read(&engine, "a"), side(1));
    assert_eq!(read(&engine, "b"), side(2));
    assert_eq!(engine.texts("a").unwrap(), ["Plot"]);
    engine.dispatch("a", "Clear").unwrap();
    assert_eq!(read(&engine, "a"), side(0));
    assert_eq!(read(&engine, "b"), side(2));
}

/// 真窗面（`AUTO_UI_ACE_NODE_E2E=1`）：同一包闭包跑真实窗口两次——helper 原值
/// → 窗口状态 `[4, 2]`；helper 改 `+10` → `[20, 10]`。
#[test]
fn p980_closure_window_package_e2e() {
    if std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_closure_window_package_e2e: not scheduled (set AUTO_UI_ACE_NODE_E2E=1 after \
             the prep stage; window single-instance discipline)"
        );
        return;
    }
    ui980::require_prepared();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/ace_node_desktop_window.exe")
        .canonicalize()
        .unwrap();
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — build the example first",
        exe.display()
    );

    let run = |helper: &str, tag: &str| -> serde_json::Value {
        let tmp =
            std::env::temp_dir().join(format!("p980-closure-e2e-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let main_path = tmp.join("app.at");
        let helper_path = tmp.join("steps.at");
        std::fs::write(&main_path, derived_counter()).unwrap();
        std::fs::write(&helper_path, helper).unwrap();
        let ev_path = tmp.join("evidence.json");
        let spec = format!(
            "app::main={};util::steps={}",
            main_path.display(),
            helper_path.display()
        );
        let out = std::process::Command::new(&exe)
            .arg("--package")
            .arg(&spec)
            .arg("--entry")
            .arg("app::main")
            .arg("--evidence")
            .arg(&ev_path)
            .arg("--instances")
            .arg("2")
            .arg("--script")
            .arg("3:click:0:1")
            .arg("--hold-ms")
            .arg("6000")
            .current_dir(&tmp)
            .output()
            .expect("spawn window exe");
        assert!(
            out.status.success(),
            "[{tag}] window exe failed: {} / {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_str(&std::fs::read_to_string(&ev_path).unwrap())
            .unwrap_or_else(|e| panic!("[{tag}] evidence json: {e}"))
    };

    for (helper, want) in [(HELPER, [4, 2]), (HELPER_EDITED, [20, 10])] {
        let tag = if want[0] == 4 { "base" } else { "edited" };
        let ev = run(helper, tag);
        assert_eq!(ev["node"].as_str().unwrap(), "Counter");
        assert!(
            ev["frames_rendered"].as_u64().unwrap_or(0) >= 4,
            "[{tag}] real window rendered"
        );
        let log = ev["dispatch_log"].as_array().unwrap();
        let msgs: Vec<&str> = log.iter().map(|d| d["msg"].as_str().unwrap()).collect();
        assert_eq!(msgs, vec!["Inc"], "[{tag}] one dispatch");
        let states = log.last().unwrap()["states"].as_array().unwrap();
        let count = states
            .iter()
            .find_map(|kv| {
                let pair = kv.as_array()?;
                (pair[0].as_str() == Some("count")).then(|| pair[1].as_str().unwrap().to_string())
            })
            .expect("count row");
        assert_eq!(
            count,
            want[0].to_string(),
            "[{tag}] helper value observed in the real window"
        );
        // 第二实例未被点击：其初值同样来自 helper。
        let sessions = ev["sessions"].as_array().unwrap();
        let this = sessions.last().expect("session");
        let inst1 = this["states"]
            .as_array()
            .unwrap()
            .get(1)
            .expect("instance 1 states");
        let i1_count = inst1
            .as_array()
            .unwrap()
            .iter()
            .find_map(|kv| {
                let pair = kv.as_array()?;
                (pair[0].as_str() == Some("count")).then(|| pair[1].as_str().unwrap().to_string())
            })
            .expect("i1 count");
        assert_eq!(i1_count, want[1].to_string(), "[{tag}] instance 1 initial");
    }
}
