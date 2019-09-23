#[cfg(windows)]
use ansi_term;
use cowsay::{Cow, CowShape};
use directories::ProjectDirs;
use fortune::*;
use lolcat::Rainbow;
use rand::prelude::*;
use std::{env, io, path::PathBuf, str};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "f", long = "files")]
    strfiles: Option<String>,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Cowsay {
        #[structopt(short = "f", long = "cow-shape", possible_values = &["cow", "clippy", "ferris", "moose"], case_insensitive = true, default_value = "cow")]
        shape: CowShape,
        #[structopt(short = "W", long = "max-length", default_value = "40")]
        max_length: usize,
        #[structopt(short = "l", long = "lolcat")]
        lolcat: bool,
    },
    Say,
}

fn get_project_dir() -> ProjectDirs {
    ProjectDirs::from("moe", "knaack", "fortune").unwrap()
}

fn get_fortune_dirs(from_opts: Option<String>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(files) = from_opts {
        files
            .split(':')
            .map(PathBuf::from)
            .for_each(|i| dirs.push(i));
    }
    dirs.push(get_project_dir().data_dir().to_path_buf());
    if let Ok(files) = env::var("FORTUNE_PATH") {
        files
            .split(':')
            .map(PathBuf::from)
            .for_each(|i| dirs.push(i));
    }
    if !cfg!(windows) {
        dirs.push("/usr/share/games/fortune/".into());
        dirs.push("/usr/share/fortune".into());
    }

    dirs
}

fn get_fortune_files(dirs: &Vec<PathBuf>) -> Option<Vec<(PathBuf, PathBuf)>> {
    dirs.iter().find_map(|i| {
        let dat_files: Vec<(PathBuf, PathBuf)> = i
            .read_dir()
            .map(|iter| {
                iter.filter_map(|j| j.ok())
                    .filter_map(|j| {
                        let path = j.path();
                        if j.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                            && path.extension().map(|ext| ext == "dat").unwrap_or(false)
                        {
                            if path.with_extension("u8").exists() {
                                return Some((path.clone(), path.clone().with_extension("u8")));
                            } else if path.with_extension("").exists() {
                                return Some((path.clone(), path.clone().with_extension("")));
                            }
                        }
                        None
                    })
                    .collect()
            })
            .unwrap_or_else(|_| Vec::new());
        if dat_files.is_empty() {
            None
        } else {
            Some(dat_files)
        }
    })
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();
    let mut rng = SmallRng::from_entropy();

    let data_dirs = get_fortune_dirs(opt.strfiles);
    let fortune_files = get_fortune_files(&data_dirs).expect("Unable to find any fortune dbs.");

    let (dat_file, str_file) = fortune_files.choose(&mut rng).unwrap();

    let mut strfile = Strfile::new(str_file, dat_file)?;
    let quote = strfile.random_quote()?;

    match opt.cmd {
        Command::Say => print!("{}", quote),
        Command::Cowsay {
            shape,
            max_length,
            lolcat,
        } => {
            let cow = Cow::new(shape, quote, max_length);
            let mut cow = format!("{}", cow);

            if lolcat {
                #[cfg(windows)]
                ansi_term::enable_ansi_support().unwrap();
                let mut rainbow = Rainbow::default();
                cow = rainbow.colorize(&cow);
            }
            print!("{}", &cow);
        }
    };
    Ok(())
}
