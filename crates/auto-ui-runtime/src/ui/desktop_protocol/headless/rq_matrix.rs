//! PLAN-061 T-04: 六格对拍驱动原型（research-level，test-only）。
//!
//! 格 = producer（Rust View / VM）× host 路径（direct / standalone-RQ /
//! desktop-RQ）。本原型只把**可运行/可近似**的格跑起来，其余格显式列
//! 缺口与后继 owner（V-04：缺格不报 PASS，不冒生产联验）。同时输出
//! M-01～M-08 的原型测量/未测分列，以及漏 mesh/错版本/错字体的负控。
//!
//! 代理终点：direct=iced_test offscreen wgpu snapshot；RQ=inproc producer
//! + broker canvas 重放。两者都不是物理上屏，真实 Host/进程证据归
//! T-04 后继（OS063/UI063）实测。

use std::path::PathBuf;
use std::time::Instant;

use super::rq_probe::{direct_frames, ms, out_dir, stats, theme};
use super::*;
use crate::ui::component::Component;
use crate::ui::view::View;
use iced_tiny_skia::Layer;

const W: f32 = 640.0;
const H: f32 = 400.0;
const SAMPLES: usize = 12;
const WARMUP: usize = 2;

#[derive(Debug, Clone)]
struct MatrixComponent {
    view: View<()>,
}

