# Architecture

`mkdv` 0.1.15 is a Rust desktop application for viewing Markdown with
egui/eframe and a custom tab system. The application currently lives in one
main module (`src/main.rs`, about 4,900 lines). Its vendored
`egui_commonmark_extended` workspace contributes about 8,900 more lines of
renderer, backend, macro, and test code under `crates/egui_commonmark/`.

## Core Components

- **MarkdownApp**: Main struct implementing `eframe::App`. Holds:
  - `tabs: Vec<Tab>` - list of open tabs
  - `active_tab: usize` - index of the currently active tab
  - `dark_mode: bool` - global theme setting
  - `scaling: f32` - global UI scaling (0.5 to 3.0)
  - `viewer_zoom: f32` - Markdown-only content zoom (0.75 to 3.0)
  - `show_outline: bool` - toggle outline sidebar visibility
  - `full_width_content: bool` - use the full content pane instead of the readable-width cap
  - `show_explorer: bool` - toggle file explorer visibility
  - `file_explorer: FileExplorer` - file explorer state
  - `watch_enabled: bool` - file watching state
  - `watcher` + `watcher_rx` - local/GVFS file watching via an mpsc bridge
  - `watched_paths: HashSet<PathBuf>` - open document paths currently watched
  - `watched_explorer_dirs: HashSet<PathBuf>` - visible local explorer directories currently watched
  - `hovered_tab: Option<usize>` - for showing close button on hover
  - `recent_files: Vec<RecentEntry>` - recently opened files for the welcome page
  - `welcome_show_all: bool` - whether the welcome page's recent list is expanded
  - `search: SearchState` - current find-bar query, focus, and active result
  - `lightbox: Option<LightboxState>` - enlarged image/Mermaid overlay state

- **Tab**: Per-tab state for a document. Each tab has:
  - `id: egui::Id` - unique identifier
  - `path: PathBuf` - file path
  - `content: String` - markdown text
  - `cache: CommonMarkCache` - **must persist across frames** (never recreate per-frame, only reset on file load)
  - `document_title: Option<String>` - first h1 used as sidebar title
  - `outline_headers: Vec<Header>` - parsed headers for outline
  - `collapsed_headers: HashSet<usize>` - collapsed outline sections
  - `scroll_offset`, `horizontal_scroll_offset`, `pending_scroll_offset`, `last_content_height`, `last_content_width`, `last_viewport_height` - scroll state
  - `pending_header_click_key`, `correct_active_search_pending` - one-shot precise-scroll correction state
  - `local_links: Vec<String>` - cached local markdown links
  - `history_back`, `history_forward: Vec<PathBuf>` - per-tab navigation history
  - `search_matches: Vec<SearchMatch>` - source byte ranges for the current query
  - `content_version: u64` - renderer invalidation token after reload/navigation

- **PersistedState**: Serializable struct for session persistence:
  - `dark_mode: Option<bool>`
  - `scaling: Option<f32>`
  - `viewer_zoom: Option<f32>`
  - `show_outline: Option<bool>`
  - `full_width_content: Option<bool>`
  - `show_explorer: Option<bool>` - file explorer visibility
  - `explorer_root: Option<PathBuf>` - file explorer root directory
  - `expanded_dirs: Option<Vec<PathBuf>>` - expanded directories in explorer
  - `explorer_sort_order: Option<SortOrder>` - name/date and ascending/descending order
  - `open_tabs: Option<Vec<PathBuf>>` - restore tabs on startup
  - `active_tab: Option<usize>` - restore active tab position
  - `recent_files: Option<Vec<RecentEntry>>` - recently opened files (welcome page)

- **FileExplorer**: Left sidebar showing markdown files in a directory tree:
  - `root: Option<PathBuf>` - root directory to display
  - `tree: Vec<FileTreeNode>` - hierarchical file tree
  - `expanded_dirs: HashSet<PathBuf>` - which directories are expanded
  - `sort_order: SortOrder` - active name/date ordering
  - `pending_scan` - background result channel for a GVFS root scan

