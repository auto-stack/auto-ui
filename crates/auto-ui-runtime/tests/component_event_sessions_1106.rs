//! PLAN-1106 T-06：UI 消费链的**执行层反例与负控**（V-06..08）。
//!
//! oracle 位置（AC-06）：`AceStateEngine::execution_audit()` 在真实生产者/
//! 执行器调用边界计量（`handler_entries` 只在真实派发成功后增长；
//! `replay_compiles`/`rebuild_compiles` 只在负控车道真实调用旧 producer/
//! 重编 producer 时增长）。每个负控：**同一 owner/同一引擎/同一计数器**上
//! 强制错误机制（外层数字与 dispatch_log 保持一致），断言执行层证据必红
//! （检测成立），恢复正常道后绿——不是另起一个桥用另一套计数器自证。

use auto_ui_runtime::ui::ace_state_bridge::{
    AceAlias, AceStateEngine, AceStateValue, DispatchOutcome,
};

/// M0 主例形 Counter。
const COUNTER_SRC: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Reset {\n        count = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

fn engine() -> AceStateEngine {
    AceStateEngine::new(COUNTER_SRC, "counter.1106", AceAlias::Accept).expect("compile")
}

fn count_of(e: &AceStateEngine, id: &str) -> i32 {
    match e.read_state(id, "count") {
        Some(AceStateValue::Int(v)) => v,
        other => panic!("count must be Int, got {other:?}"),
    }
}

/// V-06（同 owner 隐蔽重放负控）：同一引擎上强制旧 refresh 路径（真实
/// `observe_component_scenario_states` producer 调用）——外层 id/dispatch_log
/// 与最终数字保持一致，但执行层证据必红：`replay_compiles` 增长且交付事件
/// 没有 handler 入口（增量不变量破裂）。恢复正常道后再派发，绿。
#[test]
fn v06_forced_replay_lane_caught_by_execution_audit() {
    let mut e = engine();
    e.mount("a", Vec::new()).unwrap();
    // 正常道先证：3 事件 = 3 入口，不变量绿。
    for _ in 0..3 {
        e.dispatch("a", "Inc").unwrap();
    }
    let audit = e.execution_audit();
    assert_eq!(audit.handler_entries, 3);
    assert_eq!(audit.events_delivered, 3);
    assert!(audit.incremental_invariant_holds(), "normal lane green");
    assert_eq!(audit.replay_compiles, 0);
    assert_eq!(count_of(&e, "a"), 3);

    // 强制旧重放道（同 owner、同计数器）：外层数字继续正确……
    e.debug_force_replay_lane(true);
    e.dispatch("a", "Inc").unwrap();
    assert_eq!(count_of(&e, "a"), 4, "outer numbers stay consistent");
    let audit = e.execution_audit();
    // ……但执行层证据红：真实旧 producer 被调用 + 该事件没有 handler 入口。
    assert!(audit.replay_compiles >= 1, "replay producer really called");
    assert!(
        !audit.incremental_invariant_holds(),
        "hidden replay breaks the incremental invariant (delivered={}, handlers={})",
        audit.events_delivered,
        audit.handler_entries
    );

    // 恢复增量道：绿（后续事件重新一一对应）。会话状态根未被强制道
    // 触碰（重放道旁路会话——这本身就是两道分账的证据），恢复后从会话
    // 的 3 继续：+1 = 4，每事件恰一个新入口。
    e.debug_force_replay_lane(false);
    let before = e.execution_audit();
    e.dispatch("a", "Inc").unwrap();
    let after = e.execution_audit();
    assert_eq!(after.handler_entries, before.handler_entries + 1);
    assert_eq!(after.replay_compiles, before.replay_compiles);
    assert!(after.incremental_invariant_holds(), "invariant restored");
    assert_eq!(count_of(&e, "a"), 4);
}

/// V-07（重建/一条场景负控）：同一引擎上强制每事件重编重建 + seed 补数
/// ——最终数字一致，但真实编译/链接每事件增长（prepare/init 证据必红），
/// 且状态根身份每事件更换。
#[test]
fn v07_forced_rebuild_lane_caught_by_execution_audit() {
    let mut e = engine();
    e.mount("a", Vec::new()).unwrap();
    e.dispatch("a", "Inc").unwrap();
    e.dispatch("a", "Inc").unwrap();
    let audit = e.execution_audit();
    assert_eq!(audit.compile_count, 1, "single compile on the normal lane");
    assert_eq!(audit.links, 1);
    assert_eq!(count_of(&e, "a"), 2);

    // 强制重建道：数字照常推进……
    e.debug_force_rebuild_lane(true);
    e.dispatch("a", "Inc").unwrap();
    assert_eq!(count_of(&e, "a"), 3, "outer numbers stay consistent");
    let audit = e.execution_audit();
    // ……但真实编译/链接增长、不变量破裂（重建道不产生 handler 入口）。
    assert!(audit.rebuild_compiles >= 1, "recompile really happened");
    assert!(audit.links >= 2, "state root identity replaced per event");
    assert!(!audit.incremental_invariant_holds());

    // 显式 close/reopen 正控不被误杀：重开 = 新代次、init 一次、归 0。
    e.debug_force_rebuild_lane(false);
    e.unmount("a");
    e.mount("a", Vec::new()).unwrap();
    assert_eq!(count_of(&e, "a"), 0, "explicit reopen resets to seeds");
    let audit = e.execution_audit();
    assert_eq!(audit.compile_count, 1, "reopen does not recompile");
}

/// V-08（getter/UI 重复执行负控）：使文本刷新意外调用业务 handler——静态
/// 画面通过不能掩盖：handler 次数红（读取不再与交付事件对账）。UI 差量与
/// handler 执行分账（正常道读取零派发）。
#[test]
fn v08_forced_getter_dispatch_caught_by_execution_audit() {
    let mut e = engine();
    e.mount("a", Vec::new()).unwrap();
    e.dispatch("a", "Inc").unwrap();
    let audit = e.execution_audit();
    assert_eq!(audit.handler_entries, 1);

    // 正常道：读取/文本刷新零业务派发。
    let texts = e.read_texts("a").unwrap();
    assert!(!texts.is_empty());
    let audit = e.execution_audit();
    assert_eq!(audit.handler_entries, 1, "getter stays pure on normal lane");

    // 强制 getter 派发道：一次读取即一个真实 handler 入口——分账破裂必红。
    e.debug_force_getter_dispatch_lane(true);
    let _ = e.read_texts("a").unwrap();
    let audit = e.execution_audit();
    assert_eq!(
        audit.handler_entries,
        2,
        "getter-triggered business dispatch is visible in handler entries"
    );
    assert!(
        !audit.incremental_invariant_holds(),
        "delivered=1 but handlers=2 — UI reads are no longer free"
    );
    // 数字被 getter 副作用推进（静态画面背后业务重跑的可观察后果）。
    assert_eq!(count_of(&e, "a"), 2);

    // 恢复：读取重新零派发。
    e.debug_force_getter_dispatch_lane(false);
    let before = e.execution_audit();
    let _ = e.read_texts("a").unwrap();
    assert_eq!(e.execution_audit().handler_entries, before.handler_entries);
}

/// V-02（UI 桥面）：每事件恰一次入口、读取/切换零派发、双实例隔离、
/// mount 即 init（seed 生效）——桥级执行层 oracle 的正向面。
#[test]
fn v02_bridge_level_incremental_evidence() {
    let mut e = engine();
    e.mount("a", vec![("count".into(), auto_ui_runtime::ui::ace_state_bridge::AceInitialValue::Int(10))])
        .unwrap();
    e.mount("b", Vec::new()).unwrap();
    for _ in 0..3 {
        assert!(matches!(
            e.dispatch("a", "Inc").unwrap(),
            DispatchOutcome::Delivered(_)
        ));
    }
    for _ in 0..7 {
        e.dispatch("b", "Inc").unwrap();
    }
    assert_eq!(count_of(&e, "a"), 13, "seed 10 + 3");
    assert_eq!(count_of(&e, "b"), 7, "seedless 7");
    let audit = e.execution_audit();
    assert_eq!(audit.handler_entries, 10, "A3 + B7 exactly");
    assert_eq!(audit.events_delivered, 10);
    assert_eq!(audit.compile_count, 1);
    assert_eq!(audit.links, 2, "one link per mount");
    assert!(audit.incremental_invariant_holds());
    // 读取（整行/单字段/多次）零派发。
    for _ in 0..5 {
        let _ = e.read_all("a");
        let _ = e.read_state("b", "count");
    }
    assert_eq!(e.execution_audit().handler_entries, 10);
    // 卸载释放；重挂新鲜（seed 重新生效）。
    e.unmount("a");
    assert_eq!(e.read_state("a", "count"), None);
    e.mount("a", Vec::new()).unwrap();
    assert_eq!(count_of(&e, "a"), 0);
    assert_eq!(e.execution_audit().links, 3);
    assert_eq!(e.execution_audit().compile_count, 1);
}
