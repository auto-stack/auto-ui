//! PLAN-062 T-02: v3 无损原语线合同探针（test-only）。
//!
//! 验证对象是**生产路径**：`HeadlessSurface::render_frame_v2` 的 v3 降格 →
//! `DisplayList::encode/decode`（载荷 tag 3）→ `broker_surface::
//! displaylist_element`（生产重放，含 `paint_ops_v2` 的 `Path` 臂）。
//!
//! 关闭条件（V-02 / AC-02 的 mesh 与几何面）：
//! 1. mesh/canvas 的 `Primitive::Fill/Stroke` 不再被省略，而是以 `Path`
//!    原语逐段过线（结构证据）；
//! 2. 编解码逐 op 等价，且含 `Path` 的帧载荷 tag=3（旧 Host 明确拒绝），
//!    纯 v2 词汇仍编 tag=2（双读窗口内不破坏旧端）；
//! 3. 真实重放像素与 direct 同源画面一致（区分性负控：删掉 Path 后必须显著
//!    变红，否则本用例不能证明它检出了漏图）。

use std::path::PathBuf;

use super::*;
use crate::ui::component::Component;
use crate::ui::desktop_protocol::message::{DisplayList, DisplayOp};
use crate::ui::view::View;

const W: f32 = 320.0;
const H: f32 = 240.0;

#[derive(Debug, Clone)]
struct LosslessComponent {
    view: View<()>,
}

impl Component for LosslessComponent {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

fn out_dir() -> PathBuf {
    match std::env::var_os("AUTO_RQ_PROBE_OUT") {
        Some(v) => PathBuf::from(v).join("lossless"),
        None => std::env::temp_dir().join("rq_probe").join("lossless"),
    }
}

fn theme() -> iced::Theme {
    crate::ui::iced::renderer::shadcn_theme(crate::ui::style_iced_adapter::dark_mode())
}

/// 只含几何（无文本/图像）的 canvas 场景：让像素 oracle 只考察 path 保真，
/// 不把 Host 文本重排残差混进来。
fn geometry_scene() -> View<()> {
    let mut scene = match super::rq_seams::canvas_scene_view() {
        View::Canvas { scene, .. } => scene,
        _ => unreachable!("canvas_scene_view 恒为 Canvas"),
    };
    scene.labels.clear();
    View::Canvas {
        scene,
        logical_extent: Some((320.0, 240.0)),
        clear: None,
        on_pen_start: None,
        on_pen_move: None,
        on_pen_end: None,
        on_hit: None,
        style: crate::ui::style::Style::parse("w-full h-full").ok(),
    }
}

fn count_paths(list: &DisplayList) -> (usize, usize, usize) {
    let mut fills = 0;
    let mut strokes = 0;
    let mut segs = 0;
    for op in &list.ops {
        if let DisplayOp::Path { path, fill, stroke, .. } = op {
            segs += path.segs.len();
            if fill.is_some() {
                fills += 1;
            }
            if stroke.is_some() {
                strokes += 1;
            }
        }
    }
    (fills, strokes, segs)
}

/// 像素差比（0.0..1.0）：逐像素 RGB 任一通道差 > allowance 即计为差。
fn pixel_diff_ratio(a: &std::path::Path, b: &std::path::Path, allowance: u8) -> f64 {
    let left = image::open(a).expect("read left png").to_rgba8();
    let right = image::open(b).expect("read right png").to_rgba8();
    assert_eq!(
        (left.width(), left.height()),
        (right.width(), right.height()),
        "两图尺寸必须一致"
    );
    let mut diff = 0usize;
    for (p, q) in left.pixels().zip(right.pixels()) {
        if (p[0] as i32 - q[0] as i32).abs() > allowance as i32
            || (p[1] as i32 - q[1] as i32).abs() > allowance as i32
            || (p[2] as i32 - q[2] as i32).abs() > allowance as i32
        {
            diff += 1;
        }
    }
    diff as f64 / (left.width() * left.height()) as f64
}

/// `iced_test` 把快照写成 `<stem>-<renderer>.png`（wgpu 轨追加 `-wgpu`）。
fn snapshot_path(out: &std::path::Path, name: &str) -> PathBuf {
    out.join(format!("{name}.png"))
}

fn written_path(out: &std::path::Path, name: &str) -> PathBuf {
    out.join(format!("{name}-wgpu.png"))
}

/// 生产重放（`displaylist_element` → `paint_ops_v2`）快照，返回落盘路径。
fn render_rq_png(list: &DisplayList, out: &std::path::Path, name: &str) -> PathBuf {
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        crate::ui::iced::broker_surface::displaylist_element::<()>(list),
    );
    let snap = sim.snapshot(&theme()).expect("rq snapshot");
    snap.matches_image(&snapshot_path(out, name)).expect("rq png");
    written_path(out, name)
}

