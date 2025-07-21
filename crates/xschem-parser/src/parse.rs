//! Parser combinator functions.
use std::collections::HashMap;
use std::hash::Hash;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{
    char, multispace0, multispace1, none_of, one_of, space1, u64, usize,
};
use nom::combinator::{consumed, cut, eof, opt, value as nom_value};
use nom::error::{ContextError, ErrorKind, ParseError, context};
use nom::multi::{fold_many0, length_count};
use nom::number::complete::recognize_float;
use nom::sequence::{preceded, separated_pair, terminated};
use nom::{AsChar, Compare, Err, Finish, IResult, Input, Offset, ParseTo, Parser};

use crate::token::{
    Arc, Component, Coordinate, Embedding, FiniteDouble, Flip, Line, Object, Polygon, Property,
    Rectangle, Rotation, Schematic, Size, SpiceProperty, SymbolProperty, TedaXProperty, Text, Vec2,
    VerilogProperty, Version, VhdlProperty, Wire,
};

/// Reserved escapable characters in property strings.
pub const ESCAPED_CHARS: &str = r"\{}";
/// Reserved escapable characters in attribute values.
pub const ESCAPED_VALUE_CHARS: &str = r#"\""#;
/// Escape character in property strings.
pub const ESCAPE_CHAR: char = '\\';

pub(crate) fn escaped0<'a, I, Error, F, G>(
    mut normal: F,
    control_char: char,
    mut escapable: G,
) -> impl FnMut(I) -> IResult<I, I, Error>
where
    I: Input + Offset + 'a,
    <I as Input>::Item: AsChar,
    F: Parser<I, Error = Error>,
    G: Parser<I, Error = Error>,
    Error: ParseError<I>,
{
    move |input: I| {
        let mut i = input.clone();
        let mut consumed_nothing = false;

        while i.input_len() > 0 {
            let current_len = i.input_len();

            match (normal.parse(i.clone()), consumed_nothing) {
                (Ok((i2, _)), false) => {
                    if i2.input_len() == 0 {
                        return Ok((input.take_from(input.input_len()), input));
                    }
                    if i2.input_len() == current_len {
                        consumed_nothing = true;
                    }
                    i = i2;
                }
                (Ok(..), true) | (Err(Err::Error(_)), _) => {
                    let next_char = i
                        .iter_elements()
                        .next()
                        .ok_or_else(|| {
                            Err::Error(Error::from_error_kind(i.clone(), ErrorKind::Escaped))
                        })?
                        .as_char();
                    if next_char == control_char {
                        let next = control_char.len_utf8();
                        if next >= i.input_len() {
                            return Err(Err::Error(Error::from_error_kind(
                                input,
                                ErrorKind::Escaped,
                            )));
                        }
                        match escapable.parse(i.take_from(next)) {
                            Ok((i2, _)) => {
                                if i2.input_len() == 0 {
                                    return Ok((input.take_from(input.input_len()), input));
                                }
                                consumed_nothing = false;
                                i = i2;
                            }
                            Err(_) => {
                                return Err(Err::Error(Error::from_error_kind(
                                    i,
                                    ErrorKind::Escaped,
                                )));
                            }
                        }
                    } else {
                        let index = input.offset(&i);
                        return Ok(input.take_split(index));
                    }
                }
                (Err(e), _) => {
                    return Err(e);
                }
            }
        }

        Ok((input.take_from(input.input_len()), input))
    }
}

pub(crate) fn try_skip<'a, I, O, E, F>(
    mut parser: F,
) -> impl Parser<I, Output = Option<O>, Error = E>
where
    I: Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
    F: Parser<I, Output = O, Error = (I, ErrorKind)> + 'a,
{
    move |input: I| match parser.parse(input) {
        Ok((rest, kv)) => Ok((rest, Some(kv))),
        Err(Err::Error((rest, _))) => Ok((rest, None)),
        Err(Err::Failure((input, kind))) => Err(Err::Failure(E::from_error_kind(input, kind))),
        Err(Err::Incomplete(e)) => Err(Err::Incomplete(e)),
    }
}

fn is_key_char<C: AsChar>(c: C) -> bool {
    c.is_alphanum() || c.as_char() == '_'
}

pub(crate) fn key<'a, I, E>(input: I) -> IResult<I, I, E>
where
    I: Offset + Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context("key", take_while1(is_key_char)).parse(input)
}

fn is_value_char<C: AsChar>(c: C) -> bool {
    c.is_alphanum() || c.as_char().is_ascii_punctuation()
}

pub(crate) fn value<'a, I, E>(input: I) -> IResult<I, I, E>
where
    I: Offset + Input + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "value",
        alt((
            preceded(
                char('"'),
                cut(terminated(
                    escaped0(
                        none_of(ESCAPED_VALUE_CHARS),
                        ESCAPE_CHAR,
                        alt((tag(r#"\""#), tag(r"\"), tag(r"{"), tag(r"}"))),
                    ),
                    char('"'),
                )),
            ),
            take_while1(is_value_char),
        )),
    )
    .parse(input)
}

pub(crate) fn key_value<'a, I, E>(input: I) -> IResult<I, (I, I), E>
where
    I: Offset + Input + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context("key_value", separated_pair(key, char('='), value)).parse(input)
}

