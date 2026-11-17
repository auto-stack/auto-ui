//! PLAN-041 — component & store instance ownership (T-01 bounded controls).
//!
//! Fixture: `tests/fixtures/plan041_instances/` — two same-type keyed Counter
//! children (start 10 / 20) under P041Host, sharing P041Store.
//!
//! T-01 scope: prove the oracle discriminates BEFORE implementation.
//! - Control leg (green today, must stay green after T-02): build + render +
//!   host shell intact + store fields merged/readable.
//! - Isolation leg (target semantics, `#[ignore]` = T-02 flips it): per-
//!   instance props/model. Run with `cargo test -p auto-ui-runtime --test
//!   plan041_instance_ownership -- --ignored` to replay the pre-fix failure
//!   (T-01 red evidence).

use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_ui_runtime::Component;
use std::path::Path;

fn build_fixture() -> DynamicComponent {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/plan041_instances/src/front/app.at");
    let code = std::fs::read_to_string(&fixture).expect("fixture app.at readable");
    let fixture_str = fixture.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(fixture_str.as_str()))
        .expect("fixture component builds")
}

fn render_debug(comp: &mut DynamicComponent) -> String {
    let (view, _, _) = comp.view_with_debug_gated(false);
    format!("{view:?}")
}

/// Control leg (green before and after the instance split): the fixture
/// builds, both Counter call sites instantiate under the intact host shell,
/// and the store's merged fields are reachable through the root state read.
#[test]
fn t01_control_fixture_builds_and_renders_both_children() {
    let mut comp = build_fixture();
    let debug = render_debug(&mut comp);

    assert!(
        debug.contains("host above counters") && debug.contains("host below counters"),
        "host shell must stay intact: {}",
        snip(&debug)
    );
    // Both same-type call sites instantiated: the two distinct `start`
    // initial seeds (10 last-write-wins over 20 pre-fix; both present
    // post-fix) — at minimum one counter's numeric text node renders.
    assert!(
        debug.contains("content: \"20\"") || debug.contains("content: \"10\""),
        "counter start text must render: {}",
        snip(&debug)
    );
    // Store contract (Plan 370 D-GAP-4): store model fields merge into the
    // root state and read through the unified state.
    let total = comp
        .bridge()
        .read_state("total")
        .expect("store field `total` merged into root state");
    assert_eq!(total, auto_val::Value::Int(0), "initial store total");
}

/// Isolation leg — permanent AC-01 oracle (plan V-01): keyed same-type
/// children own props/model per instance while the App-level store stays
/// shared. Pre-fix red evidence (T-01): one Inc moved BOTH counters.
#[test]
fn t02_keyed_same_type_children_own_props_model_and_share_store() {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    let mut comp = build_fixture();
    let before = render_debug(&mut comp);

    // Prop ownership: A renders its start=10, B renders start=20.
    assert!(
        before.contains("content: \"10\"") && before.contains("content: \"20\""),
        "keyed same-type children must render their own props (want 10 and 20): {}",
        snip(&before)
    );

    // One Inc on instance A (real event path, instance-bound) must move A
    // and the SHARED store, leaving B's model untouched.
    comp.on(DynamicMessage::TypedInst {
        widget_name: "Counter".into(),
        event_name: "Inc".into(),
        args: vec![],
        instance: "/Counter#a".into(),
    });
    let after = render_debug(&mut comp);

    // Post-Inc render: A = (10, count=1, total=1), B = (20, count=0, total=1)
    // → exactly three `content: "1"` nodes (A.count, A.total, B.total).
    let ones = after.matches("content: \"1\"").count();
    assert_eq!(
        ones,
        3,
        "after one Inc: A.count=1 + shared store total=1 on both instances (B.count stays 0): {}",
        snip(&after)
    );
    assert!(
        after.contains("content: \"0\""),
        "B.count must stay 0 (instance isolation): {}",
        snip(&after)
    );
    assert!(
        after.contains("content: \"10\"") && after.contains("content: \"20\""),
        "starts must stay per-instance after dispatch: {}",
        snip(&after)
    );
}

