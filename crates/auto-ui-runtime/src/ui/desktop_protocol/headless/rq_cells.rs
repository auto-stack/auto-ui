//! PLAN-062 T-05: 六格绘图/资源重放矩阵与指标重放（test-only）。
//!
//! 格 = producer（Rust / VM）× host 路径（direct / standalone-RQ / desktop-RQ）。
//! 本驱动只跑**本仓可真实执行**的格，其余格显式列缺口与 owner（缺格不报
//! PASS，不以 fixture 冒充 VM 或桌面嵌载）。
//!
//! 每格每例的输出：结构清单（v3 原语/资源计数）、对 direct 金样的像素差、
//! 资源驻留/上传/命中（M-04/M-05 资源面）、produce→encode→decode→replay
//! 分段 p50/p95（M-01/M-03 的实现侧代理，present 终点仍归 OS063）。
//!
//! 代理终点：direct = iced_test offscreen wgpu snapshot；RQ = 同源记录 +
//! broker canvas 重放。两者都不是物理上屏。

use std::path::PathBuf;
use std::time::Instant;

use super::rq_probe::{direct_frames, ms, out_dir, stats, theme};
use super::*;
use crate::ui::component::Component;
use crate::ui::desktop_protocol::client_runtime::bg as client_bg;
use crate::ui::desktop_protocol::message::{DisplayList, DisplayOp};
use crate::ui::view::View;

const W: f32 = 640.0;
const H: f32 = 400.0;
const SAMPLES: usize = 12;
const WARMUP: usize = 2;

#[derive(Debug, Clone)]
struct CellComponent {
    view: View<()>,
}

