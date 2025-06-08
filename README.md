# pichu

[![CI](https://github.com/bahlo/pichu/actions/workflows/ci.yml/badge.svg)](https://github.com/bahlo/pichu/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/pichu.svg)](https://crates.io/crates/pichu)
[![docs.rs](https://docs.rs/pichu/badge.svg)](https://docs.rs/pichu/)
[![License](https://img.shields.io/crates/l/pichu)](LICENSE-APACHE)

The static site generator designed to evolve with your needs.

## Quickstart

Pichu provides the building blocks to build your own static site generator.
Parse a directory of Markdown files, including typed frontmatter, and render
them individuall and in a collection using your favorite template engine.

Batteries included, but easily swappable:
If you've outgrown the default Markdown implementation, you're encouraged to
copy-paste the implementation and plug it in instead.
Or bring your own!

```rust
pichu::glob("content/blog/*.md")?
    .parse_markdown::<Blogpost>()?
    .render_each(render_blog_post, |post| format!("dist/blog/{}/index.html", post.basename))?
    .render_all(render_blog, "dist/blog/index.html")?;

pichu::render_sass("assets/main.scss", "dist/main.css")?;

pichu::copy_dir("static/", "dist/")?;
```

## Examples

* [pichu-starter](https://github.com/bahlo/pichu-starter), a template repository to get started quickly
* [arne.me](https://github.com/bahlo/arne.me)

## Features

* `markdown` (default): Enable the [`parse_markdown`](https://docs.rs/pichu/latest/pichu/struct.Glob.html#method.parse_markdown) method.
* `sass` (default): Enable the [`render_sass`](https://docs.rs/pichu/latest/pichu/fn.render_sass.html) function.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
