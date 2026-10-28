//! GPU-agnostic core: identities, time, actions/events, scene submission,
//! the shared runtime (G-base), deterministic RNG, and the demo driver used
//! by contract tests and probes.
//!
//! Nothing in this module may reference wgpu/winit/Iced types (PLAN-829 §2).

pub mod composition;
pub mod demo;
pub mod events;
pub mod id;
pub mod rng;
pub mod runtime;
pub mod scene;
pub mod time;

pub use events::{ActionQueue, EventQueue, Queue, QueueError};
pub use id::{IdAllocator, IdentityError, ObjectId, SessionId};
pub use rng::DeterministicRng;
pub use runtime::{ChangeLog, Driver, LogEntry, Runtime, TickEvent};
pub use scene::{Color, Scene, SceneChange, SceneError, Transform};
pub use time::TickClock;
