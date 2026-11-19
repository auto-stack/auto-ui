#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_node_desktop::{
    host_session_display, host_session_stats, NodeDesktopBridge, StepOutcome,
};
use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
use auto_ui_runtime::ui::ase_m0_sessions::{AseApp, AseSessionHost};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture(app: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../auto-lang/experimental/ase-m0/fixtures")
        .join(app)
        .join("main.at")
}

fn require_prepared_provider() {
    assert!(std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_some(),
        "prepare the UI980 provider and set ACE_NODE_DESKTOP_PREPARED_RUSTC to the build rustc --version");
}

#[test]
fn checked_clock_event_reaches_text_leaf_and_rejects_bad_payload() {
    let started = Instant::now();
    require_prepared_provider();
    let path = fixture("clock");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut bridge = NodeDesktopBridge::load(&source, path.to_str().unwrap(), 1).unwrap();
    let lane = bridge.open().unwrap();
    assert!(matches!(
        bridge.dispatch_typed(0, "Tick", &[AceStateValue::Int(35_999)]),
        StepOutcome::Dispatched { .. }
    ));
    assert_eq!(host_session_display(lane).unwrap().0, [["09:59:59"]]);
    let state = bridge.states(0);
    let frame = host_session_stats(lane).unwrap().applies;
    let audit = bridge.execution_audit().unwrap();
    assert_eq!(
        (audit.compile_count, audit.links, audit.handler_entries),
        (1, 1, 1)
    );

    for (message, args) in [
        ("Missing", vec![AceStateValue::Int(0)]),
        ("Tick", vec![AceStateValue::Str("00:00:00".into())]),
        ("Tick", vec![]),
    ] {
        assert!(matches!(
            bridge.dispatch_typed(0, message, &args),
            StepOutcome::DispatchRefused(_)
        ));
        assert_eq!(bridge.states(0), state);
        assert_eq!(bridge.execution_audit().unwrap(), audit);
        assert_eq!(host_session_stats(lane).unwrap().applies, frame);
    }
    bridge.close_session().unwrap();
    eprintln!(
        "checked_clock_event_reaches_text_leaf_and_rejects_bad_payload: {:?}",
        started.elapsed()
    );
}

fn host() -> AseSessionHost {
    require_prepared_provider();
    let apps = ["clock", "counter-a"]
        .into_iter()
        .map(|key| AseApp {
            key: key.into(),
            path: fixture(key).to_str().unwrap().into(),
        })
        .collect();
    AseSessionHost::new(apps)
        .with_clock_app("clock".into())
        .unwrap()
}

fn offer(host: &mut AseSessionHost, now: Instant, sample: Result<i32, String>) -> StepOutcome {
    let wake = host.clock_wake().expect("selected clock registration");
    assert!(host.queue_clock_wake(wake, now));
    host.dispatch_clock_wake(now, || sample)
}

fn expect_time(host: &AseSessionHost, id: usize, text: &str) {
    let lane = host.ui_session_of(id).unwrap();
    assert_eq!(host_session_display(lane).unwrap().0, [[text]]);
    assert_eq!(
        host.engine_states(id).unwrap(),
        [("time".to_owned(), AceStateValue::Str(text.to_owned())),]
    );
}

