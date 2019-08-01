mod bubble;
mod cow;
mod shapes;

#[cfg(windows)]
use ansi_term;
use crate::cow::Cow;
use crate::shapes::CowShape;
use lolcat::Rainbow;
use std::io::{self, Read};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt( short = "f", long = "cow-shape", possible_values = &["cow", "clippy", "ferris", "moose"], case_insensitive = true, default_value = "cow")]
    shape: CowShape,
    #[structopt(short = "W", long = "max-length", default_value = "40")]
    max_length: usize,
    #[structopt(short = "l", long = "lolcat")]
    lolcat: bool,
    #[structopt(name = "TEXT", default_value = "")]
    text: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();
    let mut text = opt.text.join(" ");

    if text.trim() == "" {
        io::stdin().read_to_string(&mut text).unwrap();
        text = text.trim().to_string();
        if text == "" {
            panic!("No input given!")
        }
    }

    let cow = Cow::new(opt.shape, text, opt.max_length);
    let mut out = format!("{}", cow);

    if opt.lolcat {
        #[cfg(windows)]
        ansi_term::enable_ansi_support().unwrap();
        let mut rainbow = Rainbow::default();
        out = rainbow.rainbowify(&out);
    }

    print!("{}", out);
}
