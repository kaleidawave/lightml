use super::{Attribute, Cow, Element, ElementChildren, Node};

fn inner_text_element_(element: &Element, buf: &mut String) {
    if let "math" | "svg" | "title" = element.tag_name {
        return;
    }
    let skip_citations = true;
    let skip = skip_citations && element.tag_name == "sup";
    // TODO check inner is a with [] etc
    if skip {
        return;
    }
    if let ElementChildren::Children(ref children) = element.children {
        for child in children {
            inner_text_(child, buf);
        }
    }
}

fn inner_text_(node: &Node, buf: &mut String) {
    match node {
        Node::Element(element) => {
            inner_text_element_(element, buf);
        }
        Node::TextNode(content) => {
            buf.push_str(&unescape_string_content(content));
        }
        Node::Comment(..) | Node::MismatchClosingTag(..) => {}
    }
}

#[must_use]
pub fn inner_text(node: &Node) -> String {
    let mut s = String::new();
    inner_text_(node, &mut s);
    s
}

#[must_use]
pub fn inner_text_element(element: &Element) -> String {
    let mut s = String::new();
    inner_text_element_(element, &mut s);
    s
}

/// Modified version of <https://github.com/parcel-bundler/parcel/blob/f86f5f27c3a6553e70bd35652f19e6ab8d8e4e4a/crates/dev-dep-resolver/src/lib.rs#L368-L380>
/// # Panics
///
/// Will panic if trying to decode character that is invalid code
#[must_use]
pub fn unescape_string_content(on: &str) -> Cow<'_, str> {
    let mut result = Cow::Borrowed("");
    let mut start = 0;
    for (index, _matched) in on.match_indices('&') {
        result += &on[start..index];
        let after = &on[(index + '&'.len_utf8())..];
        if after.starts_with("quot;") {
            start = index + "quot;".len();
            result += "\"";
        } else if after.starts_with("amp;") {
            start = index + "amp;".len();
            result += "&";
        } else if after.starts_with("lt;") {
            start = index + "lt;".len();
            result += "<";
        } else if after.starts_with("gt;") {
            start = index + "gt;".len();
            result += ">";
        } else if let Some(after) = after.strip_prefix('#') {
            let mut found = false;
            for (idx, chr) in after.char_indices() {
                if let ';' = chr {
                    // TODO unwraps
                    let item = &after[..idx];
                    let code: u32 = if let Some(rest) = item.strip_prefix(['x', 'X']) {
                        let mut value = 0u32;
                        for c in rest.as_bytes() {
                            value <<= 4; // 16=2^4
                            match c {
                                b'0'..=b'9' => {
                                    value += u32::from(c - b'0');
                                }
                                b'a'..=b'f' => {
                                    value += u32::from(c - b'a') + 10;
                                }
                                b'A'..=b'F' => {
                                    value += u32::from(c - b'A') + 10;
                                }
                                chr => panic!("{chr:?}"),
                            }
                        }
                        value
                    } else {
                        item.parse().unwrap()
                    };
                    let entity = char::from_u32(code).unwrap().to_string();
                    // Cannot += to result because of lifetimes
                    let mut out = result.into_owned();
                    out.push_str(&entity);
                    result = Cow::Owned(out);
                    start = index + "&#".len() + idx + ";".len();
                    found = true;
                    break;
                }
            }
            assert!(found,);
        }
    }
    result += &on[start..];
    result
}

pub trait Walker {
    fn text_node(&mut self, _content: &str) {}

    fn attribute(&mut self, _key: &str, _value: &str) {}
}

pub fn walk_nodes_on_element(element: &Element, walker: &mut impl Walker) {
    for Attribute { key, value } in &element.attributes {
        walker.attribute(key, value);
    }
    if let ElementChildren::Children(ref children) = element.children {
        for child in children {
            walk_nodes_on_node(child, walker);
        }
    }
}

pub fn walk_nodes_on_node(node: &Node, walker: &mut impl Walker) {
    match node {
        Node::Element(element) => {
            walk_nodes_on_element(element, walker);
        }
        Node::TextNode(content) => {
            // cb(&unescape_string_content(content));
            walker.text_node(content);
        }
        Node::Comment(..) | Node::MismatchClosingTag(..) => {}
    }
}
