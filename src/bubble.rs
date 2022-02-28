use hyphenation::{Language, Load, Standard};
use std::fmt::Write;
use textwrap::WordSplitter;
use unicode_width::UnicodeWidthStr;

pub fn bubble(text: &str, width: usize) -> String {
    let text = text.trim_end().replace('\t', "    ");
    let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
    let options =
        textwrap::Options::new(width).word_splitter(WordSplitter::Hyphenation(dictionary));
    let text = textwrap::wrap(&text, options);

    let line_count = text.len();
    let line_lengths: Vec<usize> = text.iter().map(|i| i.width()).collect();

    let max_length = line_lengths.iter().max().unwrap();

    let mut out = String::new();
    writeln!(out, " {:_<1$}", "", max_length + 2).unwrap();
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
    write!(out, " {:-<1$}", "", max_length + 2).unwrap();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_sl() {
        let test = "foobar";
        let bubble_t = " ________\n< foobar >\n --------";

        assert_eq!(bubble(test, 20), bubble_t);
    }

    #[test]
    fn test_bubble_break() {
        let test = "foobar";
        let bubble_t = " ______\n/ foo- \\\n\\ bar  /\n ------";

        assert_eq!(bubble(test, 5), bubble_t);
    }

    #[test]
    fn test_bubble_ml() {
        let test = "mul\ntiple\nlines";
        let bubble_t = " _______\n/ mul   \\\n| tiple |\n\\ lines /\n -------";

        assert_eq!(bubble(test, 10), bubble_t);
    }

    #[test]
    fn test_bubble_tab() {
        let test = "\t.";
        let bubble_t = " _______\n<     . >\n -------";

        assert_eq!(bubble(test, 10), bubble_t);
    }
}
