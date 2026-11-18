//! PLAN-058 T-04: the UI-side **segment / sweep / image2d** arms of the AGE
//! graphics host provider, aligned with the landed record ABI
//! (`D-981-GFX-PORT-ABI`, auto-lang `5ee4d23c0` receipt `381d7a364`).
//!
//! This module is the **UI058 equivalent face** of `ace_av::age_gfx::AgeGfxHost`:
//! same symbols, same argument order, same `AGE_PORT_FIELDS` record order, same
//! "one host call returns the complete typed record" rule. It is deliberately
//! independent of the pinned AV crate (the `.sdk-source` pin predates the port
//! ABI) and carries its own tiny value enums; when the pin is refreshed the thin
//! adapter is a mechanical `Value` <-> `GfxValue` map — **the algorithms below do
//! not change**.
//!
//! Charging is **not** here: the 981 `AgeGfxSession` owns the shared 32-credit
//! pool (segment 8 / sweep 16 / image 1, sticky `age-gfx.credits`), and the
//! provider only executes a call that already passed the language/session gates.
//!
//! Frozen geometry rules (T-02/T-03 evidence, 981 ABI):
//! * `query2d.segment` uses the **bare** row box (no Minkowski shrink); the
//!   fixture's upward ray meets `target_a` at t=0.25 with point (4, 1.5) and
//!   normal (0, -1).
//! * `query2d.sweep` shrinks the candidate by the **mover's half extents**
//!   (Minkowski), so the mover stops at the wall face (8.6 for the fixture).
//! * Candidate filtering: the mover row, `RenderOnly` rows and (with
//!   `visible_only`) invisible rows never take part; nearest t wins and ties
//!   keep the **lower declaration ordinal** (strict `<`).
//! * A `RenderOnly` row as the mover is the named `age-gfx.render-only-mover`
//!   fault; an out-of-range row is `age-gfx.bad-row`; wrong argument shapes are
//!   `age-gfx.bad-args`; image failures are `age-gfx.image-*`.
use super::{
    document::Layer,
    gfx_host_port::{Cast2D, Image2D, Sweep2D},
};
use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn};
use ace_av_core::contract::{StrHandle, Value};
use std::{
    path::{Component, Path, PathBuf},
    time::Instant,
};

pub const SEGMENT_SYMBOL: &str = "age.query2d.segment.v1";
pub const SWEEP_SYMBOL: &str = "age.query2d.sweep.v1";
pub const IMAGE_SYMBOL: &str = "age.image2d.png.v1";

/// One named provider-side refusal. The AV lane wraps a provider code as
/// `age-gfx.<code>`, so the thin adapter should pass the bare suffix of these
/// names (e.g. `bad-row`) to `AgeGfxFault::new`; the UI-side test surface keeps
/// the full `age-gfx.*` spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GfxFault {
    pub code: &'static str,
    pub message: String,
}

impl GfxFault {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// One typed AGE gfx argument (the AV lane's `Value` subset for these ops).
#[derive(Clone, Debug, PartialEq)]
pub enum GfxArg {
    Bool(bool),
    I32(i32),
    F32(f32),
    Str(String),
}

/// One typed record field returned by a port op.
#[derive(Clone, Debug, PartialEq)]
pub enum GfxValue {
    Bool(bool),
    I32(i32),
    F32(f32),
    Str(String),
}

/// One host return: a scalar op result or a complete port record.
#[derive(Clone, Debug, PartialEq)]
pub enum GfxReturn {
    Scalar(GfxValue),
    Record(Vec<GfxValue>),
}

/// One scene row as the geometry owner sees it: declaration ordinal, world
/// position, half extents, the host row layer and visibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostRow {
    pub ordinal: usize,
    pub world: [f32; 2],
    pub half: [f32; 2],
    pub layer: Layer,
    pub visible: bool,
}

impl HostRow {
    pub fn lo(&self) -> [f32; 2] {
        [self.world[0] - self.half[0], self.world[1] - self.half[1]]
    }
    pub fn hi(&self) -> [f32; 2] {
        [self.world[0] + self.half[0], self.world[1] + self.half[1]]
    }
}

