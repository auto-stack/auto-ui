//! PLAN-049 T-05（driver 侧 flow 正反例门，离屏确定性、单实例 GPU）：
//! V-S01/V-S02 对应——按钮/双区域路由、同定义两实例按步推进且互不串扰、
//! 单次 Tick 纪律、动态重挂载；生命周期——陈旧代次具名拒绝、Session 故障
//! 相关域停止且最后可用呈现保留、队列满具名丢弃；预算——实例/区域/场景/
//! 边长/schema 消费门具名拒绝；受影响旧档并集——048 产物照常消费、混合
//! 解析拒 048 schema。
//! Run: cargo test -p autographics-auto-driver --test node_mixed_views_flow
///       -- --test-threads=1
use agp::renderer::GpuDevice;
use autographics_auto_driver::node_mixed::{parse_mixed_artifact, MixedUiApp};
use autographics_auto_driver::node_mixed_host::MixedHost;
use autographics_auto_driver::node_ui::{parse_artifact, NodeUiApp};

const UI_JSON: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/ui.json");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/world.at");
const UI_AB_UI: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/ui.json");
const UI_AB_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/world.at");

const BUTTON: &str = "Main#inc";
const A_INC: &str = "Main/a#inc";
const B_INC: &str = "Main/b#inc";
const REG2: &str = "Main#reg2";
const LOGICAL: [f32; 2] = [420.0, 420.0];

fn host() -> MixedHost {
    let gpu = GpuDevice::new().expect("gpu");
    let app = MixedUiApp::new(UI_JSON, WORLD, "plan049-flow").expect("mixed app");
    MixedHost::new(gpu, app, LOGICAL, None).expect("mixed host")
}

fn center(r: agp::core::composition::Rect) -> [f32; 2] {
    [r.x + r.w / 2.0, r.y + r.h / 2.0]
}

fn click(h: &mut MixedHost, key: &str) {
    let r = h.widget_bounds(key).expect(key);
    h.pointer_pressed(center(r)).expect("press");
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
}

fn texts(h: &mut MixedHost, instance: usize) -> Vec<String> {
    h.app().ui.texts(instance).to_vec()
}

// ------------------------------------------------------------- positives --

#[test]
fn flow_two_instances_route_independently_with_single_tick_discipline() {
    let mut h = host();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    // 同定义两实例：同一 ABI、各自 SoA 槽；互不串扰。
    click(&mut h, A_INC);
    assert_eq!(
        texts(&mut h, 1).get(1).map(String::as_str),
        Some("1"),
        "a.count += 1"
    );
    assert_eq!(
        texts(&mut h, 2).get(1).map(String::as_str),
        Some("10"),
        "b untouched"
    );
    click(&mut h, B_INC);
    assert_eq!(
        texts(&mut h, 2).get(1).map(String::as_str),
        Some("12"),
        "b.count += 2"
    );
    assert_eq!(
        texts(&mut h, 1).get(1).map(String::as_str),
        Some("1"),
        "a untouched"
    );

    // root 按钮 → count/2D/3D 同步推进。
    let reg2_before = h.widget_bounds(REG2).expect("reg2");
    click(&mut h, BUTTON);
    assert_eq!(texts(&mut h, 0).get(0).map(String::as_str), Some("1"));
    let ui = h.ui;
    let quad_red =
        h.rt.probe_pixel(ui, [reg2_before.x + 20.0, reg2_before.y + 32.0])
            .is_some_and(|q| u16::from(q[0]) > 150 && u16::from(q[0]) > u16::from(q[1]) + 50);
    assert!(quad_red, "2D quad must sit at x=8 after one Inc");

    // 动态重挂载（区域点击长高 spacer → 区域下移 → 挂载/路由跟随）。
    click(&mut h, REG2);
    let reg2_after = h.widget_bounds(REG2).expect("reg2 after growth");
    assert!(
        reg2_after.y >= reg2_before.y + 39.0,
        "region must move with real layout"
    );
    assert_eq!(texts(&mut h, 0).get(2).map(String::as_str), Some("1"));

    // 一次更新纪律：显式步恰一次；其后纯重绘零再入。
    let before = h.app().ui.callbacks;
    h.step(1.0 / 60.0).expect("step");
    let after_step = h.app().ui.callbacks;
    assert!(after_step > before);
    for _ in 0..5 {
        let r = h.frame(None).expect("frame");
        assert_eq!(r.dispatched, 0);
    }
    assert_eq!(
        h.app().ui.callbacks,
        after_step,
        "rendering must not re-tick"
    );
    // 空闲零 Tick。
    let idle = h.app().ui.callbacks;
    for _ in 0..30 {
        h.frame(None).expect("idle");
    }
    assert_eq!(h.app().ui.callbacks, idle);
}

