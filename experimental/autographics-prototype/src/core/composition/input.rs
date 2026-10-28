//! Hit-testing, capture and focus routing over the composition graph
//! (PLAN-830 §5.3; core model defined with T-03, host wiring in T-05/T-06).
//!
//! Coordinate chain:
//!
//!   window physical px
//!     → top view content-logical px (host maps window→view)
//!     → clip / scale (2D) or ray + plane UV (3D) per mount
//!     → child surface LOCAL px (origin top-left of the child's content)
//!
//! Rules proved by `tests/surface_contract.rs`:
//! - Hit order = INVERSE declaration order of a view's portals (last declared
//!   draws on top). The first mount that accepts the point OWNS it — an
//!   overlay consumes and never falls through.
//! - A point outside a mount's declared geometry (2D clip rect, or the child's
//!   own content bounds) never reaches the child.
//! - A non-interactive mount still OCCLUDES what is below it; it just does not
//!   deliver the event into its content (explicit passthrough is a mount
//!   property, not a per-hit accident).
//! - 3D ray/UV math is the host's (it owns the camera): the core takes a
//!   `PlaneHit` closure so traversal/routing stay testable without a GPU.
//! - Capture binds to a MOUNT PATH: after a press inside a child, moves and
//!   the release go to that same child until the press ends; destroying the
//!   target or losing focus CANCELS (generation bump), and a late release is
//!   never delivered to a different object.

use super::{CompositionError, CompositionGraph, MountKind, PortalId, SurfaceId, ViewId};

/// One level of the path from the window-facing view down to the hit child.
#[derive(Debug, Clone, PartialEq)]
pub struct HitStep {
    pub view: ViewId,
    /// Portal traversed from `view` into the next child surface.
    pub portal: PortalId,
    /// The consumer surface at this level (the view's surface).
    pub host: SurfaceId,
}

/// A resolved hit: deepest owning surface + the path + child-local coords.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// Outermost → innermost mounts traversed (empty for the top surface).
    pub path: Vec<HitStep>,
    pub surface: SurfaceId,
    /// Coordinate in `surface`'s local logical space (top-left origin).
    pub local: [f32; 2],
    pub nested: bool,
}

/// Host-provided 3D plane resolution: `(mount, parent_content_px, host)`
/// → child UV (0..1 on the plane). `host` is the surface whose VIEW owns the
/// mount, so the host resolves the ray with THAT surface's camera — borrowing
/// another parent's camera is not a valid hit (review r1 F04).
pub type PlaneHit<'a> = dyn Fn(&MountKind, [f32; 2], SurfaceId) -> Option<[f32; 2]> + 'a;

/// A `PlaneHit` that never resolves (used when the top view is 2D-only).
pub fn no_planes(_: &MountKind, _: [f32; 2], _: SurfaceId) -> Option<[f32; 2]> {
    None
}

/// Result of one descent level.
enum Descent {
    /// An interactive descendant owns the point.
    Hit(Hit),
    /// This level's host surface owns the point (no deeper interactive mount).
    Host,
    /// A non-interactive mount at this level or deeper consumes the point:
    /// nothing may receive it (review r1 F05 — the old code treated a nested
    /// occluder as "no deeper hit" and delivered to the child).
    Occluded,
}

/// Hit-test a point given in the TOP view's content-logical space.
pub fn hit_test(
    graph: &CompositionGraph,
    top_view: ViewId,
    point: [f32; 2],
    plane: &PlaneHit<'_>,
) -> Result<Option<Hit>, CompositionError> {
    if !point.iter().all(|v| v.is_finite()) {
        // Non-finite input is rejected outright (PLAN-830 §5.3 非法输入).
        return Err(CompositionError::InvalidMount {
            detail: "non-finite input point".into(),
        });
    }
    let view = graph.view(top_view)?;
    match descend(graph, view.id, view.desc.surface, point, Vec::new(), plane)? {
        Descent::Hit(hit) => Ok(Some(hit)),
        Descent::Occluded => Ok(None),
        Descent::Host => Ok(Some(Hit {
            nested: false,
            path: Vec::new(),
            surface: view.desc.surface,
            local: point,
        })),
    }
}

fn descend(
    graph: &CompositionGraph,
    view_id: ViewId,
    host_surface: SurfaceId,
    point: [f32; 2],
    path: Vec<HitStep>,
    plane: &PlaneHit<'_>,
) -> Result<Descent, CompositionError> {
    let view = graph.view(view_id)?;
    for portal_id in view.portals.iter().rev().copied() {
        let portal = graph.portal(portal_id)?;
        let child = graph.surface(portal.desc.child)?;
        let Some(local) = mount_to_child_local(
            &portal.desc.mount,
            point,
            child.desc.logical_size,
            plane,
            host_surface,
        ) else {
            continue;
        };
        if !portal.desc.interactive {
            // Declared geometry owns the point but does not deliver it: the
            // layers below must not receive it either.
            return Ok(Descent::Occluded);
        }
        let mut nested_path = path.clone();
        nested_path.push(HitStep {
            view: view_id,
            portal: portal_id,
            host: host_surface,
        });
        // Deeper nesting continues through a view of the child surface.
        if let Some(child_view) = child
            .views
            .iter()
            .copied()
            .find(|v| {
                graph
                    .view(*v)
                    .map(|v| v.desc.presents_to_window)
                    .unwrap_or(false)
            })
            .or_else(|| child.views.first().copied())
        {
            match descend(
                graph,
                child_view,
                child.id,
                local,
                nested_path.clone(),
                plane,
            )? {
                Descent::Hit(deeper) => return Ok(Descent::Hit(deeper)),
                // An inner occluder consumes the event for the WHOLE chain.
                Descent::Occluded => return Ok(Descent::Occluded),
                // No deeper interactive mount owns it: the child itself does.
                Descent::Host => {}
            }
        }
        return Ok(Descent::Hit(Hit {
            path: nested_path,
            surface: child.id,
            local,
            nested: true,
        }));
    }
    Ok(Descent::Host)
}

