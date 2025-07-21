//! Parsed data structures.
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::vec::Vec;

use derive_more::{Deref, DerefMut, Display, From, Into, TryFrom};

use crate::error::Error;
use crate::{ByteSpan, Span, parse};

/// Xschem schematic (or symbol).
#[derive(Clone, Debug, Default)]
pub struct Schematic<I> {
    pub version: Version<I>,
    pub vhdl_property: Option<VhdlProperty<I>>,
    pub symbol_property: Option<SymbolProperty<I>>,
    pub verilog_property: Option<VerilogProperty<I>>,
    pub spice_property: Option<SpiceProperty<I>>,
    pub tedax_property: Option<TedaXProperty<I>>,
    pub texts: Objects<Text<I>>,
    pub lines: Objects<Line<I>>,
    pub rectangles: Objects<Rectangle<I>>,
    pub polygons: Objects<Polygon<I>>,
    pub arcs: Objects<Arc<I>>,
    pub wires: Objects<Wire<I>>,
    pub components: Objects<Component<I>>,
}

/// Xschem property string with parsed attributes.
#[derive(Clone, Debug, Default, Display)]
#[display("{{{prop}}}")]
pub struct Property<I> {
    /// Full property input.
    pub prop: I,
    /// Parsed attributes from `prop`.
    pub attrs: HashMap<I, I>,
}

/// Xschem schematic or symbol version specifiication.
#[derive(Clone, Debug, Default, Display)]
#[display("v {_0}")]
pub struct Version<I>(pub Property<I>);

#[derive(Clone, Debug, Default, Deref, Display, From)]
#[display("G {_0}")]
pub struct VhdlProperty<I>(pub Property<I>);

#[derive(Clone, Debug, Default, Deref, Display, From)]
#[display("K {_0}")]
pub struct SymbolProperty<I>(pub Property<I>);

#[derive(Clone, Debug, Default, Deref, Display, From)]
#[display("V {_0}")]
pub struct VerilogProperty<I>(pub Property<I>);

#[derive(Clone, Debug, Default, Deref, Display, From)]
#[display("S {_0}")]
pub struct SpiceProperty<I>(pub Property<I>);

#[derive(Clone, Debug, Default, Deref, Display, From)]
#[display("E {_0}")]
pub struct TedaXProperty<I>(pub Property<I>);

#[derive(Clone, Debug, From)]
#[from(forward)]
#[allow(clippy::large_enum_variant)]
pub enum Object<I> {
    SpiceProperty(SpiceProperty<I>),
    VerilogProperty(VerilogProperty<I>),
    VhdlProperty(VhdlProperty<I>),
    TedaXProperty(TedaXProperty<I>),
    SymbolProperty(SymbolProperty<I>),

    Arc(Arc<I>),
    Component(Component<I>),
    Line(Line<I>),
    Polygon(Polygon<I>),
    Rectangle(Rectangle<I>),
    Text(Text<I>),
    Wire(Wire<I>),
}

#[derive(Clone, Debug, Deref, DerefMut, From, Into, PartialEq)]
pub struct Objects<O>(pub Vec<O>);

/// Xschem arc object.
#[derive(Clone, Debug, Default, Display)]
#[display("A {layer} {center} {radius} {start_angle} {sweep_angle} {property}")]
pub struct Arc<I> {
    pub layer: u64,
    pub center: Coordinate,
    pub radius: FiniteDouble,
    pub start_angle: FiniteDouble,
    pub sweep_angle: FiniteDouble,
    pub property: Property<I>,
}

/// Xschem component instance.
#[derive(Clone, Debug, Default)]
pub struct Component<I> {
    pub reference: I,
    pub position: Coordinate,
    pub rotation: Rotation,
    pub flip: Flip,
    pub property: Property<I>,
    pub embedding: Option<Embedding<I>>,
}

/// Xschem line object.
#[derive(Clone, Debug, Default, Display)]
#[display("L {layer} {start} {end} {property}")]
pub struct Line<I> {
    pub layer: u64,
    pub start: Coordinate,
    pub end: Coordinate,
    pub property: Property<I>,
}

/// Xschem polygon object.
#[derive(Clone, Debug, Default, Display)]
#[display("P {layer} {npoints} {points} {property}", npoints = points.len())]
pub struct Polygon<I> {
    pub layer: u64,
    pub points: Coordinates,
    pub property: Property<I>,
}

