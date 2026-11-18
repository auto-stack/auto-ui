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

/// The neutral provider: rows + an asset root for `image2d.png`.
pub struct NeutralGfxHost {
    rows: Vec<HostRow>,
    asset_root: PathBuf,
}

impl NeutralGfxHost {
    pub fn new(rows: Vec<HostRow>, asset_root: impl Into<PathBuf>) -> Self {
        Self {
            rows,
            asset_root: asset_root.into(),
        }
    }
    pub fn rows(&self) -> &[HostRow] {
        &self.rows
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

    /// The symbol dispatcher the thin AV adapter will call: three port ops with
    /// their exact `AGE_PORT_FIELDS` record order, unknown symbols refused.
    pub fn dispatch(&mut self, symbol: &str, args: &[GfxArg]) -> Result<GfxReturn, GfxFault> {
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
    let out = host.dispatch(symbol, args);
    (out, start.elapsed().as_micros())
}