/// Map a parent-content point through one mount into the child's LOCAL px.
/// `None` = the mount does not accept the point (outside clip / content /
/// plane rectangle).
pub fn mount_to_child_local(
    mount: &MountKind,
    point: [f32; 2],
    child_size: [f32; 2],
    plane: &PlaneHit<'_>,
    host: SurfaceId,
) -> Option<[f32; 2]> {
    let local = match *mount {
        MountKind::Flat2d {
            offset,
            scale,
            clip,
        } => {
            if !clip.contains(point[0], point[1]) {
                return None;
            }
            [
                (point[0] - offset[0]) / scale[0],
                (point[1] - offset[1]) / scale[1],
            ]
        }
        MountKind::Plane3d { .. } => {
            let uv = plane(mount, point, host)?;
            if !uv.iter().all(|v| v.is_finite()) {
                return None;
            }
            [uv[0] * child_size[0], uv[1] * child_size[1]]
        }
    };
    in_content(local, child_size).then_some(local)
}

fn in_content(local: [f32; 2], size: [f32; 2]) -> bool {
    local[0] >= 0.0 && local[0] < size[0] && local[1] >= 0.0 && local[1] < size[1]
}

/// One keyboard-target migration caused by a press: `from` (the surface that
/// held the single focus before) must release its real widget/IME state (the
/// host adapter owns that cache), `to` is the new target. Recorded by the
/// core so the identity of the OLD target is never lost (review r2 F02).
#[derive(Debug, Clone, PartialEq)]
pub struct FocusTransfer {
    pub from: Option<SurfaceId>,
    pub to: SurfaceId,
}