pub(crate) fn attributes<'a, I, E>(mut input: I) -> IResult<I, HashMap<I, I>, E>
where
    I: Eq + Hash + Offset + Input + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    let mut attrs = HashMap::new();

    while input.input_len() > 0 {
        input = match preceded(take_while(|c| !is_key_char(c)), try_skip(key_value)).parse(input) {
            Ok((rest, Some((k, v)))) => {
                attrs.insert(k, v);
                rest
            }
            Ok((rest, None)) => rest,
            Err(e) => return Err(e),
        };
    }

    Ok((input, attrs))
}

pub(crate) fn brace_enclosed<'a, I, O, P, E>(parser: P) -> impl Parser<I, Output = O, Error = E>
where
    I: Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
    P: Parser<I, Output = O, Error = E>,
{
    preceded(char('{'), cut(terminated(parser, char('}'))))
}

pub(crate) fn property_string<'a, I, E>(input: I) -> IResult<I, I, E>
where
    I: Input + Offset + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    escaped0(none_of(ESCAPED_CHARS), ESCAPE_CHAR, one_of(ESCAPED_CHARS)).parse(input)
}

pub(crate) fn property<'a, I, E>(input: I) -> IResult<I, Property<I>, E>
where
    I: Eq + Hash + Input + Offset + for<'s> nom::Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    brace_enclosed(context("property", property_string))
        .and_then(consumed(attributes))
        .map(|(prop, attrs)| Property { prop, attrs })
        .parse(input)
}

pub(crate) fn text<'a, I, E>(input: I) -> IResult<I, I, E>
where
    I: Input + Offset + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    brace_enclosed(context("text", property_string)).parse(input)
}

pub(crate) fn reference<'a, I, E>(input: I) -> IResult<I, I, E>
where
    I: Input + Offset + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    brace_enclosed(context("reference", property_string)).parse(input)
}

pub(crate) fn object<'a, I, O, P, E>(
    name: &'static str,
    tag: char,
    parser: P,
) -> impl Parser<I, Output = O, Error = E>
where
    I: Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
    P: Parser<I, Output = O, Error = E>,
{
    context(name, preceded(char(tag), cut(parser)))
}

pub(crate) fn layer<'a, I, E>(input: I) -> IResult<I, u64, E>
where
    I: Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    u64(input)
}

pub(crate) fn finite_double<'a, I, E>(input: I) -> IResult<I, FiniteDouble, E>
where
    I: Input + Offset + ParseTo<f64> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    let (i, s) = recognize_float(input)?;
    match s.parse_to() {
        // Safe to unwrap here cause recognize_float should only recognize
        // finite numbers.
        Some(f) => Ok((i, f.try_into().unwrap())),
        None => Err(Err::Error(E::from_error_kind(i, ErrorKind::Float))),
    }
}

pub(crate) fn vec2<'a, I, E>(input: I) -> IResult<I, Vec2, E>
where
    I: Input + Offset + ParseTo<f64> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    Parser::into(separated_pair(finite_double, multispace1, finite_double)).parse(input)
}

pub(crate) fn coordinate<'a, I, E>(input: I) -> IResult<I, Coordinate, E>
where
    I: Input + Offset + ParseTo<f64> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context("coordinate", vec2).parse(input)
}

pub(crate) fn size<'a, I, E>(input: I) -> IResult<I, Size, E>
where
    I: Input + Offset + ParseTo<f64> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context("size", vec2).parse(input)
}

pub(crate) fn rotation<'a, I, E>(input: I) -> IResult<I, Rotation, E>
where
    I: Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "rotation",
        alt((
            nom_value(Rotation::Zero, char('0')),
            nom_value(Rotation::One, char('1')),
            nom_value(Rotation::Two, char('2')),
            nom_value(Rotation::Three, char('3')),
        )),
    )
    .parse(input)
}

pub(crate) fn flip<'a, I, E>(input: I) -> IResult<I, Flip, E>
where
    I: Input + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "flip",
        alt((
            nom_value(Flip::Unflipped, char('0')),
            nom_value(Flip::Flipped, char('1')),
        )),
    )
    .parse(input)
}

pub(crate) fn embedding<'a, I, E>(input: I) -> IResult<I, Embedding<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> nom::Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "embedded symbol",
        '[',
        terminated(
            preceded(multispace1, Parser::into(schematic)),
            preceded(multispace1, char(']')),
        ),
    )
    .parse(input)
}

pub(crate) fn version_object<'a, I, E>(input: I) -> IResult<I, Version<I>, E>
where
    I: Eq + Hash + Input + Offset + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object("version", 'v', preceded(multispace1, property))
        .map(Version)
        .parse(input)
}

pub(crate) fn property_object<'a, I, E>(
    tag: char,
) -> impl Parser<I, Output = Property<I>, Error = E>
where
    I: Eq + Hash + Input + Offset + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object("global property", tag, preceded(multispace1, property))
}

