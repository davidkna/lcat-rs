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

    for path in opt.files {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        if path == PathBuf::from("-") {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            rainbow.colorize_read(&mut stdin, &mut stdout)?;
        } else {
            let f = File::open(path).unwrap();
            let mut b = BufReader::new(f);
            rainbow.colorize_read(&mut b, &mut stdout)?;
        }
    }

    Ok(())
}
