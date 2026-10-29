//! Finite trigger events and audited round state (auto-ui PLAN-002). The
//! trusted unit emits one block per triggers-Run tick after the objects block;
//! this module parses and re-verifies it against the same inputs the unit
//! saw — the candidate node worlds, the static descriptor rows and the
//! candidate instance table — so an event row must match an independently
//! recomputed enter/stay/exit transition exactly. The restart request is a
//! bounded marker carried on the same admitted frame; the host consumes it
//! only after the carrying candidate is fully accepted.
use super::runtime_objects::{Dim, RuntimeObjectsFrame, SLOTS};

/// Frozen pair table shape (contract section 3): at most 8 explicit pairs.
pub const PAIRS: usize = 8;
/// Frozen binding-generation span; exhaustion is a named rejection, no wrap.
pub const MAX_BIND_GENERATION: i32 = 65535;
/// Block layout: 8 header ints + 8 binding rows (8 ints + 1 float) + 8 event
/// rows of 8 ints, all in pair order.
pub const HEADER_LEN: usize = 8;
pub const BINDING_LEN: usize = 9;
pub const EVENT_LEN: usize = 8;
pub const BLOCK_LEN: usize = HEADER_LEN + PAIRS * BINDING_LEN + PAIRS * EVENT_LEN;

pub const PHASE_ENTER: i32 = 1;
pub const PHASE_STAY: i32 = 2;
pub const PHASE_EXIT: i32 = 3;
pub const SHAPE_AABB: i32 = 0;
pub const SHAPE_SPHERE: i32 = 1;
/// The frozen world coordinate envelope, identical to the objects unit.
pub const RADIUS_LIMIT: f32 = super::runtime_objects::COORD_LIMIT;

/// One admitted pair binding. `source_kind` 0 resolves `source_ref` as a run
/// index into the frozen base order (static leaf); 1 resolves it as a live
/// instance slot whose identity is fully bound to `source_generation` and the
/// template run index `source_template_index`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binding {
    pub bind_generation: i32,
    pub subject_index: i32,
    /// Resolved document id of the subject node (admission oracle).
    pub subject_id: u64,
    pub source_kind: i32,
    pub source_ref: i32,
    /// Resolved document id: the source leaf itself, or the template node.
    pub source_id: u64,
    pub source_template_index: i32,
    pub source_generation: i32,
    pub shape: i32,
    pub radius: f32,
}

/// One admitted event of the frozen per-tick snapshot, in pair order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerEvent {
    pub pair: i32,
    pub phase: i32,
    pub bind_generation: i32,
    pub source_kind: i32,
    /// The originally bound reference: stable document id or object handle.
    pub source_ref: i32,
    pub source_template_index: i32,
    pub source_generation: i32,
    pub geometry: i32,
}

/// The rounds outputs admitted with a Play frame (contract section 3).
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRoundFrame {
    pub dim: Dim,
    pub run_id: i32,
    pub tick: i32,
    pub evaluate_seq: i32,
    pub restart_requested: bool,
    pub restart_tick: i32,
    pub bindings: [Option<Binding>; PAIRS],
    pub events: Vec<TriggerEvent>,
}

/// Whether a subject/source pair overlaps this tick, using the same
/// closed-interval and closest-point-squared semantics as the trusted unit.
/// Boxes are descriptor rows ([valid, min*, max*]); positions are the world
/// translations the unit read after this tick's derive.
pub fn overlaps(
    dim: Dim,
    shape: i32,
    radius: f32,
    subject_box: &[f32; 7],
    subject_pos: [f32; 3],
    source_box: &[f32; 7],
    source_pos: [f32; 3],
) -> bool {
    let axes = match dim {
        Dim::TwoD => 2,
        Dim::ThreeD => 3,
    };
    if shape == SHAPE_AABB {
        for axis in 0..axes {
            let a_lo = subject_box[1 + axis] + subject_pos[axis];
            let a_hi = subject_box[4 + axis] + subject_pos[axis];
            let b_lo = source_box[1 + axis] + source_pos[axis];
            let b_hi = source_box[4 + axis] + source_pos[axis];
            if a_hi < b_lo || b_hi < a_lo {
                return false;
            }
        }
        return true;
    }
    if shape != SHAPE_SPHERE || !(radius > 0.0 && radius.is_finite()) {
        return false;
    }
    let mut square = 0.0f32;
    for axis in 0..axes {
        let lo = subject_box[1 + axis] + subject_pos[axis];
        let hi = subject_box[4 + axis] + subject_pos[axis];
        let center = source_pos[axis];
        let d = center.clamp(lo, hi) - center;
        square += d * d;
    }
    square <= radius * radius
}

