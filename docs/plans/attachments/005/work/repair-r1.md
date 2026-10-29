# UI-005 R1 invariant repairs — cycle 1

Implementation tree `D:/autostack/.wt/ui-005/auto-ui`, branch
`codex/plan-005-dev`, frozen repair `8809e545830a2d0ed6f587517e6d4d8a29a575ca`.
Plan remains r3: no support, acceptance, budget or repository change.
Review baseline was `9d680f9574275d13a0f776ee321534f5a9d4c465`.

## F-005-R1-01: classify all unsupported material texture channels

Invariant: valid unsupported channels are named `unsupported` before dependency
IO/decode, across both referenced and unreferenced materials; malformed glTF
references remain invalid. The original glTF typed/reference validation now
runs before geometry scratch strips texture declarations. Every original
material is checked for normalTexture, occlusionTexture, emissiveTexture and
metallicRoughnessTexture before scratch/common geometry admission. No extra
material channel is enabled. Tangent/UV/span checks still follow complete
geometry admission, before IO/decode; no unchecked vertex reads were moved.

Permanent test:
`unsupported_material_channels_are_named_before_io_for_used_and_unused_materials`.
For four channels × used/unused states, a valid index0 input with absent PNG
must return the named unsupported channel and decode delta0. Each index999
sibling must remain invalid with decode delta0. Original textured positive
control remains accepted. Targeted command `cargo test --locked --offline
--test textured_model_resources unsupported_material_channels -- --test-threads=1
--nocapture` exits0, 1 passed (`repair-r1-material.log`).

## F-005-R1-02: admit cached immutable content independently of display name

Invariant: root SHA/length/format/dimensions and complete closure edges bind
content; display name does not. Cached load now compares those six content
fields explicitly, matching the session validator's contract. It still
re-reads root and every dependency before reuse, including hits; same-root,
different-image closures remain distinct. It does not mutate a cached Asset
or the document's display name to force equality.

Permanent test:
`cached_closure_open_ignores_display_name_but_preserves_content_binding`.
Imports identical roots/images through model.glb and alias.glb, verifies equal
keys/different names; saves alias project, cold/same-name/different-name warm
opens all accept, document retains alias name and warm payload is the original
Arc. Actual `Host::open` + settle accepts the alias project with original-name
assets active; Editor validation also passes. A corrupt image still refuses
under the different-name cache and exact restore recovers. Targeted test exits0,
1 passed (`repair-r1-cache.log`). Existing
`dependencies_are_checked_even_with_cached_models_and_recovery_reads_again`
also exits0, 1 passed (`repair-r1-dependencies.log`), including missing/corrupt
PNG, forged rehashed wrong edge, recovery and old-schema rejection.

## Original independent replay and sibling evidence

Artifacts rooted at `D:/autostack/plan-artifacts/auto-ui/plan-005/`.
Before repair, original review assertion-enabled material replay exited1 and
name-cache replay exited101, retained `repair-r1-admission-before.log` and
`repair-r1-cache-before.log`. Original R1 evidence has not been replaced by
green output. Replay sources were copied to `repair-r1-replays/`, changing only
their output directory, then rebuilt against the current producer library.
`build_review_probes.ps1` exits0; `admission_holdouts.py --expect-contract`
exits0 with all21 cases, including5 accepted controls; all rejected inputs
have decode delta0. The accepted controls include actual normalized U8/U16 UV0,
default-scene omission and unused valid sampler. `name_cache_probe.exe
--expect-contract` exits0: save Confirmed, cold accepted, warm accepted,
same-name warm accepted. This closes original examples and affected sibling
states without removing complete binding validation.

Current complete source identity remains1456 files, new SHA256
`19d19280dc2d908cd1e984603c97d2e00b1afbf95c25d3aa03771c7fb9e61e7f`.
`verify_plan005.py --controls`, driver fmt and diff --check pass. Only source,
permanent test, current identity and checker exact count18→20 change; SD draft,
SDK681, locks, AGP/shaders and other runtime paths remain unchanged. Their R1
independent pixel/owner/transaction and prior compatibility evidence remain
applicable under this explicit unchanged-input audit. Merge still owns a new
frozen producer scoped gate after affected independent re-review.

## Current consumer binding and readiness

Fresh snapshot-only export `consumer-repair-r1` contains current source at the
repair commit, with no inherited target or prepared SDK. Cold
`prepare_sdk.py --from-snapshot` exits0,681 files/17 packages; log
`repair-r1-consumer-sdk.log`. From its driver package,
`check_textured_models.py --reports <repair-r1-consumer-gate> --controls`
exits0: driver20/AGP4, no failed/ignored/filtered cases; all12 semantic report
cases and actual pixel/partial/foreign-runner/budget/warm/stale/zero-selection
controls pass/refuse as required. Actual executable hashes, source identity,
PE32MiB/worker16MiB and logs are bound in manifest SHA256
`d8005629f13b51791fc37ad62c66254f2f4c0146212a1cf84cac4391ab93becc`.
Gate log SHA256
`49f730971e74a100158770402e18fb5645ae39c5507d2699fa50f290fa57f746`.
Source guard checks SDK479 packages/29 local path edges with no forbidden
dependency. Producer source controls pass again after all builds
(`repair-r1-final-source.log`); current HEAD is clean8809e54, fmt/diff checks
pass, no new product warning (inherited SDK146 warnings only).

Original independent holdout green log SHA256
`b7ee07e72a8d35f93877bc00d0f980eec0d5f135e1ce2a391d52424472070176`;
name-cache green log SHA256
`9b5c1c730fe82086f67f543bfad27e0873c2f4905e9e2979504fc3024b7aec8c`.
Existing frozen SDK/locks/SD draft, fresh-B physical A→B proof and compatibility
evidence are retained with unchanged-input rationale; new consumer runs current
public-project save/move/reopen plus all resource/transaction/pixel/owner cases.
There is no known remaining work-owned finding or coverage gap. Both finding
closure rows above are ready for independent affected re-review; this work
record does not grant AC-06 or merge authorization by itself.
