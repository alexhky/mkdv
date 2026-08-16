# An extended CommonMark viewer for [egui](https://github.com/emilk/egui)

[![Crate](https://img.shields.io/crates/v/egui_commonmark_extended.svg)](https://crates.io/crates/egui_commonmark_extended)
[![Documentation](https://docs.rs/egui_commonmark_extended/badge.svg)](https://docs.rs/egui_commonmark_extended)

<img src="https://raw.githubusercontent.com/lampsitter/egui_commonmark/master/assets/example-v4.png" alt="showcase" width=280/>

This is mkdv's vendored fork of `egui_commonmark`, published as
`egui_commonmark_extended` 0.25. In addition to CommonMark and GitHub-flavored
Markdown, the fork contains typography controls, search highlighting,
heading-position tracking, emoji shortcodes, resizable tables, Mermaid, and
Typst-backed math rendering.

## Usage

In Cargo.toml:

```toml
egui_commonmark_extended = "0.25"
# Specify what image formats you want to use
image = { version = "0.25", default-features = false, features = ["png"] }
```

```rust
use egui_commonmark_extended::*;
let markdown =
r"# Hello world

* A list
* [ ] Checkbox
";

let mut cache = CommonMarkCache::default();
CommonMarkViewer::new().show(ui, &mut cache, markdown);
```


## Compile time evaluation of markdown

If you want to embed markdown directly the binary then you can enable the `macros` feature.
This will do the parsing of the markdown at compile time and output egui widgets.

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


## Features

* `macros`: macros for compile time parsing of markdown
* `better_syntax_highlighting`: Syntax highlighting inside code blocks with
  [`syntect`](https://crates.io/crates/syntect)
* `svg`: Support for viewing svg images
* `fetch`: Images with urls will be downloaded and displayed
* `embedded_image`: Load base64 image data URLs from within Markdown files
* `mermaid`: Render Mermaid code blocks through merman/resvg
* `math`: Render LaTeX math through mitex and Typst
* `svg_text`: Enable system-font text rendering in SVG images


## Examples

For an easy intro check out the `hello_world` example. To see the extended
renderer features together, check out the `book` example.

## FAQ

### URL is not displayed when hovering over a link

By default egui does not show urls when you hover hyperlinks. To enable it,
you can do the following before calling any ui related functions:

```rust
ui.style_mut().url_in_tooltip = true;
```

## MSRV Policy

This crate uses the same MSRV as the latest released egui version.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
