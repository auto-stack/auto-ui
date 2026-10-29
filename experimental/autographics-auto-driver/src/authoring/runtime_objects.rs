//! Run-owned finite runtime object instances (auto-ui PLAN-001). One
//! dimension per Run and 8 reserved slots; identity is the integer handle
//! `run*524288 + slot*65536 + generation` with a per-slot generation that
//! starts at 1 and never wraps. The admitted instance block rides the Play
//! `Frame` (after the camera block), so objects, character and camera are
//! prepared, accepted and published as one transaction; a rejected candidate
//! keeps the last complete frame. Instances never enter the Document, the
//! spatial worlds or the camera/character/solid state: the host renders them
//! through a non-persistent adapter object derived from the accepted template.
use super::{document::Object, hierarchy::Node, runtime_camera::CameraDim};

#[cfg(test)]
use super::document::{Dimension, Kind};

/// Frozen slot table shape (contract section 3): 8 reserved slots per Run.
pub const SLOTS: usize = 8;
/// Frozen handle encoding: generation span and slot stride inside one i32.
/// Generation exhaustion and run-id exhaustion are named rejections on both
/// the trusted unit and the host admission; no wraparound exists.
pub const GEN_SPAN: i32 = 65536;
pub const RUN_STRIDE: i32 = GEN_SPAN * SLOTS as i32;
pub const MAX_GENERATION: i32 = GEN_SPAN - 1;
pub const MAX_HANDLE_RUN: i32 = (i32::MAX - GEN_SPAN) / RUN_STRIDE;
/// Lifecycle writes (spawn/despawn/move/visible) per callback.
pub const MAX_WRITES_PER_CALLBACK: i32 = 32;
/// Frozen world coordinate envelope, identical to the document numeric range.
pub const COORD_LIMIT: f32 = 100000.0;
/// The admitted block: 4 header ints + 8 canonical rows of 6 values.
pub const BLOCK_LEN: usize = 4 + SLOTS * 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dim {
    TwoD,
    ThreeD,
}
impl Dim {
    pub fn from_camera(dim: CameraDim) -> Self {
        match dim {
            CameraDim::TwoD => Self::TwoD,
            CameraDim::ThreeD => Self::ThreeD,
        }
    }
    pub fn code(self) -> i32 {
        match self {
            Self::TwoD => 2,
            Self::ThreeD => 3,
        }
    }
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            2 => Some(Self::TwoD),
            3 => Some(Self::ThreeD),
            _ => None,
        }
    }
    /// Spawn template kinds (contract section 4): accepted same-dimension
    /// non-group leaves. 2D accepts Circle/Quad; 3D accepts the renderable
    /// builtins and Model (the character/solid roles are separate profiles).
    pub fn supports_kind(self, kind: i32) -> bool {
        match self {
            Self::TwoD => (0..=1).contains(&kind),
            Self::ThreeD => (2..=6).contains(&kind) && kind != 5,
        }
    }
}

/// One admitted slot row. `generation == 0` marks the slot free; every other
/// field of a free row is canonical zero (admission rejects aliasing).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub generation: i32,
    /// Run index into the frozen base order of the template node.
    pub template_index: i32,
    /// Resolved document id of the template node (admission oracle).
    pub template_id: u64,
    pub translation: [f32; 3],
    pub visible: bool,
}
impl Slot {
    pub const FREE: Slot = Slot {
        generation: 0,
        template_index: 0,
        template_id: 0,
        translation: [0.0; 3],
        visible: false,
    };
    pub fn live(&self) -> bool {
        self.generation > 0
    }
}

