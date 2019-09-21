use rand::prelude::*;
use std::{
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

        Ok(Self {
            metadata,
            file,
            rng,
        })
    }

    pub fn version(&self) -> u32 {
        u32::from_be_bytes([
            self.metadata[0],
            self.metadata[1],
            self.metadata[2],
            self.metadata[3],
        ])
    }

    pub fn count(&self) -> u32 {
        u32::from_be_bytes([
            self.metadata[4],
            self.metadata[5],
            self.metadata[6],
            self.metadata[7],
        ])
    }

    pub fn max_length(&self) -> u32 {
        u32::from_be_bytes([
            self.metadata[8],
            self.metadata[9],
            self.metadata[10],
            self.metadata[11],
        ])
    }

    pub fn min_length(&self) -> u32 {
        u32::from_be_bytes([
            self.metadata[12],
            self.metadata[13],
            self.metadata[14],
            self.metadata[15],
        ])
    }

    pub fn is_encrypted(&self) -> bool {
        u32::from_be_bytes([
            self.metadata[12],
            self.metadata[13],
            self.metadata[14],
            self.metadata[15],
        ]) == 0x4
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

        let start = u32::from_be_bytes([
            self.metadata[index],
            self.metadata[index + 1],
            self.metadata[index + 2],
            self.metadata[index + 3],
        ]) as usize;
        let index = index + 4;
        let end = u32::from_be_bytes([
            self.metadata[index],
            self.metadata[index + 1],
            self.metadata[index + 2],
            self.metadata[index + 3],
        ]) as usize;

        self.file.seek(SeekFrom::Start(start as u64))?;
        let mut buf = vec![0; end - start];
        self.file.read_exact(&mut buf)?;
        let quote = String::from_utf8_lossy(&buf);
        Ok(quote
            .trim_matches(|c| self.delim() == c || c.is_whitespace())
            .into())
    }
}
