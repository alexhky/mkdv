# Feature: Publish the Cargo package

**Status:** 🚧 In Progress
**Branch:** `feature/crates-publish`
**Date:** 2026-05-16
**Lines Changed:** TBD

## Summary

`cargo publish` for mkdv initially failed:

```
package `mkdv` depends on `egui_commonmark_extended` with feature `math`
but `egui_commonmark_extended` does not have that feature
```

Re-enable crates.io publish by:

1. Bump vendored fork workspace 0.22.2 → 0.23.0 (`math` feature added since the
   last fork publish).
2. Update root `Cargo.toml` dep to `0.23.0`.
3. Add `scripts/publish-crates.sh` with idempotent publish loop in dependency order +
   sparse-index settle delay.

## Features

- [ ] Bump fork workspace + inter-deps to 0.23.0
- [ ] Update root Cargo.toml `egui_commonmark_extended` dep to 0.23.0
- [ ] Add `scripts/publish-crates.sh`
- [ ] Document the Cargo-only publish flow
- [ ] Rewrite Crates.io section of `PUBLISHING.md`
- [ ] Rewrite "cargo publish rejects vendored forks" lesson in `LESSONS.md`
- [ ] Local pre-flight (cargo check + dry-runs)

## Key Discoveries

### Fork crates already on crates.io — under renamed identifiers

The three fork crates (`egui_commonmark_extended`,
`egui_commonmark_backend_extended`, `egui_commonmark_macros_extended`) were
published at v0.22.2 on 2026-03-04 by `aydiler` (same day mkdv v0.1.2
shipped). The renamed `_extended` identifiers mean no upstream conflict.

The blocker isn't "publish under a new name" (LESSONS.md / memory's
recommendation). It's "republish with feature parity": v0.22.2 on the registry
lacks `math` (the feature was added to the local fork *after* that publish), so
`cargo publish` for mkdv fails feature-resolution.

Verified via crates.io API:

```
$ curl -s https://crates.io/api/v1/crates/egui_commonmark_extended/0.22.2 \
   | jq '.version.features.math'
null
```

### `[patch.crates-io]` is safe to keep

`cargo publish` ignores `[patch.crates-io]` during its verify step (resolves
deps against the registry directly). Once the registry version matches what
the root Cargo.toml asks for, the patch becomes neutral — useful for local dev
between fork bumps, harmless for publish.

### Sparse-index propagation needs a settle delay

After `cargo publish` for crate A, dependents publishing immediately may fail
with "not in index". Add `sleep 45` between publishes. If still flaky under
crates.io load, bump to 90s.

## Architecture

### New file: `scripts/publish-crates.sh`

Iterates over the publish dep order (backend → macros → extended → mkdv).
Catches "already uploaded" from cargo stderr → treats as success (idempotent
on re-tags). Otherwise propagates failure.

### Cargo publish helper

The helper runs the dependency order (backend → macros → extended → mkdv) and
reads `CARGO_REGISTRY_TOKEN` from the environment. It is a convenience around
Cargo's own publish command, not a separate distribution channel.

## Testing Notes

Local pre-flight before tagging:

- `cargo check --all-features` (still uses patch — verifies local builds)
- `cargo publish --dry-run` on each fork crate in turn (verifies metadata +
  version-newness on registry)
- `cargo publish --dry-run` on mkdv **will fail locally** before the
  fork-at-0.23.0 is published — expected, the failure message should say
  "version 0.23.0 not found" (NOT "feature missing") if feature parity is right

## Future Improvements

- [ ] Bump sleep to 90s if 45s proves flaky under load
- [ ] Eventually: upstream `math` feature to `lampsitter/egui_commonmark` so
      we can drop the fork entirely. Other deviations would also need
      upstreaming (line height, header positions, search highlights, table
      builder, wheel routing) — large effort, deferred.
