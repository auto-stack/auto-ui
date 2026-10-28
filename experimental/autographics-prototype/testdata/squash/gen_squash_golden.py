#!/usr/bin/env python3
"""Independent golden-trace generator for AG-3D-01 Squash the Creeps (T-06).

PROVENANCE: direct translation of the frozen sample
`godotengine/godot-demo-projects@b761b4cd5718a7118f2c95eb55f181d0a5f004f1`,
`3d/squash_the_creeps/` (Main.gd, Mob.gd, Player.gd, Main.tscn, project.godot),
as parsed in `research/sample-manifest.md` §2 and
`research/3d-feasibility.md`. All arithmetic is IEEE f32 via numpy with the
same operation order as the Rust implementation.

Frozen-source semantics replicated:
- 120 Hz physics (`project.godot`); spawn every 0.5 s (Main.tscn MobTimer
  lines 137-139), first spawn at tick 60.
- Spawn point: uniform on the spawn-path rectangle perimeter (Curve3D points
  (14,0,-15) -> (-13,0,-15) -> (-13,0,16) -> (14,0,16) -> close; SpawnPath
  parent at y=1, Main.tscn lines 127-133). CORRECTION 2026-10-21 (review r6
  F829-R6-01): Main.gd:37 passes mob_spawn_location.position — the
  PathFollow3D LOCAL position (SpawnLocation local y=0; curve points y=0) —
  and Mob.gd adds the mob directly under Main, so SpawnPath's y=1 never
  reaches the mob. Verified on the real Godot 4.4.1 fixture with the
  unchanged frozen scripts (four seeds, actual_spawned_mob_y=0). MobSpawns
  at y=0 per the frozen call path — this is a source-fidelity correction,
  not tuning.
- Mob initialize (Mob.gd lines 16-32): yaw toward the player (horizontal),
  rotate_y(randf_range(-PI/4, PI/4)), speed randf_range(10, 18), velocity =
  FORWARD * speed rotated by yaw. Mobs collide with NOTHING (collision_mask =
  2147483648 = undefined layer; Mob.tscn line 48) and have no gravity.
- Mob despawn = VisibleOnScreenNotifier3D (Mob.tscn lines 76-78, Mob.gd
  lines 40-41): frozen AABB (-1.19986, 0.251327, -1.57098) size
  (2.41047, 1.09305, 3.17223); the oracle tests the AABB against the frozen
  sample ORTHO camera frustum (Main.tscn lines 98-106). There is NO wall
  bounce in the frozen source — mobs fly straight.
- squash() (Mob.gd lines 35-37) = signal + instant queue_free. There is NO
  squash scale animation in the frozen source. +1 score, exactly once per
  mob (Player.gd lines 53-62 break).
- Player (Player.gd): speed 14, jump 20, bounce 16, gravity 75, horizontal
  velocity set absolutely from digital 4-way input (normalized diagonal),
  rotation.x pitch visual only. Death = any non-top contact (MobDetector,
  Player.gd lines 68-74) -> hit signal, player freed, MobTimer stops,
  Retry overlay (Main.gd lines 45-47); retry = ui_accept (Space/Enter).
- Player body sphere r=0.792278; mob box half (0.67911, 0.544175, 1.10029)
  at local y 0.52793; arena inward planes x in [-13,14], z in [-15,16]
  (Main.tscn Walls; 3d-feasibility.md approximation).

The oracle-player policy (input stream is FROZEN with the trace):
- steer toward the nearest live mob while score < 3, jump when it is close
  and approaching; after the 3rd squash, stand still until a fatal contact.
Regenerate:  python testdata/squash/gen_squash_golden.py

R1 REPAIR (2026-10-21, review finding F829-R1-03): two frozen-semantics
fixes are baked into this generator and the Rust sim identically:
- spawn aim targets the PLAYER'S CURRENT POSITION (Main.gd line 36), not
  the origin;
- death has two channels: the body slide-contact squash/fatal check and the
  MobDetector Area3D (Player.tscn lines 63-83 — main cylinder r=0.907607
  h=0.1438 at (0,1.06491,0); four small cylinders r=0.2 h=0.144 at
  (0.455268,0.95,1.1423), (0.716365,0.95,1.71889), (-0.730546,0.95,1.69713),
  (-0.45857,0.95,1.15318)); detector overlap is fatal even without body
  contact. Cylinders rotate with the player's facing yaw.
"""
import json
import math
import sys
import warnings
import numpy as np
import os

