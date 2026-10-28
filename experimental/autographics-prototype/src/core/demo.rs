//! Deterministic demo driver: a minimal scripted "game" used by the G-base
//! contract tests and the shared-runtime probe. It exercises every runtime
//! mechanism: spawn/move/destroy, RNG draws, resource bindings, bounce events.
//!
//! It is NOT one of the frozen samples — it is the shared-runtime exercise
//! vehicle. Sample oracles remain `research/sample-manifest.md`.

use crate::core::runtime::{Driver, Runtime};
use crate::core::scene::{Color, SceneChange, Transform};
use glam::Vec3;
use std::time::Duration;

/// Canonical fixed tick for the demo runs (Breakout's oracle rate).
pub const DEMO_DT: Duration = Duration::from_nanos(16_666_667); // 1/60 s
/// Canonical seed (frozen; arbitrary but fixed).
pub const DEMO_SEED: u64 = 0x8293_D01;
/// Arena bounds for the mover.
pub const DEMO_ARENA_MIN: f32 = -1.0;
pub const DEMO_ARENA_MAX: f32 = 1.0;
/// Mover speed in units/second.
pub const DEMO_SPEED: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoAction {
    None,
    MoveLeft,
    MoveRight,
}

/// Session-independent object reference (index + generation only) so event
/// equality is defined across separately-built runtimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjRef {
    pub index: u32,
    pub generation: u32,
}

impl ObjRef {
    fn of(id: crate::core::id::ObjectId) -> Self {
        ObjRef {
            index: id.index(),
            generation: id.generation(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DemoEvent {
    Moved { x: f32 },
    Bounced { at_min: bool },
    Spawned { who: ObjRef, jitter: f32 },
    Destroyed { who: ObjRef },
    ResourceBound { who: ObjRef, revision: u64 },
}

/// The canonical action stream: 4-phase cycle (right / none / left / none),
/// 30 ticks per phase. Pure function of tick — replayable by construction.
pub fn canonical_action(tick: u64) -> DemoAction {
    match (tick / 30) % 4 {
        0 => DemoAction::MoveRight,
        1 => DemoAction::None,
        2 => DemoAction::MoveLeft,
        _ => DemoAction::None,
    }
}

pub fn canonical_stream(n: u64) -> Vec<DemoAction> {
    (0..n).map(canonical_action).collect()
}

/// Demo driver state. `mover`/`decoy` hold ids from the runtime session.
#[derive(Debug, Default)]
pub struct DemoDriver {
    pub mover: Option<crate::core::id::ObjectId>,
    /// The decoy spawned at tick 5 and destroyed at tick 8 — tests hold this
    /// after the run to prove late-id rejection.
    pub decoy: Option<crate::core::id::ObjectId>,
    pub mover_x: f32,
}

impl Driver<DemoAction, DemoEvent> for DemoDriver {
    fn tick(&mut self, rt: &mut Runtime<DemoAction, DemoEvent>, tick: u64) {
        // Spawn phase.
        if tick == 1 {
            let errs = rt.submit(&[SceneChange::Spawn {
                transform: Transform {
                    translation: Vec3::new(self.mover_x, 0.0, 0.0),
                    scale: Vec3::new(0.5, 0.5, 1.0),
                    ..Default::default()
                },
                color: Color::rgb(0.9, 0.4, 0.3),
            }]);
            assert!(errs.is_empty(), "spawn mover: {errs:?}");
            self.mover = Some(rt.scene().objects()[0].id);
        }
        if tick == 5 {
            let jitter = rt.rng().range(0.0, 1.0);
            let errs = rt.submit(&[SceneChange::Spawn {
                transform: Transform {
                    translation: Vec3::new(2.0, 1.0, 0.0),
                    scale: Vec3::new(0.25, 0.25, 1.0),
                    ..Default::default()
                },
                color: Color::rgb(jitter, 0.5, 0.8),
            }]);
            assert!(errs.is_empty(), "spawn decoy: {errs:?}");
            let id = rt.scene().objects().last().unwrap().id;
            self.decoy = Some(id);
            rt.emit(DemoEvent::Spawned {
                who: ObjRef::of(id),
                jitter,
            });
        }
        if tick == 8 {
            if let Some(decoy) = self.decoy.take() {
                let errs = rt.submit(&[SceneChange::Destroy(decoy)]);
                assert!(errs.is_empty(), "destroy decoy: {errs:?}");
                rt.emit(DemoEvent::Destroyed {
                    who: ObjRef::of(decoy),
                });
            }
        }

        // Move phase (from the action stream).
        if let Some(mover) = self.mover {
            let dir = match rt.actions().first().copied().unwrap_or(DemoAction::None) {
                DemoAction::MoveLeft => -1.0,
                DemoAction::MoveRight => 1.0,
                DemoAction::None => 0.0,
            };
            let dt = rt.dt().as_secs_f32();
            self.mover_x += dir * DEMO_SPEED * dt;
            let mut bounced = None;
            if self.mover_x < DEMO_ARENA_MIN {
                self.mover_x = DEMO_ARENA_MIN;
                bounced = Some(true);
            } else if self.mover_x > DEMO_ARENA_MAX {
                self.mover_x = DEMO_ARENA_MAX;
                bounced = Some(false);
            }
            let errs = rt.submit(&[SceneChange::SetTransform(
                mover,
                Transform {
                    translation: Vec3::new(self.mover_x, 0.0, 0.0),
                    scale: Vec3::new(0.5, 0.5, 1.0),
                    ..Default::default()
                },
            )]);
            assert!(errs.is_empty(), "move: {errs:?}");
            rt.emit(DemoEvent::Moved { x: self.mover_x });
            if let Some(at_min) = bounced {
                rt.emit(DemoEvent::Bounced { at_min });
            }
        }

        // Resource binding at tick 10: mover gets a binding; other objects
        // must stay untouched (revision + touch-set assertions).
        if tick == 10 {
            if let Some(mover) = self.mover {
                let errs = rt.submit(&[SceneChange::SetResource(mover, None)]);
                assert!(errs.is_empty(), "bind: {errs:?}");
                let rev = rt
                    .scene()
                    .objects()
                    .iter()
                    .find(|o| o.id == mover)
                    .unwrap()
                    .resource_revision;
                rt.emit(DemoEvent::ResourceBound {
                    who: ObjRef::of(mover),
                    revision: rev,
                });
            }
        }
    }
}

/// Run the canonical demo for `n` ticks on a fresh runtime; returns the
/// runtime (final state + log) and the concatenated event payload sequence.
pub fn run_canonical(n: u64) -> (Runtime<DemoAction, DemoEvent>, Vec<DemoEvent>) {
    let mut rt = Runtime::new(DEMO_DT, DEMO_SEED);
    let mut driver = DemoDriver::default();
    let mut events = Vec::new();
    for tick in 0..n {
        let step_events = rt.step(&mut driver, vec![canonical_action(tick)]);
        events.extend(step_events.into_iter().map(|e| e.payload));
    }
    (rt, events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs_clean() {
        let (rt, events) = run_canonical(60);
        assert_eq!(rt.tick(), 60);
        assert!(events
            .iter()
            .any(|e| matches!(e, DemoEvent::Bounced { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, DemoEvent::Destroyed { .. })));
    }
}
