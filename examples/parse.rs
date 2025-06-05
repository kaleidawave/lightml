use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::{
    self,
    termcolor::{ColorChoice, StandardStream},
    Config,
};

fn main() {
    let example = r#"<div>
    <p>Hi
<p>Something
</div>"#;

    // If arg use that file, else use example above
    let path = std::env::args().nth(1);
    let content = if let Some(ref path) = path {
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
        let path = path.unwrap_or("*example*".to_owned());
        let file = SimpleFile::new(path, content.clone());
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = Config::default();

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
                    text = operations::inner_text_element(&result.unwrap().html_element)
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
                    // panic!("Could not parse {err:?}");
                }
            },
            _ => {
                eprintln!("{result:?}");
            }
        }
    });
}
