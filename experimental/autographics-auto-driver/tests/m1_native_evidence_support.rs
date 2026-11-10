//! PLAN-071 M1 本机剪裁验证与只读原生取证支撑（永久门，离屏确定性、
//! 单实例 GPU、--test-threads=1）。
//!
//! 全部像素断言读**真实混合最终合成**（`MixedRegionsHost` 把区域门户按
//! clip 合成进 UI 窗口面 target——`capture/probe_pixel(s)(host.ui)` 即所
//! 见即所呈），ROI 由独立来源计算：`widget_bounds` 实时布局 rect ＋ fixture
//! 声明几何比例——不用 clip 实现本身算 expected。反控打在生产缝隙上：
//! 门户 mount 漂移（`set_portal_mount`，clip 跟随错误 rect）与区域内容
//! 清空（`with_game2d_mut`，`present_region` 同缝），oracle 必须红，修复
//! 同副本回绿。证据面（V-03/04/05）走 `m1_evidence` schema 严格校验。
//! Run: cargo test -p autographics-auto-driver --test m1_native_evidence_support
//!       -- --test-threads=1
use agp::core::composition::{MountKind, Rect};
use agp::renderer::GpuDevice;
use autographics_auto_driver::m1_evidence::{
    self, EvidenceRecord, MIXED_MANUAL_CHECKLIST, M1_EVIDENCE_SCHEMA,
};
use autographics_auto_driver::node_mixed_regions::MixedRegionsUiApp;
use autographics_auto_driver::node_mixed_regions_host::MixedRegionsHost;

const CLIP_UI: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-m1-clip-forensics/ui.json"
);
const CLIP_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-m1-clip-forensics/world.at"
);

const LOGICAL: [f32; 2] = [480.0, 560.0];
const P1: &str = "Dash/p1#arc";
const P2: &str = "Dash/p2#arc";
const M3: &str = "Dash/m3#dial";
const STRIP: &str = "Dash#strip";

fn host(name: &str) -> MixedRegionsHost {
    let gpu = GpuDevice::new().expect("gpu");
    let app = MixedRegionsUiApp::new(CLIP_UI, CLIP_WORLD, name).expect("v2 app");
    MixedRegionsHost::new(gpu, app, LOGICAL, None).expect("host")
}

fn settle(h: &mut MixedRegionsHost, n: usize) {
    for _ in 0..n {
        h.frame(None).expect("frame");
    }
}

fn center(r: &Rect) -> [f32; 2] {
    [r.x + r.w / 2.0, r.y + r.h / 2.0]
}

fn bounds(h: &mut MixedRegionsHost, key: &str) -> Rect {
    h.widget_bounds(key).unwrap_or_else(|| panic!("bounds {key}"))
}

fn click(h: &mut MixedRegionsHost, at: [f32; 2]) -> (Option<String>, bool) {
    let report = h.pointer_pressed(at).expect("press");
    let released = h.pointer_released().expect("release");
    settle(h, 3);
    (
        report.region,
        released.delivered.is_some(),
    )
}

fn texts(h: &mut MixedRegionsHost) -> Vec<String> {
    h.app().ui.texts(0).to_vec()
}

fn quad_x_of(h: &mut MixedRegionsHost, path: &str) -> f32 {
    let st = h.app().scene_state();
    let r = st
        .iter()
        .find(|s| s.path == path)
        .unwrap_or_else(|| panic!("state {path}"));
    match &r.elems[0] {
        autographics_auto_driver::node_mixed::SceneElemState::Quad { x, .. } => *x,
        autographics_auto_driver::node_mixed::SceneElemState::Cylinder { tx, .. } => *tx,
        _ => panic!("kind"),
    }
}

fn px(h: &mut MixedRegionsHost, at: [f32; 2]) -> [u8; 4] {
    h.rt.probe_pixel(h.ui, at).expect("pixel readback")
}

fn scanline(
    h: &mut MixedRegionsHost,
    rect: &Rect,
    fx: std::ops::Range<f32>,
    step: f32,
    fy: f32,
) -> Vec<[u8; 4]> {
    let points: Vec<[f32; 2]> = std::iter::successors(Some(fx.start), |v| Some(v + step))
        .take_while(|v| *v <= fx.end)
        .map(|f| [rect.x + rect.w * f, rect.y + rect.h * fy])
        .collect();
    h.rt.probe_pixels(h.ui, &points).expect("scanline")
}

