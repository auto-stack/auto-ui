# G2 full-loop budget research (auto-ui PLAN-011 r2)

This is a replayable negative feasibility study, not a game release. The original
G2 geometry and complete public Auto controller run through the pinned source
loader, Session, Host admission/publication and offscreen GPU. Rust sends fixed
input and observes output; it does not run movement, collection or win logic.

The tested A/B candidates both reject the first original-route callback with
QueryBudget and preserve the entire last-good Frame and GPU image. A writes the
sprite every tick; B writes only when frame/facing changes. Cached grounded idle
can fit32, but a required frame transition costs33. This study does not prove
that every possible Auto program is impossible. It does not establish the
original route's reachability, G2-01..08 as a complete game, or full M2.

G2-09 remains missing_public_capability: jump/coin/HUD intent variables in Auto
are not playback or UI. Public sound/HUD implementation, final art and release
remain outside this Plan.

## Reproduce

The source-only research is an independent non-member Cargo workspace. Its
committed manifest points to this Plan's default external consumer directory;
no root/experimental workspace, lock, source, target or SDK is edited. Its lock
inherits every registry package identity from the frozen consumer, adding only
png0.17.16 for research readback. No junctions or symlinks are used.

Prepare a **new** consumer from the exact Git closure:

```powershell
python research/autographics-g2-budget/tools/prepare.py --repository D:/autostack/auto-ui --out D:/autostack/plan-artifacts/auto-ui/plan-011/consumer
$env:CARGO_TARGET_DIR='D:/autostack/plan-artifacts/auto-ui/plan-011/target'
$env:G2_ARTIFACTS='D:/autostack/plan-artifacts/auto-ui/plan-011/debug'
$env:RUST_MIN_STACK='16777216'
cargo test --manifest-path research/autographics-g2-budget/Cargo.toml --locked --offline --test full_loop -- --test-threads=1
```

The fixed closure is0261ae9dacf786beb6dceb630be6af267f242625, including PLAN-004
and PLAN-008. The SDK is7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. Source
identity covers1431 files; SDK681 files/17 local packages; dependency closure
479 packages/29 path edges. `prepare.py` verifies these before returning.
It refuses an existing output directory. An already prepared directory can be
reverified with its `tools/verify_source.py` (without CARGO_TARGET_DIR override).

For another consumer location use a generated replay workspace; source/fixtures
and the research lock are copied byte for byte, with only the dependency path
changed in the generated Cargo.toml:

```powershell
python research/autographics-g2-budget/tools/replay.py --consumer-root D:/autostack/plan-artifacts/auto-ui/plan-011/fresh-consumer/experimental --out D:/autostack/plan-artifacts/auto-ui/plan-011/fresh-replay --target D:/autostack/plan-artifacts/auto-ui/plan-011/target
```

Run GPU tests sequentially without windows or OS input automation. Four tests
are selected by the complete command: `full_loop_research` (A/B original route),
`scoped_branches_and_controls` (50 case rows,20 short Run/Stop cycles) and
`performance_prefix` (two original failing callbacks and60 legal isolated
character ticks) and `accepted_intent_observations` (13 accepted intent rows,
including duplicate/dropped-counter controls). Individual case failure makes Cargo fail. The performance
campaign is1 debug plus3 release windows: run the complete release suite once,
then select only `performance_prefix` for repetitions2/3 with distinct
G2_ARTIFACTS release-2/release-3. Fresh replay is an independent correctness
replay, not a fourth release performance sample.

Build the decision after all windows and fresh replay exist:

```powershell
python research/autographics-g2-budget/tools/report.py --artifacts D:/autostack/plan-artifacts/auto-ui/plan-011
python research/autographics-g2-budget/tools/verify.py --controls
```

Raw JSON, source snapshots, PNG, logs and consumer directories stay outside Git
under this Plan's local artifact directory. The validator binds their hashes,
requires complete case/candidate/denominator sets and fresh outcome agreement,
and rejects false go/full-M2, stale sources and deliberately partial reports.

## What each observation establishes

The normal movement price is26:1 precharge +24 v5 +1 final support. Grounded
read, each action read, sprite write, patrol move and absent event_next each
cost1. Direct A's ordinary callback needs33; B can need32 only on unchanged
frames. Initial-overlap/early-return movement may cost25, so some controlled
peak prefixes accept before event reading subsequently faults. No average is
used as a substitute for the worst callback. Three-event/danger isolation is
labelled explicitly and never counted toward a successful complete candidate.

Source size, public resource snapshots, per-tick Host wall time, opt-in VM/output,
admission/prepare windows and synchronous readback are separate observations.
The fixed public Play exposes no exact per-callback VM step/resume counter:
these fields remain explicitly unmeasured, with the unchanged SDK1000-step
slice/100000 cumulative/120-resume/250ms enforcement recorded as bounds, not
measured maxima. The 30s init/2s tick limits stay in force. Source admission
keeps256KiB user/96KiB managed/352KiB combined; a stale diagnostic string saying
320KiB in the frozen source does not change those constants.

Release CPU values describe the measured failing prefixes and isolated legal
branch only. Readback/assertions are reported separately; GPU timestamps are
unavailable. Host preparation/draw submission remain inside whole-tick timing.
Background desktop load is uncontrolled; no full-route or production performance
claim follows from a measured prefix below8ms.

The original scene has11 nodes,4 live slots,5 trigger pairs and one735-byte CC0
PNG (160x32,20480 decoded/GPU bytes). Isolated scenes add one hidden diagnostic
leaf. Only the isolated Auto source writes bag/state there with public local
setters; Rust reads the admitted node. It is oracle transport, absent from core
candidates, and does not implement HUD. Sprite samples compare actual GPU pixels
against sRGB-composited source texels with tolerance8; a wrong frame is rejected.
Faults retain the entire frame/image; Stop comparisons run the next Host draw
before readback. Lifecycle checks distinguish live VM owners, legitimate editor
and external image pins, and final owner release. Unchanged PLAN-004 sprite
resource evidence supplies the lower-level warm texture-upload guarantee;
this research adds fresh no-redecode/owner and full-chain pixel samples.

All required G2-01..09 rows remain in the decision. Original core states after
tick1 are not_established_budget_fault, even when an isolated control succeeds.
Canonical knowledge changes are proposed in spec-delta.md and published only
by merge after acceptance review.


PLAN-011 r2 user ruling (2026-10-30): exact per-callback step/resume counters
are accepted as unmeasured for this fixed consumer, with unchanged SDK limits
and all other real evidence retained. This does not claim measured consumption
or a margin below those limits and does not change game budgets or outcomes.


R2 final-review closure: callback-ledger.json binds each original/isolated/CPU
callback record to a source-derived public-call account, calibrated prices and
32/33 controls. Conditional/early-return demands are explicit ranges; rejected
callbacks report a refused33-credit boundary and budget-blocked suffix, never
newly published diagnostics or a private meter reading. Paused/Stop/last-owner
operations identify that no Auto callback ran. Initialization is separate.
`intents.json` observes existing Auto counters on accepted diagnostic isolates
via far-away hidden leaves; no core A/B file or original performance input is
changed. Triple collection produces3 coin intents and4 HUD refresh requests
(three coins+goal). These are event-driven requests, not rendered UI. They do
not establish G2-09. Air rejump leaves jump intent1; duplicate/dropped coin
intent source mutations produce6/0 instead of3 and are rejected, then exact
source restoration returns3. Original/fresh targeted replay uses
`--test-filter accepted_intent_observations` without repeating other families.
