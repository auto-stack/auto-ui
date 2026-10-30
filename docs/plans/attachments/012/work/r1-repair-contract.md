# PLAN-012 R1 bounded repair entry (round8)

Baseline `87414431b9a1dbda6eb54aad999af7f734a14516`, same ui-012tree/branch.
Independent finding input/report is `../review-r1.md`; exact raw probe/PNG/capture
logs remain external under plan-artifacts/auto-ui/plan-012/review-r1. No goal,
support, compatibility, pricing or threshold change is authorized or needed.

| finding | invariant / actual path and owners | scoped correction / distinguishing oracle | status |
|---|---|---|---|
| F012-R1-01 | Native manager/observation lives with Host across Stop/New/Open and can be required by empty-cue Run, independently of audio Asset lifetime. Existing Registry/raw/image/model/audio admission and codec peak prediction share64MiB. | Add explicit Host service requirement to the Registry budget context; a live/required manager's uncovered reservation participates in the same Usage/predict/decode/prepare/report paths. Existing conservative audio-asset reservation covers that manager once; no double charge, fake asset, hidden global context or reduction of manager-free legacy64MiB. Preflight newly required service BEFORE opening. Permanent cases: real review PNGs/normal stamped imports, cold legacy4accepted; retainedNew/Open fourth rejected predecode/no state change; near-bound accepted control; empty-cue Run refuses before native allocation; same service retained across Stop/fault/restart, pending/history and first audio import prediction remain bounded. | pending |
| F012-R1-02 | Actual compatibility test capture earlyreturn plus inherited capture environment/count-only proof skips golden. | Capture becomes additional output, never skips old15golden comparison. Gate scrubs ambient capture variable and validates explicit completed-comparison marker; reject actual R1 capture-only log and actual zero/partial as before. Run normal/capture actual cases, deliberately wrong golden with capture must fail, restore finally. No old profile source or expected byte baseline changed. | pending |

Current closure: both rows have been implemented, scoped checks passed, and code
committed as67ddbe2. Full results and remaining independent/merge gates are in
[r1-repair-closure.md](r1-repair-closure.md). The pending cells above preserve the
entry contract, not a claim that implementation remains incomplete.

Target scope: new Host service-boundary regressions and affected resource/pending
admission controls; relevant existing start/restart/fault/manager ownership checks,
legacy actual comparison and validator controls. Compile affected Registry callers.
No unrelated wholeAG/root/full-pacakge campaign; codec/wire/PCM/device conversion,
GPU visual,20Run churn and full performance evidence is reused where unchanged.
Refresh complete source identity and affected source-only moved offline consumer,
then hand committed clean implementation to the same independent reviewer.
The final complete directed gate remains merge-owned pending.
