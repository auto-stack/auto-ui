use super::{native_style, Action, Control, NodeDesktopMsg, Renderer, Target, WidgetView};
use crate::ui::ace_widget_props::{element_children, expression, literal, style};
use crate::ui::style::{theme, Breakpoint, SizeValue, StyleClass};
use crate::ui::view::PointerMoveHandler;
use crate::ui::View;
use ace_core::source::age_component::ComponentViewElement;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct Navigation {
    key: usize,
    slides: usize,
    visible: usize,
    vertical: bool,
    looping: bool,
}

#[derive(Clone, Debug)]
pub enum CarouselAction {
    Move {
        navigation: Navigation,
        forward: bool,
    },
    Press(usize),
    Sample {
        key: usize,
        offset: f32,
    },
    Release(Navigation),
    Cancel(usize),
}

impl Navigation {
    fn last(self) -> usize {
        if self.looping && self.visible < self.slides {
            self.slides - 1
        } else {
            self.slides.saturating_sub(self.visible)
        }
    }

    fn destination(self, position: usize, forward: bool) -> usize {
        let last = self.last();
        match (forward, position) {
            (true, p) if p < last => p + 1,
            (false, p) if p > 0 => p - 1,
            (false, _) if self.looping => last,
            (true, _) if self.looping => 0,
            _ => position,
        }
    }

    pub(super) fn next(self) -> CarouselAction {
        CarouselAction::Move {
            navigation: self,
            forward: true,
        }
    }

    pub(super) fn previous(self) -> CarouselAction {
        CarouselAction::Move {
            navigation: self,
            forward: false,
        }
    }

