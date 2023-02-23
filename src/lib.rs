/*!
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

# Examples

```
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
*/

#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

use core::fmt;

#[macro_use]
extern crate pest_derive;
use ordered_multimap::ListOrderedMultimap;
use pest::{error::Error, iterators::Pair, Parser};
use smartstring::alias::String;
use thiserror::Error;

use derive::{BmlParser, Rule};
use BmlKind::{Attr, Elem, Root};

pub(crate) mod derive {
	#[derive(Parser)]
	#[grammar = "bml.pest"]
	pub struct BmlParser;
}

/// BML parser error.
#[derive(Debug, PartialEq, Eq, Clone, Error)]
#[error("Invalid BML\n{}", inner)]
pub struct BmlError {
	inner: Error<Rule>,
}

type BmlName = String;
type BmlData = String;

#[derive(Debug, Eq, Clone, Copy)]
enum BmlKind {
	Root { indent: BmlIndent },
	Elem,
	Attr { quote: bool },
}

impl PartialEq for BmlKind {
	fn eq(&self, other: &BmlKind) -> bool {
		matches!(
			(self, other),
			(Root { .. }, Root { .. }) | (Elem, Elem) | (Attr { .. }, Attr { .. })
		)
	}
}

impl Default for BmlKind {
	#[inline]
	fn default() -> Self {
		Root {
			indent: BmlIndent::default(),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct BmlIndent {
	string: &'static str,
	repeat: usize,
}

impl BmlIndent {
	#[inline]
	fn next(mut self) -> Self {
		self.repeat += 1;
		self
	}
}

impl Default for BmlIndent {
	#[inline]
	fn default() -> Self {
		Self {
			string: "  ",
			repeat: 0,
		}
	}
}

impl fmt::Display for BmlIndent {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.string.repeat(self.repeat))
	}
}

/// BML node comprising data [`Self::lines()`] and child [`Self::nodes()`].
///
/// By design, attributes are considered child nodes as well but carry a flag marking them as
/// attributes for serialization purpose only.
#[derive(Debug, Eq, Clone, Default)]
pub struct BmlNode {
	kind: BmlKind,
	data: BmlData,
	node: ListOrderedMultimap<BmlName, BmlNode>,
}

impl BmlNode {
	/// Value comprising data lines with `'\n'` removed from last line.
	#[must_use]
	#[inline]
	pub fn value(&self) -> &str {
		&self.data[..self.data.len() - 1]
	}
	/// Iterator over data lines.
	#[must_use]
	#[inline]
	pub fn lines(&self) -> impl DoubleEndedIterator<Item = &str> {
		self.data.lines()
	}
	/// Iterator over child nodes as `(name, node)` tuples.
	#[must_use]
	#[inline]
	pub fn nodes(&self) -> impl DoubleEndedIterator<Item = (&str, &BmlNode)> + ExactSizeIterator {
		self.node.iter().map(|(name, node)| (name.as_str(), node))
	}
	/// Iterator over child nodes of `name`.
	///
	/// Complexity: *O(1)*
	#[must_use]
	#[inline]
	pub fn named(
		&self,
		name: &str,
	) -> impl DoubleEndedIterator<Item = &BmlNode> + ExactSizeIterator {
		self.node.get_all(name)
	}
	/// Indent `string` of child nodes and level as `repeat` times `string`.
	///
	/// Default is two spaces as in `"  "` and no root indent (`0`). Usual alternative is a
	/// tabulator as in `"\t"` and no root indent (`0`).
	///
	/// # Panics
	///
	/// Panics if invoked on non-root node.
	#[inline]
	pub fn set_indent(&mut self, string: &'static str, repeat: usize) {
		match self.kind {
			Root { ref mut indent } => *indent = BmlIndent { string, repeat },
			_ => panic!("BML indent can be set for root node only"),
		}
	}
	#[must_use]
	#[inline]
	fn root() -> Self {
		Self {
			kind: Root {
				indent: BmlIndent::default(),
			},
			..Self::default()
		}
	}
	#[must_use]
	#[inline]
	fn elem() -> Self {
		Self {
			kind: Elem,
			..Self::default()
		}
	}
	#[must_use]
	#[inline]
	fn attr() -> Self {
		Self {
			kind: Attr { quote: true },
			..Self::default()
		}
	}
	#[inline]
	fn append(&mut self, (name, node): (BmlName, BmlNode)) {
		self.node.append(name, node);
	}
	fn serialize(&self, f: &mut fmt::Formatter, name: &str, indent: BmlIndent) -> fmt::Result {
		match self.kind {
			Root { indent } => {
				let mut nodes = self.nodes().peekable();
				while let Some((name, node)) = nodes.next() {
					node.serialize(f, name, indent)?;
					if nodes.peek().is_some() {
						writeln!(f)?;
					}
				}
			}
			Elem => {
				write!(f, "{indent}{name}")?;
				let indent = indent.next();
				let mut attrs = 0;
				for (name, attr) in self
					.nodes()
					.take_while(|(_name, node)| matches!(node.kind, Attr { .. }))
				{
					attrs += 1;
					attr.serialize(f, name, indent)?;
				}
				let mut lines = self.lines();
				let first = lines.next();
				let second = lines.next();
				if attrs == 0 && first.is_some() && second.is_none() {
					writeln!(f, ": {}", first.unwrap())?;
				} else {
					writeln!(f)?;
					for line in self.lines() {
						writeln!(f, "{indent}:{line}")?;
					}
				}
				for (name, elem) in self.nodes().skip(attrs) {
					elem.serialize(f, name, indent)?;
				}
			}
			Attr { quote } => {
				write!(f, " {name}")?;
				if let Some(line) = self.lines().next() {
					if quote {
						write!(f, "=\"{line}\"")?;
					} else {
						write!(f, "={line}")?;
					}
				}
			}
		}
		Ok(())
	}
}

impl PartialEq for BmlNode {
	#[inline]
	fn eq(&self, other: &BmlNode) -> bool {
		self.data == other.data && self.node == other.node
	}
}

impl TryFrom<&str> for BmlNode {
	type Error = BmlError;