/// 高对比边界识别色（fixture edge quads 声明 (255,0,255)；3D 有光照衰减，
/// 只认通道序）。红点/蓝点等业务几何不得满足。
fn is_edge_magenta(q: &[u8; 4]) -> bool {
    let (r, g, b) = (u16::from(q[0]), u16::from(q[1]), u16::from(q[2]));
    r > 80 && b > 80 && r > g + 30 && b > g + 30
}

// ============================== C1: composed app-level clip (V-01) =======

#[test]
fn m1_clip_2d_region_boundary_content_stays_inside() {
    let mut h = host("plan071-clip-2d");
    settle(&mut h, 4);
    let rect = bounds(&mut h, STRIP);
    // 边界内：edge quad (x=104..128 of 120 → fx 0.867..，截断于 1.0)。
    let inside = scanline(&mut h, &rect, 0.87..0.995, 0.01, 0.79);
    assert!(
        inside.iter().any(is_edge_magenta),
        "edge quad must be visible inside the strip near the boundary: {inside:?}"
    );
    // 越界（右缘外到 1.5 宽）：与远端干净参照同值且无边界识别色。
    let bg_ref = px(&mut h, [470.0, rect.y + rect.h * 0.79]);
    assert!(
        !is_edge_magenta(&bg_ref),
        "reference point must be clean UI background: {bg_ref:?}"
    );
    let outside = scanline(&mut h, &rect, 1.02..1.5, 0.02, 0.79);
    for (i, q) in outside.iter().enumerate() {
        assert!(
            q == &bg_ref,
            "outside pixel {i} {q:?} != background {bg_ref:?}: region leaked"
        );
    }
}

#[test]
fn m1_clip_3d_region_boundary_content_stays_inside() {
    let mut h = host("plan071-clip-3d");
    settle(&mut h, 4);
    let rect = bounds(&mut h, M3);
    // 近边界圆柱（x=3, r=1 → 2..4，投影半宽 ≈3.31 → fx 0.80..截断于 1.0）。
    let inside = scanline(&mut h, &rect, 0.82..0.98, 0.01, 0.5);
    assert!(
        inside.iter().any(is_edge_magenta),
        "near-edge 3D cylinder must be visible inside the dial: {inside:?}"
    );
    let bg_ref = px(&mut h, [470.0, rect.y + rect.h * 0.5]);
    assert!(!is_edge_magenta(&bg_ref), "ref must be clean: {bg_ref:?}");
    let outside = scanline(&mut h, &rect, 1.02..1.5, 0.02, 0.5);
    for (i, q) in outside.iter().enumerate() {
        assert!(
            q == &bg_ref,
            "outside pixel {i} {q:?} != background {bg_ref:?}: 3D region leaked"
        );
    }
}

#[test]
fn m1_clip_mount_drift_is_caught_and_restore_recovers() {
    let mut h = host("plan071-clip-drift");
    settle(&mut h, 4);
    let rect = bounds(&mut h, STRIP);
    // 反控 A：生产缝隙上的真实故障——门户 mount 漂移（stale rect 一键三
    // 跟：offset/scale/clip 同随），区域纹理越过真实 widget rect。
    let surface = h.region_surface(STRIP).expect("strip surface");
    let portal = h
        .rt
        .graph
        .portals()
        .iter()
        .find(|p| p.desc.child == surface)
        .expect("strip portal")
        .id;
    let drifted = MountKind::Flat2d {
        offset: [rect.x, rect.y],
        scale: [rect.w * 2.0 / 120.0, rect.h / 48.0],
        clip: Rect::at(rect.x, rect.y, rect.w * 2.0, rect.h),
    };
    h.rt.set_portal_mount(portal, drifted).expect("drift");
    settle(&mut h, 2);
    // C1 oracle 在该副本上必须红：右缘外出现边界识别色（泄漏）。
    let leaked = scanline(&mut h, &rect, 1.02..1.9, 0.02, 0.79);
    assert!(
        leaked.iter().any(is_edge_magenta),
        "oracle must catch the drifted mount (leak beyond the region rect): {leaked:?}"
    );
    // 同副本恢复：精确修复（真实 rect）回绿。
    let restored = MountKind::Flat2d {
        offset: [rect.x, rect.y],
        scale: [rect.w / 120.0, rect.h / 48.0],
        clip: Rect::at(rect.x, rect.y, rect.w, rect.h),
    };
    h.rt.set_portal_mount(portal, restored).expect("restore");
    settle(&mut h, 2);
    let bg_ref = px(&mut h, [470.0, rect.y + rect.h * 0.79]);
    let outside = scanline(&mut h, &rect, 1.02..1.9, 0.02, 0.79);
    for (i, q) in outside.iter().enumerate() {
        assert!(
            q == &bg_ref,
            "restored pixel {i} {q:?} != background {bg_ref:?}"
        );
    }
    assert!(h.mount_errors.is_empty());
}