/// V-03 (AC-01/03): keyed reorder keeps instance identity — the count lives
/// with the KEY, not the visual position.
#[test]
fn t04_keyed_reorder_keeps_instance_identity() {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    let mut comp = build_fixture();
    let _ = render_debug(&mut comp); // frame1: A + B mounted

    comp.on(DynamicMessage::TypedInst {
        widget_name: "Counter".into(),
        event_name: "Inc".into(),
        args: vec![],
        instance: "/Counter#a".into(),
    });
    let id_a = |dc: &auto_ui_runtime::ui::dynamic::DynamicComponent| {
        dc.bridge()
            .read_instance_state("/Counter#a", "count")
            .expect("A live")
    };
    assert_eq!(id_a(&comp), auto_val::Value::Int(1));

    // Swap the call-site keys: A and B exchange visual positions.
    comp.call_widget_handler("P041Host", "SwapKeys", &[])
        .expect("SwapKeys");
    let after = render_debug(&mut comp);
    assert!(
        after.contains("content: \"10\"") && after.contains("content: \"20\""),
        "both instances render after reorder: {}",
        snip(&after)
    );
    // The count followed the key: A still owns count=1.
    assert_eq!(
        id_a(&comp),
        auto_val::Value::Int(1),
        "A identity survives reorder"
    );
    // B stayed untouched by A's Inc.
    assert_eq!(
        comp.bridge()
            .read_instance_state("/Counter#b", "count")
            .expect("B live"),
        auto_val::Value::Int(0)
    );
}

/// V-05 (AC-03): unmount releases ONLY the target; the surviving instance
/// keeps operating; remount is a fresh instance; a late event to the
/// released identity is dropped (no redirect).
#[test]
fn t04_unmount_releases_target_survivor_continues_late_event_dropped() {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    let mut comp = build_fixture();
    let _ = render_debug(&mut comp); // A + B live

    comp.on(DynamicMessage::TypedInst {
        widget_name: "Counter".into(),
        event_name: "Inc".into(),
        args: vec![],
        instance: "/Counter#a".into(),
    });
    assert_eq!(
        comp.bridge()
            .read_instance_state("/Counter#a", "count")
            .unwrap(),
        auto_val::Value::Int(1)
    );

    // Close B: its state object is released (identity gone).
    comp.call_widget_handler("P041Host", "ToggleB", &[])
        .expect("ToggleB");
    let _ = render_debug(&mut comp);
    assert!(
        comp.bridge()
            .child_instance_state_id("/Counter#b")
            .is_none(),
        "released B must drop out of the instance registry"
    );
    assert!(
        comp.bridge()
            .child_instance_state_id("/Counter#a")
            .is_some(),
        "A must survive B's close"
    );

    // Late event to the released identity: dropped, no redirect.
    comp.on(DynamicMessage::TypedInst {
        widget_name: "Counter".into(),
        event_name: "Inc".into(),
        args: vec![],
        instance: "/Counter#b".into(),
    });
    assert!(
        comp.bridge()
            .child_instance_state_id("/Counter#b")
            .is_none(),
        "late event must not resurrect/re-target the released identity"
    );

    // Survivor continues operating.
    comp.on(DynamicMessage::TypedInst {
        widget_name: "Counter".into(),
        event_name: "Inc".into(),
        args: vec![],
        instance: "/Counter#a".into(),
    });
    assert_eq!(
        comp.bridge()
            .read_instance_state("/Counter#a", "count")
            .unwrap(),
        auto_val::Value::Int(2)
    );

    // Remount B: fresh instance (Vue v-if semantics — no state inheritance).
    comp.call_widget_handler("P041Host", "ToggleB", &[])
        .expect("ToggleB");
    let after = render_debug(&mut comp);
    assert!(
        comp.bridge()
            .child_instance_state_id("/Counter#b")
            .is_some(),
        "B remounts as a new instance"
    );
    assert!(
        after.contains("content: \"0\""),
        "remounted B starts fresh (count=0): {}",
        snip(&after)
    );
    // A unchanged by the remount.
    assert_eq!(
        comp.bridge()
            .read_instance_state("/Counter#a", "count")
            .unwrap(),
        auto_val::Value::Int(2)
    );
}

