//! PLAN-054 T-02 全链机制先证（探针；离屏确定性、单实例 GPU、--test-threads=1）。
//!
//! 证明 schema v2（`autographics.node-ui-mixed-regions/2`）的完整链：
//! 普通 `use` 组件内具名区域可实例化（两个同定义 Gauge 实例 + 一个异维
//! Meter 组件 + root 区域，全部非 Main/reg2/reg3 命名），metadata 驱动装配
//! （无固定名分派），真实 UI 按钮/区域点击改 Auto 状态 → 各区域真实 GPU
//! 像素变化（经门户采样的 UI 面阈值 oracle，沿 049 判色式）；应用活区域
//! ≤4（第 5 个具名拒绝，卸载释放名额后可激活）；条件隐藏＝撤呈现
//! （portal+view+surface 释放、实例状态保留、重挂新代次、旧区域事件具名
//! 拒绝）；显式步逐实例 Tick 推进。
//!
//! Run: cargo test -p autographics-auto-driver --test node_mixed_regions_probe
///       -- --test-threads=1
use agp::core::composition::Rect;
use agp::renderer::GpuDevice;
use autographics_auto_driver::node_mixed::{parse_mixed_artifact, RegionState, SceneElemState};
use autographics_auto_driver::node_mixed_regions::{
    parse_mixed_regions_artifact, MixedRegionsUiApp,
};
use autographics_auto_driver::node_mixed_regions_host::MixedRegionsHost;
use autographics_auto_driver::node_ui::parse_artifact;

const UI_JSON: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-dash-regions/ui.json"
);
const WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-dash-regions/world.at"
);
const V1_UI_JSON: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/ui.json");

const LOGICAL: [f32; 2] = [480.0, 560.0];

fn host() -> MixedRegionsHost {
    let gpu = GpuDevice::new().expect("gpu");
    let app = MixedRegionsUiApp::new(UI_JSON, WORLD, "plan054-probe").expect("v2 app");
    MixedRegionsHost::new(gpu, app, LOGICAL, None).expect("v2 host")
}

fn center(r: Rect) -> [f32; 2] {
    [r.x + r.w / 2.0, r.y + r.h / 2.0]
}

fn settle(h: &mut MixedRegionsHost, n: usize) {
    for _ in 0..n {
        h.frame(None).expect("frame");
    }
}

fn click_widget(h: &mut MixedRegionsHost, key: &str) {
    let r = h.widget_bounds(key).unwrap_or_else(|| panic!("bounds {key}"));
    h.pointer_pressed(center(r)).expect("press");
    h.pointer_released();
    settle(h, 3);
}

fn texts(h: &mut MixedRegionsHost) -> Vec<String> {
    h.app().ui.texts(0).to_vec()
}

/// UI 面上区域局部坐标的合成像素（门户采样链的真实证据面）。
fn ui_px(h: &mut MixedRegionsHost, key: &str, local: [f32; 2]) -> [u8; 4] {
    let b = h.widget_bounds(key).unwrap_or_else(|| panic!("bounds {key}"));
    h.rt
        .probe_pixel(h.ui, [b.x + local[0], b.y + local[1]])
        .unwrap_or_else(|| panic!("pixel {key}"))
}

fn is_red(q: [u8; 4]) -> bool {
    q[0] > 150 && u16::from(q[0]) > u16::from(q[1]) + 50
}

fn is_blue(q: [u8; 4]) -> bool {
    q[2] > 150 && u16::from(q[2]) > u16::from(q[0]) + 50
}

fn quad_x(s: &RegionState) -> f32 {
    match &s.elems[0] {
        SceneElemState::Quad { x, .. } => *x,
        _ => panic!("quad"),
    }
}

fn state_of<'a>(st: &'a [RegionState], path: &str) -> &'a RegionState {
    st.iter().find(|s| s.path == path).unwrap_or_else(|| panic!("state {path}"))
}

// ------------------------------------------------------------- assembly --