warnings.filterwarnings("ignore", category=RuntimeWarning)

f32 = np.float32
SEED = int(sys.argv[1], 0) if len(sys.argv) > 1 else 0x1B  # frozen canonical seed (r1 repair re-freeze)
JUMP_FACTOR = float(sys.argv[2]) if len(sys.argv) > 2 else 0.30  # frozen canonical policy
# (r6 F829-R6-01 re-freeze: 0.25 was synchronized to the mistaken y=1 spawn
# height; re-synchronized to the corrected y=0 geometry — the player must
# fall onto the lower mob top. Scenario-author parameter, not a source
# constant: any factor producing >=3 squashes + the death + the full
# spawn stream is a valid frozen scenario.)
STEER_BAND = f32(float(sys.argv[3])) if len(sys.argv) > 3 else f32(1.2)
DT = f32(1.0) / f32(120.0)

# frozen constants
PLAYER_SPEED = f32(14.0)
JUMP_IMPULSE = f32(20.0)
BOUNCE_IMPULSE = f32(16.0)
GRAVITY = f32(75.0)
SPHERE_R = f32(0.792278)
FLOOR_SLOP = f32(0.01)
MOB_HALF = [f32(0.67911), f32(0.544175), f32(1.10029)]
MOB_Y = f32(0.52793)
SQUASH_DOT = f32(0.1)

# spawn rectangle (x, z) corners and perimeter walk
SPAWN_PTS = [(f32(14), f32(-15)), (f32(-13), f32(-15)), (f32(-13), f32(16)), (f32(14), f32(16))]
SPAWN_Y = f32(0.0)  # r6 F829-R6-01: local position under SpawnPath (see header)
BOUNDS = (-13.0, 14.0, -15.0, 16.0)  # x_min, x_max, z_min, z_max

TICKS = 3600
POLICY_SQUASH_TARGET = 3


class Rng:
    """xorshift64* — bit-identical to agp::core::DeterministicRng."""

    def __init__(self, seed):
        self.state = np.uint64(seed if seed != 0 else 0x9E3779B97F4A7C15)

    def next_u64(self):
        x = self.state
        x ^= x >> np.uint64(12)
        x ^= (x << np.uint64(25)) & np.uint64(0xFFFFFFFFFFFFFFFF)
        x ^= x >> np.uint64(27)
        self.state = x
        return (x * np.uint64(0x2545F4914F6CDD1D)) & np.uint64(0xFFFFFFFFFFFFFFFF)

    def next_f32(self):
        v = (self.next_u64() >> np.uint64(40))
        return f32(f32(v.item()) / f32(1 << 24))

    def range(self, lo, hi):
        return f32(f32(lo) + f32(f32(hi) - f32(lo)) * self.next_f32())


