use std::{
    convert::TryInto,
    fs::{self, File},
    io::{self, prelude::*, Seek, SeekFrom},
    num::NonZeroU64,
    path::Path,
    result::Result,
};

use deku::prelude::*;
use itertools::Itertools;
use memchr::memmem::find_iter;

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
    pub data: Vec<u64>,
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

    pub fn get(&self, index: usize) -> Option<(u64, NonZeroU64)> {
        let a = *self.data.get(index)?;
        let b = if self.is_ordered() || self.is_random() {
            self.data
                .iter()
                .filter_map(|i| i.checked_sub(2))
                .filter(|i| *i > a)
                .min()?
        } else {
            self.data.get(index + 1)?.checked_sub(2)?
        };

        Some((a, NonZeroU64::new(b)?))
    }

    pub fn build(strfile: &[u8], delim: u8, flags: u32) -> Self {
        let mut min_length = u32::MAX;
        let mut max_length = 0;

        let data: Vec<u64> = std::iter::once(0_u64)
            .chain(std::iter::once(0_u64))
            .chain(
                find_iter(strfile, &[b'\n', delim, b'\n'])
                    .filter_map(|i| i.try_into().ok())
                    .map(|i: u64| i + 3),
            )
            .tuple_windows()
            .filter_map(|(a, b)| {
                let diff = b.checked_sub(a + 2)? as u32;

                if diff > max_length {
                    max_length = diff;
                }
                if diff < min_length {
                    min_length = diff;
                }
                Some(b)
            })
            .collect();

        if max_length == u32::MAX {
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
            .seek(SeekFrom::Start(start))
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
