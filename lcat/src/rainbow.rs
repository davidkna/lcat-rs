use std::io::{prelude::*, Write};

use bstr::{io::BufReadExt, ByteSlice};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use crate::ok::{Hsv, Lab, LinRgb, Rgb};

pub trait Grad {
    fn color_at(&self, pos: f32) -> (u8, u8, u8);
}

impl<T: colorgrad::Gradient> Grad for T {
    fn color_at(&self, pos: f32) -> (u8, u8, u8) {
        let [r, g, b, _] = self.at(pos).to_rgba8();
        (r, g, b)
    }
}

pub struct HsvGrad {}

impl Grad for HsvGrad {
    #[allow(clippy::cast_possible_truncation)]
    fn color_at(&self, pos: f32) -> (u8, u8, u8) {
        let Rgb { r, g, b } = Rgb::from(&LinRgb::from(&Lab::from(&Hsv {
            h: pos,
            s: 1.0,
            v: 1.0,
        })));
        (r, g, b)
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

    pub fn get_color(&mut self) -> (u8, u8, u8) {
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
                    .map_or(false, u8::is_ascii_alphabetic);
        } else {
            let (r, g, b) = self.get_color();
            if self.invert {
                write!(out, "\x1B[38;2;0;0;0;48;2;{r};{g};{b}m{grapheme}")?;
            } else {
                write!(out, "\x1B[38;2;{r};{g};{b}m{grapheme}")?;
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
}

#[cfg(test)]
mod tests {
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
            assert_eq!(rb_actual.get_color(), rb_expected.get_color(),);
        }
    }

    #[test]
    fn test_reset_row() {
        let mut rb_a = create_rb();
        let mut rb_b = create_rb();
        rb_a.step_row(20);
        rb_a.reset_row();
        assert_eq!(rb_a.get_color(), rb_b.get_color(),);
    }

    #[test]
    fn test_reset_col() {
        let mut rb_a = create_rb();
        let mut rb_b = create_rb();
        rb_a.step_col(20);
        rb_a.reset_col();
        assert_eq!(rb_a.get_color(), rb_b.get_color(),);
    }
}
