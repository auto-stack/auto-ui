# FileDialog

`packages/file-dialog` is the official application-modal file selector for
Windows and Linux native VM apps. It lists local directories through an
App-owned asynchronous service. It does not open a system file dialog or
replace `dialog_open_file`, `dialog_open_files`, or their existing providers.

Import the package from a source-relative or absolute directory:

```auto
use { package: file_dialog from "path/to/file-dialog" }
```

The package contains `package.at`, `file_dialog.at`, and a `pac.at` dependency
descriptor. For Vue source generation, either use the supported
`src/front/components` layout or declare a path dependency on an unchanged
`vendor/file-dialog` snapshot. The latter keeps unrelated project components
outside the native package import. Generating a Vue component does not
grant browser access to native directories: the IO mailbox is a native VM
capability. Browser consumers keep their user-authorized File/byte-token path.
Generated Rust source compatibility does not claim a Rust-host directory
service; the delivered interactive selector host is native VM.

## Public interface

| Property | Default | Meaning |
|---|---|---|
| `open` | `false` | Parent-controlled visibility request |
| `title` | `选择文件` | Visible dialog title |
| `initialDirectory` | empty | Initial directory or complete file path; empty uses home |
| `selectionMode` | `singleFile` | `singleFile`, `multipleFiles`, `singleDirectory`, or `saveFile`; unknown values are named errors and never fall back to an open mode; the mode freezes at session open and parent changes mid-session never switch the live session |
| `defaultFileName` | empty | Save-mode initial filename draft; an explicit non-empty value wins over the basename prefill of a complete-file `initialDirectory` |
| `defaultExtension` | empty | Dot-less; completes the accepted result name only when the draft lacks an extension; never rewrites the typed input; incompatible with the active group is a non-retryable configuration error |
| `filters` | `[]` | Records `{id,label,extensions}`; extensions omit the dot |
| `activeFilterId` | empty | Initial configured group, otherwise the first group |
| `acceptLabel` | `打开` | Confirmation label |
| `showHidden` | `false` | Include hidden/system and dot-hidden entries |

`accepted(str)`, `cancelled`, and `error(str)` are ordinary component events;
bind them with `onaccepted`, `oncancelled`, and `onerror`.

