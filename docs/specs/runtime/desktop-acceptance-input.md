# Desktop acceptance input

Status: current contract, introduced by auto-ui/PLAN-060.

The acceptance tools run inside the desktop event loop. Pointer injection uses
the normal hit-test, focus and owning-client routing; keyboard injection uses
the normal focused input route. Command-frame coordinates are logical client
coordinates: subtract the desktop title bar once, without scaling by a window
resize ratio. Shell surfaces do not have this title bar.

For an out-of-process app, `msg` and root `handler` injection send
`InputMsg::Action { wid, name, arg }` (input tag 11) to the owning broker client.
The name is a root component message variant; the supported payload is absent
or one string. `widget: App` targets the root. Other widget names are explicitly
unsupported for broker actions; they must not silently target the root.
In-process handler/widget dispatch keeps its existing semantics.

For an in-process root, `msg` additionally accepts `args`, an array of string,
boolean, integer or floating-point scalars. It is mutually exclusive with the
legacy `arg` string. Numbers use the existing VM handler ABI (signed 32-bit Int
and single-precision Float); integers outside that range, null and nested
containers are rejected before queueing rather than truncated. The desktop update
loop delivers these values through `DynamicMessage::Typed`, preserving normal
asynchronous event handling. Broker-owned targets reject this array form without
sending a wire message or executing their local placeholder. This adds no
arbitrary native subwidget RPC or new wire payload.

The Hello capability tail retains its historical `theme_caps` field name and
advertises `input-action/1` in addition to any theme capability. A host must not
send tag 11 to a peer missing that exact capability. An unsupported or failed
broker send must not fall back to executing the desktop's placeholder component.
Old peers continue to receive their existing pointer and keyboard input.

Native HeadlessFrameSource and RqProjector dispatch Action directly to the
component, because Action has no iced event representation. VM sources dispatch
into the component event loop so asynchronous handlers can park and resume.
Handled native actions advance the frame revision. Generated native components
dispatch zero-argument and single-string variants; input variants first stage
the supplied string for generated oninput handlers that read input text.

Acceptance results require a changed application frame and an independent API
or durable-store readback where applicable. A queued input or successful socket
write alone is not a business-task pass. Unsupported peers and nonexistent
targets remain failures, not successful no-ops.

Closing a broker window requests the owning endpoint's Close before removing
the desktop window. The normal exit/buffer-release handshake reclaims the
child; in-flight frame and bitmap acknowledgements are permitted while closing.
Reopening creates or adopts a live unclaimed window, never a stale client's
already removed window. Unrelated app clients remain alive.

Verification anchors: runtime `p060_` tests (real native component input/render
and negotiated named-pipe traffic), endpoint closing-ACK regression, and the
real-child reopen/close lifecycle regression. Consumer task receipts bind the
tested executable, generated receipt and Cargo artifact by SHA256.
