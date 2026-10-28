//! Explicit full-config hierarchy profile 4; bounded private candidate VM.
use super::{
    document::{Document, Object},
    hierarchy::{self, Derived, Node},
};
use crate::{Progress, ResultKind, Scalar, Session};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
pub const LEGACY_SOURCE: &str = include_str!("../../samples/authoring/main.at");
pub const SOURCE: &str = include_str!("../../samples/authoring_hierarchy/main.at");
pub const ASSET_SOURCE: &str = include_str!("../../samples/authoring_assets/main.at");
struct Call {
    name: &'static str,
    args: Vec<Scalar>,
    kind: ResultKind,
}
pub struct Preview {
    pub objects: Vec<Object>,
    pub vm: Session,
    pub compile_us: u128,
    pub nodes: std::collections::BTreeMap<u64, Derived>,
    document: Document,
}
impl Preview {
    pub fn matches(&self, d: &Document) -> bool {
        self.document == *d
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
}
pub struct Building {
    vm: Session,
    queue: VecDeque<Call>,
    active: bool,
    expected: Document,
    order: Vec<Node>,
    read: Vec<f32>,
    asset_profile: bool,
}
impl Building {
    pub fn owner_probe(&self) -> Box<dyn Fn() -> bool> {
        Box::new(self.vm.owner_probe())
    }

