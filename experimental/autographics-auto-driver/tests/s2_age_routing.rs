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
    assert!(
        err.contains("source."),
        "AGE named diagnostics are kept: {err}"
    );

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
    assert_eq!(
        (a, b),
        (2, 2),
        "both declaring components receive the directed event"
    );
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

/// R64-10：多轮 Run/Stop 的账面净归零在 driver 层也可达。
#[test]
fn s2_multi_run_stop_balance_is_reachable() {
    const TICK: &str = "node Counter {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let mut s = Session::new(TICK).expect("opens");
    assert_eq!(s.age_run_stop_balance().unwrap(), (1, 0, true));
    s.age_stop().unwrap();
    assert_eq!(s.age_run_stop_balance().unwrap(), (1, 1, false));
    assert_eq!(s.age_run().unwrap(), 2);
    s.age_tick(1).unwrap();
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states[0][0],
        1
    );
    s.age_stop().unwrap();
    assert_eq!(s.age_run_stop_balance().unwrap(), (2, 2, false));
}

/// PLAN-981 IF-981-EVENT-A：payload 投递在宿主层**具名拒**且零效果。
#[test]
fn s2_payload_dispatch_is_named_rejected_with_zero_effect() {
    const SRC: &str = "node M {\n    msg Msg {\n        Add(str)\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let mut s = Session::new(SRC).expect("a payload declaration compiles");
    let before = s.age_observe().expect("observes");
    let before_count = before.components[0].instance_states[0][0];
    match s.age_send("Add") {
        Ok(_) => panic!("a payload dispatch without its value must be rejected"),
        Err(e) => assert!(
            e.contains("event-payload-unsupported"),
            "the rejection is named: {e}"
        ),
    }
    let after = s.age_observe().expect("still observes");
    assert_eq!(
        after.components[0].instance_states[0][0], before_count,
        "a rejected dispatch has zero state effect"
    );
    assert_eq!(s.age_dispatches().unwrap(), 0, "no dispatch was recorded");
}

/// 同一宿主上的 unit 消息投递仍然成功（负控的另一半）。
#[test]
fn s2_unit_dispatch_still_runs_next_to_payload_sources() {
    const SRC: &str = "node M {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let mut s = Session::new(SRC).expect("opens");
    s.age_send("Inc").expect("unit dispatch runs");
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states[0][0],
        1
    );
}

/// PLAN-981（B-LANGUAGE-V）：真实 980/A Counter 源（**canonical 文件**，非手抄）在宿主上
/// 可编译，事件绑定可读取（R67-5：直接读 examples 源，避免字面量漂移）。
#[test]
fn s2_real_a_counter_source_exposes_view_events() {
    const SRC: &str = include_str!("../../../examples/ui/ace-node-counter/src/front/app.at");
    let s = Session::new(SRC).expect("the delivered counter source opens");
    assert_eq!(
        s.age_component_names().unwrap(),
        vec!["Counter".to_string()]
    );
    let events = s.age_view_events(0).unwrap();
    let pairs: Vec<(&str, &str)> = events
        .iter()
        .map(|(k, m, _)| (k.as_str(), m.as_str()))
        .collect();
    assert_eq!(
        pairs,
        vec![("onclick", "Dec"), ("onclick", "Reset"), ("onclick", "Inc")]
    );
    assert!(
        events.iter().all(|(_, _, s)| s.1 > s.0),
        "bindings carry real spans"
    );
}

/// B-SESSION-V：真实 980/A Counter 源 + 两实例 2:1 定向事件、实例隔离、停/重启恢复。
///
/// R67-3：隔离断言必须能因"实例 1 的派发被丢弃"而失败——第一次 `send_to(1, Inc)` 后
/// 立刻观察 `[2, 1]`，结束值保持非零（1），而不是回到 0。
#[test]
fn s2_real_a_counter_two_instance_isolation() {
    use ace_core::source::age_component::ScenarioOp;
    const SRC: &str = include_str!("../../../examples/ui/ace-node-counter/src/front/app.at");
    let mut s = Session::new_age_with(
        SRC,
        2,
        vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Dec".into(),
            },
        ],
    )
    .expect("the delivered counter source opens with two instances");
    let o = s.age_observe().expect("observes");
    assert_eq!(o.components.len(), 1, "one component, two instances");
    assert_eq!(o.components[0].instance_states, vec![vec![2], vec![0]]);
    // 实例 1 再 Inc 一次：立刻能看到 1（若该派发被丢弃，这里会保持 0）。
    s.age_send_to(1, "Inc").unwrap();
    let o = s.age_observe().unwrap();
    assert_eq!(o.components[0].instance_states, vec![vec![2], vec![1]]);
    // 实例 0 不受影响；再给实例 1 一次 Inc，结束值 2。
    s.age_send_to(1, "Inc").unwrap();
    let o = s.age_observe().unwrap();
    assert_eq!(o.components[0].instance_states, vec![vec![2], vec![2]]);
    // Stop → Restart：状态从初值重来（同一逻辑 session id），body 代数递增。
    let (session_id, generation) = s.age_identity().unwrap();
    s.age_stop().unwrap();
    s.age_restart().unwrap();
    assert_eq!(
        s.age_identity().unwrap().0,
        session_id,
        "logical session id is stable"
    );
    assert!(
        s.age_identity().unwrap().1 > generation,
        "restart bumps the generation"
    );
    let o = s.age_observe().unwrap();
    assert_eq!(o.components[0].instance_states, vec![vec![0], vec![0]]);
}
/// R65-8/R65-11：带参 handler 在宿主层派发时具名拒，零状态/零 dispatch。
#[test]
fn s2_param_handler_dispatch_is_rejected_with_zero_effect() {
    const SRC: &str = "node M {\n    msg Msg {\n        Add\n    }\n    on {\n        .Add(t) -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let mut s = Session::new(SRC).expect("a parameterised handler parses");
    let before = s.age_observe().unwrap().components[0].instance_states[0][0];
    match s.age_send("Add") {
        Ok(_) => panic!("dispatching a parameterised handler must be rejected"),
        Err(e) => assert!(
            e.contains("event-params-unsupported"),
            "named rejection: {e}"
        ),
    }
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states[0][0],
        before
    );
    assert_eq!(s.age_dispatches().unwrap(), 0);
}

