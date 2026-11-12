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
| `selectionMode` | `singleFile` | `singleFile` or `multipleFiles` |
| `filters` | `[]` | Records `{id,label,extensions}`; extensions omit the dot |
| `activeFilterId` | empty | Initial configured group, otherwise the first group |
| `acceptLabel` | `打开` | Confirmation label |
| `showHidden` | `false` | Include hidden/system and dot-hidden entries |

`accepted(str)`, `cancelled`, and `error(str)` are ordinary component events;
bind them with `onaccepted`, `oncancelled`, and `onerror`.

- `accepted` carries JSON `{"files":[{"name":"…","path":"…"}]}`. The
  array is nonempty, contains unique absolute OS paths, and follows stable
  directory-list order. Accept closes the session.
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

## Ownership and asynchronous lifetime

Each actual component instance owns one `DialogModel`, selection, session
nonce, request generation, and private command/view/event mailbox. Parent
pages receive terminal events and do not maintain a second selection algorithm.
The private `fd_*` fields are not application or automation APIs.
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

The dialog uses the current App theme, with a scrollable body and separate
footer. Local roots can scroll horizontally. Narrow windows must retain
reachable cancel/accept controls. Explicit-modal reopening resets retained
content scroll/focus state so the initial path input is visible again.

Opening focuses the path input. Tab/Shift+Tab traverse enabled inputs,
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
  fresh successful session. Run Windows and Linux separately; missing runners,
  empty trees, missing windows, and timeouts are failures. Inspect captured
  screenshots, including a narrow window, for actual visible footer/content.
- Consumer acceptance (Reader book/cover bytes, limits, database state and Vue
  authorization) remains in the consumer's own specification and product gate.

Native evidence follows
[native-vm-app-validation](../runtime/native-vm-app-validation.md). Internal
model/headless tests and successful cross-compilation do not prove a native
window or the Windows system accessibility provider.
