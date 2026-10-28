//! LIVE-mode input state machine (review r1 F01): the real keyboard path
//! for interactive play. Frozen Player.gd:39 jumps on
//! `is_action_just_pressed("jump")` — a PRESS EDGE, not a held key and not
//! an unconditional auto-jump. This machine is what the windowed host
//! drives; tests feed synthetic key events through the SAME code path.

use super::SquashAction;

/// Physical keys the live host maps (winit `KeyCode`s are converted by the
/// host; the machine stays UI-framework-agnostic and testable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveKey {
    Space,
    Enter,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    KeyW,
    KeyA,
    KeyS,
    KeyD,
    KeyR,
}

/// Host-extension effects (not part of the frozen input profile).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveEffect {
    RetryPressed,
    ResetPressed,
}

#[derive(Debug, Default)]
pub struct LiveSquashInput {
    space_held: bool,
    jump_edge: bool,
    retry_edge: bool,
    reset_edge: bool,
    left: bool,
    right: bool,
    forward: bool,
    back: bool,
}

impl LiveSquashInput {
    /// Feed one key event (pressed/released). `dead` is the driver's
    /// dead-state this event (the frozen retry overlay state). Returns a
    /// host-extension effect when one fired this event.
    ///
    /// Frozen binding (Main.gd `_unhandled_input`: ui_accept while Retry is
    /// visible ⇒ reload; manifest E6 promises "Space or Enter"):
    /// - Space alive: jump press edge (Player.gd line 39).
    /// - Space dead: retry edge — NO jump edge is armed, so a stale edge
    ///   can never fire a jump on the first tick after the reload (r4).
    /// - Enter dead: retry edge. Enter alive: nothing (the host used to
    ///   gate this; the machine now encodes it — review r4 F829-R4-02
    ///   sibling audit).
    pub fn key_event(&mut self, key: LiveKey, pressed: bool, dead: bool) -> Option<LiveEffect> {
        match key {
            LiveKey::Space => {
                if pressed && !self.space_held {
                    // just_pressed edge: exactly one jump per press.
                    if dead {
                        // retry edge: NO jump edge is armed, so a stale
                        // edge can never fire on the first tick after the
                        // reload. space_held updates before the return.
                        self.space_held = pressed;
                        self.retry_edge = true;
                        return Some(LiveEffect::RetryPressed);
                    }
                    self.jump_edge = true;
                }
                self.space_held = pressed;
                None
            }
            LiveKey::Enter => {
                if pressed && dead {
                    self.retry_edge = true;
                    return Some(LiveEffect::RetryPressed);
                }
                None
            }
            LiveKey::KeyR => {
                if pressed {
                    self.reset_edge = true;
                    return Some(LiveEffect::ResetPressed);
                }
                None
            }
            LiveKey::ArrowLeft | LiveKey::KeyA => {
                self.left = pressed;
                None
            }
            LiveKey::ArrowRight | LiveKey::KeyD => {
                self.right = pressed;
                None
            }
            LiveKey::ArrowUp | LiveKey::KeyW => {
                self.forward = pressed;
                None
            }
            LiveKey::ArrowDown | LiveKey::KeyS => {
                self.back = pressed;
                None
            }
        }
    }

