use lightml::{Document, Lexer};

fn as_lines(content: &str) -> String {
    let mut reader = Lexer::new(content);
    let result = Document::from_reader(&mut reader);
    // TODO does this throw information away?
    let result = result.map(|document| document.html_element);
    format!("{result:#?}")
}

fn main() -> std::process::ExitCode {
    let tests = get_tests();

    println!();
    println!("running {count} tests", count = tests.len());

    let mut failures: Vec<String> = Default::default();

    for test_case in tests {
        fn test<F>(_name: &str, cb: F) -> Result<(), ()>
        where
            F: FnOnce() + std::marker::Send + 'static,
        {
            let res = std::thread::spawn(cb);
            match res.join() {
                Ok(()) => Ok(()),
                Err(_) => Err(()),
            }
        }

        // let name = format!("{name}");
        let name = test_case.name;
        let result = test(&name, move || {
            let out = as_lines(&test_case.case).replace("\r\n", "\n");
            let expectation = test_case.output.trim_end();
            pretty_assertions::assert_eq!(out.trim_end(), expectation, "expected {expectation}",);
        });
        if let Ok(()) = result {
            println!("test {name} ... \u{001b}\u{005b}\u{0033}\u{0032}\u{006d}\u{006f}\u{006b}\u{001b}\u{005b}\u{0033}\u{0039}\u{006d}");
        } else {
            println!("test {name} ... \u{001b}\u{005b}\u{0033}\u{0031}\u{006d}\u{0066}\u{0061}\u{0069}\u{006c}\u{0065}\u{0064}\u{001b}\u{005b}\u{0033}\u{0039}\u{006d}");
            failures.push(name.to_string());
        }
    }

    if failures.is_empty() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

#[derive(Debug, Default)]
struct Test {
    section: String,
    name: String,
    options: (),
    case: String,
    output: String,
}

fn get_tests() -> Vec<Test> {
    use simple_markdown_parser::{parse, MarkdownElement};

    let mut tests: Vec<Test> = Vec::new();
    let mut current_test = Test::default();
    let mut section = String::new();

    let result = parse(include_str!("./specification.md"), |element| {
        if let MarkdownElement::Heading {
            level,
            text: content,
        } = element
        {
            if level >= 3 {
                if !current_test.case.is_empty() {
                    tests.push(std::mem::take(&mut current_test));
                }
                current_test.name = content.no_decoration();
                section.clone_into(&mut current_test.section);
            } else {
                section = content.no_decoration();
            }
        } else if let MarkdownElement::Paragraph(_content) = element {
            // if content.0.ends_with("`top_level_separator = Some(\"\\n\")`") {
            //     current_test.options.top_level_separator = Some("\n");
            // }
        } else if let MarkdownElement::CodeBlock { code, .. } = element {
            let code = code.replace("\r\n", "\n");
            if current_test.case.is_empty() {
                code.clone_into(&mut current_test.case);
            } else if current_test.output.is_empty() {
                code.clone_into(&mut current_test.output);
            } else {
                eprintln!("Another code block {code:?}");
            }
        }
    });

    assert!(result.is_ok(), "{result:?}");
    tests.push(current_test);
    tests
}
