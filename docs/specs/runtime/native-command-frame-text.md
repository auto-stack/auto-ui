# Native command-frame text

Status: current contract, auto-ui/PLAN-060.

The headless iced recorder converts tiny-skia layers into the existing desktop
command frame formats. A live Paragraph or Editor carries a transformation on
the text record itself; its Item wrapper may report identity. Cached or grouped
items may additionally carry a parent transformation. Text lowering composes
the parent and text-local transformation exactly once.

For the existing translation path, v1 emits translated text coordinates. V2
emits its existing Transform/TextStyled/TransformPop operations with local run
coordinates. Scrolling must therefore move text to the same viewport position
as the interactive widget that produced it. A visible bottom button must remain
clickable at that location after scrolling.

Soft-wrapped text emits each visual run's glyph-cluster substring, preserving
UTF-8 boundaries. It does not repeat the complete logical line for every visual
run. Frame lowering preserves the established v2 axis-aligned scale/translation
support; existing rotation/shear, raw-text and mesh limitations are not expanded
by this repair.

Verification: `p060_native_scroll_reaches_bottom_and_replays_coordinates`
drives a real native scrollable component, checks v1/v2 coordinates, replays the
wire list through the host canvas painter with a non-identity parent transform,
and clicks the bottom button. `p060_wrapped_text_emits_only_each_visual_run`
covers Latin/CJK soft wrapping. Application receipts additionally require a
real adopted window and its durable data oracle; unit or queued-input success
alone is not an application task pass.

For an adopted command-frame window, Hello's width and height are the requested
client extent. The desktop adds its existing chrome and bounds the initial outer
rectangle to its usable work area. The WM rectangle remains authoritative for
later layout, resize and maximize changes. Its derived client extent is sent
over the existing primary-surface `FrameMsg::Resize`, once per changed extent;
the host records it only after a successful send. Pixel buffers and the shell's
separately declared projection surfaces retain their existing negotiation.

The app endpoint accepts positive finite extents for its own primary surface,
then forwards the existing resize lifecycle control to its frame source.
Headless and projector sources invalidate their frame while retaining the same
component and widget state. Repeated, invalid or foreign-surface frame resize
messages have no effect. This adds no protocol tag or capability; older clients
that ignore this existing notification retain that limitation until rebuilt.
`p060_broker_viewport_follows_actual_wm_client_once` covers actual broker adoption,
bounded initial geometry, real-pipe delivery and maximize deduplication, with a
pixel-mode control. The native scroll regression consumes a real endpoint resize
before checking frame coordinates and clicking the visible bottom button.
