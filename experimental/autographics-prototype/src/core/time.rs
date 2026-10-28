//! Fixed-timestep clock (PLAN-829 oracle: gameplay advances in fixed ticks;
//! rendering is decoupled).

use std::time::Duration;

/// Fixed-step simulation clock. `dt` is frozen at construction; `advance`
/// must be driven by the host (real loop or replay runner).
#[derive(Debug, Clone)]
pub struct TickClock {
    dt: Duration,
    tick: u64,
    /// Accumulated real time not yet consumed by a tick (host-side).
    acc: Duration,
}

impl TickClock {
    pub fn new(dt: Duration) -> Self {
        assert!(dt > Duration::ZERO, "TickClock dt must be positive");
        TickClock {
            dt,
            tick: 0,
            acc: Duration::ZERO,
        }
    }

    /// Feed real elapsed time; returns how many whole ticks to run now.
    pub fn feed(&mut self, elapsed: Duration) -> u64 {
        self.acc += elapsed;
        let n = (self.acc.as_nanos() / self.dt.as_nanos()) as u64;
        self.acc -= self.dt * n as u32;
        n
    }

    /// Advance exactly one tick (replay runners call this directly).
    pub fn advance(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }

    /// Feed elapsed wall time and return how many whole ticks to run NOW,
    /// capped at `max_ticks` (spiral-of-death protection). When the cap
    /// binds, the excess accumulated time is DROPPED (the simulation slows
    /// down under sustained overload rather than accumulating an
    /// ever-growing debt). Below the cap the accumulator carries the
    /// remainder, so any redraw cadence produces the same tick count for
    /// the same total elapsed time (within one tick at a boundary).
    pub fn feed_capped(&mut self, elapsed: Duration, max_ticks: u32) -> u64 {
        self.acc += elapsed;
        let whole = (self.acc.as_nanos() / self.dt.as_nanos()) as u64;
        if whole > max_ticks as u64 {
            // overload: run the cap, drop the debt.
            self.acc = Duration::ZERO;
            return max_ticks as u64;
        }
        self.acc -= self.dt * whole as u32;
        whole
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capped_feed_is_cadence_invariant_and_bounded() {
        let dt = Duration::from_nanos(8_333_333); // ~1/120 s
        let total = Duration::from_millis(1000);
        // same total elapsed fed at 30/60/120-slice cadences: equal tick
        // counts (within one boundary tick).
        let mut counts = Vec::new();
        for slices in [30u32, 60, 120] {
            let mut clock = TickClock::new(dt);
            let per = total / slices;
            let mut n = 0u64;
            for _ in 0..slices {
                n += clock.feed_capped(per, 16);
            }
            counts.push(n);
        }
        assert!(
            counts.iter().max().unwrap() - counts.iter().min().unwrap() <= 1,
            "cadence invariance: {counts:?}"
        );
        assert!(counts[0] >= 119, "1s at 120Hz >= 119 ticks: {counts:?}");
        // overload: a huge elapsed burst is capped and dropped.
        let mut clock = TickClock::new(dt);
        assert_eq!(clock.feed_capped(Duration::from_secs(10), 5), 5);
        // after the drop there is no debt: the next small feed yields one tick.
        assert_eq!(clock.feed_capped(dt, 5), 1);
        // aligned slices yield exactly one tick per dt fed (any cadence).
        let mut clock = TickClock::new(dt);
        for _ in 0..10 {
            assert_eq!(clock.feed_capped(dt, 16), 1);
        }
    }

    #[test]
    fn fixed_dt_accumulation() {
        let mut clock = TickClock::new(Duration::from_millis(16));
        assert_eq!(clock.feed(Duration::from_millis(50)), 3);
        assert_eq!(clock.tick(), 0); // feed reports; advance drives
        clock.advance();
        clock.advance();
        clock.advance();
        assert_eq!(clock.tick(), 3);
        assert_eq!(clock.feed(Duration::from_millis(1)), 0);
    }
}
