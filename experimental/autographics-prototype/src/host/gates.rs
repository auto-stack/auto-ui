//! Verifier gates (V-08): independent checks that do NOT trust the game's
//! own PASS/0-count printing — they re-derive truth from the event stream,
//! scene snapshots, keyframe hashes, and device tokens. Pure functions so
//! the negative-controls case and the contract tests share one
//! implementation.

use crate::games::squash::{SquashEvent, SQUASH_DOT_FOR_GATE};
use crate::render3d::collision::PlayerMobContact;

/// Gate 1 (missed scoring): the claimed score must equal the number of
/// Squashed events in the independent event stream.
pub fn verify_score_matches_events(
    events: &[SquashEvent],
    claimed_score: u32,
) -> Result<(), String> {
    let squashes = events
        .iter()
        .filter(|e| matches!(e, SquashEvent::Squashed { .. }))
        .count();
    if squashes as u32 != claimed_score {
        return Err(format!(
            "missed/phantom scoring: {squashes} squash events vs claimed score {claimed_score}"
        ));
    }
    // score progression must be exactly 1 per squash.
    let mut last = 0u32;
    for e in events {
        if let SquashEvent::Squashed { score, .. } = e {
            if *score != last + 1 {
                return Err(format!("score progression jumped {last} -> {score}"));
            }
            last = *score;
        }
    }
    Ok(())
}

/// Gate 2 (wrong squash semantics): with the frozen classifier, a scripted
/// SIDE contact must produce PlayerDied and NO Squashed event. A tampered
/// build that counts side contacts as squashes fails here.
pub fn verify_side_contact_is_fatal(
    events: &[SquashEvent],
    contact_normal_y: f32,
) -> Result<(), String> {
    let classified = if contact_normal_y > SQUASH_DOT_FOR_GATE {
        PlayerMobContact::TopLandingSquash
    } else {
        PlayerMobContact::Fatal
    };
    match classified {
        PlayerMobContact::Fatal => {
            let died = events.iter().any(|e| matches!(e, SquashEvent::PlayerDied));
            let squashed = events
                .iter()
                .any(|e| matches!(e, SquashEvent::Squashed { .. }));
            if !died || squashed {
                return Err(format!(
                    "side contact mis-handled: died={died} squashed={squashed} (normal_y={contact_normal_y})"
                ));
            }
            Ok(())
        }
        PlayerMobContact::TopLandingSquash => Err(format!(
            "classifier accepted a non-top contact (normal_y={contact_normal_y}) as a squash"
        )),
    }
}

/// Gate 3 (blank/undriven frames): two frames that SHOULD differ (sim time
/// advanced between them) must not hash equal. Equal hashes ⇒ undriven.
pub fn verify_frame_progression(
    expected_driven: bool,
    hash_a: &str,
    hash_b: &str,
) -> Result<(), String> {
    if expected_driven {
        if hash_a == hash_b {
            return Err("frames identical despite sim time advancing (undriven/blank)".into());
        }
    } else if hash_a != hash_b {
        return Err("positive control mismatch: expected identical initial frames".into());
    }
    Ok(())
}

/// Gate 4 (second device): every render target must carry the game's device
/// token.
pub fn verify_same_device(game_token: &str, target_tokens: &[String]) -> Result<(), String> {
    for (i, t) in target_tokens.iter().enumerate() {
        if t != game_token {
            return Err(format!(
                "target {i} runs on a different device ({t} != {game_token})"
            ));
        }
    }
    Ok(())
}

/// Gate 5 (late events): every event in the stream must belong to the given
/// (current) session.
pub fn verify_session_isolation(
    session: crate::core::id::SessionId,
    events: &[crate::core::runtime::TickEvent<SquashEvent>],
) -> Result<(), String> {
    for e in events {
        if e.session != session {
            return Err(format!(
                "late/prior-session event delivered (seq {} session {:?})",
                e.seq, e.session
            ));
        }
    }
    Ok(())
}
