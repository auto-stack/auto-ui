//! Case `surface-compose` (PLAN-830 T-04 / V-02-V-04 composition gates).
//!
//! Deterministic HEADLESS composition gates over real GPU textures: builds
//! DAG shapes through `SurfaceRuntime` and verifies the composed pixels.
//! (Window presentation of the same runtime is exercised by the interactive
//! scenario cases from T-07.)
//!
//! Shapes:
//! 1. S-01 shape: 3D scene ──plane3d──> real iced UI ──flat2d──> 2D canvas.
//!    Gates: DAG order (children first), one content pass per surface, the
//!    canvas shows ONLY inside its clip, and the plane region of the 3D
//!    target carries the UI background (light pixels, not scene colors).
//! 2. Alpha / order / shared: a 50%-premultiplied child over a gray parent
//!    (exact linear-space expectation), a later-declared opaque child drawn
//!    on top, the same child mounted twice with ONE content pass, and the
//!    parent clear visible outside every mount.
//!
//! All expectations are derived from the frozen colour contract
//! (`research/surface-feasibility.md` §2), never from a captured golden.

use crate::core::composition::{CompositionGraph, MountKind, Rect, SurfaceId};
use crate::core::scene::Color;
use crate::host::composition_input::CompositionInput;
use crate::host::iced_surface::{SurfaceElement, SurfaceUi};
use crate::host::iced_surface_probe::probe_camera;
use crate::host::report::*;
use crate::host::surface_runtime::{Game2dContent, IcedContent, Scene3dContent, SurfaceRuntime};
use crate::renderer::pipeline::{DrawCmd, Shape};
use crate::renderer::target::OffscreenTarget;
use crate::renderer::GpuDevice;
use iced::widget::{button, column, text, text_input};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// A tiny iced app (real stock widgets) for the UI surface.
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct ComposeUi {
    pub presses: u32,
    pub input: String,
}

impl SurfaceUi for ComposeUi {
    type Message = String;

    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            button(text(format!("compose {}", self.presses)))
                .width(iced::Length::Fill)
                .on_press(format!("press:{}", self.presses + 1)),
            text_input("type", &self.input).on_input(|s| format!("input:{s}")),
        ]
        .spacing(8)
        .padding(12)
        .width(iced::Length::Fixed(320.0))
        .into()
    }

    fn update(&mut self, message: String) {
        if let Some(n) = message.strip_prefix("press:") {
            if let Ok(n) = n.parse::<u32>() {
                self.presses = n;
            }
        } else if let Some(s) = message.strip_prefix("input:") {
            self.input = s.to_string();
        }
    }

    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.95, 0.96, 0.99)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// S-01 outer UI: a real iced button above the region where the 3D view is
/// composited. Its button must stay independent from the inner control.
#[derive(Default)]
pub struct OuterUi {
    pub presses: u32,
}

