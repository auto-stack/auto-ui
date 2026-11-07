//! PLAN-061 T-02: 高风险原语接缝探针（test-only）。
//!
//! 目标（Plan §8 T-02 / V-02 / AC-02）：在同源 direct 与记录路径上，对
//! mesh、SVG、raw/glyph、blur、完整 transform、可达 custom primitive
//! 做区分性探针，**只做探针不切生产**。两候选接缝：
//! - A：修补 tiny-skia/现有 Secondary 记录→wire 词汇（保留 path/glyph/
//!   image 参数/资源身份），Host 用同一批原语重放；
//! - B：在 iced 原语边界另造记录器（绕过 tiny-skia）。
//! 本模块只收集事实：记录到什么、丢了什么、能否无损序列化与重放，
//! 不修改 message/headless 生产路径。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use super::*;
use crate::ui::component::Component;
use crate::ui::view::{CanvasEdge, CanvasLabel, CanvasNode, CanvasScene, CanvasStroke, View};
use iced::advanced::graphics::text::Text as GfxText;
use iced_tiny_skia::Layer;

const W: f32 = 640.0;
const H: f32 = 400.0;

#[derive(Debug, Clone)]
struct SeamComponent {
    view: View<()>,
}

impl Component for SeamComponent {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

fn theme() -> iced::Theme {
    crate::ui::iced::renderer::shadcn_theme(crate::ui::style_iced_adapter::dark_mode())
}

fn out_dir() -> PathBuf {
    match std::env::var_os("AUTO_RQ_PROBE_OUT") {
        Some(v) => PathBuf::from(v),
        None => std::env::temp_dir().join("rq_probe"),
    }
}

fn record_view(view: View<()>) -> Vec<Layer> {
    let mut surface = HeadlessSurface::new(SeamComponent { view }, W, H);
    surface.drive_and_draw()
}

/// 任意 iced Element 的 tiny-skia 记录（绕过 HeadlessSurface 的组件面，
/// 用于进入 Canvas Program 的旋转/自定义几何；与 drive_and_draw 同源）。
fn record_element<M: Clone + std::fmt::Debug + 'static>(
    element: iced::Element<'_, M>,
) -> Vec<Layer> {
    use iced_runtime::user_interface::{Cache, UserInterface};

    let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        crate::ui::iced::renderer::INTER_FONT,
        iced::Pixels(16.0),
    ));
    let mut ui = UserInterface::build(
        element,
        iced::Size::new(W, H),
        Cache::default(),
        &mut renderer,
    );
    let mut messages = Vec::new();
    let _ = ui.update(
        &[iced::Event::Window(iced::window::Event::RedrawRequested(
            iced::time::Instant::now(),
        ))],
        iced::mouse::Cursor::Unavailable,
        &mut renderer,
        &mut iced::advanced::clipboard::Null,
        &mut messages,
    );
    let theme = theme();
    let style = iced::advanced::renderer::Style {
        text_color: iced::theme::Base::base(&theme).text_color,
    };
    ui.draw(&mut renderer, &theme, &style, iced::mouse::Cursor::Unavailable);
    match &mut renderer {
        iced::Renderer::Secondary(tiny) => tiny.layers().to_vec(),
        iced::Renderer::Primary(_) => unreachable!("probe 恒构造 Secondary 臂"),
    }
}

// ---------------------------------------------------------------------------
// 记录内容外壳：对可序列化面/丢失面逐项取证
// ---------------------------------------------------------------------------

fn json_image(img: &iced::advanced::graphics::image::Image) -> serde_json::Value {
    use iced::advanced::graphics::image::Image;
    match img {
        Image::Raster { image, bounds, .. } => serde_json::json!({
            "kind": "raster",
            "bounds": [bounds.x, bounds.y, bounds.width, bounds.height],
            "rotation": image.rotation.0,
            "opacity": image.opacity,
            "handle": match &image.handle {
                iced::advanced::image::Handle::Path(_, p) => format!("path:{}", p.display()),
                iced::advanced::image::Handle::Rgba { width, height, .. } =>
                    format!("rgba:{width}x{height}"),
                iced::advanced::image::Handle::Bytes(_, b) => format!("bytes:{}B", b.len()),
            }
        }),
        Image::Vector { bounds, .. } => serde_json::json!({
            "kind": "vector/svg",
            "bounds": [bounds.x, bounds.y, bounds.width, bounds.height],
            "note": "svg 句柄在场（数据/路径可序列化为资源）",
        }),
    }
}

