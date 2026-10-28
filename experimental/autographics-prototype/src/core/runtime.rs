//! The shared runtime (G-base): identity + fixed time + actions/events +
//! replayable scene deltas + reset semantics. GPU-agnostic.
//!
//! Contract (PLAN-829 T-03 / V-02):
//! - Same seed + same action stream + same step count ⇒ same scene snapshot
//!   and exactly-equal event sequence (positions at f32 precision within the
//!   manifest tolerance 1e-3; events exact).
//! - The change log is replayable: applying a recorded log to a fresh scene
//!   reproduces the live run's final state.
//! - Reset starts a new session; prior-session ids and events are rejected.

use crate::core::id::ObjectId;
use crate::core::id::SessionId;
use crate::core::rng::DeterministicRng;
use crate::core::scene::{Scene, SceneChange, SceneError};
use crate::core::time::TickClock;
use std::time::Duration;

/// One emitted event, stamped with its tick and session.
#[derive(Debug, Clone, PartialEq)]
pub struct TickEvent<E> {
    pub tick: u64,
    pub seq: u64,
    pub session: SessionId,
    pub payload: E,
}

/// Accepted scene changes for one tick (single-change granularity, so the
/// log replays exactly what was applied — rejected changes are not logged).
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub tick: u64,
    pub change: SceneChange,
}

/// Replayable scene change log. Bound to the session that produced it —
/// replaying rebuilds that session's id space exactly (ids are session-scoped
/// by design).
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeLog {
    pub session: crate::core::id::SessionId,
    pub entries: Vec<LogEntry>,
}

impl ChangeLog {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Rebuild the final scene by replaying the log into a scene rebound to
    /// the log's session. Tick order is preserved; tick values are
    /// informational (the scene is a fold over the ordered accepted changes).
    pub fn replay(&self) -> Scene {
        let mut scene = Scene::with_session(self.session);
        for entry in &self.entries {
            let errs = scene.submit(&[entry.change.clone()]);
            debug_assert!(
                errs.is_empty(),
                "logged change must replay cleanly: {errs:?}"
            );
        }
        scene
    }
}

/// Game logic hook. `tick` is the current tick id; actions for this tick are
/// visible via `rt.actions()`. The driver mutates the runtime's scene via
/// `rt.submit`, reads `rt.rng()`, and emits via `rt.emit`.
pub trait Driver<A, E> {
    fn tick(&mut self, rt: &mut Runtime<A, E>, tick: u64);
}

/// The shared runtime. Generic over the game's action (`A`) and event (`E`)
/// payload types.
pub struct Runtime<A, E> {
    session: SessionId,
    clock: TickClock,
    scene: Scene,
    rng: DeterministicRng,
    log: ChangeLog,
    /// Objects named by changes accepted in the CURRENT step.
    touched: Vec<ObjectId>,
    inbox: Vec<A>,
    outbox: Vec<TickEvent<E>>,
    next_event_seq: u64,
}

impl<A, E> Runtime<A, E> {
    pub fn new(dt: Duration, seed: u64) -> Self {
        let scene = Scene::new();
        let session = scene.session();
        Runtime {
            session,
            clock: TickClock::new(dt),
            scene,
            rng: DeterministicRng::new(seed),
            log: ChangeLog {
                session,
                entries: Vec::new(),
            },
            touched: Vec::new(),
            inbox: Vec::new(),
            outbox: Vec::new(),
            next_event_seq: 0,
        }
    }

    pub fn session(&self) -> SessionId {
        self.session
    }
    pub fn tick(&self) -> u64 {
        self.clock.tick()
    }
    pub fn dt(&self) -> Duration {
        self.clock.dt()
    }
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    pub fn rng(&mut self) -> &mut DeterministicRng {
        &mut self.rng
    }
    pub fn log(&self) -> &ChangeLog {
        &self.log
    }
    /// Objects touched by the most recent step's accepted changes.
    pub fn last_touched(&self) -> &[ObjectId] {
        &self.touched
    }

    /// Start a new touch-observation window (clears the accumulated set).
    /// Steps clear it automatically; this is for out-of-step observation
    /// used by contract tests and probes.
    pub fn clear_touch_log(&mut self) {
        self.touched.clear();
    }

    /// Actions presented to the driver for the current tick.
    pub fn actions(&self) -> &[A] {
        &self.inbox
    }

    /// Run one fixed tick. Returns the events emitted this tick.
    pub fn step(&mut self, driver: &mut impl Driver<A, E>, actions: Vec<A>) -> Vec<TickEvent<E>> {
        self.inbox = actions;
        self.outbox.clear();
        self.touched.clear();
        let tick = self.clock.advance();
        driver.tick(self, tick);
        self.inbox.clear();
        std::mem::take(&mut self.outbox)
    }

    /// Emit an event stamped with the current tick and session.
    pub fn emit(&mut self, payload: E) {
        let seq = self.next_event_seq;
        self.next_event_seq += 1;
        self.outbox.push(TickEvent {
            tick: self.clock.tick(),
            seq,
            session: self.session,
            payload,
        });
    }

    /// Submit scene changes. Accepted changes are appended to the change log
    /// and name their objects in `last_touched`; rejected ones (stale /
    /// cross-session / unknown ids) are returned and change nothing.
    pub fn submit(&mut self, changes: &[SceneChange]) -> Vec<SceneError> {
        let mut errors = Vec::new();
        for change in changes {
            let named = named_id(change);
            let errs = self.scene.submit(std::slice::from_ref(change));
            if errs.is_empty() {
                self.log.entries.push(LogEntry {
                    tick: self.clock.tick(),
                    change: change.clone(),
                });
                if let Some(id) = named {
                    self.touched.push(id);
                }
            } else {
                errors.extend(errs);
            }
        }
        errors
    }

    /// Attempt to deliver an event produced by another session (e.g. a late
    /// event from before a reset). Rejected — new sessions are isolated.
    pub fn accept_external_event(
        &mut self,
        event: &TickEvent<E>,
    ) -> Result<(), crate::core::events::QueueError>
    where
        E: Clone,
    {
        if event.session != self.session {
            return Err(crate::core::events::QueueError::StaleSession);
        }
        self.outbox.push(event.clone());
        Ok(())
    }

    /// Full reset: new session, empty scene/log/events, cleared pending
    /// actions, RNG reseeded. Nothing from the prior session survives.
    pub fn reset(&mut self, seed: u64) {
        self.scene.reset();
        self.session = self.scene.session();
        self.clock = TickClock::new(self.clock.dt());
        self.rng.reseed(seed);
        self.log = ChangeLog {
            session: self.session,
            entries: Vec::new(),
        };
        self.touched.clear();
        self.inbox.clear();
        self.outbox.clear();
        self.next_event_seq = 0;
    }
}

/// The id a change names (if any) — for touch tracking.
fn named_id(change: &SceneChange) -> Option<ObjectId> {
    match change {
        SceneChange::Spawn { .. } => None,
        SceneChange::Destroy(id)
        | SceneChange::SetTransform(id, _)
        | SceneChange::SetColor(id, _)
        | SceneChange::SetResource(id, _) => Some(*id),
    }
}
