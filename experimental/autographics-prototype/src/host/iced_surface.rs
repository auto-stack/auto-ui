//! Offscreen iced content adapter (PLAN-830 T-02+).
//!
//! Drives a REAL iced UI (stock widgets: `button`, `text_input`, …) without
//! an `iced::application`: the whole lifecycle runs on public APIs —
//! `iced_wgpu::Engine` (headless shell) → `iced_wgpu::Renderer` →
//! `iced_runtime::user_interface::UserInterface::{build, update, draw}` →
//! `Renderer::present` into an OFFSCREEN texture (the sampled usage
//! contract of `OffscreenTarget::new_sampled`). No window, no compositor,
//! no production UI code.
//!
//! The device/queue are the host's single `GpuDevice` (borrowed clones);
//! no second device is ever created here.

use crate::renderer::error::RendererError;
use crate::renderer::target::OffscreenTarget;
use crate::renderer::GpuDevice;
use iced_runtime::user_interface::{self, Cache};
use iced_wgpu::core::clipboard::Null;
use iced_wgpu::core::mouse;
use iced_wgpu::core::renderer;
use iced_wgpu::core::{Color, Event, Point, Size, Theme};
use iced_wgpu::graphics::Viewport;

/// Element pinned to the REAL wgpu backend renderer (the `iced` facade's
/// default `Renderer` is the wgpu+tiny-skia FALLBACK type; the offscreen
/// present path needs the actual `iced_wgpu::Renderer`).
pub type SurfaceElement<'a, M> =
    iced_wgpu::core::Element<'a, M, iced_wgpu::core::Theme, iced_wgpu::Renderer>;

/// An offscreen iced UI's application logic: pure state + view/update.
pub trait SurfaceUi {
    type Message: Clone;

    /// The current view of the UI (real stock widgets).
    fn view(&self) -> SurfaceElement<'_, Self::Message>;

    /// Apply one message (widget interaction result).
    fn update(&mut self, message: Self::Message);

    /// Clear color for the surface texture (opaque panel background).
    fn background(&self) -> Color;

    /// Downcast support so probes/tests can read concrete app state through
    /// the trait object (the runtime itself stays generic).
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// What one frame cycle observed.
#[derive(Debug, Clone)]
pub struct SurfaceFrame {
    /// Messages produced by the widgets this frame.
    pub messages: Vec<()>,
}

/// Observable content change of one iced content pass (PLAN-830 review r2
/// F01/F03). Consumer surfaces must re-sample a child whose OUTPUT changed in
/// this frame; the runtime cannot see inside the widget tree, so the adapter
/// reports the facts iced itself exposes.
#[derive(Debug, Clone, Default)]
pub struct SurfaceChange {
    /// The pass produced output that consumers must sample this frame.
    pub output_changed: bool,
    /// Messages published by the real widgets and applied to the app state.
    pub messages: u64,
    /// Events the widget tree CAPTURED (a captured widget may redraw).
    pub captured: u64,
    /// The UI asked for a redraw (hover/focus/caret/animation).
    pub redraw_requested: bool,
    /// The layout was invalidated (widget sizes/positions may differ).
    pub layout_changed: bool,
    /// The REAL input-method caret rectangle in surface-local logical px,
    /// taken from `InputMethod::Enabled.cursor` that the focused
    /// `text_input` produced from its actual layout/cursor/selection/scroll
    /// (review r2 F03 — a fixed local point is not a caret).
    pub caret: Option<iced_wgpu::core::Rectangle>,
}

impl SurfaceChange {
    /// The caret anchor in surface-local logical px (top-left of the real
    /// caret rect, rounded for the OS candidate window).
    pub fn caret_anchor(&self) -> Option<[f32; 2]> {
        self.caret.map(|r| [r.x, r.y])
    }
}

/// One offscreen iced content instance on the shared device.
pub struct IcedSurface {
    renderer: iced_wgpu::Renderer,
    theme: Theme,
    style: renderer::Style,
    cache: Cache,
    /// Owned target for the standalone probe path (`new`); `None` when the
    /// surface is driven by `SurfaceRuntime`, which supplies its own slot
    /// target per frame (`frame_into`) — one texture per surface, never two.
    target: Option<OffscreenTarget>,
    logical: Size,
    scale: f32,
    /// GPU→CPU readbacks performed for acceptance captures (never steady
    /// state; the frame path below contains none).
    pub readbacks: u64,
    /// Host device that created this adapter (composition refuses foreign
    /// device content — review r1 F09).
    pub device_ordinal: u64,
    /// Facts observed by the LAST content pass (change propagation + caret).
    pub last_change: SurfaceChange,
    /// May this surface's widgets hold keyboard focus? The runtime sets this
    /// from the ROUTER's current target before each pass, so an iced surface
    /// whose routed clicks were batched away cannot keep a second real edit
    /// target (review r3 F-830-R3-01).
    pub focus_allowed: bool,
}

impl IcedSurface {
    pub fn new(gpu: &GpuDevice, logical: Size, scale: f32) -> Result<Self, RendererError> {
        let engine = iced_wgpu::Engine::new(
            gpu.adapter(),
            gpu.device().clone(),
            gpu.queue().clone(),
            OffscreenTarget::FORMAT,
            None,
            iced_wgpu::graphics::Shell::headless(),
        );
        let renderer = iced_wgpu::Renderer::new(
            engine,
            iced_wgpu::core::Font::DEFAULT,
            iced_wgpu::core::Pixels(16.0),
        );
        let physical = Size::new(
            (logical.width * scale).round().max(1.0) as u32,
            (logical.height * scale).round().max(1.0) as u32,
        );
        let target = OffscreenTarget::new_sampled(gpu, physical.width, physical.height)?;
        Ok(IcedSurface {
            renderer,
            theme: Theme::Light,
            style: renderer::Style::default(),
            cache: Cache::new(),
            target: Some(target),
            logical,
            scale,
            readbacks: 0,
            device_ordinal: gpu.token().ordinal,
            last_change: SurfaceChange::default(),
            focus_allowed: true,
        })
    }