fn snip(s: &str) -> String {
    let mut out = s.chars().take(1600).collect::<String>();
    if out.len() < s.len() {
        out.push('…');
    }
    out
}

// ============================================================================
// PLAN-980 B-STATE — ACE 状态执行桥（`ui/ace_state_bridge.rs`）永久 case。
//
// 输入：真实 002-counter 原源（examples/ui/002-counter；lang 侧已冻结
// sha256/16 = 04937c35325faa74）+ store 形 widget 源 + 968 面 task 源。
// oracle：双 App 各两组件不串 / 异步完成只写存活实例（MIX-03 无 HTTP 版，
// 真实异步宿主 = ACE scheduler）/ 卸载晚到丢弃、重挂新鲜、世代隔离 /
// store handler 经 ACE 执行。旧链 4 case（上方）保持不动、两路并存。
// ============================================================================

#[cfg(feature = "ace-bridge")]
mod p980_ace_state {
    use auto_ui_runtime::ui::ace_state_bridge::{
        AceAlias, AceInitialValue, AceStateEngine, AceStateValue, AsyncOutcome, DispatchOutcome,
        DropReason, UnmountOutcome,
    };

    fn engine_002() -> AceStateEngine {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
        let src = std::fs::read_to_string(&at).expect("002-counter real source readable");
        assert!(
            src.contains("widget App") && src.contains("ToggleMode"),
            "002 source identity markers"
        );
        AceStateEngine::new(
            &src,
            "examples/ui/002-counter/src/front/app.at",
            AceAlias::Accept,
        )
        .expect("002 compiles on the AGE component face")
    }

    /// AC-B-STATE：双 App（双引擎）各两组件（keyed 实例）互不串扰；props
    /// 按实例种子（10/20）；int/str 状态字段均经 ACE 逐字段观察。
    #[test]
    fn p980_state_two_apps_two_instances_isolated() {
        let mut app0 = engine_002();
        let mut app1 = engine_002();
        app0.mount(
            "/Counter#a",
            vec![("count".into(), AceInitialValue::Int(10))],
        )
        .unwrap();
        app0.mount(
            "/Counter#b",
            vec![("count".into(), AceInitialValue::Int(20))],
        )
        .unwrap();
        app1.mount("/Counter#a", vec![]).unwrap();
        app1.mount("/Counter#b", vec![]).unwrap();

        // 初始观察：props 种子按实例生效。
        assert_eq!(
            app0.read_state("/Counter#a", "count"),
            Some(AceStateValue::Int(10))
        );
        assert_eq!(
            app0.read_state("/Counter#b", "count"),
            Some(AceStateValue::Int(20))
        );
        assert_eq!(
            app1.read_state("/Counter#a", "count"),
            Some(AceStateValue::Int(0))
        );

        // app0 实例 A 的 Inc：只动 A（10→11），B 与另一 App 全保持。
        match app0.dispatch("/Counter#a", "Inc").unwrap() {
            DispatchOutcome::Delivered(_) => {}
            other => panic!("dispatch must deliver: {other:?}"),
        }
        assert_eq!(
            app0.read_state("/Counter#a", "count"),
            Some(AceStateValue::Int(11))
        );
        assert_eq!(
            app0.read_state("/Counter#b", "count"),
            Some(AceStateValue::Int(20))
        );
        assert_eq!(
            app1.read_state("/Counter#a", "count"),
            Some(AceStateValue::Int(0))
        );
        assert_eq!(
            app1.read_state("/Counter#b", "count"),
            Some(AceStateValue::Int(0))
        );

        // app1 实例 B 的 Dec：只动该实例（0→-1），app0 不受影响。
        app1.dispatch("/Counter#b", "Dec").unwrap();
        assert_eq!(
            app1.read_state("/Counter#b", "count"),
            Some(AceStateValue::Int(-1))
        );
        assert_eq!(
            app0.read_state("/Counter#a", "count"),
            Some(AceStateValue::Int(11))
        );

        // str 状态字段同引擎同链路：UseSolarized 只改 app0 实例 A。
        app0.dispatch("/Counter#a", "UseSolarized").unwrap();
        assert_eq!(
            app0.read_state("/Counter#a", "theme_name"),
            Some(AceStateValue::Str("Solarized".into()))
        );
        assert_eq!(
            app0.read_state("/Counter#b", "theme_name"),
            Some(AceStateValue::Str("Auto".into()))
        );
        assert_eq!(
            app1.read_state("/Counter#a", "theme_name"),
            Some(AceStateValue::Str("Auto".into()))
        );
    }

