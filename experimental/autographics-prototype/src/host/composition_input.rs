//! Host input wiring for composed surfaces (PLAN-830 T-05/T-06).
//!
//! Turns real window events into per-surface delivery:
//!
//!   window physical px → top view content px → (mount chain) → child local px
//!
//! 2D mounts invert their affine transform; 3D plane mounts go through the
//! SAME camera the renderer uses (`probe_camera` for the probe scenes) so a
//! click on a plane lands on the child pixel that the plane samples.
//!
//! Responsibilities:
//! - `pointer_*`: hit-test + capture via `core::composition::input`.
//! - `key_*`: deliver keyboard/IME events to the FOCUSED surface only.
//! - `ime_anchor`: compute the window position of the focused child's text
//!   caret region (child local → mount chain → window), which the host then
//!   hands to the OS IME (`ImmSetCompositionWindow`) — the candidate window
//!   must sit at the CURSOR, not at the offscreen texture origin.
//! - `cancel_all`: window focus loss / destroy → cancel capture and drop
//!   focus (no dangling edit state, no late release to a new object).

use crate::core::composition::input::{CancelReason, Hit, InputOutcome, InputState};
use crate::core::composition::{CompositionGraph, MountKind, SurfaceId, ViewId};
use crate::host::surface_runtime::{SurfaceContent, SurfaceRuntime};
use glam::{Mat4, Vec3, Vec4};

/// Ray/plane resolution for 3D mounts, using the parent scene's camera.
pub struct PlaneMapper {
    /// (parent surface, camera view_proj, viewport px)
    pub parents: std::collections::HashMap<SurfaceId, (Mat4, [f32; 2])>,
}

impl PlaneMapper {
    pub fn new() -> Self {
        PlaneMapper {
            parents: Default::default(),
        }
    }

    /// Register the camera of a 3D surface (called when scenes are built).
    pub fn register(&mut self, parent: SurfaceId, view_proj: Mat4, viewport: [f32; 2]) {
        self.parents.insert(parent, (view_proj, viewport));
    }

    /// Screen point (parent content px) → child UV on a plane mount.
    pub fn hit(&self, parent: SurfaceId, mount: &MountKind, point: [f32; 2]) -> Option<[f32; 2]> {
        let (view_proj, viewport) = self.parents.get(&parent)?;
        let MountKind::Plane3d { center, half } = mount else {
            return None;
        };
        ray_to_plane(
            point,
            *view_proj,
            Vec3::from_array(*center),
            [half[0], half[1]],
            *viewport,
        )
    }

    /// Child UV → parent content px (forward projection; used for IME
    /// anchors and OS click placement).
    pub fn project(&self, parent: SurfaceId, mount: &MountKind, uv: [f32; 2]) -> Option<[f32; 2]> {
        let (view_proj, viewport) = self.parents.get(&parent)?;
        let MountKind::Plane3d { center, half } = mount else {
            return None;
        };
        let world = plane_uv_to_world(Vec3::from_array(*center), [half[0], half[1]], uv);
        let clip = *view_proj * Vec4::new(world.x, world.y, world.z, 1.0);
        if clip.w.abs() < 1e-6 {
            return None;
        }
        let ndc = [clip.x / clip.w, clip.y / clip.w];
        Some([
            (ndc[0] * 0.5 + 0.5) * viewport[0],
            (1.0 - (ndc[1] * 0.5 + 0.5)) * viewport[1],
        ])
    }
}

fn plane_local_to_world(
    center: Vec3,
    half: [f32; 2],
    local: [f32; 2],
    child_size: [f32; 2],
) -> Vec3 {
    let u = local[0] / child_size[0].max(1.0);
    let v = local[1] / child_size[1].max(1.0);
    plane_uv_to_world(center, half, [u, v])
}

fn plane_uv_to_world(center: Vec3, half: [f32; 2], uv: [f32; 2]) -> Vec3 {
    center
        + Vec3::new(
            (uv[0] * 2.0 - 1.0) * half[0],
            (1.0 - uv[1] * 2.0) * half[1],
            0.0,
        )
}

fn world_to_plane_uv(center: Vec3, half: [f32; 2], p: Vec3) -> Option<[f32; 2]> {
    let d = p - center;
    let u = d.x / (2.0 * half[0]) + 0.5;
    let v = 0.5 - d.y / (2.0 * half[1]);
    if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
        Some([u, v])
    } else {
        None
    }
}

