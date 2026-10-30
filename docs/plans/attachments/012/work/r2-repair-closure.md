# PLAN-012 second bounded repair / R3 readiness

Code7e9802bb55961bf1d22be77892e568f15c9437bf, same clean ui-012tree/branch.
Base67ddbe2. Two-failure diagnosis was recorded BEFORE this repair commit in
[r2-diagnosis.md](r2-diagnosis.md); goals, r1contract and all caps unchanged.

F012-R1-01 full preallocation invariant is now repaired at the remaining shared
closure forecast: it uses Usage::from_registry(existing) before predicting
candidate geometry/images/animation. Usage has NO Default implementation; its
only initializer explicitly requires a Registry and initializes all maps/sets/
service flags. Every use was audited (six from_registry consumer calls), and an
old Usage::default call can no longer compile. The diagnosis's design adjustment
was verified by actual successful library/bin builds and executed real controls,
not only a source grep or a repeated old example.

One permanent new Host case has SIX actual scenarios using exact R2tunedPNG
bytes, original platform/character models, original R1PNGs and normal stamped
commands. Static import, animated import and normal Open→Discard/declared-load:
manager required rejects with decode_delta0 and unchanged accepted document/
pending state. Matching manager-free controls each decode_delta2 and publish.
The generated saved closure hash equals R2's exact2fbf5eb2... input. R2raw logs
had2decodes before refusal in each bad branch; the new test binds that temporal
difference, not just final error. Both new small PNGs are permanent load-bearing
regression inputs; raw probes/logs stay external.

Original repaired budget tests2/10filtered and combined-pose1/11filtered were
replayed successfully. They preserve33528B manager accounting with0assets,
New/Open/first-audio/near-bound/duplicate/emptyRun/asset+service+pose behavior and
manager-free old64MiB controls. F012-R1-02 remains closed; compatibility test/
golden/marker predicate/scrub mechanism did not change. R1resource9/start-restart2/
postfault1 and unchanged codec/wire/nativePCM/GPUvisual/40Run evidence are reused
by explicit unchanged paths. This repair changes only how the same existing
Registry is fed into the previously orphaned forecast; no shader/codec/VM/stream/
lifetime mechanism or old supported defaultfalse budget arithmetic changed.

Current identity1536files SHA256
640126305f058592ea02423c8888cefd6a8c26f68e99a09d3f4d90f02834dd6f.
SDK681/17local/29edges/479metadata and both dependency locks remain exact.
SD repeats the existing all-entry preallocation rule, SHA256
634542205cffdb4808e2e7cb63f20aa5d470c4525d93a7185ecac90d18c17711.
Fmt/diff/Python compile and complete identity controls pass; SDK146known warnings
retained, no new delivery warning observed.

New source-only A→B export starts before preparation with no SDK/target. In B,
own frozen SDK preparation and source/metadata guard pass, then an actual locked/
offline independent compile83s and functional consumer1passed/1filtered/6.56s
pass. Both profiles, moved/cold reopened projects, source-sensitive12645frame
chosen cue/gain PCM and motion work. Actual manifest_dir is the Bdirectory and
PE main32MiB/worker16MiB, producer/B identity bytes equal. It is explicitly a
functional-only proof, not a relabeled old complete gate/performance proof.

All raw evidence under D:/autostack/plan-artifacts/auto-ui/plan-012:

| command / observed result | evidence |
|---|---|
| cargo test --locked --offline --test public_audio_host retained_service_is_in_every_model -- --test-threads=1 --nocapture;1passed/11filtered, all3bad0decode/3good2decode |r2-closure-predecode.log|
| same target budget;2passed/10filtered |r2-original-budget.log|
| same target empty_audio_service_and_animation_pose;1passed/11filtered |r2-pose-service.log|
| python tools/verify_plan012.py --controls;1536inputs/controls pass |r2-identity-controls.log|
| B python tools/verify_source.py;1536/SDK681/17/29/479 |r2-functional-source.log|
| B cargo test --locked --offline --test public_audio_consumer bound_two -- --test-threads=1 --nocapture;1passed/1filtered |r2-functional-consumer.log and r2-functional-proof.json|

Exact log/binary/proof hashes are appended in the shared Plan receipt. Two prior
failed reviews and original bad inputs are retained. Work-owned open finding/
coverage/decision list is empty for resubmission,6/10. Independent affected R3 is
still required. Final frozen gate becomes45cases (Host12); T08full current gate/
performance/fresh complete proof and T09/10knowledge-ledger-archive-guardcleanup
remain pending. No merge or canonical publication occurred.
