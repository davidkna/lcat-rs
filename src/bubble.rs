use hyphenation::{Language, Load, Standard};
use std::borrow::Borrow;
use std::fmt::Write;
use textwrap::Wrapper;
use unicode_width::UnicodeWidthStr;

pub(crate) fn bubble(text: &str, width: usize) -> String {
    let hyphenator = Standard::from_embedded(Language::EnglishUS).unwrap();
    let wrapper = Wrapper::with_splitter(width, hyphenator);

    let text = wrapper.wrap(&text);

    let max_length = text
        .iter()
        .map(|i| {
            let i: &str = i.borrow();
            UnicodeWidthStr::width(i)
        })
        .max()
        .unwrap();

    let mut out = String::new();
    writeln!(out, "╭{:─<1$}╮", "", max_length + 2).unwrap();
    for line in text {
        writeln!(out, "│ {:1$} │", &line, max_length).unwrap();
    }
    write!(out, "╰{:─<1$}╯", "", max_length + 2).unwrap();

    out
}
