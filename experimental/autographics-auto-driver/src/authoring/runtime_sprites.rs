//! PLAN-004 finite 2D sprite visuals. One bound document leaf carries an
//! accepted PNG payload; the real Auto profile chooses that leaf frame
//! rectangle and horizontal flip every fixed tick, and the reserved runtime
//! slots carry the visual of their accepted template. The admitted block is a
//! fixed-length table that rides the same Play `Frame` as the character,
//! camera, objects and triggers, so a candidate is accepted or rejected as one
//! transaction: a new position can never publish beside an old frame.
//!
//! Identity rules (contract section 5.2): the image is identified by its
//! complete original SHA/length/format, never by a name; a frame is an integer
//! pixel rectangle with positive extent inside that image; the horizontal flip
//! only mirrors the sample axis and never changes the entity bounds. The host
//! re-validates identity, bounds, run and slot generation independently of the
//! trusted unit, so a mutated output cannot smuggle a wrong or stale visual.
use super::{
    document::{Document, SpriteBinding},
    hierarchy::Node,
    runtime_objects::{RuntimeObjectsFrame, MAX_GENERATION, SLOTS},
};

/// Bound static leaves per Run (finite support domain of this profile).
pub const MAX_TARGETS: usize = 8;
/// Header ints: status, bound count, run id, tick id.
pub const HEADER: usize = 4;
/// Values per row: node/template ordinal, asset ordinal, x, y, w, h, flip.
pub const ROW_FIELDS: usize = 7;
/// The frozen read layout of one Run: header, one row per bound leaf (the
/// table length is fixed for the Run by its accepted document) and 8 slot
/// rows. A document with no bound leaf still pays only the slot rows.
pub fn block_len(targets: usize) -> usize {
    HEADER + targets * ROW_FIELDS + SLOTS * ROW_FIELDS
}

/// One bound static leaf, frozen at Run construction from the accepted
/// document. The asset ordinal is the document manifest position; the image
/// dimensions come from the persisted identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub node_index: i32,
    pub node_id: u64,
    pub asset_index: i32,
    pub asset: String,
    pub width: u32,
    pub height: u32,
    pub frame: [u32; 4],
    pub flip: bool,
}

/// One admitted visual: the immutable image identity plus the chosen frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visual {
    pub asset: String,
    pub frame: [u32; 4],
    pub flip: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotVisual {
    pub generation: i32,
    pub visual: Visual,
}

/// The admitted sprite outputs of one tick. `targets` is indexed exactly like
/// the Run's `Target` list; `slots` is indexed by reserved slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteFrame {
    pub run_id: i32,
    pub tick: i32,
    /// Stable document ids of the bound leaves, in table order (frozen per Run).
    pub target_ids: Vec<u64>,
    pub targets: Vec<Visual>,
    pub slots: [Option<SlotVisual>; SLOTS],
}

impl SpriteFrame {
    /// The visual of one bound document leaf, by stable id.
    pub fn visual_for(&self, id: u64) -> Option<&Visual> {
        let index = self.target_ids.iter().position(|t| *t == id)?;
        self.targets.get(index)
    }
    pub fn slot_visual(&self, slot: usize) -> Option<&SlotVisual> {
        self.slots.get(slot).and_then(Option::as_ref)
    }
}

impl Target {
    /// The target of an accepted template, by document id.
    pub fn find<'a>(targets: &'a [Target], id: u64) -> Option<&'a Target> {
        targets.iter().find(|t| t.node_id == id)
    }
}

/// Freeze the bound-leaf table of one Run: every accepted 2D leaf carrying a
/// PNG binding, in run order. Legacy scenes without bindings produce an empty
/// table and the profile still runs with an empty sprite block.
pub fn targets(document: &Document, order: &[Node]) -> Result<Vec<Target>, String> {
    let mut out = vec![];
    for (index, node) in order.iter().enumerate() {
        let Some(object) = document.objects.iter().find(|o| o.id == node.id) else {
            continue;
        };
        let Some(binding) = &object.sprite else {
            continue;
        };
        if !object.kind.is_2d() {
            return Err("sprite binding on a 3D leaf".into());
        }
        let asset_index = document
            .assets
            .iter()
            .position(|a| a.key() == binding.asset)
            .ok_or("sprite asset missing from the manifest")?;
        let asset = &document.assets[asset_index];
        let (width, height) = match (asset.width, asset.height) {
            (Some(w), Some(h)) => (w, h),
            _ => return Err("sprite asset dimensions missing".into()),
        };
        out.push(Target {
            node_index: index as i32,
            node_id: node.id,
            asset_index: asset_index as i32,
            asset: binding.asset.clone(),
            width,
            height,
            frame: binding.frame,
            flip: binding.flip,
        });
    }
    if out.len() > MAX_TARGETS {
        return Err("sprite target budget".into());
    }
    Ok(out)
}

