use super::{native_style, NodeDesktopMsg, Renderer, WidgetView};
use crate::ui::ace_widget_props::{expression, self_field, style};
use crate::ui::aura_view_builder::parse_coords_extent;
use crate::ui::view::PointerMoveHandler;
use crate::ui::View;
use ace_core::source::age_component::{
    ComponentEntryValue, ComponentViewElement, ComponentViewItem,
};

fn message_for<'a>(el: &'a ComponentViewElement, name: &str) -> Result<Option<&'a str>, String> {
    for item in el.items.iter().rev() {
        if let ComponentViewItem::Entry(entry) = item {
            if entry.key == name {
                if let ComponentEntryValue::Expr(expr) = &entry.value {
                    if let Some(message) = self_field(expr) {
                        return Ok(Some(message));
                    }
                }
                return Err(format!(
                    "ace.desktop.pointer: `{name}` needs a direct message binding"
                ));
            }
        }
    }
    Ok(None)
}

impl Renderer<'_> {
    pub(super) fn mouse_area(&mut self, el: &ComponentViewElement) -> WidgetView {
        let content = View::col().children(self.items(&el.items)).build();
        self.pointer_view(el, content).unwrap_or_else(View::text)
    }

    fn pointer_view(
        &self,
        el: &ComponentViewElement,
        content: WidgetView,
    ) -> Result<WidgetView, String> {
        let (session, instance) = (self.session, self.instance);
        let unit = |message: &str| NodeDesktopMsg::Pointer {
            session,
            instance,
            message: message.to_owned(),
            coordinates: None,
        };
        let area = expression(el, "coords")
            .map(|expr| {
                self.display(expr)
                    .as_deref()
                    .and_then(parse_coords_extent)
                    .filter(|(w, h)| w.is_finite() && h.is_finite())
                    .ok_or_else(|| {
                        "ace.desktop.pointer: coords must have a finite WxH extent".to_owned()
                    })
            })
            .transpose()?;
        let on_move = message_for(el, "onmousemove")?.map(|message| {
            let message = message.to_owned();
            PointerMoveHandler::new(move |x, y| NodeDesktopMsg::Pointer {
                session,
                instance,
                message: message.clone(),
                coordinates: Some((x, y)),
            })
        });
        Ok(View::MouseArea {
            content: Box::new(content),
            on_enter: message_for(el, "onmouseenter")?.map(unit),
            on_exit: message_for(el, "onmouseleave")?.map(unit),
            on_double_click: message_for(el, "ondblclick")?.map(unit),
            on_click: message_for(el, "onmousedown")?
                .or(message_for(el, "onclick")?)
                .map(unit),
            on_context_menu: message_for(el, "oncontextmenu")?.map(unit),
            on_release: message_for(el, "onmouseup")?.map(unit),
            on_move,
            logical_extent: area,
            style: Some(native_style(&style(el))),
        })
    }
}
