//! PLAN-830 review r2 permanent regressions (F-830-R2-01..06).
//!
//! Each case here is derived from the independent review's holdout: it drives
//! the REAL production path (routed press -> real iced widget message ->
//! composed consumer texture) and fails without the repaired invariant. The
//! accepted controls keep the "plausible wrong implementation" distinguishable.

use agp::core::composition::input::InputOutcome;
use agp::core::composition::{CompositionChange, CompositionError, MountKind, Rect, SurfaceId};
use agp::core::scene::Color;
use agp::host::composition_input::CompositionInput;
use agp::host::iced_surface::{SurfaceElement, SurfaceUi};
use agp::host::surface_runtime::{
    Game2dContent, IcedContent, Scene3dContent, SurfaceContent, SurfaceRuntime,
};
use agp::renderer::GpuDevice;
use iced_wgpu::core::widget::Id;
use iced_wgpu::core::{mouse, Event, Point};
use std::collections::HashMap;

fn runtime() -> SurfaceRuntime {
    SurfaceRuntime::new(GpuDevice::new().unwrap()).unwrap()
}

fn canvas(rt: &mut SurfaceRuntime, size: [f32; 2], color: Color) -> SurfaceId {
    let c = Game2dContent::simple(rt.gpu(), color);
    rt.add_surface(size, SurfaceContent::Game2d(Box::new(c)))
        .unwrap()
}

fn flat(x: f32, y: f32, w: f32, h: f32) -> MountKind {
    MountKind::Flat2d {
        offset: [x, y],
        scale: [1.0, 1.0],
        clip: Rect::at(x, y, w, h),
    }
}

// ---------------------------------------------------------------------------
// F-830-R2-01: a real widget message must refresh EVERY canvas consumer in
// the same frame (live content, not a static alpha or a manual mark).
// ---------------------------------------------------------------------------

#[derive(Default)]
struct ChangingUi {
    counter: agp::host::cases::surface_scenarios::CounterUi,
}

impl SurfaceUi for ChangingUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        self.counter.view()
    }
    fn update(&mut self, m: String) {
        self.counter.update(m)
    }
    fn background(&self) -> iced_wgpu::core::Color {
        if self.counter.presses == 0 {
            iced_wgpu::core::Color::from_rgb(1.0, 0.0, 0.0)
        } else {
            iced_wgpu::core::Color::from_rgb(0.0, 0.0, 1.0)
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn click<F>(rt: &mut SurfaceRuntime, input: &mut CompositionInput, win: [f32; 2], mutate: F)
where
    F: Fn([f32; 2]) -> [f32; 2],
{
    let Some(InputOutcome::Delivered { surface, local }) = input.pointer_pressed(&rt.graph, win)
    else {
        panic!("click must route")
    };
    let local = mutate(local);
    let cursors = HashMap::from([(surface, local)]);
    let ev = HashMap::from([(
        surface,
        vec![
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(local[0], local[1]),
            }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        ],
    )]);
    rt.frame_with_cursors(None, &ev, &cursors).unwrap();
    let released = input.pointer_released().unwrap();
    let ev = HashMap::from([(
        released,
        vec![Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))],
    )]);
    rt.frame_with_cursors(None, &ev, &cursors).unwrap();
}

