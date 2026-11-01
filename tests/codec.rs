use std::{fs, path::Path};
use tmark::*;
#[test]
fn valid_fixtures_round_trip() {
    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/valid")).unwrap() {
        let path = entry.unwrap().path();
        let input = fs::read_to_string(&path).unwrap();
        let node = if path.extension().unwrap() == "tmark" {
            parse_soft(&input)
        } else {
            parse(&input)
        }
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(serialize(&node).unwrap(), input, "{}", path.display());
    }
}
#[test]
fn invalid_fixtures_rejected() {
    for entry in
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/invalid")).unwrap()
    {
        let path = entry.unwrap().path();
        let input = fs::read(&path).unwrap();
        assert!(parse_bytes(&input, false).is_err(), "{}", path.display());
    }
}
#[test]
fn typed_construction() {
    let node = Node::from(Document {
        title: "Example".into(),
        content: vec![Paragraph {
            children: vec![
                "Hello ".into(),
                Bold {
                    children: vec!["world".into()],
                }
                .into(),
            ],
        }
        .into()],
        ..Default::default()
    });
    assert_eq!(
        serialize(&node).unwrap(),
        "{document;\n#title{Example}\n{p;Hello {b;world}}\n}"
    );
}
#[test]
fn strict_and_soft() {
    assert!(parse("{future;#label{x}body}").is_err());
    let n = parse_soft("{future;#label{x}body}").unwrap();
    assert_eq!(serialize(&n).unwrap(), "{future;#label{x}body}");
    assert!(parse_soft("{p;#future{x}body}").is_err());
}
#[test]
fn trust_boundaries() {
    for s in [
        "{p;x}x",
        "{p;\\x}",
        "{p;#x{1}#x{2}}",
        "{p;\r}",
        "\u{feff}{p;x}",
        "{datetime;-0}",
        "{datetime;1.1}",
        "{h;#s{01}x}",
        "{h;#s{7}x}",
        "{video;file}",
        "{list;{#type{bad}{p;x}}}",
        "{p;{p;x}}",
    ] {
        assert!(parse_soft(s).is_err(), "{s}")
    }
    assert!(parse_bytes(&[0xff], false).is_err());
    assert_eq!(
        serialize(&parse("{p;a\r\nb}").unwrap()).unwrap(),
        "{p;a\nb}"
    );
}
#[test]
fn depth_limit() {
    for depth in [15, 16] {
        let input = format!("{}x", "{b;".repeat(depth + 1));
        let input = format!("{}{}", input, "}".repeat(depth + 1));
        assert_eq!(parse(&input).is_ok(), depth == 15);
    }
}

#[test]
fn typed_scalars_and_sequences() {
    assert!(from_str::<bool>("t").unwrap());
    assert!(from_str::<bool>("true").is_err());
    assert_eq!(
        from_str::<u128>("340282366920938463463374607431768211455").unwrap(),
        u128::MAX
    );
    assert!(from_str::<i8>("128").is_err());
    assert!(from_str::<f64>("1e2").is_err());
    assert_eq!(to_string(&-0.0_f64).unwrap(), "0");
    assert!(to_string(&f64::INFINITY).is_err());
    assert_eq!(from_str::<String>("\\#\\{\\}\\\\").unwrap(), "#{}\\");
    let items = vec![ListItem {
        children: vec![Paragraph {
            children: vec!["one".into()],
        }
        .into()],
        ..Default::default()
    }];
    let encoded = to_string(&items).unwrap();
    assert_eq!(encoded, "{{p;one}}");
    assert_eq!(from_str::<Vec<ListItem>>(&encoded).unwrap(), items);
    let inline = vec![
        Inline::from("one"),
        Bold {
            children: vec!["two".into()],
        }
        .into(),
    ];
    assert_eq!(
        from_str::<Vec<Inline>>(&to_string(&inline).unwrap()).unwrap(),
        inline
    );
    let document = Document {
        content: vec![Paragraph {
            children: vec!["x".into()],
        }
        .into()],
        ..Default::default()
    };
    assert_eq!(
        from_str::<Document>(&to_string(&document).unwrap()).unwrap(),
        document
    );
}
#[test]
fn dynamic_model_constraints_and_structural_limits() {
    let mut header = Element::new(Kind::Header);
    header.fields.push(("s".into(), Value::Text("2".into())));
    assert!(Header::try_from(Node::Element(header)).is_err());
    assert!(Inline::try_from(Node::from(Paragraph::default())).is_err());
    assert!(Block::try_from(Node::from(Bold::default())).is_err());
    assert!(Media::try_from(Node::from(Audio {
        src: "x".into(),
        ..Default::default()
    }))
    .is_err());
    let nested = format!("{{future;{}{}}}", "#f{".repeat(1000), "}".repeat(1000));
    assert!(parse_soft(&nested).is_err());
    for raw in ["{}", "{future;\r}", "\u{feff}{future;}"] {
        assert!(serialize(&Node::Unknown(raw.into())).is_err());
    }
    assert_eq!(
        serialize(&Node::Unknown("{future;\r\nx}".into())).unwrap(),
        "{future;\nx}"
    );
}

#[test]
fn typed_soft_parsing() {
    let source = "{document;\n{future;x}\n}";
    assert!(from_str::<Document>(source).is_err());
    assert_eq!(
        to_string(&from_str_soft::<Document>(source).unwrap()).unwrap(),
        source
    );
    let source = "{future;x}{p;y}";
    assert_eq!(
        to_string(&from_str_soft::<Vec<Block>>(source).unwrap()).unwrap(),
        source
    );
}

#[test]
fn empty_dynamic_fields_still_require_the_schema_type() {
    for value in [
        Value::Text(String::new()),
        Value::Bool(false),
        Value::Nodes(vec![]),
        Value::Attached(vec![]),
    ] {
        let mut map = Element::new(Kind::Map);
        map.fields.push(("lat".into(), value));
        assert!(serialize(&Node::from(map)).is_err());
    }
    let mut link = Element::new(Kind::Link);
    link.fields.push(("href".into(), Value::Bool(false)));
    assert!(Link::try_from(Node::from(link)).is_err());
}

#[test]
fn concrete_sequences_preserve_nested_unknown_nodes() {
    let source = "{{future;x}}";
    let items = from_str_soft::<Vec<Paragraph>>(source).unwrap();
    assert_eq!(to_string(&items).unwrap(), source);
    let items = from_str_soft::<[Paragraph; 1]>(source).unwrap();
    assert_eq!(to_string(&items).unwrap(), source);
}