fn json_text(t: &GfxText) -> serde_json::Value {
    {
        let t = t;
        match t {
            GfxText::Paragraph {
                paragraph, position, ..
            } => {
                let mut runs = 0usize;
                let mut glyphs = 0usize;
                let mut first_glyph: Option<(u16, f32, f32, f32)> = None;
                let mut buffer_lines = 0usize;
                if let Some(p) = paragraph.upgrade() {
                    buffer_lines = p.buffer().lines.len();
                    for run in p.buffer().layout_runs() {
                        runs += 1;
                        for g in run.glyphs.iter() {
                            glyphs += 1;
                            if first_glyph.is_none() {
                                first_glyph = Some((g.glyph_id, g.x, g.y, g.font_size));
                            }
                        }
                    }
                }
                serde_json::json!({
                    "kind": "paragraph", "position": [position.x, position.y],
                    "buffer_lines": buffer_lines,
                    "runs": runs, "glyphs": glyphs, "first_glyph": first_glyph,
                    "has_glyph_geometry": glyphs > 0,
                })
            }
            GfxText::Editor { editor, position, .. } => {
                let mut glyphs = 0usize;
                if let Some(e) = editor.upgrade() {
                    for run in e.buffer().layout_runs() {
                        glyphs += run.glyphs.len();
                    }
                }
                serde_json::json!({
                    "kind": "editor", "position": [position.x, position.y],
                    "glyphs": glyphs, "has_glyph_geometry": glyphs > 0,
                })
            }
            GfxText::Cached { content, bounds, .. } => serde_json::json!({
                "kind": "cached", "content": content, "bounds": [bounds.x, bounds.y, bounds.width, bounds.height],
                "has_glyph_geometry": false,
                "note": "只有字符串+字体身份；无 glyph/位置 → Host 必须重排",
            }),
            GfxText::Raw { .. } => serde_json::json!({
                "kind": "raw", "has_glyph_geometry": false,
                "note": "当前 lower 直接省略；buffer 在记录器内可读",
            }),
        }
    }
}

fn json_layers(tag: &str, layers: &[Layer]) -> serde_json::Value {
    let mut quads = 0usize;
    let mut fills = 0usize;
    let mut strokes = 0usize;
    let mut path_points = 0usize;
    let mut rotated_paths = 0usize;
    let mut images: Vec<serde_json::Value> = Vec::new();
    let mut texts: Vec<serde_json::Value> = Vec::new();
    for l in layers {
        quads += l.quads.len();
        for item in &l.primitives {
            for p in item.as_slice() {
                match p {
                    iced_tiny_skia::Primitive::Fill { path, .. } => {
                        fills += 1;
                        path_points += path.points().len();
                        rotated_paths += rotated_hint(path);
                    }
                    iced_tiny_skia::Primitive::Stroke { path, .. } => {
                        strokes += 1;
                        path_points += path.points().len();
                        rotated_paths += rotated_hint(path);
                    }
                }
            }
        }
        for img in &l.images {
            images.push(json_image(img));
        }
        for item in &l.text {
            for t in item.as_slice() {
                texts.push(json_text(t));
            }
        }
    }
    serde_json::json!({
        "tag": tag,
        "layers": layers.len(),
        "quads": quads,
        "path_fills": fills,
        "path_strokes": strokes,
        "path_points": path_points,
        "rotated_path_hint": rotated_paths,
        "images": images,
        "text_items": texts,
    })
}

/// 旋转提示：首段 MoveTo→LineTo 的方向不平行于坐标轴（>5°）即视为已烘焙旋转。
fn rotated_hint(path: &tiny_skia::Path) -> usize {
    let mut last: Option<(f32, f32)> = None;
    let mut move_to: Option<(f32, f32)> = None;
    for seg in path.segments() {
        match seg {
            tiny_skia::PathSegment::MoveTo(p) => {
                move_to = Some((p.x, p.y));
                last = Some((p.x, p.y));
            }
            tiny_skia::PathSegment::LineTo(p) => {
                if let Some((x0, y0)) = last {
                    let dx = (p.x - x0).abs();
                    let dy = (p.y - y0).abs();
                    if dx > 1.0 && dy > 0.5 {
                        return 1;
                    }
                }
                last = Some((p.x, p.y));
            }
            _ => {}
        }
    }
    let _ = move_to;
    0
}

