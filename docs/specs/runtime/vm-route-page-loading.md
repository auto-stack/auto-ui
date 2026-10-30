# VM route-page loading

Current contract introduced by AutoUI PLAN-015 r2. Native session evidence also
follows [native app validation](native-vm-app-validation.md).

## Route loading and imports

An application's declared `routes` resolve page modules under `pages/` and
register the extracted page widget through the route-module alias table.
Page module names need not equal widget names and must not shadow built-in
elements. Page loading preserves the source-prefix overlay and collects the
page's own imported functions, types and stores for the shared VM.

Imported style recipes must be prepared before parsing a page, then validated
with its local recipes before widget extraction. This is the same preparation
order used for the application's root widget. View fragments are available
before extraction. A page's widgets are validated before its aliases and child
declarations are published.

## Visible failure

A declared page that cannot resolve, read, parse, validate styles or extract a
widget must produce `VM_ROUTE_PAGE_LOAD_ERROR` with its route, module, source
file, failing stage and reason. Load-time logs emit an `ERROR` line; rendering
that route exposes the same diagnostic in the native snapshot. A source module
without a widget is an extraction failure. A broken page must not silently
become `page ... not loaded`; valid sibling routes remain navigable.

Missing registry entries at rendering are an explicit registration-invariant
diagnostic. A path that matches no declared route retains the separate
`no route for ...` result. Shared VM link failures remain named initialization
errors; this contract does not provide independent per-page VM isolation.

## Page execution

A loaded page participates in ordinary child-widget rendering and the native
frame scheduler's Init demand/completion path. Timers, conditional views, lists
and string interpolation are supported together, including imported stores and
styles. Route transitions must preserve the existing mounted/timer eligibility
and Init identity behavior; loading a page does not run its Init inline during
the view build.

In handlers, `.store.Method(...)`, `self.store.Method(...)` and leading-dot
qualified store aliases use the same store dispatch as their bare forms. Only
known store aliases are normalized. Multi-store generic calls must resolve to
one declaring store; ambiguous or missing methods retain their named errors.
Ordinary model-object method receivers are not treated as stores.

## Verification boundary

The permanent `plan015_route_loading_tests` use the production component builder
and rendered Init effects, timer registration, imports and failure controls.
`plan015-route-pages` supplies a real native CLI/MCP positive and missing-page
negative. The PLAN-003 single-page fixture remains the compatibility control.
BLOG-004's `native-vm-rust` profile owns application-level creation, saving,
HTTP/disk identity and durable reopening after normal or forced termination;
passing an AutoUI fixture alone does not establish that consumer acceptance.
