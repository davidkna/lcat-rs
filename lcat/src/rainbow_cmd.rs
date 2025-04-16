use clap::{Parser, ValueEnum};

use crate::{Ansi256Grad, Grad, HsvGrad, Rainbow};

#[derive(Debug, Clone, ValueEnum)]
pub enum RainbowStyle {
    Rainbow,
    Sinebow,
    OkHsv,
    Ansi,
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

        let grad: Box<dyn Grad> = match cmd.style {
            RainbowStyle::Rainbow => Box::new(colorgrad::preset::rainbow()),
            RainbowStyle::Sinebow => Box::new(colorgrad::preset::sinebow()),
            RainbowStyle::OkHsv => Box::new(HsvGrad {}),
            RainbowStyle::Ansi => Box::new(Ansi256Grad {}),
        };

        Self::new(grad, start, shift_col, shift_row, cmd.invert)
    }
}
