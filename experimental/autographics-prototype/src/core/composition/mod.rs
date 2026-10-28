//! GPU-agnostic surface/portal composition model (PLAN-830 T-03 / V-01).
//!
//! Contract (PLAN-830 §5.1):
//! - `Surface` = a content instance with its OWN state/updates; `View` = one
//!   observation/output of a surface; `Portal` = a mount of a child surface
//!   into a parent view, with the mapping that turns parent-space coordinates
//!   into child-space coordinates (and back). None of these are OS windows.
//! - The same surface may be mounted by multiple portals: its simulation
//!   advances ONCE per composition tick, every mount presents the shared
//!   content, and per-mount hit/focus state lives in the INPUT layer
//!   (`input.rs`), never by duplicating content.
//! - A composition batch is validated WHOLE before anything is applied:
//!   identities, references, mounts (finite/invertible), acyclicity and the
//!   configurable depth budget. A rejected batch leaves the previous graph
//!   and its identity space untouched (no half-applied state, no consumed
//!   ids) — the commit is a whole-state swap.
//! - Scheduling is derived from the validated dependency DAG: consumed
//!   (child) content renders before its consumers; a surface participates
//!   exactly once regardless of mount count.
//!
//! This module must not depend on wgpu, winit, iced, or any language/VM
//! types. Identities REUSE `core::id`'s generational allocator (one allocator
//! per graph session) instead of introducing a second conflicting id source.

pub mod input;

use crate::core::id::{IdAllocator, IdentityError, ObjectId, SessionId};

// ---------------------------------------------------------------------------
// Identities (newtypes over the shared generational allocator)
// ---------------------------------------------------------------------------

macro_rules! composition_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub(crate) ObjectId);

        impl $name {
            pub fn token(self) -> String {
                self.0.token()
            }
            pub fn raw(self) -> ObjectId {
                self.0
            }
        }
    };
}

composition_id!(SurfaceId);
composition_id!(ViewId);
composition_id!(PortalId);

// ---------------------------------------------------------------------------
// Descriptions
// ---------------------------------------------------------------------------

/// What kind of content a surface carries. Adapters live host-side; the
/// graph records the kind for scheduling, budget and reports only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    /// Real iced UI instance (stock widgets).
    Iced,
    /// 2D game content (Breakout-style runtime driver).
    Game2d,
    /// 3D scene content (meshes + portal planes).
    Scene3d,
    /// Static 2D canvas content.
    Canvas2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn at(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

/// How a portal mounts child content into the parent view's space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MountKind {
    /// 2D affine mount: child-local → parent-content `offset + scale*local`,
    /// hit-clipped to `clip` (parent-content space).
    Flat2d {
        offset: [f32; 2],
        scale: [f32; 2],
        clip: Rect,
    },
    /// 3D plane mount: an axis rectangle at `center` with half extents
    /// `half`, facing +Z of the parent 3D space. Ray/perspective math is the
    /// HOST's (it owns the camera); the graph records placement only.
    Plane3d { center: [f32; 3], half: [f32; 2] },
}

impl MountKind {
    /// Reject non-finite components and uninvertible mounts (an empty or
    /// zero-scale mount cannot carry input back to the child).
    pub fn validate(&self) -> Result<(), CompositionError> {
        let bad = |what: &str| CompositionError::InvalidMount {
            detail: what.to_string(),
        };
        match *self {
            MountKind::Flat2d {
                offset,
                scale,
                clip,
            } => {
                if !offset.iter().all(|v| v.is_finite())
                    || !scale.iter().all(|v| v.is_finite())
                    || ![clip.x, clip.y, clip.w, clip.h]
                        .iter()
                        .all(|v| v.is_finite())
                {
                    return Err(bad("non-finite component"));
                }
                if scale[0] == 0.0 || scale[1] == 0.0 {
                    return Err(bad("zero scale (uninvertible)"));
                }
                if clip.w <= 0.0 || clip.h <= 0.0 {
                    return Err(bad("empty clip"));
                }
            }
            MountKind::Plane3d { center, half } => {
                if !center.iter().all(|v| v.is_finite()) || !half.iter().all(|v| v.is_finite()) {
                    return Err(bad("non-finite component"));
                }
                if half[0] <= 0.0 || half[1] <= 0.0 {
                    return Err(bad("empty plane"));
                }
            }
        }
        Ok(())
    }

