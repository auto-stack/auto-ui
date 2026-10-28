//! PLAN-830 review r3 permanent regressions (F-830-R3-01/02).
//!
//! The case the R2 regression missed: a real host batches several routed
//! clicks into ONE frame, so the runtime cannot derive the keyboard target
//! from "some surface in the map has a ButtonPressed" — every batched list
//! still runs and each iced tree re-focused itself. The router's final target
//! is the authority; these cases drive the real router and the real widgets.

use agp::core::composition::input::InputOutcome;
use agp::core::composition::{
    CompositionChange, CompositionGraph, ContentKind, MountKind, Rect, SurfaceDesc, SurfaceId,
};
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
    for (name, x) in [("batch-a", 10.0), ("batch-b", 230.0)] {
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

/// The real host shape: route a click, keep the ACTUAL returned surface/local
/// in `pending`/`cursors`, and only hand the whole batch to one frame.
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
        rt.iced_text_input_focused(ids[0], Id::new("batch-a")),
        rt.iced_text_input_focused(ids[1], Id::new("batch-b")),
    )
}

#[test]
fn separate_frame_focus_transfer_is_a_valid_control() {
    let (mut rt, mut input, ids) = setup();
    for pos in [[30.0, 25.0], [250.0, 25.0]] {
        let mut ev = HashMap::new();
        let mut cur = HashMap::new();
        enqueue_click(&rt, &mut input, pos, &mut ev, &mut cur);
        rt.frame_with_cursors(None, &ev, &cur).unwrap();
    }
    assert_eq!(input.keyboard_target(), Some(ids[1]));
    assert_eq!(focused(&mut rt, ids), (false, true));
}

#[test]
fn two_clicks_queued_before_one_frame_keep_only_last_focus() {
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    assert_eq!(input.keyboard_target(), Some(ids[1]));
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(
        focused(&mut rt, ids),
        (false, true),
        "only the router's final target may keep widget focus"
    );
    let target = input.window_focus_lost().unwrap();
    rt.release_iced_focus(target);
    assert_eq!(focused(&mut rt, ids), (false, false));
}

#[test]
fn reversed_batch_keeps_the_router_target() {
    // B then A in one batch: the LAST routed press is A, so A must be the one
    // real edit target — a render-order/iteration-order guess would fail.
    let (mut rt, mut input, ids) = setup();
    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(&rt, &mut input, [250.0, 25.0], &mut ev, &mut cur);
    enqueue_click(&rt, &mut input, [30.0, 25.0], &mut ev, &mut cur);
    assert_eq!(input.keyboard_target(), Some(ids[0]));
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(focused(&mut rt, ids), (true, false));
}

