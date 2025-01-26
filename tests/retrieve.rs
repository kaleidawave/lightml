#[test]
fn main() {
    let result = lightml::retrieve(
        "<!DOCTYPE html><html><body><h1>Hi</h1></body></html>".into(),
        "single h1\0text".into(),
    );
    assert_eq!(result, "Hi");
}
