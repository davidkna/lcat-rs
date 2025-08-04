use std::{env, io, io::Write, path::PathBuf, result::Result, str};

use clap::Parser;
use directories::ProjectDirs;
use lcat::{Rainbow, RainbowCmd};
use lcowsay::{Cow, CowShape};
use lolcow_fortune::{Strfile, StrfileError};

#[derive(Parser)]
struct Opt {
    #[clap(short = 'f', long = "files")]
    strfiles: Option<String>,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// Cowsay a fortune
    Cowsay {
        #[clap(short = 'f', long = "cow-shape", value_enum, default_value = "cow")]
        shape: CowShape,
        #[clap(short = 'W', long = "max-length", default_value = "40")]
        max_length: usize,
        #[clap(short = 'L', long = "lolcat")]
        lolcat: bool,
        #[clap(flatten)]
        rainbow: RainbowCmd,
    },
    /// Download a fortune database
    Download,
    /// Tell a fortune
    Tell,
}

fn get_project_dir() -> ProjectDirs {
    ProjectDirs::from("moe", "knaack", "fortune").unwrap()
}

#[cfg(feature = "download")]
fn download() -> Result<(), lolcow_fortune::StrfileError> {
    use std::{
        fs::{self, File},
        io::Read,
        path::Path,
        sync::Arc,
    };

    use deku::DekuWriter;
    use flate2::read::GzDecoder;
    use lolcow_fortune::Datfile;
    use tar::Archive;
    use ureq::tls::{TlsConfig, TlsProvider};

    let crypto = Arc::new(rustls_graviola::default_provider());
    let agent = ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                // requires rustls or rustls-no-provider feature
                .unversioned_rustls_crypto_provider(crypto)
                .build(),
        )
        .build()
        .new_agent();

    let response = agent
        .get("https://github.com/shlomif/fortune-mod/archive/master.tar.gz")
        .call()
        .map_err(Box::new)?;

    let (_, body) = response.into_parts();
    let reader = body.into_reader();
    let gz_data = GzDecoder::new(reader);
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
            if path.extension().is_some()
                || path.parent().is_some() && path.parent() != Some(Path::new(""))
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

            let dat_file_data = Datfile::build(&buffer, b'%', 0);
            let dat_file_writer = File::create(target.with_extension("dat"))?;

            dat_file_data
                .to_writer(&mut deku::writer::Writer::new(dat_file_writer), ())
                .map_err(StrfileError::Write)?;
        }
    }

    Ok(())
}

fn get_fortune_dirs(from_opts: Option<String>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(files) = from_opts {
        env::split_paths(&files).for_each(|i| dirs.push(i));
    }
    dirs.push(get_project_dir().data_dir().to_path_buf());
    if let Ok(files) = env::var("FORTUNE_PATH") {
        env::split_paths(&files).for_each(|i| dirs.push(i));
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
                iter.filter_map(std::result::Result::ok)
                    .filter_map(|j| {
                        let path = j.path();
                        if j.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                            && path.extension().is_some_and(|ext| ext == "dat")
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
            .unwrap_or_default();
        if dat_files.is_empty() {
            None
        } else {
            Some(dat_files)
        }
    })
}

fn get_random_quote(cmd_path: Option<String>) -> Result<String, StrfileError> {
    let data_dirs = get_fortune_dirs(cmd_path);
    let fortune_files = get_fortune_files(&data_dirs).expect("Unable to find any fortune dbs.");

    let idx = fastrand::usize(..fortune_files.len());
    let (meta_file, fortunes_file) = &fortune_files[idx];
    let mut strfile = Strfile::new(fortunes_file, meta_file)?;
    strfile.random_quote()
}

fn main() -> Result<(), lolcow_fortune::StrfileError> {
    let opt = Opt::parse();

    match opt.cmd {
        Command::Cowsay {
            shape,
            max_length,
            lolcat,
            rainbow,
        } => {
            let quote = get_random_quote(opt.strfiles)?;
            let cow = Cow::new(shape, quote, max_length);
            let cow = format!("{cow}\n");
            let stdout = io::stdout();
            let mut stdout = stdout.lock();

            if lolcat {
                let mut rainbow: Rainbow = rainbow.into();

                rainbow.colorize_str(&cow, &mut stdout)?;
            } else {
                stdout.write_all(cow.as_bytes())?;
            }
            stdout.flush()?;
        }
        Command::Download => {
            #[cfg(feature = "download")]
            {
                download()?;
                println!("Done!");
            }
            #[cfg(not(feature = "download"))]
            {
                eprintln!("This build of lcowsay was not compiled with the download feature.");
                eprintln!("Please recompile with the `download` feature to use this command.");
                std::process::exit(1);
            }
        }
        Command::Tell => {
            let quote = get_random_quote(opt.strfiles)?;
            print!("{quote}");
        }
    }
    Ok(())
}
