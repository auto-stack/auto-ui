//! PLAN-054 T-05（driver 侧 flow 永久门，离屏确定性、单实例 GPU、
//! --test-threads=1）：改名/换序等价路由（V-S01）、bool 高槽修正定向回归
//! （AG021 反例闭面）、陈旧 Run/区域代次与队列上界具名拒绝、20 轮挂卸
//! 寿命零资源增长（V-S02/06）、步进逐实例推进与 2D 步驱像素刷新。
//! Run: cargo test -p autographics-auto-driver --test mixed_regions_lifecycle_flow
///       -- --test-threads=1
use agp::core::composition::Rect;
use agp::renderer::GpuDevice;
use autographics_auto_driver::node_mixed_regions::{MixedRegionsUiApp, parse_mixed_regions_artifact};
use autographics_auto_driver::node_mixed_regions_host::MixedRegionsHost;

const SWAP_UI: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-panel-swapped/ui.json"
);
const SWAP_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-panel-swapped/world.at"
);
const BOOL_UI: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-bool-slots/ui.json"
);
const BOOL_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-bool-slots/world.at"
);

const LOGICAL: [f32; 2] = [480.0, 560.0];

fn host(ui: &str, world: &str, name: &str) -> MixedRegionsHost {
    let gpu = GpuDevice::new().expect("gpu");
    let app = MixedRegionsUiApp::new(ui, world, name).expect("app");
    MixedRegionsHost::new(gpu, app, LOGICAL, None).expect("host")
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

fn quad_x_of(h: &mut MixedRegionsHost, path: &str) -> f32 {
    let st = h.app().scene_state();
    let r = st.iter().find(|s| s.path == path).unwrap_or_else(|| panic!("state {path}"));
    match &r.elems[0] {
        autographics_auto_driver::node_mixed::SceneElemState::Quad { x, .. } => *x,
        autographics_auto_driver::node_mixed::SceneElemState::Cylinder { tx, .. } => *tx,
        _ => panic!("kind"),
    }
}

// ------------------------------------------- rename / order swap (V-S01) --

#[test]
fn flow_renamed_swapped_layout_routes_identically() {
    let mut h = host(SWAP_UI, SWAP_WORLD, "plan054-flow-swap");
    settle(&mut h, 4);
    // 全新命名（Panel/Dial/Bar/band/ring/shelf；3D 区域声明在前）——目录
    // 驱动装配无固定名：4 活动区域恰为全部声明面。
    let mut keys = h.active_keys();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "Panel#shelf".to_string(),
            "Panel/b1#band".to_string(),
            "Panel/b2#band".to_string(),
            "Panel/d1#ring".to_string(),
        ]
    );
    // 同定义两实例 b1 ×2 / b2 ×1（Nudge +9）→ 18 vs 9，像素分开。
    for _ in 0..2 {
        let r = h.widget_bounds("Panel/b1#band").expect("b1 bounds");
        h.pointer_pressed(center(r)).expect("press");
        h.pointer_released();
        settle(&mut h, 2);
    }
    let r = h.widget_bounds("Panel/b2#band").expect("b2 bounds");
    h.pointer_pressed(center(r)).expect("press");
    h.pointer_released();
    settle(&mut h, 2);
    assert_eq!(quad_x_of(&mut h, "Panel/b1"), 18.0);
    assert_eq!(quad_x_of(&mut h, "Panel/b2"), 9.0);
    // UI 按钮 → 根状态 → root 区域（bump +8；shelf 区域点击 +50 互斥）。
    click_widget(&mut h, "Panel#bump");
    assert_eq!(texts(&mut h)[0], "1");
    click_widget(&mut h, "Panel#shelf");
    assert_eq!(texts(&mut h)[0], "51");
}

// --------------------------------------- bool high-slot fix (AG021 例) --

#[test]
fn flow_bool_text_slots_use_half_buffer_after_fix() {
    // 生成面：bool 槽 2/3 必须写 b 半缓冲（修正前错写缓冲 A 覆盖槽 0/1）。
    assert!(
        BOOL_WORLD.contains("bools_ui_text_b_0[((0*64)+0)]=0x74"),
        "bool slot 2 must write the b half-buffer"
    );
    assert!(
        BOOL_WORLD.contains("if r_flag==1 {"),
        "bool branch must gate on the bool field"
    );
    // 消费面：初始 true/false 正确读回；区域点击翻转后 len/字节一致。
    let mut h = host(BOOL_UI, BOOL_WORLD, "plan054-flow-bool");
    settle(&mut h, 4);
    let t = texts(&mut h);
    assert_eq!(t[0], "42");
    assert_eq!(t[1], "const");
    assert_eq!(t[2], "true", "bool at slot 2 reads b-buffer");
    assert_eq!(t[3], "false", "bool at slot 3 reads b-buffer");
    // pad 区域点击 → 双 bool 翻转；len 与字节保持一致（修正前槽 2 写
    // 缓冲 A 覆盖 t0 且 len 错报）。
    click_widget(&mut h, "Bools#pad");
    let t = texts(&mut h);
    assert_eq!(t[0], "42", "slot 0 must stay untouched by bool writes");
    assert_eq!(t[2], "false");
    assert_eq!(t[3], "true");
}

