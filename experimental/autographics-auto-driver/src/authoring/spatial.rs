//! Read-only, Run-scoped geometry. No script output or vertex clone is used here.
use super::{
    document::{Document, Layer},
    hierarchy, interaction,
};
use agp::assets::ModelData;
use glam::Vec3;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};
static VERTEX_SCANS: AtomicU64 = AtomicU64::new(0);
pub fn vertex_scans() -> u64 {
    VERTEX_SCANS.load(Ordering::Relaxed)
}

fn local_bounds(m: &ModelData) -> Option<(Vec3, Vec3)> {
    VERTEX_SCANS.fetch_add(1, Ordering::Relaxed);
    interaction::bounds(m)
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Descriptor {
    base_hash: String,
    resource_hashes: Vec<Option<String>>,
    boxes: Vec<[f32; 7]>,
    /// PLAN-058 T-03: per ordered row layer. Not serialized: the identity is
    /// already pinned by `base_hash` (the document, which includes the layer
    /// field) and by the admitted box rows, so legacy descriptor bytes and
    /// hashes stay identical.
    #[serde(skip)]
    layers: Vec<Layer>,
    pub local_scans: usize,
}
impl Descriptor {
    pub fn build<'a>(
        d: &Document,
        model: impl Fn(&super::document::Object) -> Option<&'a ModelData>,
    ) -> Result<Self, String> {
        d.validate()?;
        let order = hierarchy::ordered(d)?;
        let mut cache = BTreeMap::new();
        let mut local_scans = 0;
        let mut boxes = Vec::new();
        let mut resource_hashes = Vec::new();
        let mut layers = Vec::new();
        for n in order {
            let mut row = [0.; 7];
            let mut resource = None;
            let mut layer = Layer::Collision;
            if let Some(o) = d.objects.iter().find(|o| o.id == n.id) {
                let offsets = if o.kind.is_2d() {
                    Some((
                        Vec3::new(-o.scale[0] * 0.5, -o.scale[1] * 0.5, 0.),
                        Vec3::new(o.scale[0] * 0.5, o.scale[1] * 0.5, 0.),
                    ))
                } else {
                    let m = model(o).ok_or("missing actual spatial resource")?;
                    resource = Some(m.source_sha256.clone());
                    // Keys include the fixed role as well as content identity. Imported
                    // resources are accepted immutable ModelData from the Registry.
                    let key = format!("{}:{}", o.kind.key(), m.source_sha256);
                    let local = *cache.entry(key).or_insert_with(|| {
                        local_scans += 1;
                        local_bounds(m)
                    });
                    local.and_then(|(lo, hi)| {
                        let mut static_object = o.clone();
                        static_object.translation = [0.; 3];
                        let t = interaction::transform(&static_object);
                        let mut a = Vec3::splat(f32::INFINITY);
                        let mut b = Vec3::splat(f32::NEG_INFINITY);
                        for x in [lo.x, hi.x] {
                            for y in [lo.y, hi.y] {
                                for z in [lo.z, hi.z] {
                                    let p = t.transform_point3(Vec3::new(x, y, z));
                                    if !p.is_finite() {
                                        return None;
                                    }
                                    a = a.min(p);
                                    b = b.max(p);
                                }
                            }
                        }
                        Some((a, b))
                    })
                };
                if let Some((lo, hi)) = offsets {
                    row = [1., lo.x, lo.y, lo.z, hi.x, hi.y, hi.z];
                }
                // PLAN-058 T-03: the declared layer rides the row table, but
                // the geometry stays the document's real AABB. Only the
                // render-only-aware query units consult the layer, so old
                // units keep every row a candidate exactly as before.
                layer = o.layer.unwrap_or(Layer::Collision);
            }
            boxes.push(row);
            resource_hashes.push(resource);
            layers.push(layer);
        }
        Ok(Self {
            base_hash: d.hash()?,
            resource_hashes,
            boxes,
            layers,
            local_scans,
        })
    }
    pub fn rows(&self) -> &[[f32; 7]] {
        &self.boxes
    }
    /// PLAN-058 T-03: the public query candidate table — ordered row indices
    /// that take part in the pinned v3/v4 nearest-hit/selections. A
    /// `render-only` row is presented but is **not** in this table.
    pub fn query_candidates(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter(|(_, layer)| layer.is_collision())
            .map(|(i, _)| i)
            .collect()
    }
    /// PLAN-058 T-03: the declared layer of one ordered row (`Collision` for
    /// legacy documents that carry no layer field).
    pub fn row_layer(&self, index: usize) -> Option<Layer> {
        self.layers.get(index).copied()
    }
    pub fn is_query_candidate(&self, index: usize) -> bool {
        self.layers
            .get(index)
            .copied()
            .is_some_and(Layer::is_collision)
    }
    pub fn hash(&self) -> String {
        agp::renderer::capture::sha256_hex(
            &serde_json::to_vec(&("aabb-TRyRxS-f32/1", self)).expect("finite descriptor"),
        )
    }
    pub fn validate(&self, d: &Document) -> Result<(), String> {
        if self.base_hash != d.hash()?
            || self.boxes.len() != hierarchy::ordered(d)?.len()
            || self.boxes.len() > 128
        {
            return Err("spatial base/ordinal identity drift".into());
        }
        Ok(())
    }
}