#[test]
fn iced_message_update_refreshes_every_composed_consumer() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [260.0, 220.0], Color::rgb(0.0, 0.0, 0.0));
    let view = rt.add_view(top, true).unwrap();
    // Two consumers of the same iced child: the portal parent AND a second
    // canvas that also consumes it, so "one consumer refreshed" is not enough.
    let second = canvas(&mut rt, [120.0, 120.0], Color::rgb(0.0, 0.0, 0.0));
    let content = IcedContent::borrowed(rt.gpu(), [200.0, 160.0], Box::new(ChangingUi::default()));
    let child = rt
        .add_surface([200.0, 160.0], SurfaceContent::Iced(Box::new(content)))
        .unwrap();
    let second_view = rt.add_view(second, false).unwrap();
    rt.add_portal(view, child, flat(20.0, 20.0, 200.0, 160.0), true)
        .unwrap();
    rt.add_portal(second_view, child, flat(10.0, 10.0, 100.0, 100.0), true)
        .unwrap();
    rt.frame(None, &HashMap::new()).unwrap();

    let mut input = CompositionInput::new([260.0, 220.0], [260.0, 220.0]);
    let before = rt.probe_pixel(top, [210.0, 160.0]).unwrap();
    assert_eq!(before, [255, 0, 0, 255], "control: red before the message");

    click(&mut rt, &mut input, [120.0, 47.0], |l| l);

    let child_px = rt.probe_pixel(child, [190.0, 140.0]).unwrap();
    let parent_px = rt.probe_pixel(top, [210.0, 160.0]).unwrap();
    let second_px = rt.probe_pixel(second, [15.0, 15.0]).unwrap();
    assert_eq!(child_px, [0, 0, 255, 255], "real button changed state");
    assert_eq!(
        parent_px, child_px,
        "same-frame composed output must follow the real event-driven child"
    );
    assert_eq!(
        second_px, child_px,
        "every consumer, not only the first portal, samples the new content"
    );

    // Control: a later static frame keeps the picture (no re-blend drift) and
    // an explicitly dirtied child still propagates through the old public path.
    rt.frame(None, &HashMap::new()).unwrap();
    assert_eq!(rt.probe_pixel(top, [210.0, 160.0]).unwrap(), child_px);
    rt.graph.mark_content_dirty(child);
    rt.frame(None, &HashMap::new()).unwrap();
    assert_eq!(rt.probe_pixel(top, [210.0, 160.0]).unwrap(), child_px);
}

// ---------------------------------------------------------------------------
// F-830-R2-04: same-device ownership is checked for EVERY adapter kind; an
// unverifiable adapter is refused instead of silently passing.
// ---------------------------------------------------------------------------

fn scene_on(device: &GpuDevice) -> Scene3dContent {
    Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh: agp::render3d::MeshPipeline::new(
            device,
            agp::renderer::target::OffscreenTarget::FORMAT,
        )
        .unwrap(),
        store: Default::default(),
        instances: vec![],
        sky: [0.0, 0.0, 0.0, 1.0],
        view_proj: agp::host::iced_surface_probe::probe_camera(1.0),
    }
}

#[test]
fn every_adapter_checks_its_real_device() {
    let mut rt = runtime();
    let second = GpuDevice::new().unwrap();
    let host = rt.gpu().token().ordinal;
    let foreign = second.token().ordinal;
    assert_ne!(host, foreign, "the control needs a real second device");

    // Foreign 3D content: the exact regression.
    let snapshot = rt.graph.snapshot();
    let scene = rt.add_surface(
        [100.0, 100.0],
        SurfaceContent::Scene3d(Box::new(scene_on(&second))),
    );
    assert_eq!(
        scene,
        Err(CompositionError::DeviceMismatch {
            expected: host,
            actual: foreign
        }),
        "foreign 3D content must be named-rejected like 2D content"
    );
    assert_eq!(snapshot, rt.graph.snapshot(), "rejection keeps the graph");

    // Foreign 2D and foreign iced content follow the same rule.
    let foreign_canvas = Game2dContent::simple(&second, Color::rgb(0.0, 0.0, 0.0));
    assert!(matches!(
        rt.add_surface(
            [100.0, 100.0],
            SurfaceContent::Game2d(Box::new(foreign_canvas))
        ),
        Err(CompositionError::DeviceMismatch { .. })
    ));
    let foreign_iced = IcedContent::borrowed(
        &second,
        [100.0, 100.0],
        Box::new(agp::host::cases::surface_scenarios::CounterUi::default()),
    );
    assert!(matches!(
        rt.add_surface([100.0, 100.0], SurfaceContent::Iced(Box::new(foreign_iced))),
        Err(CompositionError::DeviceMismatch { .. })
    ));

    // Unverifiable: a pipeline built from a raw device handle cannot prove
    // ownership -> fail closed (the old `None` branch admitted it).
    let raw = agp::render3d::MeshPipeline::new_with(
        rt.gpu().device(),
        rt.gpu().queue(),
        agp::renderer::target::OffscreenTarget::FORMAT,
    )
    .unwrap();
    let unverifiable = Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh: raw,
        store: Default::default(),
        instances: vec![],
        sky: [0.0, 0.0, 0.0, 1.0],
        view_proj: agp::host::iced_surface_probe::probe_camera(1.0),
    };
    assert!(matches!(
        rt.add_surface(
            [100.0, 100.0],
            SurfaceContent::Scene3d(Box::new(unverifiable))
        ),
        Err(CompositionError::DeviceUnverifiable { .. })
    ));
    assert_eq!(snapshot, rt.graph.snapshot(), "no admission side effects");

    // Legit control: the SAME device is accepted for all three kinds.
    assert!(rt
        .add_surface(
            [80.0, 80.0],
            SurfaceContent::Scene3d(Box::new(scene_on(rt.gpu())))
        )
        .is_ok());
    let own_canvas = Game2dContent::simple(rt.gpu(), Color::rgb(0.0, 0.0, 0.0));
    assert!(rt
        .add_surface([80.0, 80.0], SurfaceContent::Game2d(Box::new(own_canvas)))
        .is_ok());
    let own_iced = IcedContent::borrowed(
        rt.gpu(),
        [80.0, 80.0],
        Box::new(agp::host::cases::surface_scenarios::CounterUi::default()),
    );
    assert!(rt
        .add_surface([80.0, 80.0], SurfaceContent::Iced(Box::new(own_iced)))
        .is_ok());
}