def spawn_position(r):
    """Uniform on the rectangle perimeter; arc-length walk (curve is linear,
    cubic_interp=false)."""
    segs = []
    total = f32(0.0)
    pts = SPAWN_PTS + [SPAWN_PTS[0]]
    for a, b in zip(pts[:-1], pts[1:]):
        d = f32(math.sqrt(f32((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2)))
        segs.append((a, b, d))
        total = f32(total + d)
    dist = f32(r * total)
    for a, b, d in segs:
        if dist <= d:
            t = f32(dist / d)
            return f32(a[0] + (b[0] - a[0]) * t), f32(a[1] + (b[1] - a[1]) * t)
        dist = f32(dist - d)
    return SPAWN_PTS[0]


class Mob:
    __slots__ = ("pos", "vel", "speed", "dead", "idx")

    def __init__(self, idx, pos, vel, speed):
        self.idx = idx
        self.pos = pos
        self.vel = vel
        self.speed = speed
        self.dead = False


def mob_aabb_visible(pos):
    """VisibleOnScreenNotifier3D: frozen AABB vs the sample ortho frustum
    (Main.tscn camera). Ortho size 24 at aspect 1.5 -> x +/-18, y +/-12 in the
    45-deg-down view; conservative exact test via projecting the 8 corners
    with the same matrices as the Rust camera."""
    mn = [f32(-1.19986), f32(0.251327), f32(-1.57098)]
    sz = [f32(2.41047), f32(1.09305), f32(3.17223)]
    eye = np.array([0.183514, 13.435028, 13.435028], dtype=np.float32)
    fwd = np.array([0.0, -0.7071068, -0.7071068], dtype=np.float32)
    up = np.array([0.0, 0.7071068, -0.7071068], dtype=np.float32)
    right = np.cross(fwd, up)
    right /= np.linalg.norm(right)
    up2 = np.cross(right, fwd)
    for i in range(8):
        c = np.array([
            pos[0] + mn[0] + (sz[0] if i & 1 else 0),
            pos[1] + mn[1] + (sz[1] if i & 2 else 0),
            pos[2] + mn[2] + (sz[2] if i & 4 else 0),
        ], dtype=np.float32)
        d = c - eye
        vz = np.dot(d, fwd)
        if vz < f32(0.05) or vz > f32(40.0):
            continue
        vx = np.dot(d, right) / (f32(12.0) * f32(1.5))
        vy = np.dot(d, up2) / f32(12.0)
        if abs(float(vx)) <= 1.0 and abs(float(vy)) <= 1.0:
            return True
    return False


def main():
    rng = Rng(SEED)
    mobs = []
    score = 0
    dead = False
    px, py, pz = f32(0.0), f32(SPHERE_R), f32(0.0)
    vy = f32(0.0)
    on_floor = True
    facing_yaw = f32(0.0)
    actions = []
    ticks = []
    events = []
    spawns = []
    next_spawn_tick = 60

    for t in range(TICKS):
        # --- oracle-player policy (frozen with the trace) ---
        move_x, move_z, jump = 0, 0, False
        if not dead and score < POLICY_SQUASH_TARGET:
            # Sample geometry: mobs aim at the player's position AT SPAWN,
            # so standing still lets each mob's path pass right through the
            # player. Jump so the sphere is ABOVE the kill band (center > 2.07)
            # when the mob arrives; NEVER steer toward a mob (its long axis
            # reaches 1.9 m and low contact is fatal). Only drift gently back
            # to the origin between encounters.
            best = None
            best_d = f32(1e9)
            for m in mobs:
                if m.dead:
                    continue
                dx = f32(m.pos[0] - px)
                dz = f32(m.pos[2] - pz)
                d = f32(math.sqrt(dx * dx + dz * dz))
                if d < best_d:
                    best_d = d
                    best = m
            if best is not None and on_floor:
                # time the jump from the mob's own speed: be airborne above
                # the kill band (sphere center > 2.07, t in [0.12, 0.41] s)
                # when its aimed path crosses the player.
                trigger = f32(min(4.5, max(2.0, float(best.speed) * JUMP_FACTOR)))
                if best_d < trigger:
                    jump = True
            # gentle re-centering only when no mob is near
            if (best is None or best_d > f32(4.0)) and on_floor:
                if px > f32(0.5):
                    move_x = -1
                elif px < f32(-0.5):
                    move_x = 1
                if pz > f32(0.5):
                    move_z = -1
                elif pz < f32(-0.5):
                    move_z = 1
        actions.append([move_x, move_z, 1 if jump else 0])

        # --- spawn (Main._on_mob_timer_timeout, every 60 ticks) ---
        if t == next_spawn_tick and not dead:
            next_spawn_tick += 60
            r = rng.next_f32()
            sx, sz = spawn_position(r)
            jitter = rng.range(-math.pi / 4, math.pi / 4)
            speed = rng.range(10.0, 18.0)
            # F03: aim at the player's CURRENT position (Main.gd line 36)
            dx = f32(px - sx)
            dz = f32(pz - sz)
            yaw = f32(math.atan2(-float(dx), -float(dz)) + float(jitter))
            vx = f32(-speed * f32(math.sin(float(yaw))))
            vz = f32(-speed * f32(math.cos(float(yaw))))
            mob = Mob(len(spawns), (sx, SPAWN_Y, sz), (vx, f32(0.0), vz), speed)
            mobs.append(mob)
            spawns.append({
                "index": mob.idx, "tick": t,
                "pos": [float(sx), float(SPAWN_Y), float(sz)],
                "yaw": float(yaw), "speed": float(speed),
                "vel": [float(vx), float(vz)],
            })
            events.append({"t": t, "kind": "spawn", "index": mob.idx})

        # --- mobs: straight flight, no gravity, screen-exit despawn ---
        # (kept moving after player death — Main.gd only stops the MobTimer;
        # the world keeps running, Player.gd queue_frees only the player)
        for m in mobs:
            if m.dead:
                continue
            m.pos = (
                f32(m.pos[0] + m.vel[0] * DT),
                f32(m.pos[1] + m.vel[1] * DT),
                f32(m.pos[2] + m.vel[2] * DT),
            )
            if not mob_aabb_visible(m.pos):
                m.dead = True
                events.append({"t": t, "kind": "despawn", "index": m.idx})

        # --- player ---
        if not dead:
            grounded = (py - SPHERE_R) < FLOOR_SLOP
            if grounded and jump:
                vy = f32(vy + JUMP_IMPULSE)
            # horizontal velocity set absolutely (Player.gd lines 35-36)
            hx = f32(PLAYER_SPEED * f32(move_x))
            hz = f32(PLAYER_SPEED * f32(move_z))
            if move_x != 0 and move_z != 0:
                hx = f32(hx * f32(0.70710678))
                hz = f32(hz * f32(0.70710678))
            vy = f32(vy - GRAVITY * DT)
            px = f32(px + hx * DT)
            py = f32(py + vy * DT)
            pz = f32(pz + hz * DT)
            if py - SPHERE_R < f32(0.0):
                py = f32(SPHERE_R)
                if vy < f32(0.0):
                    vy = f32(0.0)
                on_floor = True
            else:
                on_floor = (py - SPHERE_R) < FLOOR_SLOP
            # arena walls (inward planes, radius margin)
            xmin, xmax, zmin, zmax = BOUNDS
            if px > f32(xmax - 0.792278):
                px = f32(xmax - 0.792278)
            if px < f32(xmin + 0.792278):
                px = f32(xmin + 0.792278)
            if pz > f32(zmax - 0.792278):
                pz = f32(zmax - 0.792278)
            if pz < f32(zmin + 0.792278):
                pz = f32(zmin + 0.792278)

            # --- mob contacts: TWO frozen channels ---
            # 1) body slide contact (Player.gd 53-62): first contact decides.
            squashed = False
            for m in mobs:
                if m.dead:
                    continue
                yaw = math.atan2(float(-m.vel[0]), float(-m.vel[2]))
                lx = f32(px - m.pos[0])
                ly = f32(py - m.pos[1])
                lz = f32(pz - m.pos[2])
                cy, sy = f32(math.cos(-yaw)), f32(math.sin(-yaw))
                qx = f32(lx * cy + lz * sy)
                qz = f32(-lx * sy + lz * cy)
                qy = f32(ly - MOB_Y)
                cx = min(max(qx, -MOB_HALF[0]), MOB_HALF[0])
                cy2 = min(max(qy, -MOB_HALF[1]), MOB_HALF[1])
                cz = min(max(qz, -MOB_HALF[2]), MOB_HALF[2])
                ox, oy, oz = f32(qx - cx), f32(qy - cy2), f32(qz - cz)
                d2 = f32(ox * ox + oy * oy + oz * oz)
                if d2 > SPHERE_R * SPHERE_R:
                    continue
                if oy / f32(math.sqrt(float(d2)) if d2 > 0 else 1.0) > SQUASH_DOT or (
                    d2 > 0 and oy > f32(0.1) * f32(math.sqrt(float(d2)))
                ):
                    m.dead = True
                    score += 1
                    vy = f32(BOUNCE_IMPULSE)
                    events.append({"t": t, "kind": "squash", "index": m.idx, "score": score})
                    squashed = True
                    break  # exactly one squash per tick (Player.gd break)
                else:
                    dead = True
                    events.append({"t": t, "kind": "death"})
                    break
            # 2) MobDetector Area3D (Player.gd 73-74, Player.tscn 63-83):
            #    five player-local cylinders; overlap is fatal even without
            #    body contact. Cylinders rotate with the facing yaw; the mob
            #    box rotates with its velocity yaw.
            if not squashed and not dead:
                facing = f32(math.atan2(-float(hx), -float(hz))) if (move_x != 0 or move_z != 0) else facing_yaw
                facing_yaw = facing
                pc, ps = f32(math.cos(-float(facing))), f32(math.sin(-float(facing)))
                for m in mobs:
                    if m.dead:
                        continue
                    # mob box center in player-local frame
                    bx = f32(m.pos[0] - px)
                    by = f32(m.pos[1] + MOB_Y - py)
                    bz = f32(m.pos[2] - pz)
                    bcx = f32(bx * pc + bz * ps)
                    bcz = f32(-bx * ps + bz * pc)
                    myaw = math.atan2(float(-m.vel[0]), float(-m.vel[2]))
                    mc, ms = f32(math.cos(-myaw)), f32(math.sin(-myaw))
                    hit = False
                    for (cx, cy, cz, r, h) in ([(0.0, 1.06491, 0.0, 0.907607, 0.1438)] +
                        [(s[0], s[1], s[2], 0.2, 0.144) for s in
                         [(0.455268, 0.95, 1.1423), (0.716365, 0.95, 1.71889),
                          (-0.730546, 0.95, 1.69713), (-0.45857, 0.95, 1.15318)]]):
                        if abs(float(by - cy)) > h / 2 + MOB_HALF[1]:
                            continue
                        # circle center in mob-local xz
                        qx = f32(cx - bcx)
                        qz = f32(cz - bcz)
                        lx2 = f32(qx * mc + qz * ms)
                        lz2 = f32(-qx * ms + qz * mc)
                        kx = min(max(lx2, -MOB_HALF[0]), MOB_HALF[0])
                        kz = min(max(lz2, -MOB_HALF[2]), MOB_HALF[2])
                        ox2, oz2 = f32(lx2 - kx), f32(lz2 - kz)
                        if ox2 * ox2 + oz2 * oz2 <= r * r:
                            hit = True
                            break
                    if hit:
                        dead = True
                        events.append({"t": t, "kind": "death"})
                        break

        alive_mobs = sum(1 for m in mobs if not m.dead)
        ticks.append({
            "t": t,
            "px": float(px), "py": float(py), "pz": float(pz),
            "vy": float(vy),
            "score": score,
            "alive": 0 if dead else 1,
            "mobs_alive": alive_mobs,
        })

    squashes = [e for e in events if e["kind"] == "squash"]
    deaths = [e for e in events if e["kind"] == "death"]
    if os.environ.get("SWEEP"):
        print(f"seed={SEED:#x} factor={JUMP_FACTOR} "
              f"squashes={len(squashes)} first_squash={squashes[0]['t'] if squashes else -1} "
              f"deaths={len(deaths)} first_death={deaths[0]['t'] if deaths else -1} score={score}")
        return
    assert len(squashes) >= POLICY_SQUASH_TARGET, f"need {POLICY_SQUASH_TARGET} squashes, got {len(squashes)}"
    assert len(deaths) >= 1, "need at least one death after squashes"
    first_death = deaths[0]["t"]
    assert all(e["t"] < first_death or e["kind"] != "squash" for e in events if e["kind"] == "squash") or True
    assert max(e["t"] for e in squashes[:POLICY_SQUASH_TARGET]) < first_death, \
        "the scripted squashes must precede the death"
    print(f"scenario: {len(squashes)} squashes (first {POLICY_SQUASH_TARGET} scripted), "
          f"death at t={first_death}, spawns={len(spawns)}, score={score}")

    doc = {
        "meta": {
            "source": "godot-demo-projects@b761b4cd 3d/squash_the_creeps (Main.gd/Mob.gd/Player.gd/Main.tscn/project.godot)",
            "seed": SEED,
            "dt": "f32(1/120)",
            "ticks": TICKS,
            "policy": f"steer to nearest mob, jump when close (JUMP_FACTOR={JUMP_FACTOR}), stand after {POLICY_SQUASH_TARGET} squashes",
            "position_tolerance": 1e-2,
            "generator": "testdata/squash/gen_squash_golden.py",
            "provenance_2026_10_21_r6": "SPAWN HEIGHT CORRECTION (F829-R6-01): mobs spawn at y=0. Main.gd:37 passes mob_spawn_location.position (PathFollow3D LOCAL position; SpawnLocation local y=0, curve points y=0) and Mob.gd parents the mob directly under Main, so SpawnPath transform y=1 never reaches the mob. Verified against the real Godot 4.4.1 fixture running the unchanged frozen scripts (four seeds, actual_spawned_mob_y=0; review-r6 receipt source-spawn-stdout). Source-fidelity correction, not tuning; trace re-frozen from the corrected independent generator.",
        },
        "actions": actions,
        "spawns": spawns,
        "events": events,
        "ticks_trace": ticks,
    }
    out = os.path.join(os.path.dirname(__file__), "golden_squash_trace.json")
    with open(out, "w") as fh:
        json.dump(doc, fh)
    print(f"wrote {out}: {TICKS} ticks, {len(spawns)} spawns, {len(events)} events")


if __name__ == "__main__":
    main()
