//! PLAN-981（B-SESSION-I，S2）：driver `Session` 的 AGE 路由与同 PID 更新。
//!
//! 这组用例只覆盖 **typed 宿主适配**这一层：AGE 源经共享 compile/verify/AV 跑，
//! 非 AGE 源保持旧路径；同 PID body 更新成功才递增代数、坏编辑保留旧代。断言
//! 读**真实 AV 观察值**（文本/状态），不是"没有 panic"——更新若没替换受检结果，
//! 这里必须变红。
use autographics_auto_driver::Session;

const GOOD_V1: &str = "node Counter {\n    model { count int = 0 }\n    on Life.Tick { count = count + 1 }\n    view {\n        text \"v1\" {}\n    }\n}\n";

const GOOD_V2: &str = "node Counter {\n    model { count int = 0 }\n    on Life.Tick { count = count + 1 }\n    view {\n        text \"v2\" {}\n    }\n}\n";

const BAD: &str = "node Counter {\n    view {\n        text nosuch_fn() {}\n    }\n}\n";

fn texts(s: &Session) -> Vec<String> {
    s.age_observe().expect("AV observation").components[0].instance_texts[0].clone()
}

/// AGE 源走 typed 宿主；普通 scalar 源保持旧路径（互斥）。
#[test]
fn s2_age_source_routes_to_the_typed_host() {
    let s = Session::new(GOOD_V1).expect("an AGE source opens");
    assert!(s.is_age(), "AGE source must route to the typed host");
    assert_eq!(
        s.age_exit().unwrap(),
        ace_core::source::age_package::AgeViewExit::View
    );
    // 5 次 Tick、1 帧：Start=1、Tick=5、Frame=1（不随出口倍增）。
    let b = s.age_lifecycle(5, 1).expect("lifecycle oracle");
    assert_eq!((b.starts, b.ticks, b.frames), (1, 5, 1));
    assert_eq!(
        texts(&s),
        vec!["v1".to_string()],
        "real AV values, not a shape check"
    );

    let scalar = Session::new("fn main() int { return 1 }").expect("a scalar source opens");
    assert!(!scalar.is_age(), "non-AGE source keeps the legacy path");
}

/// 同 PID body 更新：成功递增 generation，pid 不变，**观察值真的变化**；
/// 坏编辑拒绝且旧代旧值保留。
#[test]
fn s2_same_pid_update_keeps_identity_and_old_generation() {
    let mut s = Session::new(GOOD_V1).expect("opens");
    let (pid0, gen0) = s.age_identity().unwrap();
    assert_eq!(gen0, 0);
    let gen1 = s.update_age_body(GOOD_V2).expect("a valid update applies");
    assert_eq!(gen1, 1);
    let (pid1, gen1b) = s.age_identity().unwrap();
    assert_eq!(pid1, pid0, "same-PID update keeps the identity");
    assert_eq!(gen1b, 1);
    assert_eq!(
        texts(&s),
        vec!["v2".to_string()],
        "the update replaced the checked body"
    );

    let err = s.update_age_body(BAD).expect_err("a bad edit is rejected");
    assert!(
        err.contains("source."),
        "diagnostics keep their codes: {err}"
    );
    let (pid2, gen2) = s.age_identity().unwrap();
    assert_eq!(pid2, pid0);
    assert_eq!(gen2, 1, "a rejected edit must not bump the generation");
    assert_eq!(
        texts(&s),
        vec!["v2".to_string()],
        "the old generation stays observable"
    );
}

/// R63-3：更新必须保留打开时的场景（5 次 Tick 不能变成 0 次）。
#[test]
fn s2_update_preserves_the_open_time_scenario() {
    let ops = (0..5)
        .map(|_| ace_core::source::age_component::ScenarioOp::Tick { instance: 0 })
        .collect();
    let mut s = Session::new_age_with(GOOD_V1, 1, ops).expect("opens with a 5-Tick scenario");
    let state = s.age_observe().expect("observe").components[0].instance_states[0][0];
    assert_eq!(state, 5, "five Ticks ran before the update");
    s.update_age_body(GOOD_V2).expect("update applies");
    let state_after = s.age_observe().expect("observe").components[0].instance_states[0][0];
    assert_eq!(state_after, 5, "the update keeps the open-time scenario");
    assert_eq!(texts(&s), vec!["v2".to_string()]);
}