// ---------------------------------------------------------- lifecycle ----

#[test]
fn stale_generation_events_are_named_rejected() {
    let mut h = host();
    for _ in 0..2 {
        h.frame(None).expect("frame");
    }
    // 旧 Run 代次入队 → start_run 递增 → 派发前具名拒绝。
    let gen = h.app().ui.generation;
    assert!(h
        .app()
        .ui
        .enqueue_click(0, "main_inc".into(), None, "stale-probe".into()));
    h.app().start_run().expect("new run");
    assert_eq!(h.app().ui.generation, gen + 1);
    let report = h.app().drain();
    assert_eq!(report.stale_rejected, 1, "stale event named-rejected");
    assert_eq!(report.dispatched, 0);
    assert_eq!(texts(&mut h, 0).get(0).map(String::as_str), Some("0"));
}

#[test]
fn session_fault_stops_domain_and_keeps_last_good_presentation() {
    let mut h = host();
    for _ in 0..2 {
        h.frame(None).expect("frame");
    }
    click(&mut h, BUTTON);
    assert_eq!(texts(&mut h, 0).get(0).map(String::as_str), Some("1"));
    // 未知派发器 = 相关域故障：停止、清队列、具名错误、呈现保留。
    assert!(h
        .app()
        .ui
        .enqueue_click(0, "no_such_fn".into(), None, "fault-probe".into()));
    let report = h.app().drain();
    assert_eq!(report.faults, 1);
    assert!(h.app().ui.error.is_some(), "named domain error recorded");
    // 故障后：调用拒绝、队列清空、最后可用呈现保持。
    assert!(h
        .app()
        .ui
        .enqueue_click(0, "main_inc".into(), None, "post-fault".into()));
    let again = h.app().drain();
    assert_eq!(again.dispatched, 0, "closed session dispatches nothing");
    assert_eq!(
        texts(&mut h, 0).get(0).map(String::as_str),
        Some("1"),
        "last-good text kept"
    );
}

#[test]
fn queue_overflow_is_named_dropped() {
    let mut h = host();
    for _ in 0..2 {
        h.frame(None).expect("frame");
    }
    let mut accepted = 0usize;
    for i in 0..70 {
        // 派发器 ABI（adapter 只认派发器名；handler 直呼不经 refresh，
        // 文字缓冲不更新——T-05 门修正）。
        if h.app()
            .ui
            .enqueue_click(0, "main_ui_click_inc".into(), None, format!("ovf{i}"))
        {
            accepted += 1;
        }
    }
    assert_eq!(accepted, 64, "queue cap 64");
    assert_eq!(
        h.app().ui.dropped_overflow,
        6,
        "overflow named-dropped and counted"
    );
    let report = h.app().drain();
    assert_eq!(report.dispatched, 64);
    assert!(h.app().ui.error.is_none());
    assert_eq!(texts(&mut h, 0).get(0).map(String::as_str), Some("64"));
}

// ------------------------------------------------- budget / old archive --

