//! Finite authoring forest. CPU results are admission oracles, never renderer input.
use super::document::{Dimension, Document};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: u64,
    pub parent: Option<u64>,
    pub dimension: Dimension,
    pub kind: i32,
    pub local: [f32; 3],
    pub visible: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Derived {
    pub translation: [f32; 3],
    pub visible: bool,
}
pub fn ordered(d: &Document) -> Result<Vec<Node>, String> {
    if d.groups.len() + d.objects.len() > 128 {
        return Err("node resource limit 128".into());
    }
    let nodes: Vec<_> = d
        .groups
        .iter()
        .map(|g| Node {
            id: g.id,
            parent: g.parent,
            dimension: g.dimension,
            kind: 5,
            local: g.translation,
            visible: g.visible,
        })
        .chain(d.objects.iter().map(|o| Node {
            id: o.id,
            parent: o.parent,
            dimension: Dimension::of(o.kind),
            kind: o.kind as i32,
            local: o.translation,
            visible: o.visible,
        }))
        .collect();
    let mut all = BTreeSet::new();
    for n in &nodes {
        if n.id == 0 || !all.insert(n.id) {
            return Err("duplicate/invalid node identity".into());
        }
    }
    for n in &nodes {
        if let Some(parent) = n.parent {
            let p = nodes
                .iter()
                .find(|p| p.id == parent)
                .ok_or("graph missing parent")?;
            if p.kind != 5 {
                return Err("graph leaf cannot be parent".into());
            }
            if p.dimension != n.dimension {
                return Err("graph cross dimension".into());
            }
        }
    }
    // Iterative parent-first traversal, roots/siblings in declared group then object order.
    let mut stack: Vec<_> = nodes
        .iter()
        .rev()
        .filter(|n| n.parent.is_none())
        .cloned()
        .collect();
    let mut out = Vec::new();
    while let Some(n) = stack.pop() {
        stack.extend(
            nodes
                .iter()
                .rev()
                .filter(|c| c.parent == Some(n.id))
                .cloned(),
        );
        out.push(n);
    }
    if out.len() != nodes.len() {
        return Err("graph cycle".into());
    }
    Ok(out)
}
pub fn derive(d: &Document) -> Result<BTreeMap<u64, Derived>, String> {
    let mut out: BTreeMap<u64, Derived> = BTreeMap::new();
    for n in ordered(d)? {
        let mut v = Derived {
            translation: n.local,
            visible: n.visible,
        };
        if let Some(p) = n.parent {
            let parent = out.get(&p).ok_or("parent ordering")?;
            for axis in 0..3 {
                v.translation[axis] += parent.translation[axis];
            }
            v.visible &= parent.visible;
        }
        if v.translation
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 100000.0)
        {
            return Err("derived world numeric range".into());
        }
        out.insert(n.id, v);
    }
    Ok(out)
}
pub fn descendants(d: &Document, id: u64) -> Result<BTreeSet<u64>, String> {
    let nodes = ordered(d)?;
    if !d.contains(id) {
        return Err("target missing".into());
    }
    let mut ids = BTreeSet::from([id]);
    for n in nodes {
        if n.parent.is_some_and(|p| ids.contains(&p)) {
            ids.insert(n.id);
        }
    }
    Ok(ids)
}
pub fn reparent(
    d: &Document,
    id: u64,
    old: Option<u64>,
    parent: Option<u64>,
) -> Result<Document, String> {
    if d.parent(id)? != old {
        return Err("stale parent".into());
    }
    if old == parent {
        return Ok(d.clone());
    }
    let before = derive(d)?;
    let children = descendants(d, id)?;
    if parent.is_some_and(|p| children.contains(&p)) {
        return Err("graph cycle".into());
    }
    let p = match parent {
        Some(p) => before.get(&p).ok_or("graph missing parent")?.translation,
        None => [0.0; 3],
    };
    let local = std::array::from_fn(|i| before[&id].translation[i] - p[i]);
    let mut next = d.clone();
    if let Some(o) = next.objects.iter_mut().find(|o| o.id == id) {
        o.parent = parent;
        o.translation = local;
    } else {
        let g = next
            .groups
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or("target missing")?;
        g.parent = parent;
        g.translation = local;
    }
    next.validate()?;
    let after = derive(&next)?;
    for child in children {
        for axis in 0..3 {
            let a = before[&child].translation[axis];
            let b = after[&child].translation[axis];
            let tolerance = 1e-3f32.max(8.0 * f32::EPSILON * a.abs().max(b.abs()).max(1.0));
            if (a - b).abs() > tolerance {
                return Err("keep-world precision rejected".into());
            }
        }
    }
    Ok(next)
}