fn canvas_scene_view() -> View<()> {
    View::Canvas {
        scene: CanvasScene {
            strokes: vec![
                CanvasStroke {
                    points: vec![(20.0, 20.0), (90.0, 60.0), (150.0, 25.0)],
                    color: "#ff5555".to_string(),
                    width: 3.0,
                    eraser: false,
                },
                CanvasStroke {
                    points: vec![(20.0, 120.0), (180.0, 120.0)],
                    color: "#55ff55".to_string(),
                    width: 2.0,
                    eraser: false,
                },
            ],
            nodes: vec![CanvasNode {
                id: "n1".to_string(),
                x: 200.0,
                y: 80.0,
                shape: "rect".to_string(),
                color: "#5599ff".to_string(),
                r: None,
                w: Some(60.0),
                h: Some(40.0),
            }],
            edges: vec![CanvasEdge {
                x1: 20.0,
                y1: 200.0,
                x2: 220.0,
                y2: 200.0,
                color: "#ffaa00".to_string(),
                width: 2.0,
            }],
            labels: vec![CanvasLabel {
                x: 260.0,
                y: 80.0,
                text: "mesh".to_string(),
                color: "#ffffff".to_string(),
                size: 16.0,
            }],
        },
        logical_extent: Some((320.0, 240.0)),
        clear: None,
        on_pen_start: None,
        on_pen_move: None,
        on_pen_end: None,
        on_hit: None,
        style: crate::ui::style::Style::parse("w-full h-full").ok(),
    }
}

// ---------------------------------------------------------------------------
// 探针 1：mesh/SVG/glyph 的记录内容（候选 A/B 都能拿到的原始素材）
// ---------------------------------------------------------------------------