#[test]
fn m1_clip_missing_interior_content_is_caught() {
    let mut h = host("plan071-clip-missing");
    settle(&mut h, 4);
    let rect = bounds(&mut h, STRIP);
    let inside = scanline(&mut h, &rect, 0.87..0.995, 0.01, 0.79);
    assert!(
        inside.iter().any(is_edge_magenta),
        "positive control: edge quad present before the fault"
    );
    // 反控 B：区域内容清空（同 present_region 命令面缝隙）→ 内部缺画
    // 必须红。
    let surface = h.region_surface(STRIP).expect("strip surface");
    h.rt.with_game2d_mut(surface, |g| g.commands = Vec::new());
    settle(&mut h, 2);
    let cleared = scanline(&mut h, &rect, 0.87..0.995, 0.01, 0.79);
    assert!(
        !cleared.iter().any(is_edge_magenta),
        "oracle must catch missing interior content: {cleared:?}"
    );
    // 业务状态变化（真实事件路径）重建呈现 → 回绿（幂等恢复）。
    let bump = bounds(&mut h, "Dash#bump");
    click(&mut h, center(&bump));
    let restored = scanline(&mut h, &rect, 0.87..0.995, 0.01, 0.79);
    assert!(
        restored.iter().any(is_edge_magenta),
        "edge quad must be re-presented after a real state change: {restored:?}"
    );
}

// ============================== C2: boundary routing (V-02) ==============

#[test]
fn m1_boundary_clicks_route_only_the_hit_receiver() {
    let mut h = host("plan071-route-boundary");
    settle(&mut h, 4);
    let strip = bounds(&mut h, STRIP);
    let cb0 = h.app().ui.callbacks;
    assert_eq!(quad_x_of(&mut h, "Dash/p1"), 72.0, "seeded near-edge dot");
    // 区域内右缘一次点击 → 只推进该 receiver（strip：count +100）。
    let (region, delivered) = click(
        &mut h,
        [strip.x + strip.w - 2.0, strip.y + strip.h * 0.79],
    );
    assert_eq!(region.as_deref(), Some("Dash#strip"), "strip hit identity");
    assert!(delivered);
    assert_eq!(texts(&mut h)[0], "100", "StripHit dispatched exactly once");
    assert_eq!(quad_x_of(&mut h, "Dash/p1"), 72.0, "p1 untouched");
    assert_eq!(quad_x_of(&mut h, "Dash/p2"), 72.0, "p2 untouched");
    assert_eq!(quad_x_of(&mut h, "Dash/m3"), 0.0, "m3 untouched");
    // 另一层（p1）：只推进 p1，strip/p2/m3 不变（同定义实例状态隔离）。
    let arc = bounds(&mut h, P1);
    let (region, _) = click(&mut h, center(&arc));
    assert_eq!(region.as_deref(), Some("Dash/p1#arc"));
    assert_eq!(quad_x_of(&mut h, "Dash/p1"), 84.0);
    assert_eq!(quad_x_of(&mut h, "Dash/p2"), 72.0, "same-def sibling isolated");
    assert_eq!(texts(&mut h)[0], "100");
    // p2 同点位推进（跨区域互不误派发）。
    let arc2 = bounds(&mut h, P2);
    let (region, _) = click(&mut h, center(&arc2));
    assert_eq!(region.as_deref(), Some("Dash/p2#arc"));
    assert_eq!(quad_x_of(&mut h, "Dash/p2"), 84.0);
    // 边界外 2px：同一行、不属任何区域 → 不派发（像素级边界判别）。
    let cb = h.app().ui.callbacks;
    let (region, _) = click(&mut h, [strip.x + strip.w + 2.0, strip.y + strip.h * 0.79]);
    assert!(region.is_none(), "outside click must not hit a region");
    assert_eq!(h.app().ui.callbacks, cb, "outside click must not dispatch");
    assert_eq!(texts(&mut h)[0], "100");
    // 控件消费：bump 只推进根 count，区域不误动。
    let bump = bounds(&mut h, "Dash#bump");
    let (region, _) = click(&mut h, center(&bump));
    assert!(region.is_none(), "button is not a region");
    assert_eq!(texts(&mut h)[0], "101");
    assert_eq!(quad_x_of(&mut h, "Dash/p1"), 84.0, "button click never hits regions");
    assert_eq!(quad_x_of(&mut h, "Dash/m3"), 0.0);
    assert!(h.app().ui.callbacks > cb0, "real Auto callbacks advanced");
    assert!(h.mount_errors.is_empty());
}

