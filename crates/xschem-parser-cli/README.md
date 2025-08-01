# `xschem-parser-cli`

[![LICENSE](https://img.shields.io/badge/license-MIT_OR_APACHE_2.0-blue.svg)](#license)
[![Build Status](https://github.com/Olavhaasie/xschem-parser/actions/workflows/check.yml/badge.svg)](https://github.com/Olavhaasie/xschem-parser/actions/workflows/check.yml)
[![crates.io Version](https://img.shields.io/crates/v/xschem-parser-cli.svg)][crates.io]

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

To run on a list of files using [fd]:

```sh
fd "\.(sch|sym)$" . -X xschem-parser-cli
```

## License

xschem-parser-cli is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE] and [LICENSE-MIT] for details.

[xschem-parser]: https://crates.io/crates/xschem-parser
[crates.io]: https://crates.io/crates/xschem-parser-cli
[fd]: https://github.com/sharkdp/fd
[LICENSE-APACHE]: https://github.com/Olavhaasie/xschem-parser/blob/main/LICENSE-APACHE
[LICENSE-MIT]: https://github.com/Olavhaasie/xschem-parser/blob/main/LICENSE-MIT