#[test]
fn visibility_recalibrates_and_preserves_counter_routing() {
    let started = Instant::now();
    let mut host = host();
    let clock = host.open_session("clock").unwrap();
    let first = host.ui_session_of(clock).unwrap();
    let base = Instant::now();
    assert_eq!(host_session_stats(first).unwrap().applies, 0);
    assert!(matches!(
        offer(&mut host, base, Ok(35_999)),
        StepOutcome::Dispatched { .. }
    ));
    expect_time(&host, clock, "09:59:59");
    let first_frame_elapsed = started.elapsed();
    assert!(!host.queue_clock_wake(
        host.clock_wake().unwrap(),
        base + Duration::from_millis(999)
    ));
    offer(&mut host, base + Duration::from_secs(1), Ok(36_000));
    expect_time(&host, clock, "10:00:00");

    let counter = host.open_session("counter-a").unwrap();
    let hidden = host.execution_audit_of(clock).unwrap();
    let hidden_applies = host_session_stats(first).unwrap().applies;
    for _ in 0..3 {
        assert!(host.clock_wake().is_none());
        host.click(2).unwrap();
        assert!(matches!(
            host.step_selected(),
            StepOutcome::Dispatched { .. }
        ));
    }
    assert_eq!(host.execution_audit_of(clock).unwrap(), hidden);
    assert_eq!(host_session_stats(first).unwrap().applies, hidden_applies);
    assert_eq!(
        host.engine_states(counter).unwrap()[0].1,
        AceStateValue::Int(3)
    );
    assert_eq!(host.execution_audit_of(counter).unwrap().handler_entries, 3);

    let resume_started = Instant::now();
    host.select(clock).unwrap();
    let restored = host.ui_session_of(clock).unwrap();
    assert_ne!(restored, first);
    assert_eq!(host_session_stats(restored).unwrap().applies, 0);
    assert_eq!(host_session_display(restored).unwrap().0, [[""]]);
    let period = Duration::from_secs(1);
    let resumed = Instant::now() + period;
    offer(&mut host, resumed, Ok(40_800));
    expect_time(&host, clock, "11:20:00");
    let resumed_frame_elapsed = resume_started.elapsed();
    let audit = host.execution_audit_of(clock).unwrap();
    assert_eq!(
        (audit.compile_count, audit.links),
        (hidden.compile_count, hidden.links)
    );
    assert_eq!(audit.handler_entries, hidden.handler_entries + 1);
    offer(&mut host, resumed + period, Ok(39_600));
    expect_time(&host, clock, "11:00:00");
    offer(&mut host, resumed + 2 * period, Ok(86_399));
    expect_time(&host, clock, "23:59:59");
    offer(&mut host, resumed + 3 * period, Ok(0));
    expect_time(&host, clock, "00:00:00");
    host.close_all().unwrap();
    assert_eq!(
        host.execution_audit_of(clock)
            .unwrap()
            .retired_handler_entries,
        6
    );
    eprintln!(
        "visibility_recalibrates_and_preserves_counter_routing: {:?} (first frame {first_frame_elapsed:?}, resumed frame {resumed_frame_elapsed:?})",
        started.elapsed()
    );
}

#[test]
fn pending_work_coalesces_and_cannot_cross_visibility_or_session() {
    let started = Instant::now();
    let mut host = host();
    let first = host.open_session("clock").unwrap();
    let old_visibility = host.clock_wake().unwrap();
    host.select(first).unwrap();
    assert_eq!(
        host.clock_wake(),
        Some(old_visibility),
        "same-tab selection keeps the deadline"
    );
    let overdue = Instant::now() + Duration::from_secs(30);
    assert!(host.queue_clock_wake(old_visibility, overdue));
    for _ in 0..20 {
        assert!(!host.queue_clock_wake(old_visibility, overdue));
        assert_eq!(host.clock_queue_depth(), 1);
    }

    let second = host.open_session("clock").unwrap();
    assert_eq!(
        host.clock_queue_depth(),
        0,
        "hiding cancels the queued callback"
    );
    assert!(!host.queue_clock_wake(old_visibility, overdue));
    let other_session = host.clock_wake().unwrap();
    host.select(first).unwrap();
    assert_ne!(host.clock_wake(), Some(old_visibility));
    // Let the actual dispatch boundary decide whether an old visible callback can affect AV.
    let _ = host.queue_clock_wake(old_visibility, overdue);
    let _ = host.dispatch_clock_wake(overdue, || Ok(73));
    assert_eq!(
        host.execution_audit_of(first).unwrap().handler_entries,
        0,
        "previous visibility entered the current session"
    );
    assert!(!host.queue_clock_wake(other_session, overdue));
    assert_eq!(
        host.dispatch_clock_wake(overdue, || panic!("stale work sampled the wall clock")),
        StepOutcome::Idle
    );
    assert_eq!(host.execution_audit_of(first).unwrap().handler_entries, 0);
    assert_eq!(host.execution_audit_of(second).unwrap().handler_entries, 0);
    assert_eq!(
        host_session_stats(host.ui_session_of(first).unwrap())
            .unwrap()
            .applies,
        0
    );

    let current = host.clock_wake().unwrap();
    assert!(host.queue_clock_wake(current, overdue));
    assert!(!host.queue_clock_wake(current, overdue));
    host.dispatch_clock_wake(overdue, || Ok(42));
    expect_time(&host, first, "00:00:42");
    assert_eq!(
        host.execution_audit_of(first).unwrap().handler_entries,
        1,
        "missed periods cannot replay"
    );
    assert!(
        !host.queue_clock_wake(current, overdue),
        "a consumed deadline cannot be enqueued twice"
    );

    let _ = host.queue_clock_wake(host.clock_wake().unwrap(), overdue);
    let _ = host.dispatch_clock_wake(overdue, || Ok(42));
    expect_time(&host, first, "00:00:42");
    assert_eq!(
        host.execution_audit_of(first).unwrap().handler_entries,
        1,
        "one wall-clock value can conceal a catch-up dispatch"
    );

    let before_close = host.clock_wake().unwrap();
    let next = overdue + Duration::from_secs(2);
    assert!(host.queue_clock_wake(before_close, next));
    host.close_session(first).unwrap();
    assert_eq!(host.clock_queue_depth(), 0);
    assert!(host.engine_states(first).is_none());
    let reopened = host.open_session("clock").unwrap();
    assert_ne!(reopened, first);
    assert!(!host.queue_clock_wake(before_close, next));
    assert_eq!(
        host.dispatch_clock_wake(next, || panic!("retired callback sampled")),
        StepOutcome::Idle
    );
    assert_eq!(
        host.execution_audit_of(reopened).unwrap().handler_entries,
        0
    );
    offer(&mut host, next, Ok(42));
    expect_time(&host, reopened, "00:00:42");
    assert_eq!(
        host.execution_audit_of(reopened).unwrap().handler_entries,
        1
    );
    host.close_all().unwrap();
    eprintln!(
        "pending_work_coalesces_and_cannot_cross_visibility_or_session: {:?}",
        started.elapsed()
    );
}

