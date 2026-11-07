//! PLAN-061 T-01: direct baseline + RQ producer timing/counting probe (test-only).
//!
//! 目的（Plan §8 T-01）：
//! 1. 全能力分母的可编译清单——[`capability_id`] 对 `View` 全变体做穷尽 match
//!    （新增变体必编译失败，等价 V-01 的“删项可检出”静态负控）。
//! 2. 旧 direct 视觉基准——经 `iced_test` headless wgpu 渲染同一 `<case>` 得
//!    PNG（默认 `-wgpu` 后缀），落盘到 `AUTO_RQ_PROBE_OUT`（计划暂存区），
//!    不入仓；同帧经 RQ 记录/降格/重放另存 `-rq` 供像素差对照。
//! 3. 最小计时/计数入口——direct 帧、RQ 产帧（build/draw/lower 合计）、编码、
//!    解码、槽复制、重放各段耗时与 wire 字节/op/资源计数，输出 JSONL
//!    （`[RQPROBE] {...}`）供聚合脚本冻结 M-01～08 口径与预算。
//!
//! 边界：offline/offscreen wgpu 代理终点，不是物理上屏；不测 FPS、不替代
//! T-03/T-04 六格驱动与真实 Host。所有数字仅来自执行记录，不预设结论。

use std::path::{Path, PathBuf};
use std::time::Instant;

use super::*;
use crate::ui::component::Component;

#[path = "rq_probe/style_class_families.rs"]
mod style_class_families;
use crate::ui::style::Style;
use crate::ui::view::{PopoverAnchor, PopoverPlacement, View};

/// 静态测试组件：view 固定（direct 与 RQ 同源语料）。
#[derive(Debug, Clone)]
struct StaticComponent {
    view: View<()>,
}

impl Component for StaticComponent {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

/// 探针视口（逻辑 px，与 direct/RQ 两侧同源）。
const PROBE_W: u32 = 640;
const PROBE_H: u32 = 400;
const WARMUP: usize = 3;
const SAMPLES: usize = 20;

pub(super) fn theme() -> iced::Theme {
    crate::ui::iced::renderer::shadcn_theme(crate::ui::style_iced_adapter::dark_mode())
}

pub(super) fn out_dir() -> PathBuf {
    match std::env::var_os("AUTO_RQ_PROBE_OUT") {
        Some(v) => PathBuf::from(v),
        None => std::env::temp_dir().join("rq_probe"),
    }
}

// ---------------------------------------------------------------------------
// 1. 全能力分母（穷尽 match = 编译期完整性；ID 稳定可引用）
// ---------------------------------------------------------------------------

/// 返回能力的稳定 ID（族.名）。穷尽 match：删/漏一条臂即编译失败。
fn capability_id(v: &View<()>) -> &'static str {
    use View::*;
    match v {
        Empty => "cap.view.empty",
        AnchorSlot { .. } => "cap.view.anchor_slot",
        ScrollAnchorMark { .. } => "cap.view.scroll_anchor_mark",
        Text { .. } => "cap.view.text",
        Rich { .. } => "cap.view.rich",
        Button { .. } => "cap.view.button",
        Row { .. } => "cap.view.row",
        Column { .. } => "cap.view.column",
        Input { .. } => "cap.view.input",
        Textarea { .. } => "cap.view.textarea",
        CodeEditor { .. } => "cap.view.code_editor",
        Terminal { .. } => "cap.view.terminal",
        AutodownEditor { .. } => "cap.view.autodown_editor",
        Checkbox { .. } => "cap.view.checkbox",
        Custom { .. } => "cap.view.custom",
        ManagedScrollContent { .. } => "cap.view.managed_scroll_content",
        Container { .. } => "cap.view.container",
        Scrollable { .. } => "cap.view.scrollable",
        Radio { .. } => "cap.view.radio",
        Select { .. } => "cap.view.select",
        List { .. } => "cap.view.list",
        Table { .. } => "cap.view.table",
        Slider { .. } => "cap.view.slider",
        ProgressBar { .. } => "cap.view.progress_bar",
        Accordion { .. } => "cap.view.accordion",
        Sidebar { .. } => "cap.view.sidebar",
        Tabs { .. } => "cap.view.tabs",
        NavigationRail { .. } => "cap.view.navigation_rail",
        Image { .. } => "cap.view.image",
        ImageSurface { .. } => "cap.view.image_surface",
        Video { .. } => "cap.view.video",
        WindowThumbnail { .. } => "cap.view.window_thumbnail",
        WorkspacePreview { .. } => "cap.view.workspace_preview",
        RemoteFrame { .. } => "cap.view.remote_frame",
        Grid { .. } => "cap.view.grid",
        Overlay { .. } => "cap.view.overlay",
        Popover { .. } => "cap.view.popover",
        MouseArea { .. } => "cap.view.mouse_area",
        Canvas { .. } => "cap.view.canvas",
    }
}