pub(crate) fn arc_object<'a, I, E>(input: I) -> IResult<I, Arc<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "arc",
        'A',
        (
            preceded(multispace1, layer),
            preceded(multispace1, coordinate),
            preceded(multispace1, finite_double),
            preceded(multispace1, finite_double),
            preceded(multispace1, finite_double),
            preceded(multispace1, property),
        ),
    )
    .map(
        |(layer, center, radius, start_angle, sweep_angle, property)| Arc {
            layer,
            center,
            radius,
            start_angle,
            sweep_angle,
            property,
        },
    )
    .parse(input)
}

pub(crate) fn component_instance<'a, I, E>(input: I) -> IResult<I, Component<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "component",
        'C',
        (
            preceded(multispace1, reference),
            preceded(multispace1, coordinate),
            preceded(multispace1, rotation),
            preceded(multispace1, flip),
            preceded(multispace1, property),
            opt(preceded(multispace1, embedding)),
        ),
    )
    .map(
        |(reference, position, rotation, flip, property, embedding)| Component {
            reference,
            position,
            rotation,
            flip,
            property,
            embedding,
        },
    )
    .parse(input)
}

pub(crate) fn line_object<'a, I, E>(input: I) -> IResult<I, Line<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "line",
        'L',
        (
            preceded(multispace1, layer),
            preceded(multispace1, coordinate),
            preceded(multispace1, coordinate),
            preceded(multispace1, property),
        ),
    )
    .map(|(layer, start, end, property)| Line {
        layer,
        start,
        end,
        property,
    })
    .parse(input)
}

pub(crate) fn polygon_object<'a, I, E>(input: I) -> IResult<I, Polygon<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "polygon",
        'P',
        (
            preceded(multispace1, layer),
            preceded(
                multispace1,
                length_count(usize, preceded(space1, coordinate)),
            ),
            preceded(multispace1, property),
        ),
    )
    .map(|(layer, points, property)| Polygon {
        layer,
        points: points.into(),
        property,
    })
    .parse(input)
}

pub(crate) fn rectangle_object<'a, I, E>(input: I) -> IResult<I, Rectangle<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "rectangle",
        'B',
        (
            preceded(multispace1, layer),
            preceded(multispace1, coordinate),
            preceded(multispace1, coordinate),
            preceded(multispace1, property),
        ),
    )
    .map(|(layer, start, end, property)| Rectangle {
        layer,
        start,
        end,
        property,
    })
    .parse(input)
}

pub(crate) fn text_object<'a, I, E>(input: I) -> IResult<I, Text<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "text",
        'T',
        (
            preceded(multispace1, text),
            preceded(multispace1, coordinate),
            preceded(multispace1, rotation),
            preceded(multispace1, flip),
            preceded(multispace1, size),
            preceded(multispace1, property),
        ),
    )
    .map(|(text, position, rotation, flip, size, property)| Text {
        text,
        position,
        rotation,
        flip,
        size,
        property,
    })
    .parse(input)
}

pub(crate) fn wire_object<'a, I, E>(input: I) -> IResult<I, Wire<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    object(
        "wire",
        'N',
        (
            preceded(multispace1, coordinate),
            preceded(multispace1, coordinate),
            preceded(multispace1, property),
        ),
    )
    .map(|(start, end, property)| Wire {
        start,
        end,
        property,
    })
    .parse(input)
}

pub(crate) fn any_object<'a, I, E>(input: I) -> IResult<I, Object<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    alt((
        Parser::into(Parser::into::<VhdlProperty<I>, E>(property_object('G'))),
        Parser::into(Parser::into::<SymbolProperty<I>, E>(property_object('K'))),
        Parser::into(Parser::into::<VerilogProperty<I>, E>(property_object('V'))),
        Parser::into(Parser::into::<SpiceProperty<I>, E>(property_object('S'))),
        Parser::into(Parser::into::<TedaXProperty<I>, E>(property_object('E'))),
        Parser::into(arc_object),
        Parser::into(component_instance),
        Parser::into(line_object),
        Parser::into(polygon_object),
        Parser::into(rectangle_object),
        Parser::into(text_object),
        Parser::into(wire_object),
    ))
    .parse(input)
}

/// Parse a [`Schematic`] from input.
pub fn schematic<'a, I, E>(input: I) -> IResult<I, Schematic<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    preceded(
        multispace0,
        version_object.flat_map(|version| {
            fold_many0(
                preceded(multispace1, any_object),
                move || Schematic::new(version.clone()),
                Schematic::add_object,
            )
        }),
    )
    .parse(input)
}

/// Parses a schematic to the end of the input.
pub fn schematic_full<'a, I, E>(input: I) -> Result<Schematic<I>, E>
where
    I: Eq + Hash + Input + Offset + ParseTo<f64> + for<'s> Compare<&'s str> + 'a,
    <I as Input>::Item: AsChar,
    E: ParseError<I> + ContextError<I>,
{
    terminated(schematic, preceded(multispace0, eof))
        .parse(input)
        .finish()
        .map(|r| r.1)
}