/// One admitted render token (the fixture spawns the player leaf each Tick and
/// reuses the slot, so a token stays live across Ticks).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderToken {
    pub token: i32,
    pub leaf: String,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
}

/// One spawn publish of the Tick (leaf, world position).
#[derive(Clone, Debug, PartialEq)]
pub struct Publish {
    pub token: i32,
    pub leaf: String,
    pub x: f32,
    pub y: f32,
}

/// One `render2d.frame` publish (token, resource name, ordinal, flip).
#[derive(Clone, Debug, PartialEq)]
pub struct FramePublish {
    pub token: i32,
    pub image: String,
    pub ordinal: i32,
    pub flip: i32,
}

/// The neutral provider: rows + host state (input/dt/render tokens) + an asset
/// root for `image2d.png`. Charging is the 981 session's job, never this type's.
pub struct NeutralGfxHost {
    rows: Vec<HostRow>,
    asset_root: PathBuf,
    input: (i32, f32, f32, bool),
    dt: f32,
    tokens: Vec<RenderToken>,
    publishes: Vec<Publish>,
    frames: Vec<FramePublish>,
    last_sweep: Option<Sweep2D>,
    last_segment: Option<Cast2D>,
}

impl NeutralGfxHost {
    pub fn new(rows: Vec<HostRow>, asset_root: impl Into<PathBuf>) -> Self {
        Self {
            rows,
            asset_root: asset_root.into(),
            input: (2, 0.0, 0.0, false),
            dt: 1.0 / 60.0,
            tokens: Vec::new(),
            publishes: Vec::new(),
            frames: Vec::new(),
            last_sweep: None,
            last_segment: None,
        }
    }
    pub fn rows(&self) -> &[HostRow] {
        &self.rows
    }
    /// Host input for the next Tick (`space`, axis x/y, any pressed bit).
    pub fn set_input(&mut self, space: i32, x: f32, y: f32, pressed: bool) {
        self.input = (space, x, y, pressed);
    }
    pub fn set_dt(&mut self, dt: f32) {
        self.dt = dt;
    }
    pub fn dt(&self) -> f32 {
        self.dt
    }
    pub fn row(&self, index: usize) -> Option<&HostRow> {
        self.rows.get(index)
    }
    pub fn tokens(&self) -> &[RenderToken] {
        &self.tokens
    }
    pub fn publishes(&self) -> &[Publish] {
        &self.publishes
    }
    pub fn frames(&self) -> &[FramePublish] {
        &self.frames
    }
    pub fn last_sweep(&self) -> Option<Sweep2D> {
        self.last_sweep
    }
    pub fn last_segment(&self) -> Option<Cast2D> {
        self.last_segment
    }
    /// Forget the per-Tick publishes/results (the runner may keep them per Tick).
    pub fn clear_tick_observations(&mut self) {
        self.publishes.clear();
        self.frames.clear();
        self.last_sweep = None;
        self.last_segment = None;
    }

    fn mover(&self, mover: i32) -> Result<&HostRow, GfxFault> {
        let row = usize::try_from(mover)
            .ok()
            .and_then(|i| self.rows.get(i))
            .ok_or_else(|| {
                GfxFault::new(
                    "age-gfx.bad-row",
                    format!("row {mover} is outside the {}-row scene", self.rows.len()),
                )
            })?;
        if row.layer == Layer::RenderOnly {
            return Err(GfxFault::new(
                "age-gfx.render-only-mover",
                format!("row {mover} is render-only; it can never be a query target"),
            ));
        }
        Ok(row)
    }

    fn candidate(&self, mover: i32, visible_only: bool) -> impl Iterator<Item = &HostRow> {
        self.rows.iter().filter(move |r| {
            r.ordinal as i32 != mover
                && r.layer != Layer::RenderOnly
                && !(visible_only && !r.visible)
        })
    }