/// 分母：39 个 View 变体 + 平台服务/生成器入口（由 T-01 静态清单登记）。
const ALL_VIEW_CAPABILITY_IDS: [&str; 39] = [
    "cap.view.empty",
    "cap.view.anchor_slot",
    "cap.view.scroll_anchor_mark",
    "cap.view.text",
    "cap.view.rich",
    "cap.view.button",
    "cap.view.row",
    "cap.view.column",
    "cap.view.input",
    "cap.view.textarea",
    "cap.view.code_editor",
    "cap.view.terminal",
    "cap.view.autodown_editor",
    "cap.view.checkbox",
    "cap.view.custom",
    "cap.view.managed_scroll_content",
    "cap.view.container",
    "cap.view.scrollable",
    "cap.view.radio",
    "cap.view.select",
    "cap.view.list",
    "cap.view.table",
    "cap.view.slider",
    "cap.view.progress_bar",
    "cap.view.accordion",
    "cap.view.sidebar",
    "cap.view.tabs",
    "cap.view.navigation_rail",
    "cap.view.image",
    "cap.view.image_surface",
    "cap.view.video",
    "cap.view.window_thumbnail",
    "cap.view.workspace_preview",
    "cap.view.remote_frame",
    "cap.view.grid",
    "cap.view.overlay",
    "cap.view.popover",
    "cap.view.mouse_area",
    "cap.view.canvas",
];

// ---------------------------------------------------------------------------
// 2. 语料（direct 与 RQ 同源；未构造项在报告里具名记缺口）
// ---------------------------------------------------------------------------

struct Case {
    id: &'static str,
    view: View<()>,
    /// 构造/运行条件说明（"ready" = 直接在两侧渲染）。
    note: &'static str,
}

fn styled_text(content: &str, style: &str) -> View<()> {
    View::Text {
        content: content.to_string(),
        style: Style::parse(style).ok(),
        selectable: false,
    }
}

