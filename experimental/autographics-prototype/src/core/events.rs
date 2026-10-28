//! Ordered action (game-ward) and event (host-ward) queues.
//!
//! Every entry carries a monotonic sequence number and the session it belongs
//! to; late entries from a dead session are rejected on push (PLAN-829 §5.3:
/// stale events must not affect a new session).
use crate::core::id::SessionId;
use std::collections::VecDeque;

/// One queued entry: payload + sequence + origin session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queued<T> {
    pub seq: u64,
    pub session: SessionId,
    pub payload: T,
}

/// Errors observable by producers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    /// Entry was produced by a session that is no longer current.
    StaleSession,
}

/// Directional queue with session gating. `A` = actions sent INTO the game
/// (inputs), `E` = events emitted BY the game/host (observations). Both use
/// the same machinery; two type aliases document direction.
#[derive(Debug)]
pub struct Queue<T> {
    session: SessionId,
    next_seq: u64,
    inner: VecDeque<Queued<T>>,
}

impl<T> Queue<T> {
    pub fn new(session: SessionId) -> Self {
        Queue {
            session,
            next_seq: 0,
            inner: VecDeque::new(),
        }
    }

    /// Push a locally-produced entry (always accepted).
    pub fn push_local(&mut self, payload: T) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.inner.push_back(Queued {
            seq,
            session: self.session,
            payload,
        });
        seq
    }

    /// Push an externally-produced entry; reject if from another session.
    pub fn push_external(&mut self, session: SessionId, payload: T) -> Result<u64, QueueError> {
        if session != self.session {
            return Err(QueueError::StaleSession);
        }
        Ok(self.push_local(payload))
    }

    pub fn session(&self) -> SessionId {
        self.session
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Queued<T>> + '_ {
        self.inner.drain(..)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Actions flowing into game logic (key pressed, etc.).
pub type ActionQueue<A> = Queue<A>;
/// Events/observations flowing out (collision happened, frame presented).
pub type EventQueue<E> = Queue<E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_session_push_rejected() {
        let session = SessionId::new();
        let mut q: Queue<u8> = Queue::new(session);
        assert!(q.push_external(session, 1).is_ok());
        let dead = SessionId::new();
        assert_eq!(q.push_external(dead, 2), Err(QueueError::StaleSession));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn sequence_monotonic() {
        let session = SessionId::new();
        let mut q: Queue<u8> = Queue::new(session);
        let s0 = q.push_local(10);
        let s1 = q.push_local(20);
        assert!(s1 > s0);
        let seqs: Vec<_> = q.drain().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![s0, s1]);
    }
}
