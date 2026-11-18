//! PLAN-058 T-04 **prep**: the UI-side CANDIDATE host provider contract for the
//! pending `IF-981-GFX-HOST-PORT` handoff.
//!
//! This is **not** the AGE host-port ABI and carries no version promise. It is
//! the UI058 shape the driver adapter will be written against once 981 lands the
//! typed host port: the 17 frozen service spellings (`query2d.segment/sweep`,
//! `input.space/x/y/pressed`, `tick.dt`, `row.place/shift/show/world/local`,
//! `render2d.spawn/despawn/frame`, `image2d.png`, `frame2d.at`) with their typed
//! results and the co-frozen credit table (32 per world callback, segment 8,
//! sweep 16, other services 1, port field reads 0).
//!
//! Frozen credit semantics (PLAN-058 r5/r6 §5, measured by T-02/T-03):
//! the shared pool is charged before the guard, a charge that would push the
//! callback total past 32 is the named fault 10 with zero effect, every later
//! charged call in the same callback is still refused (sticky), and the meter
//! resets at the next callback while the fault itself is Run-terminal.
use super::document::Layer;

/// The shared per-world-callback credit pool (T-02/T-03 co-frozen).
pub const CREDIT_BUDGET: i32 = 32;
pub const CREDIT_SEGMENT: i32 = 8;
pub const CREDIT_SWEEP: i32 = 16;
pub const CREDIT_BASE: i32 = 1;
/// The named budget refusal code.
pub const FAULT_QUERY_BUDGET: i32 = 10;

/// One typed `query2d.segment` result (`Cast2D` port fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cast2D {
    pub hit: bool,
    pub node: i32,
    pub t: f32,
    pub point_x: f32,
    pub point_y: f32,
    pub normal_x: f32,
    pub normal_y: f32,
}

/// One typed `query2d.sweep` result (`Sweep2D` port fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sweep2D {
    pub hit: bool,
    pub node: i32,
    pub t: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub normal_x: f32,
    pub normal_y: f32,
    pub start_overlap: bool,
}

/// `image2d.png` identity port: content identity is host-supplied, never a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image2D {
    pub identity: String,
}

/// `frame2d.at` value-construction port: an integer pixel rect inside an image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame2D {
    /// The `Image2D` port this rect belongs to (identity, not a name).
    pub image: Image2D,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// The candidate per-callback meter. `credits` is the attempt counter (a refused
/// charge is not refunded — T-03 measured 40 -> 41); `fault` is Run-terminal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GfxMeter {
    credits: i32,
    fault: Option<i32>,
}

impl GfxMeter {
    /// Start a new world callback: the pool resets, the fault does not.
    pub fn begin_tick(&mut self) {
        self.credits = 0;
    }
    /// A fresh Run clears both the pool and the fault.
    pub fn reset_run(&mut self) {
        self.credits = 0;
        self.fault = None;
    }
    pub fn credits(&self) -> i32 {
        self.credits
    }
    pub fn fault(&self) -> Option<i32> {
        self.fault
    }
    /// Charge one service call. Returns `Err(10)` for the named budget refusal
    /// without performing the effect; the attempt counter still advances.
    pub fn charge(&mut self, cost: i32) -> Result<(), i32> {
        self.credits += cost;
        if self.credits > CREDIT_BUDGET {
            self.fault.get_or_insert(FAULT_QUERY_BUDGET);
        }
        match self.fault {
            Some(code) => Err(code),
            None => Ok(()),
        }
    }
    /// Port field reads are free and never touch the meter.
    pub fn read_field<T>(&self, value: T) -> T {
        value
    }
}

/// The candidate provider contract: one method per frozen AGE service spelling.
/// Charged operations return `None` when the meter refuses the call (zero
/// effect); typed port fields are plain struct fields and cost nothing.
pub trait GfxHostProvider {
    fn meter(&self) -> &GfxMeter;
    fn meter_mut(&mut self) -> &mut GfxMeter;
    /// One world callback boundary.
    fn begin_tick(&mut self) {
        self.meter_mut().begin_tick();
    }

    fn input_space(&mut self) -> Option<i32>;
    fn input_x(&mut self) -> Option<f32>;
    fn input_y(&mut self) -> Option<f32>;
    fn input_pressed(&mut self, action: i32) -> Option<bool>;
    fn tick_dt(&mut self) -> Option<f32>;

    fn row_place(&mut self, row: i32, x: f32, y: f32) -> Option<i32>;
    fn row_shift(&mut self, row: i32, dx: f32, dy: f32) -> Option<i32>;
    fn row_show(&mut self, row: i32, visible: bool) -> Option<i32>;
    fn row_world(&mut self, row: i32, axis: i32) -> Option<f32>;
    fn row_local(&mut self, row: i32, axis: i32) -> Option<f32>;

