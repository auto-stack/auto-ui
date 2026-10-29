# VM route-page control

Run an exported copy with `auto run -r vm`, pointing `AUTO_UI_TOOL_BIN` to the
tool being tested. This fixture has no backend or durable application data.

The Author page combines imported styles, a timer, conditions, a list, string
interpolation and Init. Its initial native snapshot must contain `Route Init
complete`, `entry first` and `entry second`. Use the live snapshot's Home,
Broken and Author controls to navigate. Home must complete Init; Broken must
report `VM_ROUTE_PAGE_LOAD_ERROR`, `module=absent`, `stage=resolve` and its
source file, while navigation back to Author remains usable. The absent page
is intentional. Send `q` to the owning CLI to close the run.

Before PLAN-015, the imported style causes Author to be silently omitted and
the outlet displays `page author not loaded`. Removing only the style import
and its input is the accepted control: all other features work on that same
old tool. The runtime's `plan015_route_loading_tests` exercises this fixture
through the production builder, checks rendered Init effects and timer
registration, and covers named missing/read/parse/extraction failures.
