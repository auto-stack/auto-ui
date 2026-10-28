//! Scene model and change submission (GPU-agnostic).
//!
//! Game logic builds `SceneChange`s and submits them; the scene validates
//! every identity against the generational allocator. Invalid (stale /
//! cross-session / unknown) changes are returned as errors and are NOT
//! applied — a rejected change never partially mutates the scene
//! (PLAN-829 §5.3 / failure policy).

use crate::core::id::{IdAllocator, IdentityError, ObjectId};
use glam::{Mat4, Quat, Vec3};

/// Linear sRGB-ish color, 0..1 range (the frozen samples author colors as
/// plain sRGB triples; gamma handling is the renderer's business).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

/// Object transform (2D-friendly; 3D uses the full rotation quaternion).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    /// Scale applied BEFORE rotation/translation (matches the frozen
    /// Breakout sprite-scale idiom).
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

/// Per-object renderable state.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectState {
    pub id: ObjectId,
    pub transform: Transform,
    pub color: Color,
    /// Optional handle into the renderer's resource cache (texture, etc.).
    pub resource: Option<ObjectId>,
    /// Bumped only when THIS object's resource binding changes. Untouched
    /// objects keep their revision — the pure-state "no rebuild" signal the
    /// contract tests assert on.
    pub resource_revision: u64,
}

/// A validated scene mutation.
#[derive(Debug, Clone, PartialEq)]
pub enum SceneChange {
    /// Spawn a new object; the allocator mints the id.
    Spawn {
        transform: Transform,
        color: Color,
    },
    Destroy(ObjectId),
    SetTransform(ObjectId, Transform),
    SetColor(ObjectId, Color),
    /// Bind/unbind a resource identity to an object.
    SetResource(ObjectId, Option<ObjectId>),
}

/// Why a change was rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum SceneError {
    InvalidId { id: ObjectId, reason: IdentityError },
    UnknownObject(ObjectId),
}

