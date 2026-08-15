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