#[test]
fn m1_resize_remount_remeasure_bounds_and_reject_stale_epochs() {
    let mut h = host("plan071-remount");
    settle(&mut h, 4);
    let epoch_before = h.region_epoch(P1).expect("p1 mounted");
    let strip_before = bounds(&mut h, STRIP);
    // 卸载（真实条件隐藏）：p1 撤呈现，strip 上移——旧 ROI 不得错计。
    let toggle = bounds(&mut h, "Dash#toggle");
    click(&mut h, center(&toggle));
    assert!(h.region_epoch(P1).is_none(), "p1 unmounted");
    assert_eq!(h.active_count(), 3);
    let strip_after = bounds(&mut h, STRIP);
    assert!(
        strip_after.y < strip_before.y - 0.5,
        "strip must follow the moved layout: {strip_before:?} -> {strip_after:?}"
    );
    // 旧 ROI 点击：同一坐标现在真实落在 p2 上——不得误派发给 strip。
    let old_point = center(&strip_before);
    let (region, _) = click(&mut h, old_point);
    assert_eq!(region.as_deref(), Some("Dash/p2#arc"), "old ROI now hits p2");
    assert_eq!(texts(&mut h)[0], "0", "old-ROI click must not advance the strip");
    assert_eq!(quad_x_of(&mut h, "Dash/p2"), 84.0, "hit follows real layout");
    // 新 ROI（重新实测 bounds）命中 strip。
    let strip_now = bounds(&mut h, STRIP);
    let (region, _) = click(&mut h, center(&strip_now));
    assert_eq!(region.as_deref(), Some("Dash#strip"));
    assert_eq!(texts(&mut h)[0], "100");
    // 重挂：新挂载代次单调递增；布局恢复；旧代次事件 drain 前具名拒绝。
    click(&mut h, center(&toggle));
    let epoch_after = h.region_epoch(P1).expect("p1 remounted");
    assert!(epoch_after > epoch_before, "remount = new mount epoch");
    let strip_restored = bounds(&mut h, STRIP);
    assert!(
        (strip_restored.y - strip_before.y).abs() < 0.5,
        "layout restored: {strip_before:?} -> {strip_restored:?}"
    );
    let dispatched_before = h.app().ui.callbacks;
    assert!(h.app().enqueue_region_click_at("Dash/p1", "arc", epoch_before));
    let r = h.frame(None).expect("drain");
    assert!(r.stale_rejected >= 1, "stale epoch must be named-rejected");
    assert_eq!(h.app().ui.callbacks, dispatched_before, "stale event never dispatches");
    assert_eq!(quad_x_of(&mut h, "Dash/p1"), 72.0);
    // resize：逻辑尺寸事务生效；重新实测 bounds 命中照常（单档 offscreen 面；
    // 真窗单档 resize 走 --probe）。
    h.resize([420.0, 560.0]).expect("resize");
    settle(&mut h, 4);
    assert_eq!(h.logical(), [420.0, 560.0]);
    let strip_resized = bounds(&mut h, STRIP);
    let (region, _) = click(&mut h, center(&strip_resized));
    assert_eq!(region.as_deref(), Some("Dash#strip"), "hit follows re-measured bounds");
    assert_eq!(texts(&mut h)[0], "200");
    assert!(h.mount_errors.is_empty());
}