/// Objects + allocator; the single authority on identity validity.
#[derive(Debug)]
pub struct Scene {
    alloc: IdAllocator,
    objects: Vec<ObjectState>,
    /// indexes[id.index] = position in `objects` (or None when dead).
    slots: Vec<Option<usize>>,
    /// First object id ever spawned in this session (review r4 F829-R4-01:
    /// stable role anchor — survives destruction and index reuse; the
    /// squash projection classifies the player by this identity, not by
    /// the mutable alive-set head).
    first_spawned: Option<ObjectId>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            alloc: IdAllocator::new(),
            objects: Vec::new(),
            slots: Vec::new(),
            first_spawned: None,
        }
    }

    pub fn session(&self) -> crate::core::id::SessionId {
        self.alloc.session()
    }

    /// Scene bound to an existing session (change-log replay path).
    pub fn with_session(session: crate::core::id::SessionId) -> Self {
        Scene {
            alloc: IdAllocator::with_session(session),
            objects: Vec::new(),
            slots: Vec::new(),
            first_spawned: None,
        }
    }

    /// Submit a batch of changes. Applies changes until the first invalid
    /// one; returns the list of rejections. Rejected changes have no effect.
    pub fn submit(&mut self, changes: &[SceneChange]) -> Vec<SceneError> {
        let mut errors = Vec::new();
        for change in changes {
            match self.apply(change) {
                Ok(()) => {}
                Err(e) => errors.push(e),
            }
        }
        errors
    }

    fn apply(&mut self, change: &SceneChange) -> Result<(), SceneError> {
        match *change {
            SceneChange::Spawn { transform, color } => {
                let id = self.alloc.allocate();
                if self.first_spawned.is_none() {
                    self.first_spawned = Some(id);
                }
                let index = id.index() as usize;
                if self.slots.len() <= index {
                    self.slots.resize(index + 1, None);
                }
                self.slots[index] = Some(self.objects.len());
                self.objects.push(ObjectState {
                    id,
                    transform,
                    color,
                    resource: None,
                    resource_revision: 0,
                });
                Ok(())
            }
            SceneChange::Destroy(id) => self.with_object(id, |scene, index| {
                scene
                    .alloc
                    .free(id)
                    .map_err(|reason| SceneError::InvalidId { id, reason })?;
                scene.objects.remove(index);
                // Every object after `index` shifted down one position: repair
                // ALL slot indexes from `index` onward (not just the gap).
                for (pos, o) in scene.objects.iter().enumerate().skip(index) {
                    scene.slots[o.id.index() as usize] = Some(pos);
                }
                scene.slots[id.index() as usize] = None;
                Ok(())
            }),
            SceneChange::SetTransform(id, t) => self.with_object_mut(id, |o| o.transform = t),
            SceneChange::SetColor(id, c) => self.with_object_mut(id, |o| o.color = c),
            SceneChange::SetResource(id, r) => self.with_object_mut(id, |o| {
                o.resource = r;
                o.resource_revision += 1;
            }),
        }
    }

    fn with_object_mut(
        &mut self,
        id: ObjectId,
        f: impl FnOnce(&mut ObjectState),
    ) -> Result<(), SceneError> {
        self.with_object(id, |scene, index| {
            f(&mut scene.objects[index]);
            Ok(())
        })
    }

    fn with_object(
        &mut self,
        id: ObjectId,
        f: impl FnOnce(&mut Scene, usize) -> Result<(), SceneError>,
    ) -> Result<(), SceneError> {
        if let Err(reason) = self.alloc.validate(id) {
            return Err(SceneError::InvalidId { id, reason });
        }
        match self.slots.get(id.index() as usize).copied().flatten() {
            Some(index) => f(self, index),
            None => Err(SceneError::UnknownObject(id)),
        }
    }

    pub fn objects(&self) -> &[ObjectState] {
        &self.objects
    }

    /// The first object id ever spawned in this session (stable role
    /// anchor; survives destruction and index reuse within the session).
    pub fn first_spawned(&self) -> Option<ObjectId> {
        self.first_spawned
    }

    pub fn live_count(&self) -> usize {
        self.objects.len()
    }

    pub fn validate_id(&self, id: ObjectId) -> Result<(), IdentityError> {
        self.alloc.validate(id)
    }

    /// Canonical, session-independent state string. Two scenes with equal
    /// snapshots are observationally identical (ids compared by index +
    /// generation only — sessions are process-unique by design and never
    /// part of the replay-equality contract).
    ///
    /// Includes the FULL transform: translation, rotation (review r4
    /// F829-R4-04 — the field the renderer consumes via `to_matrix`), scale,
    /// color and resource. Quaternions are sign-canonicalized (negated when
    /// w < 0) so matrix-equivalent orientations snapshot equal.
    pub fn snapshot(&self) -> String {
        let mut parts: Vec<String> = self
            .objects
            .iter()
            .map(|o| {
                let t = o.transform;
                // Sign-canonicalize (q ~ -q is the same rotation): negate
                // when the first non-zero component is negative, so
                // matrix-equivalent orientations snapshot equal.
                let mut q = t.rotation;
                for c in [q.w, q.x, q.y, q.z] {
                    if c != 0.0 {
                        if c.is_sign_negative() {
                            q = -q;
                        }
                        break;
                    }
                }
                // normalize -0.0 so signed zeros print identically.
                q.x += 0.0;
                q.y += 0.0;
                q.z += 0.0;
                q.w += 0.0;
                format!(
                    "i{}g{}|t({:.9},{:.9},{:.9})|q({:.9},{:.9},{:.9},{:.9})|s({:.9},{:.9},{:.9})|c({:.9},{:.9},{:.9},{:.9})|r{}|rev{}",
                    o.id.index(),
                    o.id.generation(),
                    t.translation.x, t.translation.y, t.translation.z,
                    q.x, q.y, q.z, q.w,
                    t.scale.x, t.scale.y, t.scale.z,
                    o.color.r, o.color.g, o.color.b, o.color.a,
                    o.resource
                        .map(|r| format!("i{}g{}", r.index(), r.generation()))
                        .unwrap_or_else(|| "-".into()),
                    o.resource_revision,
                )
            })
            .collect();
        parts.sort();
        parts.join(";")
    }

    /// Full reset: fresh session/allocator, empty object set. Old identities
    /// (any session) are invalid against the new scene.
    pub fn reset(&mut self) {
        *self = Scene::new();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_then_destroy_then_stale_rejected() {
        let mut scene = Scene::new();
        let errs = scene.submit(&[SceneChange::Spawn {
            transform: Transform::default(),
            color: Color::rgb(1.0, 0.0, 0.0),
        }]);
        assert!(errs.is_empty());
        assert_eq!(scene.live_count(), 1);
        let id = scene.objects()[0].id;

        let errs = scene.submit(&[SceneChange::Destroy(id)]);
        assert!(errs.is_empty());
        assert_eq!(scene.live_count(), 0);

        // Late change with the dead id must be rejected and change nothing.
        let errs = scene.submit(&[SceneChange::SetColor(id, Color::rgb(0.0, 1.0, 0.0))]);
        assert_eq!(errs.len(), 1);
    }

    /// F829-R4-04: the canonical snapshot must include orientation — the
    /// renderer consumes `Transform::to_matrix`, so rotation-only state
    /// changes are observationally different even when translation/scale/
    /// color/resource are unchanged.
    #[test]
    fn r4_snapshot_distinguishes_rotation_only_changes() {
        let mut scene = Scene::new();
        scene.submit(&[SceneChange::Spawn {
            transform: Transform::default(),
            color: Color::rgb(1.0, 1.0, 1.0),
        }]);
        let id = scene.objects()[0].id;
        let before = scene.snapshot();
        assert_eq!(before, scene.snapshot(), "unchanged state stays equal");

        // bound probe: rotationY = 180° (Quat(0,1,0,0)), nothing else moves.
        scene.submit(&[SceneChange::SetTransform(
            id,
            Transform {
                rotation: Quat::from_xyzw(0.0, 1.0, 0.0, 0.0),
                ..Default::default()
            },
        )]);
        let after = scene.snapshot();
        assert_ne!(
            before, after,
            "rotation-only edit must produce a distinguishable snapshot"
        );

        // matrices differ exactly because the orientation differs.
        let m_before = Transform::default().to_matrix();
        let m_after = scene.objects()[0].transform.to_matrix();
        assert_ne!(m_before, m_after);

        // legitimate equivalent: the sign-flipped quat is the same rotation
        // (same matrix) and must snapshot EQUAL (sign canonicalization).
        scene.submit(&[SceneChange::SetTransform(
            id,
            Transform {
                rotation: Quat::from_xyzw(0.0, -1.0, 0.0, -0.0),
                ..Default::default()
            },
        )]);
        assert_eq!(
            after,
            scene.snapshot(),
            "matrix-equivalent orientation must snapshot equal"
        );

        // resetting the rotation back to identity restores the snapshot.
        scene.submit(&[SceneChange::SetTransform(id, Transform::default())]);
        assert_eq!(before, scene.snapshot());
    }
}
