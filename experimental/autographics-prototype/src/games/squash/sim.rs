//! AG-3D-01 Squash the Creeps — game logic, faithful to the frozen sample
//! (`godotengine/godot-demo-projects@b761b4cd5718a7118f2c95eb55f181d0a5f004f1`,
//! `3d/squash_the_creeps/`). Fidelity notes live in
//! `research/sample-manifest.md` §2, `research/3d-feasibility.md`, and
//! evidence notes E1–E4/E10.
//!
//! Deliberate frozen-source semantics (NOT generic game memory):
//! - Mobs fly STRAIGHT (collision_mask = undefined layer, Mob.tscn line 48) —
//!   there is NO wall bounce; they despawn via VisibleOnScreenNotifier3D
//!   (frustum test below), approximating screen-exit with the frozen AABB.
//! - squash() (Mob.gd lines 35–37) is INSTANT despawn + signal — there is no
//!   squash scale animation in the frozen source.
//! - Spawn aim targets the PLAYER'S ACTUAL POSITION at spawn time
//!   (Main.gd line 36 passes `$Player.position`; Mob.gd line 18 aims at it).
//! - Death has TWO frozen channels (Player.gd lines 53–62 + 73–74,
//!   Player.tscn lines 63–83): the body slide-collision squash check
//!   (normal·UP > 0.1 ⇒ squash+bounce) and the MobDetector Area3D with its
//!   five cylinders — a mob overlapping any detector cylinder is fatal even
//!   when the body sphere does not touch the mob box.
//! - Physics: 120 Hz fixed step; move_and_slide approximated by single-pass
//!   resolve (3d-feasibility.md §3, item 4 — documented approximation).
//!
//! F11 (review r1): game state goes through the G-base shared contract —
//! player and mobs hold session-scoped object IDs in the Runtime scene;
//! spawn/move/animate/squash/despawn submit scene deltas; renderers consume
//! a projection of the shared scene (`project_instances`), not private
//! fields. Kind projection: the player's object is the first spawned in a
//! session (index 0); every later object is a mob (documented and covered by
//! `shared_contract_reconstructs_3d_scene`).

use crate::core::id::ObjectId;
use crate::core::runtime::{Driver, Runtime};
use crate::core::scene::{Scene, Transform};
use crate::core::DeterministicRng;
use crate::render3d::collision::{
    clamp_to_arena, sphere_vs_obb, BOUNCE_IMPULSE, GRAVITY, JUMP_IMPULSE, MOB_BOX_HALF,
    MOB_BOX_OFFSET_Y, PLAYER_SPHERE_RADIUS, SQUASH_NORMAL_DOT_UP,
};
use crate::render3d::BobAnimation;
use glam::{Mat4, Vec3, Vec4Swizzles};

pub const PLAYER_SPEED: f32 = 14.0;
/// Spawn cadence: MobTimer wait_time 0.5 s at 120 Hz (Main.tscn lines 137–139).
pub const SPAWN_EVERY_TICKS: u64 = 60;
/// Frozen canonical seed (testdata/squash/golden_squash_trace.json).
pub const SQUASH_SEED: u64 = 0x1B;

/// Spawn rectangle (Curve3D points, Main.tscn lines 37–42), SpawnPath at
/// y=1 (line 128: transform y=1). SPAWN HEIGHT = 0, NOT 1 (review r6
/// F829-R6-01): `Main.gd:37` passes `mob_spawn_location.position` — the
/// PathFollow3D's LOCAL position (SpawnLocation local y=0, curve points
/// y=0) — and Mob.gd adds the mob directly under Main, so SpawnPath's y=1
/// never reaches the mob. Verified against the real Godot 4.4.1 fixture on
/// the unchanged frozen scripts (four seeds, all actual_spawned_mob_y=0;
/// review-r6 source-spawn-stdout receipt).
pub const SPAWN_PTS: [(f32, f32); 4] = [(14.0, -15.0), (-13.0, -15.0), (-13.0, 16.0), (14.0, 16.0)];
pub const SPAWN_Y: f32 = 0.0;
pub const MOB_MIN_SPEED: f32 = 10.0;
pub const MOB_MAX_SPEED: f32 = 18.0;

/// VisibleOnScreenNotifier3D frozen AABB (Mob.tscn lines 76–78).
pub const MOB_NOTIFIER_MIN: [f32; 3] = [-1.19986, 0.251327, -1.57098];
pub const MOB_NOTIFIER_SIZE: [f32; 3] = [2.41047, 1.09305, 3.17223];

