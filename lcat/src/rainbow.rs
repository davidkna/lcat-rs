use std::{
    io::{prelude::*, Write},
    num::NonZero,
};

pub use anstyle::Color;
use anstyle::{Ansi256Color, AnsiColor, RgbColor};
use bstr::{io::BufReadExt, ByteSlice};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use crate::ok::{Hsv, Lab, LinRgb, Rgb};

pub struct AnsiFallbackGrad<G>(G);

impl<G: Grad> AnsiFallbackGrad<G> {
    pub const fn new(grad: G) -> Self {
        Self(grad)
    }
}

impl<G: Grad> Grad for AnsiFallbackGrad<G> {
    fn color_at(&self, pos: f32) -> Color {
        let xterm_col = anstyle_lossy::color_to_xterm(self.0.color_at(pos));

        Color::Ansi256(xterm_col)
    }
}

pub trait Grad {
    fn color_at(&self, pos: f32) -> Color;
}

impl<T: colorgrad::Gradient> Grad for T {
    fn color_at(&self, pos: f32) -> Color {
        let [r, g, b, _] = self.at(pos).to_rgba8();
        Color::Rgb(RgbColor(r, g, b))
    }
}

pub struct HsvGrad {}

impl Grad for HsvGrad {
    #[allow(clippy::cast_possible_truncation)]
    fn color_at(&self, pos: f32) -> Color {
        let Rgb { r, g, b } = Rgb::from(&LinRgb::from(&Lab::from(&Hsv {
            h: pos,
            s: 1.0,
            v: 1.0,
        })));
        Color::Rgb(RgbColor(r, g, b))
    }
}

pub struct Ansi256RainbowGrad {}

impl Grad for Ansi256RainbowGrad {
    fn color_at(&self, pos: f32) -> Color {
        #[allow(clippy::zero_prefixed_literal)]
        const RAINBOW: [u8; 30] = [
            135, 171, 207, 207, 207, 206, 205, 204, 203, 209, 215, 221, 227, 227, 227, 191, 155,
            119, 083, 084, 085, 086, 087, 087, 087, 081, 075, 069, 063, 099,
        ];
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Color::Ansi256(Ansi256Color(RAINBOW[(pos * RAINBOW.len() as f32) as usize]))
    }
}

pub struct Ansi256SinebowGrad {}

impl Grad for Ansi256SinebowGrad {
    fn color_at(&self, pos: f32) -> Color {
        #[allow(clippy::zero_prefixed_literal)]
        const RAINBOW: [u8; 30] = [
            196, 202, 208, 214, 220, 226, 190, 154, 118, 082, 046, 047, 048, 049, 050, 051, 045,
            039, 033, 027, 021, 057, 093, 129, 165, 201, 200, 199, 198, 197,
        ];
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Color::Ansi256(Ansi256Color(RAINBOW[(pos * RAINBOW.len() as f32) as usize]))
    }
}

pub struct Rainbow {
    current_row: usize,
    current_col: usize,
    shift_col: f32,
    shift_row: f32,
    position: f32,
    gradient: Box<dyn Grad>,
    invert: bool,
}

impl Rainbow {
    #[must_use]
    pub fn new(
        gradient: Box<dyn Grad>,
        start: f32,
        shift_col: f32,
        shift_row: f32,
        invert: bool,
    ) -> Self {
        Self {
            gradient,
            shift_col,
            shift_row,
            position: start,
            current_row: 0,
            current_col: 0,
            invert,
        }
    }

    pub fn step_row(&mut self, n_row: usize) {
        self.current_row += n_row;
        self.position += n_row as f32 * self.shift_row;
    }

    pub fn step_col(&mut self, n_col: usize) {
        self.current_col += n_col;
        self.position += n_col as f32 * self.shift_col;
    }

    pub fn reset_row(&mut self) {
        self.position -= self.current_row as f32 * self.shift_row;
        self.current_row = 0;
    }

    pub fn reset_col(&mut self) {
        self.position -= self.current_col as f32 * self.shift_col;
        self.current_col = 0;
    }

    fn get_position(&mut self) -> f32 {
        if self.position < 0.0 || self.position > 1.0 {
            self.position -= self.position.floor();
        }

        self.position
    }

    pub fn get_color(&mut self) -> Color {
        let position = self.get_position();
        self.gradient.color_at(position)
    }

