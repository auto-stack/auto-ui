//! PLAN-981（B-SESSION-I，S2）：driver `Session` 的 AGE 路由与同 PID 更新。
//!
//! 这组用例只覆盖 **typed 宿主适配**这一层：AGE 源经共享 compile/verify/AV 跑，
//! 非 AGE 源保持旧路径；同 PID body 更新成功才递增代数、坏编辑保留旧代。
use autographics_auto_driver::Session;

const GOOD_V1: &str = "node Counter {\n    model { count int = 0 }\n    on Life.Tick { count = count + 1 }\n    view {\n        text \"v1\" {}\n    }\n}\n";

const GOOD_V2: &str = "node Counter {\n    model { count int = 0 }\n    on Life.Tick { count = count + 1 }\n    view {\n        text \"v2\" {}\n    }\n}\n";

const BAD: &str = "node Counter {\n    view {\n        text nosuch_fn() {}\n    }\n}\n";

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
    let observed = s.age_observe().expect("AV observation");
    assert_eq!(observed.components.len(), 1);

    let scalar = Session::new("fn main() int { return 1 }").expect("a scalar source opens");
    assert!(!scalar.is_age(), "non-AGE source keeps the legacy path");
}

/// 同 PID body 更新：成功递增 generation，pid 不变；坏编辑拒绝且旧代保留。
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

    let err = s.update_age_body(BAD).expect_err("a bad edit is rejected");
    assert!(err.contains("source."), "diagnostics keep their codes: {err}");
    let (pid2, gen2) = s.age_identity().unwrap();
    assert_eq!(pid2, pid0);
    assert_eq!(gen2, 1, "a rejected edit must not bump the generation");
    let observed = s.age_observe().expect("the old generation stays observable");
    assert_eq!(observed.components.len(), 1);
}

/// 非 AGE Session 上的 AGE API 是具名拒绝，不是 panic/静默。
#[test]
fn s2_age_api_on_scalar_session_is_named_rejected() {
    let s = Session::new("fn main() int { return 1 }").expect("opens");
    assert!(s.age_lifecycle(1, 1).is_err());
    assert!(s.age_observe().is_err());
    assert!(s.age_identity().is_err());
}