    /// Flat mounts map child-local → parent-content trivially; 3D planes
    /// need the host's projector (returns `None` here).
    pub fn flat_child_to_parent(&self, child_local: [f32; 2]) -> Option<[f32; 2]> {
        match *self {
            MountKind::Flat2d { offset, scale, .. } => Some([
                offset[0] + child_local[0] * scale[0],
                offset[1] + child_local[1] * scale[1],
            ]),
            MountKind::Plane3d { .. } => None,
        }
    }

    /// Flat mounts map parent-content → child-local (inverse affine); the
    /// caller checks the clip rect separately.
    pub fn flat_parent_to_child(&self, parent_px: [f32; 2]) -> Option<[f32; 2]> {
        match *self {
            MountKind::Flat2d { offset, scale, .. } => Some([
                (parent_px[0] - offset[0]) / scale[0],
                (parent_px[1] - offset[1]) / scale[1],
            ]),
            MountKind::Plane3d { .. } => None,
        }
    }

    /// The parent-content clip rect (flat mounts only).
    pub fn clip(&self) -> Option<Rect> {
        match *self {
            MountKind::Flat2d { clip, .. } => Some(clip),
            MountKind::Plane3d { .. } => None,
        }
    }

    /// True when the parent-content point is inside the declared mount
    /// geometry (flat: clip rect; 3D: the host resolves this via ray hit).
    pub fn accepts_parent_point(&self, parent_px: [f32; 2]) -> bool {
        match *self {
            MountKind::Flat2d { clip, .. } => clip.contains(parent_px[0], parent_px[1]),
            MountKind::Plane3d { .. } => true, // host must resolve the ray
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceDesc {
    pub logical_size: [f32; 2],
    pub content: ContentKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewDesc {
    pub surface: SurfaceId,
    /// Window-facing views present to the host window; witness views render
    /// offscreen only (acceptance captures).
    pub presents_to_window: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortalDesc {
    pub parent: ViewId,
    pub child: SurfaceId,
    pub mount: MountKind,
    /// False = display-only mount (never hit-tested).
    pub interactive: bool,
}

// ---------------------------------------------------------------------------
// Errors (classified, never a generic failure)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CompositionError {
    InvalidIdentity {
        detail: IdentityError,
        token: String,
    },
    UnknownSurface(SurfaceId),
    UnknownView(ViewId),
    UnknownPortal(PortalId),
    /// Direct or indirect consumer cycle. The path lists the offending loop.
    CycleDetected {
        path: Vec<String>,
    },
    DepthBudgetExceeded {
        depth: usize,
        budget: usize,
    },
    InvalidMount {
        detail: String,
    },
    InvalidSurface {
        detail: String,
    },
    /// Removing a surface that still has portals or views attached.
    SurfaceInUse {
        token: String,
    },
    /// Content built on a different GPU device than the runtime.
    DeviceMismatch {
        expected: u64,
        actual: u64,
    },
    /// Content whose GPU adapter was built from a raw device handle, so the
    /// runtime CANNOT verify same-device ownership. Fail closed: an
    /// unverifiable object is not admitted under a contract that claims a
    /// single verified device (review r2 F04 — the old `None` branch was a
    /// silent pass).
    DeviceUnverifiable {
        detail: String,
    },
    /// A consumer sampled a child that had not produced this frame's content
    /// yet (submit-order DAG violation).
    DependencyOrder {
        detail: String,
    },
}

impl std::fmt::Display for CompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositionError::InvalidIdentity { detail, token } => {
                write!(f, "invalid identity {token}: {detail:?}")
            }
            CompositionError::UnknownSurface(s) => write!(f, "unknown surface {}", s.token()),
            CompositionError::UnknownView(v) => write!(f, "unknown view {}", v.token()),
            CompositionError::UnknownPortal(p) => write!(f, "unknown portal {}", p.token()),
            CompositionError::CycleDetected { path } => {
                write!(f, "consumer cycle: {}", path.join(" -> "))
            }
            CompositionError::DepthBudgetExceeded { depth, budget } => {
                write!(f, "depth {depth} exceeds budget {budget}")
            }
            CompositionError::InvalidMount { detail } => write!(f, "invalid mount: {detail}"),
            CompositionError::InvalidSurface { detail } => write!(f, "invalid surface: {detail}"),
            CompositionError::SurfaceInUse { token } => {
                write!(f, "surface {token} is still mounted or viewed")
            }
            CompositionError::DeviceMismatch { expected, actual } => {
                write!(
                    f,
                    "content built on device {actual}, runtime device {expected}"
                )
            }
            CompositionError::DependencyOrder { detail } => {
                write!(f, "dependency order violated: {detail}")
            }
            CompositionError::DeviceUnverifiable { detail } => {
                write!(f, "content device cannot be verified: {detail}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Graph state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceState {
    pub id: SurfaceId,
    pub desc: SurfaceDesc,
    /// Portals mounting this surface.
    pub mounted_by: Vec<PortalId>,
    /// Views observing this surface.
    pub views: Vec<ViewId>,
    /// Monotonic content revision (bumped by content changes/ticks).
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewState {
    pub id: ViewId,
    pub desc: ViewDesc,
    /// Portals mounted INTO this view (declaration order = draw order).
    pub portals: Vec<PortalId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortalState {
    pub id: PortalId,
    pub desc: PortalDesc,
}

/// One validated change. Batches are applied atomically by `submit`.
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionChange {
    AddSurface { desc: SurfaceDesc },
    RemoveSurface(SurfaceId),
    AddView { desc: ViewDesc },
    RemoveView(ViewId),
    AddPortal { desc: PortalDesc },
    RemovePortal(PortalId),
    SetMount(PortalId, MountKind),
    SetSurfaceSize(SurfaceId, [f32; 2]),
}

/// Ids minted by a committed batch, keyed by the change index that created
/// them (so callers can pair `AddSurface{..}` with its new id).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BatchMints {
    pub created: Vec<(usize, Id)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Id {
    Surface(SurfaceId),
    View(ViewId),
    Portal(PortalId),
}

impl BatchMints {
    pub fn surface(&self, change_index: usize) -> Option<SurfaceId> {
        self.created.iter().find_map(|(i, id)| match id {
            Id::Surface(s) if *i == change_index => Some(*s),
            _ => None,
        })
    }
    pub fn view(&self, change_index: usize) -> Option<ViewId> {
        self.created.iter().find_map(|(i, id)| match id {
            Id::View(v) if *i == change_index => Some(*v),
            _ => None,
        })
    }
    pub fn portal(&self, change_index: usize) -> Option<PortalId> {
        self.created.iter().find_map(|(i, id)| match id {
            Id::Portal(p) if *i == change_index => Some(*p),
            _ => None,
        })
    }
    /// First created surface of the batch (single-add convenience).
    pub fn first_surface(&self) -> Option<SurfaceId> {
        self.created.iter().find_map(|(_, id)| match id {
            Id::Surface(s) => Some(*s),
            _ => None,
        })
    }
    pub fn first_view(&self) -> Option<ViewId> {
        self.created.iter().find_map(|(_, id)| match id {
            Id::View(v) => Some(*v),
            _ => None,
        })
    }
    pub fn first_portal(&self) -> Option<PortalId> {
        self.created.iter().find_map(|(_, id)| match id {
            Id::Portal(p) => Some(*p),
            _ => None,
        })
    }
}

/// The content-addressed graph state (clonable so batch validation can run
/// on a throwaway copy — a rejected batch then costs no id space and leaves
/// the committed graph byte-identical).
#[derive(Debug, Clone)]
struct GraphCore {
    alloc: IdAllocator,
    surfaces: Vec<SurfaceState>,
    views: Vec<ViewState>,
    portals: Vec<PortalState>,
}

impl GraphCore {
    fn find_surface(&self, id: SurfaceId) -> Option<usize> {
        self.surfaces.iter().position(|s| s.id == id)
    }
    fn find_view(&self, id: ViewId) -> Option<usize> {
        self.views.iter().position(|v| v.id == id)
    }
    fn find_portal(&self, id: PortalId) -> Option<usize> {
        self.portals.iter().position(|p| p.id == id)
    }

    fn id_error(&self, id: ObjectId, token: String) -> CompositionError {
        CompositionError::InvalidIdentity {
            detail: self.alloc.validate(id).unwrap_err_or_unknown(),
            token,
        }
    }
}

/// Small helper: `validate` returns Ok when the id is live.
trait UnwrapErrOrUnknown {
    fn unwrap_err_or_unknown(self) -> IdentityError;
}

impl UnwrapErrOrUnknown for Result<(), IdentityError> {
    fn unwrap_err_or_unknown(self) -> IdentityError {
        match self {
            Ok(()) => IdentityError::Unknown,
            Err(e) => e,
        }
    }
}

/// The composition graph. One per host session; ids from other sessions are
/// rejected by the shared allocator's session semantics.
#[derive(Debug, Clone)]
pub struct CompositionGraph {
    core: GraphCore,
    /// Longest consumer chain allowed. Rejection is CLASSIFIED
    /// (`DepthBudgetExceeded`); this is a resource budget, not a product
    /// cap on what kinds of UIs may be built.
    pub max_depth: usize,
    /// Surfaces whose content changed since `clear_dirty`, including every
    /// consumer reached through the mount chain.
    dirty: std::collections::BTreeSet<SurfaceId>,
    /// The keyboard-target state of the router for this composition session.
    /// `Unset` = nothing has routed yet (a host may focus freely);
    /// `Target` = the single real edit target; `Cancelled` = the router lost
    /// focus or the focused path died AFTER routing, so a still-pending event
    /// batch must not revive any editor state (review r4 F-830-R4-01).
    /// Interior mutability: recording a routed target must not require
    /// `&mut` from the router, whose signature is part of the contract.
    routed_focus: std::cell::Cell<RoutedFocus>,
}

/// Tri-state keyboard-target authority shared by the router and the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutedFocus {
    /// No routed press/public focus yet in this session.
    #[default]
    Unset,
    /// The router's current single keyboard target.
    Target(SurfaceId),
    /// Focus was cancelled (window blur / pointer left / dead path). Pending
    /// events may still be delivered, but no iced surface may end the frame
    /// with a real edit state until a NEW press or focus arrives.
    Cancelled,
}

impl Default for CompositionGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositionGraph {
    pub fn new() -> Self {
        CompositionGraph {
            core: GraphCore {
                alloc: IdAllocator::new(),
                surfaces: Vec::new(),
                views: Vec::new(),
                portals: Vec::new(),
            },
            max_depth: 8,
            dirty: Default::default(),
            routed_focus: std::cell::Cell::new(RoutedFocus::Unset),
        }
    }

    /// Record the router's CURRENT keyboard target. Called by the input
    /// router on every delivered press and by the public focus operation; the
    /// runtime reads it at frame start to keep exactly one real edit target.
    pub fn note_routed_focus(&self, surface: SurfaceId) {
        self.routed_focus.set(RoutedFocus::Target(surface));
    }

    /// The router's current keyboard target, if any. `None` covers both "no
    /// routing yet" and "cancelled" — use `routed_focus_state` to tell them
    /// apart (only the latter must suppress focus for pending events).
    pub fn routed_focus(&self) -> Option<SurfaceId> {
        match self.routed_focus.get() {
            RoutedFocus::Target(id) => Some(id),
            RoutedFocus::Unset | RoutedFocus::Cancelled => None,
        }
    }

    /// The full tri-state keyboard authority.
    pub fn routed_focus_state(&self) -> RoutedFocus {
        self.routed_focus.get()
    }

    /// The router lost focus / the focused path died: pending events are still
    /// delivered by the host, but no surface may end up focused until a new
    /// press/focus records a target (review r4 F-830-R4-01).
    pub fn cancel_routed_focus(&self) {
        self.routed_focus.set(RoutedFocus::Cancelled);
    }

    /// Return to "nothing routed yet" (session reset / test scaffolding).
    pub fn clear_routed_focus(&self) {
        self.routed_focus.set(RoutedFocus::Unset);
    }

    pub fn session(&self) -> SessionId {
        self.core.alloc.session()
    }

    // -- reads --------------------------------------------------------------

    pub fn surface(&self, id: SurfaceId) -> Result<&SurfaceState, CompositionError> {
        self.core
            .find_surface(id)
            .map(|i| &self.core.surfaces[i])
            .ok_or_else(|| self.lookup_error(id.raw(), id.token(), LookupKind::Surface))
    }

    pub fn view(&self, id: ViewId) -> Result<&ViewState, CompositionError> {
        self.core
            .find_view(id)
            .map(|i| &self.core.views[i])
            .ok_or_else(|| self.lookup_error(id.raw(), id.token(), LookupKind::View))
    }

    pub fn portal(&self, id: PortalId) -> Result<&PortalState, CompositionError> {
        self.core
            .find_portal(id)
            .map(|i| &self.core.portals[i])
            .ok_or_else(|| self.lookup_error(id.raw(), id.token(), LookupKind::Portal))
    }

    pub fn surfaces(&self) -> &[SurfaceState] {
        &self.core.surfaces
    }

    pub fn views(&self) -> &[ViewState] {
        &self.core.views
    }

    pub fn portals(&self) -> &[PortalState] {
        &self.core.portals
    }

    fn lookup_error(&self, raw: ObjectId, token: String, kind: LookupKind) -> CompositionError {
        if self.core.alloc.validate(raw).is_err() {
            return self.core.id_error(raw, token);
        }
        match kind {
            LookupKind::Surface => CompositionError::UnknownSurface(SurfaceId(raw)),
            LookupKind::View => CompositionError::UnknownView(ViewId(raw)),
            LookupKind::Portal => CompositionError::UnknownPortal(PortalId(raw)),
        }
    }

    pub fn live_surfaces(&self) -> usize {
        self.core.surfaces.len()
    }

    // -- batch submission ---------------------------------------------------

    /// Validate the WHOLE batch on a throwaway copy and commit only when
    /// every entry is valid. Returns the batch's mints on success, or every
    /// detected error with the graph untouched (same `snapshot()`, same id
    /// space).
    pub fn submit(
        &mut self,
        batch: &[CompositionChange],
    ) -> Result<BatchMints, Vec<CompositionError>> {
        let mut sim = self.core.clone();
        let mut mints = BatchMints::default();
        let mut errors = Vec::new();
        // Surfaces whose COMPOSED OUTPUT changed because of a structural edit
        // (mount added/moved/removed, view closed, surface resized). The
        // committed graph must not leave a consumer clean while its output is
        // stale — otherwise unmount leaves old pixels and resize leaves blank
        // content (PLAN-830 review r1 F01).
        let mut affected: std::collections::BTreeSet<SurfaceId> = Default::default();
        for (i, change) in batch.iter().enumerate() {
            if let Err(e) = apply_change(
                &mut sim,
                change,
                i,
                &mut mints,
                self.max_depth,
                &mut affected,
            ) {
                errors.push(e);
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        // Commit: whole-state swap, then invalidate every affected consumer
        // chain. Newly created surfaces are dirty by construction (first
        // render); structural edits dirty the OWNER of the changed output
        // plus everything that consumes it.
        self.core = sim;
        for (_, id) in &mints.created {
            if let Id::Surface(s) = id {
                affected.insert(*s);
            }
        }
        for surface in affected {
            self.dirty.insert(surface);
            for consumer in self.consumer_chain(surface) {
                self.dirty.insert(consumer);
            }
        }
        Ok(mints)
    }

    /// Single-change convenience: one atomic batch of one change.
    pub fn submit_one(
        &mut self,
        change: CompositionChange,
    ) -> Result<BatchMints, Vec<CompositionError>> {
        self.submit(std::slice::from_ref(&change))
    }

    // -- scheduling ---------------------------------------------------------

    /// Surfaces that depend on `surface` (transitively), nearest first.
    pub fn consumer_chain(&self, surface: SurfaceId) -> Vec<SurfaceId> {
        let mut out = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(surface);
        let mut seen: std::collections::BTreeSet<SurfaceId> = Default::default();
        while let Some(cur) = queue.pop_front() {
            let Some(s) = self.core.surfaces.iter().find(|s| s.id == cur) else {
                continue;
            };
            for portal_id in s.mounted_by.clone() {
                let Some(portal) = self.core.portals.iter().find(|p| p.id == portal_id) else {
                    continue;
                };
                let Some(view) = self.core.views.iter().find(|v| v.id == portal.desc.parent) else {
                    continue;
                };
                let parent = view.desc.surface;
                if seen.insert(parent) {
                    out.push(parent);
                    queue.push_back(parent);
                }
            }
        }
        out
    }

    /// Deterministic render/update order: every surface exactly once,
    /// consumed content before consumers. Allocation order breaks ties.
    pub fn render_order(&self) -> Vec<SurfaceId> {
        let mut order = Vec::with_capacity(self.core.surfaces.len());
        let mut done: std::collections::BTreeSet<SurfaceId> = Default::default();
        for s in &self.core.surfaces {
            self.visit_for_order(s.id, &mut done, &mut Vec::new(), &mut order);
        }
        order
    }

    /// The per-tick UPDATE order (children first so consumers observe fresh
    /// child content in the same tick). Each surface advances exactly once.
    pub fn tick_order(&self) -> Vec<SurfaceId> {
        self.render_order()
    }

    fn visit_for_order(
        &self,
        surface: SurfaceId,
        done: &mut std::collections::BTreeSet<SurfaceId>,
        stack: &mut Vec<SurfaceId>,
        order: &mut Vec<SurfaceId>,
    ) {
        if done.contains(&surface) || stack.contains(&surface) {
            return;
        }
        stack.push(surface);
        // Dependencies first: the children THIS surface mounts (found through
        // its views' portals), so consumed content renders before consumers.
        if let Some(s) = self.core.surfaces.iter().find(|s| s.id == surface) {
            for view_id in s.views.clone() {
                let Some(view) = self.core.views.iter().find(|v| v.id == view_id) else {
                    continue;
                };
                for portal_id in view.portals.clone() {
                    if let Some(portal) = self.core.portals.iter().find(|p| p.id == portal_id) {
                        self.visit_for_order(portal.desc.child, done, stack, order);
                    }
                }
            }
        }
        stack.pop();
        done.insert(surface);
        order.push(surface);
    }

    // -- dirty tracking -----------------------------------------------------

    /// Content change on `surface`: mark it and every consumer chain dirty.
    pub fn mark_content_dirty(&mut self, surface: SurfaceId) {
        if let Some(i) = self.core.find_surface(surface) {
            self.core.surfaces[i].revision += 1;
        }
        self.dirty.insert(surface);
        for c in self.consumer_chain(surface) {
            self.dirty.insert(c);
        }
    }

    pub fn is_dirty(&self, surface: SurfaceId) -> bool {
        self.dirty.contains(&surface)
    }

    pub fn dirty_surfaces(&self) -> impl Iterator<Item = SurfaceId> + '_ {
        self.dirty.iter().copied()
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    // -- observation --------------------------------------------------------

    /// Canonical state string. Two graphs with equal snapshots are
    /// observationally identical; the contract tests use it to prove a
    /// failed batch changed nothing.
    pub fn snapshot(&self) -> String {
        let mut parts: Vec<String> = self
            .core
            .surfaces
            .iter()
            .map(|s| {
                format!(
                    "s{}|{:.6}x{:.6}|{:?}|rev{}|mounts{}|views{}",
                    s.id.token(),
                    s.desc.logical_size[0],
                    s.desc.logical_size[1],
                    s.desc.content,
                    s.revision,
                    s.mounted_by.len(),
                    s.views.len()
                )
            })
            .collect();
        for v in &self.core.views {
            parts.push(format!(
                "v{}|s{}|win{}|portals{}",
                v.id.token(),
                v.desc.surface.token(),
                v.desc.presents_to_window,
                v.portals.len()
            ));
        }
        for p in &self.core.portals {
            let m = match p.desc.mount {
                MountKind::Flat2d {
                    offset,
                    scale,
                    clip,
                } => format!(
                    "flat({:.6},{:.6}|{:.6},{:.6}|{:.6},{:.6},{:.6},{:.6})",
                    offset[0], offset[1], scale[0], scale[1], clip.x, clip.y, clip.w, clip.h
                ),
                MountKind::Plane3d { center, half } => format!(
                    "plane({:.6},{:.6},{:.6}|{:.6},{:.6})",
                    center[0], center[1], center[2], half[0], half[1]
                ),
            };
            parts.push(format!(
                "p{}|v{}|s{}|{m}|int{}",
                p.id.token(),
                p.desc.parent.token(),
                p.desc.child.token(),
                p.desc.interactive
            ));
        }
        parts.sort();
        parts.join(";")
    }
}

#[derive(Debug, Clone, Copy)]
enum LookupKind {
    Surface,
    View,
    Portal,
}

// ---------------------------------------------------------------------------
// Apply (shared by the validation sim and the committed path)
// ---------------------------------------------------------------------------

fn apply_change(
    core: &mut GraphCore,
    change: &CompositionChange,
    index: usize,
    mints: &mut BatchMints,
    max_depth: usize,
    affected: &mut std::collections::BTreeSet<SurfaceId>,
) -> Result<(), CompositionError> {
    match change {
        CompositionChange::AddSurface { desc } => {
            if !desc.logical_size.iter().all(|v| v.is_finite())
                || desc.logical_size[0] <= 0.0
                || desc.logical_size[1] <= 0.0
            {
                return Err(CompositionError::InvalidSurface {
                    detail: "logical size must be finite and positive".into(),
                });
            }
            let id = SurfaceId(core.alloc.allocate());
            core.surfaces.push(SurfaceState {
                id,
                desc: desc.clone(),
                mounted_by: Vec::new(),
                views: Vec::new(),
                revision: 0,
            });
            mints.created.push((index, Id::Surface(id)));
            Ok(())
        }
        CompositionChange::RemoveSurface(id) => {
            let Some(i) = core.find_surface(*id) else {
                return Err(core.id_error(id.raw(), id.token()));
            };
            let s = &core.surfaces[i];
            if !s.mounted_by.is_empty() || !s.views.is_empty() {
                return Err(CompositionError::SurfaceInUse { token: id.token() });
            }
            let s = core.surfaces.remove(i);
            let _ = core.alloc.free(s.id.raw());
            Ok(())
        }
        CompositionChange::AddView { desc } => {
            let Some(si) = core.find_surface(desc.surface) else {
                return Err(core.id_error(desc.surface.raw(), desc.surface.token()));
            };
            let id = ViewId(core.alloc.allocate());
            core.views.push(ViewState {
                id,
                desc: desc.clone(),
                portals: Vec::new(),
            });
            core.surfaces[si].views.push(id);
            // A new view (especially a window-facing one) changes where this
            // surface's content is observed: its output must be fresh.
            affected.insert(desc.surface);
            mints.created.push((index, Id::View(id)));
            Ok(())
        }
        CompositionChange::RemoveView(id) => {
            let Some(i) = core.find_view(*id) else {
                return Err(core.id_error(id.raw(), id.token()));
            };
            // Cascade: a view's portals cannot outlive it; the CHILD SUBSURFACE
            // and its content stay alive (closing a view never destroys
            // content that is still referenced elsewhere).
            let view = core.views.remove(i);
            affected.insert(view.desc.surface);
            for portal_id in view.portals {
                if let Some(pos) = core.find_portal(portal_id) {
                    let portal = core.portals.remove(pos);
                    if let Some(ci) = core.find_surface(portal.desc.child) {
                        core.surfaces[ci].mounted_by.retain(|p| *p != portal_id);
                    }
                    debug_assert_eq!(portal.desc.parent, view.id);
                    let _ = core.alloc.free(portal.id.raw());
                }
            }
            if let Some(si) = core.find_surface(view.desc.surface) {
                core.surfaces[si].views.retain(|v| *v != view.id);
            }
            let _ = core.alloc.free(view.id.raw());
            Ok(())
        }
        CompositionChange::AddPortal { desc } => {
            desc.mount.validate()?;
            let Some(vi) = core.find_view(desc.parent) else {
                return Err(core.id_error(desc.parent.raw(), desc.parent.token()));
            };
            let Some(ci) = core.find_surface(desc.child) else {
                return Err(core.id_error(desc.child.raw(), desc.child.token()));
            };
            // Same surface consumed by its own view: an immediate cycle.
            let parent_surface = core.views[vi].desc.surface;
            if parent_surface == desc.child {
                return Err(CompositionError::CycleDetected {
                    path: vec![
                        desc.child.token(),
                        parent_surface.token(),
                        desc.child.token(),
                    ],
                });
            }
            let id = PortalId(core.alloc.allocate());
            core.portals.push(PortalState {
                id,
                desc: desc.clone(),
            });
            core.views[vi].portals.push(id);
            core.surfaces[ci].mounted_by.push(id);
            // The parent's composed output gained a layer.
            affected.insert(parent_surface);
            // Indirect cycles / depth budget: validate the graph WITH the new
            // portal present; roll back precisely on rejection.
            if let Err(e) = check_consumer_graph(core, max_depth) {
                core.portals.pop();
                core.views[vi].portals.retain(|p| *p != id);
                core.surfaces[ci].mounted_by.retain(|p| *p != id);
                let _ = core.alloc.free(id.raw());
                return Err(e);
            }
            mints.created.push((index, Id::Portal(id)));
            Ok(())
        }
        CompositionChange::RemovePortal(id) => {
            let Some(i) = core.find_portal(*id) else {
                return Err(core.id_error(id.raw(), id.token()));
            };
            // The parent must repaint WITHOUT this layer (its old pixels must
            // not survive the unmount).
            if let Some(parent) = core
                .views
                .iter()
                .find(|v| v.id == core.portals[i].desc.parent)
                .map(|v| v.desc.surface)
            {
                affected.insert(parent);
            }
            let portal = core.portals.remove(i);
            if let Some(vi) = core.find_view(portal.desc.parent) {
                core.views[vi].portals.retain(|p| *p != portal.id);
            }
            if let Some(ci) = core.find_surface(portal.desc.child) {
                core.surfaces[ci].mounted_by.retain(|p| *p != portal.id);
            }
            let _ = core.alloc.free(portal.id.raw());
            Ok(())
        }
        CompositionChange::SetMount(id, mount) => {
            mount.validate()?;
            let Some(i) = core.find_portal(*id) else {
                return Err(core.id_error(id.raw(), id.token()));
            };
            // Re-mounting changes the layer's placement: the parent's output
            // is stale until it recomposites.
            if let Some(parent) = core
                .views
                .iter()
                .find(|v| v.id == core.portals[i].desc.parent)
                .map(|v| v.desc.surface)
            {
                affected.insert(parent);
            }
            core.portals[i].desc.mount = *mount;
            Ok(())
        }
        CompositionChange::SetSurfaceSize(id, size) => {
            if !size.iter().all(|v| v.is_finite()) || size[0] <= 0.0 || size[1] <= 0.0 {
                return Err(CompositionError::InvalidSurface {
                    detail: "logical size must be finite and positive".into(),
                });
            }
            let Some(i) = core.find_surface(*id) else {
                return Err(core.id_error(id.raw(), id.token()));
            };
            core.surfaces[i].desc.logical_size = *size;
            core.surfaces[i].revision += 1;
            // The resized surface must repaint at the new size, and every
            // consumer must re-map/re-composite it.
            affected.insert(*id);
            Ok(())
        }
    }
}

/// Consumer-graph invariants: acyclic and within the depth budget.
/// Edge direction: a parent view's surface DEPENDS ON its mounted children.
fn check_consumer_graph(core: &GraphCore, budget: usize) -> Result<(), CompositionError> {
    let mut deps: std::collections::BTreeMap<SurfaceId, Vec<SurfaceId>> = Default::default();
    for p in &core.portals {
        let parent_surface = core
            .views
            .iter()
            .find(|v| v.id == p.desc.parent)
            .map(|v| v.desc.surface);
        if let Some(parent) = parent_surface {
            deps.entry(parent).or_default().push(p.desc.child);
        }
    }
    let mut depth_cache: std::collections::BTreeMap<SurfaceId, usize> = Default::default();
    let mut on_stack: std::collections::BTreeSet<SurfaceId> = Default::default();
    let mut path: Vec<SurfaceId> = Vec::new();
    fn visit(
        s: SurfaceId,
        deps: &std::collections::BTreeMap<SurfaceId, Vec<SurfaceId>>,
        depth_cache: &mut std::collections::BTreeMap<SurfaceId, usize>,
        on_stack: &mut std::collections::BTreeSet<SurfaceId>,
        path: &mut Vec<SurfaceId>,
    ) -> Result<usize, Vec<SurfaceId>> {
        if let Some(&d) = depth_cache.get(&s) {
            return Ok(d);
        }
        if on_stack.contains(&s) {
            let start = path.iter().position(|x| *x == s).unwrap_or(0);
            let mut cycle: Vec<SurfaceId> = path[start..].to_vec();
            cycle.push(s);
            return Err(cycle);
        }
        on_stack.insert(s);
        path.push(s);
        let mut depth = 0usize;
        if let Some(children) = deps.get(&s) {
            for c in children.clone() {
                let d = visit(c, deps, depth_cache, on_stack, path)? + 1;
                depth = depth.max(d);
            }
        }
        path.pop();
        on_stack.remove(&s);
        depth_cache.insert(s, depth);
        Ok(depth)
    }
    let keys: Vec<SurfaceId> = deps.keys().copied().collect();
    for s in keys {
        let depth =
            visit(s, &deps, &mut depth_cache, &mut on_stack, &mut path).map_err(|cycle| {
                CompositionError::CycleDetected {
                    path: cycle.iter().map(|s| s.token()).collect(),
                }
            })?;
        if depth > budget {
            return Err(CompositionError::DepthBudgetExceeded { depth, budget });
        }
    }
    Ok(())
}
