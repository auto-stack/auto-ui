# Auto source → AutoGraphics (PLAN-833)

Experimental Windows x64, one native window, keyboard/mouse. Real `.at` is compiled once per instance by pinned `auto-lang-core`, driven through public CPU slices, and decoded into renderer-neutral data. No Rust game driver advances Auto mode.

## Prepare and run

From this directory (PowerShell):

```powershell
python tools/prepare_sdk.py --repository D:/autostack/auto-lang
python tools/verify_source.py
cargo run --locked -- --sample breakout
cargo run --locked -- --sample scene
cargo run --locked -- --source path/to/program.at
```

An existing `.sdk-source` is verified with `--verify-only`; preparation refuses to overwrite it. It is generated from Git objects, ignored, and never edited. `research/source-lock.json` records every input hash. Frozen SDK root manifest/lock bytes are separate from the driver consumption lock. No UI/Man/old facade lies in the SDK closure; AGP legitimately depends on its experimental Iced host. The unused production iced_widget patch is excluded from the SDK projection. All original AGP registry package identities/checksums remain in this driver lock.

Top two portals share script instance 0; bottom-left is independent instance 1. The bottom-middle slot displays existing 3D cylinder geometry for the scene sample. Click a game to focus it; left/right arrows move Breakout's paddle; R resets both instances. Real panel text holds keyboard focus. Pause/Reset are existing stock Iced controls. Breakout's bottom wall bounces; it has no lives/game-over. The score uses the original FiraMono-Medium asset and 33px text renderer; collision requests use the original OGG and Kira backend. GPU/WGSL and resource ownership stay in Rust.

## Tests and reproducible sources

```powershell
cargo fmt --check
cargo check --locked --all-targets
cargo test --locked --test sdk_probe --test numeric_probe --test output_admission --test scheduler_lifecycle
cargo test --locked --test breakout_replay -- --nocapture
cargo test --locked --test graphics_loop -- --nocapture
cargo test --locked --lib terminal_policy_tests
cargo run --locked -- --window-probe --report research/window-positive
cargo run --locked -- --window-probe --sabotage-focus --report research/window-focus-negative
python tools/package_consumer.py C:/path/to/new-package
```

The focus sabotage command MUST fail. The graphics target executes actual GPU pixel/3D hash assertions and three real fault controls (drop output, wrong routed local, duplicate delivery), plus source-color change, single/shared views, reset/close, owner-witness sensitivity and warm pipeline counts. GPU failure is a failure, never a skip. `breakout_replay` consumes the unchanged 10,800-tick golden (positions 1e-3, exact 210 contact details, score/sound/destruction/output events); speed and collision mutations must fail that same replay.

A packaged B contains sources and relative path dependencies, including generated pinned SDK input, AGP assets, and locks; no target/rlib. Run the same verify/check/scoped tests inside `B/experimental/autographics-auto-driver`. The source preparation utility still requires the frozen Git object database; a prepared external package does not. Work evidence and original failed attempts are under main-checkout `docs/plans/attachments/833/work-r2/`; review/merge retain this authoritative worktree.

## Profile and limits

Protocol 1 uses root-function exports: `init`, `update(int)`, `pointer(float,float)`, `key(int)`, `destroy`; `protocol`, `background(int)`, `object_count`, `changed(int)`, `object_alive(int)`, `generation(int)`, `shape(int)`, `field(int,int)`, `integer(int)` and `event_count/event_kind/event_a/event_b`. Scalars are checked against the SDK's parsed signatures and actual returned NanoValue tags. Float means f32. Unit callbacks are real code execution, not the high-level interpreter Nil stub.

Up to 128 slots, 256 events per output, 64 queued actions, 64 completed outputs; values finite and bounded, positive scales, RGBA 0..1. Shape 0=circle, 1=quad, 2=existing cylinder. `field` is translation xyz, scale xyz, RGBA. `changed=0` retains the last accepted slot; a producer must mark creation/change/removal. Transient events are regenerated for each callback (key/pointer callbacks clear old collision buffers); contacts/sounds must pair and destroyed objects must have a known prior identity. A complete validated snapshot is published only after all changed slots and events are decoded. Slot tombstones/generations plus session/sequence reject stale identities; a reset creates a new session/VM. No GPU/resource pointer crosses the boundary. Only fixed built-in shape/font/sound resources are exposed.

Every public callback start resets the task's cumulative CPU counter, never a resume. Each slice has 1,000 steps / 2ms / clock check every 64; one event-loop pump resumes slices until its 8ms round cap. Callback limit 100,000 instructions, 120 resumptions or 250ms elapsed; budget errors close the VM/task and retain host last-good output. Parked outcomes terminate as unsupported-wait (the policy branch has a synthetic outcome test; asynchronous script APIs are outside this profile). Queues and event history are bounded; history retains the latest 4,096 events. Each fixed 1/60 tick is accepted once per instance independently of mount/render counts. All completed output packets are consumed, so a multi-tick frame does not lose events.

Source is trusted synchronous computation. This is not a native/FFI/IO security sandbox and cannot preempt blocking native calls. Compilation itself is synchronous. No hot reload, production UI/RQHost, Forge/distribution, AutoShader/CAP, new ACE integration, or Auto Squash delivery is claimed. I1's original two-game denominator remains pending. Final independent review and frozen merge-owned full driver/AGP gates are still required.

R1 repair: `graphics_loop` now includes six tests. Ownership checks cover every quad/text resource at admission and before frame work; mixed devices never mask each other. Text upload counters measure actual camera (64B) and nonempty glyph (48B each) buffer writes, included once per rendered content even with shared mounts. Five glyphs add exactly 304B / two rewrites. Current replay, omission control and measurements: main Plan attachments/833/work-r3; prior upload totals are invalidated. Full driver/AGP gates remain merge-owned after independent review.


## Standalone Windows native build profile (T-10)

The frozen Windows profile reserves32MB for the main-thread stack via the repository's Cargo config. A source export outside that repository does not inherit it. After the locked source/tests checks, build the native entry explicitly:

```powershell
cargo rustc --locked --offline --bin autographics-auto-driver -- -C link-arg=/STACK:33554432
./target/debug/autographics-auto-driver.exe --sample scene --frames 30 --report standalone-report
```

T-10 verified the PE reserve equals33554432 and native replay passes with both producer SDK/AGP sources unavailable. The initial external default-stack native run overflowed; this corrected command preserves the already reviewed build profile, without source/SDK changes. See archived PLAN-833 merge evidence. Production migration remains pending.


## PLAN-834 profile 2 (work candidate; review/merge pending)

`cargo run --locked -- --sample squash` selects the explicit Auto Squash profile; it does not infer a profile from object shape. The unchanged Breakout/scene entrypoints remain profile 1. Squash uses the original player/mob GLBs (all three meshes each), ground/cylinders, orthographic camera, light/shadow, Montserrat 32px Score/Retry and the original looping OGG. Two upper mounts share instance 0; the lower left is independent instance 1. Click a game, then use arrows/WASD and a Space press to jump. When dead, Space/Enter retries that instance with a new session. Text focus excludes game input; blur clears held keys/edges. Retry preserves the single window-host music owner; the final owner releases its real Kira manager/sound.