    pub(super) fn keyboard(self, key: &str) -> Option<CarouselAction> {
        match (self.vertical, key) {
            (true, "ArrowUp") | (false, "ArrowLeft") => Some(self.previous()),
            (true, "ArrowDown") | (false, "ArrowRight") => Some(self.next()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct CarouselState {
    positions: BTreeMap<usize, usize>,
    drags: BTreeMap<usize, DragState>,
}

#[derive(Clone, Debug, Default)]
struct DragState {
    pointer: Option<f32>,
    pressed_at: Option<f32>,
}

impl CarouselState {
    pub(super) fn positions(&self) -> &BTreeMap<usize, usize> {
        &self.positions
    }

    fn position(&self, navigation: Navigation) -> usize {
        self.positions
            .get(&navigation.key)
            .copied()
            .unwrap_or(0)
            .min(navigation.last())
    }

    fn advance(&mut self, navigation: Navigation, forward: bool) {
        let current = self.position(navigation);
        let next = navigation.destination(current, forward);
        if next != current {
            self.positions.insert(navigation.key, next);
        }
    }

    pub(super) fn apply(&mut self, action: CarouselAction) {
        match action {
            CarouselAction::Move {
                navigation,
                forward,
            } => self.advance(navigation, forward),
            CarouselAction::Press(key) => {
                let drag = self.drags.entry(key).or_default();
                drag.pressed_at = drag.pointer;
            }
            CarouselAction::Sample { key, offset } => {
                self.drags.entry(key).or_default().pointer = Some(offset);
            }
            CarouselAction::Release(navigation) => {
                let endpoints = self
                    .drags
                    .get_mut(&navigation.key)
                    .and_then(|drag| drag.pressed_at.take().zip(drag.pointer));
                if let Some((start, end)) = endpoints {
                    if (end - start).abs() >= 24.0 {
                        self.advance(navigation, end < start);
                    }
                }
            }
            CarouselAction::Cancel(key) => {
                self.drags.remove(&key);
            }
        }
    }
}

impl Renderer<'_> {
    pub(super) fn carousel(&mut self, root: &ComponentViewElement) -> WidgetView {
        let key = root.span.start;
        let track = element_children(root)
            .find(|el| matches!(el.tag.as_str(), "carousel-content" | "carousel_content"));
        let slides: Vec<_> = track
            .into_iter()
            .flat_map(element_children)
            .filter(|el| matches!(el.tag.as_str(), "carousel-item" | "carousel_item"))
            .collect();
        let visible = slides
            .first()
            .map_or(1, |slide| visible_count(&style(slide)))
            .min(slides.len());
        let vertical = expression(root, "orientation")
            .and_then(|expr| self.display(expr))
            .is_some_and(|value| value == "vertical");
        let looping = expression(root, "loop")
            .and_then(|expr| self.display(expr))
            .is_some_and(|value| value == "true");
        let navigation = Navigation {
            key,
            slides: slides.len(),
            visible,
            vertical,
            looping,
        };
        let position = self.state.carousel.position(navigation);

        self.targets.push(Control {
            key,
            label: literal(root, "aria-label").unwrap_or("Carousel").to_owned(),
            target: Target::Carousel(navigation),
            parent: None,
        });

        let track_style = track
            .map(|track| native_style(&style(track)))
            .unwrap_or_default();
        let height = track_style
            .classes
            .iter()
            .rev()
            .find_map(|class| match class {
                StyleClass::Height(size) => Some(size.to_pixels()).filter(|px| *px > 0),
                _ => None,
            });
        let gap = 4u16;
        let mut showing = Vec::with_capacity(visible);
        for (index, slide) in slides.into_iter().enumerate() {
            let offset = (index + navigation.slides - position) % navigation.slides;
            let previous_targets = self.targets.len();
            let children = self.items(&slide.items);
            if offset >= visible {
                self.targets.truncate(previous_targets);
                continue;
            }
            let mut targets = self.targets.split_off(previous_targets);
            for control in &mut targets {
                if control.parent.is_none() {
                    control.parent = Some(slide.span.start);
                }
            }
            let target = Control {
                key: slide.span.start,
                label: format!("Slide {} of {}", index + 1, navigation.slides),
                target: Target::CarouselSlide,
                parent: Some(key),
            };
            let mut cell_style =
                native_style(&format!("w-full border rounded-md {}", style(slide)));
            let mut cell = View::container(View::col().children(children).build())
                .padding(8)
                .center_x();
            // Centering without an explicit height makes an Iced container fill
            // the vertical axis, which collapses slides in a scrolling page.
            if let Some(height) = height.filter(|_| vertical) {
                let spaces = gap.saturating_mul(visible.saturating_sub(1) as u16);
                let cell_height = f32::from(height.saturating_sub(spaces) / visible as u16);
                cell_style
                    .classes
                    .push(StyleClass::Height(SizeValue::Pixels(cell_height)));
                cell = cell.center_y();
            }
            showing.push((offset, cell.with_style(cell_style).build(), target, targets));
        }
        showing.sort_by_key(|(offset, ..)| *offset);
        let mut children = Vec::with_capacity(visible);
        for (_, child, target, targets) in showing {
            self.targets.push(target);
            self.targets.extend(targets);
            children.push(child);
        }
        let panel = if vertical {
            View::col().spacing(gap).children(children).build()
        } else {
            View::grid()
                .cols(visible.max(1))
                .spacing(gap)
                .children(children)
                .build()
        };
        let (session, instance) = (self.session, self.instance);
        let drag = View::MouseArea {
            content: Box::new(panel),
            on_enter: None,
            on_exit: Some(self.action(Action::Carousel(CarouselAction::Cancel(key)))),
            on_double_click: None,
            on_click: Some(self.action(Action::Carousel(CarouselAction::Press(key)))),
            on_context_menu: None,
            on_release: Some(self.action(Action::Carousel(CarouselAction::Release(navigation)))),
            on_move: Some(
                PointerMoveHandler::new(move |x, y| NodeDesktopMsg::Widget {
                    session,
                    instance,
                    action: Action::Carousel(CarouselAction::Sample {
                        key,
                        offset: if vertical { y } else { x },
                    }),
                })
                .with_button_positions(),
            ),
            logical_extent: None,
            style: None,
        };
        let mut controls = View::row().spacing(8);
        for child in element_children(root) {
            let forward = match child.tag.as_str() {
                "carousel-next" | "carousel_next" => true,
                "carousel-previous" | "carousel_previous" | "carousel-prev" | "carousel_prev" => {
                    false
                }
                _ => continue,
            };
            let label = self.label(child);
            let label = if label.is_empty() {
                if forward {
                    "Next Slide"
                } else {
                    "Previous Slide"
                }
                .to_owned()
            } else {
                label
            };
            let action = (navigation.destination(position, forward) != position).then_some(
                Action::Carousel(CarouselAction::Move {
                    navigation,
                    forward,
                }),
            );
            controls = controls.child(self.button(
                child.span.start,
                label,
                &format!(
                    "bg-slate-100 text-slate-900 rounded-md p-2 {}",
                    style(child)
                ),
                action,
                None,
                None,
            ));
            self.targets.last_mut().expect("button target").parent = Some(key);
        }
        // Root height belongs to the track; controls need their own space.
        let mut classes = style(root);
        if self.state.focus == Some(key) {
            classes.push_str(" border-2 border-blue-700 rounded-md");
        }
        let mut root_style = native_style(&classes);
        if height.is_some() && vertical {
            root_style
                .classes
                .retain(|class| !matches!(class, StyleClass::Height(_)));
        }
        View::col()
            .spacing(8)
            .with_style(root_style)
            .child(drag)
            .child(controls.build())
            .build()
    }
}

fn visible_count(classes: &str) -> usize {
    let mut selected = (0.0, 1);
    for class in classes.split_whitespace() {
        let (min_width, class) = if let Some((prefix, class)) = class.split_once(':') {
            let Some(breakpoint) = Breakpoint::from_prefix(prefix) else {
                continue;
            };
            (breakpoint.min_width(), class)
        } else {
            (0.0, class)
        };
        if min_width < selected.0 || min_width > theme::window_width() {
            continue;
        }
        if class == "basis-full" {
            selected = (min_width, 1);
        } else if let Some((n, d)) = class
            .strip_prefix("basis-")
            .and_then(|basis| basis.split_once('/'))
        {
            if let (Ok(n), Ok(d)) = (n.parse::<usize>(), d.parse::<usize>()) {
                if let Some(fit) = d.checked_div(n).filter(|fit| *fit > 0) {
                    selected = (min_width, fit);
                }
            }
        }
    }
    selected.1
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::ui::ace_node_desktop::widgets::{
        accessibility_action, accessibility_tree, keyboard_action, State,
    };
    use accesskit::{Action as Request, ActionRequest, Role, TreeId};
    use ace_core::source::age_component::project_component_candidates;

    #[test]
    fn native_carousel_keeps_visible_slides_and_inputs_in_sync() {
        let source = r#"
widget Preview {
    view {
        preview-card {
            carousel (orientation: "vertical", loop: "true", "aria-label": "Gallery") {
                carousel-content {
                    carousel-item (style: "basis-1/2") { text "First" }
                    carousel-item (style: "basis-1/2") { text "Second" }
                    carousel-item (style: "basis-1/2") { text "Third" }
                }
                carousel-previous {}
                carousel-next {}
            }
            carousel {
                carousel-content {
                    carousel-item { text "One" }
                    carousel-item { text "Two" }
                }
                carousel-previous {}
                carousel-next {}
            }
        }
    }
}
"#;
        let nodes = project_component_candidates(source).expect("carousel view");
        let tree = &nodes[0].view.tree;
        let mut state = State::new(BTreeMap::new());
        let snapshot = |state: &State| accessibility_tree(7, tree, &[], &[], state);
        let visible = |state: &State| {
            snapshot(state)
                .nodes
                .iter()
                .filter_map(|(_, node)| {
                    (node.role_description() == Some("slide"))
                        .then(|| node.label().unwrap().to_owned())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            visible(&state),
            ["Slide 1 of 3", "Slide 2 of 3", "Slide 1 of 2"]
        );
        let initial = snapshot(&state);
        let (region_id, region) = &initial.nodes[0];
        assert_eq!(region.role(), Role::Region);
        assert_eq!(region.label(), Some("Gallery"));
        assert_eq!(region.children().len(), 4);
        let previous = initial
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Previous Slide"))
            .unwrap()
            .0;
        assert!(initial
            .nodes
            .iter()
            .rev()
            .any(|(_, node)| node.label() == Some("Previous Slide") && node.is_disabled()));
        let action = |state: &State, what, target_node| {
            accessibility_action(
                7,
                tree,
                &[],
                &[],
                state,
                ActionRequest {
                    action: what,
                    target_tree: TreeId::ROOT,
                    target_node,
                    data: None,
                },
            )
            .expect("live carousel access action")
        };
        state.apply(action(&state, Request::Click, previous));
        assert_eq!(
            visible(&state),
            ["Slide 3 of 3", "Slide 1 of 3", "Slide 1 of 2"]
        );
        state.apply(action(&state, Request::Focus, *region_id));
        assert!(keyboard_action(tree, &[], &[], &state, "ArrowRight").is_none());
        state.apply(keyboard_action(tree, &[], &[], &state, "ArrowDown").unwrap());
        assert_eq!(
            visible(&state),
            ["Slide 1 of 3", "Slide 2 of 3", "Slide 1 of 2"]
        );

        let region_key = region_id.0 as u32 as usize;
        let navigation = Navigation {
            key: region_key,
            slides: 3,
            visible: 2,
            vertical: true,
            looping: true,
        };
        for action in [
            CarouselAction::Sample {
                key: region_key,
                offset: 90.0,
            },
            CarouselAction::Press(region_key),
            CarouselAction::Sample {
                key: region_key,
                offset: 20.0,
            },
            CarouselAction::Release(navigation),
        ] {
            state.apply(Action::Carousel(action));
        }
        assert_eq!(
            visible(&state),
            ["Slide 2 of 3", "Slide 3 of 3", "Slide 1 of 2"]
        );
        let width = theme::window_width();
        theme::set_window_width(700.0);
        assert_eq!(visible_count("md:basis-1/3 basis-1/2"), 2);
        theme::set_window_width(900.0);
        assert_eq!(visible_count("md:basis-1/3 basis-1/2"), 3);
        theme::set_window_width(width);
    }

    #[test]
    fn native_carousel_keeps_natural_and_fixed_slide_heights() {
        let source = r#"
widget Preview {
    view {
        carousel {
            carousel-content {
                carousel-item { text "Horizontal" }
            }
        }
        carousel (orientation: "vertical") {
            carousel-content (style: "h-[200px]") {
                carousel-item (style: "basis-1/2") { text "Vertical one" }
                carousel-item (style: "basis-1/2") { text "Vertical two" }
            }
        }
    }
}
"#;
        let nodes = project_component_candidates(source).expect("carousel view");
        let state = State::new(BTreeMap::new());
        let view = super::super::view(7, 0, &nodes[0].view.tree, &[], &[], &state)
            .map_msg(|message| message);
        let View::Column { children, .. } = view else {
            panic!("expected a component column");
        };
        let [horizontal, vertical] = children.as_slice() else {
            panic!("expected both carousels");
        };
        fn slide_track(carousel: &WidgetView) -> &WidgetView {
            let View::Column { children, .. } = carousel else {
                panic!("expected a carousel column");
            };
            let View::MouseArea {
                content,
                on_move: Some(pointer),
                ..
            } = &children[0]
            else {
                panic!("expected an interactive track");
            };
            assert!(pointer.sample_button_positions);
            content
        }
        let View::Grid { cells, .. } = slide_track(horizontal) else {
            panic!("expected a horizontal track");
        };
        assert!(matches!(
            &cells[0],
            View::Container {
                center_y: false,
                ..
            }
        ));
        let View::Column { children, .. } = slide_track(vertical) else {
            panic!("expected a vertical track");
        };
        assert_eq!(children.len(), 2);
        for slide in children {
            let View::Container {
                center_y: true,
                style: Some(style),
                ..
            } = slide
            else {
                panic!("expected a centered vertical slide");
            };
            assert!(style.classes.iter().any(|class| matches!(
                class, StyleClass::Height(SizeValue::Pixels(height)) if *height == 98.0
            )));
        }
    }
}
