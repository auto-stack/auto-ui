//! Character/mob collision feasibility — the frozen sample's ACTUAL scheme
//! (evidence notes E3/E4), pure-state:
//!
//! - Player body: sphere r=0.792278 at local (0, 0.771765, 0)
//!   (`Player.tscn` lines 59–62; sub-resource lines 6–8). The player collides
//!   physically with the mob layer (`collision_mask = 2147483654` includes
//!   layer 2) — slide collisions feed the squash check
//!   (`Player.gd` lines 53–62: normal·UP > 0.1 ⇒ squash + bounce 16).
//! - Mob body: box 1.35822 × 1.08835 × 2.20058 at (0, 0.52793, 0)
//!   (`Mob.tscn` lines 6–8, 56–58), yaw-only rotation, collision mask on an
//!   UNDEFINED layer ⇒ mobs never physically collide with anything.
//! - Death: `MobDetector` Area3D (mask = layer 2) — any mob contact that is
//!   not a top landing kills the player (`Player.gd` line 73–74).
//! - Ground: box 60×2×60 top at y=0 (`Main.tscn` lines 48–57); gravity 75,
//!   jump 20 (`Player.gd` lines 6–12), fixed 120 Hz (`project.godot`).
//! - Arena bounds: the four WorldBoundaryShape3D world positions from
//!   `Main.tscn` lines 59–82 (parent `Walls` at (0,0,5); shape origins world:
//!   (0,·,16), (−13,·,5), (14,·,5), (0,·,−15)) — the spawn rectangle edges.
//!   Godot's one-sided plane semantics are approximated by inward-facing
//!   planes at those frozen positions (recorded in 3d-feasibility.md).

use glam::Vec3;

pub const PLAYER_SPHERE_RADIUS: f32 = 0.792278;
pub const PLAYER_SPHERE_OFFSET_Y: f32 = 0.771765;
pub const GRAVITY: f32 = 75.0;
pub const JUMP_IMPULSE: f32 = 20.0;
pub const BOUNCE_IMPULSE: f32 = 16.0;
pub const SQUASH_NORMAL_DOT_UP: f32 = 0.1;

/// Mob box (Mob.tscn lines 56–58), half extents.
pub const MOB_BOX_HALF: [f32; 3] = [1.35822 / 2.0, 1.08835 / 2.0, 2.20058 / 2.0];
pub const MOB_BOX_OFFSET_Y: f32 = 0.52793;

/// Fixed physics step from the frozen project.godot
/// (`physics_ticks_per_second = 120`).
pub const PHYSICS_DT: f32 = 1.0 / 120.0;

