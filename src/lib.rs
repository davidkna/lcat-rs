use memchr::Memchr;
use rand::prelude::*;
use std::{
    cmp,
    convert::TryInto,
    fs::File,
    io,
    io::{prelude::*, Seek, SeekFrom},
    path::PathBuf,
};

pub struct Strfile {
    file: File,
    metadata: Vec<u8>,
    rng: SmallRng,
}

impl Strfile {
    pub fn new(strfile: &PathBuf, datfile: &PathBuf) -> io::Result<Self> {
        let mut datfile = File::open(datfile)?;
        let mut metadata = Vec::new();
        datfile.read_to_end(&mut metadata)?;

        let file = File::open(strfile)?;
        let rng = SmallRng::from_entropy();

        assert!(metadata.len() >= 32);

        Ok(Self {
            metadata,
            file,
            rng,
        })
    }

    pub fn version(&self) -> u32 {
        u32::from_be_bytes(self.metadata[0..4].try_into().unwrap())
    }

    pub fn count(&self) -> u32 {
        cmp::min(
            u32::from_be_bytes(self.metadata[4..8].try_into().unwrap()),
            (((self.metadata.len() - 24) / 4) - 1).try_into().unwrap(),
        )
    }

    pub fn max_length(&self) -> u32 {
        u32::from_be_bytes(self.metadata[8..12].try_into().unwrap())
    }

    pub fn min_length(&self) -> u32 {
        u32::from_be_bytes(self.metadata[12..14].try_into().unwrap())
    }

    pub fn is_encrypted(&self) -> bool {
        u32::from_be_bytes(self.metadata[14..16].try_into().unwrap()) == 0x4
    }

    pub fn delim(&self) -> char {
        char::from(self.metadata[20])
    }

    pub fn random_quote(&mut self) -> io::Result<String> {
        let index = self.rng.gen_range(0, self.count() as usize);
        self.get_quote(index)
    }

    pub fn get_quote(&mut self, index: usize) -> io::Result<String> {
        let index = index * 4 + 24;

        let start =
            u32::from_be_bytes(self.metadata[index..index + 4].try_into().unwrap()) as usize;
        let end =
            u32::from_be_bytes(self.metadata[index + 4..index + 8].try_into().unwrap()) as usize;

        self.file.seek(SeekFrom::Start(start as u64))?;
        let mut buf = vec![0; end - start];
        self.file.read_exact(&mut buf)?;
        let quote = String::from_utf8_lossy(&buf);
        Ok(quote
            .trim_matches(|c: char| self.delim() == c || c.is_whitespace())
            .into())
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
