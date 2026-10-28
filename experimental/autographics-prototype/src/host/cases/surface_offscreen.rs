//! Case `surface-offscreen` (PLAN-830 V-02): the T-02 whole-path probe as a
//! graphics-probe case — real iced UI → offscreen sampled texture → 3D plane
//! → same window, with the input return path exercised through REAL window
//! events (OS-injected click/keys, bounded waits, honest click-path
//! evidence). Headless fallback renders the same path offscreen with
//! synthetic event delivery (recorded as such — never claimed as OS input).

use crate::host::iced_surface_probe::{
    canonical_step, run_action, InnerMessage, PendingAction, SurfaceProbe, WINDOW_TITLE,
};
use crate::host::report::*;
use crate::renderer::target::WindowTarget;
use iced_wgpu::core::{Event, Point, Size};
use std::path::Path;
use std::sync::Arc;

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    match mode {
        "headless" => run_headless(replay, report_dir),
        "window" => run_windowed(replay, report_dir),
        "auto" => {
            let code = run_windowed(replay, report_dir);
            if code == 3 {
                eprintln!("surface-offscreen: window unavailable; headless fallback");
                run_headless(replay, report_dir)
            } else {
                code
            }
        }
        other => {
            eprintln!("unknown mode: {other}");
            2
        }
    }
}

// ---------------------------------------------------------------------------
// Windowed host
// ---------------------------------------------------------------------------

struct ProbeApp {
    probe: SurfaceProbe,
    pending: Option<PendingAction>,
    live_exit: Option<i32>,
}