fn corpus() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();

    cases.push(Case {
        id: "text_plain",
        view: View::text("Hello AutoUI"),
        note: "ready",
    });
    cases.push(Case {
        id: "text_cjk",
        view: View::text("中文路径客户数据库混排 ABC"),
        note: "ready",
    });
    cases.push(Case {
        id: "text_gradient_row",
        view: View::row()
            .spacing(8)
            .child(styled_text("A", "text-2xl font-bold"))
            .child(styled_text("B", "text-red-500"))
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "rich_spans",
        view: View::Rich {
            spans: vec![
                crate::ui::view::RichSpanView {
                    content: "富文本".to_string(),
                    style: Style::parse("font-bold").ok(),
                },
                crate::ui::view::RichSpanView {
                    content: " inline 混排".to_string(),
                    style: None,
                },
            ],
            style: Style::parse("text-base").ok(),
        },
        note: "ready",
    });
    cases.push(Case {
        id: "button",
        view: View::Button {
            label: "保存".to_string(),
            onclick: (),
            style: Style::parse("px-4 py-2 rounded bg-blue-500 text-white").ok(),
            on_right_click: None,
            content: None,
            disabled: false,
        },
        note: "ready",
    });
    cases.push(Case {
        id: "checkbox",
        view: View::checkbox(true, "启用"),
        note: "ready",
    });
    cases.push(Case {
        id: "radio",
        view: View::radio(true, "选项"),
        note: "ready",
    });
    cases.push(Case {
        id: "select",
        view: View::select(vec!["A".into(), "B".into(), "C".into()]),
        note: "ready",
    });
    cases.push(Case {
        id: "input_cjk",
        view: View::input("输入中文").value("已输入值").build(),
        note: "ready",
    });
    cases.push(Case {
        id: "textarea",
        view: View::textarea("多行").value("第一行\n第二行").build(),
        note: "ready",
    });
    cases.push(Case {
        id: "code_editor_rust",
        view: View::code_editor("probe-code")
            .value("fn main() {\n    println!(\"你好\");\n}")
            .lang("rust")
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "grid_2col",
        view: View::grid()
            .cols(2)
            .spacing(8)
            .child(View::text("g1"))
            .child(View::text("g2"))
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "container_gradient_radius",
        view: View::container(View::text("渐变圆角"))
            .style("p-4 rounded-xl bg-gradient-to-r from-blue-500 to-red-500")
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "container_shadow_border",
        view: View::container(View::text("阴影边框"))
            .style("p-4 rounded-lg bg-white border border-red-500 shadow-lg")
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "scrollable_column",
        view: View::Scrollable {
            child: Box::new(
                View::col()
                    .spacing(4)
                    .child(View::text("row-1"))
                    .child(View::text("row-2"))
                    .child(View::text("row-3"))
                    .build(),
            ),
            width: None,
            height: Some(120),
            style: Style::parse("w-full").ok(),
            auto_scroll: false,
            offset: Some((0.0, 0.0)),
            on_scroll: None,
            axes: crate::ui::scroll::ScrollAxes::Y,
            scrollbar_policy: crate::ui::scroll::ScrollbarPolicy::Auto,
            controller: None,
        },
        note: "ready",
    });
    cases.push(Case {
        id: "overlay_stack",
        view: View::Overlay {
            base: Box::new(View::text("底层")),
            content: Box::new(View::text("浮层")),
            position: crate::ui::view::OverlayPosition { top: None, right: None, bottom: None, left: None },
        },
        note: "ready",
    });
    cases.push(Case {
        id: "popover_open",
        view: View::Popover {
            anchor: PopoverAnchor::Point { x: 16.0, y: 16.0 },
            content: Box::new(View::text("弹层内容")),
            placement: PopoverPlacement::Bottom,
            open: true,
            on_dismiss: None,
        },
        note: "ready",
    });
    cases.push(Case {
        id: "mouse_area",
        view: View::MouseArea {
            content: Box::new(View::text("hover 区")),
            on_enter: None,
            on_exit: None,
            on_double_click: None,
            on_click: None,
            on_context_menu: None,
            on_release: None,
            on_move: None,
            logical_extent: Some((200.0, 80.0)),
            style: Style::parse("bg-blue-100").ok(),
        },
        note: "ready",
    });
    cases.push(Case {
        id: "image_svg_icon",
        view: View::image("lucide:bell"),
        note: "ready（lucide 内置 svg 源）",
    });
    cases.push(Case {
        id: "slider",
        view: View::slider(0.0..=100.0, 40.0).build(),
        note: "ready",
    });
    cases.push(Case {
        id: "progress_bar",
        view: View::progress_bar(0.6),
        note: "ready",
    });
    cases.push(Case {
        id: "list",
        view: View::list(vec![View::text("l1"), View::text("l2")])
            .spacing(2)
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "table",
        view: View::table(
            vec![View::text("H1"), View::text("H2")],
            vec![
                vec![View::text("a"), View::text("b")],
                vec![View::text("c"), View::text("d")],
            ],
        )
        .spacing(4)
        .col_spacing(8)
        .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "tabs",
        view: View::tabs(vec!["一".into(), "二".into()]).build(),
        note: "ready",
    });
    cases.push(Case {
        id: "sidebar",
        view: View::sidebar(View::text("侧栏"), 160.0).build(),
        note: "ready",
    });
    cases.push(Case {
        id: "custom_unknown_widget",
        view: View::Custom {
            name: "probe-widget".to_string(),
            props: vec![("label".to_string(), "x".to_string())],
            events: Vec::new(),
            style: None,
        },
        note: "ready（未注册 factory → 防御臂）",
    });
    cases.push(Case {
        id: "canvas_scene",
        view: View::Canvas {
            scene: crate::ui::view::CanvasScene::default(),
            logical_extent: Some((320.0, 200.0)),
            clear: None,
            on_pen_start: None,
            on_pen_move: None,
            on_pen_end: None,
            on_hit: None,
            style: Style::parse("w-full h-full").ok(),
        },
        note: "ready（无 scene 数据 → 空画布）",
    });
    cases.push(Case {
        id: "terminal_grid",
        view: {
            let key = "probe-terminal";
            let lines = vec!["hello world".to_string(), "第二行 中文".to_string()];
            let core = crate::ui::terminal::terminal(key, 40, 6);
            crate::ui::terminal::terminal_feed(core, &lines);
            View::Terminal {
                key: key.to_string(),
                cols: 40,
                rows: 6,
                lines,
                scroll_offset: 0,
                preedit: None,
                on_select: None,
                on_menu: None,
                on_input: None,
                cursor_row: 0,
                cursor_col: 0,
                history: 0,
                scheme: crate::ui::terminal::TERMINAL_SCHEME_FOLLOW_THEME,
                shortcuts: Vec::new(),
                style: None,
            }
        },
        note: "ready（registry feed）",
    });
    cases.push(Case {
        id: "autodown_editor",
        view: View::AutodownEditor {
            key: "probe-autodown".to_string(),
            value: "# 标题\n\n正文段落中文。".to_string(),
            is_final: true,
            on_change: None,
            on_focus: None,
            on_link: None,
            placeholder: None,
            style: None,
        },
        note: "ready（value 差分重建）",
    });
    cases.push(Case {
        id: "anchor_slot_child",
        view: View::AnchorSlot {
            index: 0,
            child: Box::new(View::text("块锚")),
        },
        note: "ready",
    });
    cases.push(Case {
        id: "scroll_anchor_mark",
        view: View::ScrollAnchorMark {
            pane: std::sync::Arc::from("probe-pane"),
            key: std::sync::Arc::from("mark"),
            child: Box::new(View::text("锚点")),
        },
        note: "ready",
    });

    cases.push(Case {
        id: "video_placeholder",
        view: View::Video {
            src: "probe://missing.mp4".to_string(),
            paused: true,
            position: None,
            volume: 100,
            muted: false,
            rate: 1.0,
            label: "视频".to_string(),
            on_time_update: None,
            on_loaded_metadata: None,
            on_play_state: None,
            on_ended: None,
            on_media_error: None,
            media_key: None,
            style: None,
        },
        note: "媒体引擎缺席 → 降级面（非真实解码帧）",
    });
    cases.push(Case {
        id: "remote_frame",
        view: View::RemoteFrame {
            url: "probe://remote".to_string(),
            style: None,
        },
        note: "无 publisher → 占位（非真实远端帧）",
    });
    cases.push(Case {
        id: "managed_scroll_content",
        view: View::ManagedScrollContent {
            key: "probe-managed-scroll".to_string(),
            logical_w: 1000.0,
            logical_h: 5000.0,
            axes: crate::ui::scroll::ScrollAxes::Y,
        },
        note: "ready（synthetic managed scroll）",
    });
    cases.push(Case {
        id: "accordion",
        view: View::accordion()
            .items(vec![
                crate::ui::view::AccordionItem::new("段一")
                    .with_children(vec![View::text("内容一")])
                    .with_expanded(true),
                crate::ui::view::AccordionItem::new("段二"),
            ])
            .build(),
        note: "ready",
    });
    cases.push(Case {
        id: "navigation_rail",
        view: View::navigation_rail()
            .items(vec![
                crate::ui::view::NavigationRailItem::new('A', "首页"),
                crate::ui::view::NavigationRailItem::new('B', "设置"),
            ])
            .build(),
        note: "ready",
    });

    // 宿主依赖变体：可在两侧渲染但不代表真实宿主内容（记录清晰限制）。
    cases.push(Case {
        id: "image_surface_uri",
        view: View::ImageSurface {
            src: "probe://missing".to_string(),
            alt: "surface".to_string(),
            width: 120,
            height: 80,
            quality: 90,
            fit: "contain".to_string(),
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0,
            filter: String::new(),
            on_error: None,
            on_loaded: None,
            on_wheel: None,
            on_pan: None,
            on_double_click: None,
            style: None,
        },
        note: "宿主/媒体通道缺席 → 占位面（非真实图像内容）",
    });
    cases.push(Case {
        id: "window_thumbnail",
        view: View::WindowThumbnail {
            wid: String::new(),
            fallback_icon: String::new(),
            style: None,
        },
        note: "宿主快照缺席 → 占位（非真实缩略图）",
    });
    cases.push(Case {
        id: "workspace_preview",
        view: View::WorkspacePreview {
            ws: String::new(),
            fallback_icon: String::new(),
            style: None,
        },
        note: "宿主合成缺席 → 占位（非真实预览）",
    });

    cases
}

