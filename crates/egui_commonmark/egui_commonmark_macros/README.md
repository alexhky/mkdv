# Compile-time Markdown macros for egui_commonmark_extended

[![Crate](https://img.shields.io/crates/v/egui_commonmark_macros_extended.svg)](https://crates.io/crates/egui_commonmark_macros_extended)
[![Documentation](https://docs.rs/egui_commonmark_macros_extended/badge.svg)](https://docs.rs/egui_commonmark_macros_extended)

<img src="https://raw.githubusercontent.com/lampsitter/egui_commonmark/master/assets/example-v3.png" alt="showcase" width=280/>

This crate is `egui_commonmark_extended`'s compile-time variant. Use it through
`egui_commonmark_extended` by enabling the `macros` feature.


## Usage

In Cargo.toml:

```toml
egui_commonmark_extended = { version = "0.25", features = ["macros"] }
# Specify what image formats you want to use
image = { version = "0.25", default-features = false, features = ["png"] }
```

### Example

```rust
use egui_commonmark_extended::{commonmark, CommonMarkCache};
let mut cache = CommonMarkCache::default();
let _response = commonmark!(ui, &mut cache, "# ATX Heading Level 1");
```

Alternatively you can embed a file

### Example

```rust
use egui_commonmark_extended::{commonmark_str, CommonMarkCache};
let mut cache = CommonMarkCache::default();
commonmark_str!(ui, &mut cache, "content.md");
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