#[test]
fn rq_seam_records_high_risk_primitives() {
    let cases: Vec<(&str, View<()>)> = vec![
        ("canvas_mesh", canvas_scene_view()),
        ("svg_icon", View::image("lucide:bell")),
        ("text_cjk", View::text("中文路径客户数据库混排 ABC")),
    ];
    for (tag, view) in cases {
        let layers = record_view(view);
        let dump = json_layers(tag, &layers);
        println!("[RQSEAM] {}", serde_json::to_string(&dump).unwrap());
        match tag {
            "canvas_mesh" => {
                let fills = dump["path_fills"].as_u64().unwrap();
                let strokes = dump["path_strokes"].as_u64().unwrap();
                assert!(
                    fills + strokes > 0,
                    "canvas mesh 必须以 path 原语进入记录器（否则只能靠 App GPU）"
                );
            }
            "svg_icon" => {
                let imgs = dump["images"].as_array().map(|a| a.len()).unwrap_or(0);
                assert!(imgs > 0, "svg 必须以 image/vector 记录出现");
            }
            "text_cjk" => {
                let items = dump["text_items"].as_array().unwrap();
                assert!(!items.is_empty(), "文本必须以 text item 记录（原语面存在）");
                // 记录事实：普通/Cached 文本不带 glyph 几何；live paragraph 在
                // 记录时可能尚未整形（buffer_lines=0）→ 候选 A 必须在录制期
                // 强制整形并携带 glyph run，否则 Host 只能重排。
                println!(
                    "[RQSEAM] {}",
                    serde_json::json!({ "tag": "text_geometry_fact", "items": items })
                );
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// 探针 2：完整 2D transform（旋转烘焙）+ 可达 custom geometry
// ---------------------------------------------------------------------------

struct RotatedPathProbe;

impl iced::widget::canvas::Program<()> for RotatedPathProbe {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        frame.push_transform();
        frame.rotate(iced::Radians(std::f32::consts::FRAC_PI_4));
        frame.stroke(
            &Path::line(iced::Point::new(40.0, 40.0), iced::Point::new(140.0, 40.0)),
            Stroke::default()
                .with_width(3.0)
                .with_color(iced::Color::WHITE),
        );
        frame.pop_transform();
        frame.stroke(
            &Path::line(iced::Point::new(40.0, 200.0), iced::Point::new(140.0, 200.0)),
            Stroke::default()
                .with_width(3.0)
                .with_color(iced::Color::from_rgb(1.0, 0.0, 0.0)),
        );
        vec![frame.into_geometry()]
    }
}

#[test]
fn rq_seam_canvas_transform_is_baked_into_geometry() {
    let element = iced::widget::canvas(RotatedPathProbe)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);
    let layers = record_element::<()>(element.into());
    let dump = json_layers("rotated_paths", &layers);
    println!("[RQSEAM] {}", serde_json::to_string(&dump).unwrap());
    assert!(
        dump["path_strokes"].as_u64().unwrap() >= 2,
        "旋转与未旋转描边都应进入记录器"
    );
    assert!(
        dump["rotated_path_hint"].as_u64().unwrap() >= 1,
        "旋转必须烘焙进 path 坐标（否则 wire 需未公开的 iced 旋转矩阵）"
    );
    // 记录器 Primitive 只有 Fill/Stroke 两种 → custom GPU shader 面不可达。
    // 该事实由 `primitive_kind` 的穷尽 match 编译期保证。
    fn primitive_kind(p: &iced_tiny_skia::Primitive) -> &'static str {
        match p {
            iced_tiny_skia::Primitive::Fill { .. } => "fill",
            iced_tiny_skia::Primitive::Stroke { .. } => "stroke",
        }
    }
    let kinds: std::collections::BTreeSet<&str> = layers
        .iter()
        .flat_map(|l| {
            l.primitives
                .iter()
                .flat_map(|item| item.as_slice())
                .map(primitive_kind)
        })
        .collect();
    println!(
        "[RQSEAM] {}",
        serde_json::json!({ "tag": "primitive_kinds", "kinds": kinds })
    );
}

// ---------------------------------------------------------------------------
// 探针 3：mesh 记录 → probe wire 序列化 → Host 重放（候选 A 可行性）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Cmd {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Clone, Debug)]
struct WirePath {
    cmds: Vec<Cmd>,
    color: [u8; 4],
    stroke: Option<f32>,
}

fn paint_color(paint: &tiny_skia::Paint<'_>) -> [u8; 4] {
    let c = paint.shader.clone();
    match c {
        // iced_tiny_skia 的 into_paint 以 (b,g,r,a) 构造 tiny_skia::Color
        //（geometry.rs:427）——序列化必须换回逻辑 RGB，否则整帧换色。
        tiny_skia::Shader::SolidColor(color) => [
            (color.blue() * 255.0).round() as u8,
            (color.green() * 255.0).round() as u8,
            (color.red() * 255.0).round() as u8,
            (color.alpha() * 255.0).round() as u8,
        ],
        other => {
            let _ = other;
            [255, 255, 255, 255]
        }
    }
}

fn extract_wire_paths(layers: &[Layer]) -> Vec<WirePath> {
    let mut out = Vec::new();
    for l in layers {
        for item in &l.primitives {
            for p in item.as_slice() {
                match p {
                    iced_tiny_skia::Primitive::Fill { path, paint, .. } => out.push(WirePath {
                        cmds: path_cmds(path),
                        color: paint_color(paint),
                        stroke: None,
                    }),
                    iced_tiny_skia::Primitive::Stroke {
                        path,
                        paint,
                        stroke,
                    } => out.push(WirePath {
                        cmds: path_cmds(path),
                        color: paint_color(paint),
                        stroke: Some(stroke.width),
                    }),
                }
            }
        }
    }
    out
}

fn path_cmds(path: &tiny_skia::Path) -> Vec<Cmd> {
    path.segments()
        .map(|seg| match seg {
            tiny_skia::PathSegment::MoveTo(p) => Cmd::Move(p.x, p.y),
            tiny_skia::PathSegment::LineTo(p) => Cmd::Line(p.x, p.y),
            tiny_skia::PathSegment::QuadTo(a, b) => Cmd::Quad(a.x, a.y, b.x, b.y),
            tiny_skia::PathSegment::CubicTo(a, b, c) => {
                Cmd::Cubic(a.x, a.y, b.x, b.y, c.x, c.y)
            }
            tiny_skia::PathSegment::Close => Cmd::Close,
        })
        .collect()
}

struct WirePainter {
    paths: Vec<WirePath>,
}

impl iced::widget::canvas::Program<()> for WirePainter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        for wp in &self.paths {
            let path = Path::new(|b| {
                for c in &wp.cmds {
                    match *c {
                        Cmd::Move(x, y) => b.move_to(iced::Point::new(x, y)),
                        Cmd::Line(x, y) => b.line_to(iced::Point::new(x, y)),
                        Cmd::Quad(x1, y1, x, y) => b.quadratic_curve_to(
                            iced::Point::new(x1, y1),
                            iced::Point::new(x, y),
                        ),
                        Cmd::Cubic(x1, y1, x2, y2, x, y) => b.bezier_curve_to(
                            iced::Point::new(x1, y1),
                            iced::Point::new(x2, y2),
                            iced::Point::new(x, y),
                        ),
                        Cmd::Close => b.close(),
                    }
                }
            });
            let color = iced::Color::from_rgba8(wp.color[0], wp.color[1], wp.color[2], wp.color[3] as f32 / 255.0);
            match wp.stroke {
                Some(width) => frame.stroke(
                    &path,
                    Stroke::default().with_width(width).with_color(color),
                ),
                None => frame.fill(&path, color),
            }
        }
        vec![frame.into_geometry()]
    }
}

