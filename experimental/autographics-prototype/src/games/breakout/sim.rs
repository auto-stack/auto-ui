//! AG-2D-01 Breakout — pure game logic, faithful to the frozen sample
//! (`bevyengine/bevy@efe61cc5b9872f3f9e1ee7553325612c094ccc97`,
//! `examples/showcase/breakout.rs`, blob b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954)
//! as parsed in `research/sample-manifest.md` §1. Do NOT add rules the frozen
//! sample does not have: bottom wall BOUNCES, there is no ball loss, no
//! lives, no game over.
//!
//! Implements `Driver` over the G-base runtime: game logic submits scene
//! deltas; the runtime owns identity/time/events. `Paddle` reset is a HOST
//! extension (probe-level), not sample behavior.

use crate::core::id::ObjectId;
use crate::core::runtime::{Driver, Runtime};
use crate::core::scene::{Color, SceneChange, Transform};
use glam::Vec3;

// --- frozen constants (manifest §1.2, quoted from breakout.rs) --------------

pub const PADDLE_SIZE: [f32; 2] = [120.0, 20.0];
pub const PADDLE_SPEED: f32 = 500.0;
pub const PADDLE_PADDING: f32 = 10.0;
pub const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;

pub const BALL_START: [f32; 2] = [0.0, -50.0];
pub const BALL_DIAMETER: f32 = 30.0;
pub const BALL_SPEED: f32 = 400.0;
pub const INITIAL_BALL_DIRECTION: [f32; 2] = [0.5, -0.5];

pub const WALL_THICKNESS: f32 = 10.0;
pub const LEFT_WALL: f32 = -450.0;
pub const RIGHT_WALL: f32 = 450.0;
pub const BOTTOM_WALL: f32 = -300.0;
pub const TOP_WALL: f32 = 300.0;

pub const BRICK_SIZE: [f32; 2] = [100.0, 30.0];
pub const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
pub const GAP_BETWEEN_BRICKS: f32 = 5.0;
pub const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
pub const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

pub const SCOREBOARD_FONT_SIZE: f32 = 33.0;
pub const SCOREBOARD_TEXT_PADDING: f32 = 5.0;

pub const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
pub const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
pub const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
pub const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
pub const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
pub const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
pub const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

/// Frozen oracle dt: f32(1/60). The sim uses this f32 constant directly
/// (never a Duration round-trip) to stay bit-compatible with the golden
/// trace, which was generated with numpy float32 arithmetic.
pub const DT: f32 = 1.0 / 60.0;

pub const BRICK_COUNT: usize = 56;

// --- derived frozen layout --------------------------------------------------

pub fn paddle_y() -> f32 {
    BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR
}

/// Paddle x clamp bounds: [-390, 390] (manifest §1.2).
pub fn paddle_clamp() -> (f32, f32) {
    let left = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE[0] / 2.0 + PADDLE_PADDING;
    let right = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE[0] / 2.0 - PADDLE_PADDING;
    (left, right)
}

/// Initial ball velocity: INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED.
pub fn initial_ball_velocity() -> [f32; 2] {
    let x = INITIAL_BALL_DIRECTION[0];
    let y = INITIAL_BALL_DIRECTION[1];
    let len = (x * x + y * y).sqrt();
    [(x / len) * BALL_SPEED, (y / len) * BALL_SPEED]
}

/// Wall centers + half-sizes, frozen iteration order: left, right, bottom,
/// top (matches the oracle collider order in the golden trace).
pub fn walls() -> [([f32; 2], [f32; 2]); 4] {
    let arena_h = TOP_WALL - BOTTOM_WALL;
    let arena_w = RIGHT_WALL - LEFT_WALL;
    [
        (
            [LEFT_WALL, 0.0],
            [WALL_THICKNESS / 2.0, (arena_h + WALL_THICKNESS) / 2.0],
        ),
        (
            [RIGHT_WALL, 0.0],
            [WALL_THICKNESS / 2.0, (arena_h + WALL_THICKNESS) / 2.0],
        ),
        (
            [0.0, BOTTOM_WALL],
            [(arena_w + WALL_THICKNESS) / 2.0, WALL_THICKNESS / 2.0],
        ),
        (
            [0.0, TOP_WALL],
            [(arena_w + WALL_THICKNESS) / 2.0, WALL_THICKNESS / 2.0],
        ),
    ]
}