/// Xschem rectangle object.
#[derive(Clone, Debug, Default, Display)]
#[display("B {layer} {start} {end} {property}")]
pub struct Rectangle<I> {
    pub layer: u64,
    pub start: Coordinate,
    pub end: Coordinate,
    pub property: Property<I>,
}

/// Xschem text object.
#[derive(Clone, Debug, Default, Display)]
#[display("T {{{text}}} {position} {rotation} {flip} {size} {property}")]
pub struct Text<I> {
    pub text: I,
    pub position: Coordinate,
    pub rotation: Rotation,
    pub flip: Flip,
    pub size: Size,
    pub property: Property<I>,
}

/// Xschem wire object.
#[derive(Clone, Debug, Default, Display)]
#[display("N {start} {end} {property}")]
pub struct Wire<I> {
    pub start: Coordinate,
    pub end: Coordinate,
    pub property: Property<I>,
}

#[derive(Clone, Debug, Default, Deref, Display, From, Into)]
#[display("[\n{_0}\n]")]
pub struct Embedding<I>(pub Schematic<I>);

/// Finite double precision type.
#[derive(Clone, Copy, Debug, Default, Deref, Display, Into, PartialEq, PartialOrd)]
pub struct FiniteDouble(f64);

#[derive(Clone, Copy, Debug, Default, Display, From, Into, PartialEq, PartialOrd)]
#[from((FiniteDouble, FiniteDouble))]
#[into((FiniteDouble, FiniteDouble))]
#[display("{x} {y}")]
pub struct Vec2 {
    pub x: FiniteDouble,
    pub y: FiniteDouble,
}

pub type Coordinate = Vec2;
pub type Size = Vec2;

#[derive(Clone, Debug, Default, Deref, DerefMut, From, Into, PartialEq)]
pub struct Coordinates(pub Vec<Coordinate>);

#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq, PartialOrd, Ord, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum Rotation {
    #[default]
    #[display("0")]
    Zero,
    #[display("1")]
    One,
    #[display("2")]
    Two,
    #[display("3")]
    Three,
}

#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq, PartialOrd, Ord, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum Flip {
    #[default]
    #[display("0")]
    Unflipped,
    #[display("1")]
    Flipped,
}

impl<'a, X: Clone + Default> TryFrom<&'a str> for Schematic<Span<'a, X>> {
    type Error = Error<Span<'a, X>>;

    /// Tries to parse a schematic from a `str`.
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(Span::new_extra(value, X::default()))
    }
}

impl<'a, X: Clone + Default> TryFrom<&'a [u8]> for Schematic<ByteSpan<'a, X>> {
    type Error = Error<ByteSpan<'a, X>>;

    /// Tries to parse a schematic from a slice.
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from(ByteSpan::new_extra(value, X::default()))
    }
}

impl<'a, X: Clone> TryFrom<Span<'a, X>> for Schematic<Span<'a, X>> {
    type Error = Error<Span<'a, X>>;

    /// Tries to parse a schematic from a spanned `str`.
    fn try_from(value: Span<'a, X>) -> Result<Self, Self::Error> {
        parse::schematic_full(value)
    }
}

impl<'a, X: Clone> TryFrom<ByteSpan<'a, X>> for Schematic<ByteSpan<'a, X>> {
    type Error = Error<ByteSpan<'a, X>>;

    /// Tries to parse a schematic from a spanned slice.
    fn try_from(value: ByteSpan<'a, X>) -> Result<Self, Self::Error> {
        parse::schematic_full(value)
    }
}

impl<I> fmt::Display for Schematic<I>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;
        if let Some(p) = &self.vhdl_property {
            write!(f, "\n{p}")?;
        }
        if let Some(p) = &self.symbol_property {
            write!(f, "\n{p}")?;
        }
        if let Some(p) = &self.verilog_property {
            write!(f, "\n{p}")?;
        }
        if let Some(p) = &self.spice_property {
            write!(f, "\n{p}")?;
        }
        if let Some(p) = &self.tedax_property {
            write!(f, "\n{p}")?;
        }
        if !self.texts.is_empty() {
            write!(f, "\n{}", self.texts)?;
        }
        if !self.lines.is_empty() {
            write!(f, "\n{}", self.lines)?;
        }
        if !self.rectangles.is_empty() {
            write!(f, "\n{}", self.rectangles)?;
        }
        if !self.polygons.is_empty() {
            write!(f, "\n{}", self.polygons)?;
        }
        if !self.arcs.is_empty() {
            write!(f, "\n{}", self.arcs)?;
        }
        if !self.wires.is_empty() {
            write!(f, "\n{}", self.wires)?;
        }
        if !self.components.is_empty() {
            write!(f, "\n{}", self.components)?;
        }
        Ok(())
    }
}