// ---------------------------------------------------------------------------
// 3. 计时/计数
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub(super) struct Stats {
    pub(super) n: usize,
    pub(super) p50: f64,
    pub(super) p95: f64,
    pub(super) p99: f64,
    pub(super) max: f64,
    pub(super) mean: f64,
}

pub(super) fn stats(mut xs: Vec<f64>) -> Stats {
    assert!(!xs.is_empty(), "stats requires samples");
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = xs.len();
    let pick = |q: f64| -> f64 {
        let idx = ((n as f64 - 1.0) * q).round() as usize;
        xs[idx.min(n - 1)]
    };
    Stats {
        n,
        p50: pick(0.50),
        p95: pick(0.95),
        p99: pick(0.99),
        max: *xs.last().unwrap(),
        mean: xs.iter().sum::<f64>() / n as f64,
    }
}

fn stats_json(s: Stats) -> serde_json::Value {
    serde_json::json!({
        "n": s.n, "p50": s.p50, "p95": s.p95, "p99": s.p99,
        "max": s.max, "mean": s.mean,
    })
}

pub(super) fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// 直接渲染帧（offscreen wgpu 代理终点）：
/// - `startup`：Simulator 构建（含 wgpu 适配器/设备初始化 + 首次布局），
///   属冷启动量级（M-06 代理；非稳态帧成本）。
/// - `frame`：同一 Simulator 上的 snapshot（update(RedrawRequested) + draw +
///   GPU 读回）稳态 p50/p95——M-02/M-03 的帧级代理，不是物理上屏。
pub(super) fn direct_frames(view: View<()>, png: Option<&Path>) -> Result<(f64, Stats), String> {
    let t = Instant::now();
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(PROBE_W as f32, PROBE_H as f32),
        view.into_iced(),
    );
    let startup = ms(t.elapsed());
    let mut frame_ms = Vec::new();
    for i in 0..(WARMUP + SAMPLES) {
        let t2 = Instant::now();
        let snap = sim
            .snapshot(&theme())
            .map_err(|e| format!("direct snapshot: {e}"))?;
        let frame = ms(t2.elapsed());
        if i == 0 {
            if let Some(p) = png {
                snap.matches_image(p).map_err(|e| format!("direct png: {e}"))?;
            }
        }
        if i >= WARMUP {
            frame_ms.push(frame);
        }
    }
    Ok((startup, stats(frame_ms)))
}

