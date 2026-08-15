# Scaling and Viewer Zoom

This document describes the current behavior. The old global zoom prototype
was renamed to **scaling** because it changes the entire egui application.

## App scaling

`MarkdownApp::scaling` is persisted in `PersistedState` and applied with
`Context::set_zoom_factor`. It affects menus, sidebars, tabs, and the window
chrome. Ctrl++ / Ctrl+- change it in 10% steps and Ctrl+0 restores 100%.

## Viewer zoom

`MarkdownApp::viewer_zoom` is separate from app scaling and is applied only to
the Markdown viewer:

- The viewer runs inside a scoped egui style whose text sizes and spacing are
  multiplied by the viewer zoom.
- `CommonMarkViewer::content_scale` scales its layout widths as well, allowing
  wide content to remain pannable instead of shrinking the application UI.
- egui's multiplicative `InputState::zoom_delta()` provides smooth native
  touchpad pinch and Ctrl/Cmd-wheel input. Input is accepted only while the
  pointer is over the viewer, so sidebars and menus are unaffected.
- The current document position under the pointer is preserved using the old
  and new content extents on both axes.
- The viewer exposes both scroll axes. Holding the middle mouse button and
  dragging updates those offsets while zoomed in; middle-click is not treated
  as a Markdown link activation.

The viewer zoom is persisted between sessions and can be reset from **View →
Reset Viewer Zoom**. Its supported range is 50%–300%.

## Validation

The pure scroll-anchor helper is unit-tested. The integration path is covered
by the normal Cargo test, format, Clippy, and package checks; interactive
verification should include pinch, Ctrl-wheel, pointer anchoring, middle-drag,
and ordinary scrolling at 100% viewer zoom.
