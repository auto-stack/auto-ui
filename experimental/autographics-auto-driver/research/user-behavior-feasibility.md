# PLAN-840 r2 feasibility: completed

The original r1 report below records a real blocker and is retained as history.
The authorized scanner correction is now frozen at SDK
`932ad4877e5968f1aa7227d9dcd7fb69b4c69a21`: the old pinned SDK plus exactly four
scanner files, 681 files, 479 dependency packages / 17 local packages / 29 edges.
Both original Cargo locks are unchanged. The three r1 dependency false positives
now pass, alongside actual dependency and fresh/cache module controls.

The selected implementation uses the actual SDK Parser/AST, compiler and VM.
`SourceSnapshot` assembles complete trusted bridge/API/entry units around the
unchanged user bytes. Trusted transport identifiers alone receive `_ag_` names;
user strings, comments, offsets and identifiers are never rewritten. A managed
`main` after all units initializes user globals before reset/config/app_start.
Callbacks and public API are real Auto functions, with no Rust motion fallback.
Profile7 and its API error header are consumed by the same frame validation and
owner/Stop machinery as Builtin6. The public API derives world transforms lazily
after writes and the first API error rejects the entire callback frame.

Evidence now includes two real user programs, 600 ticks against an independent
reference, all-part GPU pixels, twenty source A/B runs with actual VM release and
GPU sharing, source bounds/AST/diagnostics, and the native thirteen-case matrix.
The SDK source admission blocker is resolved without reducing the planned
single-file Core support. Exact commands and limits are in the API and handoff.
Independent final review and complete merge-owned gates remain required.

---

# PLAN-840 T-02 bounded feasibility: needs_replan

