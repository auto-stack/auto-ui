//! The float-bob animation, frozen from the inline Animation sub-resources in
//! `Player.tscn` / `Mob.tscn` (the GLBs contain NO animations — evidence note
//! E2). Both scenes define IDENTICAL tracks:
//!
//! - `Player.tscn` lines 17–43 / `Mob.tscn` lines 12–38 (frozen copies in
//!   `testdata/squash_the_creeps/`):
//!   - length 1.2 s, `loop_mode = 1` (looping), interpolation linear
//!     (`interp = 1`), `update = 0` (continuous);
//!   - track 0, `Pivot/Character:position`, keys t=(0.22, 0.7, 1.18) s,
//!     values y=(0.329753, 0.660351, 0.349734), x/z constant 0;
//!   - track 1, `Pivot/Character:rotation_degrees`, keys t=(0.1, 0.46, 1.18)
//!     s, values x=(-5.0326, 10.0, -10.0) degrees, y/z = 0.
//!
//! `speed_scale` is runtime-multiplied: player ×4 while moving
//! (`Player.gd` line 31), mob `random_speed / min_speed` (`Mob.gd` line 32).

/// Deterministic transform animation: y offset + x-rotation (radians) of the
/// `Pivot/Character` node at time `t` seconds (pre-speed-scale).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BobAnimation {
    pub length: f32,
}

pub const BOB_KEYS_Y: [(f32, f32); 3] = [(0.22, 0.329753), (0.7, 0.660351), (1.18, 0.349734)];
pub const BOB_KEYS_ROT_X: [(f32, f32); 3] = [(0.1, -5.0326), (0.46, 10.0), (1.18, -10.0)];
pub const BOB_LENGTH: f32 = 1.2;

impl Default for BobAnimation {
    fn default() -> Self {
        BobAnimation { length: BOB_LENGTH }
    }
}

impl BobAnimation {
    /// Sample the animation. `time` is in seconds AFTER speed scaling;
    /// loops with linear interpolation between keys.
    pub fn eval(&self, time: f32) -> (f32, f32) {
        let t = time.rem_euclid(self.length);
        (
            lerp_keys(&BOB_KEYS_Y, t),
            lerp_keys(&BOB_KEYS_ROT_X, t).to_radians(),
        )
    }

    /// Time in the loop for a given tick at fixed dt and speed scale.
    pub fn time_at(&self, tick: u64, dt: f32, speed_scale: f32) -> f32 {
        tick as f32 * dt * speed_scale
    }
}

/// Linear interpolation over cyclically-extended keys (last key value holds
/// to `length`, then wraps).
fn lerp_keys(keys: &[(f32, f32)], t: f32) -> f32 {
    if t <= keys[0].0 {
        return keys[0].1;
    }
    for w in keys.windows(2) {
        let (t0, v0) = w[0];
        let (t1, v1) = w[1];
        if t <= t1 {
            let f = (t - t0) / (t1 - t0);
            return v0 + (v1 - v0) * f;
        }
    }
    *keys.last().map(|(_, v)| v).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_match_tscn_exactly() {
        let bob = BobAnimation::default();
        for (t, want_y) in BOB_KEYS_Y {
            let (y, _) = bob.eval(t);
            assert!((y - want_y).abs() < 1e-6, "key @{t}: {y} vs {want_y}");
        }
        for (t, want_deg) in BOB_KEYS_ROT_X {
            let (_, rot) = bob.eval(t);
            assert!((rot.to_degrees() - want_deg).abs() < 1e-4, "rot key @{t}");
        }
    }

    #[test]
    fn loops_deterministically() {
        let bob = BobAnimation::default();
        let a = bob.eval(0.35);
        let b = bob.eval(0.35 + BOB_LENGTH * 7.0);
        assert!(
            (a.0 - b.0).abs() < 1e-5 && (a.1 - b.1).abs() < 1e-5,
            "loop wraps: {a:?} vs {b:?}"
        );
        // between keys: linear
        let (y0, _) = bob.eval(0.22);
        let (y1, _) = bob.eval(0.7);
        let (ym, _) = bob.eval((0.22 + 0.7) * 0.5);
        assert!((ym - (y0 + y1) * 0.5).abs() < 1e-5);
    }
}