/// Input routing state: capture + focus, keyed to mount paths so a destroyed
/// target can never receive a late event.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InputState {
    pub capture: Option<Capture>,
    pub focus: Option<Focus>,
    generation: u64,
    pub cancellations: Vec<CancelReason>,
    /// The last focus migration caused by `press`, consumed by the host so it
    /// can unfocus the OLD control (review r2 F02). Not part of the routing
    /// decision; evidence + handoff only.
    focus_transfer: Option<FocusTransfer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    pub hit: Hit,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Focus {
    pub hit: Hit,
    /// Widget-level key inside the surface (host adapter supplies it, e.g. a
    /// text_input id); opaque to the core.
    pub widget: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelReason {
    TargetDestroyed,
    WindowFocusLost,
    PointerLeftWindow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputOutcome {
    Delivered {
        surface: SurfaceId,
        local: [f32; 2],
    },
    /// Swallowed by an occluding (non-interactive) mount.
    Occluded,
    /// A capture was cancelled this event; nothing is delivered.
    Cancelled(CancelReason),
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Press: hit-test, capture when a surface owns the point, and make that
    /// surface the single keyboard target.
    pub fn press(
        &mut self,
        graph: &CompositionGraph,
        top_view: ViewId,
        point: [f32; 2],
        plane: &PlaneHit<'_>,
    ) -> Result<InputOutcome, CompositionError> {
        match hit_test(graph, top_view, point, plane)? {
            Some(hit) => {
                let previous = self.focus.as_ref().map(|f| f.hit.surface);
                // Report a REAL migration only (same surface keeps its edit
                // state; a different mount path of the same shared surface is
                // still one control instance).
                self.focus_transfer = if previous.is_some() && previous != Some(hit.surface) {
                    Some(FocusTransfer {
                        from: previous,
                        to: hit.surface,
                    })
                } else {
                    None
                };
                // The router's FINAL target is authoritative even when the
                // host batches several routed clicks into one frame: the
                // runtime reads this stamp and keeps only that surface's real
                // widget focus (review r3 F-830-R3-01).
                graph.note_routed_focus(hit.surface);
                self.capture = Some(Capture {
                    hit: hit.clone(),
                    generation: self.generation,
                });
                self.focus = Some(Focus {
                    hit: hit.clone(),
                    widget: None,
                });
                Ok(InputOutcome::Delivered {
                    surface: hit.surface,
                    local: hit.local,
                })
            }
            None => {
                self.capture = None;
                self.focus_transfer = None;
                Ok(InputOutcome::Occluded)
            }
        }
    }

    /// Consume the last press-induced focus migration (host unfocuses `from`).
    pub fn take_focus_transfer(&mut self) -> Option<FocusTransfer> {
        self.focus_transfer.take()
    }

    /// Move: a held capture keeps the SAME path (clamped at the mount edge)
    /// even when the pointer leaves the child's rect.
    pub fn moved(
        &mut self,
        graph: &CompositionGraph,
        top_view: ViewId,
        point: [f32; 2],
        plane: &PlaneHit<'_>,
    ) -> Result<InputOutcome, CompositionError> {
        if self.capture.is_some() {
            if self.stale_capture(graph) {
                let _ = self.cancel(CancelReason::TargetDestroyed);
                return Ok(InputOutcome::Cancelled(CancelReason::TargetDestroyed));
            }
            let capture = self.capture.as_ref().expect("checked");
            let local = remap_into(graph, &capture.hit.path, point, plane)?;
            let surface = capture.hit.surface;
            return Ok(InputOutcome::Delivered { surface, local });
        }
        match hit_test(graph, top_view, point, plane)? {
            Some(hit) => Ok(InputOutcome::Delivered {
                surface: hit.surface,
                local: hit.local,
            }),
            None => Ok(InputOutcome::Occluded),
        }
    }

    /// Release: delivered to the captured surface (never re-hit-tested), then
    /// the capture is released.
    pub fn release(&mut self) -> Option<SurfaceId> {
        self.capture.take().map(|c| c.hit.surface)
    }

    /// Explicit cancel (focus loss / window leave / target destroy). Clears
    /// capture AND the keyboard focus/edit target — a lost window must never
    /// keep delivering keys to an old control — bumps the generation so a
    /// late release cannot match a newer capture, and returns the surface
    /// whose focus was dropped so the host can release widget/IME state.
    pub fn cancel(&mut self, reason: CancelReason) -> Option<SurfaceId> {
        self.generation += 1;
        self.capture = None;
        self.focus_transfer = None;
        let focused = self.focus.take().map(|f| f.hit.surface);
        self.cancellations.push(reason);
        focused
    }

    fn path_is_dead(graph: &CompositionGraph, hit: &Hit) -> bool {
        graph.surface(hit.surface).is_err()
            || hit.path.iter().any(|step| {
                graph.view(step.view).is_err()
                    || graph.portal(step.portal).is_err()
                    || graph.surface(step.host).is_err()
            })
    }

    fn stale_capture(&self, graph: &CompositionGraph) -> bool {
        self.capture
            .as_ref()
            .map(|c| Self::path_is_dead(graph, &c.hit))
            .unwrap_or(false)
    }

    /// Drop a focus whose path died (no dangling edit state survives a
    /// destroy/unmount). Returns the dead surface for reports.
    pub fn revalidate_focus(&mut self, graph: &CompositionGraph) -> Option<SurfaceId> {
        let dead = self
            .focus
            .as_ref()
            .map(|f| Self::path_is_dead(graph, &f.hit))
            .unwrap_or(false);
        if dead {
            // A dead path can no longer be the routed target, and a batch
            // still pending in the host must not revive an editor state
            // (review r4 F-830-R4-01).
            graph.cancel_routed_focus();
            return self.cancel(CancelReason::TargetDestroyed);
        }
        None
    }

    /// Keyboard delivery target (the focused surface), if any.
    pub fn keyboard_target(&self) -> Option<SurfaceId> {
        self.focus.as_ref().map(|f| f.hit.surface)
    }
}

/// Re-map a point into the captured child by walking the SAME mount path.
/// Off-mount steps clamp to the mount edge (the child keeps the capture).
fn remap_into(
    graph: &CompositionGraph,
    path: &[HitStep],
    point: [f32; 2],
    plane: &PlaneHit<'_>,
) -> Result<[f32; 2], CompositionError> {
    let mut cur = point;
    for step in path {
        let portal = graph.portal(step.portal)?;
        let child = graph.surface(portal.desc.child)?;
        match mount_to_child_local(
            &portal.desc.mount,
            cur,
            child.desc.logical_size,
            plane,
            step.host,
        ) {
            Some(local) => cur = local,
            None => {
                cur = clamp_into_mount(&portal.desc.mount, cur, child.desc.logical_size);
            }
        }
    }
    Ok(cur)
}

fn clamp_into_mount(mount: &MountKind, point: [f32; 2], child_size: [f32; 2]) -> [f32; 2] {
    match mount {
        MountKind::Flat2d {
            offset,
            scale,
            clip,
        } => {
            let edge = [
                point[0].clamp(clip.x, clip.x + clip.w - f32::EPSILON),
                point[1].clamp(clip.y, clip.y + clip.h - f32::EPSILON),
            ];
            [
                ((edge[0] - offset[0]) / scale[0]).clamp(0.0, child_size[0] - f32::EPSILON),
                ((edge[1] - offset[1]) / scale[1]).clamp(0.0, child_size[1] - f32::EPSILON),
            ]
        }
        MountKind::Plane3d { .. } => [
            point[0].clamp(0.0, child_size[0] - f32::EPSILON),
            point[1].clamp(0.0, child_size[1] - f32::EPSILON),
        ],
    }
}
