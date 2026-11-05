//! PLAN-049 T-04（混合宿主公共链，离屏验证）：装配/布局驱动挂载/命中
//! 路由/resize 重挂载/一次更新与共享 owner。窗口与离屏测试共用
//! [`MixedHost`] 单一装配（无第二份装配代码）。GPU 单实例。
//! Run: cargo test -p autographics-auto-driver --test node_mixed_views_host
use agp::renderer::GpuDevice;
///       -- --test-threads=1
use autographics_auto_driver::node_mixed::MixedUiApp;
use autographics_auto_driver::node_mixed_host::MixedHost;

const UI_JSON: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/ui.json");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/world.at");

const BUTTON: &str = "Main#inc";
const REG2: &str = "Main#reg2";
const REG3: &str = "Main#reg3";
const LOGICAL: [f32; 2] = [420.0, 420.0];

fn host() -> MixedHost {
    let gpu = GpuDevice::new().expect("gpu");
    let app = MixedUiApp::new(UI_JSON, WORLD, "plan049-host").expect("mixed app");
    MixedHost::new(gpu, app, LOGICAL, None).expect("mixed host")
}

fn center(r: agp::core::composition::Rect) -> [f32; 2] {
    [r.x + r.w / 2.0, r.y + r.h / 2.0]
}

fn text0(host: &mut MixedHost) -> String {
    host.app().ui.texts(0).get(0).cloned().unwrap_or_default()
}

#[test]
fn host_routes_button_and_both_regions_with_state_and_pixel_oracles() {
    let mut h = host();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    // 实际布局 bounds（真实 layout，非固定点）。
    let button = h.widget_bounds(BUTTON).expect("button bounds");
    let reg2 = h.widget_bounds(REG2).expect("reg2 bounds");
    let reg3 = h.widget_bounds(REG3).expect("reg3 bounds");

    // 按钮 → Auto count=1；2D quad x=8、3D cylinder x=2（同一状态，多呈现）。
    h.pointer_pressed(center(button)).expect("press button");
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame after button");
    }
    assert_eq!(text0(&mut h), "1", "button click must dispatch once");

    // 2D 像素对照：新位置红、旧址清（真实二维移动，非计数回读）。
    let ui = h.ui;
    // quad x=8 占 8..32：新址中心 20 红、槽外 2 已清退。
    let at_new = h.rt.probe_pixel(ui, [reg2.x + 20.0, reg2.y + 32.0]);
    let at_old = h.rt.probe_pixel(ui, [reg2.x + 2.0, reg2.y + 32.0]);
    let red = |p: Option<[u8; 4]>| {
        p.is_some_and(|q| u16::from(q[0]) > 150 && u16::from(q[0]) > u16::from(q[1]) + 50)
    };
    assert!(red(at_new), "quad must be red at x=8: {at_new:?}");
    assert!(!red(at_old), "old spot must be cleared: {at_old:?}");

    // 3D 像素对照：x=2 → 投影偏移 2*13.58≈27px（fov45/dist8/aspect1.78，
    // px_per_world_x=160/(2*tan22.5°*8*aspect)），半径 1≈13.6px → 圆柱心
    // 在中心+27 处绿。
    let at_cyl = h.rt.probe_pixel(ui, [reg3.x + 80.0 + 27.0, reg3.y + 45.0]);
    let green = |p: Option<[u8; 4]>| {
        p.is_some_and(|q| u16::from(q[1]) > 140 && u16::from(q[1]) > u16::from(q[0]) + 30)
    };
    assert!(
        green(at_cyl),
        "cylinder body must be green at its projected x=2: {at_cyl:?}"
    );

    // 2D 区域点击回传（区域内 → child → Auto → UI 文字）；互斥：count 不动。
    h.pointer_pressed(center(reg2)).expect("press reg2");
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame after reg2");
    }
    let texts = h.app().ui.texts(0).to_vec();
    assert_eq!(
        texts.get(2).map(String::as_str),
        Some("1"),
        "reg2 click feedback"
    );
    assert_eq!(
        texts.get(0).map(String::as_str),
        Some("1"),
        "region input must not reach UI button"
    );

    // 3D 区域点击回传。
    h.pointer_pressed(center(reg3)).expect("press reg3");
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame after reg3");
    }
    let texts = h.app().ui.texts(0).to_vec();
    assert_eq!(
        texts.get(3).map(String::as_str),
        Some("1"),
        "reg3 click feedback"
    );

    // 状态推进后的 3D 像素：再按一次按钮（cylinder x=4 离轴），中心变天空。
    h.pointer_pressed(center(button)).expect("press button 2");
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame after button 2");
    }
    assert_eq!(text0(&mut h), "2");
    let at_center = h.rt.probe_pixel(ui, [reg3.x + 80.0, reg3.y + 45.0]);
    assert!(
        !green(at_center),
        "cylinder must leave center at x=6: {at_center:?}"
    );
}