/// R63-4：AGE 会话上调用 scalar VM API 是具名拒绝，且**不关闭会话**。
#[test]
fn s2_scalar_api_on_age_session_is_named_rejected_and_keeps_it_open() {
    let mut s = Session::new(GOOD_V1).expect("opens");
    let err = s
        .start("main", &[], autographics_auto_driver::ResultKind::Unit)
        .expect_err("an AGE session has no scalar main");
    assert!(err.contains("AGE session"), "{err}");
    assert!(!s.closed(), "the session must stay open");
    assert_eq!(texts(&s), vec!["v1".to_string()]);
}

/// 非 AGE Session 上的 AGE API 是具名拒绝，不是 panic/静默。
#[test]
fn s2_age_api_on_scalar_session_is_named_rejected() {
    let s = Session::new("fn main() int { return 1 }").expect("opens");
    assert!(s.age_lifecycle(1, 1).is_err());
    assert!(s.age_observe().is_err());
    assert!(s.age_identity().is_err());
}

/// R63-1：driver 层也暴露调度/回调/布局/停止恢复，且行为与 bridge 一致。
#[test]
fn s2_session_i_obligations_are_reachable_from_the_driver() {
    const TICK: &str = "node Counter {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let mut s = Session::new(TICK).expect("opens");
    let pid = s.age_identity().unwrap().0;
    let st = |s: &Session| s.age_observe().unwrap().components[0].instance_states[0][0];
    assert_eq!(st(&s), 0);
    s.age_tick(4).expect("tick");
    assert_eq!(st(&s), 4);
    s.age_send("Tick").expect("send Tick");
    assert_eq!(st(&s), 5);

    let err = s
        .age_request_layout(ace_core::source::age_package::AgeViewRequest::Scene2d)
        .expect_err("a UI-only node cannot serve a 2D scene request");
    assert!(err.contains("source.age.exit-unavailable"), "{err}");

    s.age_stop().expect("stop");
    assert!(s.age_is_stopped().unwrap());
    assert_eq!(st(&s), 5, "stop freezes the observed value");
    s.age_restart().expect("restart");
    assert!(!s.age_is_stopped().unwrap());
    assert_eq!(
        s.age_identity().unwrap().0,
        pid,
        "restart keeps the identity"
    );
    assert_eq!(st(&s), 0, "restart re-runs Start");
}

/// R64-2：坏 AGE 源保留共享 verify 的具名诊断；注释里含 `node` 的旧源不受影响。
#[test]
fn s2_age_diagnostics_and_legacy_sources_coexist() {
    let bad = "node M {\n    view {\n        text nosuch_fn() {}\n    }\n}\n";
    let err = match Session::new(bad) {
        Ok(_) => panic!("a bad AGE source must be rejected"),
        Err(e) => e,
    };
    assert!(err.contains("source."), "AGE named diagnostics are kept: {err}");

    let legacy = "// node here in a comment\nvar x int = 0\nfn main() int { return 1 }\n";
    let s = Session::new(legacy).expect("a legacy source with `node` in a comment still opens");
    assert!(!s.is_age(), "comment text must not route to the AGE host");
}

/// R64-3：driver 层也暴露多组件选择器/逐组件观察与布局。
#[test]
fn s2_multi_component_surface_is_reachable_from_the_driver() {
    let src = "node A {\n    model { a int = 0 }\n    on Tick { a = a + 1 }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    model { b int = 0 }\n    on Tick { b = b + 1 }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let mut s = Session::new(src).expect("opens two components");
    assert_eq!(s.age_component_count().unwrap(), 2);
    assert_eq!(
        s.age_component_names().unwrap(),
        vec!["A".to_string(), "B".to_string()]
    );
    s.age_tick_instance(0, 2).expect("two Ticks");
    let a = s.age_observe_component(0).unwrap().instance_states[0][0];
    let b = s.age_observe_component(1).unwrap().instance_states[0][0];
    assert_eq!((a, b), (2, 2), "both declaring components receive the directed event");
    let ui = ace_core::source::age_package::AgeViewRequest::Ui {
        width: 800,
        height: 600,
        widget_host: false,
    };
    assert_eq!(
        s.age_request_layout_for(1, ui).unwrap(),
        ace_core::source::age_package::AgeViewExit::View
    );
    assert!(s.age_observe_component(9).is_err());
}