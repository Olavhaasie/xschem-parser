# Xschem parser

[![LICENSE](https://img.shields.io/badge/license-MIT_OR_APACHE_2.0-blue.svg)](#license)
[![Build Status](https://github.com/Olavhaasie/xschem-parser/actions/workflows/check.yml/badge.svg)](https://github.com/Olavhaasie/xschem-parser/actions/workflows/check.yml)
[![crates.io Version](https://img.shields.io/crates/v/xschem-parser.svg)][crates.io]
[![docs.rs](https://img.shields.io/docsrs/xschem-parser)][docs.rs]
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.85.0+-lightgray.svg)](#rust-version-requirements-msrv)

[Xschem] schematic and symbol parser implemented with [nom] parser combinators.

## Installation

`xschem-parser` is available on [crates.io] and can be included in your Cargo
enabled project like this:

```toml
[dependencies]
xschem-parser = "0.1"
```

Specify the `no-color` feature to disable colored display of errors:

```toml
[dependencies]
xschem-parser = { version = "0.1", features = ["no-color"] }
```

### CLI

`xschem-parser-cli` is a simple command line parser that is also available on [crates.io](crates.io/crates/xschem-parser-cli).

## Rust version requirements (MSRV)

`xschem-parser` supports **Rustc version 1.85 or greater**.

## License

xschem-parser is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE] and [LICENSE-MIT] for details.

[crates.io]: https://crates.io/crates/xschem-parser
[docs.rs]: https://docs.rs/xschem-parser
[Xschem]: https://xschem.sourceforge.io/stefan/index.html
[nom]: https://github.com/rust-bakery/nom
[LICENSE-APACHE]: https://github.com/Olavhaasie/xschem-parser/blob/main/LICENSE-APACHE
[LICENSE-MIT]: https://github.com/Olavhaasie/xschem-parser/blob/main/LICENSE-MIT
