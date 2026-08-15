# Development Status and Roadmap

This document describes the current development baseline for `mkdv`
0.1.15. Historical implementation details live in `docs/devlog/` and
`docs/IMPLEMENTATION-PLAN.md`; current runtime structure lives in
`docs/ARCHITECTURE.md`.

## Project Goal

Build a fast, lightweight, distraction-free Markdown reader with strong
typography, scientific-document support, and native desktop behavior.

Current targets:

- Startup in under 200 ms.
- Smooth 60 FPS reading and scrolling for large documents.
- Approximately 35 MB for the stripped Linux release binary.
- Native Linux X11 and Wayland support, with Cargo builds for Linux, macOS,
  and Windows.

These performance values are targets and observed release sizes. CI does not
currently enforce them with benchmarks.

## Current Baseline

### Application

- Rust 2021 binary with MSRV 1.80.
- egui/eframe 0.33 using the glow backend.
- Custom `Vec<Tab>` tab system with per-tab rendering cache, scroll state,
  outline, search results, and back/forward navigation.
- Persisted tabs, active tab, theme, app scaling, viewer zoom, content width,
  explorer state, sort order, and recent files.
  order, and recent files.
- Welcome page when no documents are open.

### Markdown Rendering

- Vendored `egui_commonmark_extended` 0.25 workspace under
  `crates/egui_commonmark/`.
- GitHub-flavored Markdown, syntax highlighting, remote/local images, SVG text,
  HTML tables, task lists, footnotes, alerts, and emoji shortcodes.
- LaTeX math rendered through mitex and Typst on background threads.
- Mermaid rendered through merman/resvg on a background thread.
- Resizable Markdown and HTML table columns using
  `egui_extras::TableBuilder`.
- Search-range painting, duplicate-heading position tracking, custom line
  height, a real strong-text font family, and a zoomable image/Mermaid lightbox.

### Navigation and Files

- Native file and folder dialogs, drag-and-drop, local Markdown links, and
  Ctrl+Click to open links in a new tab.
- Current-document search with inline highlights and precise corrective
  scrolling.
- Collapsible outline sidebar and lazy hierarchical file explorer.
- Explorer sorting by name or modification time in either direction.
- Local files watched with the recommended platform watcher; open GVFS files
  use polling. Explorer watches are non-recursive and follow visible expanded
  directories to avoid large startup scans.
- Live reload is enabled by default; `--no-watch` disables it.
- Terminal launches detach by default; `--foreground` keeps the GUI attached.

## Repository Layout

```text
.
├── src/main.rs                         # Application and root unit tests
├── crates/egui_commonmark/             # Separate renderer workspace
│   ├── egui_commonmark/                # Public viewer API and parser UI
│   ├── egui_commonmark_backend/        # Shared rendering/cache backend
│   └── egui_commonmark_macros/         # Optional proc macros
├── data/                               # GNOME desktop entry template
├── docs/                               # Architecture, lessons, plans, devlogs
├── scripts/                            # Cargo publishing helper
└── .github/workflows/                  # Cargo CI
```

`src/main.rs` is currently about 4,900 lines. Keep immediate-mode work ordered
as `State → Logic → UI → Async`; see `docs/EGUI_WORKFLOW.md` before changing UI
behavior.

## Development Commands

```bash
# Root application
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run -- --foreground README.md

# Vendored renderer workspace (separate from the root Cargo package)
cargo test --manifest-path crates/egui_commonmark/Cargo.toml --workspace
```

The root `cargo test` command does not execute the nested renderer workspace's
own unit, integration, or proc-macro tests.

## Near-Term Maintenance

1. Remove the vendored renderer's current Rust/egui warnings:
   future-incompatible inferred float literals, an unused table parameter, and
   the deprecated `Ui::allocate_ui_at_rect` call.
2. Run the nested renderer workspace tests in CI in addition to root tests.
3. Modularize `src/main.rs` along stable boundaries such as explorer, watcher,
   search, tabs, persistence, and lightbox without moving parsing or expensive
   work into per-frame UI code.
4. Add repeatable startup, large-document scrolling, memory, and release-size
   measurements.
5. Keep overview documentation synchronized when dependencies, Cargo metadata,
   or shipped features change.

## Longer-Term Options

- Hybrid tabs and multiple OS windows remain unimplemented; the old exploration
  is retained in `docs/IMPLEMENTATION-PLAN.md` as historical design context.
- Nightly `-Zlocation-detail=none` remains an optional release-size experiment.
- New renderer capabilities should be added to the vendored workspace only when
  the public API, cache lifetime, async behavior, and crates.io publishing path
  are documented and tested together.

## Dependency Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| eframe / egui | 0.33 | Immediate-mode desktop UI and glow rendering |
| egui_commonmark_extended | 0.25 | Vendored Markdown renderer |
| image | 0.25 | PNG, JPEG, and GIF decoding |
| rfd | 0.17 | Native file/folder dialogs |
| notify | 6.1 | Recommended and polling file watchers |
| notify-debouncer-mini | 0.4 | Debounced watcher events |
| clap | 4 | CLI parsing |
| regex | 1.12 | Header, link, and search-related parsing |
| mimalloc | 0.1 | Global allocator |
| serde | 1 | Persisted application state |

The renderer workspace additionally brings in syntect, merman/resvg,
mitex/Typst, and egui_extras behind the feature set enabled by the root package.