// ============================== C4/C5: evidence faces (V-03/04/05) =======

fn fixture_sha() -> String {
    agp::renderer::capture::sha256_hex(CLIP_WORLD.as_bytes())
}

#[test]
fn m1_evidence_record_binds_real_run_and_bad_copies_differ() {
    let mut h = host("plan071-evidence");
    settle(&mut h, 4);
    // 从一次真实 run 记录（只读公开面）。
    let strip = bounds(&mut h, STRIP);
    let mut rec = EvidenceRecord::new("mixed-regions", "posted-window");
    rec.bind_project(
        "mixed-artifact",
        serde_json::json!({
            "ui.json": agp::renderer::capture::sha256_hex(CLIP_UI.as_bytes()),
            "world.at": fixture_sha(),
        }),
    );
    rec.bind_candidate(serde_json::json!({
        "runner": agp::renderer::capture::sha256_hex(CLIP_UI.as_bytes()),
    }));
    let cb = h.app().ui.callbacks;
    rec.checkpoint(
        "t0",
        serde_json::json!({
            "count_text": texts(&mut h)[0],
            "callbacks": cb,
            "p1_x": quad_x_of(&mut h, "Dash/p1"),
        }),
    );
    click(&mut h, [strip.x + strip.w / 2.0, strip.y + strip.h / 2.0]);
    rec.checkpoint(
        "strip-click",
        serde_json::json!({
            "count_text": texts(&mut h)[0],
            "callbacks": h.app().ui.callbacks,
        }),
    );
    rec.set_tier_channel("posted");
    let record = rec.to_value();
    // 绑定的是真实状态（检查点与宿主实测一致）。
    assert_eq!(record["checkpoints"][0]["count_text"], serde_json::json!("0"));
    assert_eq!(record["checkpoints"][1]["count_text"], serde_json::json!("100"));
    assert_eq!(record["schema"], serde_json::json!(M1_EVIDENCE_SCHEMA));
    // posted 是控制层：永不 native-pass（具名 blocked）。
    let v = m1_evidence::validate(&record);
    assert_eq!(v.status, "pending");
    assert!(v.blocked.iter().any(|b| b.contains("control/binding tier")));
    // 副本损坏 = 内容身份不同（消费端据此具名拒绝）。
    let mutated = format!("{CLIP_WORLD}// tampered copy\n");
    assert_ne!(fixture_sha(), agp::renderer::capture::sha256_hex(mutated.as_bytes()));
    // 坏记录：schema 漂移 / 伪 auto_pass 具名拒绝。
    let mut bad = record.clone();
    bad["schema"] = serde_json::json!("autographics.m1-native-evidence/0");
    assert_eq!(m1_evidence::validate(&bad).status, "invalid");
    let mut fake = record.clone();
    fake["auto_pass"] = serde_json::json!(true);
    let v = m1_evidence::validate(&fake);
    assert_eq!(v.status, "invalid");
    assert!(v.blocked[0].contains("fabricated"));
}

#[test]
fn m1_observation_does_not_disturb_business_values_or_ticks() {
    // 同一业务序列跑两遍：记录器开（每次事件读只读面）与关——业务终态
    // 与派发次数完全一致；记录检查点绑定同值。
    let mut a = host("plan071-nonint-on");
    let mut b = host("plan071-nonint-off");
    settle(&mut a, 4);
    settle(&mut b, 4);
    let mut rec = EvidenceRecord::new("mixed-regions", "offscreen");
    rec.bind_project("mixed-artifact", serde_json::json!({ "world.at": fixture_sha() }));
    let record_around = |rec: &mut EvidenceRecord,
                             h: &mut MixedRegionsHost,
                             label: &str| {
        rec.checkpoint(
            label,
            serde_json::json!({
                "count_text": h.app().ui.texts(0)[0].clone(),
                "callbacks": h.app().ui.callbacks,
                "p1_x": quad_x_of(h, "Dash/p1"),
                "m3_tx": quad_x_of(h, "Dash/m3"),
                "frames_dispatched": h.frame(None).expect("frame").dispatched,
            }),
        );
    };
    let drive = |h: &mut MixedRegionsHost| {
        let strip = bounds(h, STRIP);
        click(h, [strip.x + strip.w / 2.0, strip.y + strip.h / 2.0]);
        let arc = bounds(h, P1);
        click(h, center(&arc));
        let bump = bounds(h, "Dash#bump");
        click(h, center(&bump));
        h.step(1.0 / 30.0).expect("step");
        settle(h, 2);
    };
    drive(&mut a);
    drive(&mut b);
    record_around(&mut rec, &mut a, "end");
    let end = |h: &mut MixedRegionsHost| -> (String, f32, f32, u64) {
        (
            h.app().ui.texts(0)[0].clone(),
            quad_x_of(h, "Dash/p1"),
            quad_x_of(h, "Dash/m3"),
            h.app().ui.callbacks,
        )
    };
    assert_eq!(end(&mut a), end(&mut b), "observation must not disturb business");
    let cp = rec.to_value()["checkpoints"][0].clone();
    assert_eq!(cp["count_text"], serde_json::json!(end(&mut a).0));
    assert_eq!(cp["p1_x"], serde_json::json!(end(&mut a).1));
    // 记录器故障不吞错误：fail 后状态 fail（不是 pending/pass）。
    rec.fail("timeout injected");
    assert_eq!(m1_evidence::validate(&rec.to_value()).status, "fail");
}

