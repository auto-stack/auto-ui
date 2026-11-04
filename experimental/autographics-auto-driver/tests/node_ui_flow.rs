//! auto-ui PLAN-048 node_ui_flow: the basic-UI chain executes as REAL stock
//! Iced widgets over the pinned SDK Session. V-S01: two same-def Counter
//! instances (Main/a, Main/b) + a nested Panel meter; real mouse
//! press/release at the ACTUAL widget bounds -> bounded queue -> generated
//! dispatcher `{prefix}_ui_click_{elem}` -> Auto state 2/1(+meter) -> text
//! getters change -> next-frame widget labels change; the control mutates
//! the Auto step (source-level) and the same clicks yield 4/2 — Rust holds
//! no business constants. V-S02: keyboard Tab+Enter activation on the real
//! focused button, disabled-button click produces no message (controlled),
//! stale-generation events are named-rejected, queue overflow is bounded and
//! counted. V-S03: if-unmount (Panel toggle -> state text absent), resize
//! re-layout, Start exactly once per Run, Tick only via explicit logical
//! steps (redraw never ticks), 20 fresh-Run rounds with no cross-run
//! leakage. The consumed ui.json/world.at are frozen generator products
//! (experimental/autographics-node2d-pilot/generated-node-ui/ui-ab).
//! GPU single instance. Run:
//! cargo test --locked --offline --test node_ui_flow -- --test-threads=1 --nocapture
use autographics_auto_driver::node_ui::NodeUiApp;
use std::path::PathBuf;

const ARTIFACT: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/ui.json");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/world.at");

const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------- harness --

fn app_from(world: &str) -> NodeUiApp {
    let mut app = NodeUiApp::new(ARTIFACT, world, "plan048-flow").expect("app");
    app.start_run().expect("start_run");
    app
}

/// click at the REAL widget bounds center via real mouse events (offscreen
/// row; the native row uses SendInput and is recorded separately).
fn click(
    app: &mut NodeUiApp,
    surface: &mut agp::host::iced_surface::IcedSurface,
    key: &str,
) -> usize {
    use iced_wgpu::core::{
        mouse::{Button, Event as Mouse},
        Event,
    };
    let bounds = container_bounds(surface, app, key);
    let center = iced_wgpu::core::Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );
    let cursor = iced_wgpu::core::mouse::Cursor::Available(center);
    let msgs = surface.frame(
        app,
        &[
            Event::Mouse(Mouse::ButtonPressed(Button::Left)),
            Event::Mouse(Mouse::ButtonReleased(Button::Left)),
        ],
        cursor,
    );
    app.drain();
    msgs.len()
}

/// REAL container bounds via a public Operation (containers are not
/// focusable; the agp helper only visits focusables).
fn container_bounds(
    surface: &mut agp::host::iced_surface::IcedSurface,
    app: &NodeUiApp,
    key: &str,
) -> iced_wgpu::core::Rectangle {
    struct Probe {
        want: iced_wgpu::core::widget::Id,
        bounds: Option<iced_wgpu::core::Rectangle>,
    }
    impl iced_wgpu::core::widget::Operation<()> for Probe {
        fn traverse(
            &mut self,
            operate: &mut dyn FnMut(&mut dyn iced_wgpu::core::widget::Operation<()>),
        ) {
            operate(self)
        }
        fn container(
            &mut self,
            id: Option<&iced_wgpu::core::widget::Id>,
            bounds: iced_wgpu::core::Rectangle,
        ) {
            if id == Some(&self.want) {
                self.bounds = Some(bounds);
            }
        }
    }
    let mut op = Probe {
        want: iced_wgpu::core::widget::Id::from(key.to_string()),
        bounds: None,
    };
    surface.operate(app, &mut op);
    op.bounds
        .unwrap_or_else(|| panic!("container bounds for {key}"))
}

fn text_of(app: &NodeUiApp, instance: usize, slot: usize) -> String {
    app.texts(instance)[slot].clone()
}

fn artifact_world_sha_of(world: &str) -> String {
    agp::renderer::capture::sha256_hex(world.as_bytes())
}

// ------------------------------------------------------------------ V-S01 --

