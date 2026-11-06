# Windows VM system accessibility

Proposed contract: auto-ui/PLAN-068. Applies to standalone Windows VM windows
created by `run_dynamic_iced` and its shared dynamic rendering path. It does
not grant accessibility support to unrelated static hosts or other platforms.

## Provider and ownership

The window thread installs the Windows AccessKit subclass adapter before the
first show. Hidden and pixel-capture windows retain their visibility intent.
The provider obtains semantics from the production converted view, including
the children created during that frame. It does not require F12, an inspector
capture, an MCP client, or a prior render.

Each actionable node retains the original app root, complete child instance
identity, and receiver heap ID. Removed node IDs are never reassigned. Actions
are queued across the platform boundary and revalidated when the owning UI
thread consumes them; closed windows, released receivers, disabled controls,
and controls without a visible measured rectangle reject actions. No VM object
crosses the platform adapter boundary. The queue is bounded at 128 requests;
overflow is rejected with a diagnostic. Wakeups are event driven.

An instance identity is opaque. In a widget message carrier, split the U+0001
instance separator before interpreting an optional `@sequence` in the widget
type. Route slashes and `@` characters inside an instance identity are retained.

## Semantic mapping

| Converted view | Windows semantic data | Actions when bound and enabled |
|---|---|---|
| Button; clickable Row/Column | Button, visible label | Click, Focus |
| Input | TextInput, placeholder label, current value | SetValue when a change handler exists; Focus |
| Textarea | MultilineTextInput, placeholder label, current value | SetValue when a change handler exists; Focus |
| Text/Rich | Label, rendered text | None |
| Checkbox | CheckBox, label, toggled state | Click when a toggle handler exists; Focus |
| Radio | RadioButton, label, selected state | Click when a select handler exists; Focus |
| List | List, converted children | None |
| Other containers | GenericContainer, converted children | None |

Password inputs do not expose the text value. A missing handler does not gain
an invented action. Select, custom controls, and unsupported Aura conversions
are not promoted to supported interactive roles by this mapping. In particular,
a Radio mapping in the abstract renderer does not establish a public Aura tag.

Bounds come from the actual draw layout clipped to the viewport, then use the
window scale for physical coordinates. A node without measured visible bounds
is hidden. Input focus is read from the Iced widget tree. Button focus uses the
same widget operation path and Enter/Space activate its original bound event.
Reusing a layout slot for a different node resets its interaction state; focus
must not transfer to another instance. Edit-display caches use receiver ID plus
event and are pruned when that receiver is released.

The WebView2 document surface retains its own Windows provider. Its document
text and real selection require direct system-tree evidence; parent semantics
and DOM ARIA are not substitutes.

## Verification boundary

Permanent mechanism cases live in
`ui::iced::renderer::native_accessibility::tests` and
`tests/plan068_routed_continuations.rs`. They cover production layout, owner
and generation checks, routing carriers, disabled rejection, focus-slot reuse,
first-frame children, closure, cache isolation, and abstract selected roles.
They do not substitute for a live Windows system-tree or real input check.

The live fixture is `examples/plan068_system_accessibility.rs`. Record source
and binary revisions, actual role/value/focus observations, accepted actions,
and unmount/remount evidence separately from MCP actions. Consumer acceptance
also follows [native-vm-app-validation](native-vm-app-validation.md); pending
system/input legs cannot be reported as passed from internal snapshots.
