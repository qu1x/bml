[![Build Status][]](https://travis-ci.org/qu1x/bml)
[![Downloads][]](https://crates.io/crates/bml)
[![Version][]](https://crates.io/crates/bml)
[![Documentation][]](https://docs.rs/bml)
[![License][]](https://opensource.org/licenses/ISC)

[Build Status]: https://travis-ci.org/qu1x/bml.svg
[Downloads]: https://img.shields.io/crates/d/bml.svg
[Version]: https://img.shields.io/crates/v/bml.svg
[Documentation]: https://docs.rs/bml/badge.svg
[License]: https://img.shields.io/crates/l/bml.svg

# bml

BML Markup Language

[BML] is a simplified [XML] used as static [database], see the [grammar] using [PEG] as input for
the [pest] parser.

In contrast to its C++ [reference] implementation, this Rust implementation parses indents by
pushing them on a stack to compare them instead of counting characters (stack-based-indent) and it
allows tabulators between attributes (tabular-attributes) and between colons and multi-line data
(tabular-colon-data) supporting tabulator-based along with space-based alignments.

Syntax highlighting is trivial, see [vim-bml].

[BML]: https://news.ycombinator.com/item?id=8645591
[XML]: https://en.wikipedia.org/wiki/XML
[database]: https://github.com/ares-emulator/ares/tree/master/mia/Database
[grammar]: https://github.com/qu1x/bml/blob/main/src/bml.pest
[PEG]: https://en.wikipedia.org/wiki/Parsing_expression_grammar
[pest]: https://pest.rs/
[reference]: https://github.com/ares-emulator/ares/blob/master/nall/string/markup/bml.hpp
[vim-bml]: https://github.com/qu1x/vim-bml

## Examples

```rust
use bml::BmlNode;

let root = BmlNode::try_from(concat!(
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
assert_eq!(node.named("port").next().unwrap().value(), "80");
```

## License

Licensed under `ISC`.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
works by you shall be licensed as above, without any additional terms or conditions.
