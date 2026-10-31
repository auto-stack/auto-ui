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

## Web consumer boundary (PLAN-890)

[Integration guide](../web-integration.md); [Web canonical source](../../../auto-web/docs/specs/project.md); actual final cohort/gates in the central archived Plan.