impl<'a> Schematic<Span<'a>> {
    /// Parses a string as a [`Schematic`].
    pub fn parse_str<I: AsRef<str> + ?Sized>(input: &'a I) -> Result<Self, Error<Span<'a>>> {
        Self::try_from(input.as_ref())
    }
}

impl<'a, X: Clone> Schematic<Span<'a, X>> {
    /// Parses a string as a [`Schematic`].
    pub fn parse_str_with_extra<I: AsRef<str> + ?Sized>(
        input: &'a I,
        extra: X,
    ) -> Result<Self, Error<Span<'a, X>>> {
        Self::try_from(Span::new_extra(input.as_ref(), extra))
    }
}

impl<'a> Schematic<ByteSpan<'a>> {
    /// Parses bytes as a [`Schematic`].
    pub fn parse_slice<I: AsRef<[u8]> + ?Sized>(input: &'a I) -> Result<Self, Error<ByteSpan<'a>>> {
        Self::try_from(input.as_ref())
    }
}

impl<'a, X: Clone> Schematic<ByteSpan<'a, X>> {
    /// Parses a string as a [`Schematic`].
    pub fn parse_slice_with_extra<I: AsRef<[u8]> + ?Sized>(
        input: &'a I,
        extra: X,
    ) -> Result<Self, Error<ByteSpan<'a, X>>> {
        Self::try_from(ByteSpan::new_extra(input.as_ref(), extra))
    }
}

impl<'a, X: Clone> Schematic<Span<'a, X>> {
    /// Parses a string span as a [`Schematic`].
    pub fn parse_span(input: Span<'a, X>) -> Result<Self, Error<Span<'a, X>>> {
        Self::try_from(input)
    }
}

impl<'a, X: Clone> Schematic<ByteSpan<'a, X>> {
    /// Parses a string span as a [`Schematic`].
    pub fn parse_span(input: ByteSpan<'a, X>) -> Result<Self, Error<ByteSpan<'a, X>>> {
        Self::try_from(input)
    }
}

impl<I: PartialEq> PartialEq for Schematic<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.vhdl_property == other.vhdl_property
            && self.symbol_property == other.symbol_property
            && self.verilog_property == other.verilog_property
            && self.spice_property == other.spice_property
            && self.tedax_property == other.tedax_property
            && self.texts == other.texts
            && self.lines == other.lines
            && self.rectangles == other.rectangles
            && self.polygons == other.polygons
            && self.arcs == other.arcs
            && self.wires == other.wires
            && self.components == other.components
    }
}

impl<I> Schematic<I> {
    pub fn new(version: Version<I>) -> Self {
        Self {
            version,
            vhdl_property: Option::default(),
            symbol_property: Option::default(),
            verilog_property: Option::default(),
            spice_property: Option::default(),
            tedax_property: Option::default(),
            texts: Objects::default(),
            lines: Objects::default(),
            rectangles: Objects::default(),
            polygons: Objects::default(),
            arcs: Objects::default(),
            wires: Objects::default(),
            components: Objects::default(),
        }
    }

    #[must_use]
    pub fn add_object(mut self, object: Object<I>) -> Self {
        match object {
            Object::VhdlProperty(p) => {
                self.vhdl_property.replace(p);
            }
            Object::SymbolProperty(p) => {
                self.symbol_property.replace(p);
            }
            Object::VerilogProperty(p) => {
                self.verilog_property.replace(p);
            }
            Object::SpiceProperty(p) => {
                self.spice_property.replace(p);
            }
            Object::TedaXProperty(p) => {
                self.tedax_property.replace(p);
            }
            Object::Arc(o) => {
                self.arcs.push(o);
            }
            Object::Component(o) => {
                self.components.push(o);
            }
            Object::Line(o) => {
                self.lines.push(o);
            }
            Object::Polygon(o) => {
                self.polygons.push(o);
            }
            Object::Rectangle(o) => {
                self.rectangles.push(o);
            }
            Object::Text(o) => {
                self.texts.push(o);
            }
            Object::Wire(o) => {
                self.wires.push(o);
            }
        }

        self
    }
}