impl winit::application::ApplicationHandler for ProbeApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.probe.inner.is_some() {
            return;
        }
        let attrs = winit::window::Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(900.0, 640.0));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("surface-offscreen: create_window: {e}");
                self.live_exit = Some(3);
                event_loop.exit();
                return;
            }
        };
        self.probe.window = Some(window.clone());
        if let Err(e) = self.probe.warmup(true) {
            eprintln!("surface-offscreen: warmup: {e}");
            self.live_exit = Some(3);
            event_loop.exit();
            return;
        }
        let gpu = self.probe.gpu.as_ref().unwrap();
        let size = window.inner_size();
        match WindowTarget::new(gpu, window, size.width, size.height) {
            Ok(t) => self.probe.window_target = Some(t),
            Err(e) => {
                eprintln!("surface-offscreen: surface: {e}");
                self.live_exit = Some(3);
                event_loop.exit();
                return;
            }
        }
        // The window target's format may differ from the offscreen format:
        // rebuild the present blit against it.
        let fmt = self.probe.window_target.as_ref().unwrap().format();
        self.probe.blit = Some(crate::renderer::composition::FullScreenBlit::new(
            gpu.device(),
            fmt,
        ));
        self.probe.evidence.window_created = true;
        self.probe.evidence.window_mode = "window".into();
        self.probe.phase_started = std::time::Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::ElementState;
        use winit::event::WindowEvent as WE;
        let probe = &mut self.probe;
        match event {
            WE::CloseRequested => {
                self.live_exit = Some(probe.exit_code);
                event_loop.exit();
            }
            WE::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let (Some(gpu), Some(wt)) =
                        (probe.gpu.as_ref(), probe.window_target.as_mut())
                    {
                        let _ = wt.handle_resize(gpu, size.width, size.height);
                        let aspect = size.width as f32 / size.height.max(1) as f32;
                        probe.view_proj = crate::host::iced_surface_probe::probe_camera(aspect);
                        if let Some(c) = probe.composite.as_mut() {
                            let _ = c.resize(gpu, size.width, size.height);
                        }
                    }
                }
            }
            WE::Focused(false) => {
                probe.cursor_local = None;
                probe
                    .pending_events
                    .push(Event::Mouse(iced_wgpu::core::mouse::Event::CursorLeft));
            }
            WE::CursorMoved { position, .. } => {
                let scale = probe
                    .window
                    .as_ref()
                    .map(|w| w.scale_factor() as f32)
                    .unwrap_or(1.0);
                let px = (position.x as f32, position.y as f32);
                if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                    eprintln!(
                        "DBG CursorMoved px={px:?} cursor_local={:?}",
                        probe.cursor_local
                    );
                }
                let uv = probe.map_window_pixel(px);
                match uv {
                    Some(uv) => {
                        let logical = Size::new(
                            crate::host::iced_surface_probe::INNER_LOGICAL.0,
                            crate::host::iced_surface_probe::INNER_LOGICAL.1,
                        );
                        let local = probe.plane_xform.uv_to_local(uv, logical);
                        probe.cursor_local = Some(local);
                        probe.pending_events.push(Event::Mouse(
                            iced_wgpu::core::mouse::Event::CursorMoved { position: local },
                        ));
                    }
                    None => {
                        probe.cursor_local = None;
                        probe
                            .pending_events
                            .push(Event::Mouse(iced_wgpu::core::mouse::Event::CursorLeft));
                    }
                }
                let _ = scale;
            }
            WE::MouseInput { state, button, .. } => {
                if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                    eprintln!(
                        "DBG MouseInput {state:?} {button:?} cursor_local={:?}",
                        probe.cursor_local
                    );
                }
                if button == winit::event::MouseButton::Left {
                    let b = match state {
                        ElementState::Pressed => iced_wgpu::core::mouse::Button::Left,
                        ElementState::Released => iced_wgpu::core::mouse::Button::Left,
                    };
                    let ev = match state {
                        ElementState::Pressed => iced_wgpu::core::mouse::Event::ButtonPressed(b),
                        ElementState::Released => iced_wgpu::core::mouse::Event::ButtonReleased(b),
                    };
                    // Deliver only when the cursor is mapped onto the plane
                    // (the probe has no other interactive content; the
                    // general occlusion/capture contract is T-05).
                    if probe.cursor_local.is_some() {
                        if let ElementState::Pressed = state {
                            probe.evidence.clicks_mapped_to_plane += 1;
                        }
                        probe.pending_events.push(Event::Mouse(ev));
                    }
                }
            }
            WE::KeyboardInput { event, .. } => {
                if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                    eprintln!("DBG Key {:?}", event.logical_key);
                }
                if event.state == ElementState::Released {
                    return;
                }
                if let Some(ev) = crate::host::iced_surface_probe::winit_key_to_iced(&event) {
                    probe.pending_events.push(Event::Keyboard(ev));
                    probe.evidence.keyboard_events_delivered += 1;
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let probe = &mut self.probe;
        probe.wall_tick += 1;
        let messages: Vec<InnerMessage> = probe.render_frame();
        // Mirror the observed inner-UI state for the injection threads.
        {
            use std::sync::atomic::Ordering;
            crate::host::iced_surface_probe::PRESS_COUNT
                .store(probe.ui.presses as u64, Ordering::SeqCst);
            crate::host::iced_surface_probe::TEXT_LEN
                .store(probe.ui.input.chars().count() as u64, Ordering::SeqCst);
        }

        if probe.replay == "live" {
            // Esc closes the live probe.
            if probe.pending_events.iter().any(|e| {
                matches!(
                    e,
                    Event::Keyboard(iced_wgpu::core::keyboard::Event::KeyPressed {
                        key: iced_wgpu::core::keyboard::Key::Named(
                            iced_wgpu::core::keyboard::key::Named::Escape
                        ),
                        ..
                    })
                )
            }) {
                self.live_exit = Some(0);
                event_loop.exit();
                return;
            }
            return;
        }

        // Canonical scripted sequence.
        let stop = canonical_step(probe, &mut self.pending);
        if let Some(action) = self.pending.take() {
            // OS input injection runs on its own thread (bounded inside the
            // action script); the event loop keeps servicing real events.
            run_action(action);
        }
        let _ = messages;
        if stop {
            let code = finalize(probe);
            self.live_exit = Some(code);
            event_loop.exit();
        }
    }
}