impl Component for MatrixComponent {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

#[derive(Clone, Copy, Debug)]
enum CellStatus {
    /// 本原型可真实跑（offscreen 代理）。
    Runnable,
    /// 可跑但只是 inproc 近似，不是真实独立 Host 进程。
    InprocApprox,
    /// 本原型不可运行：显式缺口 + owner。
    Unrunnable,
}

#[derive(Clone, Copy)]
struct Cell {
    id: &'static str,
    producer: &'static str,
    host: &'static str,
    status: CellStatus,
    note: &'static str,
}

fn cells() -> Vec<Cell> {
    vec![
        Cell {
            id: "rust/direct",
            producer: "rust (View→iced)",
            host: "direct（本进程 wgpu，offscreen 代理）",
            status: CellStatus::Runnable,
            note: "与产品 direct 同 View/同 iced 组件；present 为代理终点",
        },
        Cell {
            id: "rust/standalone-rq",
            producer: "rust（headless tiny-skia Secondary 记录）",
            host: "standalone RQHost（inproc broker canvas 重放近似）",
            status: CellStatus::InprocApprox,
            note: "真实独立 RQHost 进程/窗口未启动；owner=T-04 后继（OS063 实测）",
        },
        Cell {
            id: "rust/desktop-rq",
            producer: "rust（同上记录）",
            host: "desktop RQHost（AutoOS 桌面嵌载）",
            status: CellStatus::Unrunnable,
            note: "需 AutoOS 桌面会话与嵌载 surface；owner=UI063/OS063",
        },
        Cell {
            id: "vm/direct",
            producer: "VM（a2r 执行 + VM 组件）",
            host: "direct（本进程 wgpu）",
            status: CellStatus::Unrunnable,
            note: "需 VM 运行入口与 .at 语料；owner=UI063/OS063",
        },
        Cell {
            id: "vm/standalone-rq",
            producer: "VM",
            host: "standalone RQHost",
            status: CellStatus::Unrunnable,
            note: "同 vm/direct + 独立 Host；owner=UI063/OS063",
        },
        Cell {
            id: "vm/desktop-rq",
            producer: "VM",
            host: "desktop RQHost",
            status: CellStatus::Unrunnable,
            note: "VM + 桌面嵌载；owner=UI063/OS063",
        },
    ]
}

fn case_views() -> Vec<(&'static str, View<()>)> {
    vec![
        ("mesh", super::rq_seams::canvas_scene_view()),
        ("svg_icon", View::image("lucide:bell")),
        ("text_cjk", View::text("中文路径客户数据库混排 ABC")),
        (
            "gradient_shadow",
            View::container(View::text("渐变阴影"))
                .style("p-4 rounded-xl bg-gradient-to-r from-blue-500 to-red-500 shadow-lg")
                .build(),
        ),
        ("progress_bar", View::progress_bar(0.6)),
    ]
}

// ---------------------------------------------------------------------------
// 记录/重放与签名
// ---------------------------------------------------------------------------

fn record_layers(view: View<()>) -> Vec<Layer> {
    let mut surface = HeadlessSurface::new(MatrixComponent { view }, W, H);
    surface.drive_and_draw()
}

fn recorded_primitives(layers: &[Layer]) -> usize {
    layers
        .iter()
        .map(|l| {
            l.primitives
                .iter()
                .map(|item| item.as_slice().len())
                .sum::<usize>()
        })
        .sum()
}

fn produce_v2(view: View<()>) -> (Vec<Layer>, super::super::message::DisplayList, usize) {
    let mut surface = HeadlessSurface::new(MatrixComponent { view }, W, H);
    let layers = surface.drive_and_draw();
    let mut obs = Vec::new();
    let mut bms = Vec::new();
    let mut cache = ResourceCache::default();
    let list = lower_layers_v2(&layers, &mut obs, &mut cache, &mut bms, false, false, 2.0);
    (layers, list, bms.len())
}

fn displaylist_element_snapshot(
    list: &super::super::message::DisplayList,
    png: &std::path::Path,
) -> Result<(), String> {
    // 对照口径：去掉 wire 自带 clear（direct 基准用主题底色），否则背景
    // 色差吞掉原语差（T-01 已单列该发现）。
    let mut neutral = list.clone();
    neutral.clear = None;
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        crate::ui::iced::broker_surface::displaylist_element::<()>(&neutral),
    );
    let snap = sim.snapshot(&theme()).map_err(|e| e.to_string())?;
    snap.matches_image(png).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 六格驱动
// ---------------------------------------------------------------------------

#[test]
fn rq_matrix_t04_six_cell_driver() {
    let out = out_dir().join("matrix");
    std::fs::create_dir_all(&out).expect("matrix out");
    let pid = std::process::id();
    let profile = format!(
        "debug; iced 0.14.0; iced_wgpu 0.14.0; wgpu 27.0.1; tiny-skia 0.11.4; viewport {}x{} @scale2",
        W, H
    );
    let source = "v2 DisplayList（T-02/T-03 合同；未含 v3 原语）";

    let mut cell_lines = Vec::new();
    for cell in cells() {
        let line = match cell.status {
            CellStatus::Runnable | CellStatus::InprocApprox => {
                let mut cases = Vec::new();
                for (case, view) in case_views() {
                    let direct_png = out.join(format!("{}-{}-direct.png", cell.id.replace('/', "_"), case));
                    let rq_png = out.join(format!("{}-{}-rq.png", cell.id.replace('/', "_"), case));
                    let (layers, list, bitmap_uploads) = produce_v2(view.clone());
                    let recorded = recorded_primitives(&layers);

                    // direct 代理：startup + 稳态帧
                    let direct = direct_frames(view.clone(), Some(&direct_png));
                    // RQ 产帧/编码/重放
                    let t0 = Instant::now();
                    let (layers2, list2, _) = produce_v2(view.clone());
                    let produce = ms(t0.elapsed());
                    let mut encoded = Vec::new();
                    let t1 = Instant::now();
                    list2.encode(&mut encoded);
                    let encode = ms(t1.elapsed());
                    let t2 = Instant::now();
                    let mut rd = super::super::codec::Reader::new(&encoded);
                    let decoded =
                        super::super::message::DisplayList::decode(&mut rd).expect("decode");
                    let decode = ms(t2.elapsed());
                    let _ = layers2;
                    let replay = displaylist_element_snapshot(&decoded, &rq_png);

                    cases.push(serde_json::json!({
                        "case": case,
                        "recorded_primitives": recorded,
                        "wire_ops": list.ops.len(),
                        "wire_bytes": encoded.len(),
                        "bitmap_uploads": bitmap_uploads,
                        "produce_ms": produce,
                        "encode_ms": encode,
                        "decode_ms": decode,
                        "direct": direct.as_ref().map(|(s, f)| serde_json::json!({
                            "startup_ms": s, "frame_p50_ms": f.p50, "frame_p95_ms": f.p95
                        })).unwrap_or_else(|e| serde_json::json!({ "error": e })),
                        "replay": replay.map(|_| serde_json::json!({ "png": rq_png.file_name().map(|n| n.to_string_lossy().to_string()) }))
                            .unwrap_or_else(|e| serde_json::json!({ "error": e })),
                    }));
                }
                serde_json::json!({ "cell": cell.id, "producer": cell.producer, "host": cell.host,
                                    "status": "inproc-approx", "note": cell.note, "cases": cases })
            }
            CellStatus::Unrunnable => serde_json::json!({
                "cell": cell.id, "producer": cell.producer, "host": cell.host,
                "status": "unrunnable", "owner": cell.note, "cases": [],
            }),
        };
        println!("[RQMATRIX] {}", serde_json::to_string(&line).unwrap());
        cell_lines.push(line);
    }

    // 汇总：runnable 格数必须与声明一致；不可运行格必须有 owner。
    let runnable = cell_lines
        .iter()
        .filter(|c| c["status"] == "inproc-approx")
        .count();
    let unrunnable = cell_lines
        .iter()
        .filter(|c| c["status"] == "unrunnable")
        .count();
    assert_eq!(runnable + unrunnable, 6, "六格必须逐格具名");
    assert!(unrunnable >= 1, "研究原型不得把不可运行格标成 PASS");
    for c in cell_lines.iter().filter(|c| c["status"] == "unrunnable") {
        assert!(
            c["owner"].as_str().map(|s| !s.is_empty()).unwrap_or(false),
            "不可运行格必须带 owner"
        );
    }
    println!(
        "[RQMATRIX] {}",
        serde_json::json!({
            "mode": "summary", "pid": pid, "profile": profile, "source": source,
            "runnable_cells": runnable, "unrunnable_cells": unrunnable,
            "note": "inproc 近似≠真实 Host；VM 格与 desktop 格为显式缺口（owner 见格记录）",
        })
    );
}

// ---------------------------------------------------------------------------
// 负控：漏 mesh / 错载荷版本 / 错字体 必须可检出
// ---------------------------------------------------------------------------

#[test]
fn rq_matrix_t04_negative_controls() {
    use super::super::message::{DisplayList, DrawList, DrawOp, Rgba8, WRect};

    // 1) 漏 mesh：记录器有 path 原语，而当前 v2 wire 一个都没有 → 结构性
    //    oracle 必须红。若未来 v3 上线，此断言改为“wire 与记录器差额=0”。
    let (layers, list, _) = produce_v2(super::rq_seams::canvas_scene_view());
    let recorded = recorded_primitives(&layers);
    assert!(recorded > 0, "mesh 记录器应有 path 原语");
    // 记录 4 条 path，而 wire 里可承载几何的原语（Quad）为 0：差额=漏 mesh。
    let quad_ops = list
        .ops
        .iter()
        .filter(|o| matches!(o, super::super::message::DisplayOp::Quad { .. }))
        .count();
    let text_ops = list
        .ops
        .iter()
        .filter(|o| matches!(o, super::super::message::DisplayOp::TextStyled { .. }))
        .count();
    assert!(
        recorded > quad_ops,
        "记录 path 原语({recorded}) 必须多于 wire 几何原语({quad_ops})——漏 mesh 可检出"
    );
    println!(
        "[RQMATRIX] {}",
        serde_json::json!({
            "tag": "nc_lost_mesh", "recorded_primitives": recorded,
            "wire_quad_ops": quad_ops, "wire_text_ops": text_ops,
            "detected": true, "meaning": "记录 path=4 且 wire Quad=0 → 漏 mesh 被结构 oracle 检出",
        })
    );

    // 2) 错载荷版本：v1 DrawList 载荷长度/结构当 v2 解 → 必须报错。
    let v1 = DrawList {
        clear: None,
        ops: vec![DrawOp::Quad {
            rect: WRect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 },
            color: Rgba8::new(255, 0, 0, 255),
        }],
    };
    let mut v1_bytes = Vec::new();
    v1.encode(&mut v1_bytes);
    // v1 载荷首字节 tag=1；v2 解码面应拒绝（而不是按 v2 静默解析）。
    let mut r = super::super::codec::Reader::new(&v1_bytes);
    let decoded = DisplayList::decode(&mut r);
    assert!(decoded.is_err(), "错版本载荷不得被 v2 解码面接受");
    println!(
        "[RQMATRIX] {}",
        serde_json::json!({ "tag": "nc_wrong_payload_version", "error": format!("{:?}", decoded.err()), "detected": true })
    );

