//! Stable object identities with generation + session semantics.
//!
//! Contract (PLAN-829 §5.3): an `ObjectId` that has been freed and whose slot
//! reused MUST be rejected by every consumer. IDs carry the session that
//! minted them; IDs from another session are never valid here.

use std::sync::atomic::{AtomicU64, Ordering};

/// Process-unique session token. New sessions (e.g. after a host restart)
/// never reuse the previous session's id space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(u64);

impl SessionId {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        SessionId(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// A stable object identity: arena index + generation + minting session.
/// Ordered for deterministic iteration in sets/maps (PLAN-830 composition
/// dirty tracking and render order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId {
    index: u32,
    generation: u32,
    session: SessionId,
}

impl ObjectId {
    pub fn index(self) -> u32 {
        self.index
    }
    pub fn generation(self) -> u32 {
        self.generation
    }
    pub fn session(self) -> SessionId {
        self.session
    }

    /// Compact, report-friendly form: `s<session>:i<index>:g<generation>`.
    pub fn token(self) -> String {
        format!(
            "s{}:i{}:g{}",
            self.session.raw(),
            self.index,
            self.generation
        )
    }
}

/// Why an id was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    /// Slot has never been allocated.
    Unknown,
    /// Slot was freed and (possibly) reused; this id is stale.
    Stale { current_generation: u32 },
    /// Id belongs to a different session.
    WrongSession { current: SessionId },
}

/// Generational arena allocator. `Clone` duplicates the allocator state
/// exactly (PLAN-830: composition-batch validation simulates minting on a
/// clone so a rejected batch never consumes real id space).
#[derive(Debug, Clone)]
pub struct IdAllocator {
    session: SessionId,
    generations: Vec<u32>,
    free: Vec<u32>,
    live: usize,
}

impl IdAllocator {
    pub fn new() -> Self {
        Self::with_session(SessionId::new())
    }

    /// Allocator bound to an EXISTING session (change-log replay rebuilds the
    /// original id space instead of minting a fresh one).
    pub fn with_session(session: SessionId) -> Self {
        IdAllocator {
            session,
            generations: Vec::new(),
            free: Vec::new(),
            live: 0,
        }
    }

    pub fn session(&self) -> SessionId {
        self.session
    }

    /// Allocate a fresh identity (or reuse a freed slot with bumped
    /// generation — old identities for that slot stay invalid).
    pub fn allocate(&mut self) -> ObjectId {
        self.live += 1;
        if let Some(index) = self.free.pop() {
            self.generations[index as usize] += 1;
            ObjectId {
                index,
                generation: self.generations[index as usize],
                session: self.session,
            }
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            ObjectId {
                index,
                generation: 0,
                session: self.session,
            }
        }
    }

    /// Free an identity. Freeing an already-stale/wrong-session id is an
    /// error (double-free and cross-session release are rejected, not
    /// silently absorbed).
    pub fn free(&mut self, id: ObjectId) -> Result<(), IdentityError> {
        self.validate(id)?;
        self.generations[id.index as usize] += 1;
        self.free.push(id.index);
        self.live -= 1;
        Ok(())
    }

    /// Validate an identity against the current allocator state.
    pub fn validate(&self, id: ObjectId) -> Result<(), IdentityError> {
        if id.session != self.session {
            return Err(IdentityError::WrongSession {
                current: self.session,
            });
        }
        match self.generations.get(id.index as usize) {
            None => Err(IdentityError::Unknown),
            Some(&g) if g == id.generation => Ok(()),
            Some(&g) => Err(IdentityError::Stale {
                current_generation: g,
            }),
        }
    }

    pub fn live_count(&self) -> usize {
        self.live
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reused_slot_rejects_stale_id() {
        let mut alloc = IdAllocator::new();
        let a = alloc.allocate();
        alloc.free(a).unwrap();
        let b = alloc.allocate(); // same index, generation+1
        assert_eq!(a.index(), b.index());
        assert!(alloc.validate(a).is_err());
        assert!(alloc.validate(b).is_ok());
    }

    #[test]
    fn double_free_rejected() {
        let mut alloc = IdAllocator::new();
        let a = alloc.allocate();
        alloc.free(a).unwrap();
        assert!(alloc.free(a).is_err());
    }

    #[test]
    fn wrong_session_rejected() {
        let mut alloc = IdAllocator::new();
        let a = alloc.allocate();
        let other_session = SessionId::new();
        let foreign = ObjectId {
            session: other_session,
            ..a
        };
        assert!(alloc.validate(foreign).is_err());
    }
}
