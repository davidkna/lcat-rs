mod bubble;
mod cow;
mod shapes;

use crate::{cow::Cow, shapes::CowShape};
use lcat::{Rainbow, RainbowCmd};
use std::io::{self, Read};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt( short = "f", long = "cow-shape", possible_values = &["cow", "clippy", "ferris", "moose"], case_insensitive = true, default_value = "cow")]
    shape: CowShape,
    #[structopt(short = "W", long = "max-length", default_value = "40")]
    max_length: usize,
    #[structopt(long = "lolcat")]
    nololcat: bool,
    #[structopt(name = "TEXT", default_value = "")]
    text: Vec<String>,
    #[structopt(flatten)]
    rainbow: RainbowCmd,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let mut text = opt.text.join(" ");

    if text.trim() == "" {
        io::stdin().read_to_string(&mut text).unwrap();
        text = text.trim().to_string();
    }

    let cow = Cow::new(opt.shape, text, opt.max_length);
    let cow = format!("{}", cow);

    if !opt.nololcat {
        let mut rainbow: Rainbow = opt.rainbow.into();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        rainbow.colorize_str(&cow, &mut stdout)?;
    } else {
        print!("{}", cow);
    }
    Ok(())

}