/// Brick centers, row-major (7 rows × 8 columns = 56), computed exactly as
/// breakout.rs does (manifest §1.2: first center (-367.5, 45), step (105,35)).
pub fn brick_grid() -> Vec<[f32; 2]> {
    let total_width = (RIGHT_WALL - LEFT_WALL) - 2.0 * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge = paddle_y() + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height = TOP_WALL - bottom_edge - GAP_BETWEEN_BRICKS_AND_CEILING;
    let n_columns = (total_width / (BRICK_SIZE[0] + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height / (BRICK_SIZE[1] + GAP_BETWEEN_BRICKS)).floor() as usize;
    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge = center_of_bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE[0])
        - (n_columns - 1) as f32 / 2.0 * GAP_BETWEEN_BRICKS;
    let offset_x = left_edge + BRICK_SIZE[0] / 2.0;
    let offset_y = bottom_edge + BRICK_SIZE[1] / 2.0;
    let mut out = Vec::with_capacity(n_rows * n_columns);
    for row in 0..n_rows {
        for column in 0..n_columns {
            out.push([
                offset_x + column as f32 * (BRICK_SIZE[0] + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE[1] + GAP_BETWEEN_BRICKS),
            ]);
        }
    }
    out
}

// --- actions / events --------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddleDir {
    None,
    Left,
    Right,
}

