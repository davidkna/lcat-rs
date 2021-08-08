use deku::prelude::*;
use itertools::Itertools;
use memchr::memmem::find_iter;
use std::{
    convert::TryInto,
    fs::{self, File},
    io::{self, prelude::*, Seek, SeekFrom},
    num::NonZeroU32,
    path::Path,
    result::Result,
};

#[derive(thiserror::Error, Debug)]
pub enum StrfileError {
    #[error("failed to open file {0}")]
    Open(io::Error),
    #[error("failed to parse datfile {0}")]
    DatParse(DekuError),
    #[error("failed to get quote {0}")]
    GetQuote(io::Error),
    #[error("I/O Error {0}")]
    Io(#[from] io::Error),
    #[error("QuoteIndex")]
    QuoteIndex,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Default)]
#[deku(endian = "big")]
pub struct Datfile {
    pub version: u32,
    #[deku(update = "self.data.len()")]
    pub count: u32,
    pub max_length: u32,
    pub min_length: u32,
    pub flags: u32,
    #[deku(pad_bytes_after = "3")]
    pub delim: u8,
    #[deku(count = "count")]
    pub data: Vec<u32>,
}

impl Datfile {
    pub const fn is_random(&self) -> bool {
        (self.flags & 0x1) != 0
    }

    pub const fn is_ordered(&self) -> bool {
        (self.flags & 0x2) != 0
    }

    pub const fn is_encrypted(&self) -> bool {
        (self.flags & 0x4) != 0
    }

    pub fn get(&self, index: usize) -> Option<(u32, NonZeroU32)> {
        let mut a = *self.data.get(index)?;
        if a != 0 {
            a += 2;
        };
        let b = if self.is_ordered() || self.is_random() {
            self.data
                .iter()
                .filter_map(|i| i.checked_sub(1))
                .filter(|i| *i > a)
                .min()?
        } else {
            self.data.get(index + 1)?.checked_sub(1)?
        };

        Some((a, NonZeroU32::new(b)?))
    }

    pub fn build(strfile: &[u8], delim: u8, flags: u32) -> Self {
        let mut min_length = u32::MAX;
        let mut max_length = 0;

        let data: Vec<u32> = std::iter::once(0_u32)
            .chain(find_iter(strfile, &[b'\n', delim, b'\n']).filter_map(|i| i.try_into().ok()))
            .tuple_windows()
            .filter_map(|(a, b)| {
                let skip = if a == 0 { 0 } else { 2 };
                let diff = b.checked_sub(a + skip)?;
                if diff > max_length {
                    max_length = diff;
                }
                if diff < min_length {
                    min_length = diff;
                }
                Some(b)
            })
            .collect();

        if max_length == 0 {
            max_length = 0;
        }

        Self {
            version: 2,
            count: data.len().try_into().unwrap_or(u32::MAX),
            min_length,
            max_length,
            flags,
            delim,
            data,
        }
    }
}

pub struct Strfile {
    file: File,
    pub metadata: Datfile,
}

impl Strfile {
    pub fn new(strfile: &Path, datfile: &Path) -> Result<Self, StrfileError> {
        let datfile = fs::read(datfile).map_err(StrfileError::Open)?;
        let metadata = Datfile::from_bytes((&datfile, 0))
            .map_err(StrfileError::DatParse)?
            .1;

        let file = File::open(strfile).map_err(StrfileError::Open)?;

        Ok(Self { file, metadata })
    }

    pub fn random_quote(&mut self) -> Result<String, StrfileError> {
        self.get_quote(fastrand::usize(..self.metadata.count as usize - 1))
    }

    pub fn get_quote(&mut self, index: usize) -> Result<String, StrfileError> {
        let (start, end) = self.metadata.get(index).ok_or(StrfileError::QuoteIndex)?;

        self.file
            .seek(SeekFrom::Start(u64::from(start)))
            .map_err(StrfileError::GetQuote)?;
        let mut buf = vec![0; (end.get() - start) as usize];
        self.file
            .read_exact(&mut buf)
            .map_err(StrfileError::GetQuote)?;
        let mut quote = String::from_utf8_lossy(&buf).to_string();

        if self.metadata.is_encrypted() {
            let view = unsafe { quote.as_mut_vec() };
            for i in view.iter_mut() {
                match i {
                    b'a'..=b'm' | b'A'..=b'M' => *i += 13,
                    b'n'..=b'z' | b'N'..=b'Z' => *i -= 13,
                    _ => (),
                };
            }
        }

        Ok(quote)
    }
}
