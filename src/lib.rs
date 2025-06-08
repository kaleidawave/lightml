#![allow(clippy::result_unit_err, clippy::cast_possible_truncation)]
#![doc = include_str!("../README.md")]

mod lexer;
pub mod matching;
pub mod operations;
pub mod retrieval;

pub use lexer::Lexer;
use std::borrow::Cow;

#[cfg(not(feature = "nightly"))]
use allocator_api2::{boxed::Box, vec::Vec};

#[cfg(feature = "nightly")]
use std::alloc::Allocator as AllocatorTrait;

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
    pub context: std::vec::Vec<ContextItem>,
}

#[derive(Debug, Clone, Copy)]
pub enum HTMLParseErrorReason {
    Expected { slice: &'static str },
    NoEndToStringDelimeter,
    InvalidIdentifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'a> {
    Element(Box<Element<'a>, &'a Allocator>),
    TextNode(&'a str),
    Comment(&'a str),
    MismatchClosingTag(&'a str),
}

impl Node<'_> {
    pub fn child_at(&self, matcher: impl matching::Matcher) -> Option<&Node> {
        if let Node::Element(element) = self {
            element.children.child_at(matcher)
        } else {
            None
        }
    }
}

pub type ContextChain = std::vec::Vec<ContextItem>;
pub type Allocator = bumpalo::Bump;
// pub type Allocator = allocator_api2::alloc::Global;

impl<'a> Node<'a> {
    fn from_reader(
        reader: &mut crate::Lexer<'a>,
        scope: &mut ContextChain,
        allocator: &'a Allocator,
    ) -> ParseResult<Self> {
        // Comments
        if reader.starts_with_no_advance("<!--") {
            reader.skip();
            reader.advance("<!--".len() as u32);
            let content = reader
                .parse_until("-->", true)
                .map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::Expected { slice: "-->" },
                    at,
                    context: scope.clone(),
                })?;
            Ok(Node::Comment(content))
        } else if reader.starts_with_no_advance("<") {
            let element = Element::from_reader(reader, scope, allocator)?;
            Ok(Node::Element(Box::new_in(element, allocator)))
        } else {
            let content = reader
                .parse_until("<", false)
                .map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::Expected { slice: "<" },
                    at,
                    context: scope.clone(),
                })?;
            Ok(Node::TextNode(resolve_whitespace(content)))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element<'a> {
    /// Name of the element
    pub tag_name: &'a str,
    pub attributes: Vec<Attribute<'a>, &'a Allocator>,
    pub children: ElementChildren<'a>,
}

pub type Children<'a> = Vec<Node<'a>, &'a Allocator>;

#[derive(Debug, Clone, PartialEq)]
pub enum ElementChildren<'a> {
    Children(Children<'a>),
    /// For script + style elements
    Literal(&'a str),
    /// For `img` elements etc
    SelfClosing,
}

impl ElementChildren<'_> {
    pub fn child_at(&self, matcher: impl matching::Matcher) -> Option<&Node> {
        match self {
            ElementChildren::Children(children) => matcher.extract(children),
            ElementChildren::Literal(_) | ElementChildren::SelfClosing => None,
        }
    }
}

// impl<'a> From<Element<'a>> for Node<'a> {
//     fn from(value: Element<'_>) -> Node<'_> {
//         Node::Element(value)
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Document<'a> {
    pub html_element: Element<'a>,
}

/// [see specification](https://html.spec.whatwg.org/multipage/syntax.html#the-doctype)
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
    /// # Errors
    ///
    /// Will return `Err` for invalid HTML documents
    pub fn from_reader(
        reader: &mut crate::Lexer<'a>,
        allocator: &'a Allocator,
    ) -> ParseResult<Self> {
        parse_doctype(reader);
        Element::from_reader(reader, &mut std::vec::Vec::new(), allocator)
            .map(|html_element| Document { html_element })
    }
}