/// The objects outputs admitted with a Play frame. Created by `Play::admit`
/// from the fixed block and cross-checked against the previous admitted table
/// (generations never decrease; a live generation reuses only across a
/// despawn). `template_id` resolves the run index to the document node id so
/// presentation reads the accepted frame only.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeObjectsFrame {
    pub dim: Dim,
    pub run_id: i32,
    pub tick: i32,
    pub slots: [Slot; SLOTS],
}
impl RuntimeObjectsFrame {
    pub fn live_count(&self) -> usize {
        self.slots.iter().filter(|s| s.live()).count()
    }
    pub fn get(&self, slot: usize) -> Option<&Slot> {
        self.slots.get(slot).filter(|s| s.live())
    }
    /// Total potential 3D mesh instances of the live slots given the template
    /// mesh counts (hidden slots never shrink the denominator).
    pub fn potential_meshes(
        &self,
        template_meshes: impl Fn(u64) -> Option<usize>,
    ) -> Option<usize> {
        let mut total = 0usize;
        for slot in self.slots.iter().filter(|s| s.live()) {
            total += template_meshes(slot.template_id)?;
            if total > usize::MAX / 2 {
                return None;
            }
        }
        Some(total)
    }
}

/// The frozen run-order facts admission needs to validate a template row.
pub struct TemplateRules<'a> {
    pub order: &'a [Node],
    pub dim: Dim,
}
impl TemplateRules<'_> {
    pub fn eligible(&self, index: i32) -> bool {
        let Some(node) = self.order.get(index as usize) else {
            return false;
        };
        if node.kind == 5 {
            return false;
        }
        if node.dimension.code() != self.dim.code() {
            return false;
        }
        self.dim.supports_kind(node.kind)
    }
}

/// Parse and validate the objects block at `data[block..]`. Every rejection
/// names its stage; a rejected candidate never reaches accept/publish.
pub fn admit_block(
    data: &[crate::Scalar],
    run_id: i32,
    tick: i32,
    rules: &TemplateRules<'_>,
    previous: Option<&RuntimeObjectsFrame>,
    high_water: &mut [i32; SLOTS],
) -> Result<RuntimeObjectsFrame, String> {
    let int = |i: usize| -> Result<i32, String> {
        match data.get(i) {
            Some(crate::Scalar::Int(v)) => Ok(*v),
            _ => Err("objects integer output".into()),
        }
    };
    let float = |i: usize| -> Result<f32, String> {
        match data.get(i) {
            Some(crate::Scalar::Float(v)) if v.is_finite() => Ok(*v),
            _ => Err("objects float output".into()),
        }
    };
    if int(0)? != 1 {
        return Err("objects status output".into());
    }
    let count = int(1)?;
    if !(0..=SLOTS as i32).contains(&count) {
        return Err("objects count output".into());
    }
    if int(2)? != run_id || int(3)? != tick {
        return Err("objects run/tick output".into());
    }
    let mut slots = [Slot::FREE; SLOTS];
    let mut live = 0usize;
    for (s, slot) in slots.iter_mut().enumerate() {
        let generation = int(4 + s * 6)?;
        let template_index = int(5 + s * 6)?;
        let x = float(6 + s * 6)?;
        let y = float(7 + s * 6)?;
        let z = float(8 + s * 6)?;
        let visible = int(9 + s * 6)?;
        if generation == 0 {
            if template_index != 0 || x != 0.0 || y != 0.0 || z != 0.0 || visible != 0 {
                return Err(format!("objects free slot {s} must be canonical zeros"));
            }
            continue;
        }
        live += 1;
        if !(1..=MAX_GENERATION).contains(&generation) {
            return Err(format!("objects slot {s} generation domain"));
        }
        if !rules.eligible(template_index) {
            return Err(format!("objects slot {s} template eligibility"));
        }
        if !matches!(visible, 0 | 1) {
            return Err(format!("objects slot {s} visibility domain"));
        }
        for (axis, v) in [x, y, z].iter().enumerate() {
            if !v.is_finite() || v.abs() > COORD_LIMIT {
                return Err(format!("objects slot {s} coordinate axis {axis} domain"));
            }
        }
        if rules.dim == Dim::TwoD && z != 0.0 {
            return Err(format!("objects slot {s} 2D z domain"));
        }
        let previous_slot = previous.and_then(|p| p.slots.get(s).copied());
        match previous_slot {
            Some(prev) if prev.live() => {
                if generation < prev.generation {
                    return Err(format!("objects slot {s} generation went backwards"));
                }
                if generation == prev.generation && template_index != prev.template_index {
                    return Err(format!(
                        "objects slot {s} template changed within one generation"
                    ));
                }
            }
            _ => {
                let watermark = high_water[s];
                if generation <= watermark {
                    return Err(format!(
                        "objects slot {s} birth must exceed the generation high-water {watermark}"
                    ));
                }
            }
        }
        if generation > high_water[s] {
            high_water[s] = generation;
        }
        *slot = Slot {
            generation,
            template_index,
            template_id: rules.order[template_index as usize].id,
            translation: [x, y, z],
            visible: visible == 1,
        };
    }
    if live as i32 != count {
        return Err("objects count output".into());
    }
    Ok(RuntimeObjectsFrame {
        dim: rules.dim,
        run_id,
        tick,
        slots,
    })
}