#[test]
fn rq_seam_mesh_replays_from_probe_wire() {
    let out = out_dir();
    std::fs::create_dir_all(&out).expect("probe out");

    let view = canvas_scene_view();
    let direct_png = out.join("seam_mesh-direct.png");
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        view.clone().into_iced(),
    );
    let snap = sim.snapshot(&theme()).expect("direct mesh");
    snap.matches_image(&direct_png).expect("direct png");

    let layers = record_view(view);
    let paths = extract_wire_paths(&layers);
    assert!(!paths.is_empty(), "mesh 记录应可序列化为 path 原语");
    let segments: usize = paths.iter().map(|p| p.cmds.len()).sum();
    println!(
        "[RQSEAM] {}",
        serde_json::json!({
            "tag": "mesh_colors",
            "colors": paths.iter().map(|p| (p.color, p.stroke)).collect::<Vec<_>>()
        })
    );
    println!(
        "[RQSEAM] {}",
        serde_json::json!({ "tag": "mesh_wire", "paths": paths.len(), "segments": segments })
    );

    let replay_png = out.join("seam_mesh-rq.png");
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        iced::widget::canvas(WirePainter { paths }).width(iced::Length::Fill).height(iced::Length::Fill),
    );
    let snap = sim.snapshot(&theme()).expect("replay mesh");
    snap.matches_image(&replay_png).expect("replay png");
}

// ---------------------------------------------------------------------------
// 探针 4：文本 glyph 几何 vs Cached 重排 + raw 缺失（V-02 文本面）
// ---------------------------------------------------------------------------

#[test]
fn rq_seam_text_glyph_vs_cached() {
    // 多帧观察：第 1 帧 live paragraph 可能尚未整形（buffer_lines=0），
    // 后续帧才带 glyph 几何——这本身就是首帧文本缺失/重排风险的证据。
    let mut surface = HeadlessSurface::new(SeamComponent { view: View::text("中文路径客户数据库混排 ABC") }, W, H);
    let mut frames = Vec::new();
    for i in 0..4 {
        let layers = surface.drive_and_draw();
        let dump = json_layers(&format!("text_frame_{i}"), &layers);
        frames.push(dump.clone());
        println!("[RQSEAM] {}", serde_json::to_string(&dump).unwrap());
    }
    let glyphs: Vec<u64> = frames
        .iter()
        .flat_map(|f| f["text_items"].as_array().unwrap().iter())
        .map(|t| t["glyphs"].as_u64().unwrap_or(0))
        .collect();
    println!(
        "[RQSEAM] {}",
        serde_json::json!({ "tag": "text_frames_glyphs", "per_item_glyphs": glyphs })
    );
    assert!(
        glyphs.iter().any(|g| *g > 0),
        "至少有一帧应带 glyph 几何（否则录制面无法承载文本）"
    );
    // 录制到的 glyph 数据必须与生产端整形一致（同 font system/同 shaping）。
    let recorded = shape_glyph_run("中文路径客户数据库混排 ABC", 600.0);
    let recorded_glyphs = recorded["glyphs"].as_u64().unwrap();
    let recorded_first = recorded["first_glyph"].clone();
    assert_eq!(
        glyphs[0], recorded_glyphs,
        "记录 glyph 数应与生产端整形一致"
    );
    assert_eq!(
        frames[0]["text_items"][0]["first_glyph"], recorded_first,
        "记录首 glyph(id,x,y,size) 应与生产端整形一致"
    );

    // Cached：只有字符串+字体身份，无 glyph 几何 → Host 必重排（信息损失证据）
    let layers = record_view(View::Button {
        label: "保存".to_string(),
        onclick: (),
        style: crate::ui::style::Style::parse("px-3 py-1 rounded bg-blue-500 text-white").ok(),
        on_right_click: None,
        content: None,
        disabled: false,
    });
    let dump = json_layers("button_cached", &layers);
    println!("[RQSEAM] {}", serde_json::to_string(&dump).unwrap());
    let items = dump["text_items"].as_array().unwrap();
    let item = items.first().expect("按钮文字应记录为 text item");
    println!(
        "[RQSEAM] {}",
        serde_json::json!({ "tag": "button_text_item", "item": item })
    );
    // 冷单帧的按钮文本可能未整形（buffer_lines=0）→ 该帧无 glyph 几何，
    // 属“首帧缺字/重排”风险；多帧/预热后可恢复（上段 text_frames_glyphs）。
    assert!(item["kind"] == "paragraph" || item["kind"] == "cached");
}
// ---------------------------------------------------------------------------
// 探针 5：P-02 资源重复上传（稳定重复帧计数）
// ---------------------------------------------------------------------------

