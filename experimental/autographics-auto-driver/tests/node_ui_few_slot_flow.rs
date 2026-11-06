//! auto-ui PLAN-065 node_ui_few_slot_flow: the D-06 few-slot family (0..4
//! text-slot defs) executes as REAL stock Iced widgets over the pinned SDK
//! Session. Products are frozen generator outputs of
//! fixtures/node-ui-few/{a,b,empty} (1/2-text defs previously failed VM
//! compile with the named `Undefined variable: {p}_ui_text_b_0`; the
//! frozen products carry the pruned single-branch shape). V-02: dual
//! same-def instances — Few2 c clicked twice / d once (2:1, int decrement
//! through the pruned digits path incl. negative), Few1 a/b tick-driven
//! counts differentiate by start param (instance isolation on a 1-slot
//! def), root flip button (root is itself a 2-text def), bool switching on
//! Few3 (3-slot frozen dual-branch shape), Few4 executes the 63-byte
//! multi-byte literal plus a 4-slot root (V-03 content/length boundary at
//! consumption). V-S03 pattern: Start once, Tick only via explicit logical
//! steps. GPU single instance. Run:
//! cargo test --locked --offline --test node_ui_few_slot_flow -- --test-threads=1 --nocapture
use autographics_auto_driver::node_ui::NodeUiApp;
use std::path::PathBuf;

const ARTIFACT: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-few/ui.json");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-few/world.at");
const MULTI_ARTIFACT: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-few-multi/ui.json");
const MULTI_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-few-multi/world.at");
const EMPTY_ARTIFACT: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-few-empty/ui.json");
const EMPTY_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-few-empty/world.at");

const DT: f32 = 1.0 / 60.0;
/// 21 Han chars × 3 bytes = 63 — the legal per-slot budget ceiling.
const TEXT_63_CJK: &str = "二十一字二十一字二十一字二十一字二十一字甲";

// ---------------------------------------------------------------- harness --

fn app_from(artifact: &str, world: &str, name: &str) -> NodeUiApp {
    let mut app = NodeUiApp::new(artifact, world, name).expect("app");
    app.start_run().expect("start_run");
    app
}

