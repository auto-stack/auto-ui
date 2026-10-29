# PLAN-015 root cause and bounded repair

Baseline: auto-ui `45b196e03ce5978a095cf6fd0af940240ca7dc56`;
AutoLang `eb6ecc937ce25546b46f4da510e669396ab78463`, AutoDown
`c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. Authoritative tree:
`D:/autostack/.wt/ui-015/auto-ui`, `codex/plan-015-dev`. Dependencies are
detached group siblings, unchanged. No junctions are used.

## Cause

The explicit route-page branch of `dynamic_build.rs` parsed page source without
`prepare_style_recipe_imports`. The root and ordinary imported-widget branches
already call it. A page using `use fields: route_field` and `style: route_field`
therefore fails its identifier check. Both the route parser's `if let Ok` and
widget extractor's `if let Ok` discard failure. `aura_view_builder.rs` then
replaces the missing registry alias with `<outlet: page author not loaded>`.
Resolve and read failures also silently continue. Original source anchors:
route loader 1386-1409, swallowed extraction 1423-1434, outlet 6693-6697.

This is an H2 parse/preparation failure that causes H1 registration absence;
the module name `author` is not special. Blog's author module imports
`stylekit.styles: input_field` and consumes that recipe, unlike its home control.
The shared API/backend is outside this failed boundary.

## Distinguishing evidence

- A test-only baseline executable compiled from the baseline production code
  fails `plan015_route_imported_style_is_loaded_before_page_parse` with the
  actual placeholder (exit 101). `red-style.log` SHA256
  `d73c38ecd78c313c35f28f6ac56c0709fae43d9351877a6fa2123d6503b2efe6`.
- Its missing-page counterexample also fails (exit 101), with the same silent
  placeholder; `red-missing.log` SHA256
  `d5ee396066f8ec0725e6088d6b0a2c9d392b2f189120b1ff4ca2571e2e509706`.
- Existing tool SHA256 `7150310017a4410f57d94920f329b8cc94793522d308e0053f7015f353b1f734`
  reproduces the Author failure in a visible owned native window through the
  actual CLI and MCP, while Home Init completes. `native-red-2/report.json`
  SHA256 `b500d120fe20b4db48886b2adba98c54727cfeb29f4f83d405a0a4a483f3de9d`.
  This installed binary is a runtime observation, not a claim of exact Git
  provenance; the baseline unit executable supplies the source-bound red.
- Removing only the imported style/input from that same native fixture makes
  Author Init, timer, conditions, list and interpolation work on the old tool.
  The log/snapshots in `native-old-features-control` establish this accepted
  control. Its original summary's `home_and_negative_and_return` key is overly
  broad: control mode did NOT exercise the negative page; only the snapshots,
  author/Home navigation and exit are used as evidence.
- Patched tool SHA256 `4d7a40524228004dcad772df9b0ab201220e76dc158b17166adb255fff2fd97c`
  loads the original full fixture, completes Author/Home Init through the frame
  pump, navigates to a named missing-page diagnostic, and returns to Author.
  An owned visible native window and clean CLI `q` exit 0 are recorded.
  `native-green/report.json` SHA256
  `fb924e42452200aa30949b9287ac9ae9803eb6515d08b4f886e002e1e233902d`.
- `cargo test -p auto-ui-runtime --lib plan015 -- --nocapture --test-threads=1`
  passes 6/6, zero ignored. It asserts rendered Init effects, list values,
  mounted/subscribable 3000ms timer, Home, missing page, return navigation,
  unmatched route, undefined-style parse error, invalid UTF-8 read error and
  a module with no widget. `fixture.log` SHA256
  `633e903f0a31c2d3dd36a60061aeee7ddf22e13f5ce6cc147386eb2b412a21cc`.

An initial malformed-EOF negative probe stalled in the existing parser and
was terminated by its exact owned test PID. It was replaced with a complete
undefined-style input that exercises this route error boundary without relying
on an unrelated parser recovery bug. Initial test compile errors and one MCP
startup poll race were corrected; those logs are retained, not counted as
behavioral passes.

## Repair and error design

Prepare page recipe imports before parsing, validate imported/local recipes
before extracting the view, then validate all widget extractions before
publishing registry aliases. Preserve source-prefix overlays, page fragment
registration and dependency/store collection. Keep load failures in the same
registry as route aliases, emitting `ERROR: VM_ROUTE_PAGE_LOAD_ERROR` once at
load time with route/module/file/stage/reason. Outlet snapshots show that
diagnostic; valid sibling pages remain usable. Missing registrations have an
explicit registration-invariant diagnostic. Only unmatched routes retain the
`no route for` placeholder.

This is an in-contract local repair. No Blog application source, Vue renderer,
language parser, acceptance threshold or production installation is changed.
SD-01 remains proposed until final review and merge. The native single-page
control and consumer profile are separately owned closing evidence; these
fixture passes do not substitute for BLOG-004's pending native author task.
