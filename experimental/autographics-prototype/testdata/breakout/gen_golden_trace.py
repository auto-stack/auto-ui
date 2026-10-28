#!/usr/bin/env python3
"""Independent golden-trace generator for AG-2D-01 (PLAN-829 T-04).

PROVENANCE: this is a direct translation of the frozen sample
`bevyengine/bevy@efe61cc5b9872f3f9e1ee7553325612c094ccc97`,
`examples/showcase/breakout.rs` (blob b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954),
as parsed in `research/sample-manifest.md` §1. All arithmetic is IEEE f32 via
numpy to match the Rust implementation bit-for-bit (same op order). The frozen
oracle dt is f32(1/60); system order per tick: apply_velocity -> move_paddle ->
check_for_collisions; collider iteration order (oracle-frozen): left wall,
right wall, bottom wall, top wall, paddle, bricks row-major.

The paddle action stream is produced by a deterministic follow policy
(paddle tracks ball x with a 2-unit dead zone) evaluated during the SAME pass
that produces the trace, so stream and trace are consistent by construction.
The Rust game under test consumes the FROZEN stream and is compared against
the FROZEN trace (positions |d| <= 1e-3, score/contact/sound counts exact,
bounce side sequence exact) — it cannot regenerate this file.

Regenerate:  python testdata/breakout/gen_golden_trace.py
"""
import json
import math
import numpy as np
import os

f32 = np.float32

# --- frozen constants (breakout.rs) -----------------------------------------
PADDLE_W, PADDLE_H = f32(120.0), f32(20.0)
PADDLE_SPEED = f32(500.0)
PADDLE_PADDING = f32(10.0)
BALL_START = (f32(0.0), f32(-50.0))
BALL_R = f32(15.0)          # BALL_DIAMETER / 2
BALL_SPEED = f32(400.0)
INITIAL_DIR = (f32(0.5), f32(-0.5))
WALL_T = f32(10.0)
LEFT_WALL, RIGHT_WALL = f32(-450.0), f32(450.0)
BOTTOM_WALL, TOP_WALL = f32(-300.0), f32(300.0)
BRICK_W, BRICK_H = f32(100.0), f32(30.0)
GAP_PADDLE_BRICKS = f32(270.0)
GAP_BRICKS = f32(5.0)
GAP_CEILING = f32(20.0)
GAP_SIDES = f32(20.0)
DT = f32(1.0) / f32(60.0)

TICKS = 10800

# --- derived layout (exactly as breakout.rs computes) ------------------------
paddle_y = BOTTOM_WALL + f32(60.0)  # GAP_BETWEEN_PADDLE_AND_FLOOR = 60
total_w = (RIGHT_WALL - LEFT_WALL) - f32(2.0) * GAP_SIDES
bottom_edge = paddle_y + GAP_PADDLE_BRICKS
total_h = TOP_WALL - bottom_edge - GAP_CEILING
n_cols = int(math.floor(total_w / (BRICK_W + GAP_BRICKS)))
n_rows = int(math.floor(total_h / (BRICK_H + GAP_BRICKS)))
center = (LEFT_WALL + RIGHT_WALL) / f32(2.0)
left_edge = center - (f32(n_cols) / f32(2.0) * BRICK_W) - (f32(n_cols - 1) / f32(2.0) * GAP_BRICKS)
off_x = left_edge + BRICK_W / f32(2.0)
off_y = bottom_edge + BRICK_H / f32(2.0)
BRICKS = [
    (off_x + f32(c) * (BRICK_W + GAP_BRICKS), off_y + f32(r) * (BRICK_H + GAP_BRICKS))
    for r in range(n_rows) for c in range(n_cols)
]
assert n_cols == 8 and n_rows == 7 and len(BRICKS) == 56

PADDLE_MIN = LEFT_WALL + WALL_T / f32(2.0) + PADDLE_W / f32(2.0) + PADDLE_PADDING
PADDLE_MAX = RIGHT_WALL - WALL_T / f32(2.0) - PADDLE_W / f32(2.0) - PADDLE_PADDING

# collider half-extents: (cx, cy, hx, hy, kind, index)
def colliders(paddle_x, alive):
    out = [
        (LEFT_WALL, f32(0.0), WALL_T / f32(2.0), (TOP_WALL - BOTTOM_WALL + WALL_T) / f32(2.0), "wall_left", None),
        (RIGHT_WALL, f32(0.0), WALL_T / f32(2.0), (TOP_WALL - BOTTOM_WALL + WALL_T) / f32(2.0), "wall_right", None),
        (f32(0.0), BOTTOM_WALL, (RIGHT_WALL - LEFT_WALL + WALL_T) / f32(2.0), WALL_T / f32(2.0), "wall_bottom", None),
        (f32(0.0), TOP_WALL, (RIGHT_WALL - LEFT_WALL + WALL_T) / f32(2.0), WALL_T / f32(2.0), "wall_top", None),
        (f32(paddle_x), paddle_y, PADDLE_W / f32(2.0), PADDLE_H / f32(2.0), "paddle", None),
    ]
    for i, (bx, by) in enumerate(BRICKS):
        if alive[i]:
            out.append((bx, by, BRICK_W / f32(2.0), BRICK_H / f32(2.0), "brick", i))
    return out