// ---------------------------------------------------------------------------
// F-830-R2-05: target accounting balances every real target exit, including
// resize replacement and discarded preparations.
// ---------------------------------------------------------------------------

#[test]
fn target_accounting_balances_every_exit() {
    let mut rt = runtime();
    assert_eq!(rt.stats.live_targets(), 0);
    let s = canvas(&mut rt, [20.0, 20.0], Color::rgb(1.0, 0.0, 0.0));
    assert_eq!((rt.stats.target_creations, rt.stats.target_drops), (1, 0));

    // resize replaces the old target: the old one is REALLY dropped.
    rt.resize_surface(s, [30.0, 20.0]).unwrap();
    assert_eq!((rt.stats.target_creations, rt.stats.target_drops), (2, 1));
    // identical resize is a no-op (no fake rebuild, no fake drop).
    rt.resize_surface(s, [30.0, 20.0]).unwrap();
    assert_eq!((rt.stats.target_creations, rt.stats.target_drops), (2, 1));
    // rejected oversized resize creates nothing (limit checked first).
    let too_big = [1.0, 1.0];
    let _ = too_big;
    let limit = rt.gpu().device().limits().max_texture_dimension_2d as f32;
    let rejected = rt.resize_surface(s, [limit + 1.0, 32.0]);
    assert!(rejected.is_err());
    assert_eq!((rt.stats.target_creations, rt.stats.target_drops), (2, 1));
    // rejected foreign content creates nothing.
    let second = GpuDevice::new().unwrap();
    let foreign = Game2dContent::simple(&second, Color::rgb(0.0, 0.0, 0.0));
    assert!(rt
        .add_surface([10.0, 10.0], SurfaceContent::Game2d(Box::new(foreign)))
        .is_err());
    assert_eq!((rt.stats.target_creations, rt.stats.target_drops), (2, 1));

    rt.drop_surface(s).unwrap();
    assert_eq!((rt.stats.target_creations, rt.stats.target_drops), (2, 2));
    assert_eq!(rt.stats.live_targets(), 0, "no target outlives its slot");
    assert_eq!(rt.graph.live_surfaces(), rt.stats.live_targets() as usize);
}

// ---------------------------------------------------------------------------
// F-830-R2-02: switching the routed focus between two REAL text inputs must
// leave exactly one control focused; a window blur must not find a stale one.
// ---------------------------------------------------------------------------

struct Form {
    id: &'static str,
    value: String,
}