#[test]
fn probe_metadata_assembly_is_name_free_and_cap_enforced() {
    let mut h = host();
    settle(&mut h, 4);
    // 目录驱动的 4 个活动区域（第 5 个 p3 被预算具名拒绝）；无固定名。
    let mut keys = h.active_keys();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "Dash#strip".to_string(),
            "Dash/m3#dial".to_string(),
            "Dash/p1#arc".to_string(),
            "Dash/p2#arc".to_string(),
        ]
    );
    assert_eq!(h.active_count(), 4);
    assert_eq!(h.mount_rejected, vec!["Dash/p3#arc".to_string()]);
    // 真实布局 bounds 在场；挂载代次已登记。
    for k in &keys {
        assert!(h.widget_bounds(k).is_some(), "bounds {k}");
        assert!(h.region_epoch(k).is_some(), "epoch {k}");
    }
    // GPU 真实内容（经门户合成）：p1/p2 同定义初始同状（红点），根区域蓝点。
    assert!(is_red(ui_px(&mut h, "Dash/p1#arc", [12.0, 32.0])), "p1 red");
    assert!(is_red(ui_px(&mut h, "Dash/p2#arc", [12.0, 32.0])), "p2 red");
    assert!(
        is_blue(ui_px(&mut h, "Dash#strip", [8.0, 20.0])),
        "root blue"
    );
    // 背景非内容像素（区域角落非红/蓝）。
    assert!(!is_red(ui_px(&mut h, "Dash/p1#arc", [90.0, 60.0])));
    // 3D 区域在场（合成像素非纯天空底色的断言在 step 用例做变化对照）。
}

// ------------------------------------------------- instance isolation --

#[test]
fn probe_same_def_instances_click_route_and_isolate() {
    let mut h = host();
    settle(&mut h, 4);
    // p1 ×2、p2 ×1（真实窗口坐标经门户逆变换路由；2:1 独立状态）。
    for _ in 0..2 {
        let r = h.widget_bounds("Dash/p1#arc").expect("p1 bounds");
        h.pointer_pressed(center(r)).expect("press");
        h.pointer_released();
        settle(&mut h, 2);
    }
    let r2 = h.widget_bounds("Dash/p2#arc").expect("p2 bounds");
    h.pointer_pressed(center(r2)).expect("press");
    h.pointer_released();
    settle(&mut h, 2);
    let st = h.app().scene_state();
    assert_eq!(quad_x(state_of(&st, "Dash/p1")), 24.0, "p1 clicked twice");
    assert_eq!(quad_x(state_of(&st, "Dash/p2")), 12.0, "p2 clicked once");
    // GPU 像素分开（合成面）：p1 点已位移到 24..48，p2 仍原地。
    assert!(is_red(ui_px(&mut h, "Dash/p1#arc", [36.0, 32.0])), "p1 moved");
    assert!(
        is_red(ui_px(&mut h, "Dash/p2#arc", [12.0, 32.0])),
        "p2 at origin"
    );
    assert!(
        !is_red(ui_px(&mut h, "Dash/p2#arc", [36.0, 32.0])),
        "p2 dot must not move"
    );
    // 区域点击不串 UI 文字（count 仍 0；互斥）。
    assert_eq!(texts(&mut h)[0], "0");
}

#[test]
fn probe_ui_button_drives_root_state_and_region_gpu() {
    let mut h = host();
    settle(&mut h, 4);
    click_widget(&mut h, "Dash#bump");
    assert_eq!(texts(&mut h)[0], "1", "count via UI button");
    // 根区域蓝点随 .strip_x 位移（10 → 覆盖 10..26）。
    assert!(is_blue(ui_px(&mut h, "Dash#strip", [18.0, 20.0])));
    assert!(!is_blue(ui_px(&mut h, "Dash#strip", [8.0, 20.0])), "old site clear");
}

// ----------------------------------------------------- step/tick面 --

#[test]
fn probe_steps_advance_each_instance_once_per_step() {
    let mut h = host();
    settle(&mut h, 4);
    h.step(1.0 / 30.0).expect("step");
    h.step(1.0 / 30.0).expect("step");
    let st2 = h.app().scene_state();
    // N step = N Tick，逐实例独立推进（同一定义同一 handler，各自 SoA 行）。
    assert_eq!(quad_x(state_of(&st2, "Dash/p1")), 8.0);
    assert_eq!(quad_x(state_of(&st2, "Dash/p2")), 8.0);
    match &state_of(&st2, "Dash/m3").elems[0] {
        SceneElemState::Cylinder { tx, .. } => assert_eq!(*tx, 2.0),
        _ => panic!("cylinder"),
    }
    assert_eq!(h.app().ui.queue_len(), 0, "steps never enqueue UI events");
    // 纯重绘不代 Tick：再 frame 后状态不变。
    let r = h.frame(None).expect("frame");
    assert_eq!(r.dispatched, 0);
    let st3 = h.app().scene_state();
    assert_eq!(quad_x(state_of(&st3, "Dash/p1")), 8.0);
    // 3D 像素随步进真实变化：rod 平移 → 合成扫描线变化（呈现不滞留旧帧）；
    // 绿色 rod 在场且位置随步移动。
    let scan = |h: &mut MixedRegionsHost| -> Vec<[u8; 4]> {
        let points: Vec<[f32; 2]> = (10..70).step_by(2).map(|x| [(x) as f32, 40.0]).collect();
        let b = h.widget_bounds("Dash/m3#dial").expect("dial bounds");
        h.rt
            .probe_pixels(
                h.ui,
                &points
                    .iter()
                    .map(|p| [b.x + p[0], b.y + p[1]])
                    .collect::<Vec<_>>(),
            )
            .expect("scanline")
    };
    let before = scan(&mut h);
    assert!(
        before.iter().any(|q| q[1] > 150 && u16::from(q[1]) > u16::from(q[0]) + 50),
        "green rod visible before step"
    );
    h.step(1.0 / 30.0).expect("step");
    settle(&mut h, 1);
    let after = scan(&mut h);
    assert_ne!(before, after, "step-driven 3D pixels must refresh");
}