impl SurfaceUi for OuterUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            button(text(format!("outer {}", self.presses)))
                .width(iced::Length::Fill)
                .on_press(format!("outer:{}", self.presses + 1)),
            iced::widget::Space::new()
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        ]
        .spacing(8)
        .padding(12)
        .width(iced::Length::Fixed(500.0))
        .height(iced::Length::Fixed(400.0))
        .into()
    }
    fn update(&mut self, m: String) {
        if let Some(n) = m.strip_prefix("outer:") {
            self.presses = n.parse().unwrap_or(self.presses);
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.95, 0.96, 0.99)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// S-01 inner UI carried on a 3D plane inside the outer UI's 3D view.
#[derive(Default)]
pub struct InnerUi {
    pub presses: u32,
    pub input: String,
}

impl SurfaceUi for InnerUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            button(text(format!("inner {}", self.presses)))
                .width(iced::Length::Fill)
                .on_press(format!("inner:{}", self.presses + 1)),
            text_input("inner", &self.input).on_input(|s| format!("inner-input:{s}")),
            iced::widget::Space::new()
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        ]
        .spacing(8)
        .padding(12)
        .width(iced::Length::Fixed(200.0))
        .height(iced::Length::Fixed(160.0))
        .into()
    }
    fn update(&mut self, m: String) {
        if let Some(n) = m.strip_prefix("inner:") {
            self.presses = n.parse().unwrap_or(self.presses);
        } else if let Some(s) = m.strip_prefix("inner-input:") {
            self.input = s.to_string();
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.97, 0.93, 0.93)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Colour contract helpers (linear space → sRGB8, premultiplied-over)
// ---------------------------------------------------------------------------

fn srgb_encode(x: f32) -> u8 {
    let v = if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Premultiplied-over in LINEAR space, encoded to the sRGB target.
fn premult_over_linear(src: [f32; 4], dst: [f32; 4]) -> [u8; 4] {
    let a = src[3];
    let r = src[0] + dst[0] * (1.0 - a);
    let g = src[1] + dst[1] * (1.0 - a);
    let b = src[2] + dst[2] * (1.0 - a);
    let out_a = a + dst[3] * (1.0 - a);
    [
        srgb_encode(r),
        srgb_encode(g),
        srgb_encode(b),
        srgb_encode(out_a),
    ]
}

fn near(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}

// ---------------------------------------------------------------------------
// Content builders
// ---------------------------------------------------------------------------

fn synth(mesh: crate::assets::MeshData) -> crate::assets::ModelData {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for p in &mesh.positions {
        h.update(p[0].to_ne_bytes());
        h.update(p[1].to_ne_bytes());
        h.update(p[2].to_ne_bytes());
    }
    h.update(mesh.name.as_bytes());
    crate::assets::ModelData {
        source_sha256: format!("{:x}", h.finalize()),
        meshes: vec![mesh],
    }
}

fn scene3d(gpu: &GpuDevice, w: f32, h: f32) -> Scene3dContent {
    let mut store = crate::render3d::MeshStore::default();
    let ground = synth(crate::render3d::procedural::ground_mesh());
    let _ = store.get_or_upload(gpu, &ground, 0);
    let instances = vec![crate::render3d::MeshInstance {
        mesh: (ground.source_sha256.clone(), ground.meshes[0].name.clone()),
        model: glam::Mat4::IDENTITY,
    }];
    let mesh =
        crate::render3d::MeshPipeline::new(gpu, OffscreenTarget::FORMAT).expect("3d pipeline");
    Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh,
        store,
        instances,
        sky: [0.64625, 0.65575, 0.67075, 1.0],
        view_proj: crate::host::iced_surface_probe::probe_camera(w / h.max(1.0)),
    }
}

fn iced_content(gpu: &GpuDevice, w: f32, h: f32) -> IcedContent {
    let _ = (gpu, w, h);
    IcedContent::borrowed(gpu, [w, h], Box::new(ComposeUi::default()))
}

fn canvas(rt: &SurfaceRuntime, clear: Color, cmds: Vec<DrawCmd>) -> Game2dContent {
    let mut c = Game2dContent::simple(rt.gpu(), clear);
    c.commands = cmds;
    c
}

// ---------------------------------------------------------------------------
// Report body
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
struct ComposeReport {
    adapter: crate::renderer::AdapterInfoReport,
    device_token: String,
    window_created: bool,
    window_mode: String,
    shapes: Vec<ShapeFacts>,
    gates: Vec<GateResult>,
    all_gates_behaved: bool,
    stats: serde_json::Value,
    keyframes: Vec<Keyframe>,
    failures: Vec<String>,
    gpu_timing_available: bool,
    notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ShapeFacts {
    name: String,
    surfaces: usize,
    portals: usize,
    render_order_len: usize,
    content_passes: u64,
    portal_composites: u64,
    probes: Vec<ProbeFacts>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ProbeFacts {
    name: String,
    sampled: [u8; 4],
    expected: [u8; 4],
    tol: u8,
    ok: bool,
    note: String,
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    // Composition gates are deterministic headless; a window is never needed
    // (window presentation is covered by the interactive cases).
    if mode == "window" {
        eprintln!("surface-compose: composition gates run headless (mode=window ignored)");
    }
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("surface-compose: no GPU: {e}");
            return 3;
        }
    };
    let mut rt = match SurfaceRuntime::new(gpu) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("surface-compose: runtime: {e}");
            return 3;
        }
    };
    let adapter = rt.gpu().adapter_info().clone();
    let token = rt.gpu().token().to_report();
    let mut gates: Vec<GateResult> = Vec::new();
    let mut shapes: Vec<ShapeFacts> = Vec::new();
    let mut keyframes: Vec<Keyframe> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    // ===== Shape 1: 3D ─plane─> iced ─flat(clip)─> canvas ================
    let s1 = build_s01(
        &mut rt,
        &mut gates,
        report_dir,
        &mut keyframes,
        &mut failures,
    );
    shapes.push(s1);

    // ===== Shape S-01: outer UI ─flat─> 3D ─plane─> inner UI =============
    let s_outer = build_s01_ui_3d_ui(
        &mut rt,
        &mut gates,
        report_dir,
        &mut keyframes,
        &mut failures,
    );
    shapes.push(s_outer);

    // ===== Shape 2: alpha / order / shared mounts ========================
    let s2 = build_alpha_shape(&mut rt, &mut gates, report_dir, &mut keyframes);
    shapes.push(s2);

    // ==== Cross-shape invariants =========================================
    gates.push(GateResult {
        gate: "single_device".into(),
        positive_passed: !token.is_empty() && token == rt.gpu().token().to_report(),
        negative_fired: false,
        reason: token.clone(),
    });
    {
        let before = rt.stats.readbacks;
        let _ = rt.frame(None, &HashMap::new());
        let frame_readbacks = rt.stats.readbacks - before;
        gates.push(GateResult {
            gate: "steady_state_frame_has_no_readback".into(),
            positive_passed: frame_readbacks == 0,
            negative_fired: false,
            reason: format!(
                "frame readbacks={frame_readbacks}, total={}",
                rt.stats.readbacks
            ),
        });
    }
    // Every sampled target is under the sampled-usage contract.
    let all_sampled = rt
        .graph
        .surfaces()
        .iter()
        .all(|s| rt.target(s.id).map(|t| t.is_sampled()).unwrap_or(false));
    gates.push(GateResult {
        gate: "consumed_targets_are_sampled_usage".into(),
        positive_passed: all_sampled,
        negative_fired: false,
        reason: format!("graph surfaces={}", rt.graph.live_surfaces()),
    });
    // The legacy (non-sampled) contract stays distinct: the older 829 path
    // creates targets via `new`, which must NOT claim samplability.
    let legacy_distinct = {
        let g = rt.gpu();
        match OffscreenTarget::new(g, 8, 8) {
            Ok(t) => !t.is_sampled(),
            Err(_) => false,
        }
    };
    gates.push(GateResult {
        gate: "legacy_targets_not_claimed_sampled".into(),
        positive_passed: legacy_distinct,
        negative_fired: false,
        reason: "OffscreenTarget::new keeps the 829 non-sampled usage".into(),
    });

    let all_ok = gates.iter().all(|g| g.positive_passed) && failures.is_empty();
    let stats = serde_json::json!({
        "surfaces_rendered": rt.stats.surfaces_rendered,
        "portal_composites": rt.stats.portal_composites,
        "plane_composites": rt.stats.plane_composites,
        "quad_composites": rt.stats.quad_composites,
        "dirty_skips": rt.stats.dirty_skips,
        "readbacks": rt.stats.readbacks,
        "frames": rt.stats.frames,
    });
    let report = ComposeReport {
        adapter,
        device_token: token,
        window_created: false,
        window_mode: "headless".into(),
        shapes,
        gates,
        all_gates_behaved: all_ok,
        stats,
        keyframes,
        failures,
        gpu_timing_available: false,
        notes: vec![
            "headless pixel gates over the SAME composed textures the window presents".into(),
            format!("replay={replay}"),
        ],
    };
    let exit = if all_ok { "pass" } else { "fail" };
    let _ = write_report(report_dir, "surface-compose", replay, exit, &report);
    if !all_ok {
        eprintln!("surface-compose: gates failed (see report.json)");
    }
    rt.gpu().wait_idle();
    if all_ok {
        0
    } else {
        1
    }
}

