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

#[test]
fn literal_tags() {
    let example = r#"<head>
		<style>
			/** <hiya> */
		</style>
		<script>
			const x = 5;
		</script>
	</head>"#;

    let _item = Element::from_string(example).unwrap();
}

#[test]
fn comments() {
    let example = r#"<div class="hello world">
		<!-- Something -->
	</div>"#;

    let _item = Element::from_string(example).unwrap();
}
