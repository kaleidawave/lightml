#![allow(clippy::result_unit_err)]
#![doc = include_str!("../README.md")]

pub use lexer::Lexer;
use std::borrow::Cow;

mod lexer;
pub mod matching;
pub mod operations;

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// Name of the element (TODO or reference to element)
    pub tag_name: String,
    pub attributes: Vec<Attribute>,
    pub children: ElementChildren,
}

pub type Children = Vec<Node>;

#[derive(Debug, Clone, PartialEq)]
pub enum ElementChildren {
    Children(Children),
    /// For script + style elements
    Literal(String),
    /// For `img` elements etc
    SelfClosing,
}

impl From<Element> for Node {
    fn from(value: Element) -> Node {
        Node::Element(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub html_element: Element,
}

impl Document {
    pub fn from_reader(reader: &mut crate::Lexer) -> Result<Self, ()> {
        // TODO temp
        let lowercase = reader.is_operator_advance("<!DOCTYPE html>");
        if !lowercase {
            let _ = reader.is_operator_advance("<!doctype html>");
        }
        Element::from_reader(reader).map(|html_element| Document { html_element })
    }
}

impl Element {
    pub fn from_reader(reader: &mut crate::Lexer) -> Result<Self, ()> {
        reader.expect_start('<')?;
        let tag_name = reader.parse_identifier("Element name")?.to_owned();
        let mut attributes = Vec::new();
        // TODO spread attributes
        // Kind of weird / not clear conditions for breaking out of while loop
        loop {
            reader.skip();
            if reader.is_operator_advance(">") {
                break;
            } else if reader.is_operator_advance("/>") {
                // TODO check set closing
                // Early return if self closing
                return Ok(Element {
                    tag_name,
                    attributes,
                    children: ElementChildren::SelfClosing,
                });
            } else {
                // TODO extras here @ etc
                // let start = reader.get_start();
                let key = reader.parse_identifier("Element attribute")?.to_owned();
                let attribute = if reader.is_operator_advance("=") {
                    // let start = reader.get_start();
                    if reader.starts_with_string_delimeter() {
                        // TODO _quoted
                        let (content, _quoted) = reader.parse_string_literal()?;
                        // let position = start.with_length(content.len() + 2);
                        Attribute {
                            key,
                            value: content.to_owned(),
                        }
                    } else {
                        let content = reader.parse_identifier("Attribute value")?.to_owned();
                        Attribute {
                            key,
                            value: content.to_owned(),
                        }
                        // else {
                        //     dbg!(reader.current().get(..20));
                        //     return Err(());
                        //     // let error_position = start.with_length(
                        //     // 	crate::lexer::utilities::next_empty_occurance(reader.get_current()),
                        //     // );
                        //     // return Err(ParseError::new(
                        //     // 	ParseErrors::ExpectedAttribute,
                        //     // 	error_position,
                    }
                } else {
                    // Boolean attribute
                    Attribute {
                        key,
                        value: Default::default(),
                    }
                };
                attributes.push(attribute);
            }
        }

        if html_tag_is_self_closing(&tag_name) {
            return Ok(Element {
                tag_name,
                attributes,
                children: ElementChildren::SelfClosing,
            });
        } else if html_tag_contains_literal_content(&tag_name) {
            // TODO could embedded parser?
            // TODO I think this should take into account strings and more
            let (content, _) = reader
                .parse_until_postfix("</", &tag_name)
                .map_err(|()| {
                    dbg!("lit content");
                    // TODO might be a problem
                    // let position = reader.get_start().with_length(reader.get_current().len());
                    // ParseError::new(crate::ParseErrors::UnexpectedEnd, position)
                })?
                .to_owned();

            let content = content.to_owned();

            let closing_tag_name = reader.parse_identifier("Closing tag")?;
            if tag_name != closing_tag_name {
                dbg!(content, tag_name, closing_tag_name);
                return Err(());
                // return Err(ParseError::new(
                // 	crate::ParseErrors::ClosingTagDoesNotMatch {
                // 		tag_name: &tag_name,
                // 		closing_tag_name,
                // 	},
                // 	start.with_length(closing_tag_name.len() + 2),
                // ));
            }
            reader.expect('>')?;
            let children = ElementChildren::Literal(content);
            return Ok(Element {
                tag_name,
                attributes,
                children,
            });
        }

        let children = children_from_reader(reader, &tag_name)?;
        // if reader.is_operator_advance("</") {
        //     let _closing_tag_name = reader.parse_identifier("closing tag")?;
        //     reader.expect('>')?;

        Ok(Element {
            tag_name,
            attributes,
            children: ElementChildren::Children(children),
        })
        // } else {
        //     Err(())
        // }
    }

    // fn to_string_from_buffer<T: source_map::ToString>(
    // 	&self,
    // 	buf: &mut T,
    // 	options: &crate::ToStringOptions,
    // 	local: crate::LocalToStringInformation,
    // ) {
    // 	buf.push('<');
    // 	buf.push_str(&self.tag_name);
    // 	for attribute in &self.attributes {
    // 		buf.push(' ');
    // 		attribute.to_string_from_buffer(buf, options, local);
    // 	}
    // 	buf.push('>');

    // 	match self.children {
    // 		ElementChildren::Children(ref children) => {
    // 			_children_to_string(children, buf, options, local);
    // 			buf.push_str("</");
    // 			buf.push_str(&self.tag_name);
    // 			buf.push('>');
    // 		}
    // 		ElementChildren::SelfClosing => {}
    // 		ElementChildren::Literal(ref content) => {
    // 			buf.push_str(content);
    // 			buf.push_str("</");
    // 			buf.push_str(&self.tag_name);
    // 			buf.push('>');
    // 		}
    // 	}
    // }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

impl Attribute {
    // fn get_position(&self) -> Span {
    // 	match self {
    // 		Attribute::Static(_, _, pos)
    // 		| Attribute::Dynamic(_, _, pos)
    // 		| Attribute::Boolean(_, pos) => *pos,
    // 		Attribute::Spread(_, spread_pos) => *spread_pos,
    // 		Attribute::Shorthand(expr) => expr.get_position(),
    // 	}
    // }

    fn _from_reader(reader: &mut crate::Lexer) -> Result<Self, ()> {
        // let start = reader.get_start();
        let key = reader.parse_identifier("Element attribute")?.to_owned();
        if reader.is_operator_advance("=") {
            if reader.starts_with_string_delimeter() {
                let (content, _quoted) = reader.parse_string_literal()?;
                Ok(Attribute {
                    key,
                    value: content.to_owned(),
                })
            } else {
                // let error_position = start.with_length(
                // 	crate::lexer::utilities::next_empty_occurance(reader.get_current()),
                // );
                dbg!();
                Err(())
                // Err(ParseError::new(ParseErrors::ExpectedAttribute, error_position))
            }
        } else {
            Ok(Attribute {
                key,
                value: Default::default(),
            })
        }
    }

    // fn to_string_from_buffer<T: source_map::ToString>(
    // 	&self,
    // 	buf: &mut T,
    // 	options: &crate::ToStringOptions,
    // 	local: crate::LocalToStringInformation,
    // ) {
    // 	match self {
    // 		Attribute::Static(key, expression, _) => {
    // 			buf.push_str(key.as_str());
    // 			buf.push('=');
    // 			buf.push('"');
    // 			buf.push_str(expression.as_str());
    // 			buf.push('"');
    // 		}
    // 		Attribute::Dynamic(key, expression, _) => {
    // 			buf.push_str(key.as_str());
    // 			buf.push('=');
    // 			buf.push('{');
    // 			expression.to_string_from_buffer(buf, options, local);
    // 			buf.push('}');
    // 		}
    // 		Attribute::Boolean(key, _) => {
    // 			buf.push_str(key.as_str());
    // 		}
    // 		Attribute::Spread(expr, _) => {
    // 			buf.push_str("...");
    // 			expr.to_string_from_buffer(buf, options, local);
    // 		}
    // 		Attribute::Shorthand(expr) => {
    // 			expr.to_string_from_buffer(buf, options, local);
    // 		}
    // 	}
    // }
}

type ParseResult<T> = Result<T, ()>;

/// Also parsing end tag (to account for mismatched end tags)
fn children_from_reader(
    reader: &mut crate::Lexer,
    expected_closing_tag_name: &str,
) -> ParseResult<Vec<Node>> {
    let mut children = Vec::new();
    // TODO count new lines etc
    loop {
        reader.skip();
        // for _ in 0..reader.last_was_from_new_line() {
        // 	children.push(Node::LineBreak);
        // }
        if reader.is_operator_advance("</") {
            if !expected_closing_tag_name.is_empty() {
                let closing_tag_name = reader.parse_identifier("closing tag")?;
                reader.expect('>')?;
                if expected_closing_tag_name != closing_tag_name {
                    children.push(Node::MismatchClosingTag(closing_tag_name.to_owned()));
                    continue;
                }
            }
            return Ok(children);
        }
        children.push(Node::from_reader(reader)?);
    }
}

// fn children_to_string<T: source_map::ToString>(
// 	children: &[Node],
// 	buf: &mut T,
// 	options: &crate::ToStringOptions,
// 	local: crate::LocalToStringInformation,
// ) {
// 	let element_or_line_break_in_children =
// 		children.iter().any(|node| matches!(node, Node::Element(..) | Node::LineBreak));

// 	let mut previous_was_element_or_line_break = true;

// 	for node in children {
// 		if element_or_line_break_in_children
// 			&& !matches!(node, Node::LineBreak)
// 			&& previous_was_element_or_line_break
// 		{
// 			options.add_indent(local.depth + 1, buf);
// 		}
// 		node.to_string_from_buffer(buf, options, local);
// 		previous_was_element_or_line_break =
// 			matches!(node, Node::Element(..) | Node::LineBreak);
// 	}

// 	if options.pretty && local.depth > 0 && previous_was_element_or_line_break {
// 		options.add_indent(local.depth, buf);
// 	}
// }

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Element),
    TextNode(String),
    Comment(String),
    MismatchClosingTag(String),
}

impl Node {
    // fn get_position(&self) -> Span {
    // 	match self {
    // 		Node::TextNode(_, pos)
    // 		| Node::Comment(_, pos) => *pos,
    // 		Node::Element(element) => element.get_position(),
    // 		Node::LineBreak => source_map::Nullable::NULL,
    // 	}
    // }