#[test]
fn m1_manual_checklist_is_frozen_and_never_auto_passes() {
    // N1..N6 检查表逐项冻结；手工记录（无观察人/单档）保持 pending——
    // 缺失项/缺观察人/单档全部具名 blocked。
    let mut rec = EvidenceRecord::new("mixed-regions", "native-manual");
    rec.bind_project("mixed-artifact", serde_json::json!({ "world.at": fixture_sha() }));
    rec.bind_candidate(serde_json::json!({ "window": "mixed_regions_window" }));
    rec.observe_window(2.0, [480.0, 560.0], [960, 1120]);
    // F-N4-01（PLAN-076）：N4 文本逐字冻结——2026-11-10 语义裁定（方案 b）
    // 后的可满足形态；任何措辞漂移（含回退旧"mouse leave cancels"歧义态）
    // 在此具名红。
    assert_eq!(
        MIXED_MANUAL_CHECKLIST[3].1,
        "press on empty UI space, leave the window and release outside: no button activates and no region click is dispatched",
        "N4 wording is frozen at the 2026-11-10 semantic ruling"
    );
    for (id, item) in MIXED_MANUAL_CHECKLIST {
        rec.observe_checklist(
            id,
            serde_json::json!({ "note": format!("observed counters land here: {item}") }),
        );
    }
    let v = m1_evidence::validate(&rec.to_value());
    assert_eq!(v.status, "pending");
    assert!(
        v.blocked.iter().any(|b| b.contains("human conclusion")),
        "each N item needs a human conclusion: {:?}",
        v.blocked
    );
    assert!(v.blocked.iter().any(|b| b.contains("second real DPI")));
    assert!(v.blocked.iter().any(|b| b.contains("observer")));
    // 逐项结论 + 观察人 + 第二真实档 = 唯一 native-pass 路径。
    for (id, _) in MIXED_MANUAL_CHECKLIST {
        rec.conclude_checklist(id, "pass");
    }
    rec.set_observer(Some("human"), Some("dual-scale session"));
    rec.record_scale_change(2.0, 1.25, "N6");
    let v = m1_evidence::validate(&rec.to_value());
    assert_eq!(v.status, "native-pass", "{:?}", v.blocked);
    // 任何一项 fail 即不闭。
    rec.conclude_checklist("N4", "fail");
    let v = m1_evidence::validate(&rec.to_value());
    assert_eq!(v.status, "pending");
    assert!(v.blocked.iter().any(|b| b.contains("N4")));
}

// ============================== PLAN-076: F-SCALE-02 session shape ========

