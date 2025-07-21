# Xschem parser

[![LICENSE](https://img.shields.io/badge/license-MIT_OR_APACHE_2.0-blue.svg)](./LICENSE-MIT)
[![Build Status](https://github.com/Olavhaasie/xschem-parser/actions/workflows/check.yml/badge.svg)](https://github.com/Olavhaasie/xschem-parser/actions/workflows/ci.yml)
[![crates.io Version](https://img.shields.io/crates/v/xschem-parser.svg)](https://crates.io/crates/xschem-parser)
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

`xschem-parser-cli` is a simple command line parser. See the [README] for more
info.

## Rust version requirements (MSRV)

`xschem-parser` supports **Rustc version 1.85 or greater**.

## License

xschem-parser is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

[Xschem]: https://xschem.sourceforge.io/stefan/index.html
[nom]: https://github.com/rust-bakery/nom
[crates.io]: https://crates.io/crates/xschem-parser
[README]: ./crates/xschem-parser-cli/README.md