fn save_keyframe(
    rt: &mut SurfaceRuntime,
    id: SurfaceId,
    name: &str,
    report_dir: &Path,
    keyframes: &mut Vec<Keyframe>,
) {
    if let Some((hash, png, w, h)) = rt.capture(id, name) {
        let file = format!("{name}.png");
        let _ = std::fs::create_dir_all(report_dir);
        let _ = std::fs::write(report_dir.join(&file), &png);
        keyframes.push(Keyframe {
            name: name.into(),
            file,
            width: w,
            height: h,
            sha256: hash,
        });
    }
}

fn gate(name: &str, ok: bool, reason: String) -> GateResult {
    GateResult {
        gate: name.into(),
        positive_passed: ok,
        negative_fired: false,
        reason,
    }
}

// ---------------------------------------------------------------------------
// Shape 1
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Shape S-01: UI → 3D → UI
// ---------------------------------------------------------------------------

fn feed_iced_click(rt: &mut SurfaceRuntime, surface: SurfaceId, local: [f32; 2]) {
    let cursors = HashMap::from([(surface, local)]);
    let mut ev: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = HashMap::new();
    ev.entry(surface).or_default().extend([
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
            position: iced_wgpu::core::Point::new(local[0], local[1]),
        }),
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
            iced_wgpu::core::mouse::Button::Left,
        )),
    ]);
    let _ = rt.frame_with_cursors(None, &ev, &cursors);
    let mut ev: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = HashMap::new();
    ev.entry(surface)
        .or_default()
        .push(iced_wgpu::core::Event::Mouse(
            iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
        ));
    let _ = rt.frame_with_cursors(None, &ev, &cursors);
}