def normalize(v):
    l = f32(math.sqrt(f32(v[0] * v[0] + v[1] * v[1])))
    return (f32(v[0] / l), f32(v[1] / l))


def main():
    vel = tuple(f32(c * BALL_SPEED) for c in normalize(INITIAL_DIR))
    ball = [f32(BALL_START[0]), f32(BALL_START[1])]
    paddle_x = f32(0.0)
    score = 0
    alive = [True] * len(BRICKS)
    sounds = 0

    actions = []
    ticks = []
    contacts_detail = []

    for t in range(TICKS):
        # --- action from follow policy (frozen with the trace) ---
        if ball[0] < paddle_x - f32(2.0):
            policy = -1
        elif ball[0] > paddle_x + f32(2.0):
            policy = 1
        else:
            policy = 0
        # Scripted miss windows: the paddle deliberately runs AWAY, so the
        # ball slips past and the frozen bottom-wall bounce (a key sample
        # behavior: no ball loss, the bottom wall reflects) is exercised.
        action = -policy if (1500 <= t < 1560 or 6000 <= t < 6060) else policy
        actions.append(action)

        # --- apply_velocity ---
        ball[0] = f32(ball[0] + vel[0] * DT)
        ball[1] = f32(ball[1] + vel[1] * DT)

        # --- move_paddle ---
        paddle_x = f32(paddle_x + f32(action) * PADDLE_SPEED * DT)
        if paddle_x < PADDLE_MIN:
            paddle_x = PADDLE_MIN
        if paddle_x > PADDLE_MAX:
            paddle_x = PADDLE_MAX

        # --- check_for_collisions ---
        n_contacts = 0
        r2 = f32(BALL_R * BALL_R)
        for (cx, cy, hx, hy, kind, idx) in colliders(paddle_x, alive):
            qx = min(max(ball[0], f32(cx - hx)), f32(cx + hx))
            qy = min(max(ball[1], f32(cy - hy)), f32(cy + hy))
            ox = f32(ball[0] - qx)
            oy = f32(ball[1] - qy)
            if f32(ox * ox + oy * oy) > r2:
                continue
            n_contacts += 1
            sounds += 1
            if abs(ox) > abs(oy):
                side = "L" if ox < f32(0.0) else "R"
            else:
                side = "T" if oy > f32(0.0) else "B"
            if kind == "brick":
                alive[idx] = False
                score += 1
            contacts_detail.append({"t": t, "kind": kind, "side": side, "brick": idx})
            if side == "L" and vel[0] > f32(0.0):
                vel = (f32(-vel[0]), vel[1])
            elif side == "R" and vel[0] < f32(0.0):
                vel = (f32(-vel[0]), vel[1])
            elif side == "T" and vel[1] < f32(0.0):
                vel = (vel[0], f32(-vel[1]))
            elif side == "B" and vel[1] > f32(0.0):
                vel = (vel[0], f32(-vel[1]))

        ticks.append({
            "t": t,
            "bx": float(ball[0]), "by": float(ball[1]),
            "px": float(paddle_x),
            "score": score,
            "contacts": n_contacts,
            "sounds": sounds,
        })

    kinds = {}
    for c in contacts_detail:
        kinds[c["kind"]] = kinds.get(c["kind"], 0) + 1
    assert kinds.get("paddle", 0) > 0, "trace must include paddle contacts"
    assert kinds.get("brick", 0) > 0, "trace must include brick contacts"
    assert kinds.get("wall_left", 0) > 0 and kinds.get("wall_right", 0) > 0, "trace must include side wall contacts"
    assert kinds.get("wall_top", 0) > 0, "trace must include top wall contacts"
    assert kinds.get("wall_bottom", 0) > 0, "trace must include BOTTOM wall contacts (bottom-wall bounce is frozen sample behavior)"
    assert score > 0

    doc = {
        "meta": {
            "source": "bevyengine/bevy@efe61cc5b9872f3f9e1ee7553325612c094ccc97 examples/showcase/breakout.rs blob b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954",
            "dt": "f32(1/60)",
            "ticks": TICKS,
            "collider_order": ["wall_left", "wall_right", "wall_bottom", "wall_top", "paddle", "brick:row-major"],
            "action_encoding": {"-1": "ArrowLeft", "0": "none", "1": "ArrowRight"},
            "score_final": score,
            "contacts_by_kind": kinds,
            "position_tolerance": 1e-3,
            "generator": "testdata/breakout/gen_golden_trace.py",
        },
        "actions": actions,
        "ticks": ticks,
        "contacts_detail": contacts_detail,
    }
    out = os.path.join(os.path.dirname(__file__), "golden_trace.json")
    with open(out, "w") as fh:
        json.dump(doc, fh)
    print(f"wrote {out}: {TICKS} ticks, score={score}, contacts={len(contacts_detail)}, kinds={kinds}")


if __name__ == "__main__":
    main()