    /// Construct WITHOUT an internal target: every frame must supply one via
    /// `frame_into` (the runtime path). Avoids a duplicate offscreen texture.
    pub fn new_borrowed(gpu: &GpuDevice, logical: Size, scale: f32) -> Self {
        let engine = iced_wgpu::Engine::new(
            gpu.adapter(),
            gpu.device().clone(),
            gpu.queue().clone(),
            OffscreenTarget::FORMAT,
            None,
            iced_wgpu::graphics::Shell::headless(),
        );
        let renderer = iced_wgpu::Renderer::new(
            engine,
            iced_wgpu::core::Font::DEFAULT,
            iced_wgpu::core::Pixels(16.0),
        );
        IcedSurface {
            renderer,
            theme: Theme::Light,
            style: renderer::Style::default(),
            cache: Cache::new(),
            target: None,
            logical,
            scale,
            readbacks: 0,
            device_ordinal: gpu.token().ordinal,
            last_change: SurfaceChange::default(),
            focus_allowed: true,
        }
    }

    pub fn logical_size(&self) -> Size {
        self.logical
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale
    }

    pub fn physical_size(&self) -> (u32, u32) {
        match &self.target {
            Some(t) => (t.width, t.height),
            None => (
                (self.logical.width * self.scale).round().max(1.0) as u32,
                (self.logical.height * self.scale).round().max(1.0) as u32,
            ),
        }
    }

    /// Resize to a new logical size (recreates only the texture; the iced
    /// renderer/engine survive — local change, no global rebuild).
    pub fn resize(&mut self, gpu: &GpuDevice, logical: Size) -> Result<(), RendererError> {
        let physical = Size::new(
            (logical.width * self.scale).round().max(1.0) as u32,
            (logical.height * self.scale).round().max(1.0) as u32,
        );
        // Recreate only when this surface owns a target; runtime-driven
        // surfaces are resized through the runtime's slot target.
        if self.target.is_some() {
            self.target = Some(OffscreenTarget::new_sampled(
                gpu,
                physical.width,
                physical.height,
            )?);
        }
        self.logical = logical;
        Ok(())
    }

    /// Run one update+draw cycle: feed events to the REAL widgets, apply
    /// the produced messages, then lay out and DRAW the current state into
    /// the offscreen texture. Returns the produced messages (already
    /// applied) so the host can record evidence.
    pub fn frame<M: Clone>(
        &mut self,
        ui: &mut dyn SurfaceUi<Message = M>,
        events: &[Event],
        cursor: mouse::Cursor,
    ) -> Vec<M> {
        let target = self
            .target
            .take()
            .expect("owned target; use frame_into for borrowed targets");
        let out = self.frame_into(ui, events, cursor, &target);
        self.target = Some(target);
        out
    }