/// RQ 产帧 + 编码 + 解码 + 槽复制 + 重放帧（broker 同路 canvas）。
struct RqMeasure {
    ops: usize,
    text_runs: usize,
    quad_ops: usize,
    image_ops: usize,
    wire_bytes: usize,
    bitmaps: usize,
    bitmap_bytes: usize,
    observations: Vec<String>,
    produce_ms: Stats,
    lower_ms: Stats,
    encode_ms: Stats,
    decode_ms: Stats,
    copy_ms: Stats,
    replay_build_ms: f64,
    replay_frame_ms: Stats,
}

fn rq_measure(view: View<()>, png: Option<&Path>) -> Result<RqMeasure, String> {
    let mut surface = HeadlessSurface::new(StaticComponent { view }, PROBE_W as f32, PROBE_H as f32);
    let mut produce = Vec::new();
    let mut lower = Vec::new();
    let mut encode = Vec::new();
    let mut decode = Vec::new();
    let mut copy = Vec::new();
    let mut replay_build: Option<f64> = None;
    let mut replay_frame = Vec::new();
    let mut replay_sim: Option<iced_test::simulator::Simulator<'_, (), iced::Theme, iced::Renderer>> =
        None;
    let mut ops = 0usize;
    let mut text_runs = 0usize;
    let mut quad_ops = 0usize;
    let mut image_ops = 0usize;
    let mut wire_bytes = 0usize;
    let mut bitmaps = 0usize;
    let mut bitmap_bytes = 0usize;
    let mut observations: Vec<String> = Vec::new();

    for i in 0..(WARMUP + SAMPLES) {
        let t = Instant::now();
        let layers = surface.drive_and_draw();
        let produce_d = t.elapsed();

        let t = Instant::now();
        let mut obs: Vec<String> = Vec::new();
        let mut bms: Vec<super::super::endpoint::BitmapUpload> = Vec::new();
        let mut cache = ResourceCache::default();
        let list = lower_layers_v2(&layers, &mut obs, &mut cache, &mut bms, false);
        let lower_d = t.elapsed();

        let t = Instant::now();
        let mut bytes = Vec::new();
        list.encode(&mut bytes);
        let encode_d = t.elapsed();

        let t = Instant::now();
        let mut reader = super::super::codec::Reader::new(&bytes);
        let decoded = super::super::message::DisplayList::decode(&mut reader)
            .map_err(|e| format!("decode: {e:?}"))?;
        let decode_d = t.elapsed();

        // 槽复制：与 `SharedFrameBuffer::write_slot` 同量级（payload memcpy）。
        let t = Instant::now();
        let mut slot = vec![0u8; bytes.len()];
        slot.copy_from_slice(&bytes);
        let copy_d = t.elapsed();
        std::hint::black_box(&slot);

        // 重放：broker 同路 canvas。首个解码帧建一次 Simulator（含设备初始化），
        // 其余帧复用同一设备做稳态 snapshot（每帧重建元素属 rq_view 生产形态，
        // 此处设备复用只影响测点划分，见报告口径说明）。
        if replay_sim.is_none() {
            let t = Instant::now();
            // 对照口径：重放去掉 wire 自带 clear（本机 direct 基准用主题底色），
            // 保证两侧背景同源；clear 色差本身单列为保真发现（见报告）。
            let mut replay_list = decoded.clone();
            replay_list.clear = None;
            replay_sim = Some(iced_test::simulator::Simulator::with_size(
                iced::Settings::default(),
                iced::Size::new(PROBE_W as f32, PROBE_H as f32),
                crate::ui::iced::broker_surface::displaylist_element::<()>(&replay_list),
            ));
            replay_build = Some(ms(t.elapsed()));
        }
        let sim = replay_sim.as_mut().expect("replay simulator");
        let t = Instant::now();
        let snap = sim
            .snapshot(&theme())
            .map_err(|e| format!("rq snapshot: {e}"))?;
        let rf = ms(t.elapsed());

        if i == 0 {
            if let Some(p) = png {
                snap.matches_image(p).map_err(|e| format!("rq png: {e}"))?;
            }
            observations = obs.clone();
            bitmaps = bms.len();
            bitmap_bytes = bms.iter().map(|b| b.rgba.len()).sum();
        }
        if i >= WARMUP {
            produce.push(ms(produce_d));
            lower.push(ms(lower_d));
            encode.push(ms(encode_d));
            decode.push(ms(decode_d));
            copy.push(ms(copy_d));

            replay_frame.push(rf);
            ops = decoded.ops.len();
            text_runs = decoded
                .ops
                .iter()
                .filter(|o| matches!(o, super::super::message::DisplayOp::TextStyled { .. }))
                .count();
            quad_ops = decoded
                .ops
                .iter()
                .filter(|o| matches!(o, super::super::message::DisplayOp::Quad { .. }))
                .count();
            image_ops = decoded
                .ops
                .iter()
                .filter(|o| matches!(o, super::super::message::DisplayOp::Image { .. }))
                .count();
            wire_bytes = bytes.len();
        }
    }

    Ok(RqMeasure {
        ops,
        text_runs,
        quad_ops,
        image_ops,
        wire_bytes,
        bitmaps,
        bitmap_bytes,
        observations,
        produce_ms: stats(produce),
        lower_ms: stats(lower),
        encode_ms: stats(encode),
        decode_ms: stats(decode),
        copy_ms: stats(copy),
        replay_build_ms: replay_build.unwrap_or(0.0),
        replay_frame_ms: stats(replay_frame),
    })
}

