use super::{Attribute, Element, ElementChildren, Node};

pub fn query_selector_all<'a>(element: &'a Element, matching: &Selector) -> Vec<&'a Element> {
    fn query_selector_all_<'a>(
        element: &'a Element,
        matching: &Selector,
        found: &mut Vec<&'a Element>,
    ) {
        if matches(element, matching) {
            found.push(element);
        }

        if let ElementChildren::Children(ref children) = element.children {
            for child in children {
                match child {
                    Node::Element(element) => {
                        query_selector_all_(element, matching, found);
                    }
                    Node::TextNode(..) | Node::Comment(..) => {}
                }
            }
        }
    }

    let mut found = Vec::new();
    query_selector_all_(element, matching, &mut found);
    found
}

pub fn query_selector<'a>(element: &'a Element, matching: &Selector) -> Option<&'a Element> {
    if matches(element, matching) {
        return Some(element);
    }

    if let ElementChildren::Children(ref children) = element.children {
        for child in children {
            match child {
                Node::Element(element) => {
                    if let v @ Some(_) = query_selector(element, matching) {
                        return v;
                    }
                }
                Node::TextNode(..) | Node::Comment(..) => {}
            }
        }
    }

    None
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeQuery {
    Exactly,
    ExactlyBeforeHyphen,
    Contains,
    ContainsWhitespaceSplit,
    Prefixed,
    Suffixed,
}

#[derive(Debug)]
pub struct Selector<'a> {
    pub tag: Option<&'a str>,
    pub attributes: Vec<(&'a str, AttributeQuery, &'a str)>,
}

impl<'a> Selector<'a> {
    pub fn from_string(from: &'a str) -> Self {
        let mut idx = 0;
        let tag = if from.starts_with(['*', '[', '.', '#']) {
            None
        } else if from.starts_with(char::is_alphabetic) {
            let mut broke = false;
            for (i, chr) in from.char_indices() {
                if !chr.is_alphanumeric() {
                    idx = i;
                    broke = true;
                    break;
                }
            }
            if !broke {
                idx = from.len();
            }
            Some(&from[..idx])
        } else {
            todo!("error")
        };
        let mut attributes: Vec<(&str, AttributeQuery, &str)> = Vec::new();
        while idx < from.len() {
            // eprintln!("Incoming {:?}", &from[idx..]);
            let next = from[idx..].chars().next().unwrap();
            let rest = &from[idx + next.len_utf8()..];
            idx += next.len_utf8();
            match next {
                '.' => {
                    let mut len = rest.len();
                    for (i, chr) in rest.char_indices() {
                        let valid_class_name = chr.is_alphanumeric() || matches!(chr, '-' | '_');
                        if !valid_class_name {
                            len = i;
                            break;
                        }
                    }
                    idx += len;
                    attributes.push(("class", AttributeQuery::Exactly, &rest[..len]));
                }
                '#' => {
                    let mut len = rest.len();
                    for (i, chr) in rest.char_indices() {
                        let valid_identifier_name =
                            chr.is_alphanumeric() || matches!(chr, '-' | '_');
                        if !valid_identifier_name {
                            len = i;
                            break;
                        }
                    }
                    idx += len;
                    attributes.push(("id", AttributeQuery::Exactly, &rest[..len]));
                }
                '[' => {
                    for (i, chr) in rest.char_indices() {
                        if !(chr.is_alphanumeric() || matches!(chr, '-' | '_')) {
                            idx += i;
                            let attribute = &rest[..i];
                            let rest = &rest[i..];
                            let (query, offset) = match rest.as_bytes() {
                                [b']', ..] => {
                                    // Think this is okay
                                    attributes.push((attribute, AttributeQuery::Contains, ""));
                                    break;
                                }
                                [b'~', b'=', ..] => (AttributeQuery::ContainsWhitespaceSplit, 2),
                                [b'^', b'=', ..] => (AttributeQuery::Prefixed, 2),
                                [b'$', b'=', ..] => (AttributeQuery::Suffixed, 2),
                                [b'|', b'=', ..] => (AttributeQuery::ExactlyBeforeHyphen, 2),
                                [b'=', ..] => (AttributeQuery::Exactly, 1),
                                _ => {
                                    todo!("{rest}");
                                }
                            };
                            idx += offset;
                            // dbg!(&attribute, rest, offset, &rest[offset..], query);
                            let mut found = false;
                            let rest = &rest[offset..];
                            for (j, chr) in rest.char_indices() {
                                if let ']' = chr {
                                    found = true;
                                    let value = &rest[..j];
                                    // TODO temp
                                    let value = value.trim_matches('"').trim_matches('\'');
                                    attributes.push((attribute, query, value));
                                    idx += j + ']'.len_utf8();
                                    break;
                                }
                            }
                            if !found {
                                panic!();
                            }
                            break;
                        }
                    }
                }
                chr => todo!("{chr:?}"),
            }
        }
        Selector { tag, attributes }
    }
}

pub fn matches(element: &Element, selector: &Selector) -> bool {
    let tag = if let Some(ref tag) = selector.tag {
        tag == &element.tag_name
    } else {
        true
    };
    let selector = tag
        && selector.attributes.iter().all(|query| {
            let (expected_key, kind, expected_value) = query;
            let value = element
                .attributes
                .iter()
                .find_map(|Attribute { key, value }| (key == expected_key).then_some(value));

            if let Some(value) = value {
                match kind {
                    AttributeQuery::Exactly => value == expected_value,
                    AttributeQuery::ExactlyBeforeHyphen => {
                        let value: &str = value
                            .split_once("-")
                            .map_or(value.as_str(), |(left, _)| left);
                        value == *expected_value
                    }
                    AttributeQuery::Contains => value.contains(expected_value),
                    AttributeQuery::ContainsWhitespaceSplit => value
                        .split(char::is_whitespace)
                        .any(|part| part == *expected_value),
                    AttributeQuery::Prefixed => value.starts_with(expected_value),
                    AttributeQuery::Suffixed => value.ends_with(expected_value),
                }
            } else {
                false
            }
        });

    selector
}