fn run_windowed(replay: &str, report_dir: &Path) -> i32 {
    use winit::event_loop::EventLoop;
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => return report_early(replay, report_dir, &format!("event loop: {e}")),
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = ProbeApp {
        probe: SurfaceProbe::new(replay.to_string(), report_dir.to_path_buf()),
        pending: None,
        live_exit: None,
    };
    let _ = event_loop.run_app(&mut app);
    // Finalize once the loop has fully exited (report + gates + teardown).
    if !app.probe.finished {
        app.probe.finished = true;
        let code = finalize(&mut app.probe);
        app.live_exit = Some(code);
    }
    app.live_exit.unwrap_or(0)
}

/// Write an early-failure report and return the runner exit code.
fn report_early(replay: &str, report_dir: &Path, note: &str) -> i32 {
    let body = SurfaceReport {
        adapter: crate::renderer::AdapterInfoReport {
            name: "unavailable".into(),
            backend: "n/a".into(),
            device_type: "n/a".into(),
            driver: String::new(),
            driver_info: String::new(),
            vendor: 0,
            device: 0,
        },
        device_token: "unavailable".into(),
        window_created: false,
        window_mode: "window-failed".into(),
        input_evidence: "none".into(),
        inner_ui_pressed: 0,
        inner_input_text: String::new(),
        clicks_mapped_to_plane: 0,
        keyboard_events_delivered: 0,
        plane_passes_drawn: 0,
        ui_frames_presented: 0,
        readbacks: 0,
        keyframes: Vec::new(),
        gates: Vec::new(),
        all_gates_behaved: false,
        button_bounds: None,
        input_bounds: None,
        failures: vec![note.to_string()],
        gpu_timing_available: false,
        notes: Vec::new(),
    };
    let _ = write_report(report_dir, "surface-offscreen", replay, "fail", &body);
    3
}

// ---------------------------------------------------------------------------
// Headless fallback (render path + synthetic events; input recorded as
// synthetic — never claimed as OS input evidence)
// ---------------------------------------------------------------------------

fn run_headless(replay: &str, report_dir: &Path) -> i32 {
    let mut probe = SurfaceProbe::new(replay.to_string(), report_dir.to_path_buf());
    probe.evidence.window_mode = "headless".into();
    if let Err(e) = probe.warmup(false) {
        eprintln!("surface-offscreen(headless): warmup: {e}");
        return report_early(replay, report_dir, &e);
    }
    // Warmup frames.
    for _ in 0..30 {
        probe.wall_tick += 1;
        let _ = probe.render_frame();
    }
    // Exact bounds (public traversal).
    let bounds = probe.probe_bounds();
    probe.evidence.button_bounds = bounds
        .button_bounds()
        .map(|r| [r.x, r.y, r.width, r.height]);
    probe.evidence.input_bounds = bounds.text_input.map(|r| [r.x, r.y, r.width, r.height]);
    probe.capture("initial");

    // Synthetic round-trip: a press+release PAIR delivered entirely at the
    // button's mapped local point (release must not race a cursor move).
    if let Some(b) = bounds.button_bounds() {
        let c = Point::new(b.x + b.width / 2.0, b.y + b.height / 2.0);
        // Verify the mapping round-trip: local → window px → plane → local.
        let px = probe.ui_local_click_target(c);
        if let Some(uv) = probe.map_window_pixel(px) {
            let logical = Size::new(
                crate::host::iced_surface_probe::INNER_LOGICAL.0,
                crate::host::iced_surface_probe::INNER_LOGICAL.1,
            );
            let back = probe.plane_xform.uv_to_local(uv, logical);
            let dist = ((back.x - c.x).powi(2) + (back.y - c.y).powi(2)).sqrt();
            probe
                .evidence
                .notes
                .push(format!("mapping round-trip error: {dist:.3}px"));
        }
        probe.cursor_local = Some(c);
        probe
            .pending_events
            .push(Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
                position: c,
            }));
        probe
            .pending_events
            .push(Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
                iced_wgpu::core::mouse::Button::Left,
            )));
        probe.evidence.clicks_mapped_to_plane += 1;
        let _ = probe.render_frame();
        probe
            .pending_events
            .push(Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
                iced_wgpu::core::mouse::Button::Left,
            )));
        let _ = probe.render_frame();
    }
    // Synthetic keyboard into the text_input (click focuses it first; the
    // full press+release pair happens at the input's point).
    if let Some(b) = bounds.text_input {
        let c = Point::new(b.x + b.width / 2.0, b.y + b.height / 2.0);
        probe.cursor_local = Some(c);
        probe
            .pending_events
            .push(Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
                position: c,
            }));
        probe
            .pending_events
            .push(Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
                iced_wgpu::core::mouse::Button::Left,
            )));
        let _ = probe.render_frame();
        probe
            .pending_events
            .push(Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
                iced_wgpu::core::mouse::Button::Left,
            )));
        let _ = probe.render_frame();
        for ch in crate::host::iced_surface_probe::TYPED_TEXT.chars() {
            probe.pending_events.push(Event::Keyboard(
                crate::host::iced_surface_probe::synthetic_char_event(ch),
            ));
            probe.evidence.keyboard_events_delivered += 1;
            let _ = probe.render_frame();
        }
    }
    probe.capture("after_input");
    probe.finished = true;
    finalize(&mut probe)
}

