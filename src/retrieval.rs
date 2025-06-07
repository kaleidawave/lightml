use crate::{
    matching::{query_selector, query_selector_all, Selector},
    operations::{inner_text, inner_text_element},
    Attribute, Document, Element, ElementChildren, Lexer, Node,
};

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[must_use]
/// # Panics
///
/// Will panic if failed for parse document
pub fn retrieve(content: &str, query: &str) -> String {
    let mut reader = Lexer::new(content);
    let allocator = bumpalo::Bump::new();
    let result = Document::from_reader(&mut reader, &allocator);
    let document = result.expect("failed to parse document");

    let mut current: Vec<&Element> = vec![&document.html_element];

    for query in query.split('\0') {
        if let Some(selector) = query.strip_prefix("single ") {
            let selector = Selector::from_string(selector.trim());
            current = current
                .into_iter()
                .filter_map(|element| query_selector(element, &selector))
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

                let table_body = rows.iter().find_map(|child| {
                    if let Node::Element(Element {
                        tag_name, children, ..
                    }) = child
                    {
                        (*tag_name == "tbody").then_some(children)
                    } else {
                        None
                    }
                });

                if let Some(ElementChildren::Children(ref children)) = table_body {
                    rows = children;
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