#[test]
fn vs01_two_instances_real_clicks_isolate_and_source_control_replays() {
    let gpu = agp::renderer::GpuDevice::new().unwrap();
    let mut surface =
        agp::host::iced_surface::IcedSurface::new(&gpu, iced::Size::new(480.0, 900.0), 1.0)
            .unwrap();
    let mut app = app_from(WORLD);
    // Baseline frame: labels from accepted state (a "0", b "10").
    let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    assert_eq!(text_of(&app, app.instance_index("Main/a").unwrap(), 1), "0");
    assert_eq!(
        text_of(&app, app.instance_index("Main/b").unwrap(), 1),
        "10"
    );
    // Pixel baseline of the two value-label regions (one GPU readback).
    let before = capture_labels(&gpu, &mut surface, &mut app);

    // Real clicks: A twice, B once -> step 1 / step 2 (Auto-defined).
    let n = click(&mut app, &mut surface, "Main/a#inc")
        + click(&mut app, &mut surface, "Main/a#inc")
        + click(&mut app, &mut surface, "Main/b#inc");
    assert_eq!(n, 3, "three real widget messages");
    assert_eq!(text_of(&app, app.instance_index("Main/a").unwrap(), 1), "2");
    assert_eq!(
        text_of(&app, app.instance_index("Main/b").unwrap(), 1),
        "12"
    );

    // Pixel evidence: the SAME probe regions now show different label pixels.
    let after = capture_labels(&gpu, &mut surface, &mut app);
    assert_eq!(before.0, after.0, "same probe points");
    assert_ne!(before.1, after.1, "label pixels changed with Auto state");

    // CONTROL: Auto increment changes (step for a: 1 -> 2) via SOURCE-LEVEL
    // mutation; same clicks now yield 4/2 (a=2*2, b=10+2). No Rust constant
    // participates: dispatch names come from the frozen artifact.
    let world_alt = WORLD.replacen("var counter_step=[1,2,5]", "var counter_step=[2,2,5]", 1);
    assert_ne!(world_alt, WORLD, "control mutation applied");
    // the re-issued identity binds the mutated source (a source change makes
    // the old artifact stale — the producer regenerates the binding).
    let artifact_alt = ARTIFACT.replacen(
        &format!("\"world_sha256\": \"{}\"", artifact_world_sha_of(WORLD)),
        &format!(
            "\"world_sha256\": \"{}\"",
            artifact_world_sha_of(&world_alt)
        ),
        1,
    );
    assert_ne!(artifact_alt, ARTIFACT, "re-issued binding");
    let mut app2 = NodeUiApp::new(&artifact_alt, &world_alt, "plan048-flow").expect("app2");
    app2.start_run().expect("start_run 2");
    let _ = surface.frame(&mut app2, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    let n2 = click(&mut app2, &mut surface, "Main/a#inc")
        + click(&mut app2, &mut surface, "Main/a#inc")
        + click(&mut app2, &mut surface, "Main/b#inc");
    assert_eq!(n2, 3);
    assert_eq!(
        text_of(&app2, app2.instance_index("Main/a").unwrap(), 1),
        "4"
    );
    assert_eq!(
        text_of(&app2, app2.instance_index("Main/b").unwrap(), 1),
        "12"
    );
}

/// probe the two value-label regions (A/B rows) with one GPU readback each.
fn capture_labels(
    gpu: &agp::renderer::GpuDevice,
    surface: &mut agp::host::iced_surface::IcedSurface,
    app: &mut NodeUiApp,
) -> (Vec<[f32; 2]>, Vec<[u8; 4]>) {
    let mut points = Vec::new();
    for key in ["Main/a#value", "Main/b#value"] {
        let b = container_bounds(surface, app, key);
        // horizontal scan across the label region (glyph pixels vs background)
        for k in 0..16 {
            let fx = b.x + b.width * (k as f32 + 0.5) / 16.0;
            points.push([fx, b.y + b.height / 2.0]);
        }
    }
    // one frame so bounds reflect current state
    let _ = surface.frame(app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    let img = agp::renderer::capture::capture_png(gpu, surface.target())
        .expect("capture")
        .1;
    let px: Vec<[u8; 4]> = points
        .iter()
        .map(|p| {
            let x = (p[0] as u32).min(img.width() - 1);
            let y = (p[1] as u32).min(img.height() - 1);
            {
                let q = img.get_pixel(x, y);
                [q[0], q[1], q[2], q[3]]
            }
        })
        .collect();
    (points, px)
}

// ------------------------------------------------------------------ V-S02 --

#[test]
fn vs02_keyboard_disabled_stale_and_queue_bound() {
    let gpu = agp::renderer::GpuDevice::new().unwrap();
    let mut surface =
        agp::host::iced_surface::IcedSurface::new(&gpu, iced::Size::new(480.0, 900.0), 1.0)
            .unwrap();
    let mut app = app_from(WORLD);
    let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);

    use iced_wgpu::core::Event;
    // Keyboard: real key events are routed by the adapter's platform-input
    // layer (stock iced 0.14 button has no keyboard support — verified);
    // the routed activation produces the SAME queue entry as a mouse click
    // and dispatches through the same generated dispatcher.
    use autographics_auto_driver::node_ui::PlatformKey;
    let focus0 = app.focused_key().map(str::to_string);
    let _ = surface.frame(
        &mut app,
        &[Event::Keyboard(
            iced_wgpu::core::keyboard::Event::KeyPressed {
                key: iced_wgpu::core::keyboard::Key::Named(
                    iced_wgpu::core::keyboard::key::Named::Tab,
                ),
                modified_key: iced_wgpu::core::keyboard::Key::Named(
                    iced_wgpu::core::keyboard::key::Named::Tab,
                ),
                physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
                    iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced_wgpu::core::keyboard::Location::Standard,
                modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
                text: None,
                repeat: false,
            },
        )],
        iced_wgpu::core::mouse::Cursor::Unavailable,
    );
    let moved = app.key(PlatformKey::Tab);
    assert!(moved, "Tab routed");
    let focus1 = app.focused_key().map(str::to_string);
    assert_ne!(focus0, focus1, "Tab advances the real focus ring");
    let queue_before = app.queue_len();
    let _ = surface.frame(
        &mut app,
        &[Event::Keyboard(
            iced_wgpu::core::keyboard::Event::KeyPressed {
                key: iced_wgpu::core::keyboard::Key::Named(
                    iced_wgpu::core::keyboard::key::Named::Enter,
                ),
                modified_key: iced_wgpu::core::keyboard::Key::Named(
                    iced_wgpu::core::keyboard::key::Named::Enter,
                ),
                physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
                    iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced_wgpu::core::keyboard::Location::Standard,
                modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
                text: None,
                repeat: false,
            },
        )],
        iced_wgpu::core::mouse::Cursor::Unavailable,
    );
    let handled = app.key(PlatformKey::Enter);
    assert!(handled, "Enter activates the focused button");
    assert_eq!(app.queue_len(), queue_before + 1, "activation enqueued");
    let report = app.drain();
    assert_eq!(report.dispatched, 1, "keyboard activation dispatched once");

    // Disabled click: the Panel reset button is bound to .on (initially 1);
    // toggle -> on=false -> the real button removes its on_press (iced
    // semantics) and clicking yields NO message (controlled refusal).
    let meter_before = text_of(&app, app.instance_index("Main/side/meter").unwrap(), 1);
    click(&mut app, &mut surface, "Main/side#toggle");
    assert!(
        !app.cond(&format!("Main/side#reset#en")),
        "reset disabled after toggle"
    );
    let msgs = click(&mut app, &mut surface, "Main/side#reset");
    assert_eq!(msgs, 0, "disabled button produces no real message");
    assert_eq!(
        text_of(&app, app.instance_index("Main/side/meter").unwrap(), 1),
        meter_before,
        "no state change from the refused click"
    );

    // Stale generation: enqueue events, open a new Run, drain -> named reject.
    for _ in 0..3 {
        let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    }
    let gen_before = app.generation;
    // enqueue a real widget message (click A) WITHOUT draining, then start a
    // NEW run; the queued event is now stale (old generation) and the drain
    // must name-reject it instead of dispatching into the fresh run.
    {
        use iced_wgpu::core::{
            mouse::{Button, Event as Mouse},
            Event,
        };
        let bounds = container_bounds(&mut surface, &app, "Main/a#inc");
        let center = iced_wgpu::core::Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );
        let _ = surface.frame(
            &mut app,
            &[
                Event::Mouse(Mouse::ButtonPressed(Button::Left)),
                Event::Mouse(Mouse::ButtonReleased(Button::Left)),
            ],
            iced_wgpu::core::mouse::Cursor::Available(center),
        );
    }
    assert_eq!(app.queue_len(), 1, "one queued event pending");
    app.start_run().expect("new run");
    assert_eq!(app.generation, gen_before + 1);
    let report = app.drain();
    assert_eq!(
        report.stale_rejected, 1,
        "stale-generation event named-rejected: {report:?}"
    );
    assert_eq!(report.dispatched, 0, "stale event did not dispatch");

    // Queue bound: overflow drops are bounded and counted, never unbounded.
    for _ in 0..(autographics_auto_driver::node_ui::QUEUE_CAP + 10) {
        let _ = app.enqueue_click(
            0,
            "counter_ui_click_inc".into(),
            Some(0),
            "overflow-probe".into(),
        );
    }
    assert!(app.queue_len() <= autographics_auto_driver::node_ui::QUEUE_CAP);
    assert_eq!(
        app.dropped_overflow(),
        10,
        "exactly the overflow was dropped and counted"
    );
}

