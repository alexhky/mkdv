# Publishing Guide

`mkdv` is distributed through Cargo only. There are no platform-specific
distribution assets to maintain.

## crates.io

Before publishing, verify the package locally:

```bash
cargo fmt --all -- --check
cargo test --offline --all-targets
cargo package --allow-dirty --no-verify
```

Set a crates.io token with publish access, then publish the vendored renderer
crates (only when their versions changed) before publishing the application:

```bash
export CARGO_REGISTRY_TOKEN=<your-token>
cargo publish --manifest-path crates/egui_commonmark/egui_commonmark_backend/Cargo.toml
cargo publish --manifest-path crates/egui_commonmark/egui_commonmark_macros/Cargo.toml
cargo publish --manifest-path crates/egui_commonmark/egui_commonmark/Cargo.toml
cargo publish
```

The root package's `[patch.crates-io]` entries are used for local development;
Cargo ignores them when publishing and resolves the pinned renderer crates from
the registry.

## Build profiles and verification speed

The root package enables syntax highlighting, Mermaid, SVG, and Typst-based
math rendering, so the first dependency build is intentionally substantial.
Keep Cargo's `target/` cache between checks and use the fast commands below for
the normal edit loop:

```bash
cargo check --offline
cargo test --offline
```

The standard `cargo build --release` and `cargo install` commands use the
release profile in `Cargo.toml`. It favors a small final binary with size
optimization, full LTO, and one code-generation unit, so it can take much
longer than a development build. For a local installed binary, use:

```bash
cargo install --offline --locked --profile release-dev --path .
```

Use the full release profile for the artifact that will be published. Avoid
`cargo clean` unless a complete rebuild is intentional; it removes the
incremental dependency cache.

## Release checklist

- [ ] Update `version` in `Cargo.toml`.
- [ ] Update the vendored renderer version and root dependency pin if its source changed.
- [ ] Run the verification commands above.
- [ ] Generate the changelog with `git-cliff -o CHANGELOG.md` if desired.
- [ ] Publish with `cargo publish`.
- [ ] Verify installation with `cargo install mkdv`.

On Linux, the installed executable can register its GNOME launcher entry with:

```bash
mkdv --install-desktop
```
