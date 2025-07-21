use nom::character::complete::{alpha1, digit1};
use nom::error::ErrorKind;
use nom::sequence::preceded;
use nom::{Err, Parser};

use crate::parse::{
    arc_object, attributes, component_instance, key_value, line_object, polygon_object, property,
    rectangle_object, schematic_full, text_object, try_skip, version_object, wire_object,
};
use crate::token::{
    Arc, Component, Line, Polygon, Property, Rectangle, Rotation, Text, Version, Wire,
};

#[test]
fn parse_try_skip() {
    assert_eq!(
        try_skip::<&str, &str, (&str, ErrorKind), _>(digit1).parse("abc123"),
        Ok(("abc123", None))
    );
    assert_eq!(
        try_skip::<&str, &str, (&str, ErrorKind), _>(digit1).parse("123abc"),
        Ok(("abc", Some("123")))
    );
    assert_eq!(
        try_skip::<&str, &str, (&str, ErrorKind), _>(preceded(digit1, alpha1)).parse("123abc"),
        Ok(("", Some("abc")))
    );
    assert_eq!(
        try_skip::<&str, &str, (&str, ErrorKind), _>(preceded(digit1, alpha1)).parse("123   "),
        Ok(("   ", None))
    );
    assert_eq!(
        try_skip::<&str, (&str, &str), (&str, ErrorKind), _>(key_value).parse("abc="),
        Ok(("", None))
    );
    assert_eq!(
        try_skip::<&str, (&str, &str), (&str, ErrorKind), _>(key_value).parse("test key=val"),
        Ok((" key=val", None))
    );
}

