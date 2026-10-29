//! Independent admission of the finite Auto visual wire. A pose is part of
//! the same Frame as physics/camera/objects/events and never edits collision.
use super::{
    assets::{Asset, Registry},
    document::Document,
    hierarchy::Node,
    runtime_objects::{RuntimeObjectsFrame, SLOTS},
};
use crate::Scalar;
use glam::Mat4;
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct Target {
    pub node_index: i32,
    pub node_id: u64,
    pub asset_index: i32,
    pub asset: Arc<Asset>,
}
#[derive(Clone, Debug)]
pub struct Visual {
    pub asset: String,
    pub clip: i32,
    pub phase: f32,
    pub looping: bool,
    pub yaw: f32,
    pub worlds: Arc<[Mat4]>,
}
#[derive(Clone, Debug)]
pub struct SlotVisual {
    pub generation: i32,
    pub visual: Visual,
}
#[derive(Clone, Debug)]
pub struct AnimationFrame {
    pub run_id: i32,
    pub tick: i32,
    pub targets: Vec<(u64, Visual)>,
    pub slots: [Option<SlotVisual>; SLOTS],
}
impl AnimationFrame {
    pub fn visual_for(&self, id: u64) -> Option<&Visual> {
        self.targets
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, v)| v)
    }
}
pub fn block_len(count: usize) -> usize {
    4 + (count + SLOTS) * 6
}
pub fn targets(doc: &Document, order: &[Node], assets: &Registry) -> Result<Vec<Target>, String> {
    let mut out = Vec::new();
    for (node_index, n) in order.iter().enumerate() {
        let Some(o) = doc.objects.iter().find(|o| o.id == n.id) else {
            continue;
        };
        let Some(key) = &o.resource else { continue };
        if o.kind != super::document::Kind::Model {
            continue;
        }
        let (asset_index, identity) = doc
            .assets
            .iter()
            .enumerate()
            .find(|(_, a)| a.key() == *key)
            .ok_or("visual asset missing")?;
        let asset = assets.get(key).ok_or("visual payload missing")?;
        if !asset.identity.same_content(identity) || asset.image.is_some() {
            return Err("visual asset identity".into());
        }
        out.push(Target {
            node_index: node_index as i32,
            node_id: n.id,
            asset_index: asset_index as i32,
            asset: asset.clone(),
        });
    }
    if out.len() > 128 {
        return Err("resource limit: visual targets128".into());
    }
    // Bound both retained last-good/candidate pose owners plus prepare scratch.
    // Every potential slot pays the largest template, independent of visibility.
    let nodes = |t: &Target| t.asset.animation.as_ref().map_or(0, |a| a.nodes.len());
    let scratch = (out.iter().map(nodes).sum::<usize>()
        + SLOTS * out.iter().map(nodes).max().unwrap_or(0))
        * (48 + 64 * 4);
    let cpu = super::textured_models::usage(assets).2;
    if cpu
        .checked_add(scratch)
        .is_none_or(|n| n > super::assets::MAX_GEOMETRY)
    {
        return Err("resource limit: animation pose CPU union".into());
    }
    Ok(out)
}
pub fn admit_block(
    data: &[Scalar],
    run: i32,
    tick: i32,
    targets: &[Target],
    instances: Option<&RuntimeObjectsFrame>,
) -> Result<AnimationFrame, String> {
    if data.len() != block_len(targets.len()) {
        return Err("visual block length".into());
    }
    let int = |i| match data.get(i) {
        Some(Scalar::Int(v)) => Ok(*v),
        _ => Err("visual integer output".to_string()),
    };
    let float = |i| match data.get(i) {
        Some(Scalar::Float(v)) if v.is_finite() => Ok(*v),
        _ => Err("visual float output".to_string()),
    };
    if [int(0)?, int(1)?, int(2)?, int(3)?] != [1, targets.len() as i32, run, tick] {
        return Err("visual status/count/run/tick".into());
    }
    let visual = |base: usize, t: &Target| -> Result<Visual, String> {
        if int(base + 1)? != t.asset_index {
            return Err("visual asset ordinal".into());
        }
        let clip = int(base + 2)?;
        let looping = int(base + 3)?;
        let phase = float(base + 4)?;
        let raw_yaw = float(base + 5)?;
        let count = t.asset.animation.as_ref().map_or(0, |a| a.clips.len());
        if clip < -1 || clip >= count as i32 {
            return Err("visual clip ordinal".into());
        }
        if !matches!(looping, 0 | 1) || phase < 0. {
            return Err("visual phase/loop".into());
        }
        let yaw = agp::assets::node_animation::yaw(raw_yaw)?;
        let worlds = match &t.asset.animation {
            Some(a) => a
                .sample(
                    if clip < 0 { None } else { Some(clip as usize) },
                    phase,
                    looping == 1,
                )?
                .into(),
            None => Arc::from([]),
        };
        Ok(Visual {
            asset: t.asset.identity.key(),
            clip,
            phase,
            looping: looping == 1,
            yaw,
            worlds,
        })
    };
    let mut admitted = Vec::with_capacity(targets.len());
    for (i, t) in targets.iter().enumerate() {
        let base = 4 + i * 6;
        if int(base)? != t.node_index {
            return Err("visual node ordinal".into());
        }
        admitted.push((t.node_id, visual(base, t)?));
    }
    let mut slots: [Option<SlotVisual>; SLOTS] = Default::default();
    for s in 0..SLOTS {
        let base = 4 + (targets.len() + s) * 6;
        let slot = instances.and_then(|i| i.get(s));
        let target = slot.and_then(|i| targets.iter().find(|t| t.node_id == i.template_id));
        if let Some(t) = target {
            if int(base)? != slot.unwrap().generation {
                return Err("visual stale slot generation".into());
            }
            slots[s] = Some(SlotVisual {
                generation: int(base)?,
                visual: visual(base, t)?,
            });
        } else {
            if (0..4).any(|f| int(base + f) != Ok(0))
                || float(base + 4)? != 0.
                || float(base + 5)? != 0.
            {
                return Err("visual free slot canonical zeros".into());
            }
        }
    }
    Ok(AnimationFrame {
        run_id: run,
        tick,
        targets: admitted,
        slots,
    })
}
/// Undo the admitted geometry's base bake; both passes consume this matrix.
pub fn mesh_pose(v: Option<&Visual>, mesh: &agp::assets::MeshData) -> Result<Mat4, String> {
    let Some(v) = v else {
        return Ok(Mat4::IDENTITY);
    };
    let pose = if v.worlds.is_empty() {
        Mat4::IDENTITY
    } else {
        let meta = mesh
            .visual
            .as_ref()
            .ok_or("animated primitive association missing")?;
        let world = v
            .worlds
            .get(meta.node_index)
            .ok_or("animated node association missing")?;
        *world * Mat4::from_cols_array_2d(&meta.world).inverse()
    };
    let result = Mat4::from_rotation_y(v.yaw) * pose;
    if !result.is_finite()
        || !result.determinant().is_finite()
        || result.determinant() <= 0.
        || !result.inverse().is_finite()
    {
        return Err("invalid: visual mesh pose".into());
    }
    Ok(result)
}

pub fn mesh_matrix(
    base: Mat4,
    v: Option<&Visual>,
    mesh: &agp::assets::MeshData,
) -> Result<Mat4, String> {
    let matrix = base * mesh_pose(v, mesh)?;
    if v.is_some()
        && (!matrix.is_finite()
            || !matrix.determinant().is_finite()
            || matrix.determinant() <= 0.
            || !matrix.inverse().is_finite())
    {
        return Err("invalid: final visual mesh matrix".into());
    }
    Ok(matrix)
}
