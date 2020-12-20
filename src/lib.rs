use memchr::Memchr;
use rand::prelude::*;
use std::{
    cmp,
    convert::TryInto,
    fs::File,
    io,
    io::{prelude::*, Seek, SeekFrom},
    path::Path,
    result::Result,
};

#[derive(thiserror::Error, Debug)]
pub enum StrfileError {
    #[error("failed to open file {0}")]
    Open(io::Error),
    #[error("invalid header size")]
    HeaderSize,
    #[error("failed to get quote {0}")]
    GetQuote(io::Error),
    #[error("IO Error {0}")]
    IO(#[from] io::Error),
}

pub struct Strfile {
    file: File,
    metadata: Vec<u32>,
    rng: SmallRng,
}

impl Strfile {
    pub fn new(strfile: &Path, datfile: &Path) -> Result<Self, StrfileError> {
        let mut datfile = File::open(datfile).map_err(StrfileError::Open)?;
        let mut metadata = Vec::new();
        datfile
            .read_to_end(&mut metadata)
            .map_err(StrfileError::Open)?;

        let file = File::open(strfile).map_err(StrfileError::Open)?;
        let rng = SmallRng::from_entropy();

        if metadata.len() < 32 {
            return Err(StrfileError::HeaderSize);
        }

        let metadata = metadata
            .chunks_exact(4)
            .map(|i| u32::from_be_bytes(i.try_into().unwrap()))
            .collect();

        Ok(Self {
            metadata,
            file,
            rng,
        })
    }

    pub fn version(&self) -> u32 {
        self.metadata[0]
    }

    pub fn count(&self) -> u32 {
        cmp::min(
            self.metadata[1],
            (self.metadata.len() - 6 - 1).try_into().unwrap(),
        )
    }

    pub fn max_length(&self) -> u32 {
        self.metadata[2]
    }

    pub fn min_length(&self) -> u32 {
        self.metadata[3]
    }

    pub fn is_encrypted(&self) -> bool {
        self.metadata[4] == 0x4
    }

    pub fn delim(&self) -> char {
        std::str::from_utf8(&self.metadata[5].to_be_bytes())
            .unwrap()
            .chars()
            .next()
            .unwrap()
    }

    pub fn random_quote(&mut self) -> Result<String, StrfileError> {
        let index = self.rng.gen_range(0..self.count() as usize);
        self.get_quote(index)
    }

    pub fn get_quote(&mut self, index: usize) -> Result<String, StrfileError> {
        let index = index + 6;

        let start = self.metadata[index] as usize;
        let end = self.metadata[index + 1] as usize;
        let delim = self.delim();

        self.file
            .seek(SeekFrom::Start(start as u64))
            .map_err(StrfileError::GetQuote)?;
        let mut buf = vec![0; end - start];
        self.file
            .read_exact(&mut buf)
            .map_err(StrfileError::GetQuote)?;
        let quote = String::from_utf8_lossy(&buf);

        Ok(quote.trim_matches(|c: char| delim == c || c == '\n').into())
    }
}

pub fn build_dat_file(strfile: &[u8], delim: u8, flags: u32) -> Vec<u8> {
    let mut pointers: Vec<u32> = vec![0];
    let mut x: Vec<u32> = Memchr::new(delim, strfile).map(|i| i as u32).collect();
    pointers.append(&mut x);
    pointers.push(strfile.len() as u32);

    let mut max_length: u32 = 0;
    let mut min_length: u32 = std::u32::MAX;

    let mut x: Vec<u32> = pointers
        .iter()
        .zip(pointers.iter().skip(1))
        .filter_map(|(a, b)| {
            let a = *a;
            let b = *b;
            if b > a
                && !String::from_utf8_lossy(&strfile[a as usize..b as usize])
                    .chars()
                    .all(|c: char| delim as char == c || c.is_whitespace())
            {
                let diff = (b - a) as u32;
                if diff > max_length {
                    max_length = diff;
                }
                if diff < min_length {
                    min_length = diff;
                }
                return Some(b);
            };
            None
        })
        .collect();
    let mut pointers = vec![0];
    pointers.append(&mut x);

    let version = u32::to_be_bytes(2);
    let count = u32::to_be_bytes(pointers.len() as u32);
    let max_length = u32::to_be_bytes(max_length);
    let min_length = u32::to_be_bytes(min_length);
    let flags = u32::to_be_bytes(flags);
    let del = [delim, 0, 0, 0];

    let mut out = Vec::new();
    out.extend_from_slice(&version);
    out.extend_from_slice(&count);
    out.extend_from_slice(&max_length);
    out.extend_from_slice(&min_length);
    out.extend_from_slice(&flags);
    out.extend_from_slice(&del);
    pointers
        .iter()
        .for_each(|i| out.extend_from_slice(&u32::to_be_bytes(*i)));

    out
}