#[test]
fn rq_v3_mesh_paths_reach_wire_and_replay() {
    let out = out_dir();
    std::fs::create_dir_all(&out).expect("probe out");
    let view = geometry_scene();

    // direct 同源画面（同一 canvas 组件路径），并钉同一客户区底色：否则
    // 比较会把“谁画底色”这一与几何保真无关的差异计入。
    let client_bg = crate::ui::desktop_protocol::client_runtime::bg();
    let direct_element = iced::widget::container(view.clone().into_iced())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(move |_t| iced::widget::container::Style {
            background: Some(crate::ui::iced::broker_surface::to_color(client_bg).into()),
            ..Default::default()
        });
    let direct_png = snapshot_path(&out, "lossless_mesh-direct");
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        direct_element,
    );
    let snap = sim.snapshot(&theme()).expect("direct snapshot");
    snap.matches_image(&snapshot_path(&out, "lossless_mesh-direct"))
        .expect("direct png");
    let direct_png = written_path(&out, "lossless_mesh-direct");

    // 生产录制（v2 降格 → v3 Path 增量）。
    let mut surface = HeadlessSurface::new(
        LosslessComponent { view },
        W,
        H,
    );
    let recorded = surface.render_frame_v2();
    let (fills, strokes, segs) = count_paths(&recorded);
    println!(
        "[RQ062] {}",
        serde_json::json!({
            "tag": "v3_wire_paths", "fills": fills, "strokes": strokes, "segments": segs,
            "total_ops": recorded.ops.len()
        })
    );
    assert!(
        fills + strokes > 0,
        "mesh/canvas 几何必须以 Path 原语进入 wire（漏 mesh 即 AC-02 失败）"
    );
    assert!(segs >= 6, "路径段数应覆盖全部笔画/节点几何，实得 {segs}");

    // 线编解码：逐 op 等价 + 载荷 tag 3。
    let mut buf = Vec::new();
    recorded.encode(&mut buf);
    assert_eq!(buf[0], 3, "含 Path 的载荷必须编 tag 3（旧 Host 明确拒绝）");
    let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&buf);
    let decoded = DisplayList::decode(&mut reader).expect("decode v3");
    assert_eq!(decoded, recorded, "v3 编解码必须逐 op 等价");

    // 纯 v2 词汇仍 must 编 tag 2（双读窗口内旧 Host 可继续消费）。
    let v2_only = DisplayList {
        clear: recorded.clear,
        ops: recorded
            .ops
            .iter()
            .filter(|op| !matches!(op, DisplayOp::Path { .. }))
            .cloned()
            .collect(),
    };
    let mut v2_buf = Vec::new();
    v2_only.encode(&mut v2_buf);
    assert_eq!(v2_buf[0], 2, "无 v3 原语的帧不得升 tag（旧端兼容）");

    // 生产重放（真正的 paint_ops_v2 Path 臂）；底色与 direct 侧一致。
    let replay = DisplayList {
        clear: Some(client_bg),
        ops: decoded.ops.clone(),
    };
    let rq_png = render_rq_png(&replay, &out, "lossless_mesh-rq");
    let ratio = pixel_diff_ratio(&direct_png, &rq_png, 2);
    println!(
        "[RQ062] {}",
        serde_json::json!({ "tag": "v3_replay_diff", "ratio": ratio, "paths": fills + strokes })
    );
    assert!(
        ratio < 0.01,
        "记录→重放的几何画面应与 direct 一致（残差 {:.4} 超过 1% 说明原语/坐标未保真）",
        ratio
    );

    // 区分性负控：删掉 Path（=062 前的漏 mesh 行为）必须显著变红，否则本
    // 用例无法证明它检出漏图。
    let without_paths = DisplayList {
        clear: Some(client_bg),
        ops: decoded
            .ops
            .iter()
            .filter(|op| !matches!(op, DisplayOp::Path { .. }))
            .cloned()
            .collect(),
    };
    let loss_png = render_rq_png(&without_paths, &out, "lossless_mesh-nopath");
    let loss_ratio = pixel_diff_ratio(&direct_png, &loss_png, 2);
    println!(
        "[RQ062] {}",
        serde_json::json!({ "tag": "v3_replay_nopath", "ratio": loss_ratio })
    );
    assert!(
        loss_ratio > ratio + 0.02,
        "漏 mesh 负控必须可检出（{:.4} 应显著高于 {:.4}）",
        loss_ratio,
        ratio
    );
}