/// The frozen arena boundary planes: (point on plane, inward normal).
/// Positions from Main.tscn Walls shapes (see module doc); normals point INTO
/// the playable rectangle x∈[-13,14], z∈[-15,16].
pub fn boundary_planes() -> [(Vec3, Vec3); 4] {
    [
        (Vec3::new(0.0, 0.0, 16.0), Vec3::new(0.0, 0.0, -1.0)),
        (Vec3::new(-13.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        (Vec3::new(14.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        (Vec3::new(0.0, 0.0, -15.0), Vec3::new(0.0, 0.0, 1.0)),
    ]
}

/// Sphere-vs-horizontal-plane (ground top at `ground_y`): penetration depth.
pub fn sphere_vs_ground(center: Vec3, radius: f32, ground_y: f32) -> Option<f32> {
    let pen = ground_y + radius - center.y;
    if pen > 0.0 {
        Some(pen)
    } else {
        None
    }
}

/// Sphere-vs-OBB (yaw-only rotation). Returns (world contact normal, depth).
/// Transforms the sphere into the box's local (yaw) frame, clamps to the box,
/// then transforms the offset normal back.
pub fn sphere_vs_obb(
    sphere: Vec3,
    radius: f32,
    box_center: Vec3,
    box_half: [f32; 3],
    box_yaw: f32,
) -> Option<(Vec3, f32)> {
    let inv_yaw = Mat4ish::yaw(-box_yaw);
    let local = inv_yaw * (sphere - box_center);
    let closest = Vec3::new(
        local.x.clamp(-box_half[0], box_half[0]),
        local.y.clamp(-box_half[1], box_half[1]),
        local.z.clamp(-box_half[2], box_half[2]),
    );
    let offset = local - closest;
    let dist2 = offset.length_squared();
    if dist2 > radius * radius {
        return None;
    }
    let normal_local = if dist2 > 1e-12 {
        offset / dist2.sqrt()
    } else {
        // center inside the box: push along the least-penetration axis
        let px = box_half[0] - local.x.abs();
        let py = box_half[1] - local.y.abs();
        let pz = box_half[2] - local.z.abs();
        if px <= py && px <= pz {
            Vec3::X * if local.x >= 0.0 { 1.0 } else { -1.0 }
        } else if py <= pz {
            Vec3::Y * if local.y >= 0.0 { 1.0 } else { -1.0 }
        } else {
            Vec3::Z * if local.z >= 0.0 { 1.0 } else { -1.0 }
        }
    };
    let normal = Mat4ish::yaw(box_yaw) * normal_local;
    let depth = radius - dist2.sqrt().min(radius);
    Some((normal.normalize(), depth))
}

/// Minimal yaw rotation helper (avoids pulling Mat4 conventions into tests).
struct Mat4ish;
impl Mat4ish {
    fn yaw(angle: f32) -> glam::Mat3 {
        glam::Mat3::from_rotation_y(angle)
    }
}

/// Classify a player-mob contact per the frozen squash semantics
/// (`Player.gd` lines 53–62, 73–74).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMobContact {
    /// normal·UP > 0.1: player landed on top ⇒ squash the mob, bounce.
    TopLandingSquash,
    /// Any other contact reaching the player body: fatal (MobDetector).
    Fatal,
}

pub fn classify_player_mob_contact(normal: Vec3) -> PlayerMobContact {
    if Vec3::Y.dot(normal) > SQUASH_NORMAL_DOT_UP {
        PlayerMobContact::TopLandingSquash
    } else {
        PlayerMobContact::Fatal
    }
}

/// Clamp a position inside the boundary planes (walls).
pub fn clamp_to_arena(pos: Vec3, radius: f32) -> Vec3 {
    let mut p = pos;
    for (point, inward) in boundary_planes() {
        // distance moved past the plane along -inward
        let violation = (p - point).dot(-inward) + radius;
        if violation > 0.0 {
            p += inward * violation;
        }
    }
    p
}

/// Floor snap slop: penetration down to this depth still counts as "on the
/// floor" (exact rest gives pen == 0; without a slop is_on_floor would flap).
pub const FLOOR_SLOP: f32 = 0.01;

fn on_floor(center: Vec3) -> bool {
    sphere_vs_ground(center, PLAYER_SPHERE_RADIUS + FLOOR_SLOP, 0.0).is_some()
}

/// One fixed physics step of a player body: gravity, integrate, ground
/// resolve, arena clamp. Returns (new_center, new_velocity, on_ground).
pub fn step_player(mut center: Vec3, mut velocity: Vec3, jump_pressed: bool) -> (Vec3, Vec3, bool) {
    // Player.gd: velocity.x/z are set absolutely by input each tick; this
    // feasibility body handles vertical physics only (horizontal input is the
    // caller's). is_on_floor ⇔ resting contact (with slop) at step start.
    let grounded = on_floor(center);
    if grounded && jump_pressed {
        velocity.y += JUMP_IMPULSE;
    }
    velocity.y -= GRAVITY * PHYSICS_DT;
    center += velocity * PHYSICS_DT;
    let mut on_ground_after = false;
    if let Some(pen) = sphere_vs_ground(center, PLAYER_SPHERE_RADIUS, 0.0) {
        center.y += pen;
        if velocity.y < 0.0 {
            velocity.y = 0.0;
        }
        on_ground_after = true;
    }
    center = clamp_to_arena(center, PLAYER_SPHERE_RADIUS);
    (center, velocity, on_ground_after)
}