fn json_case(id: &str, mode: &str, extra: &str) -> String {
    format!("[RQPROBE] {{\"case\":\"{id}\",\"mode\":\"{mode}\"{extra}}}")
}

// ---------------------------------------------------------------------------
// 4. 测试入口（T-01 基线 + 计数；串行执行，不并发争用 GPU）
// ---------------------------------------------------------------------------

#[test]
fn rq_probe_t01_capability_denominator_is_complete() {
    let cases = corpus();
    let ids: Vec<&str> = cases.iter().map(|c| c.id).collect();
    assert_eq!(
        ids.len(),
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        "corpus ids 必须唯一"
    );
    // 全部语料可达能力 ID（穷尽 match 的编译期完整性由函数本体保证）。
    for c in &cases {
        let id = capability_id(&c.view);
        assert!(id.starts_with("cap.view."), "非法能力 ID: {id}");
        assert!(
            ALL_VIEW_CAPABILITY_IDS.contains(&id),
            "能力 {id} 未登记进 ALL_VIEW_CAPABILITY_IDS（V-01 双向对账）"
        );
    }
    println!(
        "[RQPROBE] {}",
        serde_json::json!({
            "mode": "denominator",
            "view_variants": ALL_VIEW_CAPABILITY_IDS.len(),
            "corpus_cases": cases.len(),
            "capability_ids": ALL_VIEW_CAPABILITY_IDS.to_vec(),
        })
    );
}

#[test]
fn rq_probe_t01_direct_and_rq_baseline() {
    let out = out_dir();
    std::fs::create_dir_all(&out).expect("create probe out dir");
    let mut summary = Vec::new();
    for case in corpus() {
        let direct_png = out.join(format!("{}-direct.png", case.id));
        let rq_png = out.join(format!("{}-rq.png", case.id));

        let direct = direct_frames(case.view.clone(), Some(&direct_png));
        let rq = rq_measure(case.view.clone(), Some(&rq_png));

        match (&direct, &rq) {
            (Ok((cold_build, frame)), Ok(r)) => {
                println!(
                    "[RQPROBE] {}",
                    serde_json::json!({
                        "mode": "case",
                        "case": case.id,
                        "note": case.note,
                        "direct": { "startup_ms": cold_build, "frame": stats_json(*frame) },
                        "rq": {
                            "ops": r.ops, "quad_ops": r.quad_ops, "text_runs": r.text_runs,
                            "image_ops": r.image_ops, "wire_bytes": r.wire_bytes,
                            "bitmaps": r.bitmaps, "bitmap_bytes": r.bitmap_bytes,
                            "produce_ms": stats_json(r.produce_ms), "lower_ms": stats_json(r.lower_ms),
                            "encode_ms": stats_json(r.encode_ms), "decode_ms": stats_json(r.decode_ms),
                            "copy_ms": stats_json(r.copy_ms),
                            "replay_build_ms": r.replay_build_ms,
                            "replay_frame_ms": stats_json(r.replay_frame_ms),
                            "observations": r.observations,
                        }
                    })
                );
                summary.push(serde_json::json!({
                    "case": case.id, "note": case.note, "ok": true,
                    "direct_startup_ms": cold_build,
                    "direct_frame_p50_ms": frame.p50,
                    "direct_frame_p95_ms": frame.p95,
                    "rq_produce_p50_ms": r.produce_ms.p50,
                    "rq_produce_p95_ms": r.produce_ms.p95,
                    "rq_lower_p50_ms": r.lower_ms.p50,
                    "rq_encode_p50_ms": r.encode_ms.p50,
                    "rq_decode_p50_ms": r.decode_ms.p50,
                    "rq_copy_p50_ms": r.copy_ms.p50,
                    "rq_replay_p50_ms": r.replay_frame_ms.p50,
                    "rq_replay_build_ms": r.replay_build_ms,
                    "ops": r.ops, "wire_bytes": r.wire_bytes,
                    "text_runs": r.text_runs, "bitmaps": r.bitmaps,
                    "observations": r.observations,
                }));
            }
            _ => {
                let d = direct.err().unwrap_or_default();
                let r = rq.err().unwrap_or_default();
                println!(
                    "[RQPROBE] {}",
                    serde_json::json!({
                        "mode": "case", "case": case.id, "note": case.note,
                        "error": { "direct": d, "rq": r },
                    })
                );
                summary.push(serde_json::json!({
                    "case": case.id, "note": case.note, "ok": false,
                    "error": format!("direct={d}; rq={r}"),
                }));
            }
        }
    }
    std::fs::write(
        out.join("t01_baseline.json"),
        serde_json::to_string_pretty(&summary).unwrap(),
    )
    .expect("write baseline json");
    // 非零用例：语料里至少多数可渲染，否则说明探针环境失效。
    let ok = summary
        .iter()
        .filter(|s| s["ok"].as_bool().unwrap_or(false))
        .count();
    assert!(
        ok * 2 >= summary.len(),
        "可渲染用例不足一半：{ok}/{}",
        summary.len()
    );
}

