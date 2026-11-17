//! PLAN-981（B-SESSION-I，S1）：AGE Session 桥接层的正反例。
//!
//! 证明的是"AGE 源经 ACE 受检语义 + AV 执行拿到真实值"，不是窗口/GPU 行为：
//! - 正例：`model int` + `view { text }`，观察到的文本与状态是 AV 执行值；
//! - 反例：坏源具名拒绝（不是 panic、不是静默空表）；
//! - 生命周期：oracle 计数不随出口数倍增（Start=1 / Tick=N / Frame=M）。
use age_ace_session::AgeSession;

const COUNTER: &str = "node Counter {\n    model {\n        count int = 3\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

#[test]
fn s1_age_session_observes_av_values() {
    let s = AgeSession::open(COUNTER, 1).expect("a valid AGE node opens");
    let observed = s.observe().expect("AV observation succeeds");
    assert_eq!(observed.components.len(), 1);
    let c = &observed.components[0];
    assert_eq!(c.name, "Counter");
    assert_eq!(c.instance_states.len(), 1, "one instance observed");
    assert_eq!(c.instance_states[0], vec![3], "the model int comes from AV");
}

#[test]
fn s1_ticks_advance_the_state_through_av() {
    use ace_core::source::age_component::ScenarioOp;
    let src = "node Counter {\n    model {\n        count int = 0\n    }\n    on Life.Tick {\n        count = count + 1\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";
    let ops: Vec<ScenarioOp> = (0..5).map(|_| ScenarioOp::Tick { instance: 0 }).collect();
    let s = AgeSession::open_with(src, 1, ops).expect("a Tick handler compiles");
    let observed = s.observe().expect("AV observation succeeds");
    assert_eq!(
        observed.components[0].instance_states[0],
        vec![5],
        "five ticks must be exactly five increments"
    );
    assert_eq!(
        s.lifecycle(5, 3).expect("oracle").ticks,
        5,
        "the oracle counts the same five ticks"
    );
}

#[test]
fn s1_bad_source_is_named_rejected() {
    let err = match AgeSession::open("node Broken {\n    view {\n        text\n    }\n}\n", 1) {
        Ok(_) => panic!("a broken source must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err.iter().any(|d| d.starts_with("source.")),
        "diagnostics keep their named codes: {err:?}"
    );
}

#[test]
fn s1_lifecycle_counts_do_not_scale_with_exits() {
    let s = AgeSession::open(COUNTER, 1).expect("valid node");
    let one = s.lifecycle(5, 3).expect("lifecycle oracle");
    assert_eq!((one.starts, one.ticks, one.frames), (1, 5, 3));
    let two = AgeSession::open(
        "node A {\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    view {\n        text \"b\" {}\n    }\n}\n",
        1,
    )
    .expect("two nodes open")
    .lifecycle(5, 3)
    .expect("lifecycle oracle");
    assert_eq!((two.starts, two.ticks, two.frames), (1, 5, 3));
    assert_eq!(one, two);
}