    fn from_reader(reader: &mut crate::Lexer) -> Result<Self, ()> {
        reader.skip();
        // let start = reader.get_start();
        // if reader.is_operator_advance("{") {
        // 	let expression = FunctionArgument::from_reader(reader)?;
        // 	let end = reader.expect('}')?;
        // 	// let position = start.union(end);
        // 	let position = ();
        // 	Ok(Node::InterpolatedExpression(Box::new(expression), position))
        // } else
        if reader.starts_with_str("<!--") {
            reader.advance("<!--".len() as u32);
            // .map_err(|()| {
            // 	// TODO might be a problem
            // 	let position = reader.get_start().with_length(reader.get_current().len());
            // 	ParseError::new(crate::ParseErrors::UnexpectedEnd, position)
            // })?
            let (content, _) = reader.parse_until("-->", true)?.to_owned();
            Ok(Node::Comment(content.to_owned()))
        } else if reader.starts_with_str("<") {
            let element = Element::from_reader(reader)?;
            Ok(Node::Element(element))
        } else {
            let (content, _) = reader.parse_until("<", false)?;
            // .map_err(|()| {
            // 	// TODO might be a problem
            // 	let position = reader.get_start().with_length(reader.get_current().len());
            // 	ParseError::new(crate::ParseErrors::UnexpectedEnd, position)
            // })?;
            Ok(Node::TextNode(content.trim_start().into()))
        }
    }