/// MobDetector Area3D frozen cylinder shapes (Player.tscn lines 63–83),
/// player-node-local coordinates: (x, y, z, radius, height).
pub const DETECTOR_MAIN: (f32, f32, f32, f32, f32) = (0.0, 1.06491, 0.0, 0.907607, 0.1438);
pub const DETECTOR_SMALL_R: f32 = 0.2;
pub const DETECTOR_SMALL_H: f32 = 0.144;
pub const DETECTOR_SMALL: [(f32, f32, f32); 4] = [
    (0.455268, 0.95, 1.1423),
    (0.716365, 0.95, 1.71889),
    (-0.730546, 0.95, 1.69713),
    (-0.45857, 0.95, 1.15318),
];

/// Frozen squash-classification threshold (Player.gd line 57), re-exported
/// for the verifier gates.
pub const SQUASH_DOT_FOR_GATE: f32 = SQUASH_NORMAL_DOT_UP;

/// Digital input per tick (frozen stream encoding: [move_x, move_z, jump]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SquashAction {
    /// -1 left / +1 right (frozen: move_right adds +x, Player.gd lines 17–24).
    pub move_x: i8,
    /// -1 forward (-z) / +1 back (+z).
    pub move_z: i8,
    pub jump: bool,
    /// ui_accept while the Retry overlay is visible (Space/Enter, manifest §2.3).
    pub retry: bool,
    /// Host extension (never produced by the frozen input profile).
    pub reset: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SquashEvent {
    Spawned {
        index: u32,
        pos: Vec3,
        yaw: f32,
        speed: f32,
        vel: Vec3,
    },
    Squashed {
        index: u32,
        score: u32,
    },
    PlayerDied,
    Despawned {
        index: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Player,
    Mob,
}

#[derive(Debug, Clone)]
pub struct MobState {
    pub index: u32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub speed: f32,
    pub alive: bool,
    /// animation phase base: age in seconds since spawn (Mob.gd line 32
    /// scales the shared bob by speed/min_speed over the mob's own life).
    pub age: f32,
}

/// Uniform on the spawn-rectangle perimeter (arc-length walk; the frozen
/// curve is linear, cubic_interp=false).
pub fn spawn_position(r: f32) -> (f32, f32) {
    let mut total = 0.0f32;
    let mut lens = [0.0f32; 4];
    for i in 0..4 {
        let a = SPAWN_PTS[i];
        let b = SPAWN_PTS[(i + 1) % 4];
        let dx = b.0 - a.0;
        let dz = b.1 - a.1;
        let d = (dx * dx + dz * dz).sqrt();
        lens[i] = d;
        total += d;
    }
    let mut dist = r * total;
    for i in 0..4 {
        if dist <= lens[i] {
            let a = SPAWN_PTS[i];
            let b = SPAWN_PTS[(i + 1) % 4];
            let t = dist / lens[i];
            return (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        }
        dist -= lens[i];
    }
    SPAWN_PTS[0]
}

/// Frustum test of the frozen notifier AABB against the frozen sample camera
/// (orthographic, Main.tscn lines 98–106) — the VisibleOnScreenNotifier3D
/// screen-exit semantics.
pub fn mob_on_screen(pos: Vec3, camera_vp: &Mat4) -> bool {
    for i in 0..8 {
        let c = Vec3::new(
            pos.x
                + MOB_NOTIFIER_MIN[0]
                + if i & 1 == 1 {
                    MOB_NOTIFIER_SIZE[0]
                } else {
                    0.0
                },
            pos.y
                + MOB_NOTIFIER_MIN[1]
                + if i & 2 == 2 {
                    MOB_NOTIFIER_SIZE[1]
                } else {
                    0.0
                },
            pos.z
                + MOB_NOTIFIER_MIN[2]
                + if i & 4 == 4 {
                    MOB_NOTIFIER_SIZE[2]
                } else {
                    0.0
                },
        );
        let clip = *camera_vp * c.extend(1.0);
        if clip.w <= 0.0 {
            continue;
        }
        let ndc = clip.xyz() / clip.w;
        if ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0 && ndc.z >= 0.0 && ndc.z <= 1.0 {
            return true;
        }
    }
    false
}

/// Frozen MobDetector overlap test (Player.tscn lines 63–83): any of the
/// five player-local cylinders overlapping the mob box is fatal. Cylinders
/// rotate with the player's facing (Area3D is a child of the player node);
/// the mob box is yawed by its velocity. Vertical cylinder vs OBB is
/// tested as a y-interval overlap plus an exact 2D circle-vs-rotated-rect
/// in the horizontal plane (documented single-pass approximation of the
/// Jolt Area3D signal; the frozen geometry is exact).
pub fn detector_fatal(
    player_pos: Vec3,
    player_facing_yaw: f32,
    mob_pos: Vec3,
    mob_yaw: f32,
) -> bool {
    // Mob box center in the player-local (facing) frame.
    let to_player_local = Mat4::from_rotation_y(-player_facing_yaw);
    let box_center_world = mob_pos + Vec3::new(0.0, MOB_BOX_OFFSET_Y, 0.0);
    let box_center = (to_player_local * (box_center_world - player_pos).extend(1.0)).xyz();
    // The cylinders live in the PLAYER-LOCAL frame; the mob box is yawed in
    // the WORLD frame. Rotating a player-local offset into the mob frame
    // therefore takes the RELATIVE yaw (facing − mob_yaw) — review r4
    // F829-R4-03: the previous code applied −mob_yaw to a player-local
    // vector as if mob yaw were measured in that frame, breaking
    // rigid-rotation equivalence of the encounter.
    let to_mob_local = Mat4::from_rotation_y(player_facing_yaw - mob_yaw);
    let cylinders = std::iter::once(DETECTOR_MAIN).chain(
        DETECTOR_SMALL
            .iter()
            .map(|s| (s.0, s.1, s.2, DETECTOR_SMALL_R, DETECTOR_SMALL_H)),
    );
    for (cx, cy, cz, r, h) in cylinders {
        // y-interval overlap (cylinder axis is vertical; the mob has no pitch).
        let dy = box_center.y - cy;
        if dy.abs() > h / 2.0 + MOB_BOX_HALF[1] {
            continue;
        }
        // circle (player-local) vs yawed mob box: rotate the circle into the
        // mob's local frame, clamp to the box, compare against the radius.
        let circle_mob = (to_mob_local
            * (Vec3::new(cx, 0.0, cz) - Vec3::new(box_center.x, 0.0, box_center.z)).extend(1.0))
        .xyz();
        let closest_x = circle_mob.x.clamp(-MOB_BOX_HALF[0], MOB_BOX_HALF[0]);
        let closest_z = circle_mob.z.clamp(-MOB_BOX_HALF[2], MOB_BOX_HALF[2]);
        let dx = circle_mob.x - closest_x;
        let dz = circle_mob.z - closest_z;
        if dx * dx + dz * dz <= r * r {
            return true;
        }
    }
    false
}

#[derive(Debug)]
pub struct SquashDriver {
    /// Contact classifier — injectable so the negative-controls verifier can
    /// run a deliberately WRONG build through the same runner. Default is
    /// the frozen top-only semantics.
    pub squash_classifier: fn(Vec3) -> crate::render3d::collision::PlayerMobContact,
    pub player: Vec3,
    pub vy: f32,
    pub on_floor: bool,
    pub score: u32,
    pub dead: bool,
    pub retry_visible: bool,
    /// true while horizontal input is held this tick (Player.gd line 31:
    /// AnimationPlayer.speed_scale = 4 when moving, else 1).
    pub moving: bool,
    /// yaw the model faces while moving (Basis.looking_at(direction)).
    pub facing_yaw: f32,
    /// animation clock: advances dt*(4 when moving else 1) — speed scale is
    /// a clock warp, not a global time multiplier.
    pub anim_time: f32,
    pub mobs: Vec<MobState>,
    pub rng: DeterministicRng,
    next_spawn_step: u64,
    step: u64,
    spawn_count: u32,
    camera_vp: Mat4,
    // F11: shared-scene identity.
    player_id: Option<ObjectId>,
    mob_ids: std::collections::HashMap<u32, ObjectId>,
}

impl Default for SquashDriver {
    fn default() -> Self {
        SquashDriver::new(SQUASH_SEED)
    }
}

/// Player model matrix per the frozen source: node yaw faces movement, node
/// rotation.x = PI/6 * vy / jump_impulse (Player.gd line 66); the Character
/// child carries the bob (position y + rotation x, T-05 animation).
pub fn player_model_matrix(pos: Vec3, facing_yaw: f32, vy: f32, anim_time: f32) -> Mat4 {
    let (bob_y, bob_rot) = BobAnimation::default().eval(anim_time);
    let pitch = std::f32::consts::FRAC_PI_6 * vy / JUMP_IMPULSE;
    Mat4::from_translation(pos + Vec3::new(0.0, bob_y - PLAYER_SPHERE_RADIUS, 0.0))
        * Mat4::from_rotation_y(facing_yaw)
        * Mat4::from_rotation_x(pitch + bob_rot)
}

/// Mob model matrix: yaw from velocity, bob phase = age since spawn
/// (speed/10 scale already folded into `mob_anim_time`).
pub fn mob_model_matrix(pos: Vec3, vel: Vec3, mob_anim_time: f32) -> Mat4 {
    let (bob_y, bob_rot) = BobAnimation::default().eval(mob_anim_time);
    let yaw = (-vel.x).atan2(-vel.z);
    Mat4::from_translation(pos)
        * Mat4::from_rotation_y(yaw)
        * Mat4::from_translation(Vec3::new(0.0, bob_y, 0.0))
        * Mat4::from_rotation_x(bob_rot)
}

fn mat_to_transform(m: &Mat4) -> Transform {
    let (scale, rotation, translation) = m.to_scale_rotation_translation();
    Transform {
        translation,
        rotation,
        scale,
    }
}

/// Project the shared scene into render instances (F11): kind by the
/// session's FIRST-SPAWNED identity (review r4 F829-R4-01 — the previous
/// "first alive object" rule reclassified a surviving mob as Player after
/// the player's destruction, and again as mobs despawned). The player
/// identity is stable through deletion and index reuse; after death no
/// live id equals it, so everything remaining projects as Mob. An
/// independent consumer rebuilding from the ChangeLog applies the same
/// rule (first Spawn of the session = player identity).
pub fn project_instances(scene: &Scene) -> Vec<(ObjectId, EntityKind, Mat4)> {
    let player_id = scene.first_spawned();
    scene
        .objects()
        .iter()
        .map(|o| {
            let kind = if Some(o.id) == player_id {
                EntityKind::Player
            } else {
                EntityKind::Mob
            };
            (o.id, kind, o.transform.to_matrix())
        })
        .collect()
}

impl SquashDriver {
    pub fn new(seed: u64) -> Self {
        SquashDriver {
            squash_classifier: |n| {
                if Vec3::Y.dot(n) > SQUASH_NORMAL_DOT_UP {
                    crate::render3d::collision::PlayerMobContact::TopLandingSquash
                } else {
                    crate::render3d::collision::PlayerMobContact::Fatal
                }
            },
            player: Vec3::new(0.0, PLAYER_SPHERE_RADIUS, 0.0),
            vy: 0.0,
            on_floor: true,
            score: 0,
            dead: false,
            retry_visible: false,
            moving: false,
            facing_yaw: 0.0,
            anim_time: 0.0,
            mobs: Vec::new(),
            rng: DeterministicRng::new(seed),
            next_spawn_step: SPAWN_EVERY_TICKS,
            step: 0,
            spawn_count: 0,
            camera_vp: crate::render3d::Camera3d::sample_camera(1.5).view_proj(),
            player_id: None,
            mob_ids: std::collections::HashMap::new(),
        }
    }

    /// Host extension: full rebuild, fresh seeded RNG (pairs with
    /// `Runtime::reset` by the caller — the frozen retry is a scene reload).
    pub fn reset(&mut self) {
        *self = SquashDriver::new(SQUASH_SEED);
    }

    pub fn step_count(&self) -> u64 {
        self.step
    }

    pub fn player_object_id(&self) -> Option<ObjectId> {
        self.player_id
    }

    pub fn mob_object_id(&self, index: u32) -> Option<ObjectId> {
        self.mob_ids.get(&index).copied()
    }

    fn spawn_mob(&mut self, rt: &mut Runtime<SquashAction, SquashEvent>) {
        let r = self.rng.next_f32();
        let (sx, sz) = spawn_position(r);
        let jitter = self
            .rng
            .range(-std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4);
        let speed = self.rng.range(MOB_MIN_SPEED, MOB_MAX_SPEED);
        // Mob.initialize (Mob.gd lines 16-32): aim at the PLAYER'S CURRENT
        // POSITION (Main.gd line 36 passes it in), rotate_y(jitter),
        // velocity = FORWARD * speed.
        let (px, pz) = (self.player.x, self.player.z);
        let dx = px - sx;
        let dz = pz - sz;
        let yaw = (-dx).atan2(-dz) + jitter;
        let vx = -speed * yaw.sin();
        let vz = -speed * yaw.cos();
        let index = self.spawn_count;
        self.spawn_count += 1;
        self.mobs.push(MobState {
            index,
            pos: Vec3::new(sx, SPAWN_Y, sz),
            vel: Vec3::new(vx, 0.0, vz),
            speed,
            alive: true,
            age: 0.0,
        });
        rt.emit(SquashEvent::Spawned {
            index,
            pos: Vec3::new(sx, SPAWN_Y, sz),
            yaw,
            speed,
            vel: Vec3::new(vx, 0.0, vz),
        });
        // F11: shared-scene identity for the new mob.
        let errs = rt.submit(&[crate::core::scene::SceneChange::Spawn {
            transform: Transform::default(),
            color: crate::core::scene::Color::rgb(1.0, 1.0, 1.0),
        }]);
        debug_assert!(errs.is_empty(), "mob spawn delta: {errs:?}");
        let id = rt.scene().objects().last().unwrap().id;
        self.mob_ids.insert(index, id);
    }
}

impl Driver<SquashAction, SquashEvent> for SquashDriver {
    fn tick(&mut self, rt: &mut Runtime<SquashAction, SquashEvent>, _tick: u64) {
        let action = rt.actions().first().copied().unwrap_or_default();

        // F11: the player object exists from the first tick of a session.
        if self.player_id.is_none() {
            let errs = rt.submit(&[crate::core::scene::SceneChange::Spawn {
                transform: mat_to_transform(&player_model_matrix(
                    self.player,
                    self.facing_yaw,
                    self.vy,
                    self.anim_time,
                )),
                color: crate::core::scene::Color::rgb(1.0, 1.0, 1.0),
            }]);
            debug_assert!(errs.is_empty(), "player spawn delta: {errs:?}");
            self.player_id = Some(rt.scene().objects()[0].id);
        }

        // Spawn (Main._on_mob_timer_timeout; timer stops on death).
        if !self.dead && self.step == self.next_spawn_step {
            self.next_spawn_step += SPAWN_EVERY_TICKS;
            self.spawn_mob(rt);
        }

        // Mobs: straight flight, no gravity, screen-exit despawn — kept
        // running after player death (world keeps living; E3).
        let dt = crate::render3d::collision::PHYSICS_DT;
        let mut despawned = Vec::new();
        for m in &mut self.mobs {
            if !m.alive {
                continue;
            }
            m.pos += m.vel * dt;
            m.age += dt;
            if !mob_on_screen(m.pos, &self.camera_vp) {
                m.alive = false;
                despawned.push(m.index);
            }
        }
        for idx in despawned {
            rt.emit(SquashEvent::Despawned { index: idx });
            if let Some(id) = self.mob_ids.get(&idx).copied() {
                let errs = rt.submit(&[crate::core::scene::SceneChange::Destroy(id)]);
                debug_assert!(errs.is_empty(), "mob despawn delta: {errs:?}");
            }
        }

        // Player.
        let mut changes = Vec::new();
        if !self.dead {
            let grounded = self.player.y - PLAYER_SPHERE_RADIUS < 0.01;
            if grounded && action.jump {
                self.vy += JUMP_IMPULSE;
            }
            // Horizontal velocity set absolutely (Player.gd lines 35-36);
            // normalized diagonal (Vector3.normalized() == 1/sqrt(2)).
            let mut hx = PLAYER_SPEED * action.move_x as f32;
            let mut hz = PLAYER_SPEED * action.move_z as f32;
            if action.move_x != 0 && action.move_z != 0 {
                hx *= std::f32::consts::FRAC_1_SQRT_2;
                hz *= std::f32::consts::FRAC_1_SQRT_2;
            }
            self.moving = action.move_x != 0 || action.move_z != 0;
            if self.moving {
                // Basis.looking_at(direction): forward -Z maps to the dir.
                self.facing_yaw = (-hx).atan2(-hz);
            }
            self.anim_time += dt * if self.moving { 4.0 } else { 1.0 };
            self.vy -= GRAVITY * dt;
            self.player.x += hx * dt;
            self.player.y += self.vy * dt;
            self.player.z += hz * dt;
            if self.player.y - PLAYER_SPHERE_RADIUS < 0.0 {
                self.player.y = PLAYER_SPHERE_RADIUS;
                if self.vy < 0.0 {
                    self.vy = 0.0;
                }
                self.on_floor = true;
            } else {
                self.on_floor = self.player.y - PLAYER_SPHERE_RADIUS < 0.01;
            }
            self.player = clamp_to_arena(self.player, PLAYER_SPHERE_RADIUS);

            // Mob contacts — TWO frozen channels:
            // 1. body slide contact (Player.gd 53-62): first contact decides;
            //    top landing squashes (exactly one score), else fatal.
            // 2. MobDetector Area3D (Player.gd 73-74, Player.tscn 63-83):
            //    any detector-cylinder overlap is fatal even without body
            //    contact. Evaluated when the body channel found nothing.
            let player_pos = self.player;
            let mut squash: Option<u32> = None;
            let mut fatal = false;
            for m in &self.mobs {
                if !m.alive {
                    continue;
                }
                let yaw = (-m.vel.x).atan2(-m.vel.z);
                let box_center = m.pos + Vec3::new(0.0, MOB_BOX_OFFSET_Y, 0.0);
                if let Some((normal, _pen)) = sphere_vs_obb(
                    player_pos,
                    PLAYER_SPHERE_RADIUS,
                    box_center,
                    MOB_BOX_HALF,
                    yaw,
                ) {
                    match (self.squash_classifier)(normal) {
                        crate::render3d::collision::PlayerMobContact::TopLandingSquash => {
                            squash = Some(m.index);
                        }
                        crate::render3d::collision::PlayerMobContact::Fatal => {
                            fatal = true;
                        }
                    }
                    break; // one contact per tick (Player.gd break)
                }
            }
            if squash.is_none() && !fatal {
                for m in &self.mobs {
                    if !m.alive {
                        continue;
                    }
                    let mob_yaw = (-m.vel.x).atan2(-m.vel.z);
                    if detector_fatal(player_pos, self.facing_yaw, m.pos, mob_yaw) {
                        fatal = true;
                        break;
                    }
                }
            }
            if let Some(idx) = squash {
                if let Some(m) = self.mobs.iter_mut().find(|m| m.index == idx) {
                    m.alive = false;
                }
                self.score += 1;
                self.vy = BOUNCE_IMPULSE;
                rt.emit(SquashEvent::Squashed {
                    index: idx,
                    score: self.score,
                });
                if let Some(id) = self.mob_ids.get(&idx).copied() {
                    changes.push(crate::core::scene::SceneChange::Destroy(id));
                }
            } else if fatal {
                self.dead = true;
                self.retry_visible = true; // Main.gd: MobTimer.stop + Retry.show
                rt.emit(SquashEvent::PlayerDied);
                if let Some(id) = self.player_id {
                    changes.push(crate::core::scene::SceneChange::Destroy(id));
                }
            }
        }

        // F11: animation/position deltas for the shared scene.
        if let Some(id) = self.player_id {
            if !self.dead {
                changes.push(crate::core::scene::SceneChange::SetTransform(
                    id,
                    mat_to_transform(&player_model_matrix(
                        self.player,
                        self.facing_yaw,
                        self.vy,
                        self.anim_time,
                    )),
                ));
            }
        }
        for m in &self.mobs {
            if !m.alive {
                continue;
            }
            if let Some(id) = self.mob_ids.get(&m.index).copied() {
                changes.push(crate::core::scene::SceneChange::SetTransform(
                    id,
                    mat_to_transform(&mob_model_matrix(m.pos, m.vel, m.age * (m.speed / 10.0))),
                ));
            }
        }
        if !changes.is_empty() {
            let errs = rt.submit(&changes);
            debug_assert!(errs.is_empty(), "scene deltas: {errs:?}");
        }

        // Dead mobs are removed from the world (queue_free semantics); their
        // driver rows and id bindings are pruned so long sessions stay
        // bounded (review r1 F06). The golden trace tracks only live counts,
        // which pruning does not change.
        if self.mobs.iter().any(|m| !m.alive) {
            self.mobs.retain(|m| m.alive);
            let live: std::collections::HashSet<u32> = self.mobs.iter().map(|m| m.index).collect();
            self.mob_ids.retain(|idx, _| live.contains(idx));
        }

        self.step += 1;
    }
}
