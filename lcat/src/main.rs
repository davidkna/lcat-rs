#![warn(clippy::pedantic, clippy::nursery)]

use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
};

use clap::Parser;
use lcat::{Rainbow, RainbowCmd};

#[derive(Parser)]
#[clap(name = "lcat", about = "Terminal rainbows.")]
pub struct Cmdline {
    #[clap(name = "File", default_value = "-")]
    files: Vec<PathBuf>,

    #[clap(flatten)]
    rainbow: RainbowCmd,
}

fn main() -> Result<(), io::Error> {
    let opt = Cmdline::parse();

    let mut rainbow: Rainbow = opt.rainbow.into();
    let mut stdout = io::stdout().lock();

    for path in opt.files {
        if path == PathBuf::from("-") {
            let mut stdin = io::stdin().lock();
            rainbow.colorize_read(&mut stdin, &mut stdout)?;
        } else {
            let f = File::open(path).unwrap();
            let mut b = BufReader::new(f);
            rainbow.colorize_read(&mut b, &mut stdout)?;
        }
    }

    Ok(())
}