// ---------------------------------------------------------------------------
// Gates + report
// ---------------------------------------------------------------------------

/// Pixel analysis of one captured composite: the plane region (projected
/// plane center band) must carry the UI content (far from the sky/ground
/// colors), proving the offscreen UI actually shows through the 3D plane.
fn plane_region_shows_ui(probe: &SurfaceProbe, name: &str) -> Result<bool, String> {
    let path = probe.report_dir.join(format!("{name}_composite.png"));
    let img = image::open(&path)
        .map_err(|e| format!("open {name}: {e}"))?
        .to_rgba8();
    let (w, h) = (img.width(), img.height());
    // Project the plane center and two off-center sample points.
    let logical = Size::new(
        crate::host::iced_surface_probe::INNER_LOGICAL.0,
        crate::host::iced_surface_probe::INNER_LOGICAL.1,
    );
    let mut hits = 0usize;
    let mut total = 0usize;
    for (ux, uy) in [(0.5f32, 0.5f32), (0.35, 0.65), (0.65, 0.35)] {
        let local = Point::new(ux * logical.width, uy * logical.height);
        let (px, py) = probe.ui_local_click_target(local);
        let x = (px as i64).clamp(0, w as i64 - 1) as u32;
        let y = (py as i64).clamp(0, h as i64 - 1) as u32;
        // 5x5 sample around the point.
        for dy in 0..5i32 {
            for dx in 0..5i32 {
                let sx = (x as i32 + dx - 2).clamp(0, w as i32 - 1) as u32;
                let sy = (y as i32 + dy - 2).clamp(0, h as i32 - 1) as u32;
                let p = img.get_pixel(sx, sy);
                let [r, g, b, _] = p.0;
                total += 1;
                // UI background ≈ (237,240,247); sky ≈ (165,167,171);
                // ground ≈ (224,224,224). Count "clearly UI-ish or UI content"
                // = NOT close to sky/ground (the plane area should differ).
                let near_sky = (r as i32 - 165).abs() < 18
                    && (g as i32 - 167).abs() < 18
                    && (b as i32 - 171).abs() < 18;
                let near_ground = (r as i32 - 224).abs() < 14
                    && (g as i32 - 224).abs() < 14
                    && (b as i32 - 224).abs() < 14;
                if !near_sky && !near_ground {
                    hits += 1;
                }
            }
        }
    }
    let frac = hits as f32 / total.max(1) as f32;
    Ok(frac > 0.5)
}

