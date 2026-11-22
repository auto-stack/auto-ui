# ACE package widget projection

The AV desktop package bridge and the A2Vue package SFC generator consume the
checked AGE view tree. The interactive Widgets Gallery previews use their
original page sources. The desktop bridge projects these previews to Iced
controls. A2Vue projects the same source tags to Vue and DOM controls.

| Source controls | Behavior |
|---|---|
| `accordion`, `accordion-item`, `accordion-trigger`, `accordion-content` | A single-selection group opens at most one item. Opening another item closes the previous one. A collapsible group can close its active item. |
| `collapsible`, `collapsible-trigger`, `collapsible-content` | The trigger updates the open value. The content is visible while open. The trigger exposes the same expanded state. |
| `scroll` and bounded elements with `overflow-y-auto`, `overflow-x-auto` or `overflow-auto` | The viewport scrolls on its declared axes. Moving inside the viewport moves its own content. |
| `pagination` navigation and item slots | Navigation uses the declared numbered items and previous/next controls. The current page starts at 1 by default, respects the last page derived from the total and page size, and is reflected on the matching visible page item. The existing ellipsis stays in place. |
| `progress` | The value is readable and the visible fraction uses the declared maximum, defaulting to 100. |
| `slider` | The value uses the declared minimum, maximum and step. Pointer and keyboard updates respect those bounds. The default range and step are 0 to 100 and 1. |

AV keeps the preview state with the live package session and widget instance.
A routed Vue page keeps its state for the duration of its mount. Closing a
session and reopening its original entry restores its initial controls.

For the Windows Iced preview window, UI Automation exposes disclosure states,
pagination's current page, the read-only progress value and the slider's
current value and range. Closed disclosure contents leave the focus order and
accessibility tree. Actions are revalidated against the active session and
its current controls. The equivalent Vue state uses DOM
ARIA and the native HTML range and progress properties.

The VM [PaginationControl](../ui/pagination.md) is a separate higher-level
package. [ACE route-page loading](ace-route-page-loading.md) defines the
package page lifetime.

## Routed layout previews and built-in controls

The layout previews use the authored view tree. Grid columns, gaps and spans
resolve against the current viewport. Responsive visibility applies before
native controls enter the focus list. A hidden native subtree and a display
element with no text value preserve the alignment of following AV observations.
Fixed native preview sizes respect an authored minimum size. Native containers
apply negative margins before sibling painting in source order.

Literal composite pages use the registered controls of the destination
renderer after AGE checks the view. The Iced page keeps disclosure and input
state with its own session. Select displays the authored placeholder before
a choice and after a new page mount. The Vue page keeps those states in its
mounted controls. A sidebar used as a preview occupies a viewport inside its
page, so it does not capture the shell's navigation.

Native editors have distinct keys per page session, widget instance and
source element. Closing a page releases those native editors and its public
ImageSurface tickets. Checked local image references resolve within the
canonical front public directory. An unresolved or out-of-root reference
shows the missing-image state. ImageSurface consumes a media ticket for a
public raster. The Vue product copies public files into its build.

## Chart geometry and interaction

Checked package views can render SVG chart geometry built from the current
widget model. Each mounted chart keeps its pointer and legend state within its
own widget instance. Child event bindings deliver the declared payload to
their parent instance. Pointer leave clears the transient hit and keeps the
legend selection. A disconnected view discards late input.

A `mouse-area` with a finite, positive `coords: WxH` uses that logical extent
for pointer coordinates. Each host scales pointer coordinates from its hit
box to the declared extent. In Iced, an absolute mouse area pinned to every
edge at zero fills its enclosing stack along any axis without an explicit
size. For a moving pointer with logical coordinates and no explicit size,
the hit rectangle uses the SVG's centered aspect-preserving fit. Moving into
the surrounding margin leaves that hit rectangle. In an absolutely positioned
Vue element with `coords`,
arbitrary pixel width, height and edge-offset utilities are resolved against
the declared extent. This keeps the FlowDiagram hit boxes and tooltip aligned
with its SVG view box when the browser resizes the widget.

An AreaChart or BarChart with no rows produces no data geometry. A subsequent
nonempty input regenerates the paths or bars. The DonutChart empty state keeps
its neutral ring, and LineChart draws no series for an empty input.

A checked Iced route shell receives a bounded window viewport. Its authored
scroll surfaces, including `sidebar_content`, constrain the viewport and
let their contents scroll. The shell's responsive visibility controls apply
to the sidebar and mobile navigation.

## Gallery trees and File Manager controls

The original `/filetree` page publishes nested nodes and their badge labels.
Each branch owns its expansion state. The original `/treeview` page receives
controlled expansion and selection values and publishes the checked node ID.
Closing and reopening either page restores its authored defaults. Events
from an earlier mount are discarded.

The native File Manager uses a window-bounded fullscreen root, with authored
scroll regions and a responsive grid. Vue keeps the checked model and its
event targets for each mounted instance. Child widgets publish typed actions
to their current parent. Focus requests apply to the control in the committed
frame.

The standalone Windows Iced host uses the original pointer-message time and
the Windows double-click interval. The modal overlay handles each press
before the underlying view receives any unconsumed press. A queued double
click can survive a selection redraw if it still identifies the same keyed
item, instance and checked action.
