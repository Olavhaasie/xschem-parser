# `xschem-parser-cli`

Simple Xschem file parser program using [xschem-parser].

## Installation

Install the crate from [crates.io]:

```sh
cargo install xschem-parser-cli
```

## Usage

Pass filenames of schematics or symbols to parse:

```txt
xschem-parser-cli [FILES...]
```

To run on a list of files use [`fd`][fd]:

```sh
fd "\.(sch|sym)$" . -X xschem-parser-cli
```

## License

xschem-parser is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

[xschem-parser]: https://crates.io/crates/xschem-parser
[crates.io]: https://crates.io/crates/xschem-parser-cli
[fd]: https://github.com/sharkdp/fd
