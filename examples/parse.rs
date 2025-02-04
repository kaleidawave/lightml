fn main() {
    let example = r#"<div class="something">
        <h3>Hello World</h3>
    </div>"#;

    // If arg use that file, else use example above
    let content = if let Some(path) = std::env::args().nth(1) {
        std::fs::read_to_string(path).unwrap()
    } else {
        example.to_owned()
    };

    let second_arg = std::env::args().nth(2);
    let mode = second_arg
        .and_then(|arg| {
            matches!(arg.as_str(), "--verbose" | "--text" | "--check")
                .then_some(&*arg[2..].to_owned().leak())
        })
        .unwrap_or_default();

    use lightml::{operations, Document, Lexer};

    let result = Document::from_reader(&mut Lexer::new(&content));

    match mode {
        "text" => {
            eprintln!(
                "Text: {text}",
                text = operations::inner_text(&result.unwrap().html_element)
            );
        }
        "verbose" => {
            eprintln!("{result:#?}");
        }
        "check" => {
            if result.is_ok() {
                eprintln!("Parsed successfully");
            } else {
                panic!("Could not parse");
            }
        }
        _ => {
            eprintln!("{result:?}");
        }
    }
}
