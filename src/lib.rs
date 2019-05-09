// Copyright (c) 2019 Rouven Spreckels <n3vu0r@qu1x.org>
//
// Usage of the works is permitted provided that
// this instrument is retained with the works, so that
// any entity that uses the works is notified of this instrument.
//
// DISCLAIMER: THE WORKS ARE WITHOUT WARRANTY.

//! **BML Markup Language**
//!
//! Based on [PEG], see the interactive parser implementation at [pest.rs].
//!
//! [BML] is used by the icarus database of [higan]/[bsnes].
//!
//! [BML]: https://news.ycombinator.com/item?id=8645591
//! [PEG]: https://en.wikipedia.org/wiki/Parsing_expression_grammar
//! [pest.rs]: https://pest.rs/?bin=ov6wy#editor
//! [higan]: https://byuu.org/emulation/higan/
//! [bsnes]: https://byuu.org/emulation/bsnes/
//!
//! # Usage
//!
//! This crate works on Rust stable channel. It is
//! [on crates.io](https://crates.io/crates/bml) and can be used by adding
//! `bml` to the dependencies in your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bml = "0.1"
//! ```
//!
//! # Examples
//!
//! ```
//! use bml::{BmlNode, FromStr};
//!
//! let root = BmlNode::from_str(concat!(
//! 	"server\n",
//! 	"  path: /core/www/\n",
//! 	"  host: example.com\n",
//! 	"  port: 80\n",
//! 	"  service: true\n",
//! 	"  proxy\n",
//! 	"    host: proxy.example.com\n",
//! 	"    port: 8080\n",
//! 	"    authentication: plain\n",
//! 	"  description\n",
//! 	"    :Primary web-facing server\n",
//! 	"    :Provides commerce-related functionality\n",
//! 	"\n",
//! 	"server\n",
//! 	"  // ...\n",
//! 	"  proxy host=\"proxy.example.com\" port=\"8080\"\n",
//! 	"    authentication: plain\n",
//! )).unwrap();
//!
//! let (name, node) = root.nodes().next().unwrap();
//!
//! assert_eq!(name, "server");
//! assert_eq!(node.named("port").next().unwrap()
//! 	.lines().next().unwrap(), "80");
//! ```

#[macro_use] extern crate pest_derive;

pub use pest::Parser;
pub use std::str::FromStr;
use std::fmt;

use pest::error::Error;
use pest::iterators::Pair;

use BmlKind::*;

/// BML node comprising data `lines()` and child `nodes()`.
///
/// By design, attributes are considered child nodes as well but carry a flag
/// marking them as attributes for serialization purpose only.
#[derive(Debug, Eq, Clone, Default)]
pub struct BmlNode {
	kind: BmlKind,
	data: String,
	node: Vec<(String, BmlNode)>,
}

