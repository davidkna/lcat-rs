#![warn(clippy::pedantic, clippy::nursery)]

use std::{
    env,
    fs::File,
    io::{self, BufReader},
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
}

fn main() -> Result<(), io::Error> {
    let opt = Cmdline::parse();

    let mut rainbow: Rainbow = opt.rainbow.into();
    let mut stdout = io::stdout().lock();
    let is_truecolor = env::var("COLORTERM").is_ok_and(|val| val == "truecolor" || val == "24bit");

    if opt.help {
        rainbow.colorize_str(
            &Cmdline::command().render_help().ansi().to_string(),
            &mut stdout,
            is_truecolor,
        )?;
        return Ok(());
    }

    for path in opt.files {
        if path == PathBuf::from("-") {
            let mut stdin = io::stdin().lock();
            rainbow.colorize_read(&mut stdin, &mut stdout, is_truecolor)?;
        } else {
            let f = File::open(path).unwrap();
            let mut b = BufReader::new(f);
            rainbow.colorize_read(&mut b, &mut stdout, is_truecolor)?;
        }
    }

    Ok(())
}