/// Input actions. The frozen sample binds ONLY ArrowLeft/ArrowRight
/// (manifest §1.3); `Reset` is a HOST extension and is never produced by the
/// frozen input profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakoutAction {
    Paddle(PaddleDir),
    /// Host extension (Plan §5.2): rebuild the scene. Not sample behavior.
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColliderKind {
    WallLeft,
    WallRight,
    WallBottom,
    WallTop,
    Paddle,
    Brick(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BreakoutEvent {
    /// Ball overlapped a collider (manifest: BallCollided ⇒ sound trigger).
    Contact { kind: ColliderKind, side: Side },
    /// One-shot collision sound triggered (1:1 with contacts).
    Sound { total: u64 },
    /// Brick despawned; carries the new score (manifest: +1 per brick).
    BrickGone { index: u8, score: u32 },
}

// --- collision geometry (exact translation of ball_collision + reflect) -----

/// Circle vs AABB: returns the side of the box the ball hit
/// (breakout.rs `ball_collision`; center-inside ⇒ Bottom via offset == 0).
pub fn circle_aabb_side(
    ball: [f32; 2],
    radius: f32,
    center: [f32; 2],
    half: [f32; 2],
) -> Option<Side> {
    let qx = ball[0].clamp(center[0] - half[0], center[0] + half[0]);
    let qy = ball[1].clamp(center[1] - half[1], center[1] + half[1]);
    let ox = ball[0] - qx;
    let oy = ball[1] - qy;
    if ox * ox + oy * oy > radius * radius {
        return None;
    }
    Some(if ox.abs() > oy.abs() {
        if ox < 0.0 {
            Side::Left
        } else {
            Side::Right
        }
    } else if oy > 0.0 {
        Side::Top
    } else {
        Side::Bottom
    })
}

/// Conditional reflection (breakout.rs lines 364–379): reflect only when the
/// velocity points INTO the contacted side — prevents sticking.
pub fn reflect(vel: &mut [f32; 2], side: Side) {
    let (mut rx, mut ry) = (false, false);
    match side {
        Side::Left => rx = vel[0] > 0.0,
        Side::Right => rx = vel[0] < 0.0,
        Side::Top => ry = vel[1] < 0.0,
        Side::Bottom => ry = vel[1] > 0.0,
    }
    if rx {
        vel[0] = -vel[0];
    }
    if ry {
        vel[1] = -vel[1];
    }
}

// --- driver ------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct BreakoutIds {
    pub paddle: Option<ObjectId>,
    pub ball: Option<ObjectId>,
    pub brick_ids: Vec<ObjectId>,
    pub wall_ids: Vec<ObjectId>,
}

#[derive(Debug)]
pub struct BreakoutDriver {
    pub ball_pos: [f32; 2],
    pub ball_vel: [f32; 2],
    pub paddle_x: f32,
    pub score: u32,
    pub bricks_alive: [bool; BRICK_COUNT],
    pub ids: BreakoutIds,
    pub contacts_total: u64,
    pub sounds_total: u64,
    spawned: bool,
}

impl Default for BreakoutDriver {
    fn default() -> Self {
        BreakoutDriver {
            ball_pos: BALL_START,
            ball_vel: initial_ball_velocity(),
            paddle_x: 0.0,
            score: 0,
            bricks_alive: [true; BRICK_COUNT],
            ids: BreakoutIds::default(),
            contacts_total: 0,
            sounds_total: 0,
            spawned: false,
        }
    }
}

impl BreakoutDriver {
    /// Host extension (not sample behavior): full rebuild. Pairs with
    /// `Runtime::reset` by the caller.
    pub fn reset(&mut self) {
        *self = BreakoutDriver::default();
    }

    fn spawn_scene(&mut self, rt: &mut Runtime<BreakoutAction, BreakoutEvent>) {
        let mut changes = Vec::with_capacity(4 + 2 + BRICK_COUNT);
        for (center, half) in walls() {
            changes.push(SceneChange::Spawn {
                transform: Transform {
                    translation: Vec3::new(center[0], center[1], 0.0),
                    scale: Vec3::new(half[0] * 2.0, half[1] * 2.0, 1.0),
                    ..Default::default()
                },
                color: WALL_COLOR,
            });
        }
        changes.push(SceneChange::Spawn {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y(), 0.0),
                scale: Vec3::new(PADDLE_SIZE[0], PADDLE_SIZE[1], 1.0),
                ..Default::default()
            },
            color: PADDLE_COLOR,
        });
        changes.push(SceneChange::Spawn {
            transform: Transform {
                translation: Vec3::new(BALL_START[0], BALL_START[1], 1.0),
                scale: Vec3::new(BALL_DIAMETER, BALL_DIAMETER, 1.0),
                ..Default::default()
            },
            color: BALL_COLOR,
        });
        for center in brick_grid() {
            changes.push(SceneChange::Spawn {
                transform: Transform {
                    translation: Vec3::new(center[0], center[1], 0.0),
                    scale: Vec3::new(BRICK_SIZE[0], BRICK_SIZE[1], 1.0),
                    ..Default::default()
                },
                color: BRICK_COLOR,
            });
        }
        let errs = rt.submit(&changes);
        assert!(errs.is_empty(), "breakout spawn: {errs:?}");
        let objs = rt.scene().objects();
        self.ids.wall_ids = objs[0..4].iter().map(|o| o.id).collect();
        self.ids.paddle = Some(objs[4].id);
        self.ids.ball = Some(objs[5].id);
        self.ids.brick_ids = objs[6..6 + BRICK_COUNT].iter().map(|o| o.id).collect();
        self.spawned = true;
    }
}