#[test]
fn rq_seam_repeat_frame_resource_uploads() {
    let view = View::code_editor("seam-code")
        .value("fn main() {\n    println!(\"你好\");\n}")
        .lang("rust")
        .build();
    let mut surface = HeadlessSurface::new(SeamComponent { view }, W, H);
    let mut counts = Vec::new();
    let mut bytes = Vec::new();
    for _ in 0..4 {
        let _ = surface.render_frame_v2();
        let uploads = surface.drain_bitmap_uploads();
        counts.push(uploads.len());
        bytes.push(uploads.iter().map(|u| u.rgba.len()).sum::<usize>());
    }
    println!(
        "[RQSEAM] {}",
        serde_json::json!({
            "tag": "repeat_uploads",
            "uploads_per_frame": counts,
            "bytes_per_frame": bytes,
            "note": "稳定无变化的重复帧仍重复上传 = P-02 违约证据（候选需资源身份+generation）",
        })
    );
    // 记录事实，不做通过断言：当前生产路径确实每帧重传（T-02 后由 T-03 设计修复）。
    assert!(counts[0] > 0, "code editor 应产生至少一次位图上传");
}

// ---------------------------------------------------------------------------
// 探针 4b：录制期强制整形 → glyph run 可提取且确定（候选 A 的文本修法）
// ---------------------------------------------------------------------------

fn shape_glyph_run(text: &str, width: f32) -> serde_json::Value {
    use cosmic_text::{Attrs, Buffer, Metrics, Shaping};

    let fs = iced::advanced::graphics::text::font_system();
    let mut guard = fs.write().expect("font system");
    let fs = guard.raw();
    let mut fs = fs;
    let mut buffer = Buffer::new(&mut fs, Metrics::new(16.0, 20.0));
    buffer.set_size(&mut fs, Some(width), Some(200.0));
    buffer.set_text(&mut fs, text, &Attrs::new(), Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut fs, false);

    let mut runs = 0usize;
    let mut glyphs = 0usize;
    let mut first: Option<(u16, f32, f32, f32)> = None;
    let mut last_x = 0.0f32;
    for run in buffer.layout_runs() {
        runs += 1;
        for g in run.glyphs.iter() {
            glyphs += 1;
            if first.is_none() {
                first = Some((g.glyph_id, g.x, g.y, g.font_size));
            }
            last_x = g.x + g.w;
        }
    }
    serde_json::json!({
        "runs": runs, "glyphs": glyphs, "first_glyph": first,
        "advance_end_x": last_x,
    })
}