fn finalize(probe: &mut SurfaceProbe) -> i32 {
    probe.evidence.inner_ui_pressed = probe.ui.presses;
    probe.evidence.inner_input_text = probe.ui.input.clone();

    let mut gates: Vec<GateResult> = Vec::new();
    let mut gate = |name: &str, ok: bool, reason: String| {
        gates.push(GateResult {
            gate: name.to_string(),
            positive_passed: ok,
            // T-02 gates are positive controls only; the counterexample
            // (sabotage) controls are V-08 / T-08 — never claimed here.
            negative_fired: false,
            reason,
        });
    };

    // g1: the offscreen UI texture contains real rendered content.
    let ui_frames = probe
        .evidence
        .keyframes
        .iter()
        .filter(|k| k.name.ends_with("_ui"))
        .count();
    gate(
        "ui_offscreen_rendered",
        ui_frames >= 2,
        format!("{ui_frames} ui keyframes captured"),
    );

    // g2: the composite shows the UI through the 3D plane.
    let plane_ok = plane_region_shows_ui(probe, "after_input");
    gate(
        "plane_shows_ui",
        plane_ok.clone().unwrap_or(false),
        match &plane_ok {
            Ok(true) => "plane region carries UI content".into(),
            other => format!("{other:?}"),
        },
    );

    // g3/g4: input round-trips through the REAL widgets.
    gate(
        "click_roundtrip",
        probe.ui.presses >= 1,
        format!("presses={}", probe.ui.presses),
    );
    let typed_ok = probe.ui.input == crate::host::iced_surface_probe::TYPED_TEXT;
    gate(
        "keyboard_roundtrip",
        typed_ok,
        format!("input={:?}", probe.ui.input),
    );

    // g5: no steady-state readback (readbacks only at the 2 capture moments
    // × 2 targets).
    gate(
        "no_steady_state_readback",
        probe.evidence.readbacks <= 4,
        format!("readbacks={}", probe.evidence.readbacks),
    );

    // g6: draws actually happened.
    gate(
        "plane_and_ui_drawn",
        probe.evidence.plane_passes_drawn > 0 && probe.evidence.ui_frames_presented > 0,
        format!(
            "plane_passes={} ui_frames={}",
            probe.evidence.plane_passes_drawn, probe.evidence.ui_frames_presented
        ),
    );

    let all_ok = gates.iter().all(|g| g.positive_passed) && probe.failures.is_empty();
    probe.evidence.gates = gates
        .iter()
        .map(|g| (g.gate.clone(), g.positive_passed, g.reason.clone()))
        .collect();

    let input_evidence = if probe.evidence.window_created {
        "os_window_events"
    } else {
        "synthetic_headless"
    };
    let body = SurfaceReport {
        adapter: probe
            .gpu
            .as_ref()
            .map(|g| g.adapter_info().clone())
            .unwrap_or_else(|| crate::renderer::AdapterInfoReport {
                name: "unavailable".into(),
                backend: "n/a".into(),
                device_type: "n/a".into(),
                driver: String::new(),
                driver_info: String::new(),
                vendor: 0,
                device: 0,
            }),
        device_token: probe.evidence.device_token.clone(),
        window_created: probe.evidence.window_created,
        window_mode: probe.evidence.window_mode.clone(),
        input_evidence: input_evidence.to_string(),
        inner_ui_pressed: probe.ui.presses,
        inner_input_text: probe.ui.input.clone(),
        clicks_mapped_to_plane: probe.evidence.clicks_mapped_to_plane,
        keyboard_events_delivered: probe.evidence.keyboard_events_delivered,
        plane_passes_drawn: probe.evidence.plane_passes_drawn,
        ui_frames_presented: probe.evidence.ui_frames_presented,
        readbacks: probe.evidence.readbacks,
        keyframes: probe.evidence.keyframes.clone(),
        gates,
        all_gates_behaved: all_ok,
        button_bounds: probe.evidence.button_bounds,
        input_bounds: probe.evidence.input_bounds,
        failures: probe.failures.clone(),
        gpu_timing_available: false,
        notes: probe.evidence.notes.clone(),
    };
    let exit = if all_ok { "pass" } else { "fail" };
    let code = if all_ok { 0 } else { 1 };
    let _ = write_report(
        &probe.report_dir,
        "surface-offscreen",
        &probe.replay,
        exit,
        &body,
    );
    if !all_ok {
        eprintln!("surface-offscreen: gates failed (see report)");
    }
    // Let queued GPU work settle before teardown.
    if let Some(gpu) = probe.gpu.as_ref() {
        gpu.wait_idle();
    }
    probe.inner = None;
    probe.mesh3d = None;
    probe.plane = None;
    probe.composite = None;
    probe.window_target = None;
    code
}