impl SurfaceUi for Form {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        iced::widget::text_input("type", &self.value)
            .id(self.id)
            .on_input(|s| s)
            .into()
    }
    fn update(&mut self, m: String) {
        self.value = m;
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::WHITE
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[test]
fn focus_migration_unfocuses_the_old_iced_control() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [440.0, 160.0], Color::rgb(0.0, 0.0, 0.0));
    let v = rt.add_view(top, true).unwrap();
    let mut ids = Vec::new();
    for (name, x) in [("review-a", 10.0), ("review-b", 230.0)] {
        let ui = IcedContent::borrowed(
            rt.gpu(),
            [200.0, 100.0],
            Box::new(Form {
                id: name,
                value: String::new(),
            }),
        );
        let s = rt
            .add_surface([200.0, 100.0], SurfaceContent::Iced(Box::new(ui)))
            .unwrap();
        rt.add_portal(v, s, flat(x, 10.0, 200.0, 100.0), true)
            .unwrap();
        ids.push(s);
    }
    rt.frame(None, &HashMap::new()).unwrap();

    let mut input = CompositionInput::new([440.0, 160.0], [440.0, 160.0]);
    click(&mut rt, &mut input, [30.0, 25.0], |l| l);
    let focused_a = rt.iced_text_input_focused(ids[0], Id::new("review-a"));
    assert!(focused_a, "control: the first real control got the click");

    click(&mut rt, &mut input, [250.0, 25.0], |l| l);
    let a = rt.iced_text_input_focused(ids[0], Id::new("review-a"));
    let b = rt.iced_text_input_focused(ids[1], Id::new("review-b"));
    assert_eq!(input.keyboard_target(), Some(ids[1]), "router moved to B");
    assert!(b, "B is the real focused control");
    assert!(!a, "the OLD control must not keep widget focus");

    let cancelled = input.window_focus_lost().unwrap();
    rt.release_iced_focus(cancelled);
    let a_after = rt.iced_text_input_focused(ids[0], Id::new("review-a"));
    let b_after = rt.iced_text_input_focused(ids[1], Id::new("review-b"));
    assert!(
        !a_after && !b_after,
        "no control may keep edit state after a window blur"
    );

    // Explicit public focus also enforces the single target.
    rt.focus_iced_text_input(ids[0], Id::new("review-a"));
    assert!(rt.iced_text_input_focused(ids[0], Id::new("review-a")));
    rt.focus_iced_text_input(ids[1], Id::new("review-b"));
    assert!(
        !rt.iced_text_input_focused(ids[0], Id::new("review-a")),
        "a public focus call moves focus away from the other surface"
    );
}

// ---------------------------------------------------------------------------
// F-830-R2-03: the IME anchor follows the ACTUAL caret (index/selection/scroll
// through the real widget), and the formal oracle is the caret rect, not the
// whole plane.
// ---------------------------------------------------------------------------

fn type_key(rt: &mut SurfaceRuntime, surface: SurfaceId, ch: &str, at: [f32; 2]) {
    let key = iced_wgpu::core::keyboard::Key::Character(iced_wgpu::core::SmolStr::new(ch));
    let ev = HashMap::from([(
        surface,
        vec![Event::Keyboard(
            iced_wgpu::core::keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
                    iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced_wgpu::core::keyboard::Location::Standard,
                modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
                text: Some(iced_wgpu::core::SmolStr::new(ch)),
                repeat: false,
            },
        )],
    )]);
    let mut cursors = HashMap::new();
    cursors.insert(surface, at);
    rt.frame_with_cursors(None, &ev, &cursors).unwrap();
}