The source is `samples/squash/{math,main}.at`, compiled through the same `Session`. `SquashScript` schedules explicit scalar callbacks, random requests, eight-slot motion batches and checked output queries. `Packet3d`/`Publication3d` are separate from profile 1, with typed resource roles, slot generation, session/sequence, yaw/pitch/positive scale and transient spawn/squash/despawn/death events. Auto owns all gameplay and animation values. Fixed model roles map to this host's immutable content-hashed resources; no arbitrary paths/pointers are accepted. Per-resource GPU ownership is checked at admission/frame work, including mesh stores, HUD and overlay pipelines.

Scoped acceptance commands:

```powershell
cargo test --locked --test squash_feasibility --test squash_supplements --test squash_admission -- --nocapture
cargo test --locked --test squash_numeric_replay --test squash_profile -- --nocapture
cargo test --locked --test squash_graphics -- --nocapture
cargo run --locked -- --sample squash --window-probe --report research/squash-native
cargo run --locked -- --sample squash --window-probe --sabotage-focus --report research/squash-native-negative
```

The focus sabotage must fail the same formal window oracle. See `research/squash-driver-feasibility.md` for actual numeric failures, selected service, finite bounds and evidence limits. Full driver/AGP closeout remains merge-owned after independent review; this section is not a delivery or I1 completion claim.

Standalone scoped tests also retain the repository's existing worker-stack profile: `$env:RUST_MIN_STACK="16777216"`. This is separate from the32MB native PE stack reserve. `verify_plan834.py` binds authored source/test/data/assets to the code receipt; known text suffixes normalize CRLF to LF so a valid Windows Git checkout and an exported package have the same source identity. Binary GLB/OGG/font assets are hashed byte-for-byte; SDK source verification keeps the inherited exact-byte policy.

## PLAN-835 scene authoring (experimental)

Dedicated entry: `cargo run --locked --bin autographics-authoring`. The original default entry remains `autographics-auto-driver` (profile 1/2 unchanged). Windows x64, one stock-Iced window, keyboard/mouse, bounded static 2D/3D scene; production AutoUI/RQHost/Forge/I2/I3 remain pending.

Select an object by its stable ID in the list. Edit Name, translation, positive scale, yaw/pitch (3D), RGBA (2D), or visible; press Apply once. Invalid drafts do not change the document/history/preview. Cancel draft or focus loss restores accepted values. Scroll the panel for lower fields. Add any of the five fixed object kinds, delete, undo/redo. Yellow bounds identify the selected visible object; selection is not saved. Save uses the current file, Save as uses the typed explicit path. Open/New/Close with unsaved content offers Save/Discard/Cancel; a failed save or unverified result cannot discard it.

The current accepted document is UTF-8 `.agscene`, schema `autographics.scene/2` (PLAN-837 below); the original v1 format remains a strict import path. Stable IDs/name/kind/transform/visibility and the fixed resource manifest persist. Selection/focus, history, revisions, VM/GPU handles and game state do not. See `docs/specs/auto-graphics/{scene-document,authoring-session}.md` in the Plan worktree for the proposed contract. Maximum128 total groups and objects,1MiB file,128 Unicode name characters,64 history steps/16MiB; finite numeric magnitude≤100000, positive scales, RGBA0..1. Unsupported schema/fields/resources/types/duplicate keys or IDs are rejected, without truncation. Fixed 3D material is unchanged; 2D has z0/scale-z1/no rotation.

Historical profile3 `samples/authoring/main.at` (retained unchanged; current new/import editing explicitly uses profile4 below) receives the full checked configuration, via a fresh private `Session` for each candidate, then produces every admitted transform/color/visibility along the public scalar bridge. A bounded8ms pump stages output. Compilation is synchronous and measured separately (same inherited SDK contract); the CPU-slice budget cannot preempt native/IO/compilation. GPU preparation succeeds before document/preview/history publish; a failed candidate closes privately and leaves last-good. Draft and command messages bind epoch/revision/stable target. Reopen establishes a fresh epoch and empty history; undo/redo increment revision. Dirty compares canonical content hashes with the last confirmed saved hash.

Save owns an exclusive create_new sidecar (never degrades), a same-directory create_new staging file, complete bytes/flush/readback, last expected-original-byte validation, then rename as commit. Before-commit failures preserve old bytes. Post-commit verification failure is explicitly pending, keeps dirty and can be confirmed/retried by reading this snapshot; no fake rollback. Foreign lock/temp files are preserved. Parent directories must already exist. Single local same-volume writer only; no power-loss durability/network-drive/uncooperative writer race guarantee.

Reproduce scoped acceptance (PowerShell; process-local cache workaround only when needed):

```powershell
$env:RUSTC_WRAPPER=""
$env:RUST_MIN_STACK="16777216"
python tools/prepare_sdk.py --repository D:/autostack/auto-lang
python tools/verify_source.py
cargo test --locked --test authoring_document --test authoring_session --test authoring_storage --test authoring_host --test authoring_feasibility -- --nocapture --test-threads=1
cargo rustc --locked --bin autographics-authoring -- -C link-arg=/STACK:33554432
./target/debug/autographics-authoring.exe --window-probe --report research/authoring-native-positive.json
./target/debug/autographics-authoring.exe --window-probe --sabotage-target --frames 280 --report research/authoring-native-negative.json
./target/debug/autographics-authoring.exe --inspect samples/authoring/example.agscene
python tools/package_consumer.py C:/path/to/new-consumer
```

The sabotage invocation must exit1/`passed=false`; the same positive must reach phase9/`passed=true`, actual OS list/field/Apply/Save/Undo/Redo/resize. `--inspect` launches an independent process, reads/rebuilds/renders the file and emits semantic content/hash and actual2D/3D capture hashes, empty history and clean dirty state. Native probe files under research are evidence outputs, not authored inputs. Reports include actual CPU-frame P50/P95/compile time, process working set and GPU target/mesh/pipeline/depth/upload counts. Iced internals/atlas/driver allocations and GPU timestamps are unavailable and excluded explicitly. Twenty owner cycles test real VM weak witnesses and actual runtime/mesh last owners plus legitimate pins; counters alone are insufficient.

Prepared consumer B is self-contained source/SDK/AGP/assets/locks, with no target/rlib or live producer dependency. It requires the same explicit32MiB PE main stack on standalone native builds and16MiB worker stack. The PLAN-835 manifest supersedes source verification for the extended file set; immutable PLAN-834 receipts remain historical and unmodified. Complete driver/AGP regression gates remain merge-owned after independent final review; work-scoped passes are not delivery completion.

## PLAN-836 viewport interaction (work candidate)

