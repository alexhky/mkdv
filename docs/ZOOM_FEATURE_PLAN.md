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
- Three input sources feed the zoom, in priority order: touchpad pinch
  (`src/pinch.rs`), native `Event::Zoom`, and raw Ctrl/Cmd-wheel deltas.
  Input is accepted only while the pointer is over the viewer, so sidebars
  and menus are unaffected.
- Touchpad pinch does **not** come from winit. winit has no gesture backend on
  Linux, so `src/pinch.rs` binds `zwp_pointer_gestures_v1` itself on winit's own
  Wayland connection. X11 has no gesture protocol and falls back to Ctrl+wheel.
- Zoom snaps to a geometric ladder (`VIEWER_ZOOM_STEP`, 2% per level). Each
  distinct zoom forces a full re-wrap plus glyph re-rasterization, so limiting
  the number of levels is what keeps a pinch smooth. Gestures accumulate into
  an unsnapped `viewer_zoom_target`, because one pinch update is far smaller
  than one ladder step.
- Wheel and touchpad scrolling get an explicit speed pass after the scroll area
  runs (`VIEWER_SCROLL_SPEED`, overridable with `MKDV_SCROLL_SPEED`). It applies
  to both axes so a diagonal gesture keeps its direction, and it is what keeps
  scrolling working during a text-selection drag.
- The current document position under the pointer is preserved using the old
  and new content extents on both axes.
- The viewer exposes both scroll axes. Holding the middle mouse button and
  dragging updates those offsets while zoomed in; middle-click is not treated
  as a Markdown link activation.

The viewer zoom is persisted between sessions and can be reset from **View →
Reset Viewer Zoom**. Its supported range is 75%–300% so zoomed-out text remains
readable.

## Validation

The pure scroll-anchor helper is unit-tested. The integration path is covered
by the normal Cargo test, format, Clippy, and package checks; interactive
verification should include pinch (Wayland), Ctrl-wheel, pointer anchoring,
middle-drag, and ordinary scrolling at 100% viewer zoom.