    // 3) 错字体/字号：重放侧改字号后的图像与 direct 差异必须显著，且与
    //    正确重放差异可区分（负控敏感性）。
    let view = View::text("中文路径客户数据库混排 ABC");
    let out = out_dir().join("matrix");
    std::fs::create_dir_all(&out).expect("matrix out");
    let direct_base = out.join("nc_font-direct.png");
    let _ = direct_frames(view.clone(), Some(&direct_base));
    let (_layers, list, _) = produce_v2(view);
    let correct_base = out.join("nc_font-correct.png");
    displaylist_element_snapshot(&list, &correct_base).expect("correct replay");

    let mut wrong = list.clone();
    for op in &mut wrong.ops {
        if let super::super::message::DisplayOp::TextStyled { size, .. } = op {
            *size += 6.0;
        }
    }
    let wrong_base = out.join("nc_font-wrong.png");
    displaylist_element_snapshot(&wrong, &wrong_base).expect("wrong replay");

    let diff = |a: &std::path::Path, b: &std::path::Path| -> u64 {
        let img_a = image::open(a).expect("png a").to_rgba8();
        let img_b = image::open(b).expect("png b").to_rgba8();
        if img_a.dimensions() != img_b.dimensions() {
            return u64::MAX;
        }
        img_a
            .pixels()
            .zip(img_b.pixels())
            .filter(|(pa, pb)| pa.0 != pb.0)
            .count() as u64
    };
    // matches_image 会按渲染器名追加后缀（-wgpu）。
    let direct_png = out.join("nc_font-direct-wgpu.png");
    let correct_png = out.join("nc_font-correct-wgpu.png");
    let wrong_png = out.join("nc_font-wrong-wgpu.png");
    let correct_diff = diff(&direct_png, &correct_png);
    let wrong_diff = diff(&direct_png, &wrong_png);
    println!(
        "[RQMATRIX] {}",
        serde_json::json!({
            "tag": "nc_wrong_font", "correct_diff_px": correct_diff, "wrong_diff_px": wrong_diff,
            "detected": wrong_diff > correct_diff.saturating_add(50),
        })
    );
    assert!(
        wrong_diff > correct_diff + 50,
        "错字体重放必须显著偏离 direct（负控敏感）"
    );
}