    #[inline]
    fn handle_grapheme(
        &mut self,
        out: &mut impl Write,
        grapheme: &str,
        escaping: bool,
    ) -> std::io::Result<bool> {
        let mut escaping = escaping;
        if grapheme == "\x1B" {
            out.write_all(b"\x1B")?;
            return Ok(true);
        }
        if grapheme == "\n" || grapheme == "\r\n" {
            self.reset_col();
            self.step_row(1);
            if self.invert {
                out.write_all(b"\x1B[49m")?;
            }
            out.write_all(grapheme.as_bytes())?;

            return Ok(false);
        }

        if escaping {
            out.write_all(grapheme.as_bytes())?;
            escaping = grapheme.len() != 1
                || !grapheme
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphabetic);
        } else {
            let color = self.get_color();
            if self.invert {
                write!(out, "{}{grapheme}", AnsiColor::Black.on(color))?;
            } else {
                write!(out, "{}{grapheme}", color.render_fg())?;
            }

            self.step_col(
                grapheme
                    .chars()
                    .next()
                    .and_then(UnicodeWidthChar::width)
                    .unwrap_or(0),
            );
        }
        Ok(escaping)
    }

    /// # Errors
    ///
    /// Will return `Err` if `out` causes I/O erros
    pub fn colorize(&mut self, text: &[u8], out: &mut impl Write) -> std::io::Result<()> {
        let mut escaping = false;
        for grapheme in text.graphemes() {
            escaping = self.handle_grapheme(out, grapheme, escaping)?;
        }

        out.write_all(b"\x1B[39m")?;
        if self.invert {
            out.write_all(b"\x1B[49m")?;
        }

        out.flush()
    }

    /// # Errors
    ///
    /// Will return `Err` if `out` causes I/O erros
    pub fn colorize_str(&mut self, text: &str, out: &mut impl Write) -> std::io::Result<()> {
        let mut escaping = false;
        for grapheme in UnicodeSegmentation::graphemes(text, true) {
            escaping = self.handle_grapheme(out, grapheme, escaping)?;
        }

        out.write_all(b"\x1B[39m")?;
        if self.invert {
            out.write_all(b"\x1B[49m")?;
        }
        out.flush()
    }

    /// # Errors
    ///
    /// Will return `Err` if `input` or `out` cause I/O errors
    pub fn colorize_read(
        &mut self,
        input: &mut impl BufRead,
        out: &mut impl Write,
    ) -> std::io::Result<()> {
        input.for_byte_line_with_terminator(|line| {
            self.colorize(line, out)?;
            Ok(true)
        })
    }

    /// # Errors
    ///
    /// Will return `Err` if `input` or `out` cause I/O errors
    pub fn colorize_read_streaming(
        &mut self,
        input: &mut impl BufRead,
        out: &mut impl Write,
        buffer_size: NonZero<usize>,
    ) -> std::io::Result<()> {
        let mut buffer = vec![0u8; buffer_size.get()];
        let mut bytes_in_buffer = 0;
        let mut escaping = false;

        loop {
            let bytes_read = input.read(&mut buffer[bytes_in_buffer..])?;
            bytes_in_buffer += bytes_read;

            if bytes_read == 0 {
                break;
            }

            // Try to split on a safe boundary
            let Some(split_pos) = find_safe_boundary(&buffer[..bytes_in_buffer])
                .or_else(|| (bytes_in_buffer == buffer_size.get()).then_some(buffer_size))
            else {
                continue;
            };
            let split_pos = split_pos.get();
            // Process the safe portion
            for grapheme in buffer[..split_pos].graphemes() {
                escaping = self.handle_grapheme(out, grapheme, escaping)?;
            }

            // Move leftovers to the beginning of the buffer
            if split_pos < bytes_in_buffer {
                buffer.copy_within(split_pos..bytes_in_buffer, 0);
            }
            bytes_in_buffer -= split_pos;
        }

        // process any remaining bytes in buffer
        if bytes_in_buffer > 0 {
            for grapheme in buffer[..bytes_in_buffer].graphemes() {
                escaping = self.handle_grapheme(out, grapheme, escaping)?;
            }
        }

        out.write_all(b"\x1B[39m")?;
        if self.invert {
            out.write_all(b"\x1B[49m")?;
        }

        out.flush()
    }
}