Run the existing autographics-authoring entry. Click 2D shapes or original 3D models to select the same stable ID as the list. Blank canvas clears selection without dirty/history changes. Drag circle/quad directly after3 child pixels. For a selected3D model, use redX/greenY/blueZ world handles. The fixed camera remains orthographic; short/clipped axes are disabled, and properties remain available. Picking uses the union of all model meshes as an OBB and can select empty spaces inside that proxy; it is not triangle/pixel accurate. No rotation/scale gizmos or multi-selection.

During a gesture the panel reports preparing preview; accepted document/history/saved hash remain unchanged. Every temporary transform is checked real Auto profile3 output. The last release endpoint commits once, while noop/cancel/failure commits nothing. Esc, focus loss, leave, resize/DPI, invalid pointer or stale identity cancels and restores accepted output without recompilation/upload. Other editor commands are rejected while busy; Close first cancels then uses the original unsaved-content decision. Schema1 and Save/Open remain unchanged. Fixed original meshes are warmed once (7 parts); no per-move model upload or target/pipeline creation. One accepted, one displayed and one inflight VM plus one latest coordinate are the maximum live candidate owners. A finishing gesture has a30s terminal deadline; compile is synchronous and measured separately from8ms pumps.

Work-scoped acceptance:

```powershell
python tools/prepare_sdk.py --repository D:/autostack/auto-lang --verify-only
python tools/verify_source.py
python tools/verify_plan836.py --controls
cargo check --locked --all-targets
cargo fmt -- --check
cargo test --locked --test authoring_interaction --test authoring_viewport_feasibility --test authoring_host --test authoring_session --test authoring_storage -- --nocapture --test-threads=1
cargo rustc --locked --bin autographics-authoring -- -C link-arg=/STACK:33554432
./target/debug/autographics-authoring.exe --interaction-check --report research/viewport-native.json
./target/debug/autographics-authoring.exe --interaction-check --sabotage-target --report research/viewport-wrong-target.json
./target/debug/autographics-authoring.exe --interaction-check --drop-release --report research/viewport-drop-release.json
```

The two negative invocations must exit1 on the same oracle, respectively wrong target at phase1 and missing release timeout at phase4. --interaction-check implies --interaction-probe; without check, probe emits live observational JSON/PNG only. The check fixture posts native pointer/key messages to this process's own window, checks interim GPU frames before release, exact1e-3 world tolerance using actually quantized physical OS endpoints, Undo/Redo/Esc, original GLB each axis, resource failure/recovery, Save and resized/DPI route. Computer Use manual drag/button evidence additionally exercises physical mouse input. --interaction-fault cancel/resource-last are explicit one-shot boundary injections for manual probes, not ordinary application settings. Native commands must run serially, and the executable must exit before rebuilding on Windows.

See research/viewport-feasibility.md, viewport-handoff.md and viewport-spec-delta.md. Independent consumers include the PLAN-836 frozen manifest and pinned SDK; package/check them with the existing package_consumer.py and verify_source.py. Final complete driver/AGP gates, both original native modes and both counter/content composite gates remain merge-owned after final review. Production I2/I3/Forge/RQHost remain pending.

F-836-R1-01 completion contract: any self-check (`--interaction-check` or `--window-probe`) that returns before its own oracle completes must exit nonzero, including frame caps and normal early close. Completion is resolved before writing the report, so `passed`, `error` and process exit agree. `--interaction-probe --frames N` remains observational and may exit0 with passed=false. Initialization failures also produce a failure report without requiring an initialized Host. Permanent regression: `cargo test --locked --bin autographics-authoring --test authoring_probe_exit -- --nocapture --test-threads=1` (includes all three interaction truncations, legacy truncation, observation controls, startup failure, partial/normal-return/completed states). The native fixture establishes own-window focus/pointer and drains initial pointer events before starting; it does not suppress real leave/focus/resize cancellation. Interfering desktop input still causes a truthful failed gate.


## PLAN-837 finite hierarchy authoring (current work candidate)

The real2D/3D scene tree lists stable group/leaf IDs. Create a2D or3D group,
select a node, choose Root or a parent group by ID, then Keep world / attach.
Local positions and visible remain editable; read-only World/effective show
inheritance. Groups inherit translation/visibility only. Leaf rotation/scale
remain local. Delete on a group shows descendant count and asks to delete the
subtree; Cancel preserves it. Every command or completed drag is one history
step. Undo/Redo restore whole subtrees and array order. New IDs never retreat
on Undo/Delete; u64MAX scenes remain editable but cannot create another ID.

Pick actual leaves in either preview; select a group through its tree row.
A visible group has one aggregate outline and origin handle:2D cross,3D existing
worldX/Y/Z axes. Drag moves group local once and all descendant worlds through
real Auto. Empty/no-visible-descendant/clipped/degenerate origins display a
property-only reason. Root/parent target buttons use stable IDs even for same
names. Tree indentation visually stops growing after8 levels; all128 nodes
remain present. Scroll for lower properties/long trees. Parent/structural edits
are rejected while a gesture is active. All836 cancellation/last-endpoint,
DPI/resize/focus/old-message and resource-failure protections remain.

Schema2 stores groups and explicit parent on every node, local fields only.
Strict v1 import retains original IDs/order/fields and source bytes without
automatic save; initial imported state is clean. Explicit Save/Save As writes
v2. The old reader cannot read v2. Groups and leaves share128 nodes/IDs, with
1MiB files,128 Unicode name chars,64 history steps/16MiB. Cycles, unknown/leaf/
cross-dimensional parents, duplicate fields/IDs, unsupported group properties
or finite/range violations reject the whole candidate. Hidden worlds are
validated too. Keep-world recomputes target/descendants and rejects numerical
cancellation that exceeds max(1e-3,8*f32EPSILON*max(1,abs(before),abs(after)));
it does not move children to compensate or promise keep-visibility.

Profile4 source samples/authoring_hierarchy/main.at receives local nodes and
transaction-local parent ordinals (-1 root); it computes f32 world/effective.
Building verifies all outputs independently and actual returned values drive
render/pick/outline. Original profile3 source and profile1/2 game protocols,
SDK9576b4d, registry locks, AGP and original assets/goldens are unchanged.
The authoring entry selects profile4 explicitly, including empty/new/v1 import.
No group/source operation changes the production runtime or SDK.

Implementation-scoped acceptance (from this directory):

```powershell
python tools/prepare_sdk.py --repository D:/autostack/auto-lang --verify-only
python tools/verify_source.py
python tools/verify_plan837.py --controls
cargo fmt -- --check
cargo check --locked --all-targets
cargo test --locked --test authoring_hierarchy -- --nocapture --test-threads=1
cargo test --locked --test authoring_document --test authoring_host --test authoring_session --test authoring_storage --test authoring_interaction --test authoring_probe_exit --test hierarchy_feasibility -- --nocapture --test-threads=1
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/check_hierarchy.py research/hierarchy-native-check
python tools/check_hierarchy_pixel_faults.py research/hierarchy-pixel-faults
./target/debug/autographics-authoring.exe --open samples/authoring_hierarchy/example.agscene
./target/debug/autographics-authoring.exe --inspect samples/authoring_hierarchy/example.agscene
python tools/package_consumer.py D:/path/to/new-independent-consumer
```