	fn try_from(bml: &str) -> Result<Self, Self::Error> {
		fn parse_node(pair: Pair<Rule>) -> (BmlName, BmlNode) {
			let mut name = BmlName::new();
			let mut node = BmlNode::elem();
			for pair in pair.into_inner() {
				match pair.as_rule() {
					Rule::name => name = pair.as_str().into(),
					Rule::data => {
						node.data.push_str(pair.into_inner().as_str());
						node.data.push('\n');
					}
					Rule::attr => {
						let mut name = BmlName::new();
						let mut attr = BmlNode::attr();
						for pair in pair.into_inner() {
							match pair.as_rule() {
								Rule::name => name = pair.as_str().into(),
								Rule::data => {
									for data in pair.into_inner() {
										if data.as_rule() == Rule::space_data_inner {
											attr.kind = Attr { quote: false }
										};
										attr.data.push_str(data.as_str());
										attr.data.push('\n');
									}
								}
								_ => unreachable!(),
							}
						}
						node.append((name, attr));
					}
					Rule::node => node.append(parse_node(pair)),
					_ => unreachable!(),
				}
			}
			(name, node)
		}

		let mut root = BmlNode::root();
		for pair in BmlParser::parse(Rule::root, bml).map_err(|inner| BmlError { inner })? {
			match pair.as_rule() {
				Rule::node => root.append(parse_node(pair)),
				Rule::EOI => (),
				_ => unreachable!(),
			}
		}
		Ok(root)
	}
}

impl fmt::Display for BmlNode {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.serialize(f, "", BmlIndent::default())
	}
}

#[cfg(test)]
mod tests {
	use super::BmlNode;

	#[test]
	fn ordered_iteration() {
		let root = BmlNode::try_from("0:a\n1:b\n2:c\n1:d\n3:e\n").unwrap();
		assert_eq!(
			root.nodes()
				.map(|(name, node)| (name, node.value()))
				.collect::<Vec<_>>(),
			vec![("0", "a"), ("1", "b"), ("2", "c"), ("1", "d"), ("3", "e")]
		);
	}
}
