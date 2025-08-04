#![warn(clippy::pedantic, clippy::nursery)]

use std::{
    fs::File,
    io::{self, BufReader},
    num::NonZero,
    path::PathBuf,
};

use clap::{CommandFactory, Parser};
use lcat::{Rainbow, RainbowCmd};

#[derive(Parser)]
#[clap(name = "lcat", about = "Terminal rainbows.", disable_help_flag = true)]
pub struct Cmdline {
    #[clap(name = "File", default_value = "-")]
    files: Vec<PathBuf>,

    #[clap(flatten)]
    rainbow: RainbowCmd,

    /// Print help
    #[clap(short = 'h', long)]
    help: bool,

    /// Use streaming mode (process input as it arrives, don't wait for
    /// newlines) [EXPERIMENTAL]
    #[clap(short = 'r', long)]
    streaming: bool,
}

fn main() -> Result<(), io::Error> {
    let opt = Cmdline::parse();

    let mut rainbow: Rainbow = opt.rainbow.into();
    let mut stdout = io::stdout().lock();

    if opt.help {
        rainbow.colorize_str(
            &Cmdline::command().render_help().ansi().to_string(),
            &mut stdout,
        )?;
        return Ok(());
    }

    for path in opt.files {
        if path == PathBuf::from("-") {
            let mut stdin = io::stdin().lock();
            if opt.streaming {
                rainbow.colorize_read_streaming(
                    &mut stdin,
                    &mut stdout,
                    NonZero::new(8192).unwrap(),
                )?;
            } else {
                rainbow.colorize_read(&mut stdin, &mut stdout)?;
            }
        } else {
            let f = File::open(path).unwrap();
            let mut b = BufReader::new(f);
            if opt.streaming {
                rainbow.colorize_read_streaming(
                    &mut b,
                    &mut stdout,
                    NonZero::new(8192).unwrap(),
                )?;
            } else {
                rainbow.colorize_read(&mut b, &mut stdout)?;
            }
        }
    }

    Ok(())
}
