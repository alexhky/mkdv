# Feature: Wayland pinch-to-zoom, scroll speed, middle-button panning

**Status:** ✅ Complete
**Branch:** `rename/mkdv`
**Date:** 2026-08-16
**Lines Changed:** ~+350 / -125 in `src/main.rs`, `src/pinch.rs`, `Cargo.toml`, docs

## Summary

Fixes the touchpad gesture work landed in `028faea`/`ce4b10c`, which shipped
three problems:

1. **Pinch-to-zoom did not work at all.** The implementation rested on a false
   premise — that Linux touchpads deliver pinch as Ctrl+wheel.
2. **Touchpad scrolling became much slower.** A new gate disabled the viewer's
   scroll pass for exactly the input a touchpad produces.
3. **Middle-button drag highlighted text** while panning.

Pinch now works on Wayland, via a protocol winit does not bind. Scrolling is
back to its previous pace and then some, and the middle button is a pure pan
handle over the viewer.

## Features

- [x] Touchpad pinch-to-zoom on Wayland (`zwp_pointer_gestures_v1`)
- [x] Restored + tunable touchpad/wheel scroll speed
- [x] Middle-button drag pans without selecting text
- [x] Ctrl+Scroll zoom no longer also scrolls the document
- [x] Zoom snapped to a ladder so pinch replays cached layout/glyph levels
- [x] Click the zoom/scale readout to reset it to 100%
- [x] Lightbox gestures: wheel zooms, two-finger swipe pans, pinch zooms

## Key Discoveries

### winit has no touchpad gesture backend on Linux

`grep -rn "gesture" winit-0.30.12/src/platform_impl/linux/` returns nothing.
`WindowEvent::PinchGesture` is macOS-only, so `egui::Event::Zoom` never reaches
an eframe app on X11 or Wayland. Dumping `RawInput` during a real pinch
confirmed it: **zero** `Zoom` events, only `MouseWheel`.

The previous code's comment — "on Linux, a touchpad pinch is commonly exposed
as Ctrl+wheel" — is wrong. Browsers behave that way because *they* implement
the pinch protocol and synthesize the wheel events themselves.

### The compositor does deliver pinch; you just have to ask

Dumping the compositor's globals showed `zwp_pointer_gestures_v1 v3`. The
protocol needs a `wl_pointer`, and the non-obvious part is that pointer focus is
tracked per **client**, not per `wl_pointer` object. So a *second*
`wl_seat`/`wl_pointer` created on winit's existing connection still receives
focus for winit's surfaces:

```rust
// wl_display from CreationContext::display_handle()
let backend = unsafe { Backend::from_foreign_display(display.cast()) };
let conn = Connection::from_backend(backend);
let (globals, mut queue) = registry_queue_init::<State>(&conn)?;
// our own seat + pointer, on winit's connection
let pointer = seat.get_pointer(qh, ());
gestures.get_pinch_gesture(&pointer, qh, ());
```

A *separate connection* does not work: it owns no surfaces, so its pointer is
never focused and no gesture events arrive. Dispatch runs on a private event
queue in its own thread; libwayland serialises reads across queues, so winit is
unaffected and needs no patching.

### `pinch.update` reports absolute scale

`scale` is relative to the start of the gesture, not to the previous event, so
consecutive updates must be divided to get a multiplicative step.

### Why point-unit input made scrolling half-speed

The viewer runs a manual scroll pass *after* the renderer-owned `ScrollArea`,
originally to keep the wheel alive during a text-selection drag. Because the
ScrollArea has already consumed the same delta, that pass is also what sets the
viewer's effective scroll speed.

`ce4b10c` added a gate skipping the pass whenever input was `MouseWheelUnit::Point`
and nothing else — which is exactly what a Wayland touchpad sends. The pass
never ran for touchpads, halving them. Verified by building the pre-change
commit and comparing traces side by side.

### egui keeps zoom deltas in `raw_scroll_delta`

