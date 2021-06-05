#![warn(clippy::pedantic, clippy::nursery)]

use clap::Clap;
use lcat::{Rainbow, RainbowCmd};
use lcowsay::{Cow, CowShape};
use std::io::{self, Read, Write};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Clap)]
struct Opt {
    #[clap(short = 'f', long = "cow-shape", arg_enum, default_value = "cow")]
    shape: CowShape,
    #[clap(short = 'W', long = "max-length", default_value = "40")]
    max_length: usize,
    #[clap(long = "no-lolcat")]
    nololcat: bool,
    #[clap(name = "TEXT", default_value = "")]
    text: Vec<String>,
    #[clap(flatten)]
    rainbow: RainbowCmd,
}

fn main() -> io::Result<()> {
    let opt = Opt::parse();
    let mut text = opt.text.join(" ");

    if text.trim() == "" {
        io::stdin().read_to_string(&mut text).unwrap();
        text = text.trim().to_string();
    }

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let cow = Cow::new(opt.shape, text, opt.max_length);
    let cow = format!("{}\n", cow);
    if opt.nololcat {
        stdout.write_all(cow.as_bytes())?;
    } else {
        let mut rainbow: Rainbow = opt.rainbow.into();
        rainbow.colorize_str(&cow, &mut stdout)?;
    }
    stdout.flush()
}