    /// AC-B-STATE：卸载只释放目标实例；晚到事件丢弃（不复活/不重定向）；
    /// 存活实例继续；重挂 = 全新实例（新世代、回到种子态）。
    #[test]
    fn p980_state_unmount_late_dispatch_dropped_survivor_remount_fresh() {
        let mut e = engine_002();
        e.mount("/c#a", vec![("count".into(), AceInitialValue::Int(5))])
            .unwrap();
        let gen_b1 = match e.mount("/c#b", vec![]) {
            Ok(g) => g,
            Err(err) => panic!("mount b: {err}"),
        };

        e.dispatch("/c#a", "Inc").unwrap();
        assert_eq!(e.read_state("/c#a", "count"), Some(AceStateValue::Int(6)));

        // 关闭 B：释放（世代可见），已释放实例读不到。
        assert!(matches!(e.unmount("/c#b"), UnmountOutcome::Released(g) if g == gen_b1));
        assert_eq!(e.read_state("/c#b", "count"), None);

        // 晚到事件：丢弃（InstanceReleased），不复活、不重定向。
        assert_eq!(
            e.dispatch("/c#b", "Inc").unwrap(),
            DispatchOutcome::Dropped(DropReason::InstanceReleased)
        );
        assert_eq!(e.read_state("/c#b", "count"), None);

        // 存活者继续。
        e.dispatch("/c#a", "Inc").unwrap();
        assert_eq!(e.read_state("/c#a", "count"), Some(AceStateValue::Int(7)));

        // 重挂 B：全新实例（新世代），回到源内初值（count=0，种子为空）。
        let gen_b2 = e.mount("/c#b", vec![]).unwrap();
        assert_ne!(gen_b1, gen_b2, "remount must mint a fresh generation");
        assert_eq!(e.read_state("/c#b", "count"), Some(AceStateValue::Int(0)));
        assert_eq!(e.read_state("/c#a", "count"), Some(AceStateValue::Int(7)));
    }

    /// MIX-03 无 HTTP 版（AC-B-STATE）：两实例各触发异步（真实异步宿主 =
    /// ACE scheduler：go/sleep/虚拟时钟到静止）；一实例先关闭 → 完成只写
    /// 存活实例，关闭实例不复活；重挂后旧世代完成仍被丢弃（新旧更新代
    /// 隔离），新世代正常回写。HTTP 实腿由 983/联验补齐。
    #[test]
    fn p980_state_async_completion_writes_only_live_instance() {
        const ASYNC_TASK: &str = "fn main() {\n    go worker()\n    print(\"start\")\n}\nfn \
                                   worker() {\n    sleep(50)\n    print(\"done\")\n}\n";
        let mut e = engine_002();
        e.mount("/p#a", vec![]).unwrap();
        e.mount("/p#b", vec![]).unwrap();
        assert_eq!(
            e.read_state("/p#a", "dark_mode"),
            Some(AceStateValue::Bool(false))
        );

        let ta = e.spawn_async("/p#a", "ToggleMode", ASYNC_TASK).unwrap();
        let tb = e.spawn_async("/p#b", "ToggleMode", ASYNC_TASK).unwrap();

        // B 先关闭（请求在途）。
        assert!(matches!(e.unmount("/p#b"), UnmountOutcome::Released(_)));

        // A 完成：真实 task 执行成功后回写，dark_mode 翻转。
        match e.await_async(ta).unwrap() {
            AsyncOutcome::Completed(_) => {}
            other => panic!("live instance completion must land: {other:?}"),
        }
        assert_eq!(
            e.read_state("/p#a", "dark_mode"),
            Some(AceStateValue::Bool(true))
        );

        // B 完成：实例已释放 → 丢弃，不复活。
        assert_eq!(
            e.await_async(tb).unwrap(),
            AsyncOutcome::Dropped(DropReason::InstanceReleased)
        );
        assert_eq!(e.read_state("/p#b", "dark_mode"), None);

        // 重挂 B（新世代）：旧世代 ticket 即便实例已重挂也必须丢弃。
        e.mount("/p#b", vec![]).unwrap();
        let td = e.spawn_async("/p#b", "ToggleMode", ASYNC_TASK).unwrap();
        assert!(matches!(e.unmount("/p#b"), UnmountOutcome::Released(_)));
        e.mount("/p#b", vec![]).unwrap(); // 再重挂 → 又一个新世代
        assert_eq!(
            e.await_async(td).unwrap(),
            AsyncOutcome::Dropped(DropReason::StaleGeneration)
        );
        assert_eq!(
            e.read_state("/p#b", "dark_mode"),
            Some(AceStateValue::Bool(false))
        );

        // 新世代 ticket 正常回写。
        let te = e.spawn_async("/p#b", "ToggleMode", ASYNC_TASK).unwrap();
        match e.await_async(te).unwrap() {
            AsyncOutcome::Completed(_) => {}
            other => panic!("current-generation completion must land: {other:?}"),
        }
        assert_eq!(
            e.read_state("/p#b", "dark_mode"),
            Some(AceStateValue::Bool(true))
        );
    }