    fn query_segment(
        &mut self,
        row: i32,
        visible_only: bool,
        ax: f32,
        ay: f32,
        bx: f32,
        by: f32,
    ) -> Option<Cast2D>;
    fn query_sweep(&mut self, row: i32, visible_only: bool, dx: f32, dy: f32) -> Option<Sweep2D>;

    fn render_spawn(&mut self, leaf: &str, x: f32, y: f32) -> Option<i32>;
    fn render_despawn(&mut self, token: i32) -> Option<i32>;
    fn render_frame(&mut self, token: i32, image: &str, ordinal: i32, flip: i32) -> Option<i32>;

    /// Host identity port (`image2d.png`): the host computes the content
    /// identity; the candidate contract only carries it.
    fn image_png(&mut self, path: &str) -> Option<Image2D>;
    /// Pure value construction (`frame2d.at`).
    fn frame_at(&mut self, image: Image2D, x: i32, y: i32, w: i32, h: i32) -> Option<Frame2D>;

    /// UI058 host fact (not one of the 17 ops): the render-only row layer that
    /// AGE cannot declare yet (`t04-prep.md` §3). Query implementations must
    /// skip these rows; they are never candidates and cannot be a mover.
    fn row_layer(&self, row: i32) -> Option<Layer>;
}

/// One recorded provider call (contract evidence: order, fee, acceptance).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCall {
    pub op: &'static str,
    pub cost: i32,
    pub accepted: bool,
}

/// A recording/fake provider for contract tests. Typed query results and the
/// input surface are scripted; row state and the render publish log are real so
/// a refused call can be proven to have zero effect.
#[derive(Clone, Debug)]
pub struct RecordingProvider {
    meter: GfxMeter,
    input: (i32, f32, f32, bool),
    dt: f32,
    rows: Vec<(f32, f32)>,
    layers: Vec<Layer>,
    /// Scripted typed results (`None` = the query missed/does not apply).
    segment: Cast2D,
    sweep: Sweep2D,
    log: Vec<HostCall>,
    publishes: Vec<(i32, f32, f32)>,
    frames: Vec<(i32, String, i32, i32)>,
    image_seq: u64,
}

impl RecordingProvider {
    pub fn new(rows: Vec<(f32, f32)>, layers: Vec<Layer>) -> Self {
        assert_eq!(rows.len(), layers.len(), "one layer per row");
        Self {
            meter: GfxMeter::default(),
            input: (2, 1.0, 0.0, false),
            dt: 1.0 / 60.0,
            rows,
            layers,
            segment: Cast2D {
                hit: false,
                node: -1,
                t: 1.0,
                point_x: 0.0,
                point_y: 0.0,
                normal_x: 0.0,
                normal_y: 0.0,
            },
            sweep: Sweep2D {
                hit: false,
                node: -1,
                t: 1.0,
                origin_x: 0.0,
                origin_y: 0.0,
                normal_x: 0.0,
                normal_y: 0.0,
                start_overlap: false,
            },
            log: Vec::new(),
            publishes: Vec::new(),
            frames: Vec::new(),
            image_seq: 0,
        }
    }
    pub fn set_input(&mut self, space: i32, x: f32, y: f32, pressed: bool) {
        self.input = (space, x, y, pressed);
    }
    pub fn set_segment(&mut self, result: Cast2D) {
        self.segment = result;
    }
    pub fn set_sweep(&mut self, result: Sweep2D) {
        self.sweep = result;
    }
    pub fn log(&self) -> &[HostCall] {
        &self.log
    }
    pub fn row(&self, row: usize) -> (f32, f32) {
        self.rows[row]
    }
    pub fn publishes(&self) -> &[(i32, f32, f32)] {
        &self.publishes
    }
    pub fn frames(&self) -> &[(i32, String, i32, i32)] {
        &self.frames
    }
    pub fn charges(&self, op: &'static str) -> i32 {
        self.log.iter().filter(|c| c.op == op).count() as i32
    }
    fn step(&mut self, op: &'static str, cost: i32) -> bool {
        let accepted = self.meter.charge(cost).is_ok();
        self.log.push(HostCall {
            op,
            cost,
            accepted,
        });
        accepted
    }
}