impl<'a> Element<'a> {
    /// # Errors
    ///
    /// Will return `Err` for invalid HTML elements
    #[allow(clippy::too_many_lines)]
    pub fn from_reader(
        reader: &mut crate::Lexer<'a>,
        scope: &mut ContextChain,
        allocator: &'a Allocator,
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
                context: std::vec::Vec::new(),
            })?;

        let mut attributes = Vec::new_in(allocator);
        // Kind of weird / not clear conditions for breaking out of while loop
        loop {
            reader.skip();
            if reader.is_operator_advance(">") {
                break;
            } else if reader.is_operator_advance("/>") {
                // TODO check element is self closing
                // Early return if self closing
                return Ok(Element {
                    tag_name,
                    attributes,
                    children: ElementChildren::SelfClosing,
                });
            }
            // TODO extras here
            let key = reader
                .parse_identifier("Attribute key")
                .map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::InvalidIdentifier,
                    at,
                    context: std::vec![ContextItem {
                        tag_name: tag_name.to_owned(),
                        at: start,
                    }],
                })?;

            let attribute = if reader.is_operator_advance("=") {
                if reader.starts_with_string_delimeter() {
                    let content = reader.parse_string_literal().map_err(|at| HTMLParseError {
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
                            context: std::vec![ContextItem {
                                tag_name: tag_name.to_owned(),
                                at: start,
                            }],
                        }
                    })?;
                    Attribute {
                        key,
                        value: content,
                    }
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

        if html_tag_is_self_closing(tag_name) {
            Ok(Element {
                tag_name,
                attributes,
                children: ElementChildren::SelfClosing,
            })
        } else if html_tag_contains_literal_content(tag_name) {
            // TODO could embed parser here?
            // TODO I think this should take into account when it is not at the end
            // of an element, such as being in a string and more
            let content =
                reader
                    .parse_until_postfix("</", tag_name)
                    .map_err(|at| HTMLParseError {
                        reason: HTMLParseErrorReason::Expected { slice: "</" },
                        at,
                        context: scope.clone(),
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
            Ok(Element {
                tag_name,
                attributes,
                children,
            })
        } else {
            scope.push(ContextItem {
                tag_name: tag_name.to_owned(),
                at: start,
            });

            let children = children_from_reader(reader, scope, allocator)?;
            #[cfg(debug_assertions)]
            {
                let popped = scope.pop();
                debug_assert!(popped.is_some_and(|t| tag_name == t.tag_name));
            }
            #[cfg(not(debug_assertions))]
            let _popped = scope.pop();

            if let Some(closing_tag_name) = reader.parse_closing_tag_no_advance() {
                if tag_name == closing_tag_name {
                    reader.advance(2 + closing_tag_name.len() as u32);
                    reader.expect('>').map_err(|at| HTMLParseError {
                        reason: HTMLParseErrorReason::Expected { slice: ">" },
                        at,
                        context: std::vec::Vec::new(),
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
    }

    /// Also returns how many bytes parsed
    /// # Errors
    ///
    /// Will return `Err` for invalid HTML elements
    pub fn from_string(content: &'a str, allocator: &'a Allocator) -> ParseResult<(Self, u32)> {
        let mut lexer = Lexer::new(content);
        let element = Self::from_reader(&mut lexer, &mut std::vec::Vec::new(), allocator)?;
        Ok((element, lexer.consumed()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

impl<'a> Attribute<'a> {
    /// Not used in main loop for now
    /// # Errors
    ///
    /// Will return `Err` for invalid attributes
    pub fn from_reader(reader: &mut crate::Lexer<'a>) -> ParseResult<Self> {
        let key = reader
            .parse_identifier("Attribute key")
            .map_err(|at| HTMLParseError {
                reason: HTMLParseErrorReason::InvalidIdentifier,
                at,
                context: std::vec::Vec::new(),
            })?;

        if reader.is_operator_advance("=") {
            if reader.starts_with_string_delimeter() {
                let content = reader.parse_string_literal().map_err(|at| HTMLParseError {
                    reason: HTMLParseErrorReason::NoEndToStringDelimeter,
                    context: std::vec::Vec::new(),
                    at,
                })?;
                Ok(Attribute {
                    key,
                    value: content,
                })
            } else {
                let at = reader.consumed();
                Err(HTMLParseError {
                    reason: HTMLParseErrorReason::Expected {
                        slice: "string delimeter",
                    },
                    at,
                    context: std::vec::Vec::new(),
                })
            }
        } else {
            Ok(Attribute {
                key,
                value: Default::default(),
            })
        }
    }
}

/// Also parsing end tag (to account for mismatched end tags)
fn children_from_reader<'a>(
    reader: &mut crate::Lexer<'a>,
    scope: &mut ContextChain,
    allocator: &'a Allocator,
) -> ParseResult<Vec<Node<'a>, &'a Allocator>> {
    let mut children = Vec::new_in(allocator);
    loop {
        if let Some(closing_tag_name) = reader.parse_closing_tag_no_advance() {
            let _whitespace = reader.parse_whitespace();

            if scope.iter().rev().any(|c| c.tag_name == closing_tag_name) {
                return Ok(children);
            }

            // TODO explain
            children.push(Node::MismatchClosingTag(closing_tag_name));
            reader.advance(2 + closing_tag_name.len() as u32);
            reader.expect('>').map_err(|at| HTMLParseError {
                reason: HTMLParseErrorReason::Expected { slice: ">" },
                at,
                context: std::vec::Vec::new(),
            })?;
            continue;
        }

        let next_element_tag = reader.parse_opening_tag_no_advance();
        if let Some(next_element_tag) = next_element_tag {
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
                "p" => not_allowed_in_p(next_element_tag),
                "li" => next_element_tag == "li",
                _ => false,
            };
            if !children.is_empty() {
                // WIP
                let should_add_whitespace = match expected_closing_tag_name {
                    // TODO more branches
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "li" | "span" | "em"
                    | "strong" => true,
                    _ => false,
                };
                if should_add_whitespace {
                    let whitespace = reader.parse_whitespace();
                    if let Some(whitespace) = whitespace {
                        children.push(Node::TextNode(whitespace));
                    }
                }
            }
            if should_return {
                return Ok(children);
            }
        }

        let node = Node::from_reader(reader, scope, allocator)?;
        if let Node::TextNode(content) = node {
            if content.trim().is_empty() {
                continue;
            }
        }
        children.push(node);
    }
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

/// HTML text nodes allow at most one space
/// WIP needs new line stuff
fn resolve_whitespace(on: &str) -> &str {
    if on == " " {
        on
    } else {
        let mut chars = on.chars();
        let leading = chars.by_ref().take_while(|c| *c == ' ').count();
        let trailing = chars.rev().take_while(|c| *c == ' ').count();
        &on[leading.saturating_sub(1)..(on.len() - trailing.saturating_sub(1))]
    }
}