#[test]
fn resize_rebuilds_surface_and_keeps_real_layout_mapping() {
    let mut h = host();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    // resize：UI 逻辑尺寸＋GPU target 事务重建（本夹具子元素为固定尺寸、
    // 左上对齐——区域 rect 对 resize 不敏感是布局事实而非挂载失效；挂载
    // 一致性由重读的真实 bounds 中心点击承接）。
    h.resize([640.0, 640.0]).expect("resize");
    for _ in 0..4 {
        h.frame(None).expect("frame after resize");
    }
    assert_eq!(h.logical(), [640.0, 640.0]);
    let reg2 = h.widget_bounds(REG2).expect("reg2 after resize");
    let report = h
        .pointer_pressed(center(reg2))
        .expect("press reg2 after resize");
    assert_eq!(
        report.region,
        Some("reg2"),
        "routing must follow re-read layout"
    );
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    let texts = h.app().ui.texts(0).to_vec();
    assert_eq!(
        texts.get(2).map(String::as_str),
        Some("1"),
        "post-resize region click dispatches"
    );
}

#[test]
fn dynamic_layout_growth_moves_mounts_and_input_mapping() {
    // 真实重挂载 oracle：2D 区域点击回传同时长高 spacer（Field 绑定容器
    // 高度）→ 区域 rect 下移 → 挂载几何 SetMount 跟随 → 旧中心不再路由
    // 到区域（归 UI），新中心路由到区域。
    let mut h = host();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    let old_rect = h.widget_bounds(REG2).expect("reg2 before");
    let old_top = [old_rect.x + old_rect.w / 2.0, old_rect.y + 10.0];
    // 第一次区域点击：路由 + spacer +40。
    h.pointer_pressed(center(old_rect))
        .expect("press reg2 (moves layout)");
    h.pointer_released();
    for _ in 0..4 {
        h.frame(None).expect("frame (relayout)");
    }
    let new_bounds = h.widget_bounds(REG2).expect("reg2 after growth");
    assert!(
        new_bounds.y >= old_rect.y + 39.0,
        "region must move down by the spacer growth: old={old_rect:?} new={new_bounds:?}"
    );
    // 旧矩形顶部（原在区域内）现位于新区域上缘之外：路由到 UI。
    let stale = h.pointer_pressed(old_top).expect("press stale top");
    assert_eq!(
        stale.region, None,
        "old top must fall outside the moved region"
    );
    h.pointer_released();
    // 新中心：路由到区域并派发（clicks2 第二次）。
    let fresh = h
        .pointer_pressed(center(new_bounds))
        .expect("press reg2 (new center)");
    assert_eq!(
        fresh.region,
        Some("reg2"),
        "new center must route to the moved region"
    );
    h.pointer_released();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    let texts = h.app().ui.texts(0).to_vec();
    assert_eq!(
        texts.get(2).map(String::as_str),
        Some("2"),
        "both region clicks dispatched"
    );
    // 互斥：两次区域点击不动 count。
    assert_eq!(
        texts.get(0).map(String::as_str),
        Some("0"),
        "region clicks must not reach UI button"
    );
}

#[test]
fn idle_frames_cause_zero_session_callbacks() {
    let mut h = host();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    let before = h.app().ui.callbacks;
    for _ in 0..60 {
        h.frame(None).expect("idle frame");
    }
    let after = h.app().ui.callbacks;
    assert_eq!(before, after, "idle redraws must not tick the session");
}

#[test]
fn explicit_step_is_single_update_and_rendering_never_ticks() {
    let mut h = host();
    for _ in 0..3 {
        h.frame(None).expect("frame");
    }
    let before = h.app().ui.callbacks;
    h.step(1.0 / 60.0).expect("step");
    let after_step = h.app().ui.callbacks;
    assert!(after_step > before, "step must reach the session");
    // 步后纯重绘：无再派发/再接受（一次更新；呈现者零 VM 访问）。
    for _ in 0..10 {
        let r = h.frame(None).expect("frame after step");
        assert_eq!(r.dispatched, 0);
    }
    assert_eq!(
        h.app().ui.callbacks,
        after_step,
        "rendering after an explicit step must not re-enter the session"
    );
}