    /// App store 的 handler 经 ACE 执行（store 形 widget 源：model+msg+on）；
    /// 共享 = 跨消费方路由到同一 store 引擎；未声明消息在 ACE 面具名拒且
    /// 引擎不被毒化（失败回滚后继续可用）。
    #[test]
    fn p980_state_store_handler_executes_via_ace() {
        const STORE_ANALOG: &str = "widget P980Store {\n    msg { Bump }\n    model { var total \
                                    int = 0 }\n    on { .Bump -> { .total += 1 } }\n    view { \
                                    text \"\" {}\n    }\n}\n";
        let mut store = AceStateEngine::new(STORE_ANALOG, "p980_store.at", AceAlias::Accept)
            .expect("store-shaped source compiles");
        store.mount("store", vec![]).unwrap();

        // 两个消费方（两处组件事件）都路由到同一 store 引擎 → total 共享推进。
        store.dispatch("store", "Bump").unwrap();
        store.dispatch("store", "Bump").unwrap();
        assert_eq!(
            store.read_state("store", "total"),
            Some(AceStateValue::Int(2))
        );

        // 未声明消息：ACE 面具名拒（不静默丢）。
        let err = store.dispatch("store", "Nope").unwrap_err();
        assert!(err.contains("source.age.component-event"), "{err}");

        // 失败已回滚：同一入口继续可用（Bump 仍从 2 推进）。
        store.dispatch("store", "Bump").unwrap();
        assert_eq!(
            store.read_state("store", "total"),
            Some(AceStateValue::Int(3))
        );
    }
    /// 复核反例（B-STATE-R）：异步完成不可伪造——任务源在 ACE 内真实失败
    /// （未知 task 名，968 具名拒）时 `await_async` 必须报错，而不是把
    /// "完成"伪造成 Completed/Dropped；实例状态保持未被写入。
    #[test]
    fn p980_state_async_task_failure_surfaces_not_fabricated() {
        const BAD_TASK: &str = "fn main() {
    let h = Task.spawn(\"Nope\", 16)
}
";
        let mut e = engine_002();
        e.mount("/x#a", vec![]).unwrap();
        let t = e.spawn_async("/x#a", "Inc", BAD_TASK).unwrap();
        // 真实失败在 ACE 段具名拒（本例为编译段 bind 拒；运行段失败则报
        // "async task did not succeed"）——两形都不得伪造成完成。
        let err = e.await_async(t).unwrap_err();
        assert!(
            err.contains("no task named `Nope`") || err.contains("async task did not succeed"),
            "{err}"
        );
        assert_eq!(e.read_state("/x#a", "count"), Some(AceStateValue::Int(0)));
    }
}
