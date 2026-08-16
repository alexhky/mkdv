# Feature: Upgrade merman to 0.7

**Status:** ✅ Complete
**Branch:** `rename/mkdv`
**Date:** 2026-08-16
**Lines Changed:** +72 / -182 in `crates/egui_commonmark/egui_commonmark_backend/`

## Summary

Bumps the vendored renderer's mermaid backend from merman 0.3 to 0.7 (the
latest stable) and adapts our SVG post-processing to match. merman 0.7 fixes
two of the bugs we were working around, so this upgrade is mostly a **deletion**
of hacks — plus one fix for an output change that silently broke a third hack.

## Features

- [x] `merman` 0.3 → 0.7 in `egui_commonmark_backend`
- [x] Drop `fix_double_escaped_xml_entities()` — fixed upstream
- [x] Drop `wrap_fallback_text()` and its three helpers — merman now wraps
- [x] Fix `sanitize_svg_font_family()` for merman's new `&quot;`-quoted fonts
- [x] Unit tests covering the post-processing that remains

## Key Discoveries

### The API surface we use did not change

`HeadlessRenderer::new`, `.with_text_measurer`, `DeterministicTextMeasurer {
char_width_factor, line_height_factor }`, `Clone`, and
`render_svg_readable_sync` are all identical between 0.3 and 0.7. The upgrade
needed no call-site changes at all — every edit below is about merman's
*output* changing, not its API.

### merman 0.7 emits singly-escaped entities

0.3 double-escaped text that mermaid had already entity-escaped for
foreignObject content, so `A & B` arrived as `A &amp;amp; B`. We reverted the
second pass in `fix_double_escaped_xml_entities()`.

0.7 fixed this at the source. Rendering `A["A & B < C"]`:

| merman | `&amp;amp;` / `&amp;lt;` occurrences |
|--------|--------------------------------------|
| 0.3.0  | 3 |
| 0.7.0  | 0 |

The workaround therefore had to go — left in place it would have *introduced*
the corruption it used to fix, turning a legitimately escaped `&amp;` into a
bare `&` and producing invalid XML.

### merman 0.7 wraps fallback text itself

0.3 put an entire node label on one line in the fallback `<text>` element even
though the node rect was sized for wrapped text, so long labels overflowed.
`wrap_fallback_text()` re-emitted the group as one `<text>` per visual line,
wrapping at a hard-coded 28 characters and recomputing `dy` from scratch.

0.7 emits one `<text>` per line already, positioned with real `y` values:

```xml
<g data-merman-foreignobject="fallback" class="merman-foreignobject-fallback ...">
  <text x="138" y="47" ...>This is a very long node</text>
  <text x="138" y="71" ...>label that should wrap</text>
  <text x="138" y="95" ...>somewhere</text>
</g>
```

This is strictly better than our version, because merman wraps using the same
text measurer that sized the rect — our 28-char heuristic did not know the node
width. Removed.

### The `&quot;` change broke font sanitization

`sanitize_svg_font_family()` rewrites CSS `font-family` declarations by
skipping the old value up to the terminating `;`. merman 0.7 started quoting
font names as XML entities:

```
0.3: font-family: trebuchet ms,verdana,arial,sans-serif;
0.7: font-family: &quot;trebuchet ms&quot;,verdana,arial,sans-serif;
```

The `;` that ends `&quot;` comes first, so the naive scan stopped there and
left `trebuchet ms&quot;,verdana,arial,sans-serif;` in the output — corrupted
markup inside a `style="..."` attribute. `find_css_value_end()` now skips over
entity references while looking for the real terminator.

This one is worth remembering: a dependency upgrade broke a workaround
*silently*, with no compile error and no panic — only wrong pixels.

### `strip_stroke_text()` was left alone

Neither 0.3 nor 0.7 emits `stroke="#fff"` on a `<text>` element (0.7 only uses
it on gitgraph `<line>`s), so the function is already a guarded no-op. It is
unrelated to this upgrade and removing it would mix dead-code cleanup into a
dependency bump, so it stays.

### The `roughr-merman` pin is still required

`merman-render` 0.7.0 declares `roughr ^0.12.0` but still calls
`OptionsBuilder::seed`, which 0.12.1 removed — so a fresh resolve picks 0.12.2
and fails to compile, exactly as it did on 0.3. Building merman 0.7 against
roughr 0.12.2 produces 11 errors of the same class. The `=0.12.0` pin from the
previous commit carries over unchanged; see `docs/LESSONS.md`.

## Architecture

### Removed Functions

| Function | Why |
|----------|-----|
| `fix_double_escaped_xml_entities()` | merman 0.7 no longer double-escapes |
| `CodeBlock::wrap_fallback_text()` | merman 0.7 wraps fallback text itself |
| `CodeBlock::wrap_tspans_in_group()` | helper of the above |
| `CodeBlock::extract_attr()` | helper of the above |
| `CodeBlock::word_wrap()` | helper of the above |

### New Functions

| Function | Purpose |
|----------|---------|
| `CodeBlock::find_css_value_end()` | Locate the `;`/`}` ending a CSS declaration, skipping XML entities |

### Pipeline

```rust
// before
let svg = fix_double_escaped_xml_entities(&svg);
let svg = CodeBlock::sanitize_svg_font_family(&svg);
let svg = CodeBlock::strip_stroke_text(&svg);
let svg = CodeBlock::wrap_fallback_text(&svg);

// after
let svg = CodeBlock::sanitize_svg_font_family(&svg);
let svg = CodeBlock::strip_stroke_text(&svg);
```

## Testing Notes

Three unit tests were added in `misc.rs` (`mermaid_tests`), gated on the
`mermaid` feature: font-family rewriting in both the entity-quoted CSS form and
the plain XML-attribute form, and entity pass-through.

Beyond the unit tests, six diagram types (flowchart with `& < >` entities,
flowchart with a long wrapping label, sequence, class, state, pie) were rendered
end to end through the real pipeline — merman 0.7 → `sanitize_svg_font_family`
→ `strip_stroke_text` → `rasterize_mermaid_svg` — and the resulting PNGs were
inspected. All six render correctly: entities show as literal `&`, `<`, `>`;
fonts resolve to DejaVu/Noto rather than a monospace fallback; and long labels
wrap inside their node rect.

Gates: root `cargo fmt --check`, `cargo clippy --all-targets -D warnings`
(only the pre-existing `max_width` and `allocate_ui_at_rect` warnings), and
`cargo test --all-targets` → 49 passed.

### Do not run `cargo fmt --all` inside `crates/egui_commonmark`

The nested workspace has its own `rust-toolchain` pinning 1.88, whose rustfmt
uses the older import-ordering style. Formatting the fork with it rewrites
`use` blocks across eight otherwise-untouched files (~350 lines of pure churn).
Format only the root workspace; edit the vendored crate by hand.

## Future Improvements

- [ ] Re-tune `char_width_factor` (currently 0.65). It was raised from merman's
      default 0.55 to stop text overlapping in 0.3. Now that 0.7 wraps using the
      same measurer, the value mostly controls how much padding a node gets —
      rendered nodes currently look slightly wider than the text needs.
- [ ] merman 0.7 adds `eventmodeling`, `ishikawa`, `tree_view` and `venn`
      diagram types plus a `theme` module. None are wired up; `theme` in
      particular could replace `sanitize_svg_font_family()` if it allows setting
      the font family before rendering rather than patching the SVG afterwards.
- [ ] Revisit the `=0.12.0` roughr pin if merman 0.8 ships stable — the 0.8
      alphas are the reason 0.12.1/0.12.2 exist, so 0.8 should resolve cleanly.
