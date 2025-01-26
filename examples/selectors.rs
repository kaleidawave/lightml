fn main() {
    use lightml::matching::Selector;
    // let selector = "a[href^='/news/articles']";
    let selector = "#nations-news-uk";
    let selector = Selector::from_string(selector.into());
    dbg!(selector);
}