#[test]
fn parse_key_value() {
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>("a1A_9Z=!#$zcv_`~^)"),
        Ok(("", ("a1A_9Z", "!#$zcv_`~^)")))
    );
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>(r#"key="""#),
        Ok(("", ("key", "")))
    );
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>("=val"),
        Err(Err::Error(("=val", ErrorKind::TakeWhile1)))
    );
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>(r"key=\{val\}"),
        Ok(("", ("key", r"\{val\}")))
    );
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>(r#"key="\{val\}""#),
        Ok(("", ("key", r"\{val\}")))
    );
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>(r#"key="\\"val\\"""#),
        Ok(("", ("key", r#"\\"val\\""#)))
    );
    assert_eq!(
        key_value::<&str, (&str, ErrorKind)>(r#"key="\\val""#),
        Ok(("", ("key", r"\\val")))
    );
}

#[test]
fn parse_attributes() {
    assert_eq!(
        attributes::<&str, (&str, ErrorKind)>("key=val"),
        Ok(("", [("key", "val")].into()))
    );
    assert_eq!(
        attributes::<&str, (&str, ErrorKind)>("key=val k=v"),
        Ok(("", [("key", "val"), ("k", "v")].into()))
    );
    assert_eq!(
        attributes::<&str, (&str, ErrorKind)>("nokey k=v test"),
        Ok(("", [("k", "v")].into()))
    );
}

#[test]
fn parse_property() {
    assert_eq!(
        property::<&str, (&str, ErrorKind)>("{}"),
        Ok(("", Property::default()))
    );
    assert_eq!(
        property::<&str, (&str, ErrorKind)>("{a b c}"),
        Ok((
            "",
            Property {
                prop: "a b c",
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        property::<&str, (&str, ErrorKind)>(r"{\\\}}"),
        Ok((
            "",
            Property {
                prop: r"\\\}",
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        property::<&str, (&str, ErrorKind)>("{\t\n \\{\\}}"),
        Ok((
            "",
            Property {
                prop: "\t\n \\{\\}",
                ..Default::default()
            }
        ))
    );
}

#[test]
fn parse_version_object() {
    assert_eq!(
        version_object::<&str, (&str, ErrorKind)>("v {xschem version=3.4.0 file_version=1.0}"),
        Ok((
            "",
            Version(Property {
                prop: "xschem version=3.4.0 file_version=1.0",
                attrs: [("version", "3.4.0"), ("file_version", "1.0")].into()
            })
        ))
    );
    assert_eq!(
        version_object::<&str, (&str, ErrorKind)>(
            "v {xschem version=3.4.5 file_version=1.2\n* copyright info}"
        ),
        Ok((
            "",
            Version(Property {
                prop: "xschem version=3.4.5 file_version=1.2\n* copyright info",
                attrs: [("version", "3.4.5"), ("file_version", "1.2")].into()
            })
        ))
    );
}

#[test]
fn parse_text_object() {
    assert_eq!(
        text_object::<&str, (&str, ErrorKind)>(
            "T {3 of 4 NANDS of a 74ls00} 500 -580 0 0 0.4 0.4 {font=Monospace layer=4}",
        ),
        Ok((
            "",
            Text {
                text: "3 of 4 NANDS of a 74ls00",
                position: (500.0, -580.0).try_into().unwrap(),
                rotation: Rotation::Zero,
                flip: false.into(),
                size: (0.4, 0.4).try_into().unwrap(),
                property: Property {
                    prop: "font=Monospace layer=4",
                    attrs: [("font", "Monospace"), ("layer", "4")].into()
                },
            }
        )),
    );
    assert_eq!(
        text_object::<&str, (&str, ErrorKind)>("T {1\n2\n\n3} 1.1 4.04 3 1 1.0 2.0 {}",),
        Ok((
            "",
            Text {
                text: "1\n2\n\n3",
                position: (1.1, 4.04).try_into().unwrap(),
                rotation: Rotation::Three,
                flip: true.into(),
                size: (1.0, 2.0).try_into().unwrap(),
                property: Property::default(),
            }
        )),
    );
}

#[test]
fn parse_wire_object() {
    assert_eq!(
        wire_object::<&str, (&str, ErrorKind)>("N 890 -130 890 -110 {lab=ANALOG_GND}",),
        Ok((
            "",
            Wire {
                start: (890.0, -130.0).try_into().unwrap(),
                end: (890.0, -110.0).try_into().unwrap(),
                property: Property {
                    prop: "lab=ANALOG_GND",
                    attrs: [("lab", "ANALOG_GND")].into()
                },
            }
        )),
    );
}

#[test]
fn parse_line_object() {
    assert_eq!(
        line_object::<&str, (&str, ErrorKind)>("L 4 10 0 20 0 {}",),
        Ok((
            "",
            Line {
                layer: 4,
                start: (10.0, 0.0).try_into().unwrap(),
                end: (20.0, 0.0).try_into().unwrap(),
                property: Property::default(),
            }
        )),
    );
}

#[test]
fn parse_rectangle_object() {
    assert_eq!(
        rectangle_object::<&str, (&str, ErrorKind)>(
            "B 5 -62.5 -2.5 -57.5 2.5 {name=IN dir=in pinnumber=1}",
        ),
        Ok((
            "",
            Rectangle {
                layer: 5,
                start: (-62.5, -2.5).try_into().unwrap(),
                end: (-57.5, 2.5).try_into().unwrap(),
                property: Property {
                    prop: "name=IN dir=in pinnumber=1",
                    attrs: [("name", "IN"), ("dir", "in"), ("pinnumber", "1")].into()
                },
            }
        )),
    );
}

#[test]
fn parse_polygon_object() {
    assert_eq!(
        polygon_object::<&str, (&str, ErrorKind)>(
            "P 3 5 2450 -210 2460 -170 2500 -170 2510 -210 2450 -210 {}",
        ),
        Ok((
            "",
            Polygon {
                layer: 3,
                points: vec![
                    (2450.0, -210.0),
                    (2460.0, -170.0),
                    (2500.0, -170.0),
                    (2510.0, -210.0),
                    (2450.0, -210.0),
                ]
                .try_into()
                .unwrap(),
                property: Property::default(),
            }
        )),
    );
    assert_eq!(
        polygon_object::<&str, (&str, ErrorKind)>("P 3 2 0 0 {}",),
        Err(Err::Failure(("{}", ErrorKind::Char))),
    );
    assert_eq!(
        polygon_object::<&str, (&str, ErrorKind)>("P 3 2 0 0 1 {}",),
        Err(Err::Failure(("{}", ErrorKind::Char))),
    );
}

#[test]
fn parse_arc_object() {
    assert_eq!(
        arc_object::<&str, (&str, ErrorKind)>("A 3 450 -210 120 45 225 {}",),
        Ok((
            "",
            Arc {
                layer: 3,
                center: (450.0, -210.0).try_into().unwrap(),
                radius: 120.0.try_into().unwrap(),
                start_angle: 45.0.try_into().unwrap(),
                sweep_angle: 225.0.try_into().unwrap(),
                property: Property::default(),
            }
        )),
    );
}

#[test]
fn parse_component_instance() {
    assert_eq!(
        component_instance::<&str, (&str, ErrorKind)>("C {capa.sym} 890 -160 0 0 {name=C4}",),
        Ok((
            "",
            Component {
                reference: "capa.sym",
                position: (890.0, -160.0).try_into().unwrap(),
                rotation: Rotation::Zero,
                flip: false.into(),
                property: Property {
                    prop: "name=C4",
                    attrs: [("name", "C4")].into()
                },
                embedding: None,
            }
        )),
    );
}

#[test]
fn parse_7805_sym() {
    let input = include_str!("../../../../assets/7805.sym");
    let result = schematic_full::<&str, (&str, ErrorKind)>(input);
    assert!(result.is_ok(), "parse error: {result:?}");
}

#[test]
fn parse_embedding_sch() {
    let input = include_str!("../../../../assets/embedding.sch");
    let result = schematic_full::<&str, (&str, ErrorKind)>(input);
    assert!(result.is_ok(), "parse error: {result:?}");
}

#[test]
fn parse_pcb_test1_sch() {
    let input = include_str!("../../../../assets/pcb_test1.sch");
    let result = schematic_full::<&str, (&str, ErrorKind)>(input);
    assert!(result.is_ok(), "parse error: {result:?}");
}

#[test]
fn parse_pmos_sym() {
    let input = include_str!("../../../../assets/pmos.sym");
    let result = schematic_full::<&str, (&str, ErrorKind)>(input);
    assert!(result.is_ok(), "parse error: {result:?}");
}
