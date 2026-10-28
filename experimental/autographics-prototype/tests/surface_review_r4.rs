//! PLAN-830 review r4 permanent regressions (F-830-R4-01).
//!
//! The R3 cases covered "batch -> frame -> blur". The real host accumulates
//! events in `pending` and only flushes them in `about_to_wait`, so a blur or
//! a revalidated unmount can happen BEFORE the batch is rendered. The runtime
//! must then treat "no router target" as CANCELLED (suppress focus), not as
//! "nothing routed yet" (allow every surface to focus).

use agp::core::composition::input::InputOutcome;
use agp::core::composition::{CompositionChange, MountKind, Rect, SurfaceId};
use agp::core::scene::Color;
use agp::host::composition_input::CompositionInput;
use agp::host::iced_surface::{SurfaceElement, SurfaceUi};
use agp::host::surface_runtime::{Game2dContent, IcedContent, SurfaceContent, SurfaceRuntime};
use agp::renderer::GpuDevice;
use iced_wgpu::core::widget::Id;
use iced_wgpu::core::{mouse, Event, Point};
use std::collections::HashMap;

struct Form {
    name: &'static str,
    value: String,
}

impl SurfaceUi for Form {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        iced::widget::text_input("type", &self.value)
            .id(self.name)
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

fn setup() -> (SurfaceRuntime, CompositionInput, [SurfaceId; 2]) {
    let mut rt = SurfaceRuntime::new(GpuDevice::new().unwrap()).unwrap();
    let c = Game2dContent::simple(rt.gpu(), Color::rgb(0.0, 0.0, 0.0));
    let top = rt
        .add_surface([440.0, 160.0], SurfaceContent::Game2d(Box::new(c)))
        .unwrap();
    let v = rt.add_view(top, true).unwrap();
    let mut ids = Vec::new();
    for (name, x) in [("exit-a", 10.0), ("exit-b", 230.0)] {
        let ui = IcedContent::borrowed(
            rt.gpu(),
            [200.0, 100.0],
            Box::new(Form {
                name,
                value: String::new(),
            }),
        );
        let id = rt
            .add_surface([200.0, 100.0], SurfaceContent::Iced(Box::new(ui)))
            .unwrap();
        rt.add_portal(
            v,
            id,
            MountKind::Flat2d {
                offset: [x, 10.0],
                scale: [1.0, 1.0],
                clip: Rect::at(x, 10.0, 200.0, 100.0),
            },
            true,
        )
        .unwrap();
        ids.push(id);
    }
    rt.frame(None, &HashMap::new()).unwrap();
    (
        rt,
        CompositionInput::new([440.0, 160.0], [440.0, 160.0]),
        [ids[0], ids[1]],
    )
}

fn enqueue_click(
    rt: &SurfaceRuntime,
    input: &mut CompositionInput,
    win: [f32; 2],
    ev: &mut HashMap<SurfaceId, Vec<Event>>,
    cursors: &mut HashMap<SurfaceId, [f32; 2]>,
) {
    let Some(InputOutcome::Delivered { surface, local }) = input.pointer_pressed(&rt.graph, win)
    else {
        panic!("must hit real control")
    };
    ev.entry(surface).or_default().extend([
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(local[0], local[1]),
        }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    ]);
    cursors.insert(surface, local);
    let target = input.pointer_released().unwrap();
    ev.entry(target)
        .or_default()
        .push(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));
}

fn focused(rt: &mut SurfaceRuntime, ids: [SurfaceId; 2]) -> (bool, bool) {
    (
        rt.iced_text_input_focused(ids[0], Id::new("exit-a")),
        rt.iced_text_input_focused(ids[1], Id::new("exit-b")),
    )
}

/// The real host's `apply_focus_loss` / IME-host equivalent.
fn append_host_blur(
    rt: &mut SurfaceRuntime,
    input: &mut CompositionInput,
    ev: &mut HashMap<SurfaceId, Vec<Event>>,
) {
    if let Some(surface) = input.window_focus_lost() {
        rt.release_iced_focus(surface);
        ev.entry(surface).or_default().extend([
            Event::Window(iced_wgpu::core::window::Event::Unfocused),
            Event::InputMethod(iced_wgpu::core::input_method::Event::Closed),
        ]);
    }
}

#[test]
fn cancel_before_pending_flush_does_not_restore_focus() {
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    // Blur happens while the A/B batch is still pending.
    append_host_blur(&mut rt, &mut input, &mut ev);
    assert_eq!(input.keyboard_target(), None);
    assert_eq!(rt.graph.routed_focus(), None);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(
        focused(&mut rt, ids),
        (false, false),
        "a cancelled batch must not revive editor state"
    );
}

#[test]
fn revalidated_unmount_before_pending_flush_keeps_no_editor() {
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    // The documented protocol: unmount, revalidate, release the reported
    // focus, close IME — all before the pending batch renders.
    let portal = rt
        .graph
        .portals()
        .iter()
        .find(|p| p.desc.child == ids[1])
        .unwrap()
        .id;
    rt.graph
        .submit_one(CompositionChange::RemovePortal(portal))
        .unwrap();
    let cancelled = input.revalidate(&rt.graph);
    assert_eq!(cancelled, Some(ids[1]));
    rt.release_iced_focus(cancelled.unwrap());
    ev.entry(ids[1]).or_default().push(Event::InputMethod(
        iced_wgpu::core::input_method::Event::Closed,
    ));
    assert_eq!(input.keyboard_target(), None);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(
        focused(&mut rt, ids),
        (false, false),
        "a correctly revalidated unmount must not revive editors"
    );
}

#[test]
fn cancel_after_flush_is_a_valid_control() {
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(focused(&mut rt, ids), (false, true));
    let mut blur = HashMap::new();
    append_host_blur(&mut rt, &mut input, &mut blur);
    rt.frame_with_cursors(None, &blur, &cur).unwrap();
    assert_eq!(focused(&mut rt, ids), (false, false));
}

#[test]
fn a_new_click_after_cancel_is_accepted() {
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    append_host_blur(&mut rt, &mut input, &mut ev);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(focused(&mut rt, ids), (false, false));

    // A genuinely new routed press re-establishes exactly one target.
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(focused(&mut rt, ids), (true, false));
    assert_eq!(input.keyboard_target(), Some(ids[0]));
}

#[test]
fn public_focus_after_cancel_is_accepted() {
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    append_host_blur(&mut rt, &mut input, &mut ev);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(focused(&mut rt, ids), (false, false));
    assert!(rt.focus_iced_text_input(ids[1], Id::new("exit-b")));
    rt.frame(None, &HashMap::new()).unwrap();
    assert_eq!(focused(&mut rt, ids), (false, true));
}
