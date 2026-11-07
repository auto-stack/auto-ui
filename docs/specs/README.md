# AutoUI specifications

This index contains canonical repository-owned contracts for AutoUI runtime
behavior. Production UI implementation ownership remains described in the
language repository's `docs/specs/auto-ui/project.md`.

## Runtime

- [RenderQueue fidelity contract](runtime/rq-fidelity-contract.md) — current wire/font/resource/present behavior, atomic-frame and ACK/fence rules, frozen capability IDs, and named fidelity gaps (PLAN-061).
- [RenderQueue lossless primitives, glyph and resource replay](runtime/rq-render-packets-and-resources.md) — v3 payload (Path/GlyphRun), producer-side SDF shadow patches, content-addressed resource residency, upload budget, capability admission and slot-credit backpressure, with the measured fidelity table and named gaps (PLAN-062).
- [VM route-page loading](runtime/vm-route-page-loading.md) — page imports,
  named diagnostics, store calls and lifecycle behavior for native multi-page apps.

- [Native VM app validation](runtime/native-vm-app-validation.md) — evidence
  required to claim a native VM app was launched, interacted with, failed,
  closed, and reopened.

## Build

- [UI compilation configuration](build/ui-compilation-configuration.md) — the
  stable UI baseline is unconditional; `ui-iced` survives as a compatibility
  alias; real optional product capabilities keep their feature gates (PLAN-018).

- [UI test selection](build/ui-test-selection.md) — the three UI test-selection
  Cargo features are retired; families are compile-visible and run-selected via
  the nextest static boundary with explicit serial/manual admission (PLAN-018).

## Theme scope, publication & examples ownership (PLAN-007 / PLAN-031)

- [Theme scope and publication](ui/theme-scope-and-publication.md) — App/session
  and component-instance theme ownership, identity/revision rules, atomic candidate
  publication with keep-old-on-failure, host/controlled-input priority, local design
  scopes and the dual-end (VM/Iced ↔ Vue) projection boundaries (SD-01).

- [Official theme package](ui/official-theme-package.md) — the single-source
  `packages/themes` package, managed manifest with per-file hashes, named rejection,
  external snapshot fetch and content-digest generation admission (SD-03).

- [Theme tree pilot](ui/theme-tree-pilot.md) — tree-structured theme data
  (`pub theme`), extend composition, mode selection, typed slot references
  and widget-local design objects over the pilot support surface (SD-01/02).

- [Theme app migration](ui/theme-app-migration.md) — the official single-source
  theme families (`packages/themes/`), app adoption form, the delivered batches
  (theme-lab/002/005, then 003/010) and the confirmed per-app successor list (SD-03/04).

- [Examples ownership](ui/examples-ownership.md) — `examples/ui` relocated
  from auto-lang byte-identically; source mapping, dependency resolution and
  preserved test obligations (SD-04).

## Structure closure (PLAN-014)

- [UI structure closure](ui/structure-closure.md) — the four-face module
  layout, single state/resource owners, restored public paths and forbidden
  edges after the PLAN-014 internal-structure closure.

- [UI structure test map](ui/structure-test-map.md) — old-to-new test
  placement mapping with #[test] conservation, directed execution anchors and
  real-consumption evidence pointers (PLAN-014).

## Web consumer boundary (PLAN-890)

[Integration guide](../web-integration.md); [Web canonical source](../../../auto-web/docs/specs/project.md); actual final cohort/gates in the central archived Plan.
