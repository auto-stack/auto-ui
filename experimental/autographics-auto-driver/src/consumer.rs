use crate::output::Packet;
use agp::{
    core::{
        id::ObjectId,
        scene::{Color, Scene, SceneChange, Transform},
    },
    renderer::pipeline::{DrawCmd, Shape},
};
use glam::Vec3;
use std::collections::BTreeMap;
/// No simulation: validated data -> AGP identities, transforms and draw data.
pub struct SceneConsumer {
    pub scene: Scene,
    slots: BTreeMap<u32, (u32, ObjectId, u8)>,
}
impl Default for SceneConsumer {
    fn default() -> Self {
        Self {
            scene: Scene::new(),
            slots: BTreeMap::new(),
        }
    }
}
impl SceneConsumer {
    pub fn sync(&mut self, p: &Packet) -> Result<(), String> {
        p.validate()?;
        let mut changes = Vec::new();
        let dead: Vec<_> = self
            .slots
            .iter()
            .filter(|(slot, (g, _, _))| {
                !p.objects
                    .iter()
                    .any(|o| o.slot == **slot && o.generation == *g)
            })
            .map(|(s, v)| (*s, v.1))
            .collect();
        for (slot, id) in dead {
            changes.push(SceneChange::Destroy(id));
            self.slots.remove(&slot);
        }
        let mut born = Vec::new();
        for o in &p.objects {
            let v = o.values;
            let t = Transform {
                translation: Vec3::new(v[0], v[1], v[2]),
                scale: Vec3::new(v[3], v[4], v[5]),
                ..Default::default()
            };
            let color = Color::rgba(v[6], v[7], v[8], v[9]);
            if let Some((_, id, shape)) = self.slots.get_mut(&o.slot) {
                *shape = o.shape;
                changes.extend([
                    SceneChange::SetTransform(*id, t),
                    SceneChange::SetColor(*id, color),
                ]);
            } else {
                born.push((o.slot, o.generation, o.shape));
                changes.push(SceneChange::Spawn {
                    transform: t,
                    color,
                });
            }
        }
        let errors = self.scene.submit(&changes);
        if !errors.is_empty() {
            return Err(format!("consumer invariant: {errors:?}"));
        }
        let objects = self.scene.objects();
        let offset = objects.len() - born.len();
        for ((slot, g, shape), obj) in born.into_iter().zip(&objects[offset..]) {
            self.slots.insert(slot, (g, obj.id, shape));
        }
        Ok(())
    }
    pub fn draws(&self, size: [f32; 2]) -> Vec<DrawCmd> {
        self.slots
            .values()
            .filter(|(_, _, shape)| *shape != 2)
            .filter_map(|(_, id, shape)| {
                self.scene
                    .objects()
                    .iter()
                    .find(|o| o.id == *id)
                    .map(|o| DrawCmd {
                        center: [
                            size[0] * 0.5 + o.transform.translation.x,
                            size[1] * 0.5 - o.transform.translation.y,
                        ],
                        size: [o.transform.scale.x, o.transform.scale.y],
                        z: 0.0,
                        color: o.color,
                        shape: if *shape == 0 {
                            Shape::Circle
                        } else {
                            Shape::Rect
                        },
                    })
            })
            .collect()
    }
    pub fn object(&self, slot: u32) -> Option<&agp::core::scene::ObjectState> {
        let (_, id, _) = self.slots.get(&slot)?;
        self.scene.objects().iter().find(|o| o.id == *id)
    }
    pub fn meshes(&self) -> Vec<&agp::core::scene::ObjectState> {
        self.slots
            .values()
            .filter(|(_, _, shape)| *shape == 2)
            .filter_map(|(_, id, _)| self.scene.objects().iter().find(|o| o.id == *id))
            .collect()
    }
}
