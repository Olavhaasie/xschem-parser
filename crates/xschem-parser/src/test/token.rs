use crate::token::{
    Component, Flip, Objects, Polygon, Property, Rotation, Schematic, Text, Version, Wire,
};

#[test]
fn version_to_string() {
    let version = Version(Property {
        prop: "xschem version=1.2.3 file_version=1.0 key=value",
        attrs: [("version", "1.2.3"), ("file_version", "1.0")].into(),
    });

    let expected = "v {xschem version=1.2.3 file_version=1.0 key=value}";

    assert_eq!(version.to_string(), expected);
}

#[test]
#[allow(clippy::too_many_lines)]
fn schematic_to_string() {
    let schematic: Schematic<&'static str> = Schematic {
        version: Version(Property {
            prop: "xschem version=3.4.5 file_version=1.2",
            attrs: [("version", "3.4.5"), ("file_version", "1.2")].into(),
        }),
        spice_property: Some(Property::default().into()),
        verilog_property: None,
        vhdl_property: None,
        tedax_property: None,
        symbol_property: Some(
            Property {
                prop: "type=regulator",
                attrs: [("type", "regulator")].into(),
            }
            .into(),
        ),
        texts: vec![Text {
            text: "@name",
            position: (-17.5, -15.0).try_into().unwrap(),
            rotation: Rotation::Zero,
            flip: Flip::Unflipped,
            size: (0.2, 0.2).try_into().unwrap(),
            property: Property::default(),
        }]
        .into(),
        lines: Objects::default(),
        rectangles: Objects::default(),
        polygons: vec![Polygon {
            layer: 3,
            points: vec![(0.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)]
                .try_into()
                .unwrap(),
            property: Property::default(),
        }]
        .into(),
        arcs: Objects::default(),
        wires: vec![
            Wire {
                start: (1.0, 1.0).try_into().unwrap(),
                end: (2.0, 2.0).try_into().unwrap(),
                property: Property {
                    prop: "lab=o",
                    attrs: [("lab", "o")].into(),
                },
            },
            Wire {
                start: (2.0, 2.0).try_into().unwrap(),
                end: (3.0, 3.0).try_into().unwrap(),
                property: Property {
                    prop: "lab=p",
                    attrs: [("lab", "p")].into(),
                },
            },
        ]
        .into(),
        components: vec![
            Component {
                reference: "pin.sym",
                position: (3.0, 3.0).try_into().unwrap(),
                rotation: Rotation::Zero,
                flip: Flip::Flipped,
                property: Property {
                    prop: "name=pin\n",
                    attrs: [("name", "pin")].into(),
                },
                embedding: None,
            },
            Component {
                reference: "pmos.sym",
                position: (1.0, 1.0).try_into().unwrap(),
                rotation: Rotation::Two,
                flip: Flip::Unflipped,
                property: Property {
                    prop: "name=p",
                    attrs: [("name", "p")].into(),
                },
                embedding: Some(
                    Schematic::new(Version(Property {
                        prop: "xschem version=3.4.5 file_version=1.2",
                        attrs: [("version", "3.4.5"), ("file_version", "1.2")].into(),
                    }))
                    .into(),
                ),
            },
        ]
        .into(),
    };

    let expected = "\
        v {xschem version=3.4.5 file_version=1.2}\n\
        S {}\n\
        K {type=regulator}\n\
        T {@name} -17.5 -15 0 0 0.2 0.2 {}\n\
        P 3 4 0 0 1 1 0 1 0 0 {}\n\
        N 1 1 2 2 {lab=o}\n\
        N 2 2 3 3 {lab=p}\n\
        C {pin.sym} 3 3 0 1 {name=pin\n\
        }\n\
        C {pmos.sym} 1 1 2 0 {name=p}\n\
        [\n\
        v {xschem version=3.4.5 file_version=1.2}\n\
        ]\
    ";

    assert_eq!(schematic.to_string(), expected);
}
