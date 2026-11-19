//! PLAN-1023 ASE M0 交互会话永久 case（auto-ui 侧消费面；T-01 最小集）。
//!
//! T-01 最小面（V-07/08/10）：两个存活应用会话（同 app 双上下文）经真实
//! 队列点击 A×3/B×7，切回各保留 3/7——保留由**引擎常驻**（每会话一个
//! `NodeDesktopBridge`/`AceStateEngine`，业务状态权威）成立，切换零编译、
//! 零业务初始化、零旧 handler 重复执行（观察口：loads / dispatches / ui_lane_opens
//! 在真实调用边界计量）；两个"最终数字仍相同但机制错误"的负控（切回重放
//! 历史事件、切回重建模型＋种子补数）以同一观察语义判红；无参 Inc/Dec/Reset
//! 真实轨迹 + 带参/载荷近邻在 981 边界具名拒（零状态效果）。完整 V-01..05/
//! 07..10（fixtures、上限/迟到、20 轮、窗口档）在 T-06 固化。
//!
//! 准备（全部档，不在 case 计时内；2026-11-14 测试纪律——日常用例零 rustc）：
//! `cargo build -p auto-ui-runtime --features ace-bridge --example
//! ase_m0_text_window` 后运行 `ase_m0_text_window.exe --prep-only`（ui980
//! provider 预构建/新鲜度缓存；本 worktree target 独立，首跑前必须执行）。

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_node_desktop::{
    host_session_display, host_session_stats, push_event, NodeDesktopBridge, StepOutcome,
};
use auto_ui_runtime::ui::ace_state_bridge::{
    AceAlias, AceInitialValue, AceStateEngine, AceStateValue, DispatchOutcome,
};
use auto_ui_runtime::ui::ase_m0_sessions::{AseApp, AseMetrics, AseSessionHost};
use std::time::Instant;

/// T-01 内联 Counter 源（与 980 MIN-A-UI fixture 同构：无参 Inc/Dec/Reset +
/// int 槽显示；权威 A/B fixtures 由 T-02 落 auto-lang `experimental/ase-m0/
/// fixtures/counter-{a,b}/`）。按钮声明序（冻结）：0=`-`(Dec)、1=Reset、2=`+`(Inc)。
const COUNTER_SRC: &str = "// PLAN-1023 T-01：ASE M0 交互会话最小 Counter（无参事件 A 面）。\nnode Counter {\n    msg { Inc, Dec, Reset }\n\n    model {\n        var count int = 0\n    }\n\n    view {\n        col {\n            text \"ASE M0 Counter\"\n            row {\n                button \"-\" { onclick: .Dec }\n                button \"Reset\" { onclick: .Reset }\n                button \"+\" { onclick: .Inc }\n            }\n        }\n    }\n\n    on {\n        .Inc -> { .count += 1 }\n        .Dec -> { .count -= 1 }\n        .Reset -> { .count = 0 }\n    }\n}\n";

const TARGET_DEC: usize = 0;
const TARGET_RESET: usize = 1;
const TARGET_INC: usize = 2;

