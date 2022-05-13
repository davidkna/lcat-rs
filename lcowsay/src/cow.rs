use std::fmt;

use crate::{bubble::bubble, shapes::CowShape};

pub struct Cow {
    shape: CowShape,
    text: String,
    max_length: usize,
}

impl Cow {
    #[must_use]
    pub const fn new(shape: CowShape, text: String, max_length: usize) -> Self {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cow_new() {
        let cow = Cow::new(CowShape::Cow, "Hello, world!".to_string(), 10);
        assert_eq!(cow.text, "Hello, world!");
        assert_eq!(cow.max_length, 10);
    }
}
