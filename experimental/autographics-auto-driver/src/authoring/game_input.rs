//! Driver-side GameInput sampling for the character/1 and character3d/1
//! profiles (PLAN-861 contract section 3; PLAN-874 contract section 3). One
//! router consumes a KeyEvent: character runs feed this sampler, legacy runs
//! feed the existing `Input`. OS repeat never creates an edge; edges accumulate
//! only while `Running`; Pause/Stop/Fault/focus loss clear held state and
//! pending edges (Play `clear_clock`).

pub const ACTION_LEFT: usize = 0;
pub const ACTION_RIGHT: usize = 1;
pub const ACTION_JUMP: usize = 2;
pub const ACTION_RESTART: usize = 3;
pub const ACTIONS: usize = 4;

/// character3d/1 shares the first two slots and adds Forward/Back, so Jump and
/// Restart move to 4/5. Each profile's router only emits its own range.
pub const ACTION3_LEFT: usize = 0;
pub const ACTION3_RIGHT: usize = 1;
pub const ACTION3_FORWARD: usize = 2;
pub const ACTION3_BACK: usize = 3;
pub const ACTION3_JUMP: usize = 4;
pub const ACTION3_RESTART: usize = 5;
pub const ACTIONS3: usize = 6;

/// Widest profile slot count; narrower profiles ignore the unused slots.
const SLOTS: usize = ACTIONS3;

#[derive(Default)]
pub struct GameInput {
    held: [bool; SLOTS],
    pending: [bool; SLOTS],
    /// The next dispatch is a neutral Step: send an all-neutral snapshot
    /// regardless of tracked held keys (contract section 3).
    neutral_step: bool,
}

impl GameInput {
    pub fn key(&mut self, action: usize, down: bool, repeat: bool) {
        if action >= SLOTS || repeat {
            return;
        }
        if !down {
            self.held[action] = false;
            return;
        }
        self.held[action] = true;
        self.pending[action] = true;
    }

    /// Track a fresh press without an edge (used while not `Running`; the
    /// contract only accumulates edges during Running dispatches).
    pub fn held_track(&mut self, action: usize) {
        if action < SLOTS {
            self.held[action] = true;
        }
    }

    pub fn set_neutral_step(&mut self) {
        self.neutral_step = true;
    }

    pub fn clear(&mut self) {
        self.held = [false; SLOTS];
        self.pending = [false; SLOTS];
        self.neutral_step = false;
    }

    /// Fold the current snapshot into dispatch: held bits plus every pending
    /// edge (each dispatch consumes them; rendering alone never does).
    pub fn fold(&mut self) -> (i32, i32) {
        if self.neutral_step {
            self.neutral_step = false;
            self.pending = [false; SLOTS];
            return (0, 0);
        }
        let held = held_bits(&self.held);
        let pressed = held_bits(&self.pending);
        self.pending = [false; SLOTS];
        (held, pressed)
    }
}

fn held_bits(flags: &[bool; SLOTS]) -> i32 {
    flags
        .iter()
        .enumerate()
        .fold(0, |acc, (i, f)| acc | (i32::from(*f) << i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_never_creates_edges_and_fold_consumes_pending_once() {
        let mut g = GameInput::default();
        g.key(ACTION_JUMP, true, true);
        assert_eq!(g.fold(), (0, 0));
        g.key(ACTION_JUMP, true, false);
        g.key(ACTION_LEFT, true, false);
        // Fast down/up between dispatches keeps at least one pending edge.
        g.key(ACTION_RESTART, true, false);
        g.key(ACTION_RESTART, false, false);
        g.key(ACTION_RESTART, true, false);
        let (held, pressed) = g.fold();
        assert_eq!(held, 1 | 4 | 8);
        assert_eq!(pressed, 1 | 4 | 8);
        // The same tick's second fold sees no pending edges.
        let (held2, pressed2) = g.fold();
        assert_eq!(held2, 1 | 4 | 8);
        assert_eq!(pressed2, 0);
        // Release clears held only.
        g.key(ACTION_LEFT, false, false);
        assert_eq!(g.fold().0, 4 | 8);
    }

    #[test]
    fn neutral_step_sends_an_all_neutral_snapshot() {
        let mut g = GameInput::default();
        g.key(ACTION_RIGHT, true, false);
        g.set_neutral_step();
        assert_eq!(g.fold(), (0, 0));
        // The following dispatch reports the tracked held keys again.
        assert_eq!(g.fold().0, 2);
        g.clear();
        assert_eq!(g.fold(), (0, 0));
    }

    /// character3d/1: six actions pack into one held/pressed int each; the
    /// second fold of the same tick sees no edges and out-of-range slots stay
    /// unwritable.
    #[test]
    fn six_action_profile_packs_forward_back_and_edges_once() {
        let mut g = GameInput::default();
        g.key(ACTION3_FORWARD, true, false);
        g.key(ACTION3_BACK, true, false);
        g.key(ACTION3_JUMP, true, false);
        g.key(ACTION3_RESTART, true, false);
        let (held, pressed) = g.fold();
        assert_eq!(held, (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5));
        assert_eq!(pressed, held);
        assert_eq!(g.fold(), (held, 0));
        // A 2D-only router cannot silently drive a 3D slot high: the slot
        // bound is the widest profile, so this checks the unused slot is still
        // inert when the 2D action count is honoured by the router.
        g.key(ACTIONS3, true, false);
        assert_eq!(g.fold(), (held, 0));
        g.clear();
        assert_eq!(g.fold(), (0, 0));
    }
}