impl<I: Eq + Hash + PartialEq> PartialEq for Property<I> {
    fn eq(&self, other: &Self) -> bool {
        self.prop == other.prop && self.attrs == other.attrs
    }
}

impl<I> PartialEq for Version<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> PartialEq for SpiceProperty<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> PartialEq for VerilogProperty<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> PartialEq for VhdlProperty<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> PartialEq for TedaXProperty<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> PartialEq for SymbolProperty<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> PartialEq for Arc<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer
            && self.center == other.center
            && self.radius == other.radius
            && self.start_angle == other.start_angle
            && self.sweep_angle == other.sweep_angle
            && self.property == other.property
    }
}

impl<I> fmt::Display for Component<I>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            reference,
            position,
            rotation,
            flip,
            property,
            embedding,
        } = self;
        write!(
            f,
            "C {{{reference}}} {position} {rotation} {flip} {property}"
        )?;
        if let Some(e) = embedding {
            write!(f, "\n{e}")?;
        }
        Ok(())
    }
}

impl<I: PartialEq> PartialEq for Component<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
            && self.position == other.position
            && self.rotation == other.rotation
            && self.flip == other.flip
            && self.property == other.property
            && self.embedding == other.embedding
    }
}

impl<I> PartialEq for Line<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer
            && self.start == other.start
            && self.end == other.end
            && self.property == other.property
    }
}

impl<I> PartialEq for Polygon<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer && self.points == other.points && self.property == other.property
    }
}

impl<I> PartialEq for Rectangle<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer
            && self.start == other.start
            && self.end == other.end
            && self.property == other.property
    }
}

impl<I: PartialEq> PartialEq for Text<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.position == other.position
            && self.rotation == other.rotation
            && self.flip == other.flip
            && self.size == other.size
            && self.property == other.property
    }
}

impl<I> PartialEq for Wire<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.property == other.property
    }
}

impl<I: PartialEq> PartialEq for Embedding<I>
where
    Property<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<O> Default for Objects<O> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<O: fmt::Display> fmt::Display for Objects<O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.iter().enumerate().try_for_each(
            |(i, o)| {
                if i == 0 { o.fmt(f) } else { write!(f, "\n{o}") }
            },
        )
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.iter().enumerate().try_for_each(
            |(i, c)| {
                if i == 0 { c.fmt(f) } else { write!(f, " {c}") }
            },
        )
    }
}

impl TryFrom<f64> for FiniteDouble {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err("value is not finite")
        }
    }
}

impl Eq for FiniteDouble {}

impl TryFrom<(f64, f64)> for Vec2 {
    type Error = <FiniteDouble as TryFrom<f64>>::Error;

    fn try_from(value: (f64, f64)) -> Result<Self, Self::Error> {
        let (x, y) = value;
        let x = x.try_into()?;
        let y = y.try_into()?;
        Ok(Self { x, y })
    }
}

impl<T> From<Vec<(T, T)>> for Coordinates
where
    Vec2: From<(T, T)>,
{
    fn from(value: Vec<(T, T)>) -> Self {
        Self(value.into_iter().map(Vec2::from).collect())
    }
}

impl FromIterator<Vec2> for Coordinates {
    fn from_iter<T: IntoIterator<Item = Vec2>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl TryFrom<Vec<(f64, f64)>> for Coordinates {
    type Error = <Vec2 as TryFrom<(f64, f64)>>::Error;

    fn try_from(value: Vec<(f64, f64)>) -> Result<Self, Self::Error> {
        value.into_iter().map(Vec2::try_from).collect()
    }
}

impl From<Rotation> for u8 {
    fn from(value: Rotation) -> Self {
        match value {
            Rotation::Zero => 0,
            Rotation::One => 1,
            Rotation::Two => 2,
            Rotation::Three => 3,
        }
    }
}

impl From<bool> for Flip {
    fn from(value: bool) -> Self {
        if value {
            Flip::Flipped
        } else {
            Flip::Unflipped
        }
    }
}

impl From<Flip> for bool {
    fn from(value: Flip) -> Self {
        match value {
            Flip::Unflipped => false,
            Flip::Flipped => true,
        }
    }
}