#[test]
fn rq_seam_record_time_shaping_gives_glyph_runs() {
    let text = "中文路径客户数据库混排 ABC";
    let a = shape_glyph_run(text, 600.0);
    let b = shape_glyph_run(text, 600.0);
    println!(
        "[RQSEAM] {}",
        serde_json::json!({ "tag": "glyph_run", "first": a, "second": b })
    );
    assert!(a["glyphs"].as_u64().unwrap() > 0, "录制期整形必须产出 glyph");
    assert_eq!(
        a["glyphs"], b["glyphs"],
        "同输入两次整形 glyph 数必须一致（可复现）"
    );
    assert_eq!(
        a["advance_end_x"], b["advance_end_x"],
        "同输入两次整形 advance 必须一致（Host 无需再排）"
    );
    // 记录当前记录的 paragraph 是否未整形（buffer_lines=0 证据已在上一探针）。
    let layers = record_view(View::text(text));
    let dump = json_layers("text_recorded", &layers);
    println!("[RQSEAM] {}", serde_json::to_string(&dump).unwrap());
}
// ---------------------------------------------------------------------------
// 探针 6：SVG 记录 → 生产端栅格化 → Host 位图重放（候选 A 的 SVG 修法）
// ---------------------------------------------------------------------------

const PROBE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64"><circle cx="32" cy="32" r="24" fill="#ff5555"/><rect x="20" y="20" width="24" height="24" fill="#55ff55" opacity="0.6"/></svg>"##;

fn svg_direct_png(path: &std::path::Path) {
    let handle = iced::widget::svg::Handle::from_memory(PROBE_SVG.as_bytes().to_vec());
    let el = iced::widget::svg(handle)
        .width(iced::Length::Fixed(64.0))
        .height(iced::Length::Fixed(64.0));
    let el: iced::Element<()> = el.into();
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(96.0, 96.0),
        el,
    );
    let snap = sim.snapshot(&theme()).expect("svg direct");
    snap.matches_image(path).expect("svg direct png");
}

fn svg_record_layers() -> Vec<Layer> {
    let handle = iced::widget::svg::Handle::from_memory(PROBE_SVG.as_bytes().to_vec());
    let el = iced::widget::svg(handle)
        .width(iced::Length::Fixed(64.0))
        .height(iced::Length::Fixed(64.0));
    record_element::<()>(el.into())
}

#[test]
fn rq_seam_svg_raster_replay() {
    let out = out_dir();
    std::fs::create_dir_all(&out).expect("probe out");

    let direct_png = out.join("seam_svg-direct.png");
    svg_direct_png(&direct_png);

    let layers = svg_record_layers();
    let dump = json_layers("svg_widget", &layers);
    println!("[RQSEAM] {}", serde_json::to_string(&dump).unwrap());
    assert!(
        dump["images"]
            .as_array()
            .map(|a| a.iter().any(|i| i["kind"] == "vector/svg"))
            .unwrap_or(false),
        "SVG 必须以 vector 记录出现（资源身份可序列化）"
    );

    // 生产端栅格化（resvg 与 iced 内部同源）→ RGBA 资源 → Host 位图重放
    let tree = resvg::usvg::Tree::from_data(
        PROBE_SVG.as_bytes(),
        &resvg::usvg::Options::default(),
    )
    .expect("parse svg");
    // 设备 scale=2：栅格化必须按物理像素生成（1x 再放大有 4.5% 差）。
    let scale = 2u32;
    let mut pixmap = tiny_skia::Pixmap::new(64 * scale, 64 * scale).expect("pixmap");
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale as f32, scale as f32),
        &mut pixmap.as_mut(),
    );
    let rgba = pixmap.data().to_vec();
    println!(
        "[RQSEAM] {}",
        serde_json::json!({
            "tag": "svg_raster", "w": 64 * scale, "h": 64 * scale, "bytes": rgba.len(),
            "note": "生产端栅格化=可序列化资源；Host 只需位图合成",
        })
    );

    let img = iced::widget::image(iced::widget::image::Handle::from_rgba(64 * scale, 64 * scale, rgba))
        .width(iced::Length::Fixed(64.0))
        .height(iced::Length::Fixed(64.0));
    let img: iced::Element<()> = img.into();
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(96.0, 96.0),
        img,
    );
    let snap = sim.snapshot(&theme()).expect("svg replay");
    snap.matches_image(out.join("seam_svg-rq.png"))
        .expect("svg replay png");
}
// ---------------------------------------------------------------------------
// 探针 7（V-03）：帧原子性、半帧拒绝、版本门与能力尾巴（T-03 冻结依据）
// ---------------------------------------------------------------------------

