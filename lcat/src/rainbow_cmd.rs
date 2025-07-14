use std::env;

use clap::{Parser, ValueEnum};

use crate::{Ansi256RainbowGrad, Ansi256SinebowGrad, AnsiFallbackGrad, Grad, HsvGrad, Rainbow};

#[derive(Debug, Clone, ValueEnum)]
pub enum RainbowStyle {
    Rainbow,
    Sinebow,
    OkHsv,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    TrueColor,
    Ansi256,
}

impl ColorMode {
    pub fn from_env() -> Self {
        match env::var("COLORTERM")
            .ok()
            .filter(String::is_empty)
            .as_deref()
        {
            // Known values for truecolor
            Some("truecolor" | "24bit") => Self::TrueColor,
            // Any other unknown values maps to ANSI
            Some(_) => Self::Ansi256,
            // Apple Terminal does set COLORTERM and does not support Truecolor
            None if env::var("TERM_PROGRAM").as_deref() == Ok("Apple_Terminal") => Self::Ansi256,
            // Assume Truecolor is supported unless directed otherwise
            _ => Self::TrueColor,
        }
    }
}

#[derive(Debug, Parser)]
pub struct RainbowCmd {
    ///  How many degrees to shift text color hue for every column
    #[clap(short = 'C', long, default_value = "1.6")]
    shift_col: f32,

    /// How many degrees to shift text color hue for every row
    #[clap(short = 'R', long, default_value = "3.2")]
    shift_row: f32,

    /// Don't randomize sign of col and row shift values
    #[clap(short = 'n', long)]
    shift_sign_no_random: bool,

    /// Sets initial hue of text color in degress [default: random]
    #[clap(short = 'H', long)]
    hue: Option<f32>,

    /// Rainbow mode
    #[clap(short, long, value_enum, default_value = "rainbow")]
    style: RainbowStyle,

    // Color mode [default: from env]
    #[clap(short, long, value_enum)]
    color_mode: Option<ColorMode>,

    /// Sets seed [default: random]
    #[clap(short = 'S', long)]
    seed: Option<u64>,

    /// Invert background and foreground
    #[clap(short = 'i', long)]
    invert: bool,
}

impl From<RainbowCmd> for Rainbow {
    fn from(cmd: RainbowCmd) -> Self {
        if let Some(seed) = cmd.seed {
            fastrand::seed(seed);
        }

        let shift_col = if cmd.shift_sign_no_random || fastrand::bool() {
            cmd.shift_col
        } else {
            -cmd.shift_col
        } / 360.;

        let shift_row = if cmd.shift_sign_no_random || fastrand::bool() {
            cmd.shift_row
        } else {
            -cmd.shift_row
        } / 360.;

        let start = cmd.hue.map_or_else(fastrand::f32, |hue| hue / 360.);

        let color_mode = cmd.color_mode.unwrap_or_else(ColorMode::from_env);

        let grad: Box<dyn Grad> = match cmd.style {
            // Automatically select variant based on native color mode
            RainbowStyle::Sinebow => match color_mode {
                ColorMode::TrueColor => Box::new(colorgrad::preset::sinebow()),
                ColorMode::Ansi256 => Box::new(Ansi256SinebowGrad {}),
            },
            // For truecolor-only gradients, use fallback mapper to ANSI 256 colors
            RainbowStyle::Rainbow => match color_mode {
                ColorMode::TrueColor => Box::new(colorgrad::preset::rainbow()),
                ColorMode::Ansi256 => Box::new(Ansi256RainbowGrad {}),
            },
            RainbowStyle::OkHsv => match color_mode {
                ColorMode::TrueColor => Box::new(HsvGrad {}),
                ColorMode::Ansi256 => Box::new(AnsiFallbackGrad::new(HsvGrad {})),
            },
        };

        Self::new(grad, start, shift_col, shift_row, cmd.invert)
    }
}
