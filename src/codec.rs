use crate::{AttachedMedia, Block, Caption, Element, Inline, Kind, Node, Value};
use std::{collections::HashSet, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tmark: {}", self.0)
    }
}
impl std::error::Error for Error {}
fn err<T>(message: impl Into<String>) -> Result<T, Error> {
    Err(Error(message.into()))
}
#[derive(Clone, Debug)]
enum Part {
    Text(String),
    Node(Raw),
    Field(String, Vec<Part>),
}
#[derive(Clone, Debug)]
struct Raw {
    tag: String,
    body: Vec<Part>,
    raw: String,
}
struct Parser<'a> {
    input: &'a str,
    pos: usize,
    depth: usize,
    nesting: usize,
}
impl Parser<'_> {
    fn parts(&mut self, close: bool) -> Result<Vec<Part>, Error> {
        if close {
            self.nesting += 1;
            if self.nesting > 128 {
                return err("maximum structural nesting exceeded");
            }
        }
        let mut out = vec![];
        let mut fields = HashSet::new();
        while self.pos < self.input.len() {
            match self.input.as_bytes()[self.pos] {
                b'}' => {
                    if !close {
                        return err("unexpected closing brace");
                    }
                    self.pos += 1;
                    self.nesting -= 1;
                    return Ok(out);
                }
                b'{' => {
                    let start = self.pos;
                    self.pos += 1;
                    self.depth += 1;
                    if self.depth > 16 {
                        return err("maximum depth exceeded");
                    }
                    let content = self.pos;
                    while self.pos < self.input.len()
                        && !b";}{#\\".contains(&self.input.as_bytes()[self.pos])
                    {
                        self.pos += 1;
                    }
                    let tag = if self.input.as_bytes().get(self.pos) == Some(&b';') {
                        let tag = self.input[content..self.pos].to_string();
                        if !identifier(&tag) {
                            return err("invalid tag");
                        }
                        self.pos += 1;
                        tag
                    } else {
                        self.pos = content;
                        String::new()
                    };
                    let body = self.parts(true)?;
                    self.depth -= 1;
                    out.push(Part::Node(Raw {
                        tag,
                        body,
                        raw: self.input[start..self.pos].into(),
                    }));
                }
                b'#' => {
                    self.pos += 1;
                    let start = self.pos;
                    while self.pos < self.input.len() && id_char(self.input.as_bytes()[self.pos]) {
                        self.pos += 1;
                    }
                    let name = self.input[start..self.pos].to_string();
                    if !identifier(&name) || self.input.as_bytes().get(self.pos) != Some(&b'{') {
                        return err("malformed field");
                    }
                    if !fields.insert(name.clone()) {
                        return err("duplicate field");
                    }
                    self.pos += 1;
                    let value = self.parts(true)?;
                    out.push(Part::Field(name, value));
                }
                _ => {
                    let mut s = String::new();
                    while self.pos < self.input.len() {
                        let c = self.input.as_bytes()[self.pos];
                        if b"{}#".contains(&c) {
                            break;
                        }
                        if c == b'\\' {
                            self.pos += 1;
                            let Some(&c) = self.input.as_bytes().get(self.pos) else {
                                return err("trailing escape");
                            };
                            if !b"{}#\\".contains(&c) {
                                return err("invalid escape");
                            }
                            s.push(c as char);
                            self.pos += 1;
                        } else {
                            let c = self.input[self.pos..].chars().next().unwrap();
                            s.push(c);
                            self.pos += c.len_utf8();
                        }
                    }
                    out.push(Part::Text(s));
                }
            }
        }
        if close {
            err("missing closing brace")
        } else {
            Ok(out)
        }
    }
}
fn id_char(c: u8) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-".contains(&c)
}
fn identifier(s: &str) -> bool {
    !s.is_empty() && s.len() <= 32 && s.bytes().all(id_char)
}
fn structural(p: &Part) -> bool {
    matches!(p,Part::Text(s) if s.chars().all(|c|c=='\n'))
}
fn text(parts: &[Part], expanded: bool) -> Result<String, Error> {
    let mut s = String::new();
    for p in parts {
        if let Part::Text(t) = p {
            s.push_str(t)
        } else {
            return err("expected text");
        }
    }
    if expanded {
        s = s.strip_prefix('\n').unwrap_or(&s).to_string();
        s = s.strip_suffix('\n').unwrap_or(&s).to_string()
    }
    Ok(s)
}
fn number(s: &str) -> bool {
    if s == "-0" {
        return false;
    }
    let s = s.strip_prefix('-').unwrap_or(s);
    let mut parts = s.split('.');
    let i = parts.next().unwrap_or("");
    if i.is_empty() || !i.bytes().all(|c| c.is_ascii_digit()) || i.len() > 1 && i.starts_with('0') {
        return false;
    }
    if let Some(f) = parts.next() {
        if f.is_empty() || f.ends_with('0') || !f.bytes().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    parts.next().is_none()
}
fn numeric(s: &str, integer: bool) -> Result<(), Error> {
    if !number(s) {
        return err("non-canonical number");
    }
    if integer && s.parse::<i64>().is_err() {
        return err("expected integer within i64 range");
    }
    if !integer && s.parse::<f64>().map_or(true, |v| !v.is_finite()) {
        return err("number outside finite f64 range");
    }
    Ok(())
}
/// Parse one value, rejecting unknown tags.
pub fn parse(input: &str) -> Result<Node, Error> {
    parse_bytes(input.as_bytes(), false)
}
/// Parse one value, preserving unknown nodes verbatim.
pub fn parse_soft(input: &str) -> Result<Node, Error> {
    parse_bytes(input.as_bytes(), true)
}
pub fn parse_bytes(input: &[u8], soft: bool) -> Result<Node, Error> {
    let input = std::str::from_utf8(input).map_err(|_| Error("invalid UTF-8".into()))?;
    let normalized = normalized(input)?;
    let parts = Parser {
        input: &normalized,
        pos: 0,
        depth: 0,
        nesting: 0,
    }
    .parts(false)?;
    let parts: Vec<_> = parts.into_iter().filter(|p| !structural(p)).collect();
    if parts.len() != 1 {
        return err("expected a single value");
    }
    match &parts[0] {
        Part::Node(raw) => decode(raw, None, soft, 0),
        Part::Text(s) => Ok(Node::Text(s.clone())),
        _ => err("unexpected field"),
    }
}
fn sequence(
    parts: &[Part],
    typ: &str,
    expanded: bool,
    soft: bool,
    depth: usize,
) -> Result<Vec<Node>, Error> {
    let concrete = match typ {
        "li" => Some(Kind::ListItem),
        "tr" => Some(Kind::TableRow),
        "td" => Some(Kind::Cell),
        "attached" => Some(Kind::AttachedMedia),
        _ => None,
    };
    let mut out = vec![];
    for p in parts {
        if expanded && structural(p) {
            continue;
        }
        let n = match p {
            Part::Text(s) if typ == "inline" => Node::Text(s.clone()),
            Part::Node(r) => decode(r, concrete, soft, depth)?,
            _ => return err("invalid array element"),
        };
        let valid = match (&n, typ) {
            (Node::Text(_), "inline") => true,
            (Node::Unknown(_), "inline" | "block") => true,
            (Node::Element(e), "inline") => e.kind.is_inline(),
            (Node::Element(e), "block") => e.kind.is_block(),
            (Node::Element(e), "media") => matches!(e.kind, Kind::Image | Kind::Video),
            (Node::Element(e), _) => Some(e.kind) == concrete,
            _ => false,
        };
        if !valid {
            return err("incorrect node type in array");
        }
        out.push(n);
    }
    Ok(out)
}
fn decode(raw: &Raw, forced: Option<Kind>, soft: bool, depth: usize) -> Result<Node, Error> {
    if depth >= 16 {
        return err("maximum depth exceeded");
    }
    let reparsed;
    let raw = if forced.is_some() && !raw.raw.is_empty() {
        let input = &raw.raw[1..raw.raw.len() - 1];
        reparsed = Raw {
            tag: String::new(),
            body: Parser {
                input,
                pos: 0,
                depth,
                nesting: 0,
            }
            .parts(false)?,
            raw: String::new(),
        };
        &reparsed
    } else {
        raw
    };
    let kind = if let Some(k) = forced {
        k
    } else {
        match Kind::from_tag(&raw.tag) {
            Some(k) => k,
            None if soft && !raw.tag.is_empty() => return Ok(Node::Unknown(raw.raw.clone())),
            _ => return err("unknown or missing node tag"),
        }
    };
    let schema = kind.schema();
    let expanded = schema.expansion != 0;
    let sequence_body = !matches!(schema.unnamed, "none" | "text" | "integer");
    let last_named = raw.body.iter().rposition(|p| matches!(p, Part::Field(..)));
    let mut fields = vec![];
    let mut unnamed = vec![];
    let mut started = false;
    for (i, p) in raw.body.iter().enumerate() {
        if expanded && structural(p) && (sequence_body || last_named.is_some_and(|last| i < last)) {
            continue;
        }
        if let Part::Field(name, value) = p {
            if started {
                return err("named field follows unnamed content");
            }
            let Some(f) = schema.fields.iter().find(|f| f.name == name) else {
                return err(format!("unknown field {name}"));
            };
            let val = match f.typ {
                "text" => Value::Text(text(value, false)?),
                "integer" | "number" => {
                    let s = text(value, false)?;
                    numeric(&s, f.typ == "integer")?;
                    Value::Number(s)
                }
                "bool" => Value::Bool(match text(value, false)?.as_str() {
                    "t" => true,
                    "f" => false,
                    _ => return err("expected t or f"),
                }),
                "caption" => {
                    let cap = decode(
                        &Raw {
                            tag: String::new(),
                            body: value.clone(),
                            raw: String::new(),
                        },
                        Some(Kind::Caption),
                        soft,
                        depth + 1,
                    )?;
                    let Node::Element(e) = cap else {
                        unreachable!()
                    };
                    Value::Caption(Caption {
                        credit: e
                            .nodes("credit")
                            .to_vec()
                            .into_iter()
                            .map(Inline::try_from)
                            .collect::<Result<_, _>>()?,
                        text: e
                            .children
                            .into_iter()
                            .map(Inline::try_from)
                            .collect::<Result<_, _>>()?,
                    })
                }
                "attached" => {
                    let nodes = sequence(value, "attached", false, soft, depth + 1)?;
                    Value::Attached(
                        nodes
                            .into_iter()
                            .map(|n| {
                                let Node::Element(e) = n else { unreachable!() };
                                Ok(AttachedMedia {
                                    hash: e.string("hash").into(),
                                    content: e
                                        .children
                                        .into_iter()
                                        .map(Block::try_from)
                                        .collect::<Result<_, _>>()?,
                                })
                            })
                            .collect::<Result<_, Error>>()?,
                    )
                }
                typ => Value::Nodes(sequence(value, typ, false, soft, depth + 1)?),
            };
            fields.push((name.clone(), val));
        } else {
            started = true;
            unnamed.push(p.clone());
        }
    }
    let mut e = Element {
        kind,
        fields,
        children: vec![],
        text: String::new(),
    };
    match schema.unnamed {
        "none" => {
            if unnamed.iter().any(|p| !structural(p)) {
                return err("unexpected unnamed value");
            }
        }
        "text" | "integer" => {
            e.text = text(&unnamed, expanded)?;
            if schema.unnamed == "integer" {
                if e.text.is_empty() {
                    e.text = "0".into()
                }
                numeric(&e.text, true)?;
            }
        }
        typ => e.children = sequence(&unnamed, typ, expanded, soft, depth + 1)?,
    }
    // Go scalar number fields have meaningful zero defaults.
    for f in schema.fields {
        if !f.optional && matches!(f.typ, "integer" | "number") && e.field(f.name).is_none() {
            e.fields.push((f.name.into(), Value::Number("0".into())));
        }
    }
    validate(&e)?;
    Ok(Node::Element(e))
}
fn validate(e: &Element) -> Result<(), Error> {
    match e.kind {
        Kind::Header => {
            if !matches!(e.string("s"), "1" | "2" | "3" | "4" | "5" | "6") {
                return err("header size must be 1..6");
            }
        }
        Kind::Video => {
            if e.string("preview").is_empty() {
                return err("video preview is required");
            }
        }
        Kind::ListItem => {
            if e.field("type").is_some()
                && !matches!(e.string("type"), "a" | "A" | "i" | "I" | "1" | "checkbox")
            {
                return err("invalid list item type");
            }
        }
        Kind::Cell => {
            if e.field("align").is_some()
                && !matches!(e.string("align"), "left" | "center" | "right")
            {
                return err("invalid cell alignment");
            }
            if e.field("valign").is_some()
                && !matches!(e.string("valign"), "top" | "middle" | "bottom")
            {
                return err("invalid cell vertical alignment");
            }
        }
        _ => (),
    }
    Ok(())
}
fn escape(s: &str) -> Result<String, Error> {
    if s.contains('\r') {
        return err("standalone CR in text");
    }
    let mut out = String::new();
    for c in s.chars() {
        if "#{}\\".contains(c) {
            out.push('\\')
        }
        out.push(c)
    }
    Ok(out)
}
/// Serialize a value with canonical field order and line layout.
pub fn serialize(node: &Node) -> Result<String, Error> {
    encode(node, false, false, 0)
}
fn encode(node: &Node, omit_tag: bool, fields_only: bool, depth: usize) -> Result<String, Error> {
    match node {
        Node::Text(s) => escape(s),
        Node::Unknown(raw) => {
            let normalized = normalized(raw)?;
            let parts = Parser {
                input: &normalized,
                pos: 0,
                depth,
                nesting: 0,
            }
            .parts(false)?;
            if parts.iter().filter(|p| !structural(p)).count() != 1
                || !parts
                    .iter()
                    .any(|p| matches!(p, Part::Node(r) if !r.tag.is_empty()))
            {
                return err("unknown must contain one node");
            }
            Ok(normalized)
        }
        Node::Element(e) => {
            if depth >= 16 {
                return err("maximum depth exceeded");
            }
            validate(e)?;
            let schema = e.kind.schema();
            if matches!(schema.unnamed, "text" | "integer" | "none") && !e.children.is_empty()
                || !matches!(schema.unnamed, "text" | "integer") && !e.text.is_empty()
            {
                return err("content does not match node schema");
            }
            let expanded = !fields_only
                && (schema.expansion == 1 || schema.expansion == 2 && e.children.len() > 1);
            let mut out = String::new();
            if !fields_only {
                out.push('{');
                if !omit_tag {
                    out.push_str(e.kind.tag());
                    out.push(';')
                }
            }
            let mut seen = HashSet::new();
            for (name, _) in &e.fields {
                if !seen.insert(name) || !schema.fields.iter().any(|f| f.name == name) {
                    return err("duplicate or unknown field");
                }
            }
            for f in schema.fields {
                let default;
                let v = if let Some(v) = e.field(f.name) {
                    v
                } else if !f.optional && matches!(f.typ, "integer" | "number") {
                    default = Value::Number("0".into());
                    &default
                } else {
                    continue;
                };
                let s = match (v, f.typ) {
                    (Value::Text(s), "text") => escape(s)?,
                    (Value::Number(s), "integer" | "number") => {
                        numeric(s, f.typ == "integer")?;
                        s.clone()
                    }
                    (Value::Bool(b), "bool") => {
                        if *b {
                            "t".into()
                        } else {
                            "f".into()
                        }
                    }
                    (Value::Caption(c), "caption") => {
                        encode(&Node::from(c.clone()), false, true, depth + 1)?
                    }
                    (Value::Attached(a), "attached") => {
                        let nodes: Vec<_> = a.iter().cloned().map(Node::from).collect();
                        encode_sequence(&nodes, "attached", false, depth + 1)?
                    }
                    (Value::Nodes(nodes), "inline") => {
                        encode_sequence(nodes, "inline", false, depth + 1)?
                    }
                    _ => return err("incorrect field value type"),
                };
                if !f.optional
                    && match v {
                        Value::Text(s) => s.is_empty(),
                        Value::Bool(b) => !*b,
                        Value::Nodes(n) => n.is_empty(),
                        Value::Attached(a) => a.is_empty(),
                        _ => false,
                    }
                {
                    continue;
                }
                if expanded {
                    out.push('\n')
                }
                out.push('#');
                out.push_str(f.name);
                out.push('{');
                out.push_str(&s);
                out.push('}');
            }
            match schema.unnamed {
                "text" | "integer" => {
                    let text = if schema.unnamed == "integer" {
                        if e.text.is_empty() {
                            "0"
                        } else {
                            &e.text
                        }
                    } else {
                        &e.text
                    };
                    if schema.unnamed == "integer" {
                        numeric(text, true)?
                    }
                    if !text.is_empty() {
                        if expanded {
                            out.push('\n')
                        }
                        out.push_str(&escape(text)?)
                    }
                }
                "none" => {
                    if !e.children.is_empty() || !e.text.is_empty() {
                        return err("unexpected content");
                    }
                }
                typ => out.push_str(&encode_sequence(&e.children, typ, expanded, depth + 1)?),
            }
            if expanded {
                out.push('\n')
            }
            if !fields_only {
                out.push('}')
            }
            Ok(out)
        }
    }
}
fn encode_sequence(
    nodes: &[Node],
    typ: &str,
    expanded: bool,
    depth: usize,
) -> Result<String, Error> {
    let concrete = match typ {
        "li" => Some(Kind::ListItem),
        "tr" => Some(Kind::TableRow),
        "td" => Some(Kind::Cell),
        "attached" => Some(Kind::AttachedMedia),
        _ => None,
    };
    let mut out = String::new();
    for n in nodes {
        let valid = match (n, typ) {
            (Node::Text(_), "inline") => true,
            (Node::Unknown(_), "inline" | "block") => true,
            (Node::Element(e), "inline") => e.kind.is_inline(),
            (Node::Element(e), "block") => e.kind.is_block(),
            (Node::Element(e), "media") => matches!(e.kind, Kind::Image | Kind::Video),
            (Node::Element(e), _) => Some(e.kind) == concrete,
            _ => false,
        };
        if !valid {
            return err("incorrect node type in array");
        }
        if expanded {
            out.push('\n')
        }
        out.push_str(&encode(n, concrete.is_some(), false, depth)?)
    }
    Ok(out)
}

/// A value that can be serialized to and parsed from tmark.
///
/// Implement this trait to support application-specific values. Concrete node
/// implementations set `KIND` so arrays omit their element tags.
pub trait Tmark: Sized {
    const KIND: Option<Kind> = None;
    fn from_tmark(input: &str) -> Result<Self, Error>;
    fn from_tmark_soft(input: &str) -> Result<Self, Error> {
        Self::from_tmark(input)
    }
    fn to_tmark(&self) -> Result<String, Error>;
}
/// Parse a typed value, scalar, or sequence.
pub fn from_str<T: Tmark>(input: &str) -> Result<T, Error> {
    T::from_tmark(input)
}
/// Parse a typed value while preserving unknown inline and block nodes.
pub fn from_str_soft<T: Tmark>(input: &str) -> Result<T, Error> {
    T::from_tmark_soft(input)
}
/// Serialize a typed value, scalar, or sequence.
pub fn to_string<T: Tmark>(value: &T) -> Result<String, Error> {
    value.to_tmark()
}
fn scalar_text(input: &str) -> Result<String, Error> {
    let input = normalized(input)?;
    let parts = Parser {
        input: &input,
        pos: 0,
        depth: 0,
        nesting: 0,
    }
    .parts(false)?;
    text(&parts, false)
}
fn normalized(input: &str) -> Result<String, Error> {
    if input.starts_with('\u{feff}')
        || input
            .as_bytes()
            .iter()
            .enumerate()
            .any(|(i, c)| *c == b'\r' && input.as_bytes().get(i + 1) != Some(&b'\n'))
    {
        return err("invalid BOM or standalone CR");
    }
    Ok(input.replace("\r\n", "\n"))
}
impl Tmark for Node {
    fn from_tmark(input: &str) -> Result<Self, Error> {
        parse(input)
    }
    fn from_tmark_soft(input: &str) -> Result<Self, Error> {
        parse_soft(input)
    }
    fn to_tmark(&self) -> Result<String, Error> {
        serialize(self)
    }
}
impl Tmark for String {
    fn from_tmark(input: &str) -> Result<Self, Error> {
        scalar_text(input)
    }
    fn to_tmark(&self) -> Result<String, Error> {
        escape(self)
    }
}
impl Tmark for bool {
    fn from_tmark(input: &str) -> Result<Self, Error> {
        match scalar_text(input)?.as_str() {
            "t" => Ok(true),
            "f" => Ok(false),
            _ => err("expected t or f"),
        }
    }
    fn to_tmark(&self) -> Result<String, Error> {
        Ok(if *self { "t" } else { "f" }.into())
    }
}
macro_rules! integer_codec {
    ($($typ:ty),*) => {$(impl Tmark for $typ {
        fn from_tmark(input:&str)->Result<Self,Error>{let s=scalar_text(input)?;if !number(&s)||s.contains('.') {return err("expected canonical integer")};s.parse().map_err(|_|Error("integer outside target range".into()))}
        fn to_tmark(&self)->Result<String,Error>{Ok(self.to_string())}
    })*}
}
integer_codec!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
macro_rules! float_codec {
    ($($typ:ty),*) => {$(impl Tmark for $typ {
        fn from_tmark(input:&str)->Result<Self,Error>{let s=scalar_text(input)?;if !number(&s){return err("non-canonical number")};let value:Self=s.parse().map_err(|_|Error("invalid number".into()))?;if !value.is_finite(){return err("non-finite number")};Ok(value)}
        fn to_tmark(&self)->Result<String,Error>{if !self.is_finite(){return err("non-finite number")};Ok(if *self==0.0 {"0".into()}else{self.to_string()})}
    })*}
}
float_codec!(f32, f64);
impl<T: Tmark> Tmark for Vec<T> {
    fn from_tmark(input: &str) -> Result<Self, Error> {
        decode_vec(input, false)
    }
    fn from_tmark_soft(input: &str) -> Result<Self, Error> {
        decode_vec(input, true)
    }
    fn to_tmark(&self) -> Result<String, Error> {
        encode_slice(self)
    }
}
macro_rules! node_codec {
    ($($typ:ident),*) => {$(impl Tmark for crate::$typ {
        const KIND:Option<Kind>=Some(Kind::$typ);
        fn from_tmark(input:&str)->Result<Self,Error>{Self::try_from(parse(input)?)}
        fn from_tmark_soft(input:&str)->Result<Self,Error>{Self::try_from(parse_soft(input)?)}
        fn to_tmark(&self)->Result<String,Error>{serialize(&Node::from(self.clone()))}
    })*}
}
node_codec!(
    Caption,
    Link,
    AnchorLink,
    Reference,
    ReferenceLink,
    Bold,
    Italic,
    Marked,
    Underline,
    Strikethrough,
    Spoiler,
    Subscript,
    Superscript,
    DateTime,
    Code,
    Math,
    Icon,
    Paragraph,
    Header,
    Preformatted,
    MathBlock,
    Anchor,
    Divider,
    Blockquote,
    PullQuote,
    Collage,
    List,
    ListItem,
    Map,
    Image,
    Video,
    Audio,
    Slideshow,
    Table,
    TableRow,
    Cell,
    Details,
    AttachedMedia,
    Document
);
macro_rules! category_codec {
    ($($typ:ident),*) => {$(impl Tmark for crate::$typ {
        fn from_tmark(input:&str)->Result<Self,Error>{Self::try_from(parse(input)?)}
        fn from_tmark_soft(input:&str)->Result<Self,Error>{Self::try_from(parse_soft(input)?)}
        fn to_tmark(&self)->Result<String,Error>{serialize(self.as_node())}
    })*}
}
category_codec!(Inline, Block, Media);

fn decode_vec<T: Tmark>(input: &str, soft: bool) -> Result<Vec<T>, Error> {
    let input = normalized(input)?;
    let parts = Parser {
        input: &input,
        pos: 0,
        depth: 0,
        nesting: 0,
    }
    .parts(false)?;
    let mut out = vec![];
    for part in parts {
        match part {
            Part::Node(raw) => {
                let encoded = if let Some(kind) = T::KIND {
                    serialize(&decode(&raw, Some(kind), soft, 0)?)?
                } else {
                    raw.raw
                };
                out.push(if soft {
                    T::from_tmark_soft(&encoded)?
                } else {
                    T::from_tmark(&encoded)?
                });
            }
            Part::Text(s) => out.push(T::from_tmark(&escape(&s)?)?),
            Part::Field(..) => return err("unexpected field in sequence"),
        }
    }
    Ok(out)
}

fn encode_slice<T: Tmark>(items: &[T]) -> Result<String, Error> {
    let mut out = String::new();
    for item in items {
        let value = item.to_tmark()?;
        if T::KIND.is_some() {
            let node = parse_soft(&value)?;
            out.push_str(&encode(&node, true, false, 0)?)
        } else {
            out.push_str(&value)
        }
    }
    Ok(out)
}

impl<T: Tmark, const N: usize> Tmark for [T; N] {
    fn from_tmark(input: &str) -> Result<Self, Error> {
        decode_vec(input, false)?
            .try_into()
            .map_err(|_| Error("array length mismatch".into()))
    }
    fn from_tmark_soft(input: &str) -> Result<Self, Error> {
        decode_vec(input, true)?
            .try_into()
            .map_err(|_| Error("array length mismatch".into()))
    }
    fn to_tmark(&self) -> Result<String, Error> {
        encode_slice(self)
    }
}