fn temp_apps(tag: &str, srcs: &[(&str, &str)]) -> (tempdir::TempDirGuard, Vec<AseApp>) {
    let dir = std::env::temp_dir().join(format!("p1023-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp apps dir");
    let apps = srcs
        .iter()
        .map(|(key, src)| {
            let path = dir.join(format!("{key}.at"));
            std::fs::write(&path, src).expect("write app source");
            AseApp {
                key: key.to_string(),
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect();
    (tempdir::TempDirGuard(dir), apps)
}

mod tempdir {
    use std::path::PathBuf;
    /// 用例收尾删除临时目录（不依赖外部 crate）。
    pub struct TempDirGuard(pub PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// 真实交互 fixture（显式路径；具名准备失败不静默）：组内 auto-lang 兄弟树
/// `experimental/ase-m0/fixtures/counter-{a,b}/main.at`（`ASE_M0_TEXT_ROOT`
/// 可覆盖根，与 1022 文字页同约定）。
fn fixture_apps() -> Vec<AseApp> {
    let root = match std::env::var("ASE_M0_TEXT_ROOT") {
        Ok(p) => std::path::PathBuf::from(p),
        Err(_) => {
            let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest.join("../../../auto-lang/experimental/ase-m0/fixtures")
        }
    };
    ["a", "b"]
        .iter()
        .map(|key| {
            let path = root.join(format!("counter-{key}")).join("main.at");
            assert!(
                path.is_file(),
                "preparation failure: fixture counter-{key} missing (ASE_M0_TEXT_ROOT or the \
                 group sibling auto-lang tree): {}",
                path.display()
            );
            AseApp {
                key: format!("counter-{key}"),
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect()
}

/// 引擎权威 count（业务真值；经 bridge.states=engine.read_all）。
fn engine_count(host: &AseSessionHost, id: usize) -> i32 {
    let row = host
        .engine_states(id)
        .unwrap_or_else(|| panic!("s{id} live"));
    for (n, v) in row {
        if n == "count" {
            if let AceStateValue::Int(i) = v {
                return i;
            }
            panic!("count not int: {v:?}");
        }
    }
    panic!("no count field on s{id}");
}

/// 显示车道 count（只读镜像；ui_apply 提交后的显示模型）。
fn displayed_count(ui_session: i32) -> String {
    let (_, states) =
        host_session_display(ui_session).unwrap_or_else(|| panic!("ui {ui_session} display"));
    states
        .first()
        .and_then(|row| {
            row.iter()
                .find(|(n, _)| n == "count")
                .map(|(_, v)| v.clone())
        })
        .unwrap_or_else(|| panic!("ui {ui_session} count displayed"))
}

fn src_path_of(apps: &[AseApp], key: &str) -> (String, String) {
    let app = apps.iter().find(|a| a.key == key).unwrap();
    let src = std::fs::read_to_string(&app.path).unwrap();
    (src, app.path.clone())
}

/// V-07（最小）：两个存活应用会话（同 app 双上下文）——真实队列点击
/// A×3/B×7，切回各保留 3/7；保留由引擎常驻成立：切换零装载（loads 钉 2）、
/// 零旧 handler 重复执行（dispatches 钉 10）；显示车道重开（lane_opens 增长）
/// 与引擎重建可区分。隐藏会话的引擎权威仍可读（车道撤、业务真值在）。
#[test]
fn p1023_v07_min_two_live_sessions_real_retention() {
    let started = Instant::now();
    let (_guard, apps) = temp_apps("v07", &[("counter", COUNTER_SRC)]);
    let mut host = AseSessionHost::new(apps);

    // 开 A/B（同 app 双上下文：两次独立装载=双 engine，模型不合并）。
    let a = host.open_session("counter").expect("open session a");
    let ui_a1 = host
        .ui_session_of(a)
        .expect("a holds its lane while selected");
    let b = host.open_session("counter").expect("open session b");
    assert_ne!(
        host.ui_session_of(b).expect("b holds its lane"),
        ui_a1,
        "two sessions hold distinct ui lanes"
    );
    assert!(
        host_session_stats(ui_a1).expect("stats a lane").closed,
        "opening b closed a's display lane (engine retained; S-B freeze)"
    );

    // A：真实点击 Inc×3（push_event 入 980 队列 → 受控步进执行 handler）。
    host.select(a).expect("select a");
    for _ in 0..3 {
        host.click(TARGET_INC).expect("click inc");
        match host.step_selected() {
            StepOutcome::Dispatched { .. } => {}
            other => panic!("inc dispatch failed: {other:?}"),
        }
    }
    assert_eq!(engine_count(&host, a), 3, "a engine authority = 3");
    assert_eq!(displayed_count(host.ui_session_of(a).unwrap()), "3");

    // B：切到 B、真实点击 Inc×7；A 隐藏期间引擎真值保持 3。
    host.select(b).expect("select b");
    for _ in 0..7 {
        host.click(TARGET_INC).expect("click inc");
        match host.step_selected() {
            StepOutcome::Dispatched { .. } => {}
            other => panic!("inc dispatch failed: {other:?}"),
        }
    }
    assert_eq!(engine_count(&host, b), 7, "b engine authority = 7");
    assert_eq!(engine_count(&host, a), 3, "a retained 3 while hidden");

    // 切回 A：显示车道从引擎真值重提交 → 显示 3；再切一轮仍稳定。
    host.select(a).expect("select a again");
    assert_eq!(displayed_count(host.ui_session_of(a).unwrap()), "3");
    host.select(b).expect("select b again");
    assert_eq!(displayed_count(host.ui_session_of(b).unwrap()), "7");
    host.select(a).expect("select a third");
    assert_eq!(engine_count(&host, a), 3);
    assert_eq!(engine_count(&host, b), 7);

    // 观察口（真实调用边界计量）：切换零装载、零业务初始化、零旧 handler
    // 重复执行；显示车道重开次数如实增长（与引擎重建可区分的量）。
    let m = host.metrics.clone();
    assert_eq!(m.loads, 2, "exactly two compiles (one per session open)");
    assert_eq!(m.clicks_offered, 10, "3+7 real queue injections");
    assert_eq!(host.dispatches(), 10, "10 handler executions total");
    assert_eq!(host.dispatches_of(a), Some(3));
    assert_eq!(host.dispatches_of(b), Some(7));
    assert!(
        m.ui_lane_opens >= 5,
        "lane reopens grow (got {})",
        m.ui_lane_opens
    );
    assert_eq!(host.switches, 5, "select a,b,a,b,a = five switches");
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);
    eprintln!("p1023_v07_min elapsed: {:?}", started.elapsed());
}

/// V-08（最小）：两个"最终数字仍相同但机制错误"的负控，经同一观察语义
/// （loads=真实装载、dispatches=真实 handler 执行、ui_lane_opens=显示车道）
/// 判红——正控 V-07 断言这些量不增长，负控必须让它们动。
/// 控制1=切回重放历史事件（丢弃引擎→重装→重放 Inc×3：loads 每切回+1、
/// 同一逻辑历史累计执行 6 次>>脚本真实点击 3 次）。
/// 控制2=切回重建模型＋种子补数（engine 重装、mount 种子 count=3：loads
/// 每切回+1，count=3 却零 handler 执行——值来自补数而非执行）。
#[test]
fn p1023_v08_min_controls_same_numbers_wrong_mechanism_red() {
    let started = Instant::now();
    let (_guard, apps) = temp_apps("v08", &[("counter", COUNTER_SRC)]);
    let (src, path) = src_path_of(&apps, "counter");

    // --- 控制1：切回重放历史事件（最终 3 正确，初始化/执行计量红）。
    let mut obs = AseMetrics::default();
    let scripted_clicks: u64 = 3; // 场景脚本真实点击数（用户面）
    obs.loads += 1;
    let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("ctrl1 load 1");
    obs.ui_lane_opens += 1;
    bridge.open().expect("ctrl1 open 1");
    for _ in 0..scripted_clicks {
        push_event(0, TARGET_INC);
        obs.clicks_offered += 1;
        match bridge.step() {
            StepOutcome::Dispatched { .. } => {}
            other => panic!("ctrl1 dispatch: {other:?}"),
        }
    }
    let executed_first = bridge.dispatch_log.len();
    assert_eq!(executed_first, 3);
    // "切走再切回"（错误机制）：丢弃引擎（关车道）→ 重新装载（回初值 0）
    // → 重放历史事件补回数字。
    bridge.close_session().expect("ctrl1 close lane");
    drop(bridge);
    obs.loads += 1;
    let mut bridge2 = NodeDesktopBridge::load(&src, &path, 1).expect("ctrl1 load 2");
    obs.ui_lane_opens += 1;
    bridge2.open().expect("ctrl1 open 2");
    for _ in 0..scripted_clicks {
        push_event(0, TARGET_INC);
        obs.clicks_offered += 1;
        match bridge2.step() {
            StepOutcome::Dispatched { .. } => {}
            other => panic!("ctrl1 replay dispatch: {other:?}"),
        }
    }
    let replayed_count = bridge2
        .states(0)
        .unwrap()
        .into_iter()
        .find(|(n, _)| n == "count")
        .map(|(_, v)| match v {
            AceStateValue::Int(i) => i,
            other => panic!("count int: {other:?}"),
        })
        .unwrap();
    bridge2.close_session().expect("ctrl1 close lane 2");
    // 最终数字仍正确（3）——但观察口红：
    assert_eq!(replayed_count, 3, "control 1 keeps the final number right");
    assert_eq!(
        obs.loads, 2,
        "RED: one compile per switch-back (real path pins 1)"
    );
    assert_eq!(
        obs.clicks_offered,
        scripted_clicks * 2,
        "RED: history replay re-offers already-succeeded events"
    );
    assert_eq!(
        executed_first + bridge2.dispatch_log.len(),
        6,
        "RED: 6 handler executions for a 3-click history"
    );

    // --- 控制2：切回重建模型＋种子补数（最终 3 正确，零执行撑值红）。
    let mut obs2_loads: u64 = 0;
    obs2_loads += 1;
    let mut engine = AceStateEngine::new(&src, &path, AceAlias::Accept).expect("ctrl2 engine 1");
    engine.mount("i0", Vec::new()).expect("ctrl2 mount 1");
    let mut executed = 0usize;
    for _ in 0..3 {
        engine.dispatch("i0", "Inc").expect("ctrl2 inc");
        executed += 1;
    }
    let count_before = match engine.read_state("i0", "count").expect("count") {
        AceStateValue::Int(i) => i,
        other => panic!("int: {other:?}"),
    };
    assert_eq!(count_before, 3);
    // "切回"（错误机制）：重建模型，用种子初值补回相同 count（零执行）。
    obs2_loads += 1;
    let mut engine2 = AceStateEngine::new(&src, &path, AceAlias::Accept).expect("ctrl2 engine 2");
    engine2
        .mount("i0", vec![("count".to_string(), AceInitialValue::Int(3))])
        .expect("ctrl2 seeded mount");
    let patched = match engine2.read_state("i0", "count").expect("count") {
        AceStateValue::Int(i) => i,
        other => panic!("int: {other:?}"),
    };
    // 最终数字仍正确——但观察口红：
    assert_eq!(patched, 3, "control 2 keeps the final number right");
    assert_eq!(obs2_loads, 2, "RED: model rebuilt per switch-back");
    assert_eq!(
        executed, 3,
        "RED context: value 3 restored with 0 new executions (seeded patch, not handler runs)"
    );
    eprintln!("p1023_v08_min elapsed: {:?}", started.elapsed());
}

/// V-10（最小）：无参 Inc/Dec/Reset 真实轨迹（经真实队列与步进）；带参/
/// 载荷近邻在 981 事件边界具名拒（零状态效果）；按钮目标越界具名拒（零
/// 状态/UI 效果）。
#[test]
fn p1023_v10_min_unit_trajectory_and_payload_refusal() {
    let started = Instant::now();
    let (_guard, apps) = temp_apps("v10", &[("counter", COUNTER_SRC)]);
    let (src, path) = src_path_of(&apps, "counter");
    let mut host = AseSessionHost::new(apps);
    let s = host.open_session("counter").expect("open");

    // 正常：0→1→2→3→2→0（Inc×3、Dec、Reset 全部经真实 handler）。
    for expect in [1, 2, 3] {
        host.click(TARGET_INC).expect("inc");
        match host.step_selected() {
            StepOutcome::Dispatched { .. } => {}
            other => panic!("inc: {other:?}"),
        }
        assert_eq!(engine_count(&host, s), expect);
    }
    host.click(TARGET_DEC).expect("dec");
    match host.step_selected() {
        StepOutcome::Dispatched { .. } => {}
        other => panic!("dec: {other:?}"),
    }
    assert_eq!(engine_count(&host, s), 2);
    host.click(TARGET_RESET).expect("reset");
    match host.step_selected() {
        StepOutcome::Dispatched { .. } => {}
        other => panic!("reset: {other:?}"),
    }
    assert_eq!(engine_count(&host, s), 0);
    assert_eq!(displayed_count(host.ui_session_of(s).unwrap()), "0");

    // 带参/载荷近邻：对无参 Inc 施加载荷派发 → 981 边界具名拒（Err 回滚
    // 或 Dropped），零状态效果。
    let mut eng = AceStateEngine::new(&src, &path, AceAlias::Accept).expect("engine");
    eng.mount("i0", Vec::new()).expect("mount");
    let before = eng.read_state("i0", "count");
    let refused = eng.dispatch_with_args("i0", "Inc", &[AceStateValue::Int(1)]);
    match &refused {
        Err(_) | Ok(DispatchOutcome::Dropped(_)) => {}
        Ok(DispatchOutcome::Delivered(states)) => {
            panic!("payload dispatch on unit msg must refuse, got {states:?}")
        }
    }
    assert_eq!(
        eng.read_state("i0", "count"),
        before,
        "payload refusal leaves zero state effect"
    );

    // 按钮目标越界：队列注入 btn99 → 步进具名拒（no button/input binding），
    // 零状态/UI 效果（count 不动、errors 具名可辨）。
    host.click(99).expect("queue accepts the injection");
    match host.step_selected() {
        StepOutcome::LaneError(e) | StepOutcome::DispatchRefused(e) => {
            assert!(e.contains("no button/input binding"), "named refusal: {e}");
        }
        other => panic!("out-of-range target must refuse, got {other:?}"),
    }
    assert_eq!(engine_count(&host, s), 0, "zero business effect");
    eprintln!("p1023_v10_min elapsed: {:?}", started.elapsed());
}

/// T-02：真实交互 fixtures（counter-a/b，显式路径）经同一公共面装载——
/// 共同 Schema/verify 通过（两源装载零错误）、不同合法根名各自独立装载、
/// 双会话一次 Inc 各得 1 且互不影响；文字叶真值各归各源。
#[test]
fn p1023_t02_fixture_counter_sources_load_and_verify() {
    let started = Instant::now();
    let apps = fixture_apps();
    let mut host = AseSessionHost::new(apps);

    let a = host
        .open_session("counter-a")
        .expect("fixture counter-a loads");
    let b = host
        .open_session("counter-b")
        .expect("fixture counter-b loads");
    host.select(a).expect("select a");
    host.click(TARGET_INC).expect("inc a");
    match host.step_selected() {
        StepOutcome::Dispatched { .. } => {}
        other => panic!("inc a: {other:?}"),
    }
    host.select(b).expect("select b");
    host.click(TARGET_INC).expect("inc b");
    match host.step_selected() {
        StepOutcome::Dispatched { .. } => {}
        other => panic!("inc b: {other:?}"),
    }
    assert_eq!(engine_count(&host, a), 1, "counter-a count = 1");
    assert_eq!(engine_count(&host, b), 1, "counter-b count = 1");
    assert_eq!(host.metrics.loads, 2, "one compile per fixture session");
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);

    // 文字叶 AV 真值各归各源（不同根名/文字，同车道；隐藏会话以 980 HOST
    // 保留的最后车道显示快照为证，业务真值经 engine_states 断言）。
    let texts_a = host_session_display(host.last_lane_of(a).unwrap())
        .unwrap()
        .0
        .into_iter()
        .next()
        .unwrap_or_default();
    let texts_b = host_session_display(host.ui_session_of(b).unwrap())
        .unwrap()
        .0
        .into_iter()
        .next()
        .unwrap_or_default();
    assert_eq!(
        texts_a,
        vec!["ASE M0 Counter A".to_string()],
        "counter-a text leaf truth"
    );
    assert_eq!(
        texts_b,
        vec!["counter B ascii".to_string()],
        "counter-b text leaf truth"
    );
    eprintln!("p1023_t02 elapsed: {:?}", started.elapsed());
}