    /// Frame that renders into a CALLER-OWNED target (the runtime's slot
    /// target — the single texture that gets composited and captured).
    ///
    /// A real iced application feeds `RedrawRequested` on every frame; that
    /// event is what makes a focused `text_input` report its ACTUAL input
    /// method caret (`InputMethod::Enabled.cursor`). Without it the adapter
    /// can only see event-driven state (review r2 F03: the old adapter threw
    /// the update `State` away and the host used a fixed point).
    pub fn frame_into<M: Clone>(
        &mut self,
        ui: &mut dyn SurfaceUi<Message = M>,
        events: &[Event],
        cursor: mouse::Cursor,
        target: &OffscreenTarget,
    ) -> Vec<M> {
        let bounds = self.logical;
        let now = iced_wgpu::core::time::Instant::now();
        // 1) update pass with the CURRENT view (real widget event handling).
        let mut messages: Vec<M> = Vec::new();
        let (captured, state) = {
            let root = ui.view();
            let mut interface = user_interface::UserInterface::build(
                root,
                bounds,
                std::mem::take(&mut self.cache),
                &mut self.renderer,
            );
            let mut clipboard = Null;
            let mut with_redraw: Vec<Event> = Vec::with_capacity(events.len() + 1);
            with_redraw.extend_from_slice(events);
            with_redraw.push(Event::Window(
                iced_wgpu::core::window::Event::RedrawRequested(now),
            ));
            let (st, statuses) = interface.update(
                &with_redraw,
                cursor,
                &mut self.renderer,
                &mut clipboard,
                &mut messages,
            );
            let captured = statuses
                .iter()
                .filter(|s| matches!(s, iced_wgpu::core::event::Status::Captured))
                .count() as u64;
            self.cache = interface.into_cache();
            (captured, st)
        };
        // 2) apply messages to the app state.
        for m in &messages {
            ui.update(m.clone());
        }
        // 2b) SINGLE REAL EDIT TARGET: when the router's CURRENT target is
        //     not this surface, a widget that just took focus from a batched
        //     click must give it up BEFORE the draw — the presented texture
        //     and the widget cache then agree with the router (review r3
        //     F-830-R3-01). The events themselves are still delivered, so an
        //     earlier click's action is not silently dropped.
        let focus_released = !self.focus_allowed && self.unfocus(&*ui);
        // 3) draw the UPDATED state (iced runtime semantics: rebuild after
        //    messages, then draw — the drawn frame never lags a message).
        {
            let root = ui.view();
            let mut interface = user_interface::UserInterface::build(
                root,
                bounds,
                std::mem::take(&mut self.cache),
                &mut self.renderer,
            );
            interface.draw(&mut self.renderer, &self.theme, &self.style, cursor);
            self.cache = interface.into_cache();
        }
        // 4) present into the target texture (public Renderer::present with
        //    an arbitrary target view — the reverse-offscreen seam).
        let viewport =
            Viewport::with_physical_size(Size::new(target.width, target.height), self.scale);
        self.renderer.present(
            Some(ui.background()),
            OffscreenTarget::FORMAT,
            target.view(),
            &viewport,
        );
        // 5) record what the REAL widget tree reported.
        let mut change = SurfaceChange {
            messages: messages.len() as u64,
            captured,
            ..Default::default()
        };
        match state {
            user_interface::State::Outdated => {
                change.output_changed = true;
                change.layout_changed = true;
            }
            user_interface::State::Updated {
                redraw_request,
                input_method,
                has_layout_changed,
                ..
            } => {
                change.layout_changed = has_layout_changed;
                change.redraw_requested = match redraw_request {
                    iced_wgpu::core::window::RedrawRequest::Wait => false,
                    iced_wgpu::core::window::RedrawRequest::NextFrame => true,
                    iced_wgpu::core::window::RedrawRequest::At(at) => at <= now,
                };
                if let iced_wgpu::core::InputMethod::Enabled { cursor, .. } = input_method {
                    change.caret = Some(cursor);
                }
                // A focused text input keeps reporting a caret; its blink is a
                // real output change consumers must sample every frame.
                change.output_changed = change.messages > 0
                    || change.captured > 0
                    || change.layout_changed
                    || change.redraw_requested
                    || change.caret.is_some();
            }
        }
        if focus_released {
            // The control lost focus inside this pass: no caret may be
            // reported for it and consumers must see the unfocused texture.
            change.caret = None;
            change.output_changed = true;
        }
        self.last_change = change;
        messages
    }

