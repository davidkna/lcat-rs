# lcat

lolcat in rust! With emoji support and color transformations in the Cubehelix and OkHSV color space.

## Usage

```text
Terminal rainbows.

Usage: lcat [OPTIONS] [File]...

Arguments:
  [File]...  [default: -]

Options:
  -C, --shift-col <SHIFT_COL>    How many degrees to shift text color hue for every column [default: 1.6]
  -R, --shift-row <SHIFT_ROW>    How many degrees to shift text color hue for every row [default: 3.2]
  -n, --shift-sign-no-random     Don't randomize sign of col and row shift values
  -H, --hue <HUE>                Sets initial hue of text color in degress [default: random]
  -s, --style <STYLE>            Rainbow mode [default: rainbow] [possible values: rainbow, sinebow, ok-hsv]
  -c, --color-mode <COLOR_MODE>  [possible values: true-color, ansi256]
  -S, --seed <SEED>              Sets seed [default: random]
  -i, --invert                   Invert background and foreground
  -h, --help                     Print help
```

## Screenshot

![a demo screenshot of lcat in action](../.github/screenshot.png)