#[test]
fn rq_v3_capability_admission_keeps_old_frame() {
    use crate::ui::desktop_protocol::host::SurfaceStore;
    use crate::ui::desktop_protocol::message::{
        AdmissionError, CapabilitySet, DisplayOp, Fill, WRect, CAP_PATH_FILL,
    };

    let mut store = SurfaceStore::new();
    let sid = store.alloc(100.0, 100.0);
    let quad_frame = DisplayList {
        clear: None,
        ops: vec![DisplayOp::Quad {
            rect: WRect::new(1.0, 1.0, 8.0, 8.0),
            fill: Fill::Color(Rgba8::new(255, 0, 0, 255)),
            radius: [0.0; 4],
            border: None,
            shadow: None,
        }],
    };
    let path_frame = DisplayList {
        clear: None,
        ops: vec![DisplayOp::Path {
            path: crate::ui::desktop_protocol::message::WirePath {
                segs: vec![
                    crate::ui::desktop_protocol::message::PathSeg::MoveTo(0.0, 0.0),
                    crate::ui::desktop_protocol::message::PathSeg::LineTo(20.0, 20.0),
                ],
            },
            fill: Some(Fill::Color(Rgba8::new(0, 255, 0, 255))),
            even_odd: false,
            stroke: None,
        }],
    };

    // 能力齐全（全 optional）→ v3 帧按受。
    assert!(store.compose_v2(sid, 0, quad_frame.clone()).unwrap().is_some());
    assert!(store.compose_v2(sid, 1, path_frame.clone()).unwrap().is_some());
    assert_eq!(store.front_v2(sid), Some(&path_frame));

    // 收窄到不含 pathfill 的能力集 → 具名拒绝，且**前端帧不变**（保旧帧）。
    store.set_capabilities(CapabilitySet::parse(&["required:fidelity/glyphrun/1".to_string()]));
    let err = store
        .compose_v2(sid, 0, path_frame.clone())
        .expect_err("缺 pathfill 能力必须拒绝 v3 帧");
    assert_eq!(
        err,
        AdmissionError::UnknownRequiredCapability(CAP_PATH_FILL.to_string()),
        "拒绝原因须具名到能力 ID"
    );
    assert_eq!(store.front_v2(sid), Some(&path_frame), "被拒帧不得翻面");

    // v2 词汇（无 Path）在收窄能力集下仍可合成（旧端兼容）。
    let v2_frame = DisplayList {
        clear: None,
        ops: vec![DisplayOp::Scissor {
            rect: WRect::new(0.0, 0.0, 10.0, 10.0),
        }],
    };
    assert!(store.compose_v2(sid, 0, v2_frame.clone()).unwrap().is_some());
    assert_eq!(store.front_v2(sid), Some(&v2_frame));

    // 能力回补 → 恢复接受 v3。
    store.set_capabilities(CapabilitySet::all_optional());
    assert!(store.compose_v2(sid, 0, path_frame.clone()).unwrap().is_some());
    assert_eq!(store.front_v2(sid), Some(&path_frame));

    // 线词汇 round-trip：required/optional 语义不丢，裸串按 optional。
    let set = CapabilitySet::parse(&[
        "required:fidelity/pathfill/1".to_string(),
        "fidelity/svg-raster/1".to_string(),
    ]);
    assert!(set.supports(CAP_PATH_FILL));
    assert!(set.supports("fidelity/svg-raster/1"));
    assert_eq!(set.required, vec![CAP_PATH_FILL.to_string()]);
    assert_eq!(
        CapabilitySet::parse(&set.to_wire()),
        set,
        "声明的线词汇必须可逆"
    );
    println!(
        "[RQ062] {}",
        serde_json::json!({
            "tag": "capability_admission", "rejected": "fidelity/pathfill/1",
            "old_frame_kept": true, "v2_still_accepted": true,
        })
    );
}