Baseline: formal PLAN-839 delivery `a239653365a0c3451c85aa9921189fcf011bd8cf`.
Authoritative tree `D:/autostack/.wt/lang-840/auto-lang`, `plan-840-dev`.
SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`, 680 pinned files,
479-package dependency closure, 17 local packages / 29 path edges.
SDK, AGP, production code, locks and the original profile6 are unchanged.

This is a scratch feasibility probe, not the finished behavior API, source
admission, profile7, CLI, UI, or an acceptance pass for PLAN-840.

## Actual path and successful control

`tests/user_behavior_feasibility.rs` assembles complete trusted bridge units
around intact user source. Only the trusted bridge is adapted. It reuses the
existing `Session`, real Parser/compiler/AutoVM, Play admission and Host GPU
publication/restoration. It deliberately retains profile6 for this mechanism
probe; profile7 has not been implemented or validated.

The managed reset invokes `main` before configuring the graph, so the fixed
SDK initializes user globals (including nonzero `initial=2.0`) before callbacks.
After full graph configuration `app_start` runs once. Each tick calls only
`app_update`; same-callback group writes, child world reads and another leaf
write are executed in Auto. The first scratch attempt incorrectly reentered
app_start through derive at each tick: the independent 2D position assertion
failed. The bridge was corrected; that failure is preserved in the main
checkout's work-r1/feasibility-attempt1.log.txt. No tolerance was relaxed.

The final successful test performs two runs in one unchanged test executable:
user speed3 and speed6 source variants, 60 actual Auto ticks each; a nonzero
3D parent group and an imported two-part GLB; independent 3D and 2D trajectory
assertions; red and green pixel witnesses on both actual imported mesh parts;
changed composed image, exact Stop restoration, unchanged Editor identity and
weak runtime-owner release. It also checks the nonzero global initializer at
tick0 on each Run. User callback bytes are not rewritten or interpreted by Rust.
This proves a mechanism control, not the two formal portable samples or native
OS behavior-check required by T-06/T-07.

## B-840-T02-01: inert text enters SDK dependency resolution

Both supported candidate mechanisms have been investigated, with executable
counterexamples and legitimate controls:

| Candidate | Control | Counterexample and observation |
|---|---|---|
| Complete top-level source units via existing Session | Actual GPU/control above and empty callbacks compile | Real Parser admits a block comment and a multiline string with a line `use plan840_missing_dependency: value`; AST has no Use or Dep. `Session::new` nevertheless fails `Module not found: plan840_missing_dependency`. Two independent failing tests. |
| Existing public module service | `CompileSession::add_source_dir` + `resolve_uses("use plan840_probe.*")` loads a normal module successfully | Replacing only that explicit module's bytes with the Parser-accepted block-comment fixture causes the same missing nested dependency error. A fresh CompileSession avoids cache reuse. One independent failing test. |

Durable fixtures and assertions are in the test file, including the accepted
controls. Module scratch files are ordinary files under this tree's ignored
`target/plan840-module-probe`; no links or external dependencies are created.
All three failing assertions express required behavior and remain red; they are
not converted to expected-error passes, ignored, or counted as completed ACs.

The fixed SDK's `create_vm_from_source` (`auto-lang-core/src/lib.rs:4430..4434`)
calls collect_rust_imports/collect_py_imports/resolve_deps/resolve_uses before
Parser. `use_scanner::scan_use_statements` scans individual trimmed lines and
tracks neither block comments nor strings. `CompileSession::load_module` also
calls resolve_uses on module bytes before parsing, on fresh and cached paths.
Thus moving the source to the second existing public entrypoint does not remove
the defect. The module loader additionally reads path-backed module sources;
it is not an in-memory source-snapshot replacement.

The consequence is larger than an inconvenient diagnostic: AST admission cannot
prove the promised single-file dependency boundary while a later preprocessor
can reinterpret admitted inert text as an import. A missing file produces the
observed rejection; the resolver's source shows it searches/read-loads module
paths when present. No arbitrary filesystem-read exploit claim is made; the
probes deliberately use a nonexistent nested dependency and a controlled root
module. String/comment contents are legitimate controls under the Plan.

## Contract impact and bounded revision proposal

Affected: T-02 blocks T-03..08; AC-02/03 and V-02/03 source admission/unchanged
bytes/no untracked dependencies; unchanged source identity in T-08. The old
Builtin6 behavior is not implicated. T-01 baseline checks passed independently.

The current contract forbids changing the fixed SDK, rewriting user identifiers,
literals or function logic, reading non-snapshotted dependencies, or introducing
another compiler/runtime pipeline. Rejecting these valid strings/comments would
reduce the promised support. Stripping or escaping source would hide the SDK
defect and would not meet the intact-byte assembly contract. `compile_source`
only indexes AST fragments; it does not expose a ready AST-to-Session/VM path.
The public ABT constructor consumes assembled bytecode text, not the admitted
Auto source. Building a new lowering/linking initialization path in this driver
would be a third design beyond the two bounded candidates, requiring explicit
contract review rather than a silent workaround.

Recommended new-stage revision: first fix SDK dependency scanning so inert text
cannot create imports, with block-comment/multiline-string accepted controls,
actual-import controls and module fresh/cache coverage. Deliver that change via
its own authorized scope, then explicitly refresh PLAN-840's pinned SDK and
closure/locks/source identities. Alternatively authorize a public SDK
dependency-free in-memory compilation seam, retaining the existing compiler and
VM, with the same source identity and diagnostic guarantees. Neither change is
implemented or authorized by the current driver-only PLAN-840 r1.

No canonical Spec or ledger changes have been published; SD-01..04 stay proposed.
T-02 remains incomplete, PLAN-840 executing, no execution_done/final-review pass.
Full driver/AGP/native campaigns and root whole-tier gates have not been run;
the future complete gate remains merge-owned after independent acceptance.

## Reproduction

From this tree's `experimental/autographics-auto-driver`:

```powershell
python tools/prepare_sdk.py --repository D:/autostack/auto-lang
python tools/verify_source.py
python tools/verify_plan839.py --controls
cargo test --locked --test user_behavior_feasibility -- --nocapture --test-threads=1
```

Preparation is one-time; use `--verify-only` if .sdk-source already exists.
Final test selection is exactly four cases: one passes, three fail. None are
skipped or filtered. Main-checkout work-r1 holds raw logs, frozen identities and
the work handoff; this document's conclusion must not be reported as API delivery.