    /// The REAL caret anchor of the focused widget in local logical px.
    pub fn caret_anchor(&self) -> Option<[f32; 2]> {
        self.last_change.caret_anchor()
    }

    /// The REAL caret rectangle of the focused widget in local logical px.
    pub fn caret_rect(&self) -> Option<iced_wgpu::core::Rectangle> {
        self.last_change.caret
    }

    /// Does any widget in this tree hold keyboard focus? Used to enforce the
    /// single-keyboard-target rule across surfaces (review r2 F02: iced keeps
    /// its own focus cache per surface).
    pub fn has_focused_widget<M: Clone>(&mut self, ui: &dyn SurfaceUi<Message = M>) -> bool {
        struct AnyFocused {
            found: bool,
        }
        impl iced_wgpu::core::widget::Operation<()> for AnyFocused {
            fn traverse(
                &mut self,
                operate: &mut dyn FnMut(&mut dyn iced_wgpu::core::widget::Operation<()>),
            ) {
                operate(self)
            }
            fn focusable(
                &mut self,
                _id: Option<&iced_wgpu::core::widget::Id>,
                _bounds: iced_wgpu::core::Rectangle,
                state: &mut dyn iced_wgpu::core::widget::operation::Focusable,
            ) {
                if state.is_focused() {
                    self.found = true;
                }
            }
        }
        let mut op = AnyFocused { found: false };
        let root = ui.view();
        let mut interface = user_interface::UserInterface::build(
            root,
            self.logical,
            std::mem::take(&mut self.cache),
            &mut self.renderer,
        );
        interface.operate(&mut self.renderer, &mut op);
        self.cache = interface.into_cache();
        op.found
    }

    /// Real widget bounds in local logical px for the widget with `id`
    /// (public focusable traversal — used to click/focus the ACTUAL control
    /// instead of a hard-coded point).
    pub fn widget_bounds<M: Clone>(
        &mut self,
        ui: &dyn SurfaceUi<Message = M>,
        id: iced_wgpu::core::widget::Id,
    ) -> Option<iced_wgpu::core::Rectangle> {
        struct BoundsProbe {
            want: iced_wgpu::core::widget::Id,
            bounds: Option<iced_wgpu::core::Rectangle>,
        }
        impl iced_wgpu::core::widget::Operation<()> for BoundsProbe {
            fn traverse(
                &mut self,
                operate: &mut dyn FnMut(&mut dyn iced_wgpu::core::widget::Operation<()>),
            ) {
                operate(self)
            }
            fn focusable(
                &mut self,
                id: Option<&iced_wgpu::core::widget::Id>,
                bounds: iced_wgpu::core::Rectangle,
                _state: &mut dyn iced_wgpu::core::widget::operation::Focusable,
            ) {
                if id == Some(&self.want) {
                    self.bounds = Some(bounds);
                }
            }
        }
        let mut op = BoundsProbe {
            want: id,
            bounds: None,
        };
        let root = ui.view();
        let mut interface = user_interface::UserInterface::build(
            root,
            self.logical,
            std::mem::take(&mut self.cache),
            &mut self.renderer,
        );
        interface.operate(&mut self.renderer, &mut op);
        self.cache = interface.into_cache();
        op.bounds
    }

    /// The rendered content view (owned-target path only). Valid after
    /// `frame`; the texture carries TEXTURE_BINDING usage.
    pub fn content(&self) -> &wgpu::TextureView {
        self.target
            .as_ref()
            .expect("owned target (new); runtime surfaces use their slot target")
            .view()
    }

    pub fn target(&self) -> &OffscreenTarget {
        self.target
            .as_ref()
            .expect("owned target (new); runtime surfaces use their slot target")
    }

    /// Map a point in this surface's LOCAL logical coordinates to an iced
    /// cursor position (identity today; exists so portal transforms have a
    /// single entry point).
    pub fn local_cursor(&self, local: Point) -> mouse::Cursor {
        mouse::Cursor::Available(local)
    }

