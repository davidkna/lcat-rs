use crate::{bubble::bubble, shapes::CowShape};
use std::fmt;

pub struct Cow {
    shape: CowShape,
    text: String,
    max_length: usize,
}

impl Cow {
    pub fn new(shape: CowShape, text: String, max_length: usize) -> Self {
        Self {
            shape,
            text,
            max_length,
        }
    }
}

impl fmt::Display for Cow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let speech_bubble = bubble(&self.text, self.max_length);

        write!(f, "{}{}", speech_bubble, self.shape)
    }
}
