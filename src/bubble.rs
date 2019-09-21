use hyphenation::{Language, Load, Standard};
use std::{borrow::Borrow, fmt::Write};
use textwrap::Wrapper;
use unicode_width::UnicodeWidthStr;

pub(crate) fn bubble(text: &str, width: usize) -> String {
    let hyphenator = Standard::from_embedded(Language::EnglishUS).unwrap();
    let wrapper = Wrapper::with_splitter(width, hyphenator);

    let text = text.replace("\t", "    ");
    let text = wrapper.wrap(&text);

    let line_count = text.len();
    let line_lengths: Vec<usize> = text
        .iter()
        .map(|i| {
            let i: &str = i.borrow();
            UnicodeWidthStr::width(i)
        })
        .collect();

    let max_length = line_lengths.iter().max().unwrap();

    let mut out = String::new();
    writeln!(out, " {:_<1$} ", "", max_length + 2).unwrap();
    if line_count == 1 {
        writeln!(out, "< {} >", &text[0]).unwrap();
    } else {
        writeln!(out, "/ {:1$} \\", &text[0], max_length).unwrap();
        for (i, line) in text.iter().take(line_count - 1).skip(1).enumerate() {
            let i = i + 1;
            let spaces_count = max_length - line_lengths[i];
            writeln!(out, "| {}{} |", &line, " ".repeat(spaces_count)).unwrap();
        }
        writeln!(out, "\\ {:1$} /", &text[line_count - 1], max_length).unwrap();
    }
    write!(out, " {:-<1$} ", "", max_length + 2).unwrap();

    out
}