/// R65-11：实例定向派发到未知消息具名拒，零 dispatch。
#[test]
fn s2_send_to_unknown_message_is_rejected() {
    const SRC: &str = "node M {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let mut s = Session::new(SRC).expect("opens");
    match s.age_send_to(0, "Nope") {
        Ok(_) => panic!("an unknown message dispatch must be rejected"),
        Err(e) => assert!(!e.is_empty(), "named rejection"),
    }
    assert_eq!(s.age_dispatches().unwrap(), 0);
    s.age_send_to(0, "Inc")
        .expect("the declared message still dispatches");
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states[0][0],
        1
    );
}

/// B-SESSION-V[A]：真实 980/A Counter 源（canonical 文件）的两实例轨迹与停/重启恢复。
///
/// 这是 A 子面的**轨迹**证据（含 Reset 归零）；隔离断言在
/// `s2_real_a_counter_two_instance_isolation`。R68-1：源不再手抄，直接读 examples。
#[test]
fn s2_real_a_counter_trajectory_and_restart() {
    use ace_core::source::age_component::ScenarioOp;
    const SRC: &str = include_str!("../../../examples/ui/ace-node-counter/src/front/app.at");
    let mut s = Session::new_age_with(
        SRC,
        2,
        vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Dec".into(),
            },
        ],
    )
    .expect("the delivered counter source opens with two instances");
    let o = s.age_observe().expect("observes");
    assert_eq!(o.components.len(), 1, "one component, two instances");
    assert_eq!(o.components[0].instance_states, vec![vec![2], vec![0]]);
    // Reset 归零轨迹（A 子面唯一覆盖）：实例 1 → 1 → Reset → 0；实例 0 不受影响。
    s.age_send_to(1, "Inc").unwrap();
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states,
        vec![vec![2], vec![1]]
    );
    s.age_send_to(1, "Reset").unwrap();
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states,
        vec![vec![2], vec![0]]
    );
    // Stop → Restart：状态回初值；逻辑 session id 稳定、generation 递增。
    let (session_id, generation) = s.age_identity().unwrap();
    s.age_stop().unwrap();
    s.age_restart().unwrap();
    assert_eq!(
        s.age_identity().unwrap().0,
        session_id,
        "logical session id is stable"
    );
    assert!(
        s.age_identity().unwrap().1 > generation,
        "restart bumps the generation"
    );
    assert_eq!(
        s.age_observe().unwrap().components[0].instance_states,
        vec![vec![0], vec![0]]
    );
}

/// B-PROOF-V[A]（宿主腿）：一次事件 → 更新 → 坏编辑保旧 → 修复恢复 → 关闭/重启。
#[test]
fn s2_proof_a_event_update_close_and_recovery() {
    const UI: &str = "node Ui {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view {\n        text \"ui\" {}\n    }\n}\n";
    let mut s = Session::new(UI).expect("the MIN-A UI node opens");
    assert!(s.is_age());
    s.age_send("Inc").expect("one event");
    let seen = s.age_observe().unwrap();
    assert_eq!(seen.components[0].instance_states, vec![vec![1]]);
    assert_eq!(seen.components[0].instance_texts, vec![vec!["ui".to_string()]]);

    // 坏编辑：具名拒且保旧（状态与文本都不变）。
    let bad = UI.replace("text \"ui\" {}", "text \"ui\" {");
    match s.update_age_body(&bad) {
        Ok(_) => panic!("a malformed body edit must be rejected"),
        Err(e) => assert!(
            e.contains("source."),
            "the host surfaces a named diagnostic, not a generic message: {e}"
        ),
    }
    let seen = s.age_observe().unwrap();
    assert_eq!(seen.components[0].instance_states, vec![vec![1]]);
    assert_eq!(seen.components[0].instance_texts, vec![vec!["ui".to_string()]]);

    // 修复：同一入口恢复，文本随 body 更新，状态保留。
    let fixed = UI.replace("\"ui\"", "\"ui2\"");
    s.update_age_body(&fixed).expect("the fixed body applies");
    let seen = s.age_observe().unwrap();
    assert_eq!(seen.components[0].instance_texts, vec![vec!["ui2".to_string()]]);
    assert_eq!(seen.components[0].instance_states, vec![vec![1]]);

    // 关闭/重启：停止后冻结观察，重启回初值。
    s.age_stop().expect("stop");
    assert!(s.age_is_stopped().unwrap());
    // 停止后观察被冻结：仍返回停前状态（不是空/错误），且停止期间的调度被拒。
    let frozen = s.age_observe().unwrap();
    assert_eq!(frozen.components[0].instance_states, vec![vec![1]]);
    match s.age_send("Inc") {
        Ok(_) => panic!("a stopped session must reject dispatch"),
        Err(e) => assert!(e.contains("source.session.stopped"), "stopped dispatch is named: {e}"),
    }
    s.age_restart().expect("restart");
    let seen = s.age_observe().unwrap();
    assert_eq!(seen.components[0].instance_states, vec![vec![0]]);
}