impl GfxHostProvider for RecordingProvider {
    fn meter(&self) -> &GfxMeter {
        &self.meter
    }
    fn meter_mut(&mut self) -> &mut GfxMeter {
        &mut self.meter
    }
    fn input_space(&mut self) -> Option<i32> {
        self.step("input.space", CREDIT_BASE).then_some(self.input.0)
    }
    fn input_x(&mut self) -> Option<f32> {
        self.step("input.x", CREDIT_BASE).then_some(self.input.1)
    }
    fn input_y(&mut self) -> Option<f32> {
        self.step("input.y", CREDIT_BASE).then_some(self.input.2)
    }
    fn input_pressed(&mut self, _action: i32) -> Option<bool> {
        self.step("input.pressed", CREDIT_BASE)
            .then_some(self.input.3)
    }
    fn tick_dt(&mut self) -> Option<f32> {
        self.step("tick.dt", CREDIT_BASE).then_some(self.dt)
    }
    fn row_place(&mut self, row: i32, x: f32, y: f32) -> Option<i32> {
        if !self.step("row.place", CREDIT_BASE) {
            return None;
        }
        let slot = usize::try_from(row).ok()?;
        let entry = self.rows.get_mut(slot)?;
        *entry = (x, y);
        Some(1)
    }
    fn row_shift(&mut self, row: i32, dx: f32, dy: f32) -> Option<i32> {
        if !self.step("row.shift", CREDIT_BASE) {
            return None;
        }
        let slot = usize::try_from(row).ok()?;
        let entry = self.rows.get_mut(slot)?;
        entry.0 += dx;
        entry.1 += dy;
        Some(1)
    }
    fn row_show(&mut self, _row: i32, _visible: bool) -> Option<i32> {
        self.step("row.show", CREDIT_BASE).then_some(1)
    }
    fn row_world(&mut self, row: i32, axis: i32) -> Option<f32> {
        if !self.step("row.world", CREDIT_BASE) {
            return None;
        }
        let slot = usize::try_from(row).ok()?;
        let entry = self.rows.get(slot)?;
        Some(if axis == 0 { entry.0 } else { entry.1 })
    }
    fn row_local(&mut self, row: i32, axis: i32) -> Option<f32> {
        if !self.step("row.local", CREDIT_BASE) {
            return None;
        }
        let slot = usize::try_from(row).ok()?;
        let entry = self.rows.get(slot)?;
        Some(if axis == 0 { entry.0 } else { entry.1 })
    }
    fn query_segment(
        &mut self,
        row: i32,
        visible_only: bool,
        _ax: f32,
        _ay: f32,
        _bx: f32,
        _by: f32,
    ) -> Option<Cast2D> {
        let _ = visible_only;
        // Frozen order: the charge happens before the layer guard (T-03), so a
        // render-only mover/ignore is charged and then refused with no result.
        let charged = self.step("query2d.segment", CREDIT_SEGMENT);
        if self.row_layer(row) == Some(Layer::RenderOnly) {
            return None;
        }
        charged.then_some(self.segment)
    }
    fn query_sweep(&mut self, row: i32, visible_only: bool, _dx: f32, _dy: f32) -> Option<Sweep2D> {
        let _ = visible_only;
        let charged = self.step("query2d.sweep", CREDIT_SWEEP);
        if self.row_layer(row) == Some(Layer::RenderOnly) {
            return None;
        }
        charged.then_some(self.sweep)
    }
    fn render_spawn(&mut self, leaf: &str, x: f32, y: f32) -> Option<i32> {
        if !self.step("render2d.spawn", CREDIT_BASE) {
            return None;
        }
        self.image_seq += 1;
        let token = self.image_seq as i32;
        let _ = leaf;
        self.publishes.push((token, x, y));
        Some(token)
    }
    fn render_despawn(&mut self, token: i32) -> Option<i32> {
        self.step("render2d.despawn", CREDIT_BASE).then_some(token)
    }
    fn render_frame(&mut self, token: i32, image: &str, ordinal: i32, flip: i32) -> Option<i32> {
        if !self.step("render2d.frame", CREDIT_BASE) {
            return None;
        }
        self.frames.push((token, image.to_string(), ordinal, flip));
        Some(token)
    }
    fn image_png(&mut self, path: &str) -> Option<Image2D> {
        if !self.step("image2d.png", CREDIT_BASE) {
            return None;
        }
        self.image_seq += 1;
        Some(Image2D {
            identity: format!("{path}#{}", self.image_seq),
        })
    }
    fn frame_at(&mut self, image: Image2D, x: i32, y: i32, w: i32, h: i32) -> Option<Frame2D> {
        if !self.step("frame2d.at", CREDIT_BASE) {
            return None;
        }
        Some(Frame2D { image, x, y, w, h })
    }
    fn row_layer(&self, row: i32) -> Option<Layer> {
        usize::try_from(row)
            .ok()
            .and_then(|i| self.layers.get(i).copied())
    }
}