/// Named construction envelope: the pairs reuse the objects slots, so the
/// trigger profiles carry the same node budget as the objects profiles.
pub fn pair_capacity_error(base_nodes: usize) -> Option<String> {
    super::runtime_objects::slot_capacity_error(base_nodes)
}

fn int_at(data: &[crate::Scalar], i: usize) -> Result<i32, String> {
    match data.get(i) {
        Some(crate::Scalar::Int(v)) => Ok(*v),
        _ => Err("rounds integer output".into()),
    }
}

fn float_at(data: &[crate::Scalar], i: usize) -> Result<f32, String> {
    match data.get(i) {
        Some(crate::Scalar::Float(v)) if v.is_finite() => Ok(*v),
        _ => Err("rounds float output".into()),
    }
}

/// The frozen per-pair admission inputs: the run order, the descriptor rows
/// (aligned with the order) for the independent geometry oracle and this
/// tick's node world translations (the derive-oracle values the unit read).
pub struct RoundRules<'a> {
    pub order: &'a [super::hierarchy::Node],
    pub rows: &'a [[f32; 7]],
    pub worlds: &'a [[f32; 3]],
    pub dim: Dim,
}

impl RoundRules<'_> {
    /// A same-dimension non-group leaf with an actual accepted box.
    fn leaf(&self, index: i32) -> Option<u64> {
        let node = self.order.get(index as usize)?;
        if node.kind == 5 || node.dimension.code() != self.dim.code() {
            return None;
        }
        let row = self.rows.get(index as usize)?;
        if row[0] != 1.0 {
            return None;
        }
        Some(node.id)
    }
    fn box_row(&self, index: i32) -> Option<&[f32; 7]> {
        self.rows.get(index as usize)
    }
    fn world(&self, index: i32) -> [f32; 3] {
        self.worlds.get(index as usize).copied().unwrap_or([0.0; 3])
    }
}