// ------------------------------------------- conditional mount/unmount --

#[test]
fn probe_conditional_unmount_releases_and_rejects_stale() {
    let mut h = host();
    settle(&mut h, 4);
    // 建立独立状态：p1 ×2（x=24）。
    for _ in 0..2 {
        let r = h.widget_bounds("Dash/p1#arc").expect("p1 bounds");
        h.pointer_pressed(center(r)).expect("press");
        h.pointer_released();
        settle(&mut h, 2);
    }
    let old_epoch = h.region_epoch("Dash/p1#arc").expect("epoch");
    let targets_before = h.rt.live_target_slots();
    let bytes_before = h.rt.owned_target_bytes();
    // 条件隐藏：撤呈现（portal+view+surface 释放；实例行/状态保留）。
    click_widget(&mut h, "Dash#toggle");
    settle(&mut h, 2);
    // 隐藏期间伪造一条旧代次区域事件（区域已卸载、代次已失效；host 不可
    // 点击隐藏区——app 侧直入队代表在途/离场来源）。
    assert!(h.app().enqueue_region_click_at("Dash/p1", "arc", old_epoch));
    // 下一次 drain 即具名拒绝（不延进入到新代次/新呈现）。
    let r = h.frame(None).expect("drain while hidden");
    assert_eq!(
        r.stale_rejected, 1,
        "old-epoch region event must be named-rejected while unmounted"
    );
    assert_eq!(r.dispatched, 0, "stale event must not dispatch");
    let keys = h.active_keys();
    assert!(!keys.iter().any(|k| k.contains("p1") || k.contains("p2")));
    assert!(h.region_epoch("Dash/p1#arc").is_none(), "unmounted");
    // 释放两套 target/content，释放的名额由 p3 复用（预算收口仍可用）：
    // 5 = UI + strip + m3 + p1 + p2 → 4 = UI + strip + m3 + p3。
    assert_eq!(
        h.rt.live_target_slots(),
        targets_before - 1,
        "two released, one reused"
    );
    assert!(h.rt.owned_target_bytes() < bytes_before, "target bytes freed");
    assert!(keys.iter().any(|k| k == "Dash/p3#arc"));
    // 恢复显示：p1 重挂＝新代次；状态保留；新代次点击正常生效。
    click_widget(&mut h, "Dash#toggle");
    settle(&mut h, 2);
    let new_epoch = h.region_epoch("Dash/p1#arc").expect("remounted");
    assert_ne!(new_epoch, old_epoch, "remount = new generation");
    let st = h.app().scene_state();
    assert_eq!(
        quad_x(state_of(&st, "Dash/p1")),
        24.0,
        "state preserved across unmount/remount"
    );
    // 新代次点击正常生效（重挂后恢复正确）。
    let r = h.widget_bounds("Dash/p1#arc").expect("p1 bounds");
    h.pointer_pressed(center(r)).expect("press");
    h.pointer_released();
    settle(&mut h, 2);
    let st = h.app().scene_state();
    assert_eq!(quad_x(state_of(&st, "Dash/p1")), 36.0, "post-remount click works");
}

// ------------------------------------------------------ schema isolation --

#[test]
fn probe_v1_and_v2_schemas_reject_each_other() {
    // v2 入口拒 v1；v1 入口拒 v2；048 入口拒 v2（具名、schema 界）。
    assert!(parse_mixed_regions_artifact(V1_UI_JSON).is_err());
    assert!(parse_mixed_artifact(UI_JSON).is_err());
    assert!(parse_artifact(UI_JSON).is_err());
    // v2 自身解析成功且 048 基础面（实例 ≤8）不变：4 实例 + root 在界内。
    let a = parse_mixed_regions_artifact(UI_JSON).expect("v2 parses");
    assert_eq!(a.instances.len() + 1, 5);
}
