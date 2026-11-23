#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_node_desktop::{current_session, StepOutcome};
use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
use auto_ui_runtime::ui::ase_m0_sessions::{AseApp, AseSessionHost};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn state(host: &AseSessionHost, session: usize, name: &str) -> Option<AceStateValue> {
    host.engine_states(session)?
        .into_iter()
        .find_map(|(field, value)| (field == name).then_some(value))
}

fn visible() -> String {
    current_session(|session| {
        session
            .package_frame(0)
            .map(|frame| format!("{:?}", frame.view))
    })
    .flatten()
    .expect("selected Todo session renders a checked frame")
}

fn step(host: &mut AseSessionHost) {
    match host.step_selected() {
        StepOutcome::DispatchRefused(error) | StepOutcome::LaneError(error) => panic!("{error}"),
        _ => {}
    }
}

fn until(host: &mut AseSessionHost, mut done: impl FnMut(&AseSessionHost) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done(host) {
        assert!(
            Instant::now() < deadline,
            "Todo did not settle: {}",
            host.evidence()
        );
        step(host);
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn action(host: &mut AseSessionHost, message: &str, index: usize, text: Option<&str>) {
    host.package_input(message, index, text.map(str::to_owned))
        .unwrap_or_else(|error| panic!("{error}: {}", host.evidence()));
    step(host);
}

#[test]
fn gallery_todo_keeps_drafts_local_and_reopens_real_persisted_rows() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("examples/ui/013-todo");
    let counter = root.join("../auto-lang/experimental/ase-m0/fixtures/counter-a/main.at");
    let scratch = match std::env::var_os("ASE_TODO_TEST_ROOT") {
        Some(path) => tempfile::tempdir_in(path),
        None => tempfile::tempdir(),
    }
    .unwrap();
    let data = scratch.path().join("data");
    std::fs::create_dir(&data).unwrap();
    let make_host = || {
        AseSessionHost::new(vec![AseApp {
            key: "counter".into(),
            path: counter.to_string_lossy().into_owned(),
        }])
        .with_todo_project(&project, scratch.path().join("code"), Some(&data))
        .unwrap()
    };

    let mut host = make_host();
    let first = host.open_session("todo").expect("open real Gallery 013");
    until(&mut host, |_| visible().contains("Learn Auto Language"));
    assert_eq!(
        state(&host, first, "st_active_count"),
        Some(AceStateValue::Int(3))
    );
    action(&mut host, "SetInput", 0, Some("计划1112 Δ Todo"));
    action(&mut host, "AddTodo", 0, None);
    // Give the real request to the worker, then withdraw the display lane
    // before polling any completion.
    step(&mut host);
    let counter = host.open_session("counter").unwrap();
    host.click(2).unwrap();
    step(&mut host);
    assert_eq!(state(&host, counter, "count"), Some(AceStateValue::Int(1)));
    until(&mut host, |host| {
        state(host, first, "st_active_count") == Some(AceStateValue::Int(4))
    });
    assert_eq!(host.selected_id(), Some(counter));
    assert_eq!(state(&host, counter, "count"), Some(AceStateValue::Int(1)));
    host.select(first).unwrap();
    until(&mut host, |_| visible().contains("计划1112 Δ Todo"));
    assert_eq!(host.todo_inventory().unwrap().durable_writes, 1);
    host.close_session(counter).unwrap();

    action(&mut host, "SetInput", 0, Some("draft one"));
    let second = host.open_session("todo").unwrap();
    until(&mut host, |_| visible().contains("计划1112 Δ Todo"));
    assert_eq!(
        state(&host, second, "st_input"),
        Some(AceStateValue::Str(String::new()))
    );
    action(&mut host, "SetInput", 0, Some("draft two"));
    host.select(first).unwrap();
    assert_eq!(
        state(&host, first, "st_input"),
        Some(AceStateValue::Str("draft one".into()))
    );
    host.close_session(first).unwrap();
    host.select(second).unwrap();
    assert_eq!(
        state(&host, second, "st_input"),
        Some(AceStateValue::Str("draft two".into()))
    );
    assert_eq!(host.todo_inventory().unwrap().leases, 1);
    until(&mut host, |host| {
        state(host, second, "st_todo_pending") == Some(AceStateValue::Bool(false))
    });

    action(&mut host, "StartEdit", 4, None);
    action(&mut host, "EditInputChanged", 0, Some("临时取消"));
    action(&mut host, "CancelEdit", 0, None);
    assert!(visible().contains("计划1112 Δ Todo"));
    assert!(!visible().contains("临时取消"));
    action(&mut host, "StartEdit", 4, None);
    action(&mut host, "EditInputChanged", 0, Some("计划1112 Δ ✓"));
    action(&mut host, "CommitEdit", 0, None);
    until(&mut host, |host| {
        state(host, second, "st_editing_id") == Some(AceStateValue::Int(-1))
            && visible().contains("计划1112 Δ ✓")
    });
    assert_eq!(host.todo_inventory().unwrap().durable_writes, 2);
    action(&mut host, "ToggleTodo", 4, None);
    until(&mut host, |host| {
        state(host, second, "st_active_count") == Some(AceStateValue::Int(3))
    });
    action(&mut host, "FilterCompleted", 0, None);
    assert!(visible().contains("计划1112 Δ ✓"));
    assert!(!visible().contains("Build a TodoMVC app"));
    action(&mut host, "FilterAll", 0, None);
    action(&mut host, "DeleteTodo", 3, None);
    until(&mut host, |_| {
        !visible().contains("Write integration tests")
    });
    action(&mut host, "ClearCompleted", 0, None);
    until(&mut host, |_| !visible().contains("计划1112 Δ ✓"));
    assert_eq!(host.todo_inventory().unwrap().durable_writes, 5);
    host.close_all().unwrap();
    let terminal = host.todo_inventory().unwrap();
    assert_eq!(
        (terminal.leases, terminal.workers, terminal.owners),
        (0, 0, 0)
    );

    let mut reopened = make_host();
    let session = reopened.open_session("todo").unwrap();
    until(&mut reopened, |_| visible().contains("Build a TodoMVC app"));
    let frame = visible();
    assert!(frame.contains("Master the a2r transpiler"));
    assert!(!frame.contains("Learn Auto Language"));
    assert!(!frame.contains("计划1112 Δ"));
    assert_eq!(
        state(&reopened, session, "st_active_count"),
        Some(AceStateValue::Int(2))
    );
    reopened.close_all().unwrap();
}