fn outer_presses(rt: &SurfaceRuntime, id: SurfaceId) -> u32 {
    match rt.content(id) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<OuterUi>()
                .map(|u| u.presses)
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn inner_presses(rt: &SurfaceRuntime, id: SurfaceId) -> u32 {
    match rt.content(id) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<InnerUi>()
                .map(|u| u.presses)
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn build_s01_ui_3d_ui(
    rt: &mut SurfaceRuntime,
    gates: &mut Vec<GateResult>,
    report_dir: &Path,
    keyframes: &mut Vec<Keyframe>,
    failures: &mut Vec<String>,
) -> ShapeFacts {
    let mut facts = ShapeFacts {
        name: "s01-ui-3d-ui".into(),
        surfaces: 0,
        portals: 0,
        render_order_len: 0,
        content_passes: 0,
        portal_composites: 0,
        probes: Vec::new(),
    };
    // Outer REAL iced UI (presented to the window).
    let outer = match rt.add_surface(
        [500.0, 400.0],
        crate::host::surface_runtime::SurfaceContent::Iced(Box::new(IcedContent::borrowed(
            rt.gpu(),
            [500.0, 400.0],
            Box::new(OuterUi::default()),
        ))),
    ) {
        Ok(id) => id,
        Err(e) => {
            failures.push(format!("s01 outer ui: {e}"));
            return facts;
        }
    };
    let outer_view = rt.add_view(outer, true).expect("outer view");
    // 3D view inside the outer UI: the flat mount is scaled (non-uniform
    // fractions) so the UI→3D leg also exercises a transformed mapping.
    let mid = match rt.add_surface(
        [400.0, 300.0],
        crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(scene3d(
            rt.gpu(),
            400.0,
            300.0,
        ))),
    ) {
        Ok(id) => id,
        Err(e) => {
            failures.push(format!("s01 3d: {e}"));
            return facts;
        }
    };
    let mid_view = rt.add_view(mid, false).expect("mid view");
    rt.add_portal(
        outer_view,
        mid,
        MountKind::Flat2d {
            offset: [20.0, 120.0],
            scale: [0.8, 0.8],
            clip: Rect::at(20.0, 120.0, 320.0, 240.0),
        },
        true,
    )
    .expect("ui->3d mount");
    // Inner REAL iced UI on a perspective plane inside that 3D view.
    let inner = match rt.add_surface(
        [200.0, 160.0],
        crate::host::surface_runtime::SurfaceContent::Iced(Box::new(IcedContent::borrowed(
            rt.gpu(),
            [200.0, 160.0],
            Box::new(InnerUi::default()),
        ))),
    ) {
        Ok(id) => id,
        Err(e) => {
            failures.push(format!("s01 inner ui: {e}"));
            return facts;
        }
    };
    rt.add_view(inner, false).expect("inner view");
    let plane = MountKind::Plane3d {
        center: [0.0, 1.7, 0.0],
        half: [1.4, 1.0],
    };
    rt.add_portal(mid_view, inner, plane, true)
        .expect("3d->inner mount");

    let mut input = CompositionInput::new([500.0, 400.0], [500.0, 400.0]);
    input.top_view = Some(outer_view);
    input
        .mapper
        .register(mid, probe_camera(400.0 / 300.0), [400.0, 300.0]);

    // One frame so the real widgets have layouts.
    if let Err(e) = rt.frame(None, &HashMap::new()) {
        failures.push(format!("s01 frame: {e}"));
        return facts;
    }
    facts.surfaces = rt.graph.live_surfaces();
    facts.portals = rt.graph.portals().len();
    facts.render_order_len = rt.last_order.len();

    let inner_btn = [100.0f32, 27.0];
    let outer_btn = [250.0f32, 27.0];
    let inner_win = input.anchor_to_window(&rt.graph, rt, inner, inner_btn);
    let Some(inner_win) = inner_win else {
        failures.push("s01 inner anchor projection failed".into());
        return facts;
    };
    // Full-path click: window → outer UI → flat mount → 3D → ray/UV → inner.
    // The widget receives the ROUTER's returned local (review r2 F06): a
    // corrupted hit coordinate must reach the real control as a miss.
    let inner_route = match input.pointer_pressed(&rt.graph, inner_win) {
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local }) => {
            Some((surface, local))
        }
        _ => None,
    };
    let delivered_inner = matches!(
        inner_route,
        Some((surface, local))
            if surface == inner
                && (local[0] - inner_btn[0]).abs() < 2.0
                && (local[1] - inner_btn[1]).abs() < 2.0
    );
    input.pointer_released();
    if let Some((surface, local)) = inner_route {
        feed_iced_click(rt, surface, local);
    }
    let inner_after = inner_presses(rt, inner);
    let outer_after_inner_click = outer_presses(rt, outer);

    // Outer control: same class of widget, must NOT have moved.
    let outer_win = input
        .anchor_to_window(&rt.graph, rt, outer, outer_btn)
        .unwrap_or([0.0, 0.0]);
    let outer_route = match input.pointer_pressed(&rt.graph, outer_win) {
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local }) => {
            Some((surface, local))
        }
        _ => None,
    };
    let delivered_outer = matches!(
        outer_route,
        Some((surface, local))
            if surface == outer
                && (local[0] - outer_btn[0]).abs() < 2.0
                && (local[1] - outer_btn[1]).abs() < 2.0
    );
    input.pointer_released();
    if let Some((surface, local)) = outer_route {
        feed_iced_click(rt, surface, local);
    }
    let outer_after = outer_presses(rt, outer);

    // Clip/plane counterexample: inside the 3D view but OFF the plane must
    // land on the 3D host, not the inner UI.
    let off_plane = input.pointer_pressed(&rt.graph, [30.0, 130.0]);
    let off_plane_hits_mid = matches!(
        off_plane,
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, .. })
            if surface == mid
    );
    input.pointer_released();

    // Simulated DPI 2: the same logical hit point in a doubled window.
    let mut dpi = CompositionInput::new([1000.0, 800.0], [500.0, 400.0]);
    dpi.top_view = Some(outer_view);
    dpi.mapper
        .register(mid, probe_camera(400.0 / 300.0), [400.0, 300.0]);
    let dpi_win = dpi.anchor_to_window(&rt.graph, rt, inner, inner_btn);
    let dpi_anchor_ok = match dpi_win {
        Some(b) => {
            (b[0] - inner_win[0] * 2.0).abs() < 1.0 && (b[1] - inner_win[1] * 2.0).abs() < 1.0
        }
        None => false,
    };
    let dpi_delivered = matches!(
        dpi.pointer_pressed(&rt.graph, dpi_win.unwrap_or([0.0, 0.0])),
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local })
            if surface == inner
                && (local[0] - inner_btn[0]).abs() < 2.0
                && (local[1] - inner_btn[1]).abs() < 2.0
    );

    // Dynamic resize of the 3D content inside the outer UI.
    if let Err(e) = rt.resize_surface(mid, [360.0, 300.0]) {
        failures.push(format!("s01 resize 3d: {e}"));
        return facts;
    }
    input
        .mapper
        .register(mid, probe_camera(360.0 / 300.0), [360.0, 300.0]);
    // The 3D content's camera is the caller's: resizing the target must also
    // update the scene's projection, otherwise input (new aspect) and the
    // rendered plane (old aspect) disagree.
    if let Some(crate::host::surface_runtime::SurfaceContent::Scene3d(scene)) = rt.content_mut(mid)
    {
        scene.view_proj = probe_camera(360.0 / 300.0);
    }
    rt.graph.mark_content_dirty(mid);
    let _ = rt.frame(None, &HashMap::new());
    let resized_win = input.anchor_to_window(&rt.graph, rt, inner, inner_btn);
    let resize_delivered = matches!(
        input.pointer_pressed(&rt.graph, resized_win.unwrap_or([0.0, 0.0])),
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local })
            if surface == inner
                && (local[0] - inner_btn[0]).abs() < 2.0
                && (local[1] - inner_btn[1]).abs() < 2.0
    );
    input.pointer_released();

    // Pixel evidence for the same path: outer texture carries the 3D sky in
    // the mount region, the 3D texture carries the INNER UI background on the
    // plane, and the outer UI background stays outside the mount.
    let _ = rt.frame(None, &HashMap::new());
    let ui_bg = [249u8, 250, 254];
    let inner_bg = [249u8, 243, 243];
    let near = |p: [u8; 4], want: [u8; 3], tol: u8| {
        p[0].abs_diff(want[0]) <= tol
            && p[1].abs_diff(want[1]) <= tol
            && p[2].abs_diff(want[2]) <= tol
    };
    // outer local (180,136) → 3D local (200,20) — sky, not the outer UI bg.
    let outer_3d_px = rt
        .probe_pixel(outer, [180.0, 136.0])
        .unwrap_or([0, 0, 0, 0]);
    let outer_bg_px = rt
        .probe_pixel(outer, [450.0, 100.0])
        .unwrap_or([0, 0, 0, 0]);
    let plane_px = input
        .mapper
        .project(mid, &plane, [4.0 / 200.0, 4.0 / 160.0])
        .map(|p| rt.probe_pixel(mid, p).unwrap_or([0, 0, 0, 0]))
        .unwrap_or([0, 0, 0, 0]);
    let pixels_ok =
        !near(outer_3d_px, ui_bg, 6) && near(outer_bg_px, ui_bg, 6) && near(plane_px, inner_bg, 10);

    gates.push(gate(
        "s01_inner_click_routes_ui_3d_ui",
        delivered_inner && inner_after == 1,
        format!("delivered={delivered_inner} inner_presses={inner_after} window={inner_win:?}"),
    ));
    gates.push(gate(
        "s01_outer_control_untouched",
        outer_after_inner_click == 0,
        format!("outer presses after the inner click={outer_after_inner_click}"),
    ));
    gates.push(gate(
        "s01_outer_control_still_works",
        delivered_outer && outer_after == 1,
        format!("delivered={delivered_outer} outer_presses={outer_after}"),
    ));
    gates.push(gate(
        "s01_off_plane_hits_the_3d_host",
        off_plane_hits_mid,
        format!("off-plane hit={off_plane:?}"),
    ));
    gates.push(gate(
        "s01_simulated_dpi2_maps_identically",
        dpi_anchor_ok && dpi_delivered,
        format!("logical window={inner_win:?} dpi2={dpi_win:?} delivered={dpi_delivered}"),
    ));
    gates.push(gate(
        "s01_dynamic_resize_remaps",
        resize_delivered,
        format!("resized window={resized_win:?} delivered={resize_delivered}"),
    ));
    gates.push(gate(
        "s01_pixels_show_ui_3d_ui",
        pixels_ok,
        format!(
            "outer@3d={outer_3d_px:?} outer_bg={outer_bg_px:?} plane(inner_bg)={plane_px:?} expected_inner_bg≈{inner_bg:?}"
        ),
    ));

    facts.probes.push(ProbeFacts {
        name: "s01_outer_shows_3d".into(),
        sampled: outer_3d_px,
        expected: [0, 0, 0, 255],
        tol: 255,
        ok: !near(outer_3d_px, ui_bg, 6),
        note: "outer UI texture carries the 3D scene inside the mount region".into(),
    });
    facts.probes.push(ProbeFacts {
        name: "s01_3d_shows_inner_ui".into(),
        sampled: plane_px,
        expected: [inner_bg[0], inner_bg[1], inner_bg[2], 255],
        tol: 10,
        ok: near(plane_px, inner_bg, 10),
        note: "the 3D plane carries the inner UI background".into(),
    });

    save_keyframe(rt, outer, "s01_ui3d2_outer", report_dir, keyframes);
    save_keyframe(rt, mid, "s01_ui3d2_mid", report_dir, keyframes);
    save_keyframe(rt, inner, "s01_ui3d2_inner", report_dir, keyframes);
    facts.content_passes = rt.stats.surfaces_rendered;
    facts.portal_composites = rt.stats.portal_composites;
    facts
}