    /// `age.query2d.segment.v1`: bare-box slab query, nearest t, ties by lower
    /// ordinal, inside/boundary start at t=0 with no normal.
    pub fn segment(
        &self,
        mover: i32,
        visible_only: bool,
        ax: f32,
        ay: f32,
        bx: f32,
        by: f32,
    ) -> Result<Cast2D, GfxFault> {
        let _ = self.mover(mover)?;
        if ![ax, ay, bx, by].iter().all(|v| v.is_finite()) {
            return Err(GfxFault::new(
                "age-gfx.bad-args",
                "segment endpoints must be finite f32",
            ));
        }
        let (dx, dy) = (bx - ax, by - ay);
        let mut target = -1i32;
        let mut fraction = 2.0f32;
        for r in self.candidate(mover, visible_only) {
            let (lo, hi) = (r.lo(), r.hi());
            let mut enter = 0.0f32;
            let mut leave = 1.0f32;
            if dx != 0.0 {
                let near = (lo[0] - ax) / dx;
                let far = (hi[0] - ax) / dx;
                enter = enter.max(near);
                leave = leave.min(far);
                if enter > leave {
                    continue;
                }
            } else if ax < lo[0] || ax > hi[0] {
                continue;
            }
            if dy != 0.0 {
                let near = (lo[1] - ay) / dy;
                let far = (hi[1] - ay) / dy;
                enter = enter.max(near);
                leave = leave.min(far);
                if enter > leave {
                    continue;
                }
            } else if ay < lo[1] || ay > hi[1] {
                continue;
            }
            // Strict `<`: an equal t keeps the earlier (lower ordinal) hit.
            if enter >= fraction {
                continue;
            }
            target = r.ordinal as i32;
            fraction = enter;
        }
        if target < 0 {
            return Ok(Cast2D {
                hit: false,
                node: -1,
                t: 1.0,
                point_x: 0.0,
                point_y: 0.0,
                normal_x: 0.0,
                normal_y: 0.0,
            });
        }
        let row = self
            .rows
            .iter()
            .find(|r| r.ordinal as i32 == target)
            .expect("target row");
        let (lo, hi) = (row.lo(), row.hi());
        let mut nx = 0.0f32;
        let mut ny = 0.0f32;
        if ax < lo[0] || ax > hi[0] {
            if dx >= 0.0 && (lo[0] - ax) / dx == fraction {
                nx = -1.0;
            } else if dx < 0.0 && (hi[0] - ax) / dx == fraction {
                nx = 1.0;
            }
        } else if ay < lo[1] || ay > hi[1] {
            if dy >= 0.0 && (lo[1] - ay) / dy == fraction {
                ny = -1.0;
            } else if dy < 0.0 && (hi[1] - ay) / dy == fraction {
                ny = 1.0;
            }
        }
        Ok(Cast2D {
            hit: true,
            node: target,
            t: fraction,
            point_x: ax + fraction * dx,
            point_y: ay + fraction * dy,
            normal_x: nx,
            normal_y: ny,
        })
    }

