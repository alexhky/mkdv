# Target Metrics

- Binary size: approximately 35 MB for the stripped Linux release build,
  including syntect, Mermaid, Typst math, image support, Wayland, and X11;
  approximately 7 MB as a compressed snap.
- Startup target: under 200 ms.
- Rendering target: 60 FPS, using persistent caches and viewport clipping for
  large documents.
- Release targets: Linux x86_64, macOS arm64, and Windows x86_64. Linux supports
  native X11 and Wayland.

These are project targets and observed release sizes, not automated benchmark
assertions in CI.

## Feature Progress

- **Multi-window prototype**: superseded by the current tabbed design.
- **Tab system**: shipped as a custom `Vec<Tab>` implementation; the application
  does not use `egui_dock`.
- **Hybrid tabs + multi-window**: not implemented.

## Delivered Navigation and Table Work

1. **Search / find-all (Ctrl+F)** shipped in v0.1.4 (PR #14).
2. **Table horizontal scrolling** uses a nested horizontal `ScrollArea`, its
   scrollbar, native horizontal input, and Shift+wheel forwarding.
3. **Resizable table columns** shipped in v0.1.5. Markdown and HTML tables now
   use `egui_extras::TableBuilder` with draggable column dividers.

## Current Engineering Priorities

- Keep renderer tests and root application tests green as egui evolves.
- Remove the vendored renderer's future-incompatibility and deprecation warnings.
- Split `src/main.rs` into focused modules when doing so can preserve the
  immediate-mode `State → Logic → UI → Async` flow.
- Add repeatable startup, large-document, and memory benchmarks before treating
  the performance targets above as enforced regressions.