/// Find a safe boundary in the buffer where we can split without
/// breaking graphemes or unicode characters
///
/// TODO: Extend this with proper grapheme boundary detection for
/// complete correctness with complex scripts and emoji sequences
fn find_safe_boundary(buffer: &[u8]) -> Option<NonZero<usize>> {
    if buffer.is_empty() {
        return None;
    }

    buffer
        .iter()
        .rposition(|&b| !(128..192).contains(&b))
        .and_then(NonZero::new)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn create_rb() -> Rainbow {
        Rainbow::new(Box::new(colorgrad::preset::rainbow()), 0.0, 0.1, 0.2, false)
    }

    #[test]
    fn test_eq_str_u8() {
        let test = "foobar";

        let mut rb_a = create_rb();
        let mut out_a = Vec::new();
        rb_a.colorize(test.as_bytes(), &mut out_a).unwrap();

        let mut rb_b = create_rb();
        let mut out_b = Vec::new();
        rb_b.colorize_str(test, &mut out_b).unwrap();

        assert_eq!(out_a, out_b);
    }

    #[test]
    fn test_char_width() {
        let test = "f";
        let mut rb_a = create_rb();
        rb_a.colorize_str(test, &mut Vec::new()).unwrap();

        assert_eq!(rb_a.current_col, 1);

        let test = "\u{1f603}";
        let mut rb_b = create_rb();
        rb_b.colorize_str(test, &mut Vec::new()).unwrap();
        assert_eq!(rb_b.current_col, 2);
    }

    #[test]
    fn test_step_row() {
        let mut rb_expected = create_rb();
        rb_expected.step_row(1);

        for test_string in ["foobar\n", "foobar\r\n"] {
            let mut rb_actual = create_rb();
            rb_actual
                .colorize(test_string.as_bytes(), &mut Vec::new())
                .unwrap();
            assert_eq!(rb_actual.get_color(), rb_expected.get_color());
        }
    }

    #[test]
    fn test_reset_row() {
        let mut rb_a = create_rb();
        let mut rb_b = create_rb();
        rb_a.step_row(20);
        rb_a.reset_row();
        assert_eq!(rb_a.get_color(), rb_b.get_color());
    }

    #[test]
    fn test_reset_col() {
        let mut rb_a = create_rb();
        let mut rb_b = create_rb();
        rb_a.step_col(20);
        rb_a.reset_col();
        assert_eq!(rb_a.get_color(), rb_b.get_color());
    }

    #[test]
    fn test_fallback_is_ansi_color() {
        let grad = HsvGrad {};
        assert!(matches!(grad.color_at(0.), Color::Rgb(..)));
        let fallback_grad = AnsiFallbackGrad::new(grad);
        assert!(matches!(fallback_grad.color_at(0.), Color::Ansi256(..)));
    }

    fn trim_buf_at_str(buf: &[u8], idx: NonZero<usize>) -> Result<&str, std::str::Utf8Error> {
        let idx = idx.get();
        std::str::from_utf8(&buf[..idx])
    }

    #[test]
    fn test_streaming_with_ascii() {
        let input = b"hello world this is streaming text";
        let mut cursor = Cursor::new(input);

        let mut rb_a = create_rb();
        let mut out_a = Vec::new();
        // Small buffer to force multiple reads
        rb_a.colorize_read_streaming(&mut cursor, &mut out_a, NonZero::new(5).unwrap())
            .unwrap();

        let mut rb_b = create_rb();
        let mut out_b = Vec::new();
        rb_b.colorize(input, &mut out_b).unwrap();

        assert_eq!(out_a, out_b);
    }

    #[test]
    fn test_streaming_with_unicode() {
        let input = "hello 世界 emoji 😀 test".as_bytes();
        let mut cursor = Cursor::new(input);

        let mut rb_a = create_rb();
        let mut out_a = Vec::new();

        // Small buffer to force splits in unicode str
        rb_a.colorize_read_streaming(&mut cursor, &mut out_a, NonZero::new(4).unwrap())
            .unwrap();

        let mut rb_b = create_rb();
        let mut out_b = Vec::new();
        rb_b.colorize(input, &mut out_b).unwrap();

        assert_eq!(out_a.as_bstr(), out_b.as_bstr());
    }

    #[test]
    fn test_streaming_with_invalid_utf8() {
        // Create some invalid UTF-8 by breaking a multi-byte sequence
        let mut input = Vec::from(b"hello ");
        input.extend_from_slice(&[0xe2, 0x82]); // Incomplete UTF-8 sequence
        input.extend_from_slice(b" world");

        let mut cursor = Cursor::new(&input);

        let mut rb_a = create_rb();
        let mut out_a = Vec::new();
        // This should not panic even with invalid UTF-8
        assert!(rb_a
            .colorize_read_streaming(&mut cursor, &mut out_a, NonZero::new(3).unwrap())
            .is_ok());

        // The output should contain something (we don't test exact equality since
        // the handling of invalid UTF-8 might differ between methods)
        assert!(!out_a.is_empty());
    }

    #[test]
    fn test_find_safe_boundary() -> Result<(), std::str::Utf8Error> {
        // ASCII should find boundary after each character
        let ascii_str = b"hello";
        let pos = find_safe_boundary(ascii_str).unwrap();
        assert_eq!("hell", trim_buf_at_str(ascii_str, pos)?);

        // Unicode should find character boundaries
        let mixed_str = "hello世界".as_bytes();
        let pos = find_safe_boundary(mixed_str).unwrap();
        assert_eq!("hello世", trim_buf_at_str(mixed_str, pos)?);

        let unicode_str = "世界".as_bytes();
        let pos = find_safe_boundary(unicode_str).unwrap();
        assert_eq!("世", trim_buf_at_str(unicode_str, pos)?);

        // Empty buffer should return None
        assert_eq!(find_safe_boundary(&[]), None);

        Ok(())
    }
}