    /// `age.query2d.sweep.v1`: Minkowski-shrunk candidates, nearest t, ties by
    /// lower ordinal, start overlap at t=0.
    pub fn sweep(
        &self,
        mover: i32,
        visible_only: bool,
        dx: f32,
        dy: f32,
    ) -> Result<Sweep2D, GfxFault> {
        let m = self.mover(mover)?;
        if ![dx, dy].iter().all(|v| v.is_finite()) {
            return Err(GfxFault::new(
                "age-gfx.bad-args",
                "sweep displacement must be finite f32",
            ));
        }
        let origin = m.world;
        let mut target = -1i32;
        let mut fraction = 2.0f32;
        for r in self.candidate(mover, visible_only) {
            let (lo, hi) = (r.lo(), r.hi());
            let l = lo[0] - m.half[0];
            let h = hi[0] + m.half[0];
            let yl = lo[1] - m.half[1];
            let yh = hi[1] + m.half[1];
            let mut enter = 0.0f32;
            let mut leave = 1.0f32;
            if dx != 0.0 {
                let near = (l - origin[0]) / dx;
                let far = (h - origin[0]) / dx;
                enter = enter.max(near);
                leave = leave.min(far);
                if enter > leave {
                    continue;
                }
            } else if origin[0] < l || origin[0] > h {
                continue;
            }
            if dy != 0.0 {
                let near = (yl - origin[1]) / dy;
                let far = (yh - origin[1]) / dy;
                enter = enter.max(near);
                leave = leave.min(far);
                if enter > leave {
                    continue;
                }
            } else if origin[1] < yl || origin[1] > yh {
                continue;
            }
            if enter >= fraction {
                continue;
            }
            target = r.ordinal as i32;
            fraction = enter;
        }
        if target < 0 {
            return Ok(Sweep2D {
                hit: false,
                node: -1,
                t: 1.0,
                origin_x: origin[0],
                origin_y: origin[1],
                normal_x: 0.0,
                normal_y: 0.0,
                start_overlap: false,
            });
        }
        let row = self
            .rows
            .iter()
            .find(|r| r.ordinal as i32 == target)
            .expect("target row");
        let (lo, hi) = (row.lo(), row.hi());
        let l = lo[0] - m.half[0];
        let h = hi[0] + m.half[0];
        let yl = lo[1] - m.half[1];
        let yh = hi[1] + m.half[1];
        let overlap = origin[0] >= l && origin[0] <= h && origin[1] >= yl && origin[1] <= yh;
        let mut nx = 0.0f32;
        let mut ny = 0.0f32;
        if !overlap {
            if origin[0] < l || origin[0] > h {
                if dx >= 0.0 && (l - origin[0]) / dx == fraction {
                    nx = -1.0;
                } else if dx < 0.0 && (h - origin[0]) / dx == fraction {
                    nx = 1.0;
                }
            } else if origin[1] < yl || origin[1] > yh {
                if dy >= 0.0 && (yl - origin[1]) / dy == fraction {
                    ny = -1.0;
                } else if dy < 0.0 && (yh - origin[1]) / dy == fraction {
                    ny = 1.0;
                }
            }
        }
        Ok(Sweep2D {
            hit: true,
            node: target,
            t: fraction,
            origin_x: origin[0],
            origin_y: origin[1],
            normal_x: nx,
            normal_y: ny,
            start_overlap: overlap,
        })
    }