fn minimal_mixed_json(elements: &str) -> String {
    format!(
        r#"{{"schema":"autographics.node-ui-mixed/1","generator":"g","world_sha256":"00","root":"Main","instances":[],"defs":{{"Main":{{"prefix":"main","is_root":true,"texts":0,"elements":[{elements}]}}}}}}"#
    )
}

fn embed(n: &str) -> String {
    format!(r#"{{"kind":"Embed","name":"{n}","dim":"2d","width":40,"height":40,"scene":[]}}"#)
}

#[test]
fn capacity_envelopes_and_schema_gates_are_named() {
    // 区域预算：>4 具名拒绝。
    let five = [
        embed("r1"),
        embed("r2"),
        embed("r3"),
        embed("r4"),
        embed("r5"),
    ]
    .join(",");
    let err = parse_mixed_artifact(&minimal_mixed_json(&five)).expect_err("5 regions");
    assert!(err.contains("regions exceed the frozen 4"), "{err}");

    // 边长预算：0 与 641 拒绝。
    let big = r#"{"kind":"Embed","name":"r","dim":"2d","width":641,"height":40,"scene":[]}"#;
    let err = parse_mixed_artifact(&minimal_mixed_json(big)).expect_err("641");
    assert!(err.contains("1..=640"), "{err}");
    let zero = r#"{"kind":"Embed","name":"r","dim":"2d","width":0,"height":40,"scene":[]}"#;
    assert!(
        parse_mixed_artifact(&minimal_mixed_json(zero)).is_err(),
        "0 side"
    );

    // 坏 dim。
    let bad = r#"{"kind":"Embed","name":"r","dim":"4d","width":40,"height":40,"scene":[]}"#;
    let err = parse_mixed_artifact(&minimal_mixed_json(bad)).expect_err("4d");
    assert!(err.contains("not 2d|3d"), "{err}");

    // 非 root 区域（消费侧同一冻结切片）。
    let nonroot = r#"{"schema":"autographics.node-ui-mixed/1","generator":"g","world_sha256":"00","root":"Main","instances":[],"defs":{"Main":{"prefix":"main","is_root":true,"texts":0,"elements":[]},"C":{"prefix":"c","is_root":false,"texts":0,"elements":[{"kind":"Embed","name":"badge","dim":"2d","width":40,"height":40,"scene":[]}]}}}"#;
    let err = parse_mixed_artifact(nonroot).expect_err("non-root region");
    assert!(err.contains("non-root def"), "{err}");

    // schema 门：048 产物走混合解析拒；含嵌入的产物走 048 解析拒。
    let with_embed = minimal_mixed_json(&embed("r1"))
        .replace("autographics.node-ui-mixed/1", "autographics.node-ui/1");
    let err = autographics_auto_driver::node_ui::parse_artifact(&with_embed)
        .expect_err("048 schema must reject embeds");
    assert!(err.contains("mixed profile required"), "{err}");
    assert!(
        parse_mixed_artifact(UI_JSON).is_ok(),
        "committed mixed artifact parses"
    );
    let err = parse_mixed_artifact(UI_AB_UI).expect_err("048 schema through mixed parse");
    assert!(err.contains("schema mismatch"), "{err}");
}

#[test]
fn affected_old_archive_consumes_unchanged_on_driver_side() {
    // 受影响旧档并集（driver 侧）：048 node-ui 产物照常装配（main-first、
    // 身份链），Start 后文字照常接受。
    let mut app = NodeUiApp::new(UI_AB_UI, UI_AB_WORLD, "plan048-regress").expect("048 app");
    app.start_run().expect("start");
    assert!(!app.texts(0).is_empty(), "root texts accepted");
    // 两次点击派发走同一 048 ABI。
    app.enqueue_click(0, "main_ui_click_flip".into(), None, "old-archive".into());
    let report = app.drain();
    assert_eq!(report.dispatched, 1);
    assert!(app.error.is_none());
}
