# Repo Info
## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| GUI Framework | egui 0.33 + eframe (glow backend) |
| Markdown | egui_commonmark_extended 0.25 (vendored fork with app-specific rendering extensions) |
| File Watching | notify 6.1 + notify-debouncer-mini |
| File Dialogs | rfd |
| CLI | clap |
| Allocator | mimalloc |

**Binary:** ~35 MB release build (includes syntect, mermaid renderer, math rendering, image support, Wayland+X11).

## Before You Code

**STOP and do these checks first:**

1. **Check branch:** `git branch --show-current` (must not be `main`)
2. **Read files you'll modify** (use Read tool, don't rely on memory)
3. **Search LESSONS.md:** `grep -i "keyword" docs/LESSONS.md`
4. **Check recent changes:** `git log --oneline -5`

**For feature work, follow the egui immediate-mode pattern:**
```
State → Logic → UI → Async
```
See `docs/EGUI_WORKFLOW.md` for the complete guide.

## Project Structure

This checkout is a standard Git working tree. The application is a root Cargo
package, while the vendored renderer is a separate nested Cargo workspace.

```text
mkdv/
├── .git/                       # Git database for this checkout
├── .claude/                    # Local agent settings, hooks, and rules
├── src/main.rs                 # Application and root unit tests
├── crates/egui_commonmark/     # Nested renderer workspace
├── data/                       # GNOME desktop entry template
├── docs/                       # Architecture, plans, lessons, and devlogs
├── scripts/                    # Cargo publishing helper
├── Cargo.toml
└── README.md
```

## Documentation

**Auto-loaded** (from `.claude/rules/`):
- `egui-patterns.md` - **Critical** - Never parse in UI code, State→Logic→UI→Async
- `build-commands.md` - cargo build, run, clippy, install, and launcher registration
- `devlog-workflow.md` - How to document feature implementations
- `worktree-workflow.md` - How to create and manage worktrees
- `system-dependencies.md` - Arch Linux packages
- `refactoring-rules.md` - **Read before refactoring** - prevent regressions
- `context-awareness.md` - **Read before coding** - ensure fresh context

**Imported** (via `@path`):
- @docs/EGUI_WORKFLOW.md - **Read before coding** - State→Logic→UI→Async pattern
- @docs/ARCHITECTURE.md - Core components, libraries, rendering flow
- @docs/KEYBOARD_SHORTCUTS.md - All keyboard shortcuts
- @docs/TARGET_METRICS.md - Performance targets and planned features
- @docs/LESSONS.md - **Check before debugging** - reusable fixes and gotchas

## Quick Reference

```bash
# Create a branch before editing (main is protected)
git switch -c <type>/<short-description>

# Build and verify the application
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run -- --foreground README.md

# Test the separate renderer workspace
cargo test --manifest-path crates/egui_commonmark/Cargo.toml --workspace
```

## Branch Protection

A Claude Code hook prevents editing files on `main`. Create a feature or fix
branch before changing tracked files; a separate Git worktree is optional.