    /// `age.image2d.png.v1`: content identity — short name relative to the asset
    /// root; the record is `[sha256, byte_length, width, height]` from the PNG
    /// IHDR. Absolute paths and `..` escape are named refusals.
    pub fn image_png(&self, path: &str) -> Result<Image2D, GfxFault> {
        let rel = Path::new(path);
        if path.is_empty() {
            return Err(GfxFault::new("age-gfx.image-path", "empty image path"));
        }
        for component in rel.components() {
            match component {
                Component::Normal(_) | Component::CurDir => {}
                _ => {
                    return Err(GfxFault::new(
                        "age-gfx.image-path",
                        format!("image path `{path}` must be relative and stay inside the asset root"),
                    ))
                }
            }
        }
        let full = self.asset_root.join(rel);
        let bytes = std::fs::read(&full).map_err(|error| {
            GfxFault::new(
                "age-gfx.image-missing",
                format!("image `{path}`: cannot read `{}`: {error}", full.display()),
            )
        })?;
        if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
            return Err(GfxFault::new(
                "age-gfx.image-bad-png",
                format!("image `{path}` is not a readable PNG (signature/IHDR)"),
            ));
        }
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if width == 0 || height == 0 {
            return Err(GfxFault::new(
                "age-gfx.image-bad-png",
                format!("image `{path}` has an empty IHDR extent"),
            ));
        }
        Ok(Image2D {
            sha256: agp::renderer::capture::sha256_hex(&bytes),
            byte_length: i32::try_from(bytes.len()).map_err(|_| {
                GfxFault::new("age-gfx.image-bad-png", "image exceeds the i32 byte length")
            })?,
            width: i32::try_from(width)
                .map_err(|_| GfxFault::new("age-gfx.image-bad-png", "PNG width exceeds i32"))?,
            height: i32::try_from(height)
                .map_err(|_| GfxFault::new("age-gfx.image-bad-png", "PNG height exceeds i32"))?,
        })
    }

    /// The UI-side symbol dispatcher for the three port ops (typed UI args).
    /// The AV adapter (`impl AgeGfxHost`) maps `Value` onto this entry.
    pub fn dispatch_args(&mut self, symbol: &str, args: &[GfxArg]) -> Result<GfxReturn, GfxFault> {
        match symbol {
            SEGMENT_SYMBOL => {
                let [GfxArg::I32(mover), GfxArg::Bool(visible_only), GfxArg::F32(ax), GfxArg::F32(ay), GfxArg::F32(bx), GfxArg::F32(by)] =
                    args
                else {
                    return Err(GfxFault::new(
                        "age-gfx.bad-args",
                        format!(
                            "{symbol} takes (i32, bool, f32, f32, f32, f32); got {} argument(s)",
                            args.len()
                        ),
                    ));
                };
                let c = self.segment(*mover, *visible_only, *ax, *ay, *bx, *by)?;
                self.last_segment = Some(c);
                Ok(GfxReturn::Record(vec![
                    GfxValue::Bool(c.hit),
                    GfxValue::I32(c.node),
                    GfxValue::F32(c.t),
                    GfxValue::F32(c.point_x),
                    GfxValue::F32(c.point_y),
                    GfxValue::F32(c.normal_x),
                    GfxValue::F32(c.normal_y),
                ]))
            }
            SWEEP_SYMBOL => {
                let [GfxArg::I32(mover), GfxArg::Bool(visible_only), GfxArg::F32(dx), GfxArg::F32(dy)] =
                    args
                else {
                    return Err(GfxFault::new(
                        "age-gfx.bad-args",
                        format!(
                            "{symbol} takes (i32, bool, f32, f32); got {} argument(s)",
                            args.len()
                        ),
                    ));
                };
                let s = self.sweep(*mover, *visible_only, *dx, *dy)?;
                self.last_sweep = Some(s);
                Ok(GfxReturn::Record(vec![
                    GfxValue::Bool(s.hit),
                    GfxValue::I32(s.node),
                    GfxValue::F32(s.t),
                    GfxValue::F32(s.origin_x),
                    GfxValue::F32(s.origin_y),
                    GfxValue::F32(s.normal_x),
                    GfxValue::F32(s.normal_y),
                    GfxValue::Bool(s.start_overlap),
                ]))
            }
            IMAGE_SYMBOL => {
                let [GfxArg::Str(path)] = args else {
                    return Err(GfxFault::new(
                        "age-gfx.bad-args",
                        format!("{symbol} takes (str); got {} argument(s)", args.len()),
                    ));
                };
                let image = self.image_png(path)?;
                Ok(GfxReturn::Record(vec![
                    GfxValue::Str(image.sha256),
                    GfxValue::I32(image.byte_length),
                    GfxValue::I32(image.width),
                    GfxValue::I32(image.height),
                ]))
            }
            _ => Err(GfxFault::new(
                "age-gfx.unknown-op",
                format!("`{symbol}` is not one of the three port-result ops"),
            )),
        }
    }
}

/// Diagnostic: one dispatch round-trip timed (acceptance reports only).
pub fn timed_dispatch(
    host: &mut NeutralGfxHost,
    symbol: &str,
    args: &[GfxArg],
) -> (Result<GfxReturn, GfxFault>, u128) {
    let start = Instant::now();
    let out = host.dispatch_args(symbol, args);
    (out, start.elapsed().as_micros())
}

// ---------------------------------------------------------------- AV adapter --
//
// `impl AgeGfxHost` is the thin bridge between the 981 typed dispatch and the
// neutral arms above: `Value` -> `GfxArg` on the way in, the AGE_PORT_FIELDS
// record back as `AgeGfxReturn::Record` on the way out. Charging is the 981
// `AgeGfxSession`'s job (it wraps this provider), so nothing here meters.

fn av_str(value: &Value) -> Option<String> {
    match value {
        Value::Str(handle) => Some(String::from_utf8_lossy(handle.bytes()).into_owned()),
        // The AV lane passes literal resource/leaf names as borrowed views; the
        // fixture's names are ASCII, so scalar indices equal byte offsets.
        Value::StrView(view) => {
            let bytes = match &view.src {
                ace_av_core::contract::StrViewSrc::Share(handle) => handle.bytes().to_vec(),
                ace_av_core::contract::StrViewSrc::Static { buf } => buf.to_vec(),
            };
            let (start, end) = (view.start as usize, view.end as usize);
            bytes
                .get(start..end)
                .map(|slice| String::from_utf8_lossy(slice).into_owned())
        }
        _ => None,
    }
}