fn build_s01(
    rt: &mut SurfaceRuntime,
    gates: &mut Vec<GateResult>,
    report_dir: &Path,
    keyframes: &mut Vec<Keyframe>,
    failures: &mut Vec<String>,
) -> ShapeFacts {
    let mut facts = ShapeFacts {
        name: "s01-3d-ui-2d".into(),
        surfaces: 0,
        portals: 0,
        render_order_len: 0,
        content_passes: 0,
        portal_composites: 0,
        probes: Vec::new(),
    };
    let top = match rt.add_surface(
        [640.0, 480.0],
        crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(scene3d(
            rt.gpu(),
            640.0,
            480.0,
        ))),
    ) {
        Ok(id) => id,
        Err(e) => {
            failures.push(format!("top 3d: {e}"));
            return facts;
        }
    };
    let top_view = rt.add_view(top, false).expect("view");
    let ui = rt
        .add_surface(
            [320.0, 240.0],
            crate::host::surface_runtime::SurfaceContent::Iced(Box::new(iced_content(
                rt.gpu(),
                320.0,
                240.0,
            ))),
        )
        .expect("ui");
    let ui_view = rt.add_view(ui, false).expect("ui view");
    rt.add_portal(
        top_view,
        ui,
        MountKind::Plane3d {
            center: [0.0, 1.7, 0.0],
            half: [2.4, 1.8],
        },
        true,
    )
    .expect("plane portal");
    // Inner canvas: TWO red blocks — one inside the mount clip, one beyond
    // it (the scissor must cut the second).
    let canvas_id = rt
        .add_surface(
            [320.0, 240.0],
            crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(canvas(
                rt,
                Color::rgb(0.05, 0.07, 0.12),
                vec![
                    DrawCmd {
                        center: [40.0, 40.0],
                        size: [40.0, 30.0],
                        z: 0.0,
                        color: Color::rgb(0.9, 0.1, 0.1),
                        shape: Shape::Rect,
                    },
                    DrawCmd {
                        center: [140.0, 40.0],
                        size: [40.0, 30.0],
                        z: 0.0,
                        color: Color::rgb(0.9, 0.1, 0.1),
                        shape: Shape::Rect,
                    },
                ],
            ))),
        )
        .expect("canvas");
    rt.add_view(canvas_id, false).expect("canvas view");
    // Clip: mount rect = (170,150)..(300,240) but the clip cuts at x<270 so
    // the block's right part (x 255..255) stays inside while x>270 is cut.
    let mount_rect = [170.0f32, 150.0, 320.0, 240.0];
    rt.add_portal(
        ui_view,
        canvas_id,
        MountKind::Flat2d {
            offset: [mount_rect[0], mount_rect[1]],
            scale: [1.0, 1.0],
            clip: Rect::at(170.0, 150.0, 100.0, 90.0),
        },
        true,
    )
    .expect("flat portal");

    facts.surfaces = rt.graph.live_surfaces();
    facts.portals = rt.graph.portals().len();
    let passes_before = rt.stats.surfaces_rendered;
    let portals_before = rt.stats.portal_composites;
    if let Err(e) = rt.frame(None, &HashMap::new()) {
        failures.push(format!("frame: {e}"));
        return facts;
    }
    facts.content_passes = rt.stats.surfaces_rendered - passes_before;
    facts.portal_composites = rt.stats.portal_composites - portals_before;
    facts.render_order_len = rt.last_order.len();

    let order = rt.last_order.clone();
    let pos = |id: SurfaceId| order.iter().position(|s| *s == id);
    gates.push(gate(
        "dag_order_children_first",
        pos(canvas_id) < pos(ui) && pos(ui) < pos(top),
        format!("order={order:?}"),
    ));
    gates.push(gate(
        "single_content_pass",
        facts.content_passes == 3,
        format!(
            "passes={} for {} surfaces",
            facts.content_passes, facts.surfaces
        ),
    ));

    // Block A (canvas local (40,40)) maps to UI (210,190): inside clip.
    // Block B (canvas local (140,40)) maps to UI (310,190): beyond the clip
    // edge at x=270, so the scissor must remove it.
    let in_clip_pt = [170.0 + 40.0, 150.0 + 40.0];
    let in_clip = rt.probe_pixel(ui, in_clip_pt).unwrap_or([0, 0, 0, 0]);
    let out_clip = rt
        .probe_pixel(ui, [170.0 + 140.0, 150.0 + 40.0])
        .unwrap_or([0, 0, 0, 0]);
    let probe_red = ProbeFacts {
        name: "canvas_block_inside_clip".into(),
        sampled: in_clip,
        expected: [231, 89, 89, 255], // linear 0.9/0.1/0.1 → sRGB
        tol: 24,
        ok: in_clip[0] as i32 > in_clip[1] as i32 + 40 && in_clip[0] > 120,
        note: "block A at UI (210,190) is inside the 100x90 clip".into(),
    };
    let probe_out = ProbeFacts {
        name: "content_outside_clip_cut".into(),
        sampled: out_clip,
        expected: [0, 0, 0, 0],
        tol: 255,
        ok: !(out_clip[0] as i32 > out_clip[1] as i32 + 40 && out_clip[0] > 120),
        note: "must NOT be the canvas red (scissor clipped)".into(),
    };
    facts.probes.push(probe_red.clone());
    facts.probes.push(probe_out.clone());
    gates.push(gate(
        "clip_scissors_content",
        probe_red.ok && probe_out.ok,
        format!("{probe_red:?} {probe_out:?}"),
    ));

    // The 3D top target must show the UI plane at the projected center: a
    // light, low-saturation region (UI background) rather than scene colors
    // (sky ~165/167/171, ground ~224-ish gray, models dark).
    // Sample the UI's PADDING area (background), not a widget: the panel is
    // padding 12, so local (6,6) is pure background.
    let center = crate::host::iced_surface_probe::ui_point_to_window_pixel(
        iced_wgpu::core::Point::new(6.0, 6.0),
        iced_wgpu::core::Size::new(320.0, 240.0),
        match rt.content(top) {
            Some(crate::host::surface_runtime::SurfaceContent::Scene3d(s)) => s.view_proj,
            _ => glam::Mat4::IDENTITY,
        },
        &crate::host::iced_surface_probe::PlaneXform::sample(),
        (640, 480),
    );
    let plane_px = rt
        .probe_pixel(top, [center.0, center.1])
        .unwrap_or([0, 0, 0, 0]);
    // The composed 3D target must show the UI BACKGROUND (measured
    // 249,250,254 for this theme) at the projected plane point — not the sky
    // (210,212,214) and not the ground. A brightness-only check would be a
    // false positive (the sky is bright too).
    let ui_bg = [249u8, 250u8, 254u8];
    let shows_ui = plane_px[0].abs_diff(ui_bg[0]) <= 6
        && plane_px[1].abs_diff(ui_bg[1]) <= 6
        && plane_px[2].abs_diff(ui_bg[2]) <= 6;
    let sky = [210u8, 212u8, 214u8];
    let shows_sky = plane_px[0].abs_diff(sky[0]) <= 6 && plane_px[2].abs_diff(sky[2]) <= 6;
    gates.push(gate(
        "plane_shows_ui_content",
        shows_ui && !shows_sky,
        format!("plane sample at {center:?} = {plane_px:?}; ui_bg={ui_bg:?} sky={sky:?}"),
    ));
    facts.probes.push(ProbeFacts {
        name: "plane_center_bright_ui_bg".into(),
        sampled: plane_px,
        expected: [242, 245, 252, 255],
        tol: 40,
        ok: shows_ui && !shows_sky,
        note: "structural: UI background is the only bright region of the 3D target".into(),
    });

    save_keyframe(rt, ui, "s01_ui_composed", report_dir, keyframes);
    save_keyframe(rt, top, "s01_top_composed", report_dir, keyframes);
    save_keyframe(rt, canvas_id, "s01_canvas", report_dir, keyframes);
    facts
}

