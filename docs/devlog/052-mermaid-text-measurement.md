# Feature: Match mermaid text measurement to the rendered font

**Status:** ✅ Complete
**Branch:** `rename/mkdv`
**Date:** 2026-08-16
**Lines Changed:** +23 / -15 in `crates/egui_commonmark/egui_commonmark_backend/src/misc.rs`

## Summary

Follow-up to [051](051-merman-0.7-upgrade.md), which left `char_width_factor:
0.65` as an open question. Measuring it turned up something bigger than a
mis-tuned constant: `HeadlessRenderer::new()` already ships a per-glyph text
measurer, and our `with_text_measurer(...)` call was *downgrading* it to a flat
one-width-fits-all model. Removing the override and reordering the substituted
font stack makes layout metrics agree with render metrics.

## Features

- [x] Drop the `DeterministicTextMeasurer` override, use merman's default
      `VendoredFontMetricsTextMeasurer`
- [x] Reorder the substituted font stack by metric compatibility
- [x] Update the sanitizer tests for the new stack

## Key Discoveries

### A flat `char_width_factor` cannot be tuned to a good value

`DeterministicTextMeasurer` charges every glyph `font_size * char_width_factor`,
so the factor has to cover the widest glyph in the string or the label overflows
its shape. Advance widths at 16px, measured with `ttf-parser` against the fonts
actually installed, versus what each measurer predicted:

| text | det 0.65 | merman heuristic | vendored | Noto Sans | Liberation Sans |
|------|---------:|-----------------:|---------:|----------:|----------------:|
| `Process the input data` | 228.8 | 146.9 | 160.3 | 168.3 | 159.2 |
| `Database Connection Pool` | 249.6 | 165.3 | 185.7 | 198.1 | 190.4 |
| `Retry with exponential backoff` | 312.0 | 206.6 | 222.1 | 228.9 | 213.4 |
| `illlliii` | 83.2 | 35.8 | 37.1 | 33.0 | 28.4 |
| `MMMMMMMM` | 83.2 | 76.8 | 90.9 | 116.1 | 106.6 |

Totals over a prose corpus, as a ratio to the real rendered width:

| measurer | vs Noto Sans | vs Liberation Sans |
|----------|-------------:|-------------------:|
| `DeterministicTextMeasurer { 0.65 }` | **1.333** | **1.396** |
| `DeterministicTextMeasurer { 0.0 }` (built-in heuristic) | 0.891 | 0.933 |
| `VendoredFontMetricsTextMeasurer` (merman default) | 0.954 | 0.999 |

0.65 sizes labels a third wider than they render. Lowering it does not fix the
shape of the error — `illlliii` is over-measured 2.5× while `MMMMMMMM` is
under-measured, so any single value trades bloat against overflow.

### `HeadlessRenderer::new()` already uses per-glyph metrics

merman 0.7's `HeadlessRenderer::default()` builds
`LayoutOptions::headless_svg_defaults()`, documented as "suitable for headless
SVG rendering in UI integrations", which installs
`VendoredFontMetricsTextMeasurer` — real per-glyph advance tables plus kerning
pairs, extracted from a browser. Our `with_text_measurer(...)` call replaced it
with the placeholder measurer. The fix is a deletion.

This override predates the 0.7 upgrade; merman 0.3 needed it. Keeping it after
the bump was the mistake, and 051 carried it over without re-checking.

### The vendored tables are Trebuchet MS, and the font stack has to match

Layout metrics only help if the SVG is then rendered in a font with those
metrics. `0123456789` measures 83.9px vendored = 0.5244 em/char, which is
Trebuchet MS's digit advance (Liberation Sans is 0.556). Our stack led with
DejaVu Sans, which is wider than Trebuchet across the board — so the shapes came
out too small for the text drawn into them.

Reordered to put metrically-compatible families first:

```
Trebuchet MS, Liberation Sans, Arial, Helvetica, DejaVu Sans, Noto Sans
```

Liberation Sans (Arial-metric) totals 0.999 of the vendored measurement on prose
and never diverges more than ~3% per string, so shapes fit their labels. DejaVu
Sans and Noto Sans stay at the back so text still renders on systems without any
of the leading families.

### Custom measurers need a direct `merman-render` dependency

The obvious "measure with the font we actually rasterize with" approach — a
`TextMeasurer` backed by `MERMAID_FONTDB` — is not reachable through the
`merman` facade: it re-exports `TextMeasurer` and the two implementations, but
not `TextStyle`, `TextMetrics` or `WrapMode`, which the trait's methods need.
Implementing one means depending on `merman-render` directly. Not worth it while
the vendored tables land within a few percent.

## Architecture

`CommonMarkCache::default()` now constructs the renderer with no measurer
override:

```rust
// before
mermaid_renderer: merman::render::HeadlessRenderer::new().with_text_measurer(Arc::new(
    merman::render::DeterministicTextMeasurer { char_width_factor: 0.65, line_height_factor: 0.0 },
)),

// after
mermaid_renderer: merman::render::HeadlessRenderer::new(),
```

## Testing Notes

Eight diagrams (flowchart, wrapping flowchart, entity flowchart, unicode
flowchart, sequence, class, state, pie) were rendered end to end through the
real pipeline before and after, and the PNGs compared. Diagram widths dropped
without any label overflowing:

| diagram | before | after |
|---------|-------:|------:|
| flowchart | 586.0 × 651.6 | 555.4 × 558.0 |
| class | 397.4 × 294 | 301.6 × 294 |
| state | 140.8 × 298 | 97.0 × 298 |
| sequence | 590.0 × 259 | 534.0 × 259 |
| pie | 567.2 × 450 | 570.2 × 450 |

Arrow glyphs improved as a side effect: `→`/`←` are absent from Noto Sans but
present in Liberation Sans, so promoting Liberation moved arrow labels from a
bold monospace fallback into the proportional font.

Gates: root `cargo fmt --check`, `cargo clippy --all-targets -D warnings` (only
the pre-existing `max_width` and `allocate_ui_at_rect` warnings),
`cargo test --all-targets` → 49 passed, backend with `mermaid` → 18 passed.

## Future Improvements

- [ ] A glyph missing from *every* listed family (e.g. `✓`) makes resvg swap the
      whole text run to a monospace fallback, not just that glyph. Pre-existing,
      unrelated to metrics, and not fixed here — a symbol font appended to the
      stack would not help, since the run needs one font covering all its glyphs.
- [ ] If merman ever exposes `TextStyle`/`WrapMode` through the facade, a
      measurer reading `MERMAID_FONTDB` would make layout exact on every machine
      instead of within a few percent on machines with an Arial-metric font.