fn av_arg(value: &Value) -> Result<GfxArg, AgeGfxFault> {
    match value {
        Value::I32(v) => Ok(GfxArg::I32(*v)),
        Value::F32(v) => Ok(GfxArg::F32(*v)),
        Value::Bool(v) => Ok(GfxArg::Bool(*v)),
        Value::Str(_) | Value::StrView(_) => av_str(value)
            .map(GfxArg::Str)
            .ok_or_else(|| AgeGfxFault::new("bad-args", "malformed string argument")),
        other => Err(AgeGfxFault::new(
            "bad-args",
            format!("unsupported argument value {other:?}"),
        )),
    }
}

fn av_value(value: GfxValue) -> Value {
    match value {
        GfxValue::Bool(v) => Value::Bool(v),
        GfxValue::I32(v) => Value::I32(v),
        GfxValue::F32(v) => Value::F32(v),
        GfxValue::Str(v) => Value::Str(StrHandle::detached(v.into_bytes().into_boxed_slice())),
    }
}

/// The AV lane formats a provider code as `age-gfx.<code>`, so the UI-side
/// `age-gfx.` prefix is stripped here exactly once.
fn av_fault(fault: GfxFault) -> AgeGfxFault {
    AgeGfxFault::new(
        fault.code.strip_prefix("age-gfx.").unwrap_or(fault.code),
        fault.message,
    )
}

fn scalar(value: GfxValue) -> Result<AgeGfxReturn, AgeGfxFault> {
    Ok(AgeGfxReturn::Scalar(av_value(value)))
}