    /// Build this tick's action. `dead` gates the jump (the frozen retry
    /// overlay state): a dead player cannot jump. The edge is CONSUMED each
    /// tick — holding Space never re-fires until released and re-pressed.
    pub fn action(&mut self, dead: bool) -> SquashAction {
        let jump = self.jump_edge && !dead;
        self.jump_edge = false;
        let move_x = match (self.left, self.right) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };
        let move_z = match (self.forward, self.back) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };
        SquashAction {
            move_x,
            move_z,
            jump,
            retry: false,
            reset: false,
        }
    }

    /// True exactly once per Enter press (host extension; the caller only
    /// acts on it while the retry overlay is visible).
    pub fn take_retry_edge(&mut self) -> bool {
        let r = self.retry_edge;
        self.retry_edge = false;
        r
    }

    pub fn take_reset_edge(&mut self) -> bool {
        let r = self.reset_edge;
        self.reset_edge = false;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_idle_never_jumps() {
        let mut input = LiveSquashInput::default();
        for _ in 0..10 {
            assert!(!input.action(false).jump, "idle must not jump");
        }
    }

    #[test]
    fn single_space_press_is_exactly_one_jump() {
        let mut input = LiveSquashInput::default();
        input.key_event(LiveKey::Space, true, false);
        assert!(input.action(false).jump, "press edge fires once");
        for _ in 0..5 {
            assert!(
                !input.action(false).jump,
                "held Space must not re-fire the jump"
            );
        }
        input.key_event(LiveKey::Space, false, false);
        for _ in 0..3 {
            assert!(!input.action(false).jump);
        }
        input.key_event(LiveKey::Space, true, false);
        assert!(input.action(false).jump, "re-press fires again");
        assert!(!input.action(false).jump);
    }

    #[test]
    fn dead_player_cannot_jump() {
        let mut input = LiveSquashInput::default();
        input.key_event(LiveKey::Space, true, false);
        assert!(!input.action(true).jump, "dead gates the jump");
        assert!(!input.action(true).jump);
    }

    #[test]
    fn movement_keys_map_to_digital_actions() {
        let mut input = LiveSquashInput::default();
        input.key_event(LiveKey::KeyW, true, false);
        let a = input.action(false);
        assert_eq!((a.move_x, a.move_z), (0, -1));
        input.key_event(LiveKey::ArrowRight, true, false);
        let a = input.action(false);
        assert_eq!((a.move_x, a.move_z), (1, -1));
        input.key_event(LiveKey::KeyW, false, false);
        input.key_event(LiveKey::ArrowRight, false, false);
        let a = input.action(false);
        assert_eq!((a.move_x, a.move_z), (0, 0));
    }

    #[test]
    fn retry_edge_fires_once_per_press() {
        let mut input = LiveSquashInput::default();
        assert!(input.take_retry_edge() == false);
        assert_eq!(
            input.key_event(LiveKey::Enter, true, true),
            Some(LiveEffect::RetryPressed)
        );
        assert!(input.take_retry_edge());
        assert!(!input.take_retry_edge(), "edge consumed");
    }

    /// F829-R4-02: Space restarts the failed game (frozen ui_accept promise:
    /// "Space or Enter", manifest E6) — through the same press-edge machine.
    #[test]
    fn r4_dead_space_fires_retry_not_jump() {
        let mut input = LiveSquashInput::default();
        assert_eq!(
            input.key_event(LiveKey::Space, true, true),
            Some(LiveEffect::RetryPressed),
            "dead Space must offer retry"
        );
        assert!(input.take_retry_edge());
        // no jump edge was armed: the first tick after reload must not jump.
        assert!(
            !input.action(false).jump,
            "stale Space edge must not fire a jump after the reload"
        );
        assert!(!input.action(false).jump);
    }

    #[test]
    fn r4_dead_enter_fires_retry() {
        let mut input = LiveSquashInput::default();
        assert_eq!(
            input.key_event(LiveKey::Enter, true, true),
            Some(LiveEffect::RetryPressed)
        );
        assert!(input.take_retry_edge());
        // Enter while alive is nothing (sibling audit: retry class is
        // dead-only in the machine, not just host-gated).
        let mut alive = LiveSquashInput::default();
        assert_eq!(alive.key_event(LiveKey::Enter, true, false), None);
        assert!(!alive.take_retry_edge(), "alive Enter arms no retry edge");
    }

    #[test]
    fn r4_living_space_is_jump_edge_only() {
        let mut input = LiveSquashInput::default();
        assert_eq!(input.key_event(LiveKey::Space, true, false), None);
        assert!(!input.take_retry_edge(), "living Space arms no retry edge");
        assert!(input.action(false).jump, "jump edge consumed");
        // held through a death and reload: no new edge until re-press.
        input.key_event(LiveKey::Space, true, true); // still held: no edge
        assert_eq!(
            input.key_event(LiveKey::Space, false, true),
            None,
            "release never fires an effect"
        );
        assert_eq!(
            input.key_event(LiveKey::Space, true, false),
            None,
            "alive re-press is a jump edge, not a retry effect"
        );
        assert!(
            input.action(false).jump,
            "fresh alive press after release fires exactly one jump"
        );
        assert!(!input.action(false).jump);
    }
}