- `accepted` carries JSON `{"files":[{"name":"…","path":"…"}]}`. The
  array is nonempty, contains unique absolute OS paths, and follows stable
  directory-list order. Accept closes the session. Both new modes return
  exactly one entry: directory mode returns the canonical directory (root
  included) and save mode returns the exact target path, which may not exist
  yet; Windows canonical parents may carry a `\\?\` prefix.
- `cancelled` carries no data and closes the session. Cancel, close, Escape,
  and a parent closing an open request share the cancellation path.
- `error` carries JSON `{code,message,retryable}`. Recoverable failures keep
  the dialog open. Codes include `invalid_path`, `not_found`,
  `permission_denied`, `io_error`, and `unsupported_path`. Cancellation is not
  an error; an unreadable directory is not an empty listing.

Consumers parse JSON and preserve the returned OS path representation. In
particular, Windows canonical paths may have a `\\?\` prefix. Paths are not
shell expressions and must not be split on spaces or expanded as commands.

## Directory and selection rules

Home and local roots, parent navigation, refresh, path entry, directory
entry, name/type filters, hidden entries, selection, clear, paging, retry,
cancel, and accept all have ordinary rendered controls.

An absolute path or a path relative to the current directory can be entered.
Enter and “转到” perform the same navigation. A complete file path navigates
to its parent and preselects that file if eligible; it never accepts the
file without confirmation or bypasses the active type group. Empty input is
an error. Invalid initial input stays recoverable through home/path entry.

Directories sort before files. Names sort deterministically, with original
name/path as tie breakers. Extension matching and filename-substring search
are case insensitive. Both filters retain navigable directories. No group
means all ordinary files. Windows hidden/system attributes and Unix dot
names are hidden by default; the explicit hidden switch reveals them.

Selection is confined to the current listing. Successful directory changes,
filter changes, and refresh clear it; failed navigation preserves the previous
valid directory and selection. Single mode retains at most one file. Multiple
mode uses each row's checkbox and needs no keyboard modifier.

In directory mode only directories are selectable: each directory row keeps
its enter action separate from a select action, a dedicated control selects
the currently listed directory (root included), file rows render as context
without selection affordances, and the type-group buttons are hidden without
excluding directories from name filters. In save mode nothing is selectable;
the filename input is the draft.

Accept is disabled without an eligible selection. Before emitting, a worker
rechecks every selected path for existence, ordinary-file status, readability,
active type eligibility, and canonical identity. One failure prevents the
entire acceptance; no partial accepted array is emitted. The user can adjust
selection or retry. File contents, publication formats, limits, duplicate
business records, and managed copies remain the consumer's responsibility.

Symlinks resolve once during explicit navigation; no recursive scanning or
cycle traversal occurs. Broken links are named failures. Unix non-UTF-8 names
produce `unsupported_path`, rather than returning a different lossy path.
UNC/network paths may be attempted through OS APIs; network discovery and
timely completion of unreachable network operations are not promised.

## Save-target rules

The save-mode filename is a basename only: path separators, `.`/`..`, NUL,
and Windows reserved device names are named recoverable `invalid_path`
errors; the draft is never trimmed or rewritten and an empty name disables
accept without an error event. Extension completion happens at confirm time
in the accepted result only; an explicit extension outside the active group
is a recoverable error and switching groups never rewrites the input.

Destination resolution runs on a worker against the canonical parent
directory and never creates, truncates, or renames anything, including
permission probing: existing-target writability is read from metadata
attributes only. A symlink target is `unsupported_path`; a directory target
is a recoverable `invalid_path`; special files are `unsupported_path`; a
read-only regular file is a recoverable `permission_denied`. The final write
permission and the write itself stay with the consumer app.

Overwrite confirmation always asks, shows the real target path, and binds
the confirmed file identity (length + modification-time fingerprint).
确认覆盖 revalidates the same session target: the same identity accepts, a
vanished target accepts as a create, and a replaced or retyped identity
re-presents the layer so an old confirmation can never hit different bytes.
Back keeps the drafts and the session; any directory/name/listing change
voids a pending confirmation. The layer is a nested controlled modal of the
same app: Escape and the close path return to the selector, outside clicks
are swallowed, and background actions are rejected at both the modal scope
and the model. Acceptance is a selection and an intent only; TOCTOU handling
stays with the consumer app.

## Ownership and asynchronous lifetime

Each actual component instance owns one `DialogModel`, selection, session
nonce, request generation, and private command/view/event mailbox. Parent
pages receive terminal events and do not maintain a second selection algorithm.
The private `fd_*` fields are not application or automation APIs.
Row identities carry a JSON-escaped path as opaque data, so valid filename
control characters cannot become message separators or alias another row.

A parent closing and immediately reopening after a terminal callback must not
need an extra timer frame. The host retains closed props observed in rendered views and event boundaries;
terminal callbacks remain single-shot, including when the parent closes first.


Workers receive owned values and cancellation flags, never VM objects. A new
request supersedes the previous receiver. Late listings/validation, closed
sessions, stale row tokens, and released owners cannot change a later session
or publish acceptance. Accepted/cancelled are mutually exclusive and occur at
most once per session. Instance release and app reload drop the owned service.
Timer dispatch addresses actual instances, including same-type siblings.

Only the current directory is scanned. The complete result is retained and
published in pages of at most 100 entries, without truncating later pages. The
local acceptance fixture has 1003 long Unicode names: every entry must be
reachable and the last one selectable. Its directory-ready budget is 2 seconds;
ordinary MCP action acknowledgement is under 1 second. This is a bounded local
fixture promise, not an arbitrary-size/network-directory latency guarantee.

## Modal, focus, and automation contract

The dialog uses the current App theme. Its content sets an 800px preferred
width, clamped to the viewport, with a title, compact navigation/path toolbar,
locations sidebar, and a main file list. Name/type filters sit above the list;
hidden-file and paging controls sit below it. The file list and local roots
scroll independently, while selection status and cancel/accept remain in a
fixed footer. File rows show type icons, a theme-derived selected background,
and readable B/KB/MB sizes. Checkboxes occupy their own fixed-width column;
the filename remains their semantic name, and is drawn beside the file icon
in the separate name column. Parent/refresh are icon-only buttons with named
hover tooltips. Filename input and type-group buttons share one toolbar row.
Hidden-file and pagination controls share one status row. Pagination is the
shared controlled `PaginationControl` package (PLAN-086; `packages/pagination`,
public 1-based page, contract in docs/specs/ui/pagination.md): the dialog binds
`page = displayPage` (host model page + 1), `pageSize = 100`,
`total = visibleFileCount`, and translates `changed {page}` into the existing
`fd_command` `page {index: page-1}` operation — the host model stays the sole
owner of 0-based paging, slicing, filter/selection resets. The entire pagination
group is absent when there is only one page (including empty results); the
package carries the same hide-when-single default. The package dependency is
part of the delivered closure: consumers vendor both packages byte-equal
(`packages/pagination` alongside `packages/file-dialog`) and declare both roots;
a missing dependency is a named build failure.
A disabled accept button is visibly muted.
The sidebar and list use compact dimensions below the small-screen breakpoint.
Narrow windows must retain reachable cancel/accept controls.
Native Dialog content `style`/`class` overrides its default panel chrome in
both ordinary and tracked views; dialogs without an override keep their
existing dimensions. Explicit-modal reopening resets retained
content scroll/focus state so the initial path input is visible again.

Opening focuses the path input; save mode focuses the filename input, once
per session through the shared focus builtin. Tab/Shift+Tab traverse enabled inputs,
buttons, and checkboxes within the active modal and wrap at both ends. Enter
performs the focused control's declared action; Space activates its bound
button/checkbox handler. Escape cancels. Outside clicks do not close or reach
the background; losing window focus does not dismiss. Closing restores the
original trigger if present, otherwise an available App control. Delayed
focus operations are guarded against newer modal sessions. Windows semantic
focus uses the same actual control state. System accessibility removes or
disables background actions while a modal is active, and revalidates its live
scope when a queued action is consumed. Closed content and actions from a
different initialized VM owner are rejected; heap IDs alone are not App IDs.

Automation uses ordinary `autoui_snapshot`, `autoui_find`, `autoui_action`,
`autoui_type`, keyboard, scroll, and screenshot tools. No API injects selected
paths or fabricates callbacks. While the modal owns input, background targets
and shortcuts are rejected. Closed content is not an actionable tree.

Current directory plus each original row name identify the visible path.
Checkbox checked/enabled state and button disabled state are published.
Action-node IDs bind the actual instance/handler and row session/revision/path;
reusing a list position must not let an old ID select a different file. View
and snapshot publication races also revalidate the binding before dispatch.
Clients discover IDs from the current tree, rather than retaining them across
navigation, filtering, paging, refresh, or reopen.

## Verification

- Runtime `file_dialog` and `file_dialog_model` integration targets exercise
  package/VM callbacks, filters, atomic validation, async/stale lifetime,
  instance isolation, OS path failures, and complete paging.
- Tools `file_dialog_component` exercises official-source generation and the
  supported project package layout. Old `plan804_dialog_dnd_provider` tests
  retain the system API contract.
- `tests/product/verify_file_dialog.py` starts the real CLI with a pinned UI
  tool, discovers its owned native window, drives ordinary controls, checks
  durable callback identity, tests background/stale failures, single mode,
  1003 entries, themes, ten cancel/reopen cycles, process/port cleanup and a
  fresh successful session, and covers all four modes: directory selection
  (current and child, files rejected), save names with completion and named
  errors, overwrite Back/Replace with byte baselines, and fixture bytes
  unchanged for the whole campaign. Run Windows and Linux separately; missing
  runners, empty trees, missing windows, and timeouts are failures, and a
  missing Linux runner is recorded `not_run`, never a pass. Inspect captured
  screenshots, including a narrow window, for actual visible footer/content
  and single-line type groups.
- `tests/product/verify_file_dialog_modes_consumer.py` consumes the dialog
  from the real 041 editor through ordinary controls: Ctrl+S opens the save
  dialog for an untitled tab, 另存为 prefills the current basename, the app
  writes the accepted path and verifies the bytes read-back, cancel keeps the
  dirty state without creating files, a read-only target is a named
  recoverable failure, and quit goes through the documented confirm path with
  no Env path shortcut present.
- Consumer acceptance (Reader book/cover bytes, limits, database state and Vue
  authorization) remains in the consumer's own specification and product gate.

Native evidence follows
[native-vm-app-validation](../runtime/native-vm-app-validation.md). Internal
model/headless tests and successful cross-compilation do not prove a native
window or the Windows system accessibility provider.
