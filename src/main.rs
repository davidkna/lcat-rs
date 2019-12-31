#[cfg(windows)]
use ansi_term;
use cowsay::{Cow, CowShape};
use directories::ProjectDirs;
use lolcat::Rainbow;
use neo_fortune::*;
use rand::prelude::*;
use std::{
    env, fs,
    fs::File,
    io,
    io::{Read, Write},
    path::PathBuf,
    str,
};
use structopt::StructOpt;

use flate2::read::GzDecoder;
use reqwest;
use std::path::Path;
use tar::Archive;

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
    Download,
    Tell,
}

fn get_project_dir() -> ProjectDirs {
    ProjectDirs::from("moe", "knaack", "fortune").unwrap()
}

fn download() -> io::Result<()> {
    let request =
        reqwest::blocking::get("https://github.com/shlomif/fortune-mod/archive/master.tar.gz")
            .unwrap();
    let gz_data = GzDecoder::new(request);
    let mut archive = Archive::new(gz_data);

    let project_dir = get_project_dir();
    let target_dir = project_dir.data_dir();

    for file in archive.entries()? {
        let mut file = file?;
        if !file.header().entry_type().is_file() {
            continue;
        }
        let p = file.path()?.into_owned();
        if let Ok(path) = p.strip_prefix("fortune-mod-master/fortune-mod/datfiles") {
            if path.extension() != None
                || path.parent() != None && path.parent() != Some(&Path::new(""))
            {
                continue;
            }
            println!("Downloaded {}…", path.display());
            let target = target_dir.join(path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let mut str_file = File::create(&target)?;
            str_file.write_all(&buffer)?;

            let dat_file = build_dat_file(&buffer, b'%', 0);
            let mut dat = File::create(target.with_extension("dat"))?;
            dat.write_all(&dat_file)?;
        }
    }

    Ok(())
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

fn get_fortune_files(dirs: &[PathBuf]) -> Option<Vec<(PathBuf, PathBuf)>> {
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
                                return Some((path.clone(), path.with_extension("u8")));
                            } else if path.with_extension("").exists() {
                                return Some((path.clone(), path.with_extension("")));
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

fn get_random_quote(cmd_path: Option<String>) -> io::Result<String> {
    let mut rng = SmallRng::from_entropy();

    let data_dirs = get_fortune_dirs(cmd_path);
    let fortune_files = get_fortune_files(&data_dirs).expect("Unable to find any fortune dbs.");

    let (dat_file, str_file) = fortune_files.choose(&mut rng).unwrap();
    let mut strfile = Strfile::new(str_file, dat_file)?;
    strfile.random_quote()
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Cowsay {
            shape,
            max_length,
            lolcat,
        } => {
            let quote = get_random_quote(opt.strfiles)?;
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
        Command::Download => {
            download()?;
            println!("Done!");
        }
        Command::Tell => {
            let quote = get_random_quote(opt.strfiles)?;
            print!("{}", quote);
        }
    };
    Ok(())
}
