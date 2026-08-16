# Feature: Application icon

**Status:** ✅ Complete
**Branch:** `rename/mkdv`
**Date:** 2026-08-16
**Lines Changed:** +47 / -2 in `src/main.rs`, `data/`, `README.md`

## Summary

The GNOME launcher entry shipped `Icon=application-x-executable`, the generic
"some binary" cog. This adds a real icon — a stylised cat holding binoculars,
matching what the app does — as a hand-authored SVG, installs it where the icon
theme can find it, and uses a rasterized copy as the window/taskbar icon.

## Features

- [x] `data/mkdv.svg` — scalable launcher icon
- [x] `data/mkdv-256.png` — rasterized copy for the window icon
- [x] `install_desktop_entry()` writes the SVG into `hicolor`
- [x] `Icon=mkdv` in the desktop entry
- [x] `ViewportBuilder::with_icon` for the window/taskbar

## Key Discoveries

### `Icon=` is a theme lookup, not a path

A bare `Icon=mkdv` only resolves if the file sits in an icon theme directory on
the search path. Installing the entry without the icon leaves the launcher on
its fallback, with no error anywhere. Both writes belong in the same function:

```
$XDG_DATA_HOME/applications/mkdv.desktop
$XDG_DATA_HOME/icons/hicolor/scalable/apps/mkdv.svg
```

An absolute `Icon=/path/to/mkdv.svg` also works and would skip the second write,
but it bakes a build-machine path into a file the app rewrites on every start.

### The window icon needs pixels, not SVG

`ViewportBuilder::with_icon` takes `egui::IconData`, which is a raw RGBA buffer,
so the SVG cannot be used directly. `image` is already a dependency with `png`
enabled, so a 256×256 PNG is embedded and decoded at startup. Both icon files
are generated from the same SVG and must be regenerated together:

```bash
magick -background none data/mkdv.svg -resize 256x256 -strip data/mkdv-256.png
```

ImageMagick renders SVG through librsvg here (`magick -list format | grep SVG`
shows `RSVG`), so the raster matches what GTK draws rather than ImageMagick's
weaker internal MSVG renderer.

### Decoration must never be fatal

Neither the icon write nor the PNG decode can fail startup — both log a warning
and continue. The launcher still works with a generic icon; a window with no
icon is still a usable window.

## Architecture

| Item | Purpose |
|------|---------|
| `APP_ICON_SVG` | `include_str!` of `data/mkdv.svg`, written to `hicolor` on install |
| `APP_ICON_PNG` | `include_bytes!` of `data/mkdv-256.png` |
| `window_icon()` | Decodes the PNG to `egui::IconData`; `None` on failure |

## Design Notes

The first draft drew the binoculars as two circles with a bridge, which reads as
**round glasses**, not binoculars. What separates the two visually:

- Barrels taller than wide, so they read as tubes rather than lenses.
- A ridged focus wheel standing proud of the bridge — the strongest single cue.
- A seam between eyecup and prism housing, giving the barrel depth.

Paws gripping the barrels were tried and dropped: at 48px they collided with the
whiskers and read as orange blobs beside the head rather than as paws.

Legibility was checked by rendering at 256, 48 and 32px. Tabby stripes on the
forehead survive down to 32px; whiskers blur out there but do no harm.

## Testing Notes

- `desktop_entry_uses_the_installed_executable_path` also asserts `Icon=mkdv`.
- `window_icon_decodes_to_a_square_rgba_buffer` proves the embedded PNG decodes
  and that its buffer length matches its dimensions.
- `--install-desktop` was run against a throwaway `XDG_DATA_HOME`, confirming
  both files land in the right place.

Gates: root `cargo fmt --check`, `cargo clippy --all-targets -D warnings` (only
the pre-existing `max_width` and `allocate_ui_at_rect` warnings), and
`cargo test --all-targets` → 50 passed.

## Future Improvements

- [ ] Ship PNG fallbacks under `hicolor/{48x48,256x256}/apps` for environments
      that do not rasterize SVG icons. Not needed for GNOME.
- [ ] A monochrome symbolic variant (`mkdv-symbolic.svg`) for GNOME shell
      surfaces that prefer one.