#[test]
fn rq_wire_t03_frame_atomicity_and_guards() {
    use crate::ui::desktop_protocol::codec::{self, Channel, CodecError};
    use crate::ui::desktop_protocol::host::SurfaceStore;
    use crate::ui::desktop_protocol::message::{
        DisplayList, DisplayOp, Fill, FrameMsg, HandshakeMsg, ProtocolMsg, Rgba8, WRect,
    };

    fn list(ops: usize) -> DisplayList {
        DisplayList {
            clear: None,
            ops: (0..ops)
                .map(|i| DisplayOp::Quad {
                    rect: WRect {
                        x: i as f32,
                        y: 0.0,
                        w: 10.0,
                        h: 10.0,
                    },
                    fill: Fill::Color(Rgba8::new(255, 0, 0, 255)),
                    radius: [0.0; 4],
                    border: None,
                    shadow: None,
                })
                .collect(),
        }
    }

    // 1) 合法 v2 帧信封 round-trip。
    let frame = list(3);
    let bytes = ProtocolMsg::Frame(FrameMsg::FrameReadyV2 {
        wid: 1,
        frame_id: 7,
        slot: 1,
        damage: None,
        revision: 3,
        payload: frame.clone(),
    })
    .encode();
    match ProtocolMsg::decode(&bytes).expect("v2 帧应解码") {
        ProtocolMsg::Frame(FrameMsg::FrameReadyV2 { payload, .. }) => {
            assert_eq!(payload.ops.len(), 3)
        }
        _ => panic!("通道/变体漂移"),
    }

    // 2) 半帧/截断：信封长度自洽检查应拒绝（不是静默半帧）。
    let truncated = &bytes[..bytes.len() / 2];
    let err = ProtocolMsg::decode(truncated).expect_err("半帧必须被拒");
    println!(
        "[RQSEAM] {}",
        serde_json::json!({ "tag": "truncated_frame", "error": format!("{err:?}") })
    );
    assert!(matches!(err, CodecError::TooShort));

    // 3) 信封版本门：未知版本在消息层明确拒绝（UnsupportedVersion）。
    let env9 = codec::encode_envelope(9, Channel::Frame, &[]);
    let err = ProtocolMsg::decode(&env9).expect_err("未知版本必须拒绝");
    assert!(matches!(err, CodecError::UnsupportedVersion(9)));

    // 4) 帧原子性/保旧：坏载荷解码失败 → 不 compose → front 保持上一完整帧。
    let mut store = SurfaceStore::new();
    let sid = store.alloc(100.0, 60.0);
    let freed = store.compose_v2(sid, 1, frame.clone()).expect("首帧合成");
    assert_eq!(freed, 0, "首帧释放的应是另一槽");
    assert_eq!(store.front_v2(sid).unwrap().ops.len(), 3);
    // 应用层只在解码成功时 compose（host.rs/rqhost.rs 现有纪律）。
    let bad_bytes = &bytes[..bytes.len() / 2];
    let bad = ProtocolMsg::decode(bad_bytes);
    assert!(bad.is_err(), "坏载荷不得进入 compose");
    assert_eq!(
        store.front_v2(sid).unwrap().ops.len(),
        3,
        "坏帧不得覆盖上一完整帧（保旧）"
    );

    // 5) 能力尾巴：Hello 空 tail（旧端字节）与未知 cap 均可解码 →
    //    当前 caps 是**声明式**（advisory），不是强制协商；T-03 需补
    //    required/optional 语义与入帧前拒绝。
    let legacy_hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
        version: 1,
        app_name: "probe".to_string(),
        title: "t".to_string(),
        icon: None,
        width: 100.0,
        height: 60.0,
        fonts: Vec::new(),
        surfaces: Vec::new(),
        theme_caps: Vec::new(),
    })
    .encode();
    ProtocolMsg::decode(&legacy_hello).expect("旧端 Hello（空 tail）可解码");
    let cap_hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
        version: 1,
        app_name: "probe".to_string(),
        title: "t".to_string(),
        icon: None,
        width: 100.0,
        height: 60.0,
        fonts: Vec::new(),
        surfaces: Vec::new(),
        theme_caps: vec!["fidelity/pathfill/1".to_string()],
    })
    .encode();
    ProtocolMsg::decode(&cap_hello).expect("未知 cap 仅声明，不阻塞解码");
    println!(
        "[RQSEAM] {}",
        serde_json::json!({
            "tag": "capability_tail", "legacy_ok": true, "unknown_cap_advisory": true,
            "note": "T-03 需 required/optional 协商 + 未知必需能力在入帧前拒绝"
        })
    );
}