    // fn to_string_from_buffer<T: source_map::ToString>(
    // 	&self,
    // 	buf: &mut T,
    // 	options: &crate::ToStringOptions,
    // 	local: crate::LocalToStringInformation,
    // ) {
    // 	match self {
    // 		Node::Element(element) => {
    // 			element.to_string_from_buffer(buf, options, local.next_level());
    // 		}
    // 		Node::TextNode(text, _) => buf.push_str(text),
    // 		Node::InterpolatedExpression(expression, _) => {
    // 			buf.push('{');
    // 			expression.to_string_from_buffer(buf, options, local.next_level());
    // 			buf.push('}');
    // 		}
    // 		Node::LineBreak => {
    // 			if options.pretty {
    // 				buf.push_new_line();
    // 			}
    // 		}
    // 		Node::Comment(comment, _) => {
    // 			if options.pretty {
    // 				buf.push_str("<!--");
    // 				buf.push_str(comment);
    // 				buf.push_str("-->");
    // 			}
    // 		}
    // 	}
    // }
}

/// Used for lexing
#[must_use]
pub fn html_tag_contains_literal_content(tag_name: &str) -> bool {
    matches!(tag_name, "script" | "style")
}

/// Used for lexing
#[must_use]
pub fn html_tag_is_self_closing(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn retrieve(content: String, query: String) -> String {
    use crate::{
        matching::{query_selector, query_selector_all, Selector},
        operations::inner_text,
    };
    let mut lexer = Lexer::new(&content);
    let result = Document::from_reader(&mut lexer);
    let document = result.expect("failed to parse document");
    let mut current: Vec<&Element> = vec![&document.html_element];
    for query in query.split('\0') {
        if let Some(selector) = query.strip_prefix("single ") {
            let selector = Selector::from_string(selector.trim());
            current = current
                .into_iter()
                .flat_map(|element| query_selector(element, &selector))
                .collect();
        } else if let Some(selector) = query.strip_prefix("all ") {
            let selector = Selector::from_string(selector.trim());
            current = current
                .into_iter()
                .flat_map(|element| query_selector_all(element, &selector))
                .collect();
        } else if let Some(expected_key) = query.strip_prefix("attribute ") {
            let mut buf = String::new();
            for element in current {
                let value = element
                    .attributes
                    .iter()
                    .find_map(|Attribute { key, value }| (key == expected_key).then_some(value));
                if let Some(value) = value {
                    if !buf.is_empty() {
                        buf.push('\0');
                    }
                    buf.push_str(value);
                }
            }
            return buf;
        } else if let "text" = query {
            let mut buf = String::new();
            for element in current {
                if !buf.is_empty() {
                    buf.push('\0');
                }
                buf.push_str(&inner_text(element));
            }
            return buf;
        }
    }

    panic!("no end query")
}
