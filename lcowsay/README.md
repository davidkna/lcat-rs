# lcowsay-rs

```
 ___________
< Hello 🌍! >
 -----------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```

Cowsay, but combined with lolcat.

## Flags

```
USAGE:
    lcowsay [FLAGS] [OPTIONS] [TEXT]...

FLAGS:
        --help                    Prints help information
        --lolcat
    -n, --shift-sign-no-random    Don't randomize sign of col and row shift value
    -V, --version                 Prints version information

OPTIONS:
    -c, --chroma <chroma>            Sets initial chroma as defined by CIE L*C*h Color Scale [default: 128]
    -h, --hue <hue>                  Sets initial hue as defined by CIE L*C*h Color Scale [default: random]
    -l, --luminance <luminance>      Sets initial luminance as defined by CIE L*C*h Color Scale [default: 50]
    -W, --max-length <max-length>     [default: 40]
    -f, --cow-shape <shape>           [default: cow]  [possible values: cow, clippy, ferris, moose]
    -C, --shift-col <shift-col>      How much the hue of the color gets shifted every column [default: 1.6]
    -R, --shift-row <shift-row>      How much the hue of the color gets shifted every row [default: 3.2]

ARGS:
    <TEXT>...     [default: ]
```
