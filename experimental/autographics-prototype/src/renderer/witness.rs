//! Owned-lifetime witness (review r1 F06): derives "was it actually
//! released?" from Drop, not from counters incremented by hand. A value
//! wrapped in `Tracked` decrements the shared counter exactly when its
//! lifetime ends — `std::mem::forget` leaks are observable, so a lifecycle
//! gate built on this witness cannot be green while targets are retained.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct DropWitness {
    alive: Arc<AtomicUsize>,
}

impl DropWitness {
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrap a value so its death decrements the counter.
    pub fn track<T>(&self, value: T) -> Tracked<T> {
        self.alive.fetch_add(1, Ordering::SeqCst);
        Tracked {
            value,
            witness: self.clone(),
        }
    }

    /// Currently alive tracked values (derived from Drop).
    pub fn alive(&self) -> usize {
        self.alive.load(Ordering::SeqCst)
    }
}

pub struct Tracked<T> {
    pub value: T,
    witness: DropWitness,
}

impl<T> std::ops::Deref for Tracked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Tracked<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for Tracked<T> {
    fn drop(&mut self) {
        self.witness.alive.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Permanent negative control for the F06 lifecycle gate: forgetting a
    /// tracked value leaks it, and the witness PROVES it (the review's
    /// injection — 20 `mem::forget` targets with a hand-incremented "dropped"
    /// counter — cannot pass a gate derived from this witness).
    #[test]
    fn witness_detects_forget_leaks() {
        let witness = DropWitness::new();
        let mut kept = Vec::new();
        for _ in 0..20 {
            kept.push(witness.track(String::from("target")));
        }
        assert_eq!(witness.alive(), 20);
        // honest release
        kept.truncate(10);
        assert_eq!(witness.alive(), 10);
        // the review's mutation: leak five of them
        let mut leaked = Vec::new();
        for _ in 0..5 {
            leaked.push(witness.track(String::from("leak")));
        }
        assert_eq!(witness.alive(), 15);
        std::mem::forget(leaked);
        assert_eq!(witness.alive(), 15, "forgotten values stay alive");
        // a lifecycle gate comparing against the baseline fails here:
        assert_ne!(witness.alive(), 0, "gate predicate: leaks are red");
        // drop the rest
        drop(kept);
        assert_eq!(witness.alive(), 5, "only the forgotten five remain");
    }
}