check_hierarchy.py requires full phase16/exit0/passed=true, actual OS/Iced commands,
seven required final-composition pixel records (temporary, move, Undo, Redo,
hidden, Show, saved/unselected), matching world/pick/GPU commands, then saved
new-process reopen with matching document/world/unselected2D/3D capture and clean
empty history/selection. --wrong-parent, --drop-release and --frames 1 each must
fail the same self-check, with matching error/report/nonzero exit. These modes
cannot treat early window close as success. Every visible-state record compares
both actual endpoint leaf color and vacated-origin background against independently
captured initial colors; Undo reverses them, and hidden requires both background.
Missing readback fails. check_hierarchy_pixel_faults.py builds test-only native
copies against the real probe/libraries, injects actual transparent endpoint or
old-position ghost rendering, restores command-inspection values, and requires
the native/report/aggregate to fail at the same move-pixel gate. It modifies no
producer/SDK/AGP source. This repairs the missing oracle in F-837-R1-01 (independent re-review pending):
earlier work collected initial
colors and hashes but omitted the visible-state comparisons, so its prior pixel
coverage statement was invalid. Matched wrong reopen hashes cannot replace the
actual content gate. A raw observational window may
still be bounded with --frames N. check_hierarchy_sensitivity.py is a sequential
PLAN-837-tree-only mutation gate; it must run with no other writer/build and
restores exact sources before a passing control. It deliberately retains an
old VM or reuploads resident meshes and requires the20-round hierarchy test to
fail at the lifetime/Arc-identity assertion, not merely fail compilation.

See research/hierarchy-{source,feasibility,handoff,spec-delta}.md for actual paths,
AC/case evidence, ownership, measurements and remaining merge gates. Source
identity is research/plan837-identity.json, not a rewritten historical manifest.
External consumers retain the same source/SDK/asset closure and their own target.
No unbounded original GLB uploads:7 resident meshes/14 initial writes136952B;
20 hierarchy rounds preserve actual mesh Arc identity, targets and VM owner bound.
GPU timings/residency, driver allocations and Iced internal buffers are excluded.
Compilation is synchronous and measured separately from the8ms pump budget.

Work completion awaits the recorded handoff; independent final review and the
complete frozen merge-owned driver/AGP/native/counter+content/source/consumer
matrix remain separate. See the inherited README/research merge commands and
hierarchy-handoff.md. Source lands only autographics-dev. I1 dual Auto games
remain delivered; I2 production AutoUI/RQHost and I3 formal Forge/distribution
remain pending. This adds finite scene organization, not general TRS/Prefab,
asset import/material expansion/game editing/public stable engine APIs or full
engine completion. No Linux/Shader/CAP/ACE/production delivery is claimed.
# PLAN-838 current asset authoring contract

Current authoring saves strict `autographics.scene/3`. Existing v1/v2 files open
without rewriting; explicit Save writes v3. The older sections below retain
their original feature/test provenance; the current source fence is
`research/plan838-identity.json` and `tools/verify_plan838.py --controls`.

Enter a local GLB path and use **Import GLB** to create one 3D leaf in the selected
3D group or at root. The registered-resource rows create another instance or
replace the selected 3D leaf while retaining its ID, parent and local properties.
Undo/Redo retain original immutable resources; changes to the original file take
effect only through another explicit Import and receive a new SHA identity.
Busy imports expose Cancel import. Parsing/compilation and GPU preparation are
synchronous at their stage boundaries; cancellation does not preempt native calls.

The supported subset is GLB2, one embedded buffer and scene, static triangles,
POSITION/NORMAL, optional unused TEXCOORD_0, opaque single-sided untextured flat
color. The sole scene is selected when its optional default declaration is absent.
U8/U16/U32 indices must be losslessly u16; nonindexed triangles and bounded
sparse accessors are supported. All declared nodes/meshes/materials are checked,
including unused declarations. Texture/image/URI/PBR/animation/skin/morph,
compressed/unknown extensions, mirrored/singular transforms and invalid geometry
are rejected before the importing decoder or GPU. A missing material maps to
white, metallic0/roughness1. This is a finite geometry importer.

Limits: 8MiB/file, 1MiB JSON,128 GLB nodes,16 expanded parts,65536 vertices/part,
262144 vertices and786432 indices/asset; active current/history/candidate union
8 external SHA identities/32MiB raw,64MiB final CPU geometry and64MiB expected GPU
geometry. Raw duplicates can be stream-verified and reused even at the full
budget. Every 3D leaf's potential parts count, including hidden nodes, must total
at most the renderer's512 instances. Scene/history retain128 nodes,1MiB entry and
64 steps/16MiB canonical history. Registered unused resources remain in the
document; New/Open/Close release them with their last legal owner.

Save As targets an existing local directory. Keep `scene.agscene` and its sibling
`assets/<sha>.glb` directory together. A project-root owned lock is acquired before
the entry lock; immutable blobs are flushed, checked and installed without
overwrite, then the entry is committed and the complete closure read back.
Pre-entry failures preserve the previous entry. Correct newly installed blobs
can remain as reported owned residuals and are reused on retry. Foreign/bad blobs
are preserved and reported. Unknown partial/temp files or an occupied lock require
explicit local inspection/recovery; no stale-lock deletion or disk GC is performed.
The8-file/32MiB disk budget includes residuals and temporary reservations. Windows
reparse paths and network/device roots are refused. This local single-writer
protocol has the inherited same-volume, non-malicious path-race/power-loss bounds.

Run from this workspace:

```powershell
python tools/prepare_sdk.py --repository D:/autostack/auto-lang --verify-only
python tools/verify_source.py
python tools/verify_plan838.py --controls
python research/asset-probe/generate.py --check
cargo test --locked --test authoring_assets -- --nocapture --test-threads=1
cargo test --locked --manifest-path ../autographics-prototype/Cargo.toml --test asset_resources -- --nocapture
cargo build --locked --bin autographics-authoring
python tools/check_scene_assets.py research/asset-native-NEW
python tools/check_asset_owners.py research/asset-owners-NEW
python tools/package_consumer.py D:/autostack/.wt/lang-838/consumer-NEW
```

The native gate types the actual path through OS/Iced and checks both colored
parts and their vacated/background positions at9 states. Wrong Auto resource,
actual missing/transparent/wrong GPU model, missing release and partial runs must
fail native/report/aggregate. It relocates the saved project, removes its source
and original root, and reopens in another process/cwd; missing/tampered resources
must reject and restored resources must recover. Owner controls retain an actual
payload or force an actual GPU reupload; the same20-round assertions must fail.
All controls use fresh output directories. See
`research/scene-assets-handoff.md` and `research/scene-assets-spec-delta.md`.
An exported source consumer has its own target and does not inherit the repository
root Cargo settings. Set `$env:RUST_MIN_STACK="16777216"` and
`$env:RUSTFLAGS="-C link-arg=/STACK:33554432"` before its runtime tests/builds:
the pinned SDK requires the worker stack and Windows program stack separately.