    /// Focus a `text_input` by its widget id through the PUBLIC focus
    /// operation (used before feeding IME events so commits land in the
    /// control, not in the void).
    pub fn focus_text_input<M: Clone>(
        &mut self,
        ui: &dyn SurfaceUi<Message = M>,
        id: iced_wgpu::core::widget::Id,
    ) {
        // Public `focusable::focus(id)` operation (iced_core); the widget was
        // given this Id via `text_input(..).id(..)`.
        let mut op = iced_wgpu::core::widget::operation::focusable::focus::<()>(id);
        self.operate(ui, &mut op);
    }

    /// Release keyboard focus for the whole tree (window blur / destroy /
    /// unmount / router target moved). Returns whether an edit state was
    /// actually released — callers use it to avoid claiming an output change
    /// (and re-compositing consumers) on every clean frame.
    pub fn unfocus<M: Clone>(&mut self, ui: &dyn SurfaceUi<Message = M>) -> bool {
        let had_focus = self.last_change.caret.is_some() || self.has_focused_widget(ui);
        if !had_focus {
            return false;
        }
        // Close any in-progress composition FIRST: a focus migration must not
        // leave a preedit attached to the old control (review r2 F02).
        {
            let mut messages: Vec<M> = Vec::new();
            let root = ui.view();
            let mut interface = user_interface::UserInterface::build(
                root,
                self.logical,
                std::mem::take(&mut self.cache),
                &mut self.renderer,
            );
            let mut clipboard = Null;
            let _ = interface.update(
                &[Event::InputMethod(
                    iced_wgpu::core::input_method::Event::Closed,
                )],
                mouse::Cursor::Unavailable,
                &mut self.renderer,
                &mut clipboard,
                &mut messages,
            );
            self.cache = interface.into_cache();
        }
        let mut op = iced_wgpu::core::widget::operation::focusable::unfocus::<()>();
        self.operate(ui, &mut op);
        // No stale caret/IME anchor survives an unfocus: the next frame must
        // not reuse the last known edit point.
        self.last_change.caret = None;
        self.last_change.output_changed = true;
        true
    }

    /// Is the widget with `id` focused? Implemented as a probe Operation
    /// (the public `UserInterface::operate` erases the outcome type).
    pub fn is_focused<M: Clone>(
        &mut self,
        ui: &dyn SurfaceUi<Message = M>,
        id: iced_wgpu::core::widget::Id,
    ) -> bool {
        struct FocusProbe {
            want: iced_wgpu::core::widget::Id,
            found: bool,
        }
        impl iced_wgpu::core::widget::Operation<()> for FocusProbe {
            fn traverse(
                &mut self,
                operate: &mut dyn FnMut(&mut dyn iced_wgpu::core::widget::Operation<()>),
            ) {
                operate(self)
            }
            fn focusable(
                &mut self,
                id: Option<&iced_wgpu::core::widget::Id>,
                _bounds: iced_wgpu::core::Rectangle,
                state: &mut dyn iced_wgpu::core::widget::operation::Focusable,
            ) {
                if id == Some(&self.want) && state.is_focused() {
                    self.found = true;
                }
            }
        }
        let mut op = FocusProbe {
            want: id,
            found: false,
        };
        let root = ui.view();
        let mut interface = user_interface::UserInterface::build(
            root,
            self.logical,
            std::mem::take(&mut self.cache),
            &mut self.renderer,
        );
        interface.operate(&mut self.renderer, &mut op);
        self.cache = interface.into_cache();
        op.found
    }

    /// Apply a `widget::Operation` to the current tree (public traversal —
    /// used to harvest EXACT widget bounds instead of guessing layout).
    pub fn operate<M: Clone>(
        &mut self,
        ui: &dyn SurfaceUi<Message = M>,
        op: &mut dyn iced_wgpu::core::widget::Operation<()>,
    ) {
        let root = ui.view();
        let mut interface = user_interface::UserInterface::build(
            root,
            self.logical,
            std::mem::take(&mut self.cache),
            &mut self.renderer,
        );
        interface.operate(&mut self.renderer, op);
        self.cache = interface.into_cache();
    }
}

/// Translate a winit logical position to this surface's local coordinates,
/// given the portal-local point (the caller performs the window→portal
/// chain; see `core::composition::input` for the full contract).
pub fn cursor_at(local: Point) -> mouse::Cursor {
    mouse::Cursor::Available(local)
}