/// Perspective ray (from the parent content pixel) against an axis plane.
pub fn ray_to_plane(
    point: [f32; 2],
    view_proj: Mat4,
    center: Vec3,
    half: [f32; 2],
    viewport: [f32; 2],
) -> Option<[f32; 2]> {
    let ndc_x = 2.0 * point[0] / viewport[0].max(1.0) - 1.0;
    let ndc_y = 1.0 - 2.0 * point[1] / viewport[1].max(1.0);
    let inv = view_proj.inverse();
    let unproject = |z: f32| {
        let v = inv * Vec4::new(ndc_x, ndc_y, z, 1.0);
        Vec3::new(v.x / v.w, v.y / v.w, v.z / v.w)
    };
    let near = unproject(-1.0);
    let far = unproject(1.0);
    let dir = (far - near).normalize();
    let denom = dir.z; // plane normal = +Z
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (center.z - near.z) / denom;
    if t <= 0.0 {
        return None;
    }
    world_to_plane_uv(center, half, near + dir * t)
}

/// Owned plane-hit closure (see `CompositionInput::plane`).
pub struct PlaneHitClosure {
    parents: std::collections::HashMap<SurfaceId, (Mat4, [f32; 2])>,
}

impl PlaneHitClosure {
    /// Resolve a plane mount with the camera of the surface that OWNS the
    /// mount's view. A mount whose parent camera is not registered is a miss —
    /// never a hit borrowed from another 3D parent (review r1 F04).
    fn hit(&self, parent: SurfaceId, mount: &MountKind, point: [f32; 2]) -> Option<[f32; 2]> {
        let (vp, viewport) = self.parents.get(&parent)?;
        let MountKind::Plane3d { center, half } = mount else {
            return None;
        };
        ray_to_plane(
            point,
            *vp,
            Vec3::from_array(*center),
            [half[0], half[1]],
            *viewport,
        )
    }
}

/// The host-side input router over one runtime.
pub struct CompositionInput {
    pub state: InputState,
    pub mapper: PlaneMapper,
    /// Window physical size (px) and the top surface's logical size.
    pub window_size: [f32; 2],
    pub top_logical: [f32; 2],
    /// Last pointer position in window px (for re-dispatch after resize).
    pub last_window_px: Option<[f32; 2]>,
    /// Explicit window-facing view (multi-scene graphs otherwise resolve to
    /// the FIRST window view, which may belong to another composition).
    pub top_view: Option<ViewId>,
}

impl Default for CompositionInput {
    fn default() -> Self {
        Self::new([1.0, 1.0], [1.0, 1.0])
    }
}

impl CompositionInput {
    pub fn new(window_size: [f32; 2], top_logical: [f32; 2]) -> Self {
        CompositionInput {
            state: InputState::new(),
            mapper: PlaneMapper::new(),
            window_size,
            top_logical,
            last_window_px: None,
            top_view: None,
        }
    }

    pub fn set_sizes(&mut self, window_size: [f32; 2], top_logical: [f32; 2]) {
        self.window_size = window_size;
        self.top_logical = top_logical;
    }

    /// Window physical px → top view content px.
    pub fn window_to_top(&self, px: [f32; 2]) -> [f32; 2] {
        [
            px[0] * (self.top_logical[0] / self.window_size[0].max(1.0)),
            px[1] * (self.top_logical[1] / self.window_size[1].max(1.0)),
        ]
    }

    /// Top view content px → window physical px (inverse).
    pub fn top_to_window(&self, local: [f32; 2]) -> [f32; 2] {
        [
            local[0] * (self.window_size[0] / self.top_logical[0].max(1.0)),
            local[1] * (self.window_size[1] / self.top_logical[1].max(1.0)),
        ]
    }

    fn top_view(&self, graph: &CompositionGraph) -> Option<ViewId> {
        self.top_view.or_else(|| {
            graph
                .views()
                .iter()
                .find(|v| v.desc.presents_to_window)
                .map(|v| v.id)
        })
    }

