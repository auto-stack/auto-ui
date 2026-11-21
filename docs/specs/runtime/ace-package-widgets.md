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

## Chart geometry and interaction

Checked package views can render SVG chart geometry built from the current
widget model. Each mounted chart keeps its pointer and legend state within its
own widget instance. Child event bindings deliver the declared payload to
their parent instance. Pointer leave clears the transient hit and keeps the
legend selection. A disconnected view discards late input.

A `mouse-area` with a finite, positive `coords: WxH` uses that logical extent
for pointer coordinates. The Vue projection scales the pointer from its DOM
box to the declared extent. In an absolutely positioned element with `coords`,
arbitrary pixel width, height and edge-offset utilities are resolved against
the declared extent. This keeps the FlowDiagram hit boxes and tooltip aligned
with its SVG view box when the browser resizes the widget.

An AreaChart or BarChart with no rows produces no data geometry. A subsequent
nonempty input regenerates the paths or bars. The DonutChart empty state keeps
its neutral ring, and LineChart draws no series for an empty input.