fn rect_ok(x: i32, y: i32, w: i32, h: i32, width: u32, height: u32) -> bool {
    if w <= 0 || h <= 0 {
        return false;
    }
    let (w, h) = (w as u32, h as u32);
    w <= width
        && h <= height
        && (x as i64) >= 0
        && (y as i64) >= 0
        && (x as u32) <= width - w
        && (y as u32) <= height - h
}

/// Validate the fixed sprite block of one tick against the frozen target table,
/// the accepted instance table and the registry identity. Every rejection names
/// its stage; a rejected candidate keeps the last complete frame.
pub fn admit_block(
    data: &[crate::Scalar],
    run_id: i32,
    tick: i32,
    targets: &[Target],
    dims: &dyn Fn(&str) -> Option<(u32, u32)>,
    instances: Option<&RuntimeObjectsFrame>,
) -> Result<SpriteFrame, String> {
    let int = |i: usize| -> Result<i32, String> {
        match data.get(i) {
            Some(crate::Scalar::Int(v)) => Ok(*v),
            _ => Err("sprite integer output".into()),
        }
    };
    if int(0)? != 1 {
        return Err("sprite status output".into());
    }
    if int(1)? != targets.len() as i32 {
        return Err("sprite bound count output".into());
    }
    if int(2)? != run_id || int(3)? != tick {
        return Err("sprite run/tick output".into());
    }
    // The registry must still hold exactly the persisted identity: a mutated or
    // replaced payload can never be painted as this binding.
    for t in targets {
        if dims(&t.asset) != Some((t.width, t.height)) {
            return Err("sprite asset identity/registry".into());
        }
    }
    let mut admitted = Vec::with_capacity(targets.len());
    for (i, t) in targets.iter().enumerate() {
        let base = HEADER + i * ROW_FIELDS;
        if int(base)? != t.node_index {
            return Err(format!("sprite row {i} node output"));
        }
        if int(base + 1)? != t.asset_index {
            return Err(format!("sprite row {i} asset output"));
        }
        let [x, y, w, h] = [
            int(base + 2)?,
            int(base + 3)?,
            int(base + 4)?,
            int(base + 5)?,
        ];
        if !rect_ok(x, y, w, h, t.width, t.height) {
            return Err(format!("sprite row {i} frame bounds"));
        }
        let flip = int(base + 6)?;
        if !matches!(flip, 0 | 1) {
            return Err(format!("sprite row {i} flip domain"));
        }
        admitted.push(Visual {
            asset: t.asset.clone(),
            frame: [x as u32, y as u32, w as u32, h as u32],
            flip: flip == 1,
        });
    }
    let slot_base = HEADER + targets.len() * ROW_FIELDS;
    let mut slots: [Option<SlotVisual>; SLOTS] = Default::default();
    for (s, row) in slots.iter_mut().enumerate() {
        let base = slot_base + s * ROW_FIELDS;
        let generation = int(base)?;
        if generation == 0 {
            for f in 1..ROW_FIELDS {
                if int(base + f)? != 0 {
                    return Err(format!("sprite slot {s} free row must be canonical zeros"));
                }
            }
            continue;
        }
        if !(1..=MAX_GENERATION).contains(&generation) {
            return Err(format!("sprite slot {s} generation domain"));
        }
        let instance = instances
            .and_then(|i| i.get(s).copied())
            .ok_or_else(|| format!("sprite slot {s} without a live instance"))?;
        if instance.generation != generation {
            return Err(format!("sprite slot {s} stale generation"));
        }
        let template = Target::find(targets, instance.template_id)
            .ok_or_else(|| format!("sprite slot {s} template has no visual"))?;
        if int(base + 1)? != template.asset_index {
            return Err(format!("sprite slot {s} asset output"));
        }
        let [x, y, w, h] = [
            int(base + 2)?,
            int(base + 3)?,
            int(base + 4)?,
            int(base + 5)?,
        ];
        if !rect_ok(x, y, w, h, template.width, template.height) {
            return Err(format!("sprite slot {s} frame bounds"));
        }
        let flip = int(base + 6)?;
        if !matches!(flip, 0 | 1) {
            return Err(format!("sprite slot {s} flip domain"));
        }
        *row = Some(SlotVisual {
            generation,
            visual: Visual {
                asset: template.asset.clone(),
                frame: [x as u32, y as u32, w as u32, h as u32],
                flip: flip == 1,
            },
        });
    }
    // Every live instance of a bound template must carry its row: a silent
    // default would let presentation disagree with the base leaf binding.
    if let Some(instances) = instances {
        for (s, slot) in instances.slots.iter().enumerate() {
            if !slot.live() {
                continue;
            }
            if Target::find(targets, slot.template_id).is_some() && slots[s].is_none() {
                return Err(format!("sprite slot {s} missing row"));
            }
        }
    }
    Ok(SpriteFrame {
        run_id,
        tick,
        target_ids: targets.iter().map(|t| t.node_id).collect(),
        targets: admitted,
        slots,
    })
}

/// The persisted binding of one document leaf.
pub fn binding(document: &Document, id: u64) -> Option<&SpriteBinding> {
    document
        .objects
        .iter()
        .find(|o| o.id == id)
        .and_then(|o| o.sprite.as_ref())
}