#[test]
fn earlier_batched_click_still_acts() {
    // The invariant is about focus, not about dropping the earlier click.
    // Surface A carries a real button ABOVE its text input; a batched
    // A-button click followed by a B-input click must still fire A's button
    // message while only B keeps the edit target.
    let mut rt = SurfaceRuntime::new(GpuDevice::new().unwrap()).unwrap();
    let top = rt
        .add_surface(
            [440.0, 160.0],
            SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                rt.gpu(),
                Color::rgb(0.0, 0.0, 0.0),
            ))),
        )
        .unwrap();
    let v = rt.add_view(top, true).unwrap();
    let mut ids = Vec::new();
    for (name, x) in [("mix-a", 10.0), ("mix-b", 230.0)] {
        let ui = IcedContent::borrowed(
            rt.gpu(),
            [200.0, 100.0],
            Box::new(Mixed {
                name,
                count: 0,
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
    let mut input = CompositionInput::new([440.0, 160.0], [440.0, 160.0]);
    let a_input = rt
        .iced_widget_bounds(ids[0], Id::new("mix-a"))
        .expect("A input bounds");
    let button_y = a_input.y - 16.0; // inside A's button, above the input

    let mut ev = HashMap::new();
    let mut cur = HashMap::new();
    enqueue_click(
        &rt,
        &mut input,
        [a_input.x + 20.0, 10.0 + button_y],
        &mut ev,
        &mut cur,
    );
    let b_input = rt
        .iced_widget_bounds(ids[1], Id::new("mix-b"))
        .expect("B input bounds");
    enqueue_click(
        &rt,
        &mut input,
        [
            230.0 + b_input.x + 20.0,
            10.0 + b_input.y + b_input.height / 2.0,
        ],
        &mut ev,
        &mut cur,
    );
    rr_frame(&mut rt, ev, cur);
    assert_eq!(count_of(&rt, ids[0]), 1, "the earlier click's action fired");
    let a_focused = rt.iced_text_input_focused(ids[0], Id::new("mix-a"));
    let b_focused = rt.iced_text_input_focused(ids[1], Id::new("mix-b"));
    assert_eq!(
        (a_focused, b_focused),
        (false, true),
        "only the last routed target keeps the edit state"
    );

    // Typing goes to exactly one control (B), proving the single target.
    let key = iced_wgpu::core::keyboard::Key::Character(iced_wgpu::core::SmolStr::new("z"));
    let ev = HashMap::from([(
        ids[1],
        vec![Event::Keyboard(
            iced_wgpu::core::keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
                    iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced_wgpu::core::keyboard::Location::Standard,
                modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
                text: Some(iced_wgpu::core::SmolStr::new("z")),
                repeat: false,
            },
        )],
    )]);
    let mut cur = HashMap::new();
    cur.insert(ids[1], [120.0, 50.0]);
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
    assert_eq!(mixed_text(&rt, ids[0]), "");
    assert_eq!(mixed_text(&rt, ids[1]), "z");
}

struct Mixed {
    name: &'static str,
    count: u32,
    value: String,
}

impl SurfaceUi for Mixed {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        iced::widget::column![
            iced::widget::button(iced::widget::text(format!("bump {}", self.count)))
                .on_press("bump".to_string()),
            iced::widget::text_input("type", &self.value)
                .id(self.name)
                .on_input(|s| s),
        ]
        .spacing(8)
        .padding(10)
        .into()
    }
    fn update(&mut self, m: String) {
        if m == "bump" {
            self.count += 1;
        } else {
            self.value = m;
        }
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

fn mixed_text(rt: &SurfaceRuntime, id: SurfaceId) -> String {
    match rt.content(id) {
        Some(SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<Mixed>()
                .map(|m| m.value.clone())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn count_of(rt: &SurfaceRuntime, id: SurfaceId) -> u32 {
    match rt.content(id) {
        Some(SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<Mixed>()
                .map(|m| m.count)
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn rr_frame(
    rt: &mut SurfaceRuntime,
    ev: HashMap<SurfaceId, Vec<Event>>,
    cur: HashMap<SurfaceId, [f32; 2]>,
) {
    rt.frame_with_cursors(None, &ev, &cur).unwrap();
}

#[test]
fn rejected_foreign_focus_keeps_current_edit_state() {
    let (mut rt, _, ids) = setup();
    assert!(rt.focus_iced_text_input(ids[0], Id::new("batch-a")));
    assert_eq!(focused(&mut rt, ids), (true, false));
    let mut other = CompositionGraph::new();
    let foreign = other
        .submit_one(CompositionChange::AddSurface {
            desc: SurfaceDesc {
                logical_size: [10.0, 10.0],
                content: ContentKind::Iced,
            },
        })
        .unwrap()
        .first_surface()
        .unwrap();
    let accepted = rt.focus_iced_text_input(foreign, Id::new("batch-a"));
    assert!(!accepted, "the foreign id is rejected");
    assert_eq!(
        focused(&mut rt, ids),
        (true, false),
        "rejection must not clear unrelated valid focus"
    );
    // The rejected call must also not have moved the router authority: a
    // later frame must still keep A as the only edit target.
    rt.frame(None, &HashMap::new()).unwrap();
    assert_eq!(focused(&mut rt, ids), (true, false));
    // A later VALID focus still migrates the single target.
    assert!(rt.focus_iced_text_input(ids[1], Id::new("batch-b")));
    assert_eq!(focused(&mut rt, ids), (false, true));
}

#[test]
fn rejected_stale_and_non_iced_focus_keep_current_state() {
    let (mut rt, _, ids) = setup();
    assert!(rt.focus_iced_text_input(ids[0], Id::new("batch-a")));
    assert_eq!(focused(&mut rt, ids), (true, false));

    // Stale id: a real surface of THIS runtime that was dropped.
    let scratch = {
        let c = Game2dContent::simple(rt.gpu(), Color::rgb(0.0, 0.0, 0.0));
        rt.add_surface([16.0, 16.0], SurfaceContent::Game2d(Box::new(c)))
            .unwrap()
    };
    rt.drop_surface(scratch).unwrap();
    assert!(!rt.focus_iced_text_input(scratch, Id::new("batch-a")));
    assert_eq!(focused(&mut rt, ids), (true, false));

    // Non-iced surface: exists, but has no widget focus to give.
    let game = {
        let c = Game2dContent::simple(rt.gpu(), Color::rgb(0.0, 0.0, 0.0));
        rt.add_surface([16.0, 16.0], SurfaceContent::Game2d(Box::new(c)))
            .unwrap()
    };
    assert!(!rt.focus_iced_text_input(game, Id::new("batch-a")));
    assert_eq!(focused(&mut rt, ids), (true, false));
}