#[test]
fn failed_samples_recover_and_twenty_rounds_release_the_state_root() {
    let started = Instant::now();
    let mut host = host();
    for _ in 0..20 {
        let clock = host.open_session("clock").unwrap();
        let base = Instant::now();
        let one = Duration::from_secs(1);
        assert!(matches!(
            offer(&mut host, base, Err("wall clock is unavailable".into())),
            StepOutcome::DispatchRefused(_)
        ));
        let lane = host.ui_session_of(clock).unwrap();
        assert_eq!(host_session_stats(lane).unwrap().applies, 0);
        assert_eq!(host.execution_audit_of(clock).unwrap().handler_entries, 0);
        offer(&mut host, base + one, Ok(1));
        expect_time(&host, clock, "00:00:01");
        let applies = host_session_stats(lane).unwrap().applies;
        assert!(matches!(
            offer(&mut host, base + 2 * one, Ok(86_400)),
            StepOutcome::DispatchRefused(_)
        ));
        assert_eq!(host_session_stats(lane).unwrap().applies, applies);
        expect_time(&host, clock, "00:00:01");

        let counter = host.open_session("counter-a").unwrap();
        assert!(host.clock_wake().is_none());
        host.select(clock).unwrap();
        let next = host.ui_session_of(clock).unwrap();
        assert_eq!(host_session_stats(next).unwrap().applies, 0);
        offer(&mut host, base + 3 * one, Ok(1));
        expect_time(&host, clock, "00:00:01");
        host.close_all().unwrap();
        assert!(host_session_stats(lane).unwrap().closed);
        assert!(host_session_stats(next).unwrap().closed);
        assert!(host.live_ids().is_empty());
        assert!(host.engine_states(clock).is_none());
        assert!(host.engine_states(counter).is_none());
        assert!(host.clock_wake().is_none());
        assert_eq!(host.clock_queue_depth(), 0);
        let audit = host.execution_audit_of(clock).unwrap();
        assert_eq!(
            (
                audit.compile_count,
                audit.links,
                audit.retired_handler_entries
            ),
            (1, 1, 2)
        );
        assert_eq!((audit.init_runs, audit.released_roots), (1, 1));
        assert!(audit.incremental_invariant_holds());
    }
    eprintln!(
        "failed_samples_recover_and_twenty_rounds_release_the_state_root: {:?}",
        started.elapsed()
    );
}
