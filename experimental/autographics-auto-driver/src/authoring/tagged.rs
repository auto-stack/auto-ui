//! Driver-local layout witnesses. No event, state, focus or rendering substitution.
use agp::host::iced_surface::SurfaceElement;
use iced_wgpu::core::{
    self as c,
    widget::{tree, Operation, Tree},
    Widget,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
pub type Bounds = Arc<Mutex<HashMap<String, c::Rectangle>>>;
pub fn tag<'a>(
    element: impl Into<SurfaceElement<'a, String>>,
    id: String,
    bounds: Bounds,
) -> SurfaceElement<'a, String> {
    c::Element::new(Tagged {
        element: element.into(),
        id,
        bounds,
        scroll: None,
    })
}
/// Observe the stock Scrollable's actual clamped translation, including layout shrink.
/// Run a read-only operation first, then forward the original operation unchanged.
pub fn observe_scroll<'a>(
    element: impl Into<SurfaceElement<'a, String>>,
    offset: Arc<Mutex<f32>>,
    bounds: Bounds,
) -> SurfaceElement<'a, String> {
    c::Element::new(Tagged {
        element: element.into(),
        id: "panel-scroll-frame".into(),
        bounds,
        scroll: Some(offset),
    })
}
struct Tagged<'a> {
    element: SurfaceElement<'a, String>,
    id: String,
    bounds: Bounds,
    scroll: Option<Arc<Mutex<f32>>>,
}
impl Widget<String, c::Theme, iced_wgpu::Renderer> for Tagged<'_> {
    fn size(&self) -> c::Size<c::Length> {
        self.element.as_widget().size()
    }
    fn size_hint(&self) -> c::Size<c::Length> {
        self.element.as_widget().size_hint()
    }
    fn tag(&self) -> tree::Tag {
        self.element.as_widget().tag()
    }
    fn state(&self) -> tree::State {
        self.element.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.element.as_widget().children()
    }
    fn diff(&self, t: &mut Tree) {
        self.element.as_widget().diff(t)
    }
    fn layout(
        &mut self,
        t: &mut Tree,
        r: &iced_wgpu::Renderer,
        l: &c::layout::Limits,
    ) -> c::layout::Node {
        self.element.as_widget_mut().layout(t, r, l)
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut iced_wgpu::Renderer,
        theme: &c::Theme,
        s: &c::renderer::Style,
        l: c::Layout<'_>,
        cur: c::mouse::Cursor,
        v: &c::Rectangle,
    ) {
        self.element.as_widget().draw(t, r, theme, s, l, cur, v)
    }
    fn operate(
        &mut self,
        t: &mut Tree,
        l: c::Layout<'_>,
        r: &iced_wgpu::Renderer,
        op: &mut dyn Operation,
    ) {
        if let Some(offset) = &self.scroll {
            struct Probe(Arc<Mutex<f32>>);
            impl Operation for Probe {
                fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                    operate(self);
                }
                fn scrollable(
                    &mut self,
                    _id: Option<&c::widget::Id>,
                    _bounds: c::Rectangle,
                    _content: c::Rectangle,
                    translation: c::Vector,
                    _state: &mut dyn c::widget::operation::Scrollable,
                ) {
                    *self.0.lock().unwrap() = translation.y;
                }
            }
            self.element
                .as_widget_mut()
                .operate(t, l, r, &mut Probe(offset.clone()));
        }
        self.bounds
            .lock()
            .unwrap()
            .insert(self.id.clone(), l.bounds());
        self.element.as_widget_mut().operate(t, l, r, op)
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &c::Event,
        l: c::Layout<'_>,
        cur: c::mouse::Cursor,
        r: &iced_wgpu::Renderer,
        cb: &mut dyn c::Clipboard,
        sh: &mut c::Shell<'_, String>,
        v: &c::Rectangle,
    ) {
        self.element
            .as_widget_mut()
            .update(t, e, l, cur, r, cb, sh, v)
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: c::Layout<'_>,
        cur: c::mouse::Cursor,
        v: &c::Rectangle,
        r: &iced_wgpu::Renderer,
    ) -> c::mouse::Interaction {
        self.element.as_widget().mouse_interaction(t, l, cur, v, r)
    }
    fn overlay<'a>(
        &'a mut self,
        t: &'a mut Tree,
        l: c::Layout<'a>,
        r: &iced_wgpu::Renderer,
        v: &c::Rectangle,
        translation: c::Vector,
    ) -> Option<c::overlay::Element<'a, String, c::Theme, iced_wgpu::Renderer>> {
        self.element
            .as_widget_mut()
            .overlay(t, l, r, v, translation)
    }
}
