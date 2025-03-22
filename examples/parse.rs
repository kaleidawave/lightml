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

    const STACK_SIZE: usize = 8 * 1024 * 1024;

    std::thread::scope(|s| {
        let thread = std::thread::Builder::new()
            .name("Parsing thread".to_owned())
            .stack_size(STACK_SIZE)
            .spawn_scoped(s, || {
                let document = Document::from_reader(&mut Lexer::new(&content));
                document
            })
            .unwrap();

        let result = thread.join().unwrap();

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
            "check" => match result {
                Ok(_) => {
                    eprintln!("Parsed successfully");
                }
                Err(err) => {
                    let at = err.at as usize;
                    const SPACE: usize = 50;
                    let lhs = content.get(at.saturating_sub(SPACE)..at);
                    let rhs = content.get(at..(at + SPACE));
                    panic!("Could not parse {err:?} {:?}", (lhs, rhs));
                    // panic!("Could not parse {err:?}");
                }
            },
            _ => {
                eprintln!("{result:?}");
            }
        }
    });
}
