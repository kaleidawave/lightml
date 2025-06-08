use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::{
    self,
    termcolor::{ColorChoice, StandardStream},
    Config,
};

use lightml::{operations, Allocator, Document, Lexer};

fn main() -> std::process::ExitCode {
    let example = r"<div>
    <p>Hi
<p>Something
</div>";

    // If arg use that file, else use example above
    let path = std::env::args().nth(1);
    let content = if let Some(ref path) = path {
        std::fs::read_to_string(path).unwrap()
    } else {
        example.to_owned()
    };

    let second_arg = std::env::args().nth(2);
    let mode = second_arg.unwrap_or_default();

    const STACK_SIZE: usize = 8 * 1024 * 1024;

    std::thread::scope(|s| {
        let thread = std::thread::Builder::new()
            .name("Parsing thread".to_owned())
            .stack_size(STACK_SIZE)
            .spawn_scoped(s, || {
                let path = path.unwrap_or("*example*".to_owned());
                let file = SimpleFile::new(path, content.clone());
                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = Config::default();

                let allocator = Allocator::default();
                let mut lexer = Lexer::new(&content);
                let result = Document::from_reader(&mut lexer, &allocator);

                // dbg!(content.len(), allocator.allocated_bytes());

                match mode.as_str() {
                    "--text" => {
                        eprintln!(
                            "Text: {text}",
                            text = operations::inner_text_element(&result.unwrap().html_element)
                        );
                        std::process::ExitCode::SUCCESS
                    }
                    "--count-character" => {
                        let root = &result.unwrap().html_element;
                        pub struct Counter(pub usize);

                        impl operations::Walker for Counter {
                            fn text_node(&mut self, content: &str) {
                                self.0 += content.chars().filter(|chr| *chr == 'a').count();
                            }

                            fn attribute(&mut self, _key: &str, value: &str) {
                                self.0 += value.chars().filter(|chr| *chr == 'a').count();
                            }
                        }

                        let mut count = Counter(0);
                        operations::walk_nodes_on_element(&root, &mut count);
                        eprintln!("Found {count} 'a's", count = count.0);
                        std::process::ExitCode::SUCCESS
                    }
                    "--verbose" => {
                        println!("{result:#?}");
                        std::process::ExitCode::SUCCESS
                    }
                    "--check" => match result {
                        Ok(_) => {
                            eprintln!("Parsed successfully");
                            std::process::ExitCode::SUCCESS
                        }
                        Err(err) => {
                            let at = err.at as usize;
                            let errors = Diagnostic::error();
                            let diagnostic = errors
                                .with_label(Label::primary((), at..at))
                                .with_labels_iter(err.context.into_iter().rev().map(|context| {
                                    let at = context.at as usize;
                                    Label::secondary((), at..at).with_message(format!(
                                        "Parsing element {tag_name}",
                                        tag_name = context.tag_name
                                    ))
                                }))
                                .with_message(format!("Error: {reason:?}", reason = err.reason));
                            term::emit(&mut writer.lock(), &config, &file, &diagnostic)
                                .expect("Error emitting");

                            std::process::ExitCode::FAILURE
                        }
                    },
                    _ => {
                        eprintln!("{result:?}");
                        std::process::ExitCode::SUCCESS
                    }
                }
            })
            .unwrap();

        let exit_code = thread.join().unwrap();

        exit_code
    })
}