impl Component for CellComponent {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellStatus {
    /// 本仓可真实执行（offscreen 代理终点）。
    Runnable,
    /// 可跑但为同进程近似（真实独立 Host 进程归 OS063）。
    InprocApprox,
    /// 不可运行：显式缺口 + owner。
    Unrunnable,
}

struct Cell {
    id: &'static str,
    producer: &'static str,
    host: &'static str,
    status: CellStatus,
    owner: &'static str,
    note: &'static str,
}

fn cells() -> Vec<Cell> {
    vec![
        Cell {
            id: "rust/direct",
            producer: "rust（View→iced 组件树）",
            host: "direct（本进程 wgpu，offscreen 代理）",
            status: CellStatus::Runnable,
            owner: "T-05",
            note: "产品 direct 同源画面 = 迁移前金样",
        },
        Cell {
            id: "rust/standalone-rq",
            producer: "rust（headless tiny-skia Secondary 记录 → v3 wire）",
            host: "standalone RQHost（broker canvas 重放，inproc 近似）",
            status: CellStatus::InprocApprox,
            owner: "T-05（真实独立进程窗口归 OS063）",
            note: "同 source/主题/viewport/scale/事件；present 为代理终点",
        },
        Cell {
            id: "rust/desktop-rq",
            producer: "rust（同上）",
            host: "desktop RQHost（AutoOS 桌面嵌载 surface）",
            status: CellStatus::Unrunnable,
            owner: "UI063 / OS063",
            note: "需桌面会话与嵌载 surface；本仓无自动运行入口",
        },
        Cell {
            id: "vm/direct",
            producer: "VM（a2r 执行）",
            host: "direct",
            status: CellStatus::Unrunnable,
            owner: "UI063",
            note: "需 VM 公开运行入口与 .at 语料（plan §5：fixture 不得替代 producer）",
        },
        Cell {
            id: "vm/standalone-rq",
            producer: "VM",
            host: "standalone RQHost",
            status: CellStatus::Unrunnable,
            owner: "UI063 / OS063",
            note: "同上 + 独立 Host",
        },
        Cell {
            id: "vm/desktop-rq",
            producer: "VM",
            host: "desktop RQHost",
            status: CellStatus::Unrunnable,
            owner: "UI063 / OS063",
            note: "同上 + 桌面嵌载",
        },
    ]
}

/// 绘图语料：覆盖 062 增量（path/mesh、SVG 资源、图像、渐变/边框/阴影、
/// 文本、clip 嵌套）。同 source 同时喂 direct 与 RQ 两侧。
fn case_views() -> Vec<(&'static str, View<()>)> {
    vec![
        ("path_mesh", super::rq_seams::canvas_scene_view()),
        ("svg_icon", View::image("lucide:bell")),
        (
            "text_cjk",
            View::text("中文路径客户数据库混排 ABC"),
        ),
        (
            "gradient_border_shadow",
            View::container(View::text("渐变边框阴影"))
                .style("p-4 rounded-xl bg-gradient-to-r from-blue-500 to-red-500 border shadow-lg")
                .build(),
        ),
        (
            "scroll_clip",
            View::scrollable(View::col().child(View::text("第一行")).child(View::text("第二行")).build())
                .style("max-h-24")
                .build(),
        ),
        (
            "shadow_only",
            View::container(View::empty())
                .style("w-24 h-16 m-8 rounded-xl shadow-lg bg-muted")
                .build(),
        ),
        ("progress_bar", View::progress_bar(0.6)),
    ]
}

fn out_cells() -> PathBuf {
    out_dir().join("cells")
}

/// 快照落盘（`matches_image` 对已存在文件只比对不覆盖——先清掉旧档，
/// 否则读数会停在上一轮的图上）。
fn write_snapshot(
    snap: &iced_test::simulator::Snapshot,
    out: &std::path::Path,
    name: &str,
) -> Result<(), String> {
    let req = out.join(format!("{name}.png"));
    let _ = std::fs::remove_file(&req);
    let _ = std::fs::remove_file(out.join(format!("{name}-wgpu.png")));
    snap.matches_image(&req)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn written(out: &std::path::Path, name: &str) -> PathBuf {
    out.join(format!("{name}-wgpu.png"))
}

/// 结构清单：按 v3/v2 原语统计（漏图在结构面即可见）。
fn inventory(list: &DisplayList) -> serde_json::Value {
    let (mut quad, mut text, mut path, mut img, mut clip, mut xform) = (0, 0, 0, 0, 0, 0);
    let mut path_segs = 0usize;
    for op in &list.ops {
        match op {
            DisplayOp::Quad { .. } => quad += 1,
            DisplayOp::TextStyled { .. } => text += 1,
            DisplayOp::Path { path: p, .. } => {
                path += 1;
                path_segs += p.segs.len();
            }
            DisplayOp::Image { .. } => img += 1,
            DisplayOp::GlyphRun { glyphs, .. } => {
                text += 1;
                path_segs += glyphs.len();
            }
            DisplayOp::Scissor { .. } | DisplayOp::ScissorPop => clip += 1,
            DisplayOp::Transform { .. } | DisplayOp::TransformPop => xform += 1,
        }
    }
    serde_json::json!({
        "ops": list.ops.len(), "quad": quad, "text": text, "path": path,
        "path_segments": path_segs, "image": img, "clip": clip, "transform": xform,
        "payload_tag": if list.requires_v3() { 3 } else { 2 },
        "capabilities": list.required_capabilities(),
    })
}

/// direct 金样左上像素（= 该 View 族的实际底色）。
fn corner_color(png: &std::path::Path) -> Rgba8 {
    let img = image::open(png).expect("corner png").to_rgba8();
    let p = img.get_pixel(0, 0);
    Rgba8::new(p[0], p[1], p[2], 255)
}

fn pixel_diff_ratio(a: &std::path::Path, b: &std::path::Path, allowance: u8) -> f64 {    let left = image::open(a).expect("left png").to_rgba8();
    let right = image::open(b).expect("right png").to_rgba8();
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

/// 单例 rust/rq 测点：产帧 → 编码 → 解码 → 重放，分段计时 + 资源计数。
struct CellReading {
    inventory: serde_json::Value,
    produce_p50: f64,
    produce_p95: f64,
    encode_p50: f64,
    decode_p50: f64,
    replay_p50: f64,
    replay_p95: f64,
    uploads: u64,
    uploaded_bytes: u64,
    hits: u64,
    raster_hits: u64,
    resident_bytes: usize,
    diff_ratio: f64,
}

fn measure_case(
    view: View<()>,
    out: &std::path::Path,
    name: &str,
    mode: &str,
    glyph_runs: bool,
) -> Result<CellReading, String> {
    let name = &format!("{name}-{mode}");
    let client_bg = client_bg();

    // direct 代理金样：同一客户区底色（否则“谁画底色”会吞掉原语差）。
    let direct_element = iced::widget::container(view.clone().into_iced())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(move |_t| iced::widget::container::Style {
            background: Some(crate::ui::iced::broker_surface::to_color(client_bg).into()),
            ..Default::default()
        });
    let mut direct_sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        direct_element,
    );
    let mut direct_frames_ms = Vec::new();
    for i in 0..(WARMUP + SAMPLES) {
        let t = Instant::now();
        let snap = direct_sim
            .snapshot(&theme())
            .map_err(|e| format!("direct snapshot: {e}"))?;
        let elapsed = ms(t.elapsed());
        if i == 0 {
            write_snapshot(&snap, out, &format!("{name}-direct"))?;
        }
        if i >= WARMUP {
            direct_frames_ms.push(elapsed);
        }
    }
    let direct_stats = stats(direct_frames_ms);
    let direct_png = written(out, &format!("{name}-direct"));
    // 用 direct 金样的左上像素作为重放底色：不同 View 族（canvas 自绘底 vs
    // 透明）底色来源不同，硬编码主题色会把“谁画底色”记成保真差。
    let backdrop = corner_color(&direct_png);

    let mut surface = HeadlessSurface::new(CellComponent { view }, W, H);
    // 两种 producer 模式都要量：default = 生产默认（字符串面），glyph_runs =
    // 用户裁定打开的 glyph run 面（非默认）。
    surface.set_glyph_runs(glyph_runs);
    // “v3 富模式”同时打开 SDF 阴影贴片；默认面保留 scrim 近似（见 Spec §2.3）。
    surface.set_shadow_patches(glyph_runs);
    let (mut produce, mut encode, mut decode, mut replay) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut inventory_json = serde_json::Value::Null;
    let mut replay_sim: Option<iced_test::simulator::Simulator<'static, (), iced::Theme, iced::Renderer>> =
        None;

    for i in 0..(WARMUP + SAMPLES) {
        let t = Instant::now();
        let list = surface.render_frame_v2();
        produce.push(ms(t.elapsed()));

        let t = Instant::now();
        let mut buf = Vec::new();
        list.encode(&mut buf);
        encode.push(ms(t.elapsed()));

        let t = Instant::now();
        let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&buf);
        let decoded = DisplayList::decode(&mut reader).map_err(|e| format!("decode: {e:?}"))?;
        decode.push(ms(t.elapsed()));

        // 宿主把本帧资源登记进图像缓存（生产路径 = BitmapReady → cache）。
        for up in surface.drain_bitmap_uploads() {
            crate::ui::iced::broker_surface::bitmap_cache_put(
                &up.id,
                up.w,
                up.h,
                up.stride,
                up.rgba.clone(),
            );
        }

        if replay_sim.is_none() {
            inventory_json = inventory(&decoded);
            let mut replay_list = decoded.clone();
            replay_list.clear = Some(backdrop);
            replay_sim = Some(iced_test::simulator::Simulator::with_size(
                iced::Settings::default(),
                iced::Size::new(W, H),
                crate::ui::iced::broker_surface::displaylist_element::<()>(&replay_list),
            ));
        }
        let t = Instant::now();
        let snap = replay_sim
            .as_mut()
            .expect("上方已建")
            .snapshot(&theme())
            .map_err(|e| format!("replay snapshot: {e}"))?;
        let elapsed = ms(t.elapsed());
        if i == 0 {
            write_snapshot(&snap, out, &format!("{name}-rq"))?;
        }
        if i >= WARMUP {
            replay.push(elapsed);
        }
    }

    let rq_png = written(out, &format!("{name}-rq"));
    let diff = pixel_diff_ratio(&direct_png, &rq_png, 2);
    let (uploads, uploaded_bytes, hits, _evictions, resident) = surface.resource_stats();
    let raster_hits = surface.raster_cache_hits();
    let mut inventory_json = inventory_json;
    inventory_json["direct_frame_p50_ms"] = serde_json::json!(direct_stats.p50);
    inventory_json["direct_frame_p95_ms"] = serde_json::json!(direct_stats.p95);
    Ok(CellReading {
        inventory: inventory_json,
        produce_p50: stats(produce.clone()).p50,
        produce_p95: stats(produce).p95,
        encode_p50: stats(encode).p50,
        decode_p50: stats(decode).p50,
        replay_p50: stats(replay.clone()).p50,
        replay_p95: stats(replay).p95,
        uploads,
        uploaded_bytes,
        hits,
        raster_hits,
        resident_bytes: resident,
        diff_ratio: diff,
    })
}

#[test]
fn rq_cells_t05_six_cell_matrix() {
    let out = out_cells();
    std::fs::create_dir_all(&out).expect("cells out");

    let mut cell_rows = Vec::new();
    for cell in cells() {
        match cell.status {
            CellStatus::Unrunnable => {
                cell_rows.push(serde_json::json!({
                    "cell": cell.id, "producer": cell.producer, "host": cell.host,
                    "status": "unrunnable", "owner": cell.owner, "note": cell.note,
                }));
                println!(
                    "[RQCELLS] {}",
                    serde_json::json!({
                        "tag": "cell", "cell": cell.id, "status": "unrunnable",
                        "owner": cell.owner,
                    })
                );
            }
            CellStatus::Runnable | CellStatus::InprocApprox => {
                let mut cases = Vec::new();
                for (name, view) in case_views() {
                  for (mode, glyph_runs) in [("default", false), ("glyph_runs", true)] {
                    match measure_case(view.clone(), &out, name, mode, glyph_runs) {
                        Ok(reading) => {
                            if mode == "default" {
                                // 门只压默认 producer（生产路径）：文字/路径/SVG/滚动
                                // 必须 ≤0.2%（阈值 2/255）；阴影两例的残差已具名，
                                // 这里记录不设门。
                                if matches!(name, "path_mesh" | "svg_icon" | "text_cjk" | "scroll_clip" | "progress_bar") {
                                    assert!(
                                        reading.diff_ratio <= 0.002,
                                        "{name} 默认 producer 的像素差 {:.4} 超 0.2% 门",
                                        reading.diff_ratio
                                    );
                                }
                            }
                            println!(
                                "[RQCELLS] {}",
                                serde_json::json!({
                                    "tag": "case", "cell": cell.id, "case": name, "producer_mode": mode,
                                    "inventory": reading.inventory,
                                    "produce_p50": reading.produce_p50,
                                    "produce_p95": reading.produce_p95,
                                    "encode_p50": reading.encode_p50,
                                    "decode_p50": reading.decode_p50,
                                    "replay_p50": reading.replay_p50,
                                    "replay_p95": reading.replay_p95,
                                    "uploads": reading.uploads,
                                    "uploaded_bytes": reading.uploaded_bytes,
                                    "hits": reading.hits,
                                    "resident_bytes": reading.resident_bytes, "raster_hits": reading.raster_hits,
                                    "pixel_diff_vs_direct": reading.diff_ratio,
                                })
                            );
                            cases.push(serde_json::json!({
                                "case": name, "producer_mode": mode,
                                "inventory": reading.inventory,
                                "timing_ms": {
                                    "produce_p50": reading.produce_p50,
                                    "produce_p95": reading.produce_p95,
                                    "encode_p50": reading.encode_p50,
                                    "decode_p50": reading.decode_p50,
                                    "replay_p50": reading.replay_p50,
                                    "replay_p95": reading.replay_p95,
                                },
                                "resources": {
                                    "uploads": reading.uploads,
                                    "uploaded_bytes": reading.uploaded_bytes,
                                    "hits": reading.hits,
                                    "resident_bytes": reading.resident_bytes, "raster_hits": reading.raster_hits,
                                },
                                "pixel_diff_vs_direct": reading.diff_ratio,
                            }));
                        }
                        Err(err) => {
                            cases.push(serde_json::json!({
                                "case": name, "producer_mode": mode, "error": err, "status": "failed",
                            }));
                        }
                    }
                  }
                }
                cell_rows.push(serde_json::json!({
                    "cell": cell.id, "producer": cell.producer, "host": cell.host,
                    "status": if cell.status == CellStatus::Runnable { "runnable" } else { "inproc_approx" },
                    "owner": cell.owner, "note": cell.note, "cases": cases,
                }));
            }
        }
    }

    let report = serde_json::json!({
        "tag": "plan062_t05_cells",
        "viewport": [W, H],
        "scale": 2,
        "profile": "debug; iced 0.14.0; tiny-skia 0.11.4; resvg 0.45; offscreen 代理终点",
        "cells": cell_rows,
        "metrics": {
            "M-01": {"status": "proxy", "note": "输入→帧的产帧/回放分段；物理上屏归 OS063"},
            "M-02": {"status": "unmeasured", "note": "真实刷新 deadline 需 OS063 双 Host 实机"},
            "M-03": {"status": "proxy", "note": "produce/encode/decode/replay 分段 + direct 稳态帧"},
            "M-04": {"status": "measured", "note": "资源上传/命中计数（stable 帧 0 重传）"},
            "M-05": {"status": "proxy", "note": "producer 驻留字节；Host GPU 驻留未测"},
            "M-06": {"status": "unmeasured", "note": "冷/热启动归 OS063"},
            "M-07": {"status": "unmeasured", "note": "1/4/8 App 真实规模归 UI063/OS063"},
            "M-08": {"status": "measured", "note": "像素差 + 结构清单同时断言"}
        }
    });
    std::fs::write(
        out.join("t05-cells.json"),
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    )
    .expect("write report");
    println!("[RQCELLS] {}", serde_json::json!({ "tag": "cells_written", "path": out.join("t05-cells.json").display().to_string() }));

    // 可运行格必须真有 v3 路径原语与资源身份；不可运行格必须有 owner。
    let runnable = report["cells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["status"] == "runnable" || c["status"] == "inproc_approx")
        .count();
    assert!(runnable >= 2, "至少 direct + standalone-RQ 两格可跑，实得 {runnable}");
    let blocked = report["cells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["status"] == "unrunnable")
        .count();
    assert_eq!(blocked, 4, "四个不可运行格必须显式登记（不删分母）");
    for cell in report["cells"].as_array().unwrap() {
        if cell["status"] == "unrunnable" {
            assert!(
                cell["owner"].as_str().map(|o| !o.is_empty()).unwrap_or(false),
                "不可运行格必须具名 owner"
            );
        }
    }

    // mesh 例：v3 Path 必须在结构面出现（漏 mesh = AC-02 失败）。
    let mesh = report["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["cell"] == "rust/standalone-rq")
        .and_then(|c| c["cases"].as_array().cloned())
        .and_then(|cases| {
            cases
                .into_iter()
                .find(|case| case["case"] == "path_mesh")
        })
        .expect("path_mesh 例存在");
    assert!(
        mesh["inventory"]["path"].as_u64().unwrap_or(0) > 0,
        "mesh 必须以 v3 Path 过线"
    );
    // SVG 例：必须作为资源过线（upload 计数 > 0）。
    let svg = report["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["cell"] == "rust/standalone-rq")
        .and_then(|c| c["cases"].as_array().cloned())
        .and_then(|cases| cases.into_iter().find(|case| case["case"] == "svg_icon"))
        .expect("svg_icon 例存在");
    assert!(
        svg["resources"]["uploads"].as_u64().unwrap_or(0) > 0,
        "SVG 必须栅格化为资源上传"
    );
}