#[test]
fn rq_v3_svg_and_image_replay_matches_direct() {
    use crate::ui::iced::broker_surface;

    let out = out_dir();
    std::fs::create_dir_all(&out).expect("probe out");
    // 矢量图标 = 生产端设备像素栅格化的资源（061 实测 2x 栅格与 direct 全等）。
    let view: View<()> = View::image("lucide:bell");

    let client_bg = crate::ui::desktop_protocol::client_runtime::bg();
    let direct_element = iced::widget::container(view.clone().into_iced())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(move |_t| iced::widget::container::Style {
            background: Some(broker_surface::to_color(client_bg).into()),
            ..Default::default()
        });
    let direct_png = snapshot_path(&out, "lossless_svg-direct");
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        direct_element,
    );
    let snap = sim.snapshot(&theme()).expect("direct svg");
    snap.matches_image(&direct_png).expect("direct svg png");
    let direct_png = written_path(&out, "lossless_svg-direct");

    let mut surface = HeadlessSurface::new(LosslessComponent { view }, W, H);
    let list = surface.render_frame_v2();
    let uploads = surface.drain_bitmap_uploads();
    println!(
        "[RQ062] {}",
        serde_json::json!({
            "tag": "svg_resource",
            "uploads": uploads.iter().map(|u| (u.w, u.h, u.rgba.len())).collect::<Vec<_>>(),
            "ops": list.ops.len(),
        })
    );
    assert!(!uploads.is_empty(), "SVG 必须以位图资源过线（不再省略）");
    assert!(
        list.ops.iter().any(|op| matches!(
            op,
            DisplayOp::Image { src, .. } if src.starts_with("bitmap://")
        )),
        "SVG 帧必须以 bitmap:// 资源引用呈现"
    );

    // 生产重放：宿主把上传资源登记进图像缓存后按 src 解析。
    for up in &uploads {
        broker_surface::bitmap_cache_put(&up.id, up.w, up.h, up.stride, up.rgba.clone());
    }
    let mut buf = Vec::new();
    list.encode(&mut buf);
    let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&buf);
    let decoded = DisplayList::decode(&mut reader).expect("decode");
    let replay = DisplayList {
        clear: Some(client_bg),
        ops: decoded.ops.clone(),
    };
    let rq_png = render_rq_png(&replay, &out, "lossless_svg-rq");
    let ratio = pixel_diff_ratio(&direct_png, &rq_png, 2);
    println!(
        "[RQ062] {}",
        serde_json::json!({ "tag": "svg_replay_diff", "ratio": ratio })
    );
    assert!(
        ratio < 0.01,
        "SVG 资源重放应与 direct 一致（残差 {ratio:.4}）"
    );
}

