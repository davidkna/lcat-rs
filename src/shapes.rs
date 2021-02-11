use clap::ArgEnum;
use std::fmt;

#[derive(Debug)]
#[cfg_attr(feature = "clap", derive(ArgEnum))]
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
