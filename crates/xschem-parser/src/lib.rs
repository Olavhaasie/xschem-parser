//! [Xschem] schematic and symbol parser.
//!
//! This library supports up to Xschem file version 1.2.
//! See Xschem [developer info] for more information on the file format.
//!
//! # Usage
//!
//! Use [`from_str`] or [`from_slice`] to parse a [`token::Schematic`] from a
//! string or byte slice. The parser is zero-copy so the resulting data
//! structure contains references to the input.
//!
//! The parse error result [`Error`] implements [`std::fmt::Display`] to convert
//! the error to a nice human readable format.
//!
//! # Examples
//!
//! ## Parse from string
//!
//! ```
//! use nom::Input;
//! use xschem_parser::Span;
//! use xschem_parser::token::{Flip, Objects, Property, Rotation, Schematic, Text, Version};
//!
//! let input = "\
//! v {xschem version=3.4.5 file_version=1.2}
//! K {type=regulator}
//! T {@name} -17.5 -15 0 0 0.2 0.2 {}
//! ";
//!
//! // Get a span so we can reference to locations in the input string.
//! // The parsed schematic contains references with column and line in
//! // the input string.
//! let span = Span::new(input);
//!
//! let expected = Schematic {
//!     version: Version(Property {
//!         prop: span.take_from(3).take(37),
//!         attrs: [
//!             (span.take_from(10).take(7), span.take_from(18).take(5)),
//!             (span.take_from(24).take(12), span.take_from(37).take(3)),
//!         ].into(),
//!     }),
//!     spice_property: None,
//!     verilog_property: None,
//!     vhdl_property: None,
//!     tedax_property: None,
//!     symbol_property: Some(Property {
//!         prop: span.take_from(45).take(14).into(),
//!         attrs: [
//!             (span.take_from(45).take(4), span.take_from(50).take(9)),
//!         ].into(),
//!     }.into()),
//!     texts: vec![Text {
//!         text: span.take_from(64).take(5),
//!         position: (-17.5, -15.0).try_into().unwrap(),
//!         rotation: Rotation::Zero,
//!         flip: Flip::Unflipped,
//!         size: (0.2, 0.2).try_into().unwrap(),
//!         property: Property {
//!             prop: span.take_from(94).take(0),
//!             attrs: [].into(),
//!         },
//!     }].into(),
//!     lines: Objects::default(),
//!     rectangles: Objects::default(),
//!     polygons: Objects::default(),
//!     arcs: Objects::default(),
//!     wires: Objects::default(),
//!     components: Objects::default(),
//! };
//!
//! let result = xschem_parser::from_str(input);
//!
//! assert_eq!(result, Ok(expected));
//! ```
//!
//! ## Parse from invalid string
//!
//! ```
//! // invalid input, wrong brackets
//! let input = "v []";
//!
//! let expected = "\
//! error: expected '{'
//!   --> :1:3
//!    |
//!  1 | v []
//!    |   ^
//!    |
//! in version
//!   --> :1:1
//!    |
//!  1 | v []
//!    | ^
//!    |";
//!
//! let result = xschem_parser::from_str(input);
//!
//! assert!(result.is_err());
//! assert_eq!(result.unwrap_err().to_string(), expected);
//! ```
//!
//! ## Parse from file
//!
//! Since a parsed schematic contains references to the input, this library
//! cannot provide an implementation of parsing from file, since that would
//! require copying the contents of the file contents to make the lifetimes work
//! out.
//!
//! ```no_run
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let path = Path::new("test.sch");
//! let contents = std::fs::read_to_string(path)?;
//! match xschem_parser::from_str_file(&contents, path) {
//!     Ok(schematic) => println!("{schematic}"),
//!     Err(e) => eprintln!("{e}"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [Xschem]: https://xschem.sourceforge.io/stefan/index.html
//! [developer info]: https://xschem.sourceforge.io/stefan/xschem_man/developer_info.html

use std::path::Path;

use nom_locate::LocatedSpan;

use crate::error::Error;
use crate::token::Schematic;

pub mod error;
pub mod parse;
pub mod token;

#[cfg(test)]
mod test;

/// String reference with location.
pub type Span<'a, X = ()> = LocatedSpan<&'a str, X>;
/// String reference with location in file.
pub type FileSpan<'a, 'b> = Span<'a, &'b Path>;
/// Bytes reference with location.
pub type ByteSpan<'a, X = ()> = LocatedSpan<&'a [u8], X>;
/// Bytes reference with location in file.
pub type ByteFileSpan<'a, 'b> = ByteSpan<'a, &'b Path>;

/// Parse a [`Schematic`] from a [`str`].
pub fn from_str(s: &str) -> Result<Schematic<Span<'_>>, Error<Span<'_>>> {
    Schematic::parse_str(s)
}

/// Parse a [`Schematic`] from a byte slice.
pub fn from_slice(s: &[u8]) -> Result<Schematic<ByteSpan<'_>>, Error<ByteSpan<'_>>> {
    Schematic::parse_slice(s)
}

/// Parse a [`Schematic`] from a [`str`] with [`Path`] info.
pub fn from_str_file<'a, 'b>(
    s: &'a str,
    path: &'b Path,
) -> Result<Schematic<FileSpan<'a, 'b>>, Error<FileSpan<'a, 'b>>> {
    Schematic::parse_str_with_extra(s, path)
}

/// Parse a [`Schematic`] from a byte slice with [`Path`] info.
pub fn from_slice_file<'a, 'b>(
    s: &'a [u8],
    path: &'b Path,
) -> Result<Schematic<ByteFileSpan<'a, 'b>>, Error<ByteFileSpan<'a, 'b>>> {
    Schematic::parse_slice_with_extra(s, path)
}
