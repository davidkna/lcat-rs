use hyphenation::{Language, Load, Standard};
use std::borrow::Borrow;
use std::fmt::Write;
use textwrap::Wrapper;
use unicode_width::UnicodeWidthStr;

pub(crate) fn bubble(text: &str, width: usize) -> String {
    let hyphenator = Standard::from_embedded(Language::EnglishUS).unwrap();
    let wrapper = Wrapper::with_splitter(width, hyphenator);

    let text = wrapper.wrap(&text);

    let line_count = text.len();
    let max_length = text
        .iter()
        .map(|i| {
            let i: &str = i.borrow();
            UnicodeWidthStr::width(i)
        })
        .max()
        .unwrap();

    let mut out = String::new();
    writeln!(out, " {:_<1$} ", "", max_length + 2).unwrap();
    if line_count == 1 {
        writeln!(out, "< {} >", &text[0]).unwrap();
    } else {
        writeln!(out, "/ {:1$} \\", &text[0], max_length).unwrap();
        for line in text.iter().take(line_count - 1).skip(1) {
            writeln!(out, "| {:1$} |", &line, max_length).unwrap();
        }
        writeln!(out, "\\ {:1$} /", &text[line_count - 1], max_length).unwrap();
    }
    write!(out, " {:-<1$} ", "", max_length + 2).unwrap();

    out
}