/// Collider iteration order is part of the frozen oracle: walls (left,
/// right, bottom, top), paddle, bricks row-major. See golden trace meta.
fn collider_order(
    paddle_x: f32,
    bricks_alive: &[bool; BRICK_COUNT],
) -> Vec<(ColliderKind, [f32; 2], [f32; 2])> {
    let mut out = Vec::with_capacity(5 + BRICK_COUNT);
    for (i, (center, half)) in walls().iter().enumerate() {
        let kind = match i {
            0 => ColliderKind::WallLeft,
            1 => ColliderKind::WallRight,
            2 => ColliderKind::WallBottom,
            _ => ColliderKind::WallTop,
        };
        out.push((kind, *center, *half));
    }
    out.push((
        ColliderKind::Paddle,
        [paddle_x, paddle_y()],
        [PADDLE_SIZE[0] / 2.0, PADDLE_SIZE[1] / 2.0],
    ));
    for (i, center) in brick_grid().iter().enumerate() {
        if bricks_alive[i] {
            out.push((
                ColliderKind::Brick(i as u8),
                *center,
                [BRICK_SIZE[0] / 2.0, BRICK_SIZE[1] / 2.0],
            ));
        }
    }
    out
}

impl Driver<BreakoutAction, BreakoutEvent> for BreakoutDriver {
    fn tick(&mut self, rt: &mut Runtime<BreakoutAction, BreakoutEvent>, _tick: u64) {
        if !self.spawned {
            self.spawn_scene(rt);
        }

        // --- apply_velocity (ball moves first, per the frozen chain) ---
        self.ball_pos[0] += self.ball_vel[0] * DT;
        self.ball_pos[1] += self.ball_vel[1] * DT;

        // --- move_paddle ---
        let dir = rt
            .actions()
            .iter()
            .find_map(|a| match a {
                BreakoutAction::Paddle(d) => Some(*d),
                BreakoutAction::Reset => None,
            })
            .unwrap_or(PaddleDir::None);
        let direction = match dir {
            PaddleDir::Left => -1.0,
            PaddleDir::Right => 1.0,
            PaddleDir::None => 0.0,
        };
        let (lo, hi) = paddle_clamp();
        self.paddle_x = (self.paddle_x + direction * PADDLE_SPEED * DT).clamp(lo, hi);

        // Scene deltas for movement (local edits: ball + paddle only).
        let mut changes = Vec::with_capacity(2 + BRICK_COUNT);
        changes.push(SceneChange::SetTransform(
            self.ids.ball.unwrap(),
            Transform {
                translation: Vec3::new(self.ball_pos[0], self.ball_pos[1], 1.0),
                scale: Vec3::new(BALL_DIAMETER, BALL_DIAMETER, 1.0),
                ..Default::default()
            },
        ));
        changes.push(SceneChange::SetTransform(
            self.ids.paddle.unwrap(),
            Transform {
                translation: Vec3::new(self.paddle_x, paddle_y(), 0.0),
                scale: Vec3::new(PADDLE_SIZE[0], PADDLE_SIZE[1], 1.0),
                ..Default::default()
            },
        ));

        // --- check_for_collisions (frozen collider order) ---
        let mut brick_destroys = Vec::new();
        for (kind, center, half) in collider_order(self.paddle_x, &self.bricks_alive) {
            if let Some(side) = circle_aabb_side(self.ball_pos, BALL_DIAMETER / 2.0, center, half) {
                self.contacts_total += 1;
                self.sounds_total += 1;
                rt.emit(BreakoutEvent::Contact { kind, side });
                rt.emit(BreakoutEvent::Sound {
                    total: self.sounds_total,
                });
                if let ColliderKind::Brick(index) = kind {
                    self.bricks_alive[index as usize] = false;
                    self.score += 1;
                    brick_destroys.push((index, self.score));
                }
                reflect(&mut self.ball_vel, side);
            }
        }
        for (index, score) in brick_destroys {
            rt.emit(BreakoutEvent::BrickGone { index, score });
            changes.push(SceneChange::Destroy(self.ids.brick_ids[index as usize]));
        }
        let errs = rt.submit(&changes);
        assert!(errs.is_empty(), "breakout deltas: {errs:?}");
    }
}