// ---------------------------------------------------------------------------
// 5. 直接交互基准（输入 → 消息 → 状态 → 帧）：T-01 冻结旧 direct 交互面
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Counter {
    count: u32,
}

#[derive(Clone, Debug)]
enum CounterMsg {
    Inc,
}

impl Component for Counter {
    type Msg = CounterMsg;

    fn on(&mut self, msg: CounterMsg) {
        match msg {
            CounterMsg::Inc => self.count += 1,
        }
    }

    fn view(&self) -> View<CounterMsg> {
        View::col()
            .spacing(4)
            .child(View::Button {
                label: "inc".to_string(),
                onclick: CounterMsg::Inc,
                style: Style::parse("px-3 py-1 rounded bg-blue-500 text-white").ok(),
                on_right_click: None,
                content: None,
                disabled: false,
            })
            .child(View::text(self.count.to_string()))
            .build()
    }
}

/// 旧 direct 交互基准（输入 → 消息 → 状态 → 帧）：经 headless 录制器的
/// 真实 iced 事件路径点击计数按钮，断言组件状态与下一帧结构可见变化；
/// 两侧帧另存 PNG（direct 参考 = 同状态 View 直接渲染）。
#[test]
fn rq_probe_t01_interaction_baseline() {
    use super::super::message::DisplayOp;

    let mut surface = HeadlessSurface::new(Counter { count: 0 }, 240.0, 160.0);
    let list0 = surface.render_frame_v2();
    let text0 = list0.ops.iter().any(|o| {
        matches!(
            o,
            DisplayOp::TextStyled { text, .. } if text.as_str() == "0"
        )
    });
    assert!(text0, "初始帧应含计数文本 0");

    // 按钮 quad（宽 > 40px 的浅色块）中心作为点击点。
    let center = list0
        .ops
        .iter()
        .find_map(|o| match o {
            DisplayOp::Quad { rect, .. } if rect.w > 40.0 => {
                Some((rect.x + rect.w / 2.0, rect.y + rect.h / 2.0))
            }
            _ => None,
        })
        .expect("应能找到按钮 quad");

    surface.cursor = iced::mouse::Cursor::Available(iced::Point::new(center.0, center.1));
    let events = vec![
        iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: iced::Point::new(center.0, center.1),
        }),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ];
    let _ = surface.update(&events);
    assert_eq!(surface.component.count, 1, "点击按钮后组件状态应为 1");

    let list1 = surface.render_frame_v2();
    let text1 = list1.ops.iter().any(|o| {
        matches!(
            o,
            DisplayOp::TextStyled { text, .. } if text.as_str() == "1"
        )
    });
    assert!(text1, "点击后帧应含计数文本 1");

    // 帧产物留档（direct 参考 vs 交互后重放）。
    let out = out_dir();
    std::fs::create_dir_all(&out).expect("create probe out dir");
    let direct_ref = out.join("interaction_count1-direct.png");
    let _ = direct_frames(
        View::text("1"),
        Some(direct_ref.as_path()),
    )
    .map_err(|e| eprintln!("[RQPROBE] interaction direct ref skipped: {e}"));
    for (tag, list) in [("interaction_click0", &list0), ("interaction_click1", &list1)] {
        let mut sim = iced_test::simulator::Simulator::with_size(
            iced::Settings::default(),
            iced::Size::new(240.0, 160.0),
            crate::ui::iced::broker_surface::displaylist_element::<()>(list),
        );
        let snap = sim.snapshot(&theme()).expect("interaction replay frame");
        snap.matches_image(out.join(format!("{tag}-rq.png")))
            .expect("write interaction png");
    }
    println!(
        "[RQPROBE] {}",
        serde_json::json!({
            "mode": "interaction",
            "case": "counter_click_via_iced_events",
            "click_point": [center.0, center.1],
            "state_after_click": surface.component.count,
            "ops_before": list0.ops.len(),
            "ops_after": list1.ops.len(),
            "result": "click -> count=1 -> next frame text 1",
        })
    );
}

// ---------------------------------------------------------------------------
// 6. wire 编解码/复制成本随载荷规模（M-04 代理；同机 CPU 串行）
// ---------------------------------------------------------------------------

