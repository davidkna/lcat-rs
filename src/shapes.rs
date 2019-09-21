use std::fmt;
use std::str::FromStr;

pub enum CowShape {
    Clippy,
    Cow,
    Moose,
    Ferris,
}

const COW: &str = r#"
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#;

const CLIPPY: &str = r#"
         \
          \
             __
            /  \
            |  |
            @  @
            |  |
            || |/
            || ||
            |\_/|
            \___/"#;

const FERRIS: &str = r#"
        \
         \
            _~^~^~_
        \) /  o o  \ (/
          '_   -   _'
          / '-----' \"#;

const MOOSE: &str = r#"
  \
   \   \_\_    _/_/
    \      \__/
           (oo)\_______
           (__)\       )\/\
               ||----w |
               ||     ||"#;

impl fmt::Display for CowShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            CowShape::Cow => COW,
            CowShape::Clippy => CLIPPY,
            CowShape::Ferris => FERRIS,
            CowShape::Moose => MOOSE,
        };
        f.write_str(display)
    }
}

impl FromStr for CowShape {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cow" => Ok(CowShape::Cow),
            "clippy" => Ok(CowShape::Clippy),
            "ferris" => Ok(CowShape::Ferris),
            "moose" => Ok(CowShape::Moose),
            _ => Err("Unknown Value"),
        }
    }
}