// --------------------------------- stale run / epochs / queue bounds --

#[test]
fn flow_stale_run_generation_and_queue_cap_are_named_rejections() {
    let mut h = host(SWAP_UI, SWAP_WORLD, "plan054-flow-stale");
    settle(&mut h, 4);
    // 陈旧 Run 代次：入队后 start_run（代次+1）→ drain 具名拒绝。
    let epoch = h.region_epoch("Panel/b1#band").expect("epoch");
    assert!(h.app().enqueue_region_click_at("Panel/b1", "band", epoch));
    h.app().start_run().expect("new run");
    let r = h.frame(None).expect("drain");
    assert!(r.stale_rejected >= 1, "old-run region event must reject");
    // 队列上界 64：第 65 个事件具名丢弃（区域事件同界）。
    let epoch = h.region_epoch("Panel/b1#band").expect("epoch");
    for _ in 0..64 {
        assert!(h.app().enqueue_region_click_at("Panel/b1", "band", epoch));
    }
    assert!(!h.app().enqueue_region_click_at("Panel/b1", "band", epoch));
    assert_eq!(h.app().ui.dropped_overflow, 1, "65th event named-dropped");
    let r = h.frame(None).expect("drain 64");
    assert_eq!(r.dispatched, 64, "in-budget events all dispatch");
}

// ------------------------------------- 20-round lifecycle (V-S02/06) --

#[test]
fn flow_twenty_round_lifecycle_has_zero_resource_growth() {
    let mut h = host(SWAP_UI, SWAP_WORLD, "plan054-flow-life");
    settle(&mut h, 4);
    // 2D 步驱像素真实刷新（x=0 起：pip 覆盖 0..18；一步后 5..23——
    // 点 (3,22) 由内变外，呈现不滞留旧帧）。
    let px = |h: &mut MixedRegionsHost, key: &str, local: f32| -> [u8; 4] {
        let b = h.widget_bounds(key).expect(key);
        h.rt
            .probe_pixel(h.ui, [b.x + local, b.y + 22.0])
            .expect("pixel")
    };
    let before = px(&mut h, "Panel/b1#band", 3.0);
    assert!(before[0] > 150 && u16::from(before[0]) > u16::from(before[1]) + 50,
        "pip visible at start");
    h.step(1.0 / 30.0).expect("step");
    settle(&mut h, 1);
    let after = px(&mut h, "Panel/b1#band", 3.0);
    assert_ne!(before, after, "step-driven 2D pixels must refresh");
    let mut steps = 1u64;
    let steady_slots = h.rt.live_target_slots();
    let steady_bytes = h.rt.owned_target_bytes();
    for round in 0..20 {
        // 挂载轮换：toggle（卸 b1/b2 → 2 活动环）→ step ×2（环上 Tick）
        // → toggle（重挂新代次）。
        click_widget(&mut h, "Panel#toggle");
        settle(&mut h, 2);
        assert_eq!(h.active_count(), 2, "round {round}: hidden face");
        assert!(h.region_epoch("Panel/b1#band").is_none());
        assert!(h.mount_errors.is_empty(), "round {round}");
        h.step(1.0 / 30.0).expect("step");
        h.step(1.0 / 30.0).expect("step");
        steps += 2;
        // 环上状态继续推进（d1 Tick +1/步；卸载区状态 owner 不受影响）。
        assert_eq!(quad_x_of(&mut h, "Panel/d1"), steps as f32);
        click_widget(&mut h, "Panel#toggle");
        settle(&mut h, 2);
        assert_eq!(h.active_count(), 4, "round {round}: remounted face");
        assert!(h.mount_errors.is_empty());
        // 资源收支闭合：targets/bytes 回到稳态（释放＋重挂零增长）。
        assert_eq!(h.rt.live_target_slots(), steady_slots, "round {round}");
        assert_eq!(h.rt.owned_target_bytes(), steady_bytes, "round {round}");
    }
    // 20 轮后步驱状态一致：b1/b2 状态未被挂卸重置（21 步 ×5/步）。
    assert_eq!(quad_x_of(&mut h, "Panel/b1"), (steps * 5) as f32);
    assert_eq!(quad_x_of(&mut h, "Panel/b2"), (steps * 5) as f32);
}

// -------------------------------------------- schema admission unions --

#[test]
fn flow_v2_parse_gates_frozen_capacities() {
    // v2 消费入口冻结容量复检（执行前具名拒绝；与 pilot 生成面同数）。
    let a = parse_mixed_regions_artifact(SWAP_UI).expect("swap artifact parses");
    assert_eq!(a.instances.len() + 1, 4, "instances <= 8 face");
    // 实例表声明先序：3D 组件在前（换序面）。
    assert_eq!(a.instances[0].path, "Panel/d1");
    assert_eq!(a.instances[1].path, "Panel/b1");
}