- **FileTreeNode**: Enum representing a node in the file explorer tree:
  - `File { path, name, display_name, modified }` - a Markdown file
  - `Directory { path, name, display_name, modified, children }` - a directory whose `children` are loaded lazily

- **File Watching**: Uses `notify-debouncer-mini` with a 200ms debounce.
  Local files use the platform-recommended watcher (inotify on Linux); open
  files under GVFS use a two-second polling watcher. The local explorer root
  and expanded directories are watched non-recursively. A bridge thread wakes
  egui when events arrive, matching tabs are reloaded, and failures are retried
  up to three times.

- **Header Outline**: `parse_headers()` returns a `ParsedHeaders` struct containing `document_title` (first h1) and `outline_headers` (remaining headers). Duplicate headings receive stable composite keys. The outline is rendered as a collapsible, resizable right sidebar and uses renderer-recorded positions for precise navigation.

- **Link Navigation**: Uses the extended renderer's link-hook mechanism. Ctrl+Click opens links in new tabs, regular click navigates within the current tab.

- **Search (Ctrl+F)**: Current-document find bar with inline highlights. `SearchState` lives on `MarkdownApp`; per-tab `search_matches: Vec<SearchMatch>` cache match byte ranges. Highlights are painted by the vendored renderer through `CommonMarkCache::set_search_ranges`; the renderer splits `Event::Text` and non-wrapped `Event::Code` at range boundaries and applies a background color to matching segments. Enter/Shift+Enter cycle matches with line-ratio bootstrap scrolling followed by renderer-recorded position correction; Esc closes the bar and clears highlights on all tabs.

- **Markdown strong text**: mkdv registers the renderer's `STRONG_FONT_FAMILY` (`MarkdownStrong`) during `setup_fonts`, then enables `CommonMarkViewer::use_strong_font_family(true)` so `**strong**` spans can use a real bold face while generic `egui_commonmark` consumers remain opt-in.

- **Keyboard document scrolling**: Plain document scroll keys are handled in `MarkdownApp::update` after mode-specific shortcuts are checked. `KeyboardScrollAction` maps Up/Down to fixed line steps and Page Up/Page Down to viewport-relative page steps through `keyboard_scroll_target`, which clamps against the active tab's `last_content_height`. The chosen target is assigned to the tab's `pending_scroll_offset`, so keyboard scrolling uses the same renderer-owned `ScrollArea` pipeline as outline and search jumps. Arrow keys are reserved for search-result navigation while the find bar is open, and document scrolling ignores Ctrl/Alt/Command-modified keypresses so it does not steal existing shortcuts.

- **App scaling and viewer zoom**: `scaling` is applied with egui's global zoom factor and affects menus, sidebars, and window chrome. `viewer_zoom` is applied inside a scoped viewer style, so only Markdown typography, spacing, and layout widths change. Raw native zoom events and Ctrl/Cmd-wheel deltas feed the viewer-only zoom path; point-unit touchpad scrolling stays a direct two-axis translation. The viewer keeps the pointer's document position stable by proportionally anchoring both scroll axes. A middle-button drag updates the renderer-owned scroll offsets when zoomed in, without selecting text or activating links.

- **Fullscreen toggle**: The unused portion of the top menu/title bar detects a primary-button double-click and sends `ViewportCommand::Fullscreen` with the inverse of the reported native viewport state. The View menu exposes the same toggle. Controls embedded in the bar continue to consume their own pointer interactions, and an unknown initial fullscreen state is treated as windowed.

- **Wide table scrolling**: Wide markdown / HTML tables are wrapped in a nested `egui::ScrollArea::horizontal()` so columns wider than the content area can still be reached. Plain vertical wheel stays with the outer document scroller; table horizontal movement uses the bottom scrollbar, native horizontal input, or `Shift+vertical-wheel` (routed via `forward_shift_wheel_to_horizontal_scroll` in `crates/egui_commonmark/egui_commonmark/src/parsers/pulldown.rs`) so the cursor crossing a wide table during normal scrolling does not change its horizontal offset.

