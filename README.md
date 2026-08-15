# mkdv

[![Crates.io](https://img.shields.io/crates/v/mkdv.svg)](https://crates.io/crates/mkdv)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![GitHub stars](https://img.shields.io/github/stars/aydiler/mkdv)](https://github.com/aydiler/mkdv/stargazers)

A fast, lightweight desktop Markdown viewer built with Rust and egui. Designed for distraction-free reading with excellent typography, syntax highlighting, and LaTeX math — from quick notes to scientific papers.

![mkdv rendering a LaTeX-heavy scientific paper](screenshots/math-rendering.png)

## Features

### Rendering
- **GitHub Flavored Markdown** - Full GFM support including tables, task lists, footnotes, and recognized emoji shortcodes such as `:pushpin:`
- **LaTeX Math** - Inline `$…$` and display `$$…$$` equations rendered via typst + mitex — fractions, sub/superscripts, `\boxed`, accents, matrices, and more — sized and baseline-aligned to the surrounding text
- **Syntax Highlighting** - 200+ languages via syntect with beautiful color schemes
- **Mermaid Diagrams** - Flowcharts, sequence diagrams, and more rendered natively via [merman](https://github.com/Latias94/merman) (click to enlarge)
- **Resizable Table Columns** - Drag column dividers to fit content (new in v0.1.5)
- **HTML Tables** - Rendered as formatted grids with proper cell padding
- **Images & SVG** - Embedded and remote image support (PNG, JPEG, GIF, SVG, HTTP URLs)
- **Unicode Support** - System font fallbacks (Noto, DejaVu) for emojis, CJK, and non-Latin scripts
- **60 FPS Rendering** - Viewport virtualization keeps scroll smooth on 100k+ line docs
- **Typography** - 1.5x line height for optimal readability (WCAG 2.1 compliant)

### Navigation
- **Tab System** - Open multiple documents with tab bar (Ctrl+Click links to open in new tab)
- **In-Document Search (Ctrl+F)** - Find bar with inline highlights, Enter/Shift+Enter to cycle matches
- **File Explorer** - Hierarchical sidebar with lazy-loading directories and sorting options
- **Open Folder** - Use File → Open Folder… to choose and persist the file explorer root
- **Outline Sidebar** - Click-to-navigate table of contents from document headers
- **Navigation Buttons** - Back/forward buttons in title bar for quick history navigation
- **Per-Tab History** - Independent back/forward navigation within each tab (Alt+Left/Right)
- **Internal Links** - Navigate between markdown files with relative links

### View
- **Dark & Light Themes** - Toggle with Ctrl+D
- **Scaling & Viewer Zoom** - Ctrl++/-/0 changes whole-app scaling; touchpad pinch or Ctrl+Scroll zooms the Markdown viewer only (75%–300%). Pinch works on Wayland, where mkdv binds the compositor's gesture protocol directly because winit does not expose touchpad gestures on Linux; on X11 use Ctrl+Scroll. Two-axis touchpad scrolling and middle-drag panning keep the page moving naturally when zoomed in
- **Fullscreen** - Double-click unused space in the top bar to enter or leave fullscreen; the View menu offers the same toggle
- **Keyboard Scrolling** - Scroll documents with ↑/↓ by line or Page Up/Page Down by page when the find bar is closed
- **Live Reload** - Auto-refresh on file changes (enabled by default)

### Usability
- **Drag and Drop** - Drop markdown files onto the window to open
- **Native Dialogs** - System file and folder picker integration
- **Welcome Page & Recent Files** - Open files or folders from the idle screen and reopen recent documents
- **Session Persistence** - Remembers open tabs, theme, app scaling, viewer zoom, and sidebar state
- **Cross-Platform** - Linux, macOS, and Windows support; native X11 and Wayland support on Linux

## Screenshots

### Scientific paper — math, tables, figures, and navigation
*A DESI dark-energy paper: display & inline LaTeX equations, a data table with math headers, an embedded figure, the file explorer, and the click-to-navigate outline — all at once*

![Scientific paper in dark mode](screenshots/math-rendering.png)

### Same document, light mode
*The χ² comparison table, inline math, and the BAO-fit figure rendered on the light theme*

![Scientific paper in light mode](screenshots/math-light.png)

### Figures, equations, and lists together
*Embedded plots with math captions, a checklist with inline equations and emoji, and both sidebars*

![Math, figures and lists](screenshots/model-doc.png)

### Dark Mode — Diagrams & Tables
*Mermaid flowchart, tech-stack table, file explorer, and outline sidebar*

![Dark Mode](screenshots/dark-mode.png)

### Syntax Highlighting
*Rust and YAML code blocks with semantic coloring and a mermaid sequence diagram*

![Syntax Highlighting](screenshots/syntax-highlighting.png)

### Light Mode — Prose & Code
*Documentation with bullet lists, blockquotes, inline code, and bash code blocks*

![Light Mode](screenshots/light-mode.png)

### Tables & Lists
*Troubleshooting table with inline code in cells, ordered list, and resizable columns (drag dividers)*

![Tables](screenshots/tables.png)

### Search (Ctrl+F)
*Find bar with inline highlights and match counter; Enter / Shift+Enter to cycle*

![Search](screenshots/search.png)

### Resizable Table Columns
*Drag column dividers to fit wide content*

![Resizable Tables](screenshots/resizable-tables.png)

## Keyboard Shortcuts

### Tab Management

| Shortcut | Action |
|----------|--------|
| Ctrl+T | New tab (open file dialog) |
| Ctrl+W | Close current tab |
| Ctrl+Tab | Next tab |
| Ctrl+Shift+Tab | Previous tab |
| Ctrl+1-9 | Switch to tab 1-9 |

### Navigation

| Shortcut | Action |
|----------|--------|
| Ctrl+O | Open file dialog |
| Alt+Left | Navigate back in history |
| Alt+Right | Navigate forward in history |
| Click link | Navigate in current tab |
| Ctrl+Click link | Open link in new tab |

### Search

| Shortcut | Action |
|----------|--------|
| Ctrl+F | Open find bar (or refocus if already open) |
| Enter / ↓ | Jump to next match |
| Shift+Enter / ↑ | Jump to previous match |
| Esc | Close find bar and clear highlights |

### View

| Shortcut | Action |
|----------|--------|
| Ctrl+D | Toggle dark/light mode |
| Ctrl+Shift+E | Toggle file explorer |
| Ctrl+Shift+O | Toggle outline sidebar |
| Ctrl++ / Ctrl+= | Increase whole-app scaling |
| Ctrl+- | Decrease whole-app scaling |
| Ctrl+0 | Reset whole-app scaling to 100% |
| ↑ / ↓ (when find bar is closed) | Scroll document up/down by line |
| Page Up / Page Down | Scroll document up/down by page |
| Touchpad pinch over viewer | Zoom the Markdown viewer (Wayland only) |
| Ctrl+Scroll over viewer | Zoom the Markdown viewer |
| Middle-drag over viewer | Pan the zoomed viewer page |
| Shift+Scroll over a wide table | Scroll the table horizontally |

### File Operations

| Shortcut | Action |
|----------|--------|
| F5 | Toggle file watching |
| Ctrl+Q | Quit application |

## Installation

### Cargo (crates.io)

```bash
cargo install mkdv
```

Cargo compiles the application locally and installs `mkdv` into Cargo's binary directory. Update it with `cargo install --force mkdv`.

On Linux, register the executable with the GNOME application launcher:

```bash
mkdv --install-desktop
```

The application also refreshes this entry automatically whenever it starts. It writes `mkdv.desktop` to `$XDG_DATA_HOME/applications` (or `~/.local/share/applications`) and records the absolute installed executable path, so GNOME can launch it even when it does not inherit the shell's `PATH`.

### From Source

```bash
git clone https://github.com/aydiler/mkdv
cd mkdv
cargo install --path .
mkdv --install-desktop   # Linux/GNOME only
```

`cargo install --path .` uses Cargo's full release profile and may take several
minutes on a clean checkout. For repeated local development, use the faster
optimized profile instead:

```bash
cargo install --offline --locked --profile release-dev --path .
```

For the quickest edit/test loop, run `cargo check --offline` followed by
`cargo test --offline`; keep `target/` so Cargo can reuse compiled dependencies.

### System Dependencies (Arch Linux)

Only needed for `cargo install` / building from source:

```bash
sudo pacman -S --needed \
    base-devel clang pkg-config \
    libxcb libxkbcommon openssl \
    gtk3 fontconfig dbus zenity \
    xdg-desktop-portal xdg-desktop-portal-gtk
```

## Usage

```bash
# Open a file and return the terminal prompt (live reload is enabled by default)
mkdv README.md

# Keep the viewer attached to the terminal for debugging/logs
mkdv --foreground README.md

# Disable live reload
mkdv README.md --no-watch
```

Run `mkdv` with no file to start on the welcome page, then choose Open File, Open Folder, or a recent document. In the app, use File → Open File… or Ctrl+O to open a document, and File → Open Folder… to choose the file explorer root.

When launched from a terminal, `mkdv` detaches by default so the shell prompt is available while the window stays open. Use `--foreground` when you want terminal logs or blocking process behavior.

## Technical Details

- **Binary size**: ~35 MB (includes syntax highlighting, mermaid renderer, math rendering, image support, X11+Wayland).
- **Startup time**: < 200ms
- **Rendering**: 60 FPS with viewport-based clipping
- **Memory**: Uses mimalloc for improved allocation performance
- **Platforms**: Linux x86_64 (X11 and Wayland), macOS arm64, and Windows x86_64 via the glow backend

### Built With

- [eframe/egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [egui_commonmark](https://github.com/lampsitter/egui_commonmark) - Markdown rendering (vendored fork with typography, math, and alignment improvements)
- [emojis](https://crates.io/crates/emojis) - GitHub/gemoji shortcode lookup data (`(MIT OR Apache-2.0) AND Unicode-3.0`; see `THIRD_PARTY_NOTICES`)
- [typst](https://github.com/typst/typst) + [mitex](https://github.com/mitex-rs/mitex) - LaTeX math rendering (LaTeX → typst → rasterized inline)
- [merman](https://github.com/Latias94/merman) - Mermaid diagram rendering
- [syntect](https://github.com/trishume/syntect) - Syntax highlighting
- [notify](https://github.com/notify-rs/notify) - File watching
- [rfd](https://github.com/PolyMeilex/rfd) - Native file dialogs

## License

MIT
