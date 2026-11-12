//! Shared modal scope queries for native input and automation. Logical paths
//! use the same child order as the renderer's VTree.
use super::view::{PopoverAnchor, View};
use super::vnode_converter::extract_children_ref;

/// The last painted open modal owns input. Nested modals further narrow it.
pub fn active_content_path<M: Clone + std::fmt::Debug>(view: &View<M>) -> Option<Vec<u16>> {
    fn visit<M: Clone + std::fmt::Debug>(
        view: &View<M>,
        path: &mut Vec<u16>,
        found: &mut Option<Vec<u16>>,
    ) {
        if let View::AnchorSlot { child, .. } | View::ScrollAnchorMark { child, .. } = view {
            visit(child, path, found);
            return;
        }
        if let View::Popover {
            placement,
            open: true,
            anchor,
            content,
            ..
        } = view
        {
            if placement.is_modal_chrome() {
                let index = if matches!(anchor, PopoverAnchor::Widget(_)) {
                    1
                } else {
                    0
                };
                path.push(index);
                *found = Some(path.clone());
                visit(content, path, found);
                path.pop();
                return;
            }
        }
        for (index, child) in extract_children_ref(view).iter().enumerate() {
            path.push(index as u16);
            visit(child, path, found);
            path.pop();
        }
    }
    let mut found = None;
    visit(view, &mut Vec::new(), &mut found);
    found
}

pub fn allows_path<M: Clone + std::fmt::Debug>(view: &View<M>, path: &[u16]) -> bool {
    active_content_path(view)
        .map(|scope| path.starts_with(&scope))
        .unwrap_or(true)
}

pub fn at_path<'a, M: Clone + std::fmt::Debug>(
    view: &'a View<M>,
    path: &[u16],
) -> Option<&'a View<M>> {
    if let View::AnchorSlot { child, .. } | View::ScrollAnchorMark { child, .. } = view {
        return at_path(child, path);
    }
    if path.is_empty() {
        return Some(view);
    }
    at_path(
        extract_children_ref(view).get(path[0] as usize)?,
        &path[1..],
    )
}

pub fn escape_message<M: Clone + std::fmt::Debug>(view: &View<M>) -> Option<M> {
    let path = active_content_path(view)?;
    let View::Popover { on_dismiss, .. } = at_path(view, &path[..path.len() - 1])? else {
        return None;
    };
    on_dismiss.clone()
}
