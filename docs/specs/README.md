# AutoUI specifications

This index contains canonical repository-owned contracts for AutoUI runtime
behavior. Production UI implementation ownership remains described in the
language repository's `docs/specs/auto-ui/project.md`.

## Runtime

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

## Theme pilot & examples ownership (PLAN-007)

- [Theme tree pilot](ui/theme-tree-pilot.md) — tree-structured theme data
  (`pub theme`), extend composition, mode selection, typed slot references
  and widget-local design objects over the pilot support surface (SD-01/02).

- [Theme app migration](ui/theme-app-migration.md) — the official single-source
  theme families (`examples/ui/themes/`), app adoption form and the confirmed
  per-app successor batches (SD-03).

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