impl BmlNode {
	/// Iterator over data lines.
	pub fn lines(&self) -> impl Iterator<Item = &str> {
		self.data.lines()
	}
	/// Iterator over child nodes as `(name, node)` tuples.
	pub fn nodes(&self) -> impl Iterator<Item = (&String, &BmlNode)> {
		// TODO Remove `map()` when using ordered multi map.
		self.node.iter().map(|(name, node)| (name, node))
	}
	/// Iterator over child nodes of `name`.
	pub fn named(&self, name: &str) -> impl Iterator<Item = &BmlNode> {
		// TODO Use ordered multi map.
		let name = name.to_string();
		self.node.iter().filter_map(move |(k, v)|
			if k == &name { Some(v) } else { None })
	}
	pub fn set_indent(&mut self, string: &'static str, repeat: usize) {
		match self.kind {
			Root { ref mut indent } => *indent = BmlIndent { string, repeat },
			_ => panic!("BML indent can be set for root node only"),
		}
	}
	fn root() -> Self {
		Self { kind: Root { indent: BmlIndent::default() }, ..Self::default() }
	}
	fn elem() -> Self {
		Self { kind: Elem, ..Self::default() }
	}
	fn attr() -> Self {
		Self { kind: Attr { quote: true }, ..Self::default() }
	}
	fn append(&mut self, (name, node): (String, BmlNode)) {
		self.node.push((name, node));
	}
	fn serialize(&self, f: &mut fmt::Formatter,
		name: &str, indent: BmlIndent,
	) -> fmt::Result {
		match self.kind {
			Root { indent } => {
				let mut nodes = self.nodes().peekable();
				while let Some((name, node)) = nodes.next() {
					node.serialize(f, name, indent)?;
					if nodes.peek().is_some() {
						writeln!(f)?;
					}
				}
			},
			Elem => {
				write!(f, "{}{}", indent, name)?;
				let indent = indent.next();
				let mut attrs = 0;
				for (name, attr) in self.nodes().take_while(|(_name, node)|
					if let Attr { .. } = node.kind
						{ true} else { false }
				) {
					attrs += 1;
					attr.serialize(f, name, indent)?;
				}
				let mut lines = self.lines();
				let line0 = lines.next();
				let line1 = lines.next();
				if attrs == 0 && line0.is_some() && line1.is_none() {
					writeln!(f, ": {}", line0.unwrap())?;
				} else {
					writeln!(f)?;
					for line in self.lines() {
						writeln!(f, "{}:{}", indent, line)?;
					}
				}
				for (name, elem) in self.nodes().skip(attrs) {
					elem.serialize(f, name, indent)?;
				}
			},
			Attr { quote } => {
				write!(f, " {}", name)?;
				if let Some(line) = self.lines().next() {
					if quote {
						write!(f, "=\"{}\"", line)?;
					} else {
						write!(f, "={}", line)?;
					}
				}
			},
		}
		Ok(())
	}
}

impl PartialEq for BmlNode {
	fn eq(&self, other: &BmlNode) -> bool {
		self.data == other.data && self.node == other.node
	}
}

impl FromStr for BmlNode {
	type Err = BmlError;

	fn from_str(bml: &str) -> Result<Self, Self::Err> {
		fn parse_node(pair: Pair<Rule>) -> (String, BmlNode) {
			let mut name = String::new();
			let mut node = BmlNode::elem();
			for pair in pair.into_inner() {
				match pair.as_rule() {
					Rule::name => name = pair.as_str().into(),
					Rule::data => {
						node.data += pair.into_inner().as_str();
						node.data += "\n";
					},
					Rule::attr => {
						let mut name = String::new();
						let mut attr = BmlNode::attr();
						for pair in pair.into_inner() {
							match pair.as_rule() {
								Rule::name => name = pair.as_str().into(),
								Rule::data => for data in pair.into_inner() {
									if data.as_rule() == Rule::space_data_inner
										{ attr.kind = Attr { quote: false } };
									attr.data += data.as_str();
									attr.data += "\n";
								},
								_ => unreachable!(),
							}
						}
						node.append((name, attr));
					},
					Rule::node => node.append(parse_node(pair)),
					_ => unreachable!(),
				}
			}
			(name, node)
		}
		let mut root = BmlNode::root();
		for pair in BmlParser::parse(Rule::root, &bml)? {
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
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.serialize(f, "", BmlIndent::default())
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct BmlIndent {
	string: &'static str,
	repeat: usize,
}

impl BmlIndent {
	fn next(mut self) -> Self {
		self.repeat += 1;
		self
	}
}

impl Default for BmlIndent {
	fn default() -> Self {
		Self { string: "  ".into(), repeat: 0 }
	}
}

impl fmt::Display for BmlIndent {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.string.repeat(self.repeat))
	}
}

#[derive(Debug, Eq, Clone, Copy)]
enum BmlKind {
	Root { indent: BmlIndent },
	Elem,
	Attr { quote: bool },
}

impl PartialEq for BmlKind {
	fn eq(&self, other: &BmlKind) -> bool {
		match (self, other) {
			(Root { .. }, Root { .. }) => true,
			(Elem, Elem) => true,
			(Attr { .. }, Attr { .. }) => true,
			_ => false,
		}
	}
}

impl Default for BmlKind {
	fn default() -> Self {
		Root { indent: BmlIndent::default() }
	}
}

/// BML parser.
#[derive(Parser)]
#[grammar = "bml.pest"]
pub struct BmlParser;

/// BML parser errors.
pub type BmlError = Error<Rule>;