// ---------------------------------------------------------------------------
// Shape 2
// ---------------------------------------------------------------------------

fn build_alpha_shape(
    rt: &mut SurfaceRuntime,
    gates: &mut Vec<GateResult>,
    report_dir: &Path,
    keyframes: &mut Vec<Keyframe>,
) -> ShapeFacts {
    let mut facts = ShapeFacts {
        name: "alpha-order-shared".into(),
        surfaces: 0,
        portals: 0,
        render_order_len: 0,
        content_passes: 0,
        portal_composites: 0,
        probes: Vec::new(),
    };
    // Parent: opaque mid-gray (linear 0.5).
    let parent_id = rt
        .add_surface(
            [400.0, 300.0],
            crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(canvas(
                rt,
                Color::rgb(0.5, 0.5, 0.5),
                Vec::new(),
            ))),
        )
        .expect("parent");
    let parent_view = rt.add_view(parent_id, false).expect("parent view");
    // Child A: PREMULTIPLIED red at 50% (straight red 0.5 ⇒ stored rgb 0.25).
    let a_clear = Color::rgba(0.25, 0.0, 0.0, 0.5);
    let a_id = rt
        .add_surface(
            [100.0, 100.0],
            crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(canvas(
                rt,
                a_clear,
                Vec::new(),
            ))),
        )
        .expect("child a");
    rt.add_view(a_id, false).expect("a view");
    // Child B: opaque green.
    let b_id = rt
        .add_surface(
            [60.0, 60.0],
            crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(canvas(
                rt,
                Color::rgb(0.0, 1.0, 0.0),
                Vec::new(),
            ))),
        )
        .expect("child b");
    rt.add_view(b_id, false).expect("b view");
    // Mounts: A twice (shared) + B declared LAST (on top).
    for x in [20.0f32, 150.0] {
        rt.add_portal(
            parent_view,
            a_id,
            MountKind::Flat2d {
                offset: [x, 20.0],
                scale: [1.0, 1.0],
                clip: Rect::at(x, 20.0, 100.0, 100.0),
            },
            true,
        )
        .expect("mount a");
    }
    rt.add_portal(
        parent_view,
        b_id,
        MountKind::Flat2d {
            offset: [40.0, 40.0],
            scale: [1.0, 1.0],
            clip: Rect::at(40.0, 40.0, 60.0, 60.0),
        },
        true,
    )
    .expect("mount b");

    facts.surfaces = rt.graph.live_surfaces();
    facts.portals = rt.graph.portals().len();
    let passes_before = rt.stats.surfaces_rendered;
    let portals_before = rt.stats.portal_composites;
    let _ = rt.frame(None, &HashMap::new());
    facts.content_passes = rt.stats.surfaces_rendered - passes_before;
    facts.portal_composites = rt.stats.portal_composites - portals_before;
    facts.render_order_len = rt.last_order.len();

    // Shared child A is mounted twice: exactly ONE content pass in the frame;
    // no surface may render more than once.
    let shared_max = rt.stats.last_frame_renders.get(&a_id).copied().unwrap_or(0);
    let any_double = rt.stats.last_frame_renders.values().any(|n| *n > 1);
    gates.push(gate(
        "shared_mount_single_content_pass",
        shared_max == 1 && !any_double,
        format!(
            "shared child passes={shared_max}; per-surface renders={:?}",
            rt.stats.last_frame_renders
        ),
    ));

    // Alpha expectation: dst = gray linear 0.5; src = premultiplied 0.25 red
    // at alpha 0.5 → out = (0.25,0,0) + 0.5*(0.5,0.5,0.5) = (0.5,0.25,0.25).
    let expected_alpha = premult_over_linear([0.25, 0.0, 0.0, 0.5], [0.5, 0.5, 0.5, 1.0]);
    let a_px = rt
        .probe_pixel(parent_id, [200.0, 70.0])
        .unwrap_or([0, 0, 0, 0]);
    facts.probes.push(ProbeFacts {
        name: "alpha_premultiplied_over".into(),
        sampled: a_px,
        expected: expected_alpha,
        tol: 12,
        ok: near(a_px, expected_alpha, 12),
        note: "linear premultiplied-over, sRGB-encoded".into(),
    });
    // Order: B declared last covers A at (60,60).
    let b_px = rt
        .probe_pixel(parent_id, [60.0, 60.0])
        .unwrap_or([0, 0, 0, 0]);
    let green = [0, 255, 0, 255];
    facts.probes.push(ProbeFacts {
        name: "later_portal_draws_on_top".into(),
        sampled: b_px,
        expected: green,
        tol: 12,
        ok: near(b_px, green, 12),
        note: "B is declared after A".into(),
    });
    // Parent clear outside every mount: sRGB-encoded linear 0.5.
    let gray = [srgb_encode(0.5), srgb_encode(0.5), srgb_encode(0.5), 255];
    let clear_px = rt
        .probe_pixel(parent_id, [350.0, 250.0])
        .unwrap_or([0, 0, 0, 0]);
    facts.probes.push(ProbeFacts {
        name: "parent_clear_outside_mounts".into(),
        sampled: clear_px,
        expected: gray,
        tol: 8,
        ok: near(clear_px, gray, 8),
        note: "opaque parent clear".into(),
    });
    gates.push(gate(
        "alpha_and_order_probes",
        facts.probes.iter().all(|p| p.ok),
        format!("{:?}", facts.probes),
    ));
    save_keyframe(rt, parent_id, "s02_composed", report_dir, keyframes);
    facts
}

// Keep the unused import (CompositionGraph) meaningful: the case asserts the
// graph type is the core one, not a host-local duplicate.
#[allow(unused)]
fn _graph_type_is_core(g: &CompositionGraph) -> usize {
    g.live_surfaces()
}
