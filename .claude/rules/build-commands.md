# Build Commands

```bash
cargo build                                # Debug build
cargo build --release                      # Release build (optimized for size)
cargo run -- --foreground                  # Run attached for logs/debugging
cargo run -- --foreground file.md          # Open a file; live reload is on
cargo run -- --foreground file.md --no-watch  # Disable live reload
cargo fmt --check                          # Formatting gate
cargo clippy --all-targets -- -D warnings  # Root lint gate
cargo test --all-targets                   # Root application tests
cargo install --path .                     # Install the binary from this checkout
cargo run -- --install-desktop             # Register the GNOME launcher entry
```

The release profile is configured for minimal binary size (`opt-level = "z"`, LTO, strip symbols).

The vendored renderer is a separate nested workspace, so its tests need an
explicit command:

```bash
cargo test --manifest-path crates/egui_commonmark/Cargo.toml --workspace
```