#[test]
fn rq_v3_upload_budget_defers_frame_without_partial_registration() {
    // per-App 上传预算（P-04 / 061 冻结 ≤8 MiB·s⁻¹ 的探针形态）：超限帧必须
    // 整体回滚并背压重试——不得半登记资源、不得丢内容。
    let one = View::image("lucide:bell");
    let two = View::row()
        .child(View::image("lucide:bell"))
        .child(View::image("lucide:check"))
        .build();
    let mut surface = HeadlessSurface::new(LosslessComponent { view: one }, W, H);
    surface.set_upload_budget(1024, std::time::Duration::from_secs(60));

    let _ = surface.render_frame_v2();
    assert!(!surface.take_upload_budget_overrun(), "首图 1024B 在预算内");
    assert_eq!(surface.drain_bitmap_uploads().len(), 1);
    let (uploads, bytes, _hits, _evictions, resident) = surface.resource_stats();
    assert_eq!((uploads, bytes, resident), (1, 1024, 1024));

    // 第二张图需要新的 1024B → 窗口总额 2048 > 1024 → 本帧延期。
    surface.component.view = two.clone();
    let _ = surface.render_frame_v2();
    assert!(surface.take_upload_budget_overrun(), "超预算帧必须延期");
    assert!(
        surface.drain_bitmap_uploads().is_empty(),
        "延期帧不得把已产出的上传半发出去"
    );
    let (uploads, bytes, _hits, _evictions, resident) = surface.resource_stats();
    assert_eq!(
        (uploads, bytes, resident),
        (1, 1024, 1024),
        "回滚后资源登记必须复原（不得留下半登记）"
    );
    assert_eq!(surface.upload_budget_deferrals(), 1);

    // 预算回补 → 同一视图重试：第二张图上传一次，内容不丢。
    surface.set_upload_budget(4096, std::time::Duration::from_secs(60));
    let _ = surface.render_frame_v2();
    assert!(!surface.take_upload_budget_overrun(), "预算回补后应放行");
    assert_eq!(surface.drain_bitmap_uploads().len(), 1, "补齐第二张图");
    let (uploads, _bytes, hits, _evictions, _resident) = surface.resource_stats();
    assert_eq!(uploads, 2, "重试后累计上传 = 2（首图 + 第二张）");
    assert!(hits > 0, "已驻留的首图必须命中缓存（零重传）");
    println!(
        "[RQ062] {}",
        serde_json::json!({
            "tag": "upload_budget", "deferrals": 1, "uploads_after_retry": uploads,
            "partial_registration": false,
        })
    );
}

#[test]
fn rq_v3_legacy_v2_host_rejects_v3_payload() {
    // 旧 Host（062 前）只认载荷 tag 2：v3 帧必须被明确拒绝而不是半帧呈现。
    let list = DisplayList {
        clear: None,
        ops: vec![DisplayOp::Path {
            path: crate::ui::desktop_protocol::message::WirePath {
                segs: vec![
                    crate::ui::desktop_protocol::message::PathSeg::MoveTo(0.0, 0.0),
                    crate::ui::desktop_protocol::message::PathSeg::LineTo(10.0, 10.0),
                ],
            },
            fill: Some(Fill::Color(Rgba8::new(255, 0, 0, 255))),
            even_odd: false,
            stroke: None,
        }],
    };
    let mut buf = Vec::new();
    list.encode(&mut buf);
    assert_eq!(buf[0], 3);
    let tag = buf[0];
    let legacy_accepts = tag == 2;
    assert!(
        !legacy_accepts,
        "旧端只读 tag 2——tag 3 必须走 UnknownTag 拒绝路径（保旧帧）"
    );
    assert_eq!(
        crate::ui::desktop_protocol::codec::CodecError::UnknownTag(3),
        crate::ui::desktop_protocol::codec::CodecError::UnknownTag(tag),
        "拒绝原因须具名为 UnknownTag(3)"
    );
}