Work/review use targeted checks. Complete frozen driver/AGP/game/composition/source
gates and landing belong to merge after final review. Source lands on
autographics-dev; I1's delivered two games remain the denominator. Production
I2/I3, public stable engine APIs, broader materials/animation and distribution
remain pending. The current slice is portable finite scene and static-model
authoring.
## PLAN-838 final-R1 repair boundaries

Static import checks semantic accessor types before typed readers: POSITION and
NORMAL are non-normalized FLOAT VEC3, optional TEXCOORD_0 is FLOAT VEC2 or
normalized U8/U16 VEC2, and indices are non-normalized U8/U16/U32 SCALAR. Sparse
and interleaved storage keep these same rules. All morph declarations and raw
matrix-plus-TRS declarations reject, including unused meshes/nodes. Save As
resolves the real destination before reserved blob/lock and conflict checks,
including Windows case and ordinary `..` aliases. These rules reject invalid
inputs while preserving the accepted scene and immutable resource bytes.

## PLAN-839 scene Run mode (experimental)

Open the authoring window and choose the2D/3D actor IDs in the panel. Run uses the
current accepted scene, including dirty and unsaved imported models. Click a canvas
for its keyboard focus: arrows move that actor (2D X/Y at60 units/s,3D X/Z at3 units/s),
V toggles visibility. Pause/Resume, neutral Single Step and Stop are actual retained
Auto operations. Stop restores the original editable scene; Esc also Stops. Apply or
Cancel a property draft before Run. Editing/import/save/actor choice are locked during
play. Blur/leave/resize/DPI/minimize or overload clears input and auto-Pauses.
During startup, cancellation finishes the complete initial frame into Paused;
only explicit Resume starts ticking. Initialization retains its30s budget.

This delivers the built-in samples/authoring_play/main.at profile6 behavior. It does
not load arbitrary user scripts or save code/ticks/runtime state into scene files.
Schema3 and prior resource limits/profiles remain. Production I2/I3 remain pending.

```powershell
cargo run --locked --bin autographics-authoring
cargo test --locked --test authoring_play --test authoring_play_host -- --nocapture --test-threads=1
python tools/check_scene_play.py research/play-NEW
python tools/check_play_owners.py research/play-owners-NEW
python tools/verify_source.py
python tools/verify_plan839.py --controls
```

See research/scene-play-feasibility.md, scene-play-spec-delta.md and
scene-play-handoff.md for the exact state/protocol/budgets, verification boundaries
and review/merge-owned matrix. Complete frozen merge gates follow final review.

## User behaviors (PLAN-840)

The experimental authoring host accepts one local UTF-8 `.at` source using
`app_start()` and `app_update(dt float)`. Select User source in the Editing panel,
or use the ordinary CLI entry:

```powershell
cargo run --locked --bin autographics-authoring -- --open samples/user_behavior/move/scene.agscene --behavior main.at
cargo run --locked --bin autographics-authoring -- --open samples/user_behavior/patrol/scene.agscene --behavior main.at
```

A relative behavior path resolves against the opened scene's directory. An
unsaved project needs an absolute path. Each Run snapshots the file; edit then
Stop/Run to reload. Save/Save As saves the scene and does not save the script or
persist its selection. Missing or invalid sources report failure.

Copy either complete sample folder anywhere and run the same executable with
`--open <copied-folder>/scene.agscene --behavior main.at`; no Rust rebuild is
needed when changing the script. With neither User selection nor `--behavior`,
the original Builtin6 remains selected. User source uses versioned profile7.

See [the public API](api/README.md), [portable examples](samples/user_behavior/README.md)
and [verification/handoff](research/user-behavior-handoff.md). The corrected SDK
pin is `932ad4877e5968f1aa7227d9dcd7fb69b4c69a21`; the original AGP and registry
locks are retained. This is a trusted local scripting experiment, not an
untrusted-code sandbox or a production engine.

## Bound behavior projects (PLAN-845)

Scene/4 stores one behavior/1 binding with exact source SHA256/length and optional persistent 2D/3D leaf IDs. Open `samples/behavior_projects/move/scene.agscene` or `patrol/scene.agscene`, then Run. Copy the complete scene/assets/scripts directory to move the project. Older v1–3 files migrate in memory with no binding; explicit Save writes v4.

In Editing, choose a user .at path and Bind/update script. Binding compiles the snapshot without executing callbacks, then prepares and accepts one history step. Apply bound actors commits the current temporary actor choices; No bound 2D/No bound 3D clears each persistent role explicitly. Detach removes the binding. Undo/Redo and dirty include source identity and roles. A referenced actor or ancestor cannot be deleted until detached/retargeted. Cancel binding rejects a pending candidate. User file changes require an explicit update.

Run initially uses project default: the accepted bound bytes/roles, or Builtin6 for an unbound document. Use user script/--behavior is a temporary reread on each Run; Use built-in is a temporary built-in selection. Use project default returns to the accepted binding. Temporary sources and roles do not enter Save or history. Active Play rejects edits/files. Binding/Open never executes app_start/app_update; Run does, under the existing budgets. Stop restores the editing content.

Save prepares GLB and source blobs under the original root lock, commits only the scene entry, and confirms only after full closure readback. A pending verification receipt retains its snapshot; Verify/retry must resolve it before leaving. Immutable wrong-content files and foreign files are retained and reported. Code quotas are 8 distinct payload identities/2MiB and 8 scripts files/2MiB including residuals/temporary reservations; original GLB quotas remain. No automatic GC or support for linked/network paths is added.


Native Step regression (F-845-M1-04): `python tools/check_play_step_ack.py <fresh-output>` runs normal/delayed actual OS Step positives plus missing-click deadline and duplicate-tick negatives. These are probe-only fault controls, never Host-command replacements. Native authoring probes place the owned window within the desktop work area and reject clamped/foreign-hit pointer positions; normal application windows retain their placement. A posted click is asynchronous: phase6 awaits exactly one accepted Auto tick under the unchanged360-frame deadline, never retries the click or resets the budget. Original state/paused-tick/Stop/GPU/owner assertions remain. Merge includes this four-case regression for both producer and independent consumer in addition to the original81 invocations (83 total); work/review remain scoped.

Legacy native window regression (F-845-M1-05): the original --window-probe now uses the same verified OS click/scroll path and propagates every input error with its phase. Setup waits for window focus; no runtime focus/cancel semantics change. Actual End and Unicode character events edit the original field. The permanent authoring_probe_exit::legacy_scroll_errors_fail_before_global_timeout case checks a real clipped-field scroll failure before the unchanged1400-frame deadline; the full legacy positive still checks target/pixels, Save/Undo/Redo and resize. Merge retains83 invocations; full driver suites include this additional regression.

