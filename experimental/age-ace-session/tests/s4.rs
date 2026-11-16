//! PLAN-981（B-SESSION-I，R63-1）：SESSION I 的调度/回调/资源/停止恢复/布局义务。
use ace_core::source::age_component::ScenarioOp;
use ace_core::source::age_package::{AgeViewExit, AgeViewRequest};
use age_ace_session::AgeSession;

/// Tick 场景源：`on Tick` 是 AV 调度的定向事件。
const TICK: &str = "node Counter {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"t\" {}\n    }\n}\n";

/// 消息源：`on Inc` 由 `send(\"Inc\")` 触发。
const INC: &str = "node Counter {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view {\n        text \"t\" {}\n    }\n}\n";

fn state(s: &AgeSession) -> i32 {
    s.observe().unwrap().components[0].instance_states[0][0]
}

/// tick 是**增量**调度：0→5→6，且不换 Session 身份。
#[test]
fn s4_tick_is_incremental() {
    let mut s = AgeSession::open(TICK, 1).expect("opens");
    assert_eq!(state(&s), 0);
    s.tick(5).expect("tick");
    assert_eq!(state(&s), 5);
    s.tick(1).expect("tick");
    assert_eq!(state(&s), 6);
}

/// 投递作者消息触发 `on <Msg>` 回调（同一 AV 执行链）。
#[test]
fn s4_send_runs_the_message_handler() {
    let mut s = AgeSession::open(INC, 1).expect("opens");
    s.send("Inc").expect("send");
    assert_eq!(state(&s), 1);
    assert_eq!(
        s.observe().unwrap().components[0].handler_msgs,
        vec!["Inc".to_string()]
    );
}

/// 资源行进 Session 事实（名字/端口/模块），不是静默丢弃。
#[test]
fn s4_resources_are_exposed() {
    let src = "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n\nnode M {\n    view {\n        text \"t\" {}\n    }\n}\n";
    let s = AgeSession::open(src, 0).expect("opens with a module resource");
    let r = s.resources();
    assert_eq!(r.len(), 1, "{r:?}");
    assert_eq!(r[0].0, "red");
    assert_eq!(r[0].1, "Material2D");
}

/// 布局请求按已声明出口回答；不兼容布局具名拒（不是空画面）。
#[test]
fn s4_layout_request_is_checked_against_declared_exits() {
    let ui = AgeViewRequest::Ui {
        width: 800,
        height: 600,
        widget_host: false,
    };
    let s = AgeSession::open(INC, 0).expect("opens");
    assert_eq!(s.request_layout(ui).unwrap(), AgeViewExit::View);

    let scene = "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n        }\n    }\n}\n";
    let s2 = AgeSession::open(scene, 0).expect("opens 2d");
    let err = s2
        .request_layout(ui)
        .expect_err("a UI request cannot be served by a 2D scene exit");
    assert_eq!(err.code, "source.age.exit-unavailable");
}

/// 停止/恢复：停止期间冻结观察值并拒绝调度；恢复后 Start 重新为 1、PID 不变。
#[test]
fn s4_stop_freezes_and_restart_resumes_same_pid() {
    let mut s = AgeSession::open(TICK, 1).expect("opens");
    let pid = s.pid();
    s.tick(3).expect("tick");
    assert_eq!(state(&s), 3);
    s.stop().expect("stop");
    assert!(s.is_stopped());
    assert_eq!(state(&s), 3, "stop freezes the last observation");
    assert!(s.tick(1).is_err(), "scheduling while stopped is rejected");
    let body_gen = s.restart().expect("restart");
    assert_eq!(body_gen, 1, "restart bumps the body generation");
    assert_eq!(s.generation(), 1, "dispatch count is tracked separately");
    assert!(!s.is_stopped());
    assert_eq!(s.pid(), pid, "restart keeps the logical identity");
    assert_eq!(state(&s), 0, "a restart re-runs Start from zero");
}

/// 场景 ops 与增量调度可以叠加（open_with 的 5 Tick 后再 tick 1 = 6）。
#[test]
fn s4_open_time_ops_and_later_ticks_compose() {
    let mut s = AgeSession::open_with(TICK, 1, vec![ScenarioOp::Tick { instance: 0 }; 5])
        .expect("opens with 5 ticks");
    assert_eq!(state(&s), 5);
    s.tick(1).expect("tick");
    assert_eq!(state(&s), 6);
}

/// R63-11/R63-8：多出口 Session 暴露全部出口；生命周期计数用**出口数**回答，
/// 但仍不乘出口数（Tick/Frame 保持调度给定值）。
#[test]
fn s4_exits_and_lifecycle_oracle_use_the_declared_exits() {
    let src = "node M {\n    model { count int = 0 }\n    view {\n        text \"u\" {}\n    }\n    view-portrait {\n        text \"p\" {}\n    }\n}\n";
    let s = AgeSession::open(src, 0).expect("opens with two exits");
    assert_eq!(
        s.exits(),
        vec![AgeViewExit::View, AgeViewExit::ViewPortrait],
        "all declared exits are exposed"
    );
    let b = s.lifecycle(5, 2).expect("lifecycle oracle");
    assert_eq!(
        (b.starts, b.ticks, b.frames),
        (1, 5, 2),
        "exits do not scale counts"
    );
}

/// R64-1：被拒的调度必须**原子**——坏消息不能毒化后续 tick/send。
#[test]
fn s4_rejected_dispatch_is_atomic() {
    let mut s = AgeSession::open(TICK, 1).expect("opens");
    let gen = s.generation();
    assert!(s.send("Nope").is_err(), "an undeclared message is rejected");
    assert_eq!(
        s.generation(),
        gen,
        "a rejected dispatch does not bump the generation"
    );
    s.tick(1).expect("a later valid dispatch still works");
    assert_eq!(s.observe().unwrap().components[0].instance_states[0][0], 1);
    assert_eq!(s.generation(), gen + 1);
}

/// R64-3：多组件宿主面——组件选择器/逐组件观察与布局可达。场景 op 目前是
/// **实例级**（不是组件级）：同一次 Tick 会推进所有声明了该 handler 的组件，
/// 所以"两个组件各自 2:1 收事件"仍要 ACE 场景扩展（见 D-981-SESSION-MULTI ①）。
#[test]
fn s4_multi_component_surface_and_layout() {
    let src = "node A {\n    model { a int = 0 }\n    on Tick { a = a + 1 }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    model { b int = 0 }\n    on Tick { b = b + 1 }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let mut s = AgeSession::open(src, 1).expect("opens two components");
    assert_eq!(s.component_names(), vec!["A".to_string(), "B".to_string()]);
    assert_eq!(s.component_count(), 2);
    assert_eq!(s.exits_of(0).unwrap(), vec![AgeViewExit::View]);
    assert_eq!(s.exits_of(1).unwrap(), vec![AgeViewExit::View]);

    let a = |s: &AgeSession| s.observe_component(0).unwrap().instance_states[0][0];
    let b = |s: &AgeSession| s.observe_component(1).unwrap().instance_states[0][0];
    assert_eq!((a(&s), b(&s)), (0, 0));
    s.tick_instance(0, 2).expect("two Ticks");
    assert_eq!(
        (a(&s), b(&s)),
        (2, 2),
        "the directed event reaches every declaring component"
    );
    // 逐组件布局与越界具名拒。
    let ui = AgeViewRequest::Ui {
        width: 800,
        height: 600,
        widget_host: false,
    };
    assert_eq!(s.request_layout_for(1, ui).unwrap(), AgeViewExit::View);
    assert!(s.observe_component(9).is_err());
    assert!(s.exits_of(9).is_err());
}