fn display_list_with_text_ops(n: usize) -> super::super::message::DisplayList {
    use super::super::message::{DisplayList, DisplayOp, Rgba8};
    let ops = (0..n)
        .map(|i| DisplayOp::TextStyled {
            x: (i % 40) as f32 * 8.0,
            y: (i / 40) as f32 * 16.0,
            size: 14.0,
            line_height: 16.0,
            color: Rgba8::new(255, 255, 255, 255),
            weight: 400,
            italic: false,
            text: format!("run-{i:05} 文本样本 abcdefghij"),
        })
        .collect();
    DisplayList { clear: None, ops }
}

#[test]
fn rq_probe_t01_wire_cost_scaling() {
    let reps = 20usize;
    for n in [128usize, 1024, 4096] {
        let list = display_list_with_text_ops(n);
        let mut encoded = Vec::new();
        list.encode(&mut encoded);
        let bytes = encoded.len();

        let mut enc = Vec::new();
        let mut dec = Vec::new();
        let mut cpy = Vec::new();
        for _ in 0..reps {
            let t = Instant::now();
            let mut buf = Vec::new();
            list.encode(&mut buf);
            enc.push(ms(t.elapsed()));
            std::hint::black_box(&buf);

            let t = Instant::now();
            let mut r = super::super::codec::Reader::new(&encoded);
            let d = super::super::message::DisplayList::decode(&mut r).expect("decode");
            dec.push(ms(t.elapsed()));
            std::hint::black_box(&d);

            let t = Instant::now();
            let mut slot = vec![0u8; bytes];
            slot.copy_from_slice(&encoded);
            cpy.push(ms(t.elapsed()));
            std::hint::black_box(&slot);
        }
        let (e, d, c) = (stats(enc), stats(dec), stats(cpy));
        let mb = bytes as f64 / (1024.0 * 1024.0);
        println!(
            "[RQPROBE] {}",
            serde_json::json!({
                "mode": "wire_cost",
                "ops": n,
                "bytes": bytes,
                "per_op_bytes": bytes as f64 / n as f64,
                "encode_p50_ms": e.p50, "encode_p95_ms": e.p95,
                "decode_p50_ms": d.p50, "decode_p95_ms": d.p95,
                "copy_p50_ms": c.p50, "copy_p95_ms": c.p95,
                "encode_mb_s": mb / (e.p50 / 1000.0),
                "decode_mb_s": mb / (d.p50 / 1000.0),
                "copy_mb_s": mb / (c.p50 / 1000.0),
            })
        );
    }

    // Rgba 位图上传复制：lower_image_v2 对 Handle::Rgba 做 `pixels.to_vec()`，
    // 与共享槽写入同量级；测 4MiB 单次复制。
    let pixels = vec![0x7fu8; 4 * 1024 * 1024];
    let mut samples = Vec::new();
    for _ in 0..reps {
        let t = Instant::now();
        let copy = pixels.clone();
        samples.push(ms(t.elapsed()));
        std::hint::black_box(&copy);
    }
    let s = stats(samples);
    println!(
        "[RQPROBE] {}",
        serde_json::json!({
            "mode": "bitmap_copy",
            "bytes": pixels.len(),
            "copy_p50_ms": s.p50,
            "copy_mb_s": (pixels.len() as f64 / (1024.0 * 1024.0)) / (s.p50 / 1000.0),
        })
    );
}
#[test]
fn rq_probe_t01_style_class_denominator_is_complete() {
    let total: usize = style_class_families::STYLE_CLASS_FAMILY_COUNTS
        .iter()
        .map(|(_, n)| n)
        .sum();
    assert_eq!(total, style_class_families::STYLE_CLASS_TOTAL);
    assert_eq!(
        total, 218,
        "StyleClass 类数漂移：分母变化必须具名修订 Plan §5/§9"
    );
    for (fam, n) in style_class_families::STYLE_CLASS_FAMILY_COUNTS {
        assert!(*n > 0, "族 {fam} 不得为空");
    }
    // 抽检映射：真实 Style::parse 产物 → 族（穷尽 match 由编译期保证）。
    let samples = [
        ("p-4", "spacing"),
        ("bg-red-500", "paint.fill"),
        ("rounded-lg", "paint.border"),
        ("shadow-lg", "paint.shadow"),
        ("text-sm", "text"),
        ("rotate-45", "transform"),
    ];
    for (class, want) in samples {
        let style = Style::parse(class).expect("style parse");
        assert!(!style.classes.is_empty(), "{class} 应解析出至少一个类");
        let got = style_class_families::style_class_family(&style.classes[0]);
        assert_eq!(got, want, "{class} 应归 {want}，实际 {got}");
    }
    println!(
        "[RQPROBE] {}",
        serde_json::json!({
            "mode": "style_denominator",
            "classes": total,
            "families": style_class_families::STYLE_CLASS_FAMILY_COUNTS,
            "samples": samples.len(),
        })
    );
}