Native setup order regression (F-845-M1-06): hierarchy/assets prepare focus and the physical pointer before any automatic scroll, including a clipped first widget. Two permanent authoring_probe_exit tests run both actual accepted sequences with delayed setup and assert every mouse input follows preparation; over-budget setup still fails under the original240-frame phase deadline. Only initial readiness moved; later blur/cancel, pointer-owner guards and original pixel/acceptance checks remain. Complete driver count158; merge matrix stays83.


## PLAN-846 explicit spatial behavior/2 (work complete; independent review/merge pending)

Open `samples/spatial_projects/zone2d/scene.agscene` or `zone3d/scene.agscene`
with `autographics-authoring --open <entry>`. Run, click its 2D/3D canvas and hold
Right through the region. Enter/stay/exit counters and door feedback are ordinary
private Auto state, not Rust presets or engine callbacks. Pause emits nothing;
Step is one neutral update, Resume does not restart, Stop restores editing and
new Run resets state. The yellow 2D corner marker demonstrates the circle's
rectangular proxy. The 3D project moves a parent before querying its actual
nonuniformly scaled/yaw/pitch two-part GLB. Copy the complete project directory
and reopen it under a different cwd with the same binary.

Editing provides API v1/API v2 spatial choices for a temporary source or the
next Bind/update. For a temporary file, use `--behavior file.at --behavior-api v2`;
default is v1. A lone/unknown API option is a usage error. Project-default Run
uses the exact accepted Binding.api; Save never silently binds a temporary
script. API switching, Attach/Update, Undo/Redo, raw bytes, closure checks and
failure atomicity retain the scene4 project rules. The same raw Arc may be
shared, but admission credentials are API-specific and no compiled VM is cached.
V1's managed source identity and allowed user helper names remain unchanged.

Behavior/2 retains all v1 functions and adds:

| Function | Return | Rule |
|---|---|---|
| `ag_aabb_min(node,axis)` / `ag_aabb_max(node,axis)` | float | world endpoint, axis0/1/2; 2D z0 |
| `ag_aabb_contains(node,x,y,z)` | int0/1 | finite world point; 2D z must be0 |
| `ag_aabb_overlaps(a,b)` | int0/1 | same-dimension leaves, closed intervals |

Groups are not queryable. Hidden/alpha0 leaves remain geometric; source code
chooses whether visibility affects its business rule. Bounds derive from all
actual accepted model meshes, with eight local-box corners under static Ry*Rx*S,
then current Auto world translation. They contain empty/concave space and are not
exact circle/ellipse/OBB/triangle collision. No raycast, response, rigid body,
dynamic topology, index, automatic pair matrix or new physics event queue exists.

At most32 new public query attempts per app_start/app_update; the33rd is
QueryBudget before geometry access. Existing100000-step/120-resume/250ms callback,
30s initialization/2s tick and shared8ms pump limits cover these operations.
Logical metadata is at most128*7*f32=3584 bytes, excluding bounded queue/container/
identity overhead.128-node wide/deep Host controls actually execute897 additional
configuration calls, about13ms config time in the observed run. These are measured
controls, not an unlimited throughput promise. Actual Auto derives the world and
queries; Rust only prepares accepted static geometry. Warm ticks do not rescan
vertices or upload model geometry. Invalid handle/axis/group/parameter/space/
unavailable geometry/quota/configuration are sticky named errors; the complete
callback is rejected, VM closes, last-good remains. Wrong/missing configuration
cannot dispatch app_start. Stop/fixed source/new Run recovers.

SDK pin is `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`, parent932ad4877e5968f1aa7227d9dcd7fb69b4c69a21,
with only upstream auto-val nano_value.rs changed. NaN cannot alias integer/object
tags or zero; negative infinity retains its IEEE bits and floating category.
Finite/±0/±Inf and nonfloat tags retain their bits. Permanent local SDK tag:
`autographics-sdk-846-nan`; keep it after plan cleanup. Generated SDK is never
edited, only exported from that Git object. Three locks stay byte-identical.

Scoped reproduction (each command selects a defined acceptance surface):

```powershell
cargo test --locked --test spatial_api nan_guard_distinguishability_gate_original_and_alternate -- --nocapture --test-threads=1
cargo test --locked --lib spatial_admission_tests -- --nocapture --test-threads=1
cargo test --locked --test spatial_host actual_host_128_wide_deep_original_initialization_and_mixed_query_budget -- --nocapture --test-threads=1
cargo test --locked --test spatial_geometry -- --nocapture --test-threads=1
cargo test --locked --test spatial_project -- --nocapture --test-threads=1
python tools/check_spatial_queries.py <new-output> --cases zone2d,zone3d,zone2d-narrow,zone3d-narrow,partial,transparent,missing-part,wrong-asset,missing-code,bad-code,bad-api,missing-glb
python tools/check_spatial_owners.py <new-output>
python tools/verify_source.py
python tools/verify_plan846.py --controls
```

Native probes read pixels only in Paused/Editing so their GPU readback cannot
induce Running clock backlog; no runtime deadline is raised. Full source/SDK,
owner weak references and real mesh Arc/upload/target observations accompany
counter AND final-content checks. Source-only condition changes go through OS
Bind/update and Save, then another process reopens without the original source
file. Partial native completion is nonzero. No production I2/I3/Forge/AutoUI or
complete engine claim is made. The frozen full178 expected driver tests, original
163 AGP tests/83 native calls and root t+tv belong to merge after independent
acceptance; work's scoped evidence does not claim those gates passed.


### PLAN-848 finite segment examples

Explicit `autographics.behavior/3`/profile9 adds finite world-segment nearest
queries with one callback-scoped result snapshot, beside unchanged v1/v2.
Open `samples/segment_projects/beam2d/scene.agscene` or
`samples/segment_projects/sight3d/scene.agscene`, then Run the bound project.
The source owns target selection and feedback; editing reach and updating the
binding changes behavior without recompiling Rust. See `api/README.md` for
closed AABB semantics, tokens,32 credits, numerical errors and limitations.


## PLAN-853 current research baseline (work; review/merge pending)

Current source entry is tools/verify_plan853.py (selected explicitly before851 by verify_source.py/package_consumer.py); historical manifests are unchanged. Prepare the frozen SDK as above, then cargo build --locked --bin autographics-m2-baseline --bin autographics-authoring. Run python tools/check_m2_baseline.py D:/autostack/plan-artifacts/plan-853/NEW_OUTPUT --self-check. This runs50 fresh processes (five paired repeats of Move, Maze2D, Corridor3D, wide128/deep128), actual Auto/Host, five warmup plus30 accepted ticks, captures, scalar observations, external10ms working-set samples and ten actual generated API fixtures. Debug Windows configuration remains RUST_MIN_STACK=16777216 / RUSTFLAGS=-C link-arg=/STACK:33554432. No SDK/lock/API/math changes.

Host.prepare_measurement.enable(true) before Run enables numeric-only4096-sample observers. Default off. vm_output_admission_pump includes callback execution AND output query execution/collection AND nested admission; output_admission and renderer_prepare are nested observations. Never add these intervals to the existing pump/transaction/frame times. Driver timing is CPU wall time including synchronous submission waits, not GPU timestamps. Reading, source generation, compile, Run/init, first render and Stop are separate named bounds; sampled working-set maximum may miss shorter peaks. Statistics use nearest rank, retain all raw samples, show across-process medians/range and observed on/off differences without promising improvement.

