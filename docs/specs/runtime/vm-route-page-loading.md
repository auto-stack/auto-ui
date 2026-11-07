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

## Suspended navigation and result publication (PLAN-068 SD02)

A page's HTTP or CPU continuation retains its original instance receiver.
Local result text, including a conditional multiline summary, must reach that
instance's rendered view after completion. Navigation writes the owning App's
root route and back-request slots; route parameters and history are published
by the same completion hook as synchronous handlers. Handler-driven pushes and
host link transitions both enter the bounded return history; returning consumes
history without adding the outgoing route again.

Unmount cancels continuations owned by the released receiver before a fresh
mount can inherit its textual identity. A late HTTP completion or CPU slice
cannot redirect the surviving root. The permanent
`plan068_routed_continuations` integration target exercises root controls,
synchronous/HTTP/CPU navigation, push then back, conditional results, Store and
App isolation, and unmount/remount. Actual Reader acceptance remains with its
consumer Plan and is not replaced by these component-level checks.

## Route-decorated instance identity and dispatch resolution (PLAN-069 SD02)

- An outlet page instance's identity carries its route as a decoration
  (`Widget@/book/7/chapter/1`; the root route renders as `Widget@/`). Nested
  child identities keep the ordinary `<parent>/<Child>#<key|slot>` shape and
  may embed a routed parent prefix.
- Widget-type resolution for dispatch MUST use the type registered with the
  instance, never a suffix parse of the identity string. A route tail (`"1"`,
  or `""` for `/`) is not a widget name: deriving the type from the identity's
  last path segment made every routed page unresolvable by name, so name-only
  dispatch and identity-miss retarget fell back to root state — where page
  model fields (including route-parameter copies such as `.route_id`) do not
  exist and read empty.
- Name-only dispatch and identity-miss retarget resolve by exact registered
  type with the existing cardinality rule (exactly one live instance → that
  instance; zero or several → root legacy / drop). A released identity is never
  retargeted. `child_instance_identities_of_type` and
  `child_instance_widget_name` are the shared resolution surface; diagnostic
  and auxiliary registries (input cache keys, native accessibility retarget,
  reload schema lookup) use the same rule.
- A page's Init writes its route parameters onto the page instance; later
  handlers of the same page read those fields. A handler that rebuilds a route
  from them (`router.push("/book/" + .route_id)`) therefore addresses the same
  book; Back-style chains must not degrade to the bare prefix (`/book/`).
- Synthetic dispatch (MCP press/action) uses the same instance resolution as
  real input: a carried instance identity is authoritative; otherwise the
  type-unique live instance receives the event. Route commits are observable
  through `__current_route` and the mounted page instance.
- Permanent fixtures: `plan069_route_dispatch_tests` — loop card click commits
  the route and the target page reads the same param; name-only dispatch
  reaches the routed page instance; the Back chain builds the parameterised
  route and mounts the detail page; an identity-carrying control is unchanged.
  Pre-fix red proof: all four fail, Back produces exactly `/book/`.