impl NeutralGfxHost {
    fn row_index(&self, row: i32) -> Result<usize, GfxFault> {
        usize::try_from(row)
            .ok()
            .filter(|i| *i < self.rows.len())
            .ok_or_else(|| {
                GfxFault::new(
                    "age-gfx.bad-row",
                    format!("row {row} is outside the {}-row scene", self.rows.len()),
                )
            })
    }
    fn token_mut(&mut self, token: i32) -> Result<&mut RenderToken, GfxFault> {
        self.tokens
            .iter_mut()
            .find(|t| t.token == token && t.alive)
            .ok_or_else(|| {
                GfxFault::new(
                    "age-gfx.bad-token",
                    format!("render token {token} is unknown or dead"),
                )
            })
    }
}
impl AgeGfxHost for NeutralGfxHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        // Every argument arrives as a typed `Value`; map once to the UI-side
        // `GfxArg` so both the three record ops and the 13 scalar ops share one
        // argument surface.
        let mapped: Result<Vec<GfxArg>, AgeGfxFault> = args.iter().map(av_arg).collect();
        let mapped = mapped?;
        // The three port-result ops reuse the neutral algorithms and return the
        // complete AGE_PORT_FIELDS-ordered record.
        if matches!(symbol, SEGMENT_SYMBOL | SWEEP_SYMBOL | IMAGE_SYMBOL) {
            return match self.dispatch_args(symbol, &mapped) {
                Ok(GfxReturn::Record(fields)) => {
                    Ok(AgeGfxReturn::Record(fields.into_iter().map(av_value).collect()))
                }
                Ok(GfxReturn::Scalar(value)) => Ok(AgeGfxReturn::Scalar(av_value(value))),
                Err(fault) => Err(av_fault(fault)),
            };
        }
        // The 13 scalar-result ops of IF-981-GFX-HOST-PORT.
        match (symbol, mapped.as_slice()) {
            ("age.input.space.v1", []) => scalar(GfxValue::I32(self.input.0)),
            ("age.input.x.v1", []) => scalar(GfxValue::F32(self.input.1)),
            ("age.input.y.v1", []) => scalar(GfxValue::F32(self.input.2)),
            ("age.input.pressed.v1", [GfxArg::I32(action)]) => {
                if !(0..=3).contains(action) {
                    return Err(AgeGfxFault::new(
                        "bad-args",
                        format!("input.pressed action {action} is outside 0..=3"),
                    ));
                }
                scalar(GfxValue::Bool(self.input.3))
            }
            ("age.tick.dt.v1", []) => scalar(GfxValue::F32(self.dt)),
            ("age.row.place.v1", [GfxArg::I32(row), GfxArg::F32(x), GfxArg::F32(y)]) => {
                let slot = self.row_index(*row).map_err(av_fault)?;
                self.rows[slot].world = [*x, *y];
                scalar(GfxValue::I32(1))
            }
            ("age.row.shift.v1", [GfxArg::I32(row), GfxArg::F32(dx), GfxArg::F32(dy)]) => {
                let slot = self.row_index(*row).map_err(av_fault)?;
                self.rows[slot].world[0] += *dx;
                self.rows[slot].world[1] += *dy;
                scalar(GfxValue::I32(1))
            }
            ("age.row.show.v1", [GfxArg::I32(row), GfxArg::Bool(visible)]) => {
                let slot = self.row_index(*row).map_err(av_fault)?;
                self.rows[slot].visible = *visible;
                scalar(GfxValue::I32(1))
            }
            ("age.row.world.v1", [GfxArg::I32(row), GfxArg::I32(axis)]) => {
                let slot = self.row_index(*row).map_err(av_fault)?;
                let value = match axis {
                    0 => self.rows[slot].world[0],
                    1 => self.rows[slot].world[1],
                    _ => {
                        return Err(AgeGfxFault::new(
                            "bad-args",
                            format!("row.world axis {axis} is outside 0..=1"),
                        ))
                    }
                };
                scalar(GfxValue::F32(value))
            }
            ("age.row.local.v1", [GfxArg::I32(row), GfxArg::I32(axis)]) => {
                let slot = self.row_index(*row).map_err(av_fault)?;
                let value = match axis {
                    0 => self.rows[slot].world[0],
                    1 => self.rows[slot].world[1],
                    _ => {
                        return Err(AgeGfxFault::new(
                            "bad-args",
                            format!("row.local axis {axis} is outside 0..=1"),
                        ))
                    }
                };
                scalar(GfxValue::F32(value))
            }
            ("age.render2d.spawn.v1", [GfxArg::Str(leaf), GfxArg::F32(x), GfxArg::F32(y)]) => {
                // Reuse the live token of the same leaf (022 respawn semantics).
                if let Some(token) = self.tokens.iter_mut().find(|t| t.alive && t.leaf == *leaf)
                {
                    token.x = *x;
                    token.y = *y;
                    let id = token.token;
                    self.publishes.push(Publish {
                        token: id,
                        leaf: leaf.clone(),
                        x: *x,
                        y: *y,
                    });
                    return scalar(GfxValue::I32(id));
                }
                let token = self.tokens.len() as i32 + 1;
                self.tokens.push(RenderToken {
                    token,
                    leaf: leaf.clone(),
                    x: *x,
                    y: *y,
                    alive: true,
                });
                self.publishes.push(Publish {
                    token,
                    leaf: leaf.clone(),
                    x: *x,
                    y: *y,
                });
                scalar(GfxValue::I32(token))
            }
            ("age.render2d.despawn.v1", [GfxArg::I32(token)]) => {
                self.token_mut(*token).map_err(av_fault)?.alive = false;
                scalar(GfxValue::I32(1))
            }
            (
                "age.render2d.frame.v1",
                [GfxArg::I32(token), GfxArg::Str(image), GfxArg::I32(ordinal), GfxArg::I32(flip)],
            ) => {
                if *ordinal < 0 || !(0..=1).contains(flip) {
                    return Err(AgeGfxFault::new(
                        "bad-args",
                        format!("render2d.frame ordinal {ordinal}/flip {flip} is out of domain"),
                    ));
                }
                let id = self.token_mut(*token).map_err(av_fault)?.token;
                self.frames.push(FramePublish {
                    token: id,
                    image: image.clone(),
                    ordinal: *ordinal,
                    flip: *flip,
                });
                scalar(GfxValue::I32(id))
            }
            (symbol, _) => Err(AgeGfxFault::new(
                "unknown-op",
                format!(
                    "`{symbol}` is not a graphics host op with {} argument(s)",
                    mapped.len()
                ),
            )),
        }
    }
}
