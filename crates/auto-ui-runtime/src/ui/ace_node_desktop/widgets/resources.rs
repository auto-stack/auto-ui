use std::path::{Component, Path, PathBuf};

use ace_core::source::age_component::{ComponentViewElement, ComponentViewItem};

use super::{literal, Renderer, WidgetView};
use crate::ui::style::{SizeValue, StyleClass};
use crate::ui::{Style, View};

pub(in crate::ui) fn public_assets(source: &Path) -> Option<PathBuf> {
    let root = source
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "front"))?
        .join("public")
        .canonicalize()
        .ok()?;
    root.is_dir().then_some(root)
}

fn widget_key(session: i32, instance: usize, element: &ComponentViewElement) -> String {
    format!("ace-page/{session}/{instance}/{}", element.span.start)
}

pub(super) fn release_editors(session: i32, instances: usize, items: &[ComponentViewItem]) {
    for item in items {
        let ComponentViewItem::Element(element) = item else {
            continue;
        };
        match element.tag.as_str() {
            #[cfg(feature = "autodown")]
            "autodown_editor" | "autodown-editor" => {
                for instance in 0..instances {
                    crate::ui::autodown_editor::autodown_editor_dispose(&widget_key(
                        session, instance, element,
                    ));
                }
            }
            #[cfg(feature = "code-editor")]
            "code_editor" | "code-editor" => {
                for instance in 0..instances {
                    crate::ui::code_editor::code_editor_dispose(&widget_key(
                        session, instance, element,
                    ));
                }
            }
            _ => {}
        }
        release_editors(session, instances, &element.items);
    }
}

impl Renderer<'_> {
    fn image_source(&self, rendered: &str) -> String {
        if let Some((root, relative)) = self
            .state
            .public_assets
            .as_deref()
            .zip(rendered.strip_prefix('/'))
        {
            let relative = Path::new(relative);
            if relative
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
            {
                if let Ok(file) = root.join(relative).canonicalize() {
                    if file.starts_with(root) && file.is_file() {
                        return file.to_string_lossy().into_owned();
                    }
                }
            }
        }
        rendered.to_owned()
    }

    pub(super) fn native_icon(
        &self,
        element: &ComponentViewElement,
        mut style: Style,
    ) -> WidgetView {
        let size = self.number(element, "size", 20.0).max(1.0);
        if !style
            .classes
            .iter()
            .any(|class| matches!(class, StyleClass::Width(_)))
        {
            style
                .classes
                .push(StyleClass::Width(SizeValue::Pixels(size)));
        }
        if !style
            .classes
            .iter()
            .any(|class| matches!(class, StyleClass::Height(_)))
        {
            style
                .classes
                .push(StyleClass::Height(SizeValue::Pixels(size)));
        }
        let name = literal(element, "name").unwrap_or("circle");
        let src = if name.contains(':') {
            name.to_owned()
        } else {
            format!("lucide:{name}")
        };
        View::Image {
            src,
            style: Some(style),
        }
    }

    pub(super) fn native_image(&self, element: &ComponentViewElement, style: Style) -> WidgetView {
        let source = self.image_source(literal(element, "src").unwrap_or_default());
        let fit = literal(element, "fit").unwrap_or("contain");
        if matches!(
            element.tag.as_str(),
            "imagesurface" | "image-surface" | "image_surface"
        ) {
            let mut surface = View::image_surface(source);
            if let View::ImageSurface {
                alt,
                width,
                height,
                fit: image_fit,
                style: actual,
                ..
            } = &mut surface
            {
                *alt = literal(element, "alt").unwrap_or_default().to_owned();
                *width = self.number(element, "width", 0.0) as u32;
                *height = self.number(element, "height", 0.0) as u32;
                *image_fit = fit.to_owned();
                *actual = Some(style);
            }
            return surface;
        }

        let class = match fit {
            "cover" => "object-cover",
            "fill" => "object-fill",
            "none" => "object-none",
            "scale-down" => "object-scale-down",
            _ => "object-contain",
        };
        let mut style = style;
        style
            .classes
            .extend(Style::parse(class).unwrap_or_default().classes);
        View::Image {
            src: source,
            style: Some(style),
        }
    }

    pub(super) fn native_editor(
        &mut self,
        element: &ComponentViewElement,
        mut style: Style,
    ) -> WidgetView {
        let key = widget_key(self.session, self.instance, element);
        if style.classes.is_empty() {
            style = Style::parse("w-96 h-24").unwrap_or_default();
        }
        let value = literal(element, "content")
            .or_else(|| literal(element, "value"))
            .map(str::to_owned)
            .unwrap_or_else(|| self.label(element));
        if matches!(element.tag.as_str(), "code_editor" | "code-editor") {
            return View::code_editor(key)
                .value(value)
                .lang(literal(element, "lang").unwrap_or("none"))
                .with_style(style)
                .build();
        }
        View::AutodownEditor {
            key,
            value,
            is_final: true,
            on_change: None,
            on_focus: None,
            on_link: None,
            placeholder: literal(element, "placeholder").map(str::to_owned),
            style: Some(style),
        }
    }
}
