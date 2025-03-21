use lightml::Element;

#[test]
fn parse_consumed() {
    let example = r#"<div class="something">
		<h3>Hello World</h3>
	</div>
	
	something"#;

    let (_item, consumed) = Element::from_string(example).unwrap();
    assert_eq!(example[consumed as usize..].trim(), "something");
}
