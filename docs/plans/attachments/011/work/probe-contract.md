# UI-011 r1 execution contract (2026-10-30)

Fixed engine: 0261ae9dacf786beb6dceb630be6af267f242625, copied to a new local consumer preparation directory. Identity verifier: 1431 files; SDK snapshot 681 files/17 local packages; metadata 479 packages/29 path edges. Fixed assets/G09 reference: ag-examples d1b4f9baf6f83575af4c669efa9b17f507d9003d. No engine patches.

## Producer, ownership and candidates
Storage::load_project → SourceSnapshot::from_bytes_for(Api::Sprites2V1) → Host::start_user_snapshot → Play/Session callback → Host::frame prepare/accept/publish → Frame and GPU readback. Auto alone owns velocity, patrol, collection, win/fail, frame selection, intents and restart. Rust sends predetermined action bits and inspects output. Templates remain outside the camera; instances alone represent coins/enemy.

A writes the current sprite each update. B caches only the last sprite rectangle and facing, writing only when changed; all inputs, movement, patrol and event draining remain at the same frequency. No third core candidate. Public ag_world reads are existing observations available to Auto; no private engine symbols are called by gameplay. State and event sequence must match between A/B whenever both accept. Core route has the original geometry, speed4, jump8, gravity20, dt1/60, clearance.002.

## Cases and bounded input
Each core candidate receives the identical fixed input schedule (ticks 1..2400 maximum): idle 1..20, right21..400, with jump at21/100/140/200/260/320/380; release401..420, left421..460 with jump430/440, neutral461..480, Enter481. Stop on first Fault; subsequent required states become not_established_budget_fault, never semantic pass. This route is an attempted route, not a prior reachability proof.

Additional controlled scenes preserve base geometry except named subject/coin/enemy starting positions: grounded at(2,.502), rise/fall and left/right, coin single at subject, three distinct coins together, enemy/goal/coin adjacent, uncollected-goal, pit, wall, ceiling, edge. These are branch isolation, never substitutes for base-route success. If full callback faults, isolate character/presentation and event/patrol callbacks explicitly by removing only named cost centres; reports label the changed source hashes and do not count them toward core semantics. Events are N geometry → N+1 callback, with enter only collection and real despawn. Multiple events must drain in the same callback; a missing last coin or frozen patrol is a failing control.

Budgets: 32credits/callback; 1000steps/slice,100000steps/callback,120resumes/250ms; user256KiB,managed96KiB,combined352KiB; init30s/tick2s; 8ms pump target separately classified. Preserve 004 node/slot/pair/image resource limits. Source and threshold experiments cross-check 26 normal move + input3 + grounded1 + sprite1 + patrol1 + absent-next1 =33 for A; B is32 only on unchanged sprite ticks. Event next/pair/phase/despawn each1. Draining a final absent-next costs1.

Observations: init and each dispatched callback record candidate/case/run/tick, action bits, result, source hash, actor/camera/sprite/live slots, full Host pumping time and separate readback. No public VM step/resume meter exists on Play: SDK limits stay fixed; step/resume consumption remains unmeasured unless a public observation is found, never inferred from elapsed CPU. Record this limitation explicitly. Credit counts are calibrated costs and thresholds, not a direct private-meter read. Host frame pumping includes VM + preparation + draw submission; readback/assertions measured separately. Opt-in public Play VM/admission measurement is recorded independently if supported by fixed source.

GPU: key accepted frames captured plus source-sheet five nontransparent sample pixels projected through the accepted runtime camera, RGB tolerance8 (existing sRGB blend), alpha255; compare correct frame vs wrong-frame/flip counterexample. All failed transactions compare entire Frame debug representation and capture bytes to last-good, not tick only. Stop restores editor, new Run tick0/coins4/initial frames, previous handles rejected in a separately labelled public stale-handle control; weak VM owner after Stop and after external pin release. Warm static image reuse binds unchanged 004 evidence plus fresh chain sampling.

Performance: exactly3 release repetitions of core failing prefix/representative legal isolated branch and1 debug; do not fabricate successful-route metrics.20 short Run/Stop cycles, sequential GPU only; fixed seeds/no random searching. Fault restores exact original source before legal control replay.

## Decision and report gate
Required rows G2-01..09, cases S-01..05, both candidates, original route, single/triple/danger peaks, legitimate32 and rejected33 controls, faulty output/last-good, Pause/neutral Step/Stop/fresh Run/old generation, CPU/source/resources/lifecycle observations. Each row includes execution status or explicit budget-blocked status with earliest trace. No zero-selected success. Decision fields: core_semantics,hard_budgets,target_8ms,g2_09_public_services,full_m2_proven. Any required core row unestablished or failed prohibits go_core. G2-09=missing_public_capability and full_m2_proven=false unconditionally for this research. Mutations: missing denominator, partial report, false go, false full-M2, wrong source pin, stale input, dropped third event and wrong sprite must be rejected. Negative result is bounded to these two candidates/current profile, not a language impossibility proof.

Entry points: cargo test --manifest-path research/autographics-g2-budget/Cargo.toml --locked --offline --test full_loop -- --test-threads=1; python research/autographics-g2-budget/tools/verify.py --controls. CARGO_TARGET_DIR and G2_ARTIFACTS must point to this Plan independent local artifact tree. Runtime bounds, missing rows or unavailable required oracles leave tasks open, never a manufactured pass.


## Fixture/oracle reconciliation during T-03/T-04

Pixel ROIs use the inherited G09 sRGB alpha composite against an actual background pixel; transparent ROI texels are compared rather than incorrectly asserted opaque. All required frame samples still use tolerance8 and wrong-frame rejection. Isolated scenes append a hidden diagnostic leaf id12; the isolated Auto callback writes bag/state to this leaf through existing ag_set_local, and Rust reads the admitted node translation. This is read-only oracle transport, absent from both core candidate files and not HUD rendering or gameplay logic in Rust. Original base has11 nodes, isolated scenes12; original geometry and collision roles remain unchanged. Both candidates give enemy failure precedence over a same-callback goal event (goal only wins while state0).

Deterministic input transactions request Pause to cap one tick. Restart is a separate Running Enter transaction: the frozen Host intentionally never consumes a restart marker from Paused/neutral Step. The adapter observes a new Run, then requests Pause and finishes initialization. This preserves the existing public restart semantics. Source transformations normalize CRLF before exact replacement and assert their anchor; a prior failed diagnostic caught an ineffective CRLF replacement. Triple fixture mutations use complete spawn-call anchors so coin3 at24 cannot be accidentally changed to228 by a substring replacement; the live-slot oracle caught that setup error before acceptance. These were harness construction failures, not engine defects or final review cycles.


## Open acceptance gap, not waived (2026-10-30)

Exact step/resume consumption is not exposed by fixed Play. The observation paragraph above records this fact but does not override Plan §5.4/AC-04. Existing SDK enforcement only supplies bounds. Explicit user acceptance of unmeasured fields is pending; until then T-04/AC-04 and ready-for-review remain incomplete. This decision does not alter32credits or any game semantics. Full result and immutable raw evidence bindings are in [results](results.md).


## r2 accepted observability decision (2026-10-30)

User explicitly accepted the unmeasured exact steps/resumes annotation and continuation through review/merge, then sent `/goal resume`. The previous pending decision is resolved. SDK limits stay fixed; runtime source, fixtures, lock, consumer, resources and measurement inputs have not changed. Historical gaps/blocked statements above describe r1 only. r2 permits this explicitly labelled observation boundary; it never converts unmeasured maxima into a measured pass or complete-game success.
