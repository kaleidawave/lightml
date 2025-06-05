#![allow(clippy::result_unit_err)]
#![doc = include_str!("../README.md")]

pub use lexer::Lexer;
use std::borrow::Cow;

mod lexer;
pub mod matching;
pub mod operations;

type ParseResult<T> = Result<T, HTMLParseError>;

#[derive(Debug, Clone)]
pub struct ContextItem {
    pub tag_name: String,
    pub at: u32,
}

#[derive(Debug, Clone)]
pub struct HTMLParseError {
    pub reason: HTMLParseErrorReason,
    pub at: u32,
    pub context: Vec<ContextItem>,
}

#[derive(Debug, Clone, Copy)]
pub enum HTMLParseErrorReason {
    Expected { slice: &'static str },
    NoEndToStringDelimeter,
    InvalidIdentifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'a> {
    Element(Element<'a>),
    TextNode(&'a str),
    Comment(&'a str),
    MismatchClosingTag(&'a str),
}

impl<'a> Node<'a> {
    pub fn child_at(&self, matcher: impl matching::Matcher) -> Option<&Node> {
        if let Node::Element(element) = self {
            element.children.child_at(matcher)
        } else {
            None
        }
    }
}

pub type ContextChain = Vec<ContextItem>;

impl<'a> Node<'a> {
    // fn get_position(&self) -> Span {
    // 	match self {
    // 		Node::TextNode(_, pos)
    // 		| Node::Comment(_, pos) => *pos,
    // 		Node::Element(element) => element.get_position(),
    // 		Node::LineBreak => source_map::Nullable::NULL,
    // 	}
    // }

    fn from_reader(reader: &mut crate::Lexer<'a>, scope: &mut ContextChain) -> ParseResult<Self> {
        if reader.starts_with_no_advance("<!--") {
            reader.skip();
            reader.advance("<!--".len() as u32);
            // .map_err(|()| {
            // 	// TODO might be a problem
            // 	let position = reader.get_start().with_length(reader.get_current().len());
            // 	ParseError::new(crate::ParseErrors::UnexpectedEnd, position)
            // })?
            let content = reader
                .parse_until("-->", true)
                .map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::Expected { slice: "-->" },
                    at,
                    context: scope.clone(),
                })?;
            Ok(Node::Comment(content))
        } else if reader.starts_with_no_advance("<") {
            let element = Element::from_reader(reader, scope)?;
            Ok(Node::Element(element))
        } else {
            let content = reader
                .parse_until("<", false)
                .map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::Expected { slice: "<" },
                    at,
                    context: scope.clone(),
                })?;
            // dbg!(content.char_indices().filter(|(_, chr)| *chr == '`').collect::<Vec<_>>());
            // .map_err(|()| {
            // 	// TODO might be a problem
            // 	let position = reader.get_start().with_length(reader.get_current().len());
            // 	ParseError::new(crate::ParseErrors::UnexpectedEnd, position)
            // })?;
            Ok(Node::TextNode(content))
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

#[derive(Debug, Clone, PartialEq)]
pub struct Element<'a> {
    /// Name of the element
    pub tag_name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
    pub children: ElementChildren<'a>,
}

pub type Children<'a> = Vec<Node<'a>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ElementChildren<'a> {
    Children(Children<'a>),
    /// For script + style elements
    Literal(&'a str),
    /// For `img` elements etc
    SelfClosing,
}

impl<'a> ElementChildren<'a> {
    pub fn child_at(&self, matcher: impl matching::Matcher) -> Option<&Node> {
        match self {
            ElementChildren::Children(children) => matcher.extract(children),
            ElementChildren::Literal(_) | ElementChildren::SelfClosing => None,
        }
    }
}

impl<'a> From<Element<'a>> for Node<'a> {
    fn from(value: Element<'_>) -> Node<'_> {
        Node::Element(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document<'a> {
    pub html_element: Element<'a>,
}

/// [Specification](https://html.spec.whatwg.org/multipage/syntax.html#the-doctype)
fn parse_doctype(reader: &mut crate::Lexer<'_>) {
    reader.skip();
    let starts = reader.starts_with_ascii_case_ignore("<!DOCTYPE");
    if starts {
        let _ = reader.parse_one_or_more_whitespace();
        let _ = reader.starts_with_ascii_case_ignore("html");
        // TODO legacy string
        let _ = reader.is_operator_advance(">");
    }
}

impl<'a> Document<'a> {
    pub fn from_reader(reader: &mut crate::Lexer<'a>) -> ParseResult<Self> {
        parse_doctype(reader);
        Element::from_reader(reader, &mut Vec::new()).map(|html_element| Document { html_element })
    }
}

impl<'a> Element<'a> {
    pub fn from_reader(
        reader: &mut crate::Lexer<'a>,
        scope: &mut ContextChain,
    ) -> ParseResult<Self> {
        reader.skip();
        let start = reader.consumed();
        reader.expect('<').map_err(|at| HTMLParseError {
            reason: HTMLParseErrorReason::Expected { slice: "<" },
            at,
            context: scope.clone(),
        })?;
        let tag_name = reader
            .parse_identifier("Element name")
            .map_err(|at| HTMLParseError {
                reason: HTMLParseErrorReason::InvalidIdentifier,
                at,
                context: Vec::new(),
            })?;
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
                let key =
                    reader
                        .parse_identifier("Attribute key")
                        .map_err(|at| HTMLParseError {
                            reason: HTMLParseErrorReason::InvalidIdentifier,
                            at,
                            context: vec![ContextItem {
                                tag_name: tag_name.to_owned(),
                                at: start,
                            }],
                        })?;
                let attribute = if reader.is_operator_advance("=") {
                    // let start = reader.get_start();
                    if reader.starts_with_string_delimeter() {
                        let content =
                            reader.parse_string_literal().map_err(|at| HTMLParseError {
                                reason: HTMLParseErrorReason::NoEndToStringDelimeter,
                                at,
                                context: scope.clone(),
                            })?;
                        Attribute {
                            key,
                            value: content,
                        }
                    } else {
                        let content = reader.parse_identifier("Attribute value").map_err(|at| {
                            HTMLParseError {
                                reason: HTMLParseErrorReason::InvalidIdentifier,
                                at,
                                context: vec![ContextItem {
                                    tag_name: tag_name.to_owned(),
                                    at: start,
                                }],
                            }
                        })?;
                        Attribute {
                            key,
                            value: content,
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

        if html_tag_is_self_closing(tag_name) {
            return Ok(Element {
                tag_name,
                attributes,
                children: ElementChildren::SelfClosing,
            });
        } else if html_tag_contains_literal_content(tag_name) {
            // TODO could embedded parser?
            // TODO I think this should take into account strings and more
            let content = reader.parse_until_postfix("</", tag_name).map_err(|at| {
                HTMLParseError {
                    reason: HTMLParseErrorReason::Expected { slice: "</" },
                    at,
                    context: scope.clone(),
                }
                // TODO might be a problem
                // let position = reader.get_start().with_length(reader.get_current().len());
                // ParseError::new(crate::ParseErrors::UnexpectedEnd, position)
            })?;

            // TODO is this the best way?
            while !reader.is_finished() {
                let closing_tag_name =
                    reader
                        .parse_identifier("Closing tag")
                        .map_err(|at| HTMLParseError {
                            reason: HTMLParseErrorReason::InvalidIdentifier,
                            at,
                            context: scope.clone(),
                        })?;
                if tag_name == closing_tag_name {
                    break;
                }
            }

            reader.expect('>').map_err(|at| HTMLParseError {
                reason: HTMLParseErrorReason::Expected { slice: "</" },
                at,
                context: scope.clone(),
            })?;

            let children = ElementChildren::Literal(content);
            return Ok(Element {
                tag_name,
                attributes,
                children,
            });
        }

        scope.push(ContextItem {
            tag_name: tag_name.to_owned(),
            at: start,
        });
        let children = children_from_reader(reader, scope);
        let _popped = scope.pop();
        // TODO pointer equality on tagname
        debug_assert!(_popped.is_some_and(|t| tag_name == t.tag_name));

        match children {
            Ok(children) => {
                if let Some(closing_tag_name) = reader.parse_closing_tag_no_advance() {
                    if tag_name == closing_tag_name {
                        reader.advance(2 + closing_tag_name.len() as u32);
                        reader.expect('>').map_err(|at| HTMLParseError {
                            reason: HTMLParseErrorReason::Expected { slice: ">" },
                            at,
                            context: Vec::new(),
                        })?;
                    } else {
                        // TODO function should check. This is a valid path for mismatched tags
                        // dbg!("should not be here", reader.consumed());
                    }
                }
                Ok(Element {
                    tag_name,
                    attributes,
                    children: ElementChildren::Children(children),
                })
            }
            Err(err) => {
                // err.context.push(ContextItem {
                //     tag_name: tag_name.to_owned(),
                //     at: start,
                // });
                Err(err)
            }
        }
    }

    /// Also returns how many bytes parsed
    pub fn from_string(content: &'a str) -> ParseResult<(Self, u32)> {
        let mut lexer = Lexer::new(content);
        let element = Self::from_reader(&mut lexer, &mut Vec::new())?;
        Ok((element, lexer.consumed()))
    }

    // fn to_string_from_buffer<T: source_map::ToString>(
    // 	&self,
    // 	buf: &mut T,
    // 	options: &crate::ToStringOptions,
    // 	local: crate::LocalToStringInformation,
    // ) {
    //
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

impl<'a> Attribute<'a> {
    // fn get_position(&self) -> Span {
    // 	match self {
    // 		Attribute::Static(_, _, pos)
    // 		| Attribute::Dynamic(_, _, pos)
    // 		| Attribute::Boolean(_, pos) => *pos,
    // 		Attribute::Spread(_, spread_pos) => *spread_pos,
    // 		Attribute::Shorthand(expr) => expr.get_position(),
    // 	}
    // }

    fn _from_reader(reader: &mut crate::Lexer<'a>) -> ParseResult<Self> {
        // let start = reader.get_start();
        let key = reader
            .parse_identifier("Attribute key")
            .map_err(|at| HTMLParseError {
                reason: HTMLParseErrorReason::InvalidIdentifier,
                at,
                context: Vec::new(),
            })?;

        if reader.is_operator_advance("=") {
            if reader.starts_with_string_delimeter() {
                let content = reader.parse_string_literal().map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::NoEndToStringDelimeter,
                    context: Vec::new(),
                    at,
                })?;
                Ok(Attribute {
                    key,
                    value: content,
                })
            } else {
                // let error_position = start.with_length(
                // 	crate::lexer::utilities::next_empty_occurance(reader.get_current()),
                // );
                let at = reader.consumed();
                Err(HTMLParseError {
                    reason: HTMLParseErrorReason::Expected {
                        slice: "string delimeter",
                    },
                    at,
                    context: Vec::new(),
                })
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

/// Also parsing end tag (to account for mismatched end tags)
fn children_from_reader<'a>(
    reader: &mut crate::Lexer<'a>,
    scope: &mut ContextChain,
) -> ParseResult<Vec<Node<'a>>> {
    let mut children = Vec::new();
    // TODO count new lines etc
    loop {
        // for _ in 0..reader.last_was_from_new_line() {
        // 	children.push(Node::LineBreak);
        // }
        if let Some(closing_tag_name) = reader.parse_closing_tag_no_advance() {
            let _whitespace = reader.parse_whitespace();
            // if let Some(whitespace) = whitespace {
            //     children.push(Node::TextNode(whitespace));
            // }

            // TODO wip
            if scope.iter().rev().any(|c| c.tag_name == closing_tag_name) {
                return Ok(children);
            }

            // TODO explain
            children.push(Node::MismatchClosingTag(closing_tag_name));
            reader.advance(2 + closing_tag_name.len() as u32);
            reader.expect('>').map_err(|at| HTMLParseError {
                reason: HTMLParseErrorReason::Expected { slice: ">" },
                at,
                context: Vec::new(),
            })?;
            continue;
        }

        let next = reader.parse_opening_tag_no_advance();
        if let Some(next) = next {
            #[rustfmt::skip]
            fn not_allowed_in_p(on: &str) -> bool {
                matches!(on,
                    "address" | "article" | "aside" | "blockquote" | "details" | "dialog" | "div" | "dl" | "fieldset" 
                    | "figcaption" | "figure" | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" 
                    | "header" | "hgroup" | "hr" | "main" | "menu" | "nav" | "ol" | "p" | "pre" | "search" 
                    | "section" | "table" | "ul"
                )
            }

            // TODO unwrap
            let expected_closing_tag_name = scope.last().unwrap().tag_name.as_str();
            let should_return = match expected_closing_tag_name {
                // TODO more branches
                "p" => not_allowed_in_p(next),
                "li" => next == "li",
                _ => false,
            };
            if should_return {
                let whitespace = reader.parse_whitespace();
                if let Some(whitespace) = whitespace {
                    children.push(Node::TextNode(whitespace));
                }
                return Ok(children);
            }
        }

        let node = Node::from_reader(reader, scope)?;
        if let Node::TextNode(content) = node {
            if content.trim().is_empty() {
                continue;
            }
        }
        children.push(node);
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

// pub fn parse_head(content: &str) -> ParseResult<Element> {
//     let mut reader = Lexer::new(content);
//     let lowercase = reader.is_operator_advance("<!DOCTYPE html>");
//     if !lowercase {
//         let _ = reader.is_operator_advance("<!doctype html>");
//     }
//     let _ = reader.is_operator_advance("<html>");
//     Element::from_reader(&mut reader, &mut )
// }

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn retrieve(content: String, query: String) -> String {
    use crate::{
        matching::{query_selector, query_selector_all, Selector},
        operations::{inner_text, inner_text_element},
    };

    let mut reader = Lexer::new(&content);
    let result = Document::from_reader(&mut reader);
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
                    .find_map(|Attribute { key, value }| (key == &expected_key).then_some(value));
                if let Some(value) = value {
                    if !buf.is_empty() {
                        buf.push_str("\0a");
                    }
                    buf.push_str(value);
                }
            }
            return buf;
        } else if let "text" = query {
            let mut buf = String::new();
            for element in current {
                if !buf.is_empty() {
                    buf.push_str("\0t");
                }
                buf.push_str(&inner_text_element(element));
            }
            return buf;
        } else if let "table" = query {
            let mut buf = String::new();
            for element in current {
                if !buf.is_empty() {
                    buf.push('\0');
                }
                let mut rows: &[_] =
                    if let ElementChildren::Children(ref children) = element.children {
                        children
                    } else {
                        &[]
                    };
                if let Some(children) = rows.iter().find_map(|child| {
                    if let Node::Element(Element {
                        tag_name, children, ..
                    }) = child
                    {
                        (*tag_name == "tbody").then_some(children)
                    } else {
                        None
                    }
                }) {
                    if let ElementChildren::Children(ref children) = children {
                        rows = children;
                    }
                }
                for child in rows {
                    if let Node::Element(Element { children, .. }) = child {
                        buf.push_str("\0r");
                        if let ElementChildren::Children(ref children) = children {
                            for element in children {
                                buf.push_str("\0d");
                                buf.push_str(&inner_text(element));
                            }
                        }
                    }
                }
            }
            return buf;
        }
    }

    panic!("no end query")
}