#[test]
fn ime_anchor_follows_the_real_caret() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [400.0, 300.0], Color::rgb(0.0, 0.0, 0.0));
    let top_view = rt.add_view(top, true).unwrap();
    let ui = IcedContent::borrowed(
        rt.gpu(),
        [200.0, 80.0],
        Box::new(Form {
            id: "caret-form",
            value: String::new(),
        }),
    );
    let child = rt
        .add_surface([200.0, 80.0], SurfaceContent::Iced(Box::new(ui)))
        .unwrap();
    let mount = flat(40.0, 30.0, 200.0, 80.0);
    let portal = rt.add_portal(top_view, child, mount, true).unwrap();
    rt.frame(None, &HashMap::new()).unwrap();

    let mut input = CompositionInput::new([400.0, 300.0], [400.0, 300.0]);
    // Click the ACTUAL control (its real bounds), not a hard-coded point.
    let bounds = rt.iced_widget_bounds(child, Id::new("caret-form")).unwrap();
    let click_at = [
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ];
    click(&mut rt, &mut input, [70.0, 70.0], |_| click_at);

    let caret = rt.ime_caret_local(child);
    assert!(caret.is_some(), "the focused text_input reports a caret");
    let anchor0 = input.ime_anchor_window(&rt.graph, &rt, child).unwrap();
    let rect0 = input.ime_caret_window_rect(&rt.graph, &rt, child).unwrap();
    assert!(
        anchor0[0] >= rect0[0] - 1.0
            && anchor0[0] <= rect0[2] + 1.0
            && anchor0[1] >= rect0[1] - 1.0
            && anchor0[1] <= rect0[3] + 1.0,
        "the anchor is inside the REAL caret rect: anchor={anchor0:?} caret={rect0:?}"
    );
    // The caret rect is strictly smaller than the mount: it is not the plane.
    assert!(
        (rect0[2] - rect0[0]) < 10.0 && (rect0[3] - rect0[1]) < 40.0,
        "caret rect is a caret, not the whole plane: {rect0:?}"
    );

    // Typing moves the caret inside the SAME mount -> the OS anchor follows.
    let at = [
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ];
    type_key(&mut rt, child, "a", at);
    type_key(&mut rt, child, "b", at);
    let anchor1 = input.ime_anchor_window(&rt.graph, &rt, child).unwrap();
    let rect1 = input.ime_caret_window_rect(&rt.graph, &rt, child).unwrap();
    assert!(
        anchor1[0] > anchor0[0] + 1.0,
        "caret moved right with the text: {anchor0:?} -> {anchor1:?}"
    );
    assert!(
        (rect1[2] - rect1[0]) < 10.0,
        "still a caret rect after typing: {rect1:?}"
    );

    // Moving the MOUNT keeps the SAME caret local and projects it elsewhere.
    let moved = flat(120.0, 80.0, 200.0, 80.0);
    rt.graph
        .submit_one(CompositionChange::SetMount(portal, moved))
        .unwrap();
    let anchor2 = input.ime_anchor_window(&rt.graph, &rt, child).unwrap();
    let rect2 = input.ime_caret_window_rect(&rt.graph, &rt, child).unwrap();
    assert!(
        (anchor2[0] - anchor1[0]).abs() > 20.0,
        "the anchor follows the mount: {anchor1:?} -> {anchor2:?}"
    );
    assert!(
        anchor2[0] >= rect2[0] - 1.0 && anchor2[0] <= rect2[2] + 1.0,
        "and lands inside the moved caret rect"
    );

    // Scrolling: typing past the control width moves the caret only within
    // the VISIBLE text area (iced applies its scroll offset), so the anchor
    // must not run off the mount. A host-side fixed point could never see
    // this; it is read from the widget's own caret rect.
    let bounds = rt.iced_widget_bounds(child, Id::new("caret-form")).unwrap();
    let at = [
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ];
    for _ in 0..40 {
        type_key(&mut rt, child, "m", at);
    }
    let caret_scrolled = rt.ime_caret_local(child).expect("caret after long input");
    assert!(
        caret_scrolled[0] < bounds.x + bounds.width,
        "the caret stays inside the visible control after scrolling: \
         caret={caret_scrolled:?} bounds=({}, {})",
        bounds.x,
        bounds.width
    );
    let anchor3 = input.ime_anchor_window(&rt.graph, &rt, child).unwrap();
    let rect3 = input.ime_caret_window_rect(&rt.graph, &rt, child).unwrap();
    assert!(
        anchor3[0] >= rect3[0] - 1.0 && anchor3[0] <= rect3[2] + 1.0,
        "the OS anchor follows the scrolled caret: {anchor3:?} vs {rect3:?}"
    );
}