/// Parse and validate the rounds block at `data[block..]`. `instances` is this
/// tick's admitted object table (the same candidate), `previous` the last
/// admitted rounds frame, `contact`/`bind_water` the host-tracked per-pair
/// state. Every rejection names its stage; a rejected candidate never reaches
/// accept/publish.
pub fn admit_round_block(
    data: &[crate::Scalar],
    run_id: i32,
    tick: i32,
    rules: &RoundRules<'_>,
    instances: Option<&RuntimeObjectsFrame>,
    previous: Option<&RuntimeRoundFrame>,
    contact: &mut [bool; PAIRS],
    bind_water: &mut [i32; PAIRS],
) -> Result<RuntimeRoundFrame, String> {
    if int_at(data, 0)? != 1 {
        return Err("rounds status output".into());
    }
    if int_at(data, 1)? != run_id || int_at(data, 2)? != tick {
        return Err("rounds run/tick output".into());
    }
    let event_count = int_at(data, 3)?;
    if !(0..=PAIRS as i32).contains(&event_count) {
        return Err("rounds event count output".into());
    }
    let restart_requested = int_at(data, 4)?;
    let restart_tick = int_at(data, 5)?;
    let evaluate_seq = int_at(data, 6)?;
    if int_at(data, 7)? != 0 {
        return Err("rounds canonical header output".into());
    }
    if tick == 0 {
        // app_start only establishes the registry: no events, no restart.
        if event_count != 0 || restart_requested != 0 || restart_tick != 0 || evaluate_seq != 0 {
            return Err("rounds tick 0 registry output".into());
        }
    } else if evaluate_seq != tick {
        return Err("rounds evaluate sequence output".into());
    }
    if !matches!(restart_requested, 0 | 1) {
        return Err("rounds restart marker output".into());
    }
    if restart_requested == 1 {
        if tick == 0 || restart_tick != tick {
            return Err("rounds restart marker output".into());
        }
    } else if restart_tick != 0 {
        return Err("rounds restart marker output".into());
    }
    // The binding table: every unbound/inactive row is canonical zeros.
    let mut bindings: [Option<Binding>; PAIRS] = [None; PAIRS];
    for p in 0..PAIRS {
        let base = HEADER_LEN + p * BINDING_LEN;
        let bound = int_at(data, base)?;
        if !matches!(bound, 0 | 1) {
            return Err(format!("rounds pair {p} bound domain"));
        }
        if bound == 0 {
            let canonical = (1..BINDING_LEN).all(|offset| match data.get(base + offset) {
                Some(crate::Scalar::Int(0)) => true,
                Some(crate::Scalar::Float(v)) if *v == 0.0 => true,
                _ => false,
            });
            if !canonical {
                return Err(format!(
                    "rounds pair {p} unbound row must be canonical zeros"
                ));
            }
            contact[p] = false;
            continue;
        }
        let bind_generation = int_at(data, base + 1)?;
        let subject_index = int_at(data, base + 2)?;
        let source_kind = int_at(data, base + 3)?;
        let source_ref = int_at(data, base + 4)?;
        let source_template_index = int_at(data, base + 5)?;
        let source_generation = int_at(data, base + 6)?;
        let shape = int_at(data, base + 7)?;
        let radius = float_at(data, base + 8)?;
        if !(1..=MAX_BIND_GENERATION).contains(&bind_generation) {
            return Err(format!("rounds pair {p} bind generation domain"));
        }
        let Some(subject_id) = rules.leaf(subject_index) else {
            return Err(format!("rounds pair {p} subject eligibility"));
        };
        if !matches!(source_kind, 0 | 1) {
            return Err(format!("rounds pair {p} source kind domain"));
        }
        if !matches!(shape, SHAPE_AABB | SHAPE_SPHERE) {
            return Err(format!("rounds pair {p} shape domain"));
        }
        match shape {
            SHAPE_AABB => {
                if radius != 0.0 {
                    return Err(format!(
                        "rounds pair {p} aabb radius must be canonical zero"
                    ));
                }
            }
            _ => {
                if !(radius > 0.0 && radius.is_finite() && radius <= RADIUS_LIMIT) {
                    return Err(format!("rounds pair {p} radius domain"));
                }
            }
        }
        let source_id = match source_kind {
            0 => {
                if source_template_index != source_ref || source_generation != 0 {
                    return Err(format!("rounds pair {p} static source canonical row"));
                }
                match rules.leaf(source_ref) {
                    Some(id) => id,
                    None => return Err(format!("rounds pair {p} source eligibility")),
                }
            }
            _ => {
                if !(0..SLOTS as i32).contains(&source_ref) {
                    return Err(format!("rounds pair {p} source slot domain"));
                }
                let frame = instances.ok_or(format!("rounds pair {p} missing instances"))?;
                match frame.get(source_ref as usize) {
                    Some(slot_row)
                        if slot_row.generation == source_generation
                            && slot_row.template_index == source_template_index =>
                    {
                        slot_row.template_id
                    }
                    _ => {
                        return Err(format!(
                            "rounds pair {p} source instance is not the bound live generation"
                        ))
                    }
                }
            }
        };
        // The binding is frozen within one generation and generations never
        // decrease against the admitted watermark (mirrors the objects rule).
        match previous.and_then(|f| f.bindings[p]) {
            Some(prev) => {
                if bind_generation < prev.bind_generation {
                    return Err(format!("rounds pair {p} bind generation went backwards"));
                }
                if bind_generation == prev.bind_generation {
                    let same = prev.subject_index == subject_index
                        && prev.source_kind == source_kind
                        && prev.source_ref == source_ref
                        && prev.source_template_index == source_template_index
                        && prev.source_generation == source_generation
                        && prev.shape == shape
                        && prev.radius == radius;
                    if !same {
                        return Err(format!(
                            "rounds pair {p} binding changed within one generation"
                        ));
                    }
                }
            }
            None => {
                if bind_generation <= bind_water[p] {
                    return Err(format!(
                        "rounds pair {p} bind must exceed the generation watermark {}",
                        bind_water[p]
                    ));
                }
            }
        }
        if bind_generation > bind_water[p] {
            bind_water[p] = bind_generation;
        }
        bindings[p] = Some(Binding {
            bind_generation,
            subject_index,
            subject_id,
            source_kind,
            source_ref,
            source_id,
            source_template_index,
            source_generation,
            shape,
            radius,
        });
    }
    // Independent geometry: recompute every pair against this tick's final
    // positions and require the emitted events to match the expected phase
    // sequence row by row.
    let mut events: Vec<TriggerEvent> = Vec::with_capacity(event_count as usize);
    for p in 0..PAIRS {
        let Some(binding) = bindings[p] else {
            contact[p] = false;
            continue;
        };
        if tick == 0 {
            // Tick 0 is the registry frame: the unit never evaluates there,
            // so no phase is expected and contact stays false.
            contact[p] = false;
            continue;
        }
        let subject_box = rules
            .box_row(binding.subject_index)
            .ok_or("rounds subject box output")?;
        let (source_box, source_pos) = match binding.source_kind {
            0 => {
                let row = rules
                    .box_row(binding.source_ref)
                    .ok_or("rounds source box output")?;
                let world = rules.world(binding.source_ref);
                (row, world)
            }
            _ => {
                let frame = instances.ok_or("rounds missing instances")?;
                let slot = frame
                    .get(binding.source_ref as usize)
                    .ok_or("rounds source instance left")?;
                let row = rules
                    .box_row(slot.template_index)
                    .ok_or("rounds source box output")?;
                (row, slot.translation)
            }
        };
        let now = overlaps(
            rules.dim,
            binding.shape,
            binding.radius,
            subject_box,
            rules.world(binding.subject_index),
            source_box,
            source_pos,
        );
        // The previous contact only carries under the same binding generation.
        let carried = contact[p]
            && previous
                .and_then(|f| f.bindings[p])
                .is_some_and(|prev| prev.bind_generation == binding.bind_generation);
        let expected_phase = match (carried, now) {
            (false, true) => Some(PHASE_ENTER),
            (true, true) => Some(PHASE_STAY),
            (true, false) => Some(PHASE_EXIT),
            (false, false) => None,
        };
        contact[p] = now;
        let Some(phase) = expected_phase else {
            continue;
        };
        if events.len() >= event_count as usize {
            return Err("rounds event count output".into());
        }
        let base = HEADER_LEN + PAIRS * BINDING_LEN + events.len() * EVENT_LEN;
        let row = TriggerEvent {
            pair: int_at(data, base)?,
            phase: int_at(data, base + 1)?,
            bind_generation: int_at(data, base + 2)?,
            source_kind: int_at(data, base + 3)?,
            source_ref: int_at(data, base + 4)?,
            source_template_index: int_at(data, base + 5)?,
            source_generation: int_at(data, base + 6)?,
            geometry: int_at(data, base + 7)?,
        };
        let name = |what: &str| format!("rounds event {what} output");
        if row.pair != p as i32 {
            return Err(name("pair"));
        }
        if row.phase != phase {
            return Err(name("phase"));
        }
        if row.bind_generation != binding.bind_generation {
            return Err(name("bind generation"));
        }
        if row.source_kind != binding.source_kind {
            return Err(name("source kind"));
        }
        let expected_ref = match binding.source_kind {
            0 => i32::try_from(binding.source_id).map_err(|_| name("source ref"))?,
            _ => {
                run_id * super::runtime_objects::RUN_STRIDE
                    + binding.source_ref * super::runtime_objects::GEN_SPAN
                    + binding.source_generation
            }
        };
        if row.source_ref != expected_ref {
            return Err(name("source ref"));
        }
        if row.source_template_index != binding.source_template_index {
            return Err(name("source template"));
        }
        if row.source_generation != binding.source_generation {
            return Err(name("source generation"));
        }
        if row.geometry != binding.shape {
            return Err(name("geometry"));
        }
        events.push(row);
    }
    if events.len() as i32 != event_count {
        return Err("rounds event count output".into());
    }
    Ok(RuntimeRoundFrame {
        dim: rules.dim,
        run_id,
        tick,
        evaluate_seq,
        restart_requested: restart_requested == 1,
        restart_tick,
        bindings,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::super::document::Dimension;
    use super::super::hierarchy::Node;
    use super::*;
    use crate::Scalar as S;
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

    /// One 2D run order: char quad (0), floor quad (1), circle leaf (2).
    /// Descriptor rows align with the order; the character sits at y 0.5 so
    /// its box spans y 0..1, and the circle overlaps it at the origin.
    fn fixture() -> (Vec<Node>, Vec<[f32; 7]>, Vec<[f32; 3]>) {
        let order = vec![node(1, 1, 2), node(2, 1, 2), node(3, 0, 2)];
        let rows = vec![
            [1.0, -0.4, -0.5, 0.0, 0.4, 0.5, 0.0],
            [1.0, -4.0, -1.0, 0.0, 34.0, 0.0, 0.0],
            [1.0, -0.4, -0.5, 0.0, 0.4, 0.5, 0.0],
        ];
        let worlds = vec![[0.0, 0.5, 0.0], [0.0; 3], [0.0; 3]];
        (order, rows, worlds)
    }

    type BRow = (i32, i32, i32, i32, i32, i32, i32, i32, f32);
    type ERow = (i32, i32, i32, i32, i32, i32, i32, i32);

    fn block(
        bindings: &[BRow; PAIRS],
        events: &[ERow],
        run: i32,
        tick: i32,
        count: i32,
        restart: (i32, i32),
        seq: i32,
    ) -> Vec<S> {
        let mut data = vec![
            Int(1),
            Int(run),
            Int(tick),
            Int(count),
            Int(restart.0),
            Int(restart.1),
            Int(seq),
            Int(0),
        ];
        for (bound, bg, sub, sk, sr, st, sg, shape, radius) in bindings {
            data.push(Int(*bound));
            data.push(Int(*bg));
            data.push(Int(*sub));
            data.push(Int(*sk));
            data.push(Int(*sr));
            data.push(Int(*st));
            data.push(Int(*sg));
            data.push(Int(*shape));
            data.push(Float(*radius));
        }
        for r in 0..PAIRS {
            match events.get(r) {
                Some((p, ph, bg, sk, sr, st, sg, geo)) => {
                    data.push(Int(*p));
                    data.push(Int(*ph));
                    data.push(Int(*bg));
                    data.push(Int(*sk));
                    data.push(Int(*sr));
                    data.push(Int(*st));
                    data.push(Int(*sg));
                    data.push(Int(*geo));
                }
                None => {
                    for _ in 0..EVENT_LEN {
                        data.push(Int(0));
                    }
                }
            }
        }
        data
    }

    /// pair 0: char (node 0) vs the circle leaf (node 2, static source).
    fn canon_bindings() -> [BRow; PAIRS] {
        let mut rows: [BRow; PAIRS] = [(0, 0, 0, 0, 0, 0, 0, 0, 0.0); PAIRS];
        rows[0] = (1, 1, 0, 0, 2, 2, 0, 0, 0.0);
        rows
    }

    fn rules<'a>(
        order: &'a [Node],
        rows: &'a [[f32; 7]],
        worlds: &'a [[f32; 3]],
    ) -> RoundRules<'a> {
        RoundRules {
            order,
            rows,
            worlds,
            dim: Dim::TwoD,
        }
    }

    fn enter_events() -> Vec<ERow> {
        vec![(0, PHASE_ENTER, 1, 0, 3, 2, 0, 0)]
    }

    #[test]
    fn overlaps_matches_closed_interval_and_closest_point() {
        let char_box = [1.0, -0.4, -0.5, 0.0, 0.4, 0.5, 0.0];
        let src_box = [1.0, -0.4, -0.5, 0.0, 0.4, 0.5, 0.0];
        assert!(overlaps(
            Dim::TwoD,
            SHAPE_AABB,
            0.0,
            &char_box,
            [0.0, 0.5, 0.0],
            &src_box,
            [0.8, 0.5, 0.0]
        ));
        let ulp = f32::from_bits(0.8f32.to_bits() + 1);
        assert!(!overlaps(
            Dim::TwoD,
            SHAPE_AABB,
            0.0,
            &char_box,
            [0.0, 0.5, 0.0],
            &src_box,
            [ulp, 0.5, 0.0]
        ));
        assert!(overlaps(
            Dim::TwoD,
            SHAPE_SPHERE,
            0.5,
            &char_box,
            [0.0, 0.5, 0.0],
            &src_box,
            [0.0, 1.5, 0.0]
        ));
        assert!(!overlaps(
            Dim::TwoD,
            SHAPE_SPHERE,
            0.5,
            &char_box,
            [0.0, 0.5, 0.0],
            &src_box,
            [0.75, 1.4, 0.0]
        ));
        assert!(!overlaps(
            Dim::TwoD,
            SHAPE_SPHERE,
            0.0,
            &char_box,
            [0.0, 0.5, 0.0],
            &src_box,
            [0.0, 0.5, 0.0]
        ));
    }

    #[test]
    fn tick0_registry_and_first_update_enter_admit() {
        let (order, rows, worlds) = fixture();
        let r = rules(&order, &rows, &worlds);
        let mut contact = [false; PAIRS];
        let mut water = [0i32; PAIRS];
        let f0 = admit_round_block(
            &block(&canon_bindings(), &[], 1, 0, 0, (0, 0), 0),
            1,
            0,
            &r,
            None,
            None,
            &mut contact,
            &mut water,
        )
        .expect("tick 0 registry");
        assert_eq!(f0.events.len(), 0);
        assert_eq!(f0.bindings[0].unwrap().subject_id, 1);
        assert!(!contact[0]);
        let f1 = admit_round_block(
            &block(&canon_bindings(), &enter_events(), 1, 1, 1, (0, 0), 1),
            1,
            1,
            &r,
            None,
            Some(&f0),
            &mut contact,
            &mut water,
        )
        .expect("tick 1 enter");
        assert_eq!(f1.events.len(), 1);
        assert_eq!(f1.events[0].phase, PHASE_ENTER);
        assert!(contact[0]);
        // The next frame under the same binding carries the stay.
        let stay = vec![(0, PHASE_STAY, 1, 0, 3, 2, 0, 0)];
        let f2 = admit_round_block(
            &block(&canon_bindings(), &stay, 1, 2, 1, (0, 0), 2),
            1,
            2,
            &r,
            None,
            Some(&f1),
            &mut contact,
            &mut water,
        )
        .expect("tick 2 stay");
        assert_eq!(f2.events[0].phase, PHASE_STAY);
    }

    #[test]
    fn every_named_round_counterexample_rejects() {
        let (order, rows, worlds) = fixture();
        let r = rules(&order, &rows, &worlds);
        let mut contact = [false; PAIRS];
        let mut water = [0i32; PAIRS];
        let f0 = admit_round_block(
            &block(&canon_bindings(), &[], 1, 0, 0, (0, 0), 0),
            1,
            0,
            &r,
            None,
            None,
            &mut contact,
            &mut water,
        )
        .unwrap();
        let cases: Vec<(&str, i32, i32, Vec<S>)> = vec![
            ("status", 1, 1, {
                let mut d = block(&canon_bindings(), &enter_events(), 1, 1, 1, (0, 0), 1);
                d[0] = Int(0);
                d
            }),
            (
                "run",
                2,
                1,
                block(&canon_bindings(), &enter_events(), 2, 1, 1, (0, 0), 1),
            ),
            (
                "tick",
                1,
                2,
                block(&canon_bindings(), &enter_events(), 1, 2, 1, (0, 0), 2),
            ),
            (
                "sequence",
                1,
                1,
                block(&canon_bindings(), &enter_events(), 1, 1, 1, (0, 0), 5),
            ),
            (
                "count-high",
                1,
                1,
                block(&canon_bindings(), &enter_events(), 1, 1, 2, (0, 0), 1),
            ),
            (
                "count-low",
                1,
                1,
                block(&canon_bindings(), &enter_events(), 1, 1, 0, (0, 0), 1),
            ),
            (
                "fake-exit",
                1,
                1,
                block(
                    &canon_bindings(),
                    &[(0, PHASE_EXIT, 1, 0, 3, 2, 0, 0)],
                    1,
                    1,
                    1,
                    (0, 0),
                    1,
                ),
            ),
            (
                "fake-stay",
                1,
                1,
                block(
                    &canon_bindings(),
                    &[(0, PHASE_STAY, 1, 0, 3, 2, 0, 0)],
                    1,
                    1,
                    1,
                    (0, 0),
                    1,
                ),
            ),
            (
                "old-generation",
                1,
                1,
                block(
                    &canon_bindings(),
                    &[(0, PHASE_ENTER, 0, 0, 3, 2, 0, 0)],
                    1,
                    1,
                    1,
                    (0, 0),
                    1,
                ),
            ),
            (
                "wrong-source-identity",
                1,
                1,
                block(
                    &canon_bindings(),
                    &[(0, PHASE_ENTER, 1, 0, 99, 2, 0, 0)],
                    1,
                    1,
                    1,
                    (0, 0),
                    1,
                ),
            ),
            (
                "wrong-template",
                1,
                1,
                block(
                    &canon_bindings(),
                    &[(0, PHASE_ENTER, 1, 0, 3, 1, 0, 0)],
                    1,
                    1,
                    1,
                    (0, 0),
                    1,
                ),
            ),
            (
                "wrong-geometry",
                1,
                1,
                block(
                    &canon_bindings(),
                    &[(0, PHASE_ENTER, 1, 0, 3, 2, 0, 1)],
                    1,
                    1,
                    1,
                    (0, 0),
                    1,
                ),
            ),
            (
                "restart-without-tick",
                1,
                1,
                block(&canon_bindings(), &[], 1, 1, 0, (1, 5), 1),
            ),
            ("canonical-violation", 1, 1, {
                let mut b = canon_bindings();
                b[3].0 = 1;
                block(&b, &[], 1, 1, 0, (0, 0), 1)
            }),
            ("subject-group", 1, 1, {
                let mut b = canon_bindings();
                b[0].2 = 5;
                block(&b, &[], 1, 1, 0, (0, 0), 1)
            }),
            ("subject-out-of-order", 1, 1, {
                let mut b = canon_bindings();
                b[0].2 = 9;
                block(&b, &[], 1, 1, 0, (0, 0), 1)
            }),
            ("radius-domain", 1, 1, {
                let mut b = canon_bindings();
                b[0].7 = 1;
                b[0].8 = 0.0;
                block(&b, &[], 1, 1, 0, (0, 0), 1)
            }),
        ];
        for (name, _run, _tick, data) in cases {
            let mut c2 = [false; PAIRS];
            let mut w2 = [0i32; PAIRS];
            let error = admit_round_block(&data, 1, 1, &r, None, Some(&f0), &mut c2, &mut w2)
                .expect_err(name);
            assert!(error.contains("rounds"), "{name}: {error}");
        }
        // The watermark: a first frame whose bind generation sits at or below
        // the admitted watermark is rejected.
        let mut w3 = [9i32; PAIRS];
        let mut c3 = [false; PAIRS];
        let error = admit_round_block(
            &block(&canon_bindings(), &enter_events(), 1, 1, 1, (0, 0), 1),
            1,
            1,
            &r,
            None,
            None,
            &mut c3,
            &mut w3,
        )
        .expect_err("watermark");
        assert!(error.contains("watermark"), "{error}");
        // A rebinding generation must change the configuration.
        let mut c4 = [false; PAIRS];
        let mut w4 = [0i32; PAIRS];
        let f0b = admit_round_block(
            &block(&canon_bindings(), &[], 1, 0, 0, (0, 0), 0),
            1,
            0,
            &r,
            None,
            None,
            &mut c4,
            &mut w4,
        )
        .unwrap();
        let mut rebound = canon_bindings();
        rebound[0] = (1, 1, 1, 0, 1, 1, 0, 0, 0.0);
        let error = admit_round_block(
            &block(&rebound, &[], 1, 1, 0, (0, 0), 1),
            1,
            1,
            &r,
            None,
            Some(&f0b),
            &mut c4,
            &mut w4,
        )
        .expect_err("same-generation-change");
        assert!(error.contains("within one generation"), "{error}");
    }

    #[test]
    fn pair_capacity_matches_the_objects_envelope() {
        assert!(pair_capacity_error(120).is_none());
        assert!(pair_capacity_error(121).is_some());
    }
}
