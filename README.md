# bml

**BML Markup Language**

[![Build Status][]](https://travis-ci.org/qu1x/bml)
[![Downloads][]](https://crates.io/crates/bml)
[![Version][]](https://crates.io/crates/bml)
[![Documentation][]](https://docs.rs/bml)
[![License][]](https://opensource.org/licenses/Fair)

[Build Status]: https://travis-ci.org/qu1x/bml.svg
[Downloads]: https://img.shields.io/crates/d/bml.svg
[Version]: https://img.shields.io/crates/v/bml.svg
[Documentation]: https://docs.rs/bml/badge.svg
[License]: https://img.shields.io/crates/l/bml.svg

Based on [PEG], see the interactive parser implementation at [pest.rs].

[BML] is used by the icarus database of [higan]/[bsnes].

[BML]: https://news.ycombinator.com/item?id=8645591
[PEG]: https://en.wikipedia.org/wiki/Parsing_expression_grammar
[pest.rs]: https://pest.rs/?bin=ov6wy#editor
[higan]: https://byuu.org/emulation/higan/
[bsnes]: https://byuu.org/emulation/bsnes/

## Contents

  * [Usage](#usage)
  * [Examples](#examples)
  * [License](#license)
  * [Contribution](#contribution)

## Usage

This crate works on Rust stable channel. It is
[on crates.io](https://crates.io/crates/bml) and can be used by adding
`bml` to the dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
bml = "0.1"
```

Use **experimental** `ordered-multimap` feature on Rust nightly channel:

```toml
[dependencies]
bml = { version = "0.1", features = ["ordered-multimap"] }
```

## Examples

```rust
use bml::{BmlNode, FromStr};

let root = BmlNode::from_str(concat!(
	"server\n",
	"  path: /core/www/\n",
	"  host: example.com\n",
	"  port: 80\n",
	"  service: true\n",
	"  proxy\n",
	"    host: proxy.example.com\n",
	"    port: 8080\n",
	"    authentication: plain\n",
	"  description\n",
	"    :Primary web-facing server\n",
	"    :Provides commerce-related functionality\n",
	"\n",
	"server\n",
	"  // ...\n",
	"  proxy host=\"proxy.example.com\" port=\"8080\"\n",
	"    authentication: plain\n",
)).unwrap();

let (name, node) = root.nodes().next().unwrap();

assert_eq!(name, "server");
assert_eq!(node.named("port").next().unwrap()
	.lines().next().unwrap(), "80");
```

## License

Copyright (c) 2019 Rouven Spreckels <n3vu0r@qu1x.org>

Usage of the works is permitted provided that
this instrument is retained with the works, so that
any entity that uses the works is notified of this instrument.

DISCLAIMER: THE WORKS ARE WITHOUT WARRANTY.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the works by you shall be licensed as above, without any
additional terms or conditions.