    /// Build the plane closure bound to this router's mapper. The closure
    /// needs the PARENT surface, which the core traversal supplies through
    /// the path's `host` field — so we resolve each portal's parent at call
    /// time by looking the portal up in the graph.
    fn plane(&self, _graph: &CompositionGraph) -> PlaneHitClosure {
        // Owned copy of the parent cameras so no borrow of `self` survives
        // into the mutable routing call (the map holds ≤ a few entries).
        PlaneHitClosure {
            parents: self.mapper.parents.clone(),
        }
    }

    pub fn pointer_moved(
        &mut self,
        graph: &CompositionGraph,
        window_px: [f32; 2],
    ) -> Option<InputOutcome> {
        self.last_window_px = Some(window_px);
        let top_view = self.top_view(graph)?;
        let local = self.window_to_top(window_px);
        let planes = self.plane(graph);
        self.state
            .moved(graph, top_view, local, &|m, p, host| planes.hit(host, m, p))
            .ok()
    }

    pub fn pointer_pressed(
        &mut self,
        graph: &CompositionGraph,
        window_px: [f32; 2],
    ) -> Option<InputOutcome> {
        self.last_window_px = Some(window_px);
        let top_view = self.top_view(graph)?;
        let local = self.window_to_top(window_px);
        let planes = self.plane(graph);
        self.state
            .press(graph, top_view, local, &|m, p, host| planes.hit(host, m, p))
            .ok()
    }

    pub fn pointer_released(&mut self) -> Option<SurfaceId> {
        self.state.release()
    }

    /// Window focus lost / pointer left the window / target destroyed:
    /// clear capture AND keyboard focus, so no stale edit/IME state or late
    /// release survives. Returns the surface whose focus was dropped, which
    /// the host must unfocus (iced) and close IME for (review r1 F06).
    pub fn cancel_all(&mut self, reason: CancelReason) -> Option<SurfaceId> {
        self.last_window_px = None;
        self.state.cancel(reason)
    }

    /// Window focus lost / leave: cancel capture and drop focus.
    pub fn pointer_left_window(&mut self) -> Option<SurfaceId> {
        self.cancel_all(CancelReason::PointerLeftWindow)
    }

    /// Real window deactivation (`WindowEvent::Focused(false)`).
    pub fn window_focus_lost(&mut self) -> Option<SurfaceId> {
        self.cancel_all(CancelReason::WindowFocusLost)
    }

    /// Structural change happened: drop a dead capture/focus path and report
    /// the surface whose focus died (host releases its widget state).
    pub fn revalidate(&mut self, graph: &CompositionGraph) -> Option<SurfaceId> {
        self.state.revalidate_focus(graph)
    }

    pub fn keyboard_target(&self) -> Option<SurfaceId> {
        self.state.keyboard_target()
    }

    /// Consume the last press-induced focus migration. The host MUST release
    /// the OLD surface's real widget/IME state (review r2 F02). The runtime
    /// also enforces this at frame time, so a caller that forgets cannot
    /// leave two focused controls behind.
    pub fn take_focus_transfer(
        &mut self,
    ) -> Option<crate::core::composition::input::FocusTransfer> {
        self.state.take_focus_transfer()
    }

    /// The REAL caret of the focused iced widget, in the surface's LOCAL
    /// logical px — read from iced's own `InputMethod::Enabled.cursor`, which
    /// the widget computes from its layout, cursor index, selection and
    /// scroll (review r2 F03). `None` when nothing is being edited.
    pub fn ime_caret_local(rt: &SurfaceRuntime, surface: SurfaceId) -> Option<[f32; 2]> {
        rt.ime_caret_local(surface)
    }

    /// Project the ACTUAL caret RECTANGLE of `surface` to window px through
    /// the active mount chain. The gate compares the OS anchor against THIS
    /// rect, not against the whole plane or window.
    pub fn ime_caret_window_rect(
        &self,
        graph: &CompositionGraph,
        rt: &SurfaceRuntime,
        surface: SurfaceId,
    ) -> Option<[f32; 4]> {
        let caret = rt.ime_caret_rect(surface)?;
        let tl = self.anchor_to_window(graph, rt, surface, [caret.x, caret.y])?;
        let br = self.anchor_to_window(
            graph,
            rt,
            surface,
            [caret.x + caret.width, caret.y + caret.height],
        )?;
        Some([
            tl[0].min(br[0]),
            tl[1].min(br[1]),
            tl[0].max(br[0]),
            tl[1].max(br[1]),
        ])
    }