#[test]
fn m1_manual_session_report_keeps_start_tier_and_real_scale_pairs() {
    // F-SCALE-02（PLAN-076）：mixed_regions_window --manual 的单次切档会话
    // 全形——开窗播种（resumed 读回 2.0）→ 真实 ScaleFactorChanged
    // (2.0→2.5，last_scale 跟踪) → 关窗读回 2.5（observe_window 合并幂等
    // 播种起始档后覆盖 window.* 实测字段）。AG026 实证旧实现（替换写法
    // ＋事件内晚读）产 scales_observed=[2.5] 且 scale_factor_changed=
    // [(2.5,2.5)]，validator 误判 single scale tier——本用例对旧实现必红。
    let mut rec = EvidenceRecord::new("mixed-regions", "native-manual");
    rec.bind_project(
        "mixed-artifact",
        serde_json::json!({ "world.at": fixture_sha() }),
    );
    rec.bind_candidate(serde_json::json!({ "window": "mixed_regions_window" }));
    rec.observe_window(2.0, [480.0, 560.0], [960, 1120]); // resumed() 播种
    rec.record_scale_change(2.0, 2.5, "manual"); // 真实事件对（push_scale_pair）
    rec.observe_window(2.5, [480.0, 560.0], [1200, 1400]); // 关窗读回（合并）
    let record = rec.to_value();
    assert_eq!(
        record["tier"]["scales_observed"],
        serde_json::json!([2.0, 2.5]),
        "single-change session keeps [start, end]"
    );
    assert_eq!(
        record["window"]["scale_factor_changed"],
        serde_json::json!([{ "before": 2.0, "after": 2.5, "at": "manual" }]),
        "pairs are the real transition, not the late (X, X) readback"
    );
    assert_eq!(record["window"]["scale"], serde_json::json!(2.5));
    assert_eq!(record["window"]["physical"], serde_json::json!([1200, 1400]));
    // validator 面起始档缺失即误拒：双档在列后 single-scale blocked 消失
    // （其余人工项仍 pending，不在此关闭手工门）。
    let v = m1_evidence::validate(&record);
    assert!(
        !v.blocked.iter().any(|b| b.contains("single scale tier")),
        "dual-scale session must not read as single tier: {:?}",
        v.blocked
    );
    assert_eq!(v.status, "pending", "manual gate still needs human conclusions");
    assert!(v.blocked.iter().any(|b| b.contains("human conclusion")));
}

// ============================== PLAN-076: F-N4-01 semantic ruling ==========

#[test]
fn m1_n4_empty_press_cancels_on_leave_and_region_press_dispatches_at_hit() {
    // N4（PLAN-076 语义裁定·方案 b）行为回归：空白 UI 区按下 → 离窗
    // （CursorLeft→cancel_all）→ 窗外释放（宿主不见真实释放；迟到释放无处
    // 投递）⇒ 无按钮激活、无区域点击入队。canonical 对照（054“命中即入
    // 队”，裁定保留）：strip 按下即派发，离窗取消只作用于在途捕获，不可
    // 撤回已派发交互。区分性：若派发改到 release-inside（被否决的方案 a）
    // 或 cancel 泄漏到已派发交互，两侧断言必红。
    let mut h = host("plan076-n4");
    settle(&mut h, 4);
    let strip = bounds(&mut h, STRIP);

    // 取消路径：空白区按下 → 离窗 → 窗外释放。
    let cb0 = h.app().ui.callbacks;
    let count0 = texts(&mut h)[0].clone();
    let empty = [strip.x + strip.w + 2.0, strip.y + strip.h * 0.79];
    let press = h.pointer_pressed(empty).expect("press on empty UI space");
    assert!(press.region.is_none(), "empty-space press hits no region");
    h.cancel_all(agp::core::composition::input::CancelReason::PointerLeftWindow);
    let late = h.pointer_released().expect("release outside (late arrival)");
    assert!(
        late.region.is_none() && late.delivered.is_none(),
        "cancelled press delivers nothing on release"
    );
    settle(&mut h, 3);
    assert_eq!(h.app().ui.callbacks, cb0, "no button activates");
    assert_eq!(texts(&mut h)[0], count0, "no region click is dispatched");

    // canonical 对照：区域命中即派发（按下时），离窗不可撤回。
    let press = h
        .pointer_pressed([strip.x + strip.w - 2.0, strip.y + strip.h * 0.79])
        .expect("press strip region");
    assert_eq!(
        press.region.as_deref(),
        Some("Dash#strip"),
        "region click dispatches at press (hit-enqueues, 054 canonical)"
    );
    h.cancel_all(agp::core::composition::input::CancelReason::PointerLeftWindow);
    settle(&mut h, 3);
    assert_eq!(
        texts(&mut h)[0],
        "100",
        "already-dispatched interaction survives pointer leave"
    );
    assert!(h.mount_errors.is_empty());
}