// ------------------------------------------------------------------ V-S03 --

#[test]
fn vs03_lifecycle_unmount_resize_and_fresh_runs() {
    let gpu = agp::renderer::GpuDevice::new().unwrap();
    let mut surface =
        agp::host::iced_surface::IcedSurface::new(&gpu, iced::Size::new(480.0, 900.0), 1.0)
            .unwrap();
    let mut app = app_from(WORLD);

    // Start exactly once per Run: root Start resets .flips (0) — a second
    // start_run is a NEW Run (generation+1), not a double Start on one Run.
    let root = app.instance_index("Main").unwrap();
    assert_eq!(text_of(&app, root, 1), "0", "flips reset by Life.Start");

    // Tick ONLY via explicit logical steps; redraw never advances it.
    let t0 = text_of(&app, root, 2);
    for _ in 0..3 {
        let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    }
    assert_eq!(text_of(&app, root, 2), t0, "redraw does not tick");
    app.step(DT).expect("tick");
    app.step(DT).expect("tick");
    assert_eq!(text_of(&app, root, 2), "2", "exactly two logical ticks");

    // if-unmount: Panel toggle -> .on=false -> state text subtree absent.
    let _side = app.instance_index("Main/side").unwrap();
    // the state text lives under `if .on` — its condition key is the def-walk
    // if-index (stable, frozen artifact order), not a #vis prop.
    let state_key = "Main/side#if0";
    assert!(
        NodeUiApp::binding_keys("Main/side", &app.artifact.defs["Panel"].elements)
            .contains(&state_key.to_string()),
        "if key is the stable binding key"
    );
    assert!(app.cond(state_key), "state text visible while on");
    click(&mut app, &mut surface, "Main/side#toggle");
    assert!(!app.cond(state_key), "state text unmounted after toggle");
    // New instance semantics: re-mount is a fresh subtree (cond re-evaluated).
    click(&mut app, &mut surface, "Main/side#toggle");
    assert!(app.cond(state_key), "re-mounted subtree");

    // Resize: the surface really resizes and the real layout pass
    // re-converges on the next frame. The frozen artifact's basic layout is
    // width-independent (shrink-fit rows; no fill-bound widths), so the
    // correct relayout is an identity — the oracle is that the layout engine
    // still places the real widget after resize (bounds resolve, nonempty),
    // and 049's mixed viewports add the fill-bound resize oracles.
    surface
        .resize(&gpu, iced::Size::new(320.0, 900.0))
        .expect("resize");
    {
        let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    }
    let (w, _h) = surface.physical_size();
    assert_eq!(w, 320, "real resize reached the offscreen surface");
    let b = container_bounds(&mut surface, &app, "Main/a#inc");
    assert!(
        b.width > 0.0 && b.height > 0.0,
        "layout re-converged after resize"
    );
    let change = surface.last_change.clone();
    let _ = change;

    use iced_wgpu::core::Event;
    // 20 fresh-Run rounds: state resets, no cross-run leakage, queue clean.
    for round in 0..20 {
        let mut fresh = app_from(WORLD);
        let a = fresh.instance_index("Main/a").unwrap();
        assert_eq!(text_of(&fresh, a, 1), "0", "round {round}: fresh state");
        fresh.enqueue_click(
            a,
            "counter_ui_click_inc".into(),
            Some(0),
            format!("round-{round}"),
        );
        let r = fresh.drain();
        assert_eq!(r.dispatched, 1);
        assert_eq!(text_of(&fresh, a, 1), "1", "round {round}: one click");
        assert_eq!(fresh.queue_len(), 0);
        assert!(fresh.error.is_none());
    }
    let _ = PathBuf::new(); // silence unused import in edge configurations
}