m2_evidence.py --self-check checks actual frozen-file corruption/missing/restoration, target rebuild independence, nested child/0selected/skip/partial/interruption and named negative controls. --record-runner EXE --output NEW_RECEIPT copies to a SHA directory outside Git and records source/SDK/locks/configuration. --verify-runner RECEIPT [--validate-run RUN_RECEIPT] rejects mismatched/missing/modified runner/raw report. Native authoring reports now identify their own actual executable; check_bounded_motion.py freezes the supplied executable, validates every report/reopen identity. check_motion_owners.py freezes native before owner build and invokes the immutable actual owner-test artifact for all original six negative owners/content controls and restored positive.

Generated handoff records preserve pre-namespace API credential separately from actual emitted API-region SHA. Full actual UTF8 combined/user bytes, region labels/offsets, bridge/combined independent hashes are exported; caller source bytes remain LF/CRLF exact. See research/m2-baseline-spec-delta.md. Production I2/I3 and later854/855 remain pending; this is not M2 delivery.

Replay completed research evidence without resampling: python tools/check_m2_baseline.py EXISTING_OUTPUT --validate-only --self-check. This verifies50 raw process-report hashes/commands, all accepted ticks/memory samples/statistics, actual generated bytes/regions and immutable runner. Fresh collection still requires a new output path. Exported consumers have no Git checkout: runner receipts explicitly use verified source identity and source_base_commit with code_commit=null; no producer checkout lookup or copied compiled artifact is substituted. Each new runner receipt archives the exact full identity JSON beside retained artifacts.


R1 evidence repairs: current admission closes the driver/AGP package-root files and authored directories in addition to recursive source folders; undeclared build.rs, package/shared experimental .cargo configuration and automatic examples/benches reject before Cargo metadata or export. The original listed AGP build.rs stays accepted/exported. Generated build/SDK outputs and research captures retain target/.sdk-source/research homes. All five pre-namespace API credentials are independently assembled from actual fixed API inputs by tools/m2_api_credentials.py; final emitted-region hashes remain separate. Test selection now requires every announced binary to finish even when older required IDs are complete; completed zero Doc-tests and nested portable_child remain valid.

Finding-specific permanent tests (no Rust/full-tier run): set M2_GENERATED_FIXTURE to an actual --generated directory, optionally set M2_TEST_ARTIFACTS to a fresh evidence folder, then python -m unittest discover -s tools -p test_m2_evidence.py -v. The source tests exercise actual verifier/metadata gate/export failures in disposable inputs and verify the restored independent consumer. Generated tests cover all five APIs and LF/CRLF; selection tests cover unfinished headers/counts/results, unframed failure, completed extra binary and nested child.


R2 completion repair: each log frame now owns announced case names separately from completed IDs. A new case cannot replace a pending one, starts cannot exceed the declared count, and a summary requires no pending case and exact announced/completed/count equality. Explicit completion needs an owner; nested child exit leaves the parent pending until its own completion. Targeted replay: python -m unittest discover -s tools -p test_m2_evidence.py -k SelectionCompletionTests -v. These tests exercise both selection and validate_run, including pending parent/child cases, missing parent completion, unowned completion, valid multiple cases/children, and empty Doc-tests. No source/API/performance/full Cargo campaign is needed for this parser-only repair.


## PLAN-854 trusted source assembly

Current source/export admission is `tools/verify_plan854.py` and `research/plan854-identity.json`; historical manifests/verifiers remain unchanged. `authoring/behavior/assembly.rs` owns the exhaustive five-version recipe, checked transport/setter/token/entry boundaries and region label. User admission, namespace, original bytes, compatibility credentials and final region bytes remain unchanged. The old pre-namespace v5 `_ag_derive` replacement matched nothing; its removal intentionally preserves the emitted transport rather than enabling a new optimization.

`cargo test --locked --test behavior_source_identity -- --test-threads=1` checks ten actual pre-change853 fixtures (LF/CRLF, Unicode, local reuse), independently hashing combined/user bytes and exact metadata. `cargo test --locked --lib authoring::behavior::assembly -- --test-threads=1` mutates trusted input boundaries and checks rejection/restoration before compilation. Neither command updates the frozen expectations. Use `cargo fmt -p autographics-auto-driver` to avoid descending into the pinned SDK's separate workspace.

The853 baseline command now records the854 current identity and retains its artifacts under plan-854. Same five workloads, flags, samples and measurement limits apply; no performance improvement is claimed solely from assembly structure. Independent consumers export the new assembler and frozen tests, build their own target against their copied fixed SDK, and execute real Auto behavior.


## Current authoring responsibility map (PLAN-855)

This remains the finite Windows experimental authoring host on `autographics-dev`.
Production AutoUI/RQHost/Forge integration, Grid/camera navigation, dynamic game
objects and general engine SDK remain pending. Behavior/1..5 and profiles3..11,
scene4 project bindings, fixed budgets, original sample bytes and SDK pin remain.

| Current entry | Responsibility |
|---|---|
| `authoring::host::Host` in `src/authoring/host.rs` | Original public Rust facade, sole Editor/Play/runtime/resource composition, construction and accounting |
| `src/authoring/host/panel.rs` | Stock Iced widgets, local drafts and stamped messages |
| `src/authoring/host/commands.rs` | Editor/project dispatch, accepted-only persistence and unsaved decisions |
| `src/authoring/host/play_adapter.rs` | Source admission, single Play wiring, Pause/Step/Stop and restored editing frame |
| `src/authoring/host/viewport.rs` | Real Auto candidate presentation, input/capture/gesture cancellation and frame publication |
| `authoring::{session,play,interaction}` | Existing transaction, VM scheduler and gesture state owners; no duplicate state |
| `src/bin/autographics-authoring.rs` | Window lifecycle and OS event delivery |
| `src/bin/authoring/{cli,probes}.rs` | Existing CLI/report/exit and explicitly selected native probes; shared real Host paths |

Current source check: `python tools/verify_plan855.py --controls`, followed by
`python tools/verify_source.py` for pinned SDK/Cargo closure. Export with
`python tools/package_consumer.py <new-outside-directory>`; the consumer owns its
SDK and target. `python tools/check_m2_baseline.py <new-output> --self-check`
records five workloads with observation off/on and immutable executable identity.
`cargo test --locked --test behavior_source_identity -- --test-threads=1` retains
the frozen 853/854 byte fixtures. Existing native CLI flags/report fields and
Rust method signatures remain available. Splitting implementation does not claim
faster execution; measurements distinguish CPU timing from GPU or frame promises.
Historical identities remain historical and are not rewritten to admit current
sources. The historical PLAN-855 closure is `research/plan855-identity.json`.