fn text_of(app: &NodeUiApp, instance: usize, slot: usize) -> String {
    app.texts(instance)[slot].clone()
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

fn scratch_dir(tag: &str) -> PathBuf {
    PathBuf::new().join(format!(
        "D:/autostack/plan-artifacts/plan-065/auto-ui/t03/{tag}-{}",
        std::process::id()
    ))
}

// ------------------------------------------------------------------ V-02 --

#[test]
fn few_slot_family_a_real_clicks_tick_isolation_and_pruned_digits() {
    let gpu = agp::renderer::GpuDevice::new().unwrap();
    let mut surface =
        agp::host::iced_surface::IcedSurface::new(&gpu, iced::Size::new(480.0, 900.0), 1.0)
            .unwrap();
    let mut app = app_from(ARTIFACT, WORLD, "plan065-few-a");
    let (root, a, b, c, d) = (
        app.instance_index("Main").unwrap(),
        app.instance_index("Main/a").unwrap(),
        app.instance_index("Main/b").unwrap(),
        app.instance_index("Main/c").unwrap(),
        app.instance_index("Main/d").unwrap(),
    );
    // Baseline frame: start params and zero states (a "0", b "100", c "0",
    // d "10"; root flips "0"; Few2 button labels ride slot 1).
    let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    assert_eq!(text_of(&app, a, 0), "0");
    assert_eq!(text_of(&app, b, 0), "100");
    assert_eq!(text_of(&app, c, 0), "0");
    assert_eq!(text_of(&app, d, 0), "10");
    assert_eq!(text_of(&app, c, 1), "加");
    assert_eq!(text_of(&app, root, 0), "翻转");
    assert_eq!(text_of(&app, root, 1), "0");

    // Real clicks: c twice / d once — 2:1 isolation through the PRUNED
    // digits path (decrement: 0-2 = -2 exercises the negative branch, no
    // half-buffer b dispatch anywhere).
    let n = click(&mut app, &mut surface, "Main/c#bump")
        + click(&mut app, &mut surface, "Main/c#bump")
        + click(&mut app, &mut surface, "Main/d#bump");
    assert_eq!(n, 3, "three real widget messages");
    assert_eq!(text_of(&app, c, 0), "-2", "pruned digits negative branch");
    assert_eq!(text_of(&app, d, 0), "9");
    assert_eq!(text_of(&app, d, 1), "加", "button label slot intact");

    // Root 2-text def: real flip click increments the IntField slot.
    let n = click(&mut app, &mut surface, "Main#flip");
    assert_eq!(n, 1);
    assert_eq!(text_of(&app, root, 1), "1");

    // Tick only via explicit logical steps: 1-slot Few1 counts ride the
    // per-instance Life.Tick handler — starts differ by param (isolation).
    for _ in 0..3 {
        app.step(DT).expect("step");
    }
    let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    assert_eq!(text_of(&app, a, 0), "3", "a: 0 + 3 ticks");
    assert_eq!(text_of(&app, b, 0), "103", "b: 100 + 3 ticks (multi-digit)");
    assert_eq!(text_of(&app, c, 0), "-2", "Few2 has no tick handler");
    assert!(app.error.is_none());
}

#[test]
fn few_slot_family_b_bool_switch_and_63_byte_content() {
    let gpu = agp::renderer::GpuDevice::new().unwrap();
    let mut surface =
        agp::host::iced_surface::IcedSurface::new(&gpu, iced::Size::new(520.0, 980.0), 1.0)
            .unwrap();
    let mut app = app_from(MULTI_ARTIFACT, MULTI_WORLD, "plan065-few-b");
    let (root, e, f, g, h) = (
        app.instance_index("Main").unwrap(),
        app.instance_index("Main/e").unwrap(),
        app.instance_index("Main/f").unwrap(),
        app.instance_index("Main/g").unwrap(),
        app.instance_index("Main/h").unwrap(),
    );
    let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    // Few3 baseline: bool "true", num start; Few4: 63-byte literal slot 0,
    // bool "false", param num, static tail — 4-slot def through the real VM.
    assert_eq!(text_of(&app, e, 0), "true");
    assert_eq!(text_of(&app, e, 1), "0");
    assert_eq!(text_of(&app, e, 2), "切换");
    assert_eq!(text_of(&app, f, 1), "10");
    assert_eq!(
        text_of(&app, g, 0),
        TEXT_63_CJK,
        "63-byte content byte-exact"
    );
    assert_eq!(text_of(&app, g, 1), "false");
    assert_eq!(text_of(&app, g, 2), "7");
    assert_eq!(text_of(&app, g, 3), "尾");
    assert_eq!(text_of(&app, h, 2), "8", "param isolation on 4-slot def");
    // Root 4-text def: static hint, flips, ticks.
    assert_eq!(text_of(&app, root, 0), "翻转");
    assert_eq!(text_of(&app, root, 1), "0");
    assert_eq!(text_of(&app, root, 2), "提示");
    assert_eq!(text_of(&app, root, 3), "0");

    // Real clicks: e once -> bool switches false, num -5 (frozen dual-branch
    // digits negative); f twice -> bool returns true, num 0.
    let n = click(&mut app, &mut surface, "Main/e#bump")
        + click(&mut app, &mut surface, "Main/f#bump")
        + click(&mut app, &mut surface, "Main/f#bump");
    assert_eq!(n, 3);
    assert_eq!(text_of(&app, e, 0), "false", "bool switch once");
    assert_eq!(
        text_of(&app, e, 1),
        "-5",
        "negative through dual-branch digits"
    );
    assert_eq!(text_of(&app, f, 0), "true", "bool switch twice returns");
    assert_eq!(text_of(&app, f, 1), "0");
    // Root flip button.
    let n = click(&mut app, &mut surface, "Main#flip");
    assert_eq!(n, 1);
    assert_eq!(text_of(&app, root, 1), "1");
    // 12 logical steps: root ticks text goes multi-digit.
    for _ in 0..12 {
        app.step(DT).expect("step");
    }
    let _ = surface.frame(&mut app, &[], iced_wgpu::core::mouse::Cursor::Unavailable);
    assert_eq!(text_of(&app, root, 3), "12", "multi-digit root ticks");
    assert_eq!(text_of(&app, g, 0), TEXT_63_CJK, "63-byte content stable");
    assert!(app.error.is_none());
}

#[test]
fn zero_text_app_runs_and_stays_minimal() {
    // 0-slot legal application: starts, steps, renders state, no texts.
    let mut app = app_from(EMPTY_ARTIFACT, EMPTY_WORLD, "plan065-few-empty");
    assert_eq!(app.row_count(), 2, "root + one Few0 instance");
    for _ in 0..3 {
        app.step(DT).expect("step");
    }
    assert!(app.error.is_none());
    for idx in 0..app.row_count() {
        assert!(app.texts(idx).is_empty(), "no text slots anywhere");
    }
}

// ------------------------------------------------------------------ V-03 --

#[test]
fn damaged_few_slot_world_is_named_rejected_and_repairs_next_run() {
    // Identity chain on a few-slot product: byte flip -> named rejection
    // BEFORE any Run; exact restore -> the next actual Run recovers.
    let mut world = WORLD.to_string();
    let at = world.find("var main_ui_text_0").expect("marker");
    world.replace_range(at..at + 1, "X");
    let err = NodeUiApp::new(ARTIFACT, &world, "plan065-damaged")
        .err()
        .expect("damaged world must be rejected");
    assert!(!err.is_empty(), "named rejection: {err}");
    // exact repair -> next Run green.
    let repaired = app_from(ARTIFACT, WORLD, "plan065-repaired");
    assert!(repaired.error.is_none());
    let _ = scratch_dir("damaged"); // scratch scope registered; no files kept
}
