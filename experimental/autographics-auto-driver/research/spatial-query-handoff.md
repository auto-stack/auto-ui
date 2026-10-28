# PLAN-846:r2 implementation and gate handoff

Authoritative tree: D:/autostack/.wt/lang-846/auto-lang, plan-846-dev.
Base:c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f. Pure upstream fix:4cef80c1b.
Frozen work code is recorded by full SHA in the main Plan; do not infer it from
this report's parent commit. Shared Plan/evidence live on main. Preserve the
read-only sibling auto-down tree for root checks and guard both trees at merge
cleanup. Keep refs/tags/autographics-sdk-846-nan permanently; the SDK artifact
must remain reachable even after plan branches are removed.

SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467, parent932ad, only nano_value.rs.
AGP and all three SDK/driver/AGP locks remain unchanged. V1 API and original
Builtin6/profile6 and profiles1..5 remain. Root auto-val package check is scoped;
root whole tiers have not been run by work. SDK's historical nested workspace
makes its own workspace-wide unit command unsuitable; the real pure module was
tested directly, and actual SDK/native Auto consumers exercise the encoding.

F-846-R1-01 repair adds spatial_cli: three permanent tests execute ten real CLI invocations for terminal missing values, next-option missing value, orphan/unknown values and absent/default/v1/v2 valid controls; inspect never rewrites the project binding. spatial_host also exercises actual file version selection/Run.

Work cases map to permanent entrypoints: spatial_geometry (all-mesh/8-corner/
cache/stale/degenerate), spatial_api (closed/ULP/hidden/self/space/errors/32-33/
native NaN guards), spatial_project (API-aware same-raw history/persistence),
spatial_host (last-good/VM close/Stop/Starting/forged identity/128-node budget),
spatial_feasibility (source-v1 helper plus real GLB save/reopen/Auto/GPU),
spatial_samples (actual circle proxy-corner pixel), spatial_owners (20 real
lifecycle cycles plus actual owner/scan/upload mutation runner), lib
spatial_admission_tests (configuration before callback), bin
spatial_completion_tests (partial/failure finalization). The original
user-behavior12/project-session4/storage5 scoped controls remain explicit.
Native records retain exact user/API/bridge/combined/geometry/SDK identity,
independent input/event trace, full-part pixels and unchanged binary SHA.

## Required final review and merge-owned complete matrix

| Gate | Required denominator / input | Owner / current state |
|---|---|---|
| Final independent review | AC01..08, SD01..05, whole diff and raw oracles, code/SDK/lock identity | review, pending |
| Upstream crate | selected nano_value module17 tests in root source, frozen pure fix | merge after review; work direct module17 pass is scoped evidence |
| Root regression | bare cargo t + cargo tv on correct dependency group/frozen code | merge, pending; no tt/tb/tu/taa trigger in this diff; tf only main single-instance due batch |
| Producer driver full test/build/source | expected178=158 inherited+20 new tests, no zero selections/skipped required cases | merge, pending; count confirmed from actual final campaign, not assumed passed |
| Fresh own-SDK/own-target consumer full test/build/source | same expected178 and own standalone32MB native profile | merge, pending; work runs only selected finding/feature cases |
| AGP complete | original163 tests/build, original16 native cases/counter AND actual-content | merge, pending; reuse only actual unchanged code/dependency/config/input receipts |
| Original native/user/project/Step/owners/source controls | all83 effective original845 invocations, including15 project cases per side and2 actual UI-output relocations per side | merge, pending; exact inherited invocation receipt: main docs/plans/attachments/845/merge-r1/validated.json.txt; map roots explicitly, never reduce denominator |
| Spatial native | each side:12 distinct cases+2 new-process narrow reopens=14 actual invocations; real14-phase completion, three GPU corruption controls | merge frozen matrix pending; work producer scoped positives/negatives are retained |
| Spatial ownership/source sensitivity |20 cycles,5 actual mutation reds+restored positive; new API/stale pin/source-lock/missing GLB fences | merge after review; work scoped evidence retained |
| Closeout | graphics only autographics-dev; pure SDK source diff only to master; canonical/ledger/archive/guard/actual due batch | merge, pending |

A changed implementation invalidates affected review evidence and gates; repair
uses scoped checks before returning to review. Do not run full tiers in work or
reinterpret a review pass as complete delivery. Work may hand off execution_done
only with all work criteria complete and clean committed delivery files. No
root/global batch red is silently waived; attribute and route actual failures.
