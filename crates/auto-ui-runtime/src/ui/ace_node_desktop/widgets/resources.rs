use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use ace_core::source::age_component::{ComponentViewElement, ComponentViewItem};
use auto_media_core::image_pipeline::{
    global_media_registry, media_uri, queue_media_path, MediaAssetTicket, MediaPriority,
};

use super::{literal, Renderer, WidgetView};
use crate::ui::style::{SizeValue, StyleClass};
use crate::ui::{Style, View};

pub(in crate::ui) fn public_assets(source: &Path) -> Option<PathBuf> {
    let front = source
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "front"))?
        .canonicalize()
        .ok()?;
    let root = front.join("public").canonicalize().ok()?;
    (root.starts_with(front) && root.is_dir()).then_some(root)
}

#[derive(Debug)]
pub(super) struct PublicImages {
    root: Arc<PathBuf>,
    surfaces: Mutex<BTreeMap<PathBuf, MediaAssetTicket>>,
}

impl PublicImages {
    pub(super) fn new(root: Arc<PathBuf>) -> Self {
        Self {
            root,
            surfaces: Mutex::default(),
        }
    }

    fn resolve(&self, source: &str) -> Option<PathBuf> {
        let path = Path::new(source.strip_prefix('/').unwrap_or(source));
        if !path.components().all(|part| matches!(part, Component::Normal(_))) {
            return None;
        }
        let file = self.root.join(path).canonicalize().ok()?;
        (file.starts_with(self.root.as_path()) && file.is_file()).then_some(file)
    }

    fn surface(&self, file: PathBuf) -> String {
        let mut surfaces = self.surfaces.lock().unwrap();
        let ticket = *surfaces
            .entry(file.clone())
            .or_insert_with(|| queue_media_path(file, MediaPriority::Current));
        media_uri(ticket)
    }
}

impl Drop for PublicImages {
    fn drop(&mut self) {
        for ticket in self.surfaces.get_mut().unwrap().values() {
            global_media_registry().release(ticket.id);
        }
    }
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
    fn image_source(&self, rendered: &str, surface: bool) -> String {
        if ["http://", "https://", "data:", "builtin:", "/api/__auto/media/"]
            .iter()
            .any(|&prefix| rendered.starts_with(prefix))
        {
            return rendered.to_owned();
        }
        let Some(assets) = self.state.public_assets.as_deref() else {
            return String::new();
        };
        let Some(file) = assets.resolve(rendered) else {
            return String::new();
        };
        if surface {
            assets.surface(file)
        } else {
            file.to_string_lossy().into_owned()
        }
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
        let name = name.strip_prefix("lucide:").unwrap_or(name);
        View::Image {
            src: format!("lucide:{name}"),
            style: Some(style),
        }
    }

    pub(super) fn native_image(&self, element: &ComponentViewElement, style: Style) -> WidgetView {
        let surface = matches!(
            element.tag.as_str(),
            "imagesurface" | "image-surface" | "image_surface"
        );
        let source = self.image_source(literal(element, "src").unwrap_or_default(), surface);
        let fit = literal(element, "fit").unwrap_or("contain");
        if surface {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_image_sources_stay_inside_the_front_public_directory() {
        let fixture = tempfile::tempdir().unwrap();
        let public = fixture.path().join("front/public");
        std::fs::create_dir_all(&public).unwrap();
        std::fs::write(public.join("icon.png"), b"public fixture").unwrap();
        let private = fixture.path().join("front/private.png");
        std::fs::write(&private, b"outside public fixture").unwrap();
        let root = public_assets(&fixture.path().join("front/pages/gallery.at")).unwrap();
        let state = super::super::State::new(BTreeMap::new()).with_public_assets(Some(Arc::new(root)));
        let mut renderer = Renderer {
            session: 1,
            instance: 0,
            texts: [].iter(),
            values: &[],
            state: &state,
            targets: Vec::new(),
            panel_close: None,
        };
        assert_eq!(
            renderer.image_source("/icon.png", false),
            public.join("icon.png").canonicalize().unwrap().to_string_lossy()
        );
        assert_eq!(renderer.image_source("../private.png", false), "");
        assert_eq!(renderer.image_source("/../private.png", false), "");
        assert_eq!(renderer.image_source(&private.to_string_lossy(), false), "");
        assert_eq!(renderer.image_source("/missing.png", false), "");
        assert_eq!(renderer.image_source("/../private.png", true), "");
        assert_eq!(
            renderer.image_source("https://example.test/icon.png", false),
            "https://example.test/icon.png"
        );
        let no_public = super::super::State::new(BTreeMap::new());
        renderer.state = &no_public;
        assert_eq!(renderer.image_source(&private.to_string_lossy(), false), "");

        let file_name = serde_json::to_string(&private.to_string_lossy()).unwrap();
        let source = format!(r#"widget Preview {{
    view {{
        icon (name: {file_name}) {{}}
        icon (name: "lucide:circle") {{}}
    }}
}}"#);
        let nodes = ace_core::source::age_component::project_component_candidates(&source).unwrap();
        let tree = &nodes[0].view.tree;
        let View::Column { children, .. } = super::super::view(1, 0, tree, &[], &[], &state) else {
            panic!("expected a frame for the icons");
        };
        assert!(matches!(
            children.as_slice(),
            [View::Image { src: invalid, .. }, View::Image { src: valid, .. }]
                if invalid == &format!("lucide:{}", private.display()) && valid == "lucide:circle"
        ));
    }
}