/// The non-persistent render adapter for one admitted instance (contract R-02):
/// kind/scale/yaw/pitch/material/resource come from the accepted template
/// object; the position/visibility come from the instance. The id is 0, which
/// no admitted document object can carry, and the value is never stored into
/// the Document, preview or history; it exists only inside prepare_frame.
pub fn instance_object(template: &Object, slot: &Slot) -> Option<Object> {
    if !slot.live() {
        return None;
    }
    Some(Object {
        id: 0,
        name: format!(
            "runtime-instance slot{} gen{}",
            slot.template_index, slot.generation
        ),
        kind: template.kind,
        translation: slot.translation,
        scale: template.scale,
        yaw: template.yaw,
        pitch: template.pitch,
        rgba: template.rgba,
        visible: slot.visible,
        parent: None,
        resource: template.resource.clone(),
    })
}

/// The named construction envelope: `base nodes + reserved slots <= 128`.
pub fn slot_capacity_error(base_nodes: usize) -> Option<String> {
    if base_nodes + SLOTS > 128 {
        Some(format!(
            "objects slot capacity: base nodes {base_nodes} + reserved {SLOTS} = {} exceed 128",
            base_nodes + SLOTS
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scalar;
    use crate::Scalar::{Float, Int};

    fn node(id: u64, kind: i32, dim: i32) -> Node {
        Node {
            id,
            parent: None,
            dimension: if dim == 2 {
                Dimension::Two
            } else {
                Dimension::Three
            },
            kind,
            local: [0.0; 3],
            visible: true,
        }
    }

    fn rules(order: &[Node], dim: Dim) -> TemplateRules<'_> {
        TemplateRules { order, dim }
    }

    fn block(rows: [(i32, i32, f32, f32, f32, i32); SLOTS], run: i32, tick: i32) -> Vec<Scalar> {
        let mut data = vec![
            Int(1),
            Int(rows.iter().filter(|r| r.0 > 0).count() as i32),
            Int(run),
            Int(tick),
        ];
        for (gen, tmpl, x, y, z, vis) in rows {
            data.push(Int(gen));
            data.push(Int(tmpl));
            data.push(Float(x));
            data.push(Float(y));
            data.push(Float(z));
            data.push(Int(vis));
        }
        data
    }

    fn free_row() -> (i32, i32, f32, f32, f32, i32) {
        (0, 0, 0.0, 0.0, 0.0, 0)
    }

    #[test]
    fn handle_encoding_bounds_leave_room_for_named_exhaustion() {
        assert_eq!(RUN_STRIDE, 524288);
        assert_eq!(MAX_GENERATION, 65535);
        assert!(MAX_HANDLE_RUN >= 4095);
        // The frozen encoding must stay inside the positive i32 domain for
        // every admitted run id, so a foreign-Run handle decodes negative.
        let top = MAX_HANDLE_RUN * RUN_STRIDE + 7 * GEN_SPAN + MAX_GENERATION;
        assert!(top > 0 && top <= i32::MAX);
    }

    #[test]
    fn block_admission_accepts_birth_updates_and_despawn_sequences() {
        let order = [
            node(1, 1, 2), // quad, 2D template
            node(2, 0, 2), // circle
            node(3, 2, 3), // cylinder, wrong dimension for a 2D run
            node(4, 5, 2), // group: never a template
        ];
        let mut high_water = [0i32; SLOTS];
        let rules = rules(&order, Dim::TwoD);
        // Tick 0: spawn slot 0 gen 1 and slot 3 gen 1.
        let mut rows = [free_row(); SLOTS];
        rows[0] = (1, 0, 10.0, 20.0, 0.0, 1);
        rows[3] = (1, 1, -5.0, 0.0, 0.0, 1);
        let f0 =
            admit_block(&block(rows, 7, 0), 7, 0, &rules, None, &mut high_water).expect("tick 0");
        assert_eq!(f0.live_count(), 2);
        assert_eq!(f0.get(0).unwrap().template_id, 1);
        assert_eq!(high_water, [1, 0, 0, 1, 0, 0, 0, 0]);
        // Tick 1: move slot 0 (same generation), hide slot 3, respawn slot 5.
        let mut rows = [free_row(); SLOTS];
        rows[0] = (1, 0, 12.5, 20.0, 0.0, 1);
        rows[3] = (1, 1, -5.0, 0.0, 0.0, 0);
        rows[5] = (2, 0, 1.0, 2.0, 0.0, 1);
        let f1 = admit_block(&block(rows, 7, 1), 7, 1, &rules, Some(&f0), &mut high_water)
            .expect("tick 1");
        assert_eq!(f1.get(0).unwrap().translation, [12.5, 20.0, 0.0]);
        assert!(
            f1.get(3).unwrap().live() && !f1.get(3).unwrap().visible,
            "a hidden instance keeps its identity and slot"
        );
        assert_eq!(f1.get(5).unwrap().generation, 2);
        assert_eq!(high_water[5], 2);
        // Tick 2: slot 0 despawned; slot 3 returns with a NEW generation.
        let mut rows = [free_row(); SLOTS];
        rows[3] = (2, 1, -5.0, 0.0, 0.0, 1);
        let f2 = admit_block(&block(rows, 7, 2), 7, 2, &rules, Some(&f1), &mut high_water)
            .expect("tick 2");
        assert_eq!(f2.live_count(), 1);
        assert_eq!(f2.get(3).unwrap().generation, 2);
    }

    #[test]
    fn block_admission_rejects_every_named_counterexample_stage() {
        let order = [node(1, 1, 2), node(2, 6, 3), node(3, 5, 2)];
        let mut high_water = [0i32; SLOTS];
        let rules2 = rules(&order, Dim::TwoD);
        let rules3 = rules(&order, Dim::ThreeD);
        let valid3 = {
            let mut rows = [free_row(); SLOTS];
            rows[0] = (1, 1, 1.0, 1.0, 1.0, 1);
            admit_block(&block(rows, 1, 0), 1, 0, &rules3, None, &mut high_water).expect("3d birth")
        };
        let cases: Vec<(&str, Dim, Vec<Scalar>, i32, i32)> = vec![
            (
                "status",
                Dim::TwoD,
                {
                    let mut d = block([free_row(); SLOTS], 1, 0);
                    d[0] = Int(0);
                    d
                },
                1,
                0,
            ),
            (
                "count-high",
                Dim::TwoD,
                {
                    let mut d = block([free_row(); SLOTS], 1, 0);
                    d[1] = Int(9);
                    d
                },
                1,
                0,
            ),
            (
                "count-mismatch",
                Dim::TwoD,
                {
                    let mut d = block([free_row(); SLOTS], 1, 0);
                    d[1] = Int(1);
                    d
                },
                1,
                0,
            ),
            (
                "run",
                Dim::TwoD,
                {
                    let mut d = block([free_row(); SLOTS], 1, 0);
                    d[2] = Int(8);
                    d
                },
                1,
                0,
            ),
            (
                "tick",
                Dim::TwoD,
                {
                    let mut d = block([free_row(); SLOTS], 1, 0);
                    d[3] = Int(9);
                    d
                },
                1,
                0,
            ),
            (
                "float-domain",
                Dim::TwoD,
                {
                    let mut d = block([free_row(); SLOTS], 1, 0);
                    d[6] = Int(1);
                    d
                },
                1,
                0,
            ),
            (
                "template-group",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 2, 1.0, 0.0, 0.0, 1);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "template-dimension",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 1, 1.0, 0.0, 0.0, 1);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "template-missing",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 99, 1.0, 0.0, 0.0, 1);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "generation-zero-birth",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[1] = (0, 0, 1.0, 0.0, 0.0, 1);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "noncanonical-free-row",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[2] = (0, 0, 1.0, 0.0, 0.0, 0);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "2d-z-domain",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 0, 1.0, 0.0, 0.5, 1);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "coordinate-limit",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 0, 100001.0, 0.0, 0.0, 1);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "visibility-domain",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 0, 1.0, 0.0, 0.0, 2);
                    block(rows, 1, 0)
                },
                1,
                0,
            ),
            (
                "generation-high-water",
                Dim::TwoD,
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 0, 1.0, 0.0, 0.0, 1);
                    block(rows, 1, 0)
                },
                1,
                4,
            ),
        ];
        let mut high_water_a = [0i32; SLOTS];
        let mut high_water_b = [4i32; SLOTS];
        for (i, (name, dim, data, run, tick)) in cases.into_iter().enumerate() {
            let (rules, hw) = if name == "generation-high-water" {
                (&rules2, &mut high_water_b)
            } else if dim == Dim::TwoD {
                (&rules2, &mut high_water_a)
            } else {
                (&rules3, &mut high_water_a)
            };
            let error = admit_block(&data, run, tick, rules, None, hw).expect_err(name);
            assert!(
                error.contains("objects"),
                "case {name} must fail at the named objects stage: {error}"
            );
            let _ = i;
        }
        // The accepted 3D control still admits beside the rejections.
        let mut control_water = [0i32; SLOTS];
        assert!(admit_block(
            &block(
                {
                    let mut rows = [free_row(); SLOTS];
                    rows[0] = (1, 1, 1.0, 1.0, 1.0, 1);
                    rows
                },
                1,
                0,
            ),
            1,
            0,
            &rules3,
            None,
            &mut control_water
        )
        .is_ok());
        // Generation must never decrease against the previous admitted frame.
        let mut hw = [0i32; SLOTS];
        let mut rows = [free_row(); SLOTS];
        rows[0] = (1, 1, 1.0, 1.0, 1.0, 1);
        let f0 = admit_block(&block(rows, 1, 0), 1, 0, &rules3, None, &mut hw).expect("birth");
        let mut rows = [free_row(); SLOTS];
        rows[0] = (1, 1, 2.0, 1.0, 1.0, 1);
        let f1 =
            admit_block(&block(rows, 1, 1), 1, 1, &rules3, Some(&f0), &mut hw).expect("update");
        // A second live frame with the same generation but a different template
        // is rejected; the template is frozen within one generation.
        let mut rows_swap = [free_row(); SLOTS];
        rows_swap[0] = (1, 0, 3.0, 1.0, 1.0, 1);
        assert!(admit_block(&block(rows_swap, 1, 2), 1, 2, &rules3, Some(&f1), &mut hw).is_err());
        let _ = valid3;
    }

    #[test]
    fn capacity_envelope_counts_base_nodes_plus_reserved_slots() {
        assert!(slot_capacity_error(120).is_none());
        assert!(slot_capacity_error(121).is_some());
        let error = slot_capacity_error(128).unwrap();
        assert!(error.contains("136"), "{error}");
    }

    #[test]
    fn instance_adapter_is_nonpersistent_and_carries_the_template_shape() {
        let mut template = Object::new(9, Kind::Cylinder);
        template.scale = [2.0, 1.0, 2.0];
        template.translation = [77.0, 77.0, 77.0];
        let slot = Slot {
            generation: 3,
            template_index: 0,
            template_id: 9,
            translation: [1.0, 2.0, 3.0],
            visible: true,
        };
        let o = instance_object(&template, &slot).expect("instance");
        assert_eq!(o.id, 0, "adapter ids can never alias a document object");
        assert_eq!(o.kind, Kind::Cylinder);
        assert_eq!(o.scale, template.scale);
        assert_eq!(o.translation, [1.0, 2.0, 3.0]);
        assert_eq!(o.resource, template.resource);
        assert!(instance_object(&template, &Slot::FREE).is_none());
    }
}