    /// Window position of the ACTUAL caret anchor (top-left of the real caret
    /// rect) for the OS candidate window.
    pub fn ime_anchor_window(
        &self,
        graph: &CompositionGraph,
        rt: &SurfaceRuntime,
        surface: SurfaceId,
    ) -> Option<[f32; 2]> {
        let local = rt.ime_caret_local(surface)?;
        self.anchor_to_window(graph, rt, surface, local)
    }

    /// The active mount path for `surface`: focus first, then capture, then
    /// the top surface itself (empty path), then a deterministic search.
    /// Anchors must follow the mount the user is actually editing
    /// (review r1 F07 — the old BFS always picked the FIRST mount).
    fn active_path(
        &self,
        graph: &CompositionGraph,
        surface: SurfaceId,
    ) -> Option<Vec<crate::core::composition::input::HitStep>> {
        if let Some(focus) = &self.state.focus {
            if focus.hit.surface == surface {
                return Some(focus.hit.path.clone());
            }
        }
        if let Some(capture) = &self.state.capture {
            if capture.hit.surface == surface {
                return Some(capture.hit.path.clone());
            }
        }
        if let Some(top) = self.top_view(graph) {
            if graph.view(top).ok().map(|v| v.desc.surface) == Some(surface) {
                return Some(Vec::new());
            }
            return find_path(graph, top, surface);
        }
        None
    }

    /// Does the event target a real iced surface? (The adapter only exists
    /// for iced content; keys are dropped for other kinds.)
    pub fn is_iced(rt: &SurfaceRuntime, surface: SurfaceId) -> bool {
        matches!(rt.content(surface), Some(SurfaceContent::Iced(_)))
    }

    /// Compute the WINDOW position for the IME candidate/anchor of a child
    /// point (usually the text caret) by walking the ACTIVE mount chain
    /// forward. The path comes from the focused (or captured) hit, so a
    /// surface mounted twice anchors at the mount the user is editing —
    /// including the root surface (empty path).
    pub fn anchor_to_window(
        &self,
        graph: &CompositionGraph,
        rt: &SurfaceRuntime,
        surface: SurfaceId,
        local: [f32; 2],
    ) -> Option<[f32; 2]> {
        let path = self.active_path(graph, surface)?;
        let mut cur = local;
        // Walk backwards: each step's parent camera maps the child UV into
        // parent px.
        for step in path.iter().rev() {
            let portal = graph.portal(step.portal).ok()?;
            let child_size = match graph.surface(portal.desc.child) {
                Ok(s) => s.desc.logical_size,
                Err(_) => {
                    let t = rt.target(portal.desc.child)?;
                    [t.width as f32, t.height as f32]
                }
            };
            cur = match portal.desc.mount {
                MountKind::Flat2d { offset, scale, .. } => {
                    [offset[0] + cur[0] * scale[0], offset[1] + cur[1] * scale[1]]
                }
                MountKind::Plane3d { .. } => {
                    let uv = [
                        cur[0] / child_size[0].max(1.0),
                        cur[1] / child_size[1].max(1.0),
                    ];
                    let parent = step.host;
                    self.mapper.project(parent, &portal.desc.mount, uv)?
                }
            };
        }
        Some(self.top_to_window(cur))
    }
}

/// The portal path from the given window-facing view down to `surface`
/// (outermost first). BFS over the mount DAG; fallback only — the ACTIVE
/// focus/capture path takes precedence for anchors.
fn find_path(
    graph: &CompositionGraph,
    top_view: ViewId,
    surface: SurfaceId,
) -> Option<Vec<crate::core::composition::input::HitStep>> {
    use crate::core::composition::input::HitStep;
    let top = graph.view(top_view).ok()?;
    let mut queue: std::collections::VecDeque<(ViewId, SurfaceId, Vec<HitStep>)> =
        Default::default();
    queue.push_back((top.id, top.desc.surface, Vec::new()));
    let mut seen: std::collections::BTreeSet<SurfaceId> = Default::default();
    while let Some((view, host, path)) = queue.pop_front() {
        if host == surface && !path.is_empty() {
            return Some(path);
        }
        if !seen.insert(host) {
            continue;
        }
        let v = graph.view(view).ok()?;
        for portal_id in v.portals.clone() {
            if let Ok(portal) = graph.portal(portal_id) {
                let mut p = path.clone();
                p.push(HitStep {
                    view: view,
                    portal: portal_id,
                    host,
                });
                if portal.desc.child == surface {
                    return Some(p);
                }
                if let Some(child_view) = graph
                    .surface(portal.desc.child)
                    .ok()
                    .and_then(|s| s.views.first().copied())
                {
                    queue.push_back((child_view, portal.desc.child, p));
                }
            }
        }
    }
    None
}