The active hierarchy pixel and Play clock negative-control tools resolve both
`authoring_probe/` and private `authoring/` modules when compiling temporary
native entries. Pixel faults also copy the CLI inspect consumer before injection;
real GPU/clock rejection oracles and original sources remain unchanged.

Patrol origin pixel expectations use the actual floored readback pixel center
against the independent two-triangle fixture, preserving original color,
all-part and genuinely vacated-origin assertions. A deterministic real Auto/GPU
regression covers the recorded boundary, a clear neighbor and opposite parts;
restoring the old ideal-point formula makes that regression fail.


## Current finite 2D editor view (PLAN-856)

Editing uses a separate `authoring::view2d::View2d`, with a presented snapshot
shared by drawing, exact picking, group outlines and Auto translation candidates.
The default center (0,0), zoom 1, Grid off and Snap off retain the original 2D
scene image. The original panel controls retain their positions; the scrollable
panel ends with Grid/Snap, fixed world step 1/5/10/50/100, Focus 2D and Reset 2D.
Middle mouse pans, wheel zooms around the pointer, F focuses the selection and
Home resets when the 2D surface owns keyboard focus. Zoom is 1/8..8 and center
is bounded to +/-65536 world units. Unframeable selections report a reason.
Grid uses world-aligned 1/2/5 spacing, at most128 rectangles in a separate
CPU batch prepended to content. It adds no scene nodes, mesh uploads or VM state.
Snap rounds the final world origin away from zero at half steps; group drags
change only the group's local translation. View changes never enter scene4,
dirty state or undo history. Save/Undo retain the view, successful New/Open
reset it, failures retain it. Play uses the original view; Stop restores the
editing view. Object gestures and navigation exclude each other; Esc, blur,
leave and resize cancel continuous navigation back to its starting view.

Historical PLAN-856 closure: `research/plan856-identity.json`, `tools/verify_plan856.py`.
Run `cargo test --locked --test editor_viewport2d -- --test-threads=1` for named
mapping, lifecycle, resources and real Auto gesture cases. The native check is
`python tools/check_editor_viewport2d.py <fresh-output> --executable <retained-exe> --controls`.
It uses actual OS events and winit, reads composite pixels and requires every
phase to finish; partial reports fail. This extends the original native gates;
runtime cameras and production I2/I3 remain pending.


## Current finite 3D editor view (PLAN-857)

The editing-only 3D view keeps its camera, projection, grid and snap separate
from the document and the runtime camera. The default and reset reuse the
original sample camera exactly. In the 3D surface, right drag orbits, middle
drag pans and the wheel zooms; P changes orthographic/perspective projection,
F frames the selected visible content, and Home resets when that surface owns
keyboard focus. The panel also exposes these actions, Grid, Snap, and fixed
world steps. Unframeable selections and bounded projection saturation report
a reason. Cancel, blur or resize restores an in-progress navigation; Play
uses the original camera and Stop restores the editing view.

The finite XZ grid is an editor-only depth-tested pass, with no scene nodes or
shadow casters. The default leaves it disabled. World-axis translation uses a
frozen camera ray/plane and world-coordinate snap, including parented objects
and groups; unstable axes or invalid samples are rejected without jumping.
Save and Undo retain the view, successful New/Open reset it, and failed opens
retain the previous view. Grid-off after use retains its bounded buffers for
reuse until the scene is destroyed.

The current source closure is `research/plan857-identity.json`, verified by
`tools/verify_plan857.py` and `tools/verify_source.py`. Run
`cargo test --locked --test editor_viewport3d -- --test-threads=1` for scoped
view, interaction and resource cases. The native check is
`python tools/check_editor_viewport3d.py <fresh-output> --executable <retained-exe> --controls`;
it checks real OS input, Auto edits, composed grid visibility and depth,
focus/cancel/Play lifecycle, and rejects incomplete reports. Full producer,
consumer and affected AGP gates belong to the later merge stage.

## PLAN-879 repository move (current home: auto-ui)

The AutoGraphics sources now live in the **auto-ui repository** at
`experimental/autographics-auto-driver/` (this directory) and
`experimental/autographics-prototype/`. The historical home
`auto-lang` branch `autographics-dev` (through commit
`1cc68aba012f76951af2556763a5fb21be6a6c8b`, the PLAN-875 delivery) keeps the
full history, design docs, and Specs; new development happens here only.

Cold start from this directory — no auto-lang checkout, no network, no Git
object database:

```powershell
python tools/prepare_sdk.py --from-snapshot
python tools/verify_source.py
```

`--from-snapshot` materializes `.sdk-source` from the versioned frozen source
snapshot in `sdk/frozen/` (681 files; per-file verified against
`research/source-lock.json` before anything is published; refuses an existing
`.sdk-source`, links/junctions, path escapes, and any missing/extra/mismatched
file, and never leaves a half-prepared directory behind). The legacy
`--repository <auto-lang>` export path still exists for historical
reproduction of the (now unreachable) pinned Git object
`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`.

`research/plan879-identity.json` is the current tracked source closure: it
records every file with original-byte and CRLF-normalized SHA256, the old
auto-lang blob SHA for all 702 migrated files, the declared PLAN-879
adaptations (path/prepare/identity tooling only — algorithms, Auto API, scene
formats, budgets and both Cargo.locks are unchanged), and the frozen snapshot
identity. `python tools/verify_plan879.py --controls` is the current verifier;
historical planNNN verifiers remain for history.

This workspace now also carries its own `.cargo/config.toml` (32MiB PE
main-thread stack for Windows MSVC, 16MiB `RUST_MIN_STACK`),
as does AGP. package_consumer exports include it, so standalone consumer
builds no longer need the manual `RUSTFLAGS`/`RUST_MIN_STACK` environment
workaround described in the historical sections above; those sections are
retained unchanged.

## Finite animated GLB fixture (PLAN-006)

The explicit `import-animated` action and `autographics.node_animation3d/1`
(CLI `node-animation3d`, version110/profile21) add bounded no-skin node TRS
clips and visual facing. Auto selects clip, seconds and yaw; the Host validates
and samples them as part of the same frame as physics/camera/objects/events.
The ordinary import and earlier behavior profiles keep their support domains.

Open `samples/node_animation/scene.agscene` and Run its project-bound Auto
source. It retains the original four Kenney clips and shared PNG, survives
SaveAs/move/cold reopen, and uses normal character/orbit input. See the
[sample](samples/node_animation/README.md) and the
[finite API/limits](research/node-animation-spec-delta.md). This is an engine
fixture, not a complete game or Gallery release. The new input snapshot and
one visual setter each cost1; the full input/move/grounded/event/visual path
fits the unchanged32 shared credits.

`python tools/verify_plan006.py --controls` checks the complete current source
closure; `python tools/check_node_animation.py --controls --reports <external-dir>`
runs the bound new-family gate plus three selected inherited cases. Current
identity is006; historical001..005 receipts are preserved. The source-only
exporter chooses006, and its frozen SDK can be prepared with
`python tools/prepare_sdk.py --from-snapshot` in the exported driver directory.