# PLAN-012 two-failure diagnosis and verified repair entry

Baseline67ddbe2, same authoritative ui-012tree. Independent R2 keeps
F012-R1-01 partially open and closes F012-R1-02. R2raw report SHA256
926b90203ed8ad887cb5a277852687e95fae3a45f8db458b9ef4612ea0248e0a
and complete replay inputs are external under review-r2; the durable summary is
[review-r2.md](../review-r2.md).

## Why the first repair did not close the invariant

The real live/required manager context is correct in Registry and the common
Usage/check/candidate/audio/Host paths. The first repair changed those constructors
and added live-manager PNG/first-audio/emptyRun/pose controls. It missed the separate
full textured-closure forecast at textured_models.rs908: this still built empty
Usage::default and copied only assets. A late insert caught the excess after two
decodes. Source-only sample loading/imports before native creation (contextfalse)
cannot distinguish this error, so normal3D/sample/resource positives did not cover
the relevant retained-manager state. Existing scalar limits were not wrong.

## Changed design/oracle before the next repair commit

Every Usage must be constructed with a Registry context. Remove its Default
constructor entirely, explicitly initialize only inside from_registry, and convert
the standalone textured/animated/load forecast to that same entry. An unconverted
Usage::default must then be a compiler error, rather than a quietly incomplete
budget. Audit all Usage literals/construction sites and all import consumers.
This is an implementation guard for the existing quota, not a new requirement or
quota/profile change.

Permanent replay will use the EXACT R2tuned PNGs plus original models and R1PNGs,
normal stamped Host imports and normal Open→Discard/declared closure loading.
Cover static textured platform, animated character and saved closure, each with
manager-required rejection BEFORE any decode/allocation (counter0) and accepted
manager-free matching control. Preserve old document/pending state on rejection.
Retain original PNG/New/Open/near-bound/duplicate/first-audio/emptyRun/combined-pose
cases, plus resource regressions sharing the modified forecast. Add an audit that
no standalone/default Usage construction remains. Recheck old total cap and real
successful import results, not just final CLI failure.

F02 needs no further implementation repair: actual capture executes unchanged15
golden comparison, malformed capture-only proof rejects and prior wrong-golden
control fails. Auto/wire/mixer/nativePCM/cursor/40Run/GPUvisual/SDK/locks remain
unchanged. Do not repeat global/performance campaigns. Regenerate complete source
identity and affected functional source consumer; keep the complete unified gate
merge-owned pending. Independent R3 must inspect the full cause after scoped repair.

Current closure row: F01 → common preallocation CPU invariant → every Usage entry
and static/animated/load consumer with required service → forced constructor plus
exact three positive/negative pairs → pending measured results → final re-review.
Two failed final reviews/firstrepair are retained; this is the second bounded
repair cycle, within original r1scope and maximum3cycles. No user decision needed.

Verified closure before resubmission:7e9802b removes Default, converts the orphaned
forecast and passes all six exact static/animated/load controls with0bad/2good
decode counts. Original budget and pose cases pass; fresh moved functional consumer
and1536identity controls pass. See [r2-repair-closure.md](r2-repair-closure.md).