- **Resizable tables**: Markdown and HTML tables use
  `egui_extras::TableBuilder` with resizable columns. Table width defaults to
  hugging the available content and can overflow into the horizontal scroller.

- **Math and Mermaid**: The renderer converts LaTeX through mitex and Typst,
  and Mermaid through merman/resvg. Both expensive rendering paths run on
  background threads and publish results through per-tab `CommonMarkCache`
  channels. Images and Mermaid diagrams can open in a zoomable lightbox.

- **Global Allocator**: mimalloc for performance

## Key Libraries

| Crate | Purpose |
|-------|---------|
| eframe/egui 0.33 | GUI framework (glow backend for Wayland) |
| egui_commonmark_extended 0.25 | Vendored Markdown renderer with typography, search, math, Mermaid, and table extensions |
| notify 6.1 + notify-debouncer-mini 0.4 | File watching |
| rfd | Native file dialogs |
| clap | CLI argument parsing |
| regex | Header parsing for outline |

## Rendering Flow

```
update() → check_file_changes() → reload affected tabs
         → poll pending GVFS scan and active animations
         → Apply theme, app scaling
         → TopBottomPanel (menu bar + LIVE indicator + file path)
         → TopBottomPanel (error bar, if any)
         → TopBottomPanel (tab bar) → render_tab_bar()
         → SidePanel::left (file explorer) → render_file_explorer()
           → render_tree_node() (recursive)
         → CentralPanel → render_tab_content()
           → SidePanel::right (outline, if show_outline && headers exist)
           → CommonMarkViewer::show_scrollable (viewer zoom + middle-button pan)
           → check_link_hooks() → handle navigation
         → Lightbox and drag-and-drop overlays
```

The renderer owns the document `ScrollArea` through `show_scrollable`; it
keeps the scroll state and layout cache aligned with viewer zoom and panning.

## Custom Tab System

The tab system uses a simple `Vec<Tab>` with an `active_tab` index:
- Tab bar rendered using `ui.selectable_label()` in a horizontal scroll area
- Close button (×) shown on hover or for active tab
- Context menu with "Close" and "Close Others" options
- Middle-click to close tabs
- Ctrl+Click on links opens in a new tab
- Regular click navigates within the current tab
- Each tab maintains independent navigation history (Alt+Left/Right)
- Session restore opens previously open tabs and restores active tab
- Closing the last tab is allowed; `tabs` may be empty, which renders the **welcome page** (issue #28). `render_welcome` (shown from `render_tab_content` when no tab is active) has Open File / Open Folder buttons and a recent-files list. `push_recent`/`record_recent` maintain `recent_files` (deduped, capped at `RECENT_FILES_CAP`, persisted). A fresh launch with no file shows it too — the old built-in sample document was removed.

## File Explorer

Left sidebar showing all markdown files in a hierarchical directory tree:
- Root directory determined by: CLI file → persisted state → first open tab → cwd
- Can be re-pointed at runtime via File → Open Folder… (`open_folder_dialog` → `FileExplorer::set_root`)
- Shallow initial scan; directory children load lazily on expansion
- Filters: .md, .markdown, .txt files only
- Skip hidden files (starting with .)
- Shows directories without recursively proving that they contain Markdown
- Directories sort first; users can sort by name or modification time in either direction
- Toggle visibility with Ctrl+Shift+E
- Click file to open in new tab (or focus if already open)
- Expand/collapse directories with arrow buttons
- Refresh button to rescan directory
- Local visible directories receive non-recursive watcher updates; GVFS roots use background scans and manual explorer refresh
- Session persistence for: visibility, root directory, expanded directories

## Repository and Test Layout

- The root Cargo package builds the `mkdv` binary and patches crates.io to
  use the vendored renderer crates.
- `crates/egui_commonmark/Cargo.toml` is a separate nested workspace containing
  the renderer, backend, and proc-macro crates.
- `cargo test` at the repository root tests application logic. Renderer tests
  must be run separately with
  `cargo test --manifest-path crates/egui_commonmark/Cargo.toml --workspace`.
- GitHub Actions formats, lints, builds, and tests the root package. Cargo is
  the sole distribution path.
