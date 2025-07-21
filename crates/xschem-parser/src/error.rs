//! Parser errors.
use std::fmt::{self, Display};

use colored::Colorize;
use derive_more::From;
use nom::error::{ContextError, ErrorKind as NomErrorKind, FromExternalError, ParseError};

use crate::{FileSpan, Span};

#[derive(Clone, Debug, Eq, From, PartialEq)]
pub enum ErrorKind {
    /// Indicates which character was expected by the `char` function
    Char(char),
    /// Error kind given by various nom parsers
    Nom(NomErrorKind),
}

/// Input with an error.
#[derive(Clone, Debug, Eq, From, PartialEq)]
pub struct ErrorInput<I> {
    pub input: I,
    pub kind: ErrorKind,
}

/// Input with context.
#[derive(Clone, Debug, Eq, From, PartialEq)]
pub struct InputContext<'a, I> {
    pub input: I,
    pub name: &'a str,
}

#[derive(Clone, Debug, Eq, From, PartialEq)]
pub struct Error<I> {
    pub err: ErrorInput<I>,
    pub context: Vec<InputContext<'static, I>>,
}

impl std::error::Error for Error<&str> {}
impl std::error::Error for Error<Span<'_>> {}
impl std::error::Error for Error<FileSpan<'_, '_>> {}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        Self {
            err: ErrorInput {
                input,
                kind: kind.into(),
            },
            context: Vec::default(),
        }
    }

    fn append(_input: I, _kind: NomErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Self {
            err: ErrorInput {
                input,
                kind: c.into(),
            },
            context: Vec::default(),
        }
    }
}

impl<I> ContextError<I> for Error<I> {
    fn add_context(input: I, name: &'static str, mut other: Self) -> Self {
        other.context.push(InputContext { input, name });
        other
    }
}

impl<I, E> FromExternalError<I, E> for Error<I> {
    fn from_external_error(input: I, kind: NomErrorKind, _e: E) -> Self {
        Self {
            err: ErrorInput {
                input,
                kind: kind.into(),
            },
            context: Vec::default(),
        }
    }
}

macro_rules! format_line {
    ($input:expr) => {
        format_args!(
            "{space:width$}{ptr}:{line_number}:{column_number}\n\
             {space:width$}{gutter}\n\
             {line_number:>width$}{gutter} {line}\n\
             {space:width$}{gutter}{space:column_number$}{column}\n\
             {space:width$}{gutter}",
            space = ' ',
            ptr = "--> ".blue().bold(),
            gutter = " |".blue(),
            line_number = $input.location_line(),
            column_number = $input.get_utf8_column(),
            width = usize::try_from($input.location_line().ilog10() + 1).unwrap_or(6) + 1,
            line = std::str::from_utf8($input.get_line_beginning()).unwrap_or("<invalid UTF-8>"),
            column = "^".red().bold(),
        )
    };
}
macro_rules! format_file_line {
    ($input:expr, $path:expr $(,)?) => {
        format_args!(
            "{space:width$}{ptr}{path}:{line_number}:{column_number}\n\
             {space:width$}{gutter}\n\
             {line_number:>width$}{gutter} {line}\n\
             {space:width$}{gutter}{space:column_number$}{column}\n\
             {space:width$}{gutter}",
            space = ' ',
            ptr = "--> ".blue().bold(),
            gutter = " |".blue(),
            path = $path.display(),
            line_number = $input.location_line(),
            column_number = $input.get_utf8_column(),
            width = usize::try_from($input.location_line().ilog10() + 1).unwrap_or(6) + 1,
            line = std::str::from_utf8($input.get_line_beginning()).unwrap_or("<invalid UTF-8>"),
            column = "^".red().bold(),
        )
    };
}

macro_rules! format_error {
    ($desc:expr) => {
        format_args!(
            "{error}: {desc}",
            error = "error".red().bold(),
            desc = format!("{}", $desc).bold(),
        )
    };
}
macro_rules! format_error_line {
    ($input:expr, $desc:expr $(,)?) => {
        format_args!(
            "{error}\n{line}",
            error = format_error!($desc),
            line = format_line!($input),
        )
    };
}
macro_rules! format_error_file_line {
    ($input:expr, $desc:expr $(,)?) => {
        format_args!(
            "{error}\n{line}",
            error = format_error!($desc),
            line = format_file_line!($input, $input.extra),
        )
    };
}

macro_rules! format_context {
    ($context:expr) => {
        format_args!(
            "{context_in} {context}",
            context_in = "in".blue().bold(),
            context = $context.bold(),
        )
    };
}
macro_rules! format_context_line {
    ($input:expr, $context:expr $(,)?) => {
        format_args!(
            "{context}\n{line}",
            context = format_context!($context),
            line = format_line!($input),
        )
    };
}
macro_rules! format_context_file_line {
    ($input:expr, $context:expr $(,)?) => {
        format_args!(
            "{context}\n{line}",
            context = format_context!($context),
            line = format_file_line!($input, $input.extra),
        )
    };
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Char(expected) => write!(f, "expected '{expected}'"),
            ErrorKind::Nom(nom_err) => write!(f, "{}", nom_err.description()),
        }
    }
}

impl Display for ErrorInput<&str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_error!(format_args!("{}", self.kind)))
    }
}

impl Display for ErrorInput<Span<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_error_line!(self.input, self.kind))
    }
}

impl Display for ErrorInput<FileSpan<'_, '_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_error_file_line!(self.input, self.kind))
    }
}

impl Display for InputContext<'_, &str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_context!(self.name))
    }
}

impl Display for InputContext<'_, Span<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_context_line!(self.input, self.name))
    }
}

impl Display for InputContext<'_, FileSpan<'_, '_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_context_file_line!(self.input, self.name))
    }
}

impl<I> Display for Error<I>
where
    ErrorInput<I>: Display,
    InputContext<'static, I>: Display,
{
    /// Write human readable error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)?;

        self.context
            .iter()
            .try_for_each(|context| write!(f, "\n{context}"))
    }
}