    pub fn new(d: &Document, source: &str) -> Result<Self, String> {
        d.validate()?;
        let asset_profile = !d.assets.is_empty();
        let source = if asset_profile && source == SOURCE {
            ASSET_SOURCE
        } else {
            source
        };
        let vm = Session::new(source)?;
        let mut queue = VecDeque::new();
        queue.push_back(Call {
            name: "reset",
            args: vec![],
            kind: ResultKind::Unit,
        });
        let order = hierarchy::ordered(d)?;
        for (i, n) in order.iter().enumerate() {
            let parent = n
                .parent
                .map(|id| order.iter().position(|p| p.id == id).unwrap() as i32)
                .unwrap_or(-1);
            queue.push_back(Call {
                name: "identity",
                args: vec![
                    Scalar::Int(i as i32 + 1),
                    Scalar::Int(n.kind),
                    Scalar::Int(n.dimension.code()),
                    Scalar::Int(parent),
                ],
                kind: ResultKind::Unit,
            });
            queue.push_back(Call {
                name: "visibility",
                args: vec![Scalar::Int(i32::from(n.visible))],
                kind: ResultKind::Unit,
            });
            if asset_profile {
                queue.push_back(Call {
                    name: "bind_resource",
                    args: vec![Scalar::Int(resource_ordinal(d, n.id)?)],
                    kind: ResultKind::Unit,
                });
            }
            let mut fields = vec![];
            fields.extend(n.local);
            if let Some(o) = d.objects.iter().find(|o| o.id == n.id) {
                fields.extend(o.scale);
                fields.extend([o.yaw, o.pitch]);
                fields.extend(o.rgba);
            } else {
                fields.extend([1.0, 1.0, 1.0, 0.0, 0.0, 0.9, 0.3, 0.2, 1.0]);
            }
            for v in fields {
                queue.push_back(Call {
                    name: "append_field",
                    args: vec![Scalar::Float(v)],
                    kind: ResultKind::Unit,
                });
            }
        }
        queue.push_back(Call {
            name: "derive",
            args: vec![],
            kind: ResultKind::Unit,
        });
        for name in ["profile", "count"] {
            queue.push_back(Call {
                name,
                args: vec![],
                kind: ResultKind::Int,
            });
        }
        for i in 0..order.len() {
            for name in ["id", "kind", "dimension", "parent", "visible"] {
                queue.push_back(Call {
                    name,
                    args: vec![Scalar::Int(i as i32)],
                    kind: ResultKind::Int,
                });
            }
            if asset_profile {
                queue.push_back(Call {
                    name: "resource",
                    args: vec![Scalar::Int(i as i32)],
                    kind: ResultKind::Int,
                });
            }
            for f in 0..12 {
                queue.push_back(Call {
                    name: "field",
                    args: vec![Scalar::Int(i as i32), Scalar::Int(f)],
                    kind: ResultKind::Float,
                });
            }
        }
        Ok(Self {
            vm,
            queue,
            active: false,
            expected: d.clone(),
            order,
            read: vec![],
            asset_profile,
        })
    }
    pub fn pump(&mut self) -> Result<bool, String> {
        let deadline = Instant::now() + Duration::from_millis(8);
        loop {
            let Some(c) = self.queue.front() else {
                return Ok(true);
            };
            let p = if self.active {
                self.vm.pump()?
            } else {
                self.vm.start(c.name, &c.args, c.kind)?
            };
            match p {
                Progress::Pending => self.active = true,
                Progress::Complete(v) => {
                    if let Some(v) = v {
                        self.read.push(match v {
                            Scalar::Int(v) => v as f32,
                            Scalar::Float(v) => v,
                        })
                    }
                    self.queue.pop_front();
                    self.active = false;
                }
            }
            if Instant::now() >= deadline {
                return Ok(self.queue.is_empty());
            }
        }
    }
    pub fn finish(self) -> Result<Preview, String> {
        if !self.queue.is_empty() {
            return Err("preview still pending".into());
        }
        let r = &self.read;
        let stride = if self.asset_profile { 18 } else { 17 };
        if r.len() != 2 + self.order.len() * stride
            || r[0] != if self.asset_profile { 5.0 } else { 4.0 }
            || r[1] != self.order.len() as f32
        {
            return Err("authoring profile/count output".into());
        }
        let reference = hierarchy::derive(&self.expected)?;
        let mut nodes = std::collections::BTreeMap::new();
        let mut objects = self.expected.objects.clone();
        for (i, n) in self.order.iter().enumerate() {
            let raw = &r[2 + i * stride..2 + (i + 1) * stride];
            let a: Vec<f32> = if self.asset_profile {
                if raw[5] != resource_ordinal(&self.expected, n.id)? as f32 {
                    return Err("authoring wrong-resource ordinal".into());
                }
                raw[..5].iter().chain(&raw[6..]).copied().collect()
            } else {
                raw.to_vec()
            };
            let parent = n
                .parent
                .map(|id| self.order.iter().position(|p| p.id == id).unwrap() as i32)
                .unwrap_or(-1);
            if a[..5]
                != [
                    i as f32 + 1.0,
                    n.kind as f32,
                    n.dimension.code() as f32,
                    parent as f32,
                    i32::from(reference[&n.id].visible) as f32,
                ]
            {
                return Err("authoring output target/type/parent/visibility".into());
            }
            if a[5..8] != reference[&n.id].translation {
                return Err("authoring world accumulation mismatch".into());
            }
            let mut expected_fields = vec![];
            if let Some(o) = objects.iter_mut().find(|o| o.id == n.id) {
                expected_fields.extend(o.scale);
                expected_fields.extend([o.yaw, o.pitch]);
                expected_fields.extend(o.rgba);
                if a[8..17] != expected_fields {
                    return Err("authoring output/config mismatch".into());
                }
                // Render/pick this returned world/effective, never the CPU oracle.
                o.translation.copy_from_slice(&a[5..8]);
                o.visible = a[4] == 1.0;
                o.scale.copy_from_slice(&a[8..11]);
                o.yaw = a[11];
                o.pitch = a[12];
                o.rgba.copy_from_slice(&a[13..17]);
                if self.asset_profile && o.kind == super::document::Kind::Model {
                    o.resource = Some(self.expected.assets[(raw[5] as usize) - 5].key());
                }
            } else if a[8..17] != [1.0, 1.0, 1.0, 0.0, 0.0, 0.9, 0.3, 0.2, 1.0] {
                return Err("group output fields".into());
            }
            nodes.insert(
                n.id,
                Derived {
                    translation: [a[5], a[6], a[7]],
                    visible: a[4] == 1.0,
                },
            );
        }
        let compile_us = self.vm.compile_us;
        Ok(Preview {
            objects,
            vm: self.vm,
            compile_us,
            nodes,
            document: self.expected,
        })
    }
}
pub(crate) fn resource_ordinal(d: &Document, id: u64) -> Result<i32, String> {
    let Some(o) = d.objects.iter().find(|o| o.id == id) else {
        return Ok(-1);
    };
    if o.kind == super::document::Kind::Model {
        Ok(5 + d
            .assets
            .iter()
            .position(|a| Some(a.key()).as_ref() == o.resource.as_ref())
            .ok_or("resource mapping")? as i32)
    } else {
        Ok(o.kind as i32)
    }
}
pub fn build(d: &Document) -> Result<Preview, String> {
    let mut b = Building::new(d, SOURCE)?;
    while !b.pump()? {}
    b.finish()
}