`input_state/mod.rs` does `raw_scroll_delta += delta` *before* branching on
`is_zoom`; only `smooth_scroll_delta` is withheld. Anything reading
`raw_scroll_delta` directly has to check the zoom modifier itself.

### Text selection starts on *any* pointer button

`label_text_selection.rs` uses `i.pointer.any_pressed()`, which includes the
middle button, and no widget-level flag disables it. The only reliable fix is to
remove the middle press/release pair from `RawInput` before `InputState` sees it
and track the button ourselves.

### Wheel unit is the only mouse-vs-touchpad signal available

winit reports a mouse wheel in `MouseWheelUnit::Line` and a touchpad swipe in
`MouseWheelUnit::Point`. That is what lets the lightbox give the two devices
different meanings — a wheel notch zooms, a two-finger swipe pans — without
asking the platform what kind of pointing device is attached. The same
distinction is what made the scroll-speed regression device-specific.

### Smooth pinch needs a zoom ladder

Every distinct zoom re-wraps the document *and* makes egui rasterize a fresh
glyph set. A browser scales an already-rasterized layer; egui has no such path.
Snapping to a geometric ladder (2% per level, ~65 levels) means a repeated
gesture replays cached levels. The gesture must accumulate into a separate
unsnapped target, or each sub-step rounds away and zoom never moves.

## Architecture

### New module

`src/pinch.rs` — `PinchGestures`, an accumulator drained once per frame.
Inert on X11 and non-Linux platforms; every failure path logs and degrades to
"no zoom" rather than surfacing an error.

### New/Modified fields

```rust
struct MarkdownApp {
    viewer_zoom: f32,        // always on the VIEWER_ZOOM_STEP ladder
    viewer_zoom_target: f32, // unsnapped accumulator for gestures
    viewer_rect: egui::Rect, // previous frame, for raw_input_hook
    middle_pan_active: bool, // events are stripped, so we own this state
    pinch: pinch::PinchGestures,
}
```

### New functions

| Function | Purpose |
|----------|---------|
| `quantize_viewer_zoom()` | Snap a zoom factor to the ladder |
| `viewer_scroll_speed()` | Resolve `MKDV_SCROLL_SPEED`, default `VIEWER_SCROLL_SPEED` |
| `pinch::PinchGestures::take_zoom_factor()` | Drain accumulated pinch |
| `pinch::imp::spawn()` | Bind the protocol and start the dispatch thread |

### Zoom input priority

1. Touchpad pinch (`src/pinch.rs`, Wayland)
2. `Event::Zoom` (macOS)
3. Ctrl + wheel / Ctrl + two-finger scroll

## Testing Notes

Diagnosis and verification were done against real hardware, not by inspection —
the earlier attempt failed precisely because it reasoned from an assumed
platform behavior. Temporary `RawInput` instrumentation captured the actual
event stream during pinch, scroll, and Ctrl+scroll, and a baseline binary built
from `97dfb01` was compared side by side for scroll pacing.

Confirmed interactively on GNOME/Wayland:

- Pinch zooms; second sweep is noticeably smoother (warm caches)
- Scroll speed matches the pre-regression build at `MKDV_SCROLL_SPEED=1.0`,
  ships at `2.0`
- Middle-drag pans with no text selection; middle-click still closes tabs and
  opens explorer files
- Clicking the top-bar zoom readout and the lightbox zoom percentage resets to
  100%
- In an opened image: two-finger swipe pans, pinch zooms, wheel zooms

49 unit tests pass, including new coverage of the zoom ladder.

**Not covered:** X11 (no gesture protocol exists — Ctrl+Scroll is the fallback),
macOS/Windows (`PinchGestures` compiles to a no-op there).

## Future Improvements

- [ ] Expose scroll speed as a setting rather than an env var
- [ ] Anchor pinch zoom at the gesture centroid rather than the pointer
- [ ] Revisit if winit ever gains a Linux gesture backend; `src/pinch.rs` could
      then be deleted in favor of `Event::Zoom`