/// Set the Windows IME composition window to `screen_px` (physical pixels).
/// No-op off Windows; returns whether the OS accepted the call.
pub fn set_ime_anchor_screen(window_title: &str, screen_px: [f32; 2]) -> bool {
    #[cfg(windows)]
    {
        let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class ImeAnchor {{
    [StructLayout(LayoutKind.Sequential)]
    public struct COMPOSITIONFORM {{
        public int dwStyle;
        public int x;
        public int y;
        public int left;
        public int top;
        public int right;
        public int bottom;
    }}
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("imm32.dll")] public static extern IntPtr ImmGetContext(IntPtr h);
    [DllImport("imm32.dll")] public static extern bool ImmSetCompositionWindow(IntPtr imc, ref COMPOSITIONFORM form);
    [DllImport("imm32.dll")] public static extern bool ImmReleaseContext(IntPtr h, IntPtr imc);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    public static bool Anchor(string title, int x, int y) {{
        SetProcessDPIAware();
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) return false;
        IntPtr imc = ImmGetContext(h);
        if (imc == IntPtr.Zero) return false;
        COMPOSITIONFORM cf;
        cf.dwStyle = 0x0002; // CFS_POINT
        cf.x = x; cf.y = y; cf.left = 0; cf.top = 0; cf.right = 0; cf.bottom = 0;
        bool ok = ImmSetCompositionWindow(imc, ref cf);
        ImmReleaseContext(h, imc);
        return ok;
    }}
}}
"@
if ([ImeAnchor]::Anchor("TITLE", X, Y)) { exit 0 } else { exit 1 }
"#
        .replace("TITLE", window_title)
        .replace("X", &(screen_px[0].round() as i64).to_string())
        .replace("Y", &(screen_px[1].round() as i64).to_string());
        return std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }
    #[cfg(not(windows))]
    {
        let _ = (window_title, screen_px);
        false
    }
}

// Keeps `Hit` reachable for callers that want the full path (evidence).
pub use crate::core::composition::input::Hit as SurfaceHit;

#[allow(unused)]
fn _hit_is_used(h: &Hit) -> [f32; 2] {
    h.local
}

/// Convert a window CLIENT px into absolute screen px (physical), for OS IME
/// anchoring. Uses the real window handle when available.
pub fn client_to_screen(window_title: &str, client_px: [f32; 2]) -> Option<[f32; 2]> {
    #[cfg(windows)]
    {
        let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class C2S {
    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    public static void Go(string title, int x, int y) {
        SetProcessDPIAware();
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) { Environment.Exit(1); }
        POINT p = new POINT { X = x, Y = y };
        if (!ClientToScreen(h, ref p)) { Environment.Exit(1); }
        Console.WriteLine(p.X + "," + p.Y);
    }
}
"@
[C2S]::Go("TITLE", X, Y)
"#
        .replace("TITLE", window_title)
        .replace("X", &(client_px[0].round() as i64).to_string())
        .replace("Y", &(client_px[1].round() as i64).to_string());
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let mut it = text.trim().split(',');
        let x: f32 = it.next()?.trim().parse().ok()?;
        let y: f32 = it.next()?.trim().parse().ok()?;
        return Some([x, y]);
    }
    #[cfg(not(windows))]
    {
        let _ = (window_title, client_px);
        None
    }
}

// Silences an unused-import warning while keeping the plane helper reachable.
#[allow(unused)]
fn _plane_local(center: Vec3, half: [f32; 2], local: [f32; 2], size: [f32; 2]) -> Vec3 {
    plane_local_to_world(center, half, local, size)
}
