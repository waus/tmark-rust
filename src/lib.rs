#![doc = include_str!("../README.md")]
mod codec;
mod render;
pub use codec::{
    from_str, from_str_soft, parse, parse_bytes, parse_soft, serialize, to_string, Error, Tmark,
};
pub use render::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Text(String),
    Element(Element),
    Unknown(String),
}
impl From<&str> for Node {
    fn from(s: &str) -> Self {
        Self::Text(s.into())
    }
}
impl From<String> for Node {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub kind: Kind,
    pub fields: Vec<(String, Value)>,
    pub children: Vec<Node>,
    pub text: String,
}
impl Element {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            fields: vec![],
            children: vec![],
            text: String::new(),
        }
    }
    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
    pub fn string(&self, name: &str) -> &str {
        match self.field(name) {
            Some(Value::Text(v) | Value::Number(v)) => v,
            _ => "",
        }
    }
    pub fn boolean(&self, name: &str) -> bool {
        matches!(self.field(name), Some(Value::Bool(true)))
    }
    pub fn nodes(&self, name: &str) -> &[Node] {
        match self.field(name) {
            Some(Value::Nodes(v)) => v,
            _ => &[],
        }
    }
    pub fn caption(&self) -> Option<&Caption> {
        match self.field("caption") {
            Some(Value::Caption(v)) => Some(v),
            _ => None,
        }
    }
}
impl From<Element> for Node {
    fn from(e: Element) -> Self {
        Self::Element(e)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Number(String),
    Bool(bool),
    Nodes(Vec<Node>),
    Caption(Caption),
    Attached(Vec<AttachedMedia>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
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
    Document,
}
impl Kind {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Caption => "caption",
            Self::Link => "a",
            Self::AnchorLink => "anchor-link",
            Self::Reference => "ref",
            Self::ReferenceLink => "ref-link",
            Self::Bold => "b",
            Self::Italic => "i",
            Self::Marked => "m",
            Self::Underline => "u",
            Self::Strikethrough => "s",
            Self::Spoiler => "spoiler",
            Self::Subscript => "sub",
            Self::Superscript => "sup",
            Self::DateTime => "datetime",
            Self::Code => "code",
            Self::Math => "math",
            Self::Icon => "icon",
            Self::Paragraph => "p",
            Self::Header => "h",
            Self::Preformatted => "pre",
            Self::MathBlock => "math-block",
            Self::Anchor => "anchor",
            Self::Divider => "hr",
            Self::Blockquote => "q",
            Self::PullQuote => "as",
            Self::Collage => "collage",
            Self::List => "list",
            Self::ListItem => "li",
            Self::Map => "map",
            Self::Image => "img",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Slideshow => "slideshow",
            Self::Table => "table",
            Self::TableRow => "tr",
            Self::Cell => "td",
            Self::Details => "details",
            Self::AttachedMedia => "attached",
            Self::Document => "document",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
            "caption" => Self::Caption,
            "a" => Self::Link,
            "anchor-link" => Self::AnchorLink,
            "ref" => Self::Reference,
            "ref-link" => Self::ReferenceLink,
            "b" => Self::Bold,
            "i" => Self::Italic,
            "m" => Self::Marked,
            "u" => Self::Underline,
            "s" => Self::Strikethrough,
            "spoiler" => Self::Spoiler,
            "sub" => Self::Subscript,
            "sup" => Self::Superscript,
            "datetime" => Self::DateTime,
            "code" => Self::Code,
            "math" => Self::Math,
            "icon" => Self::Icon,
            "p" => Self::Paragraph,
            "h" => Self::Header,
            "pre" => Self::Preformatted,
            "math-block" => Self::MathBlock,
            "anchor" => Self::Anchor,
            "hr" => Self::Divider,
            "q" => Self::Blockquote,
            "as" => Self::PullQuote,
            "collage" => Self::Collage,
            "list" => Self::List,
            "li" => Self::ListItem,
            "map" => Self::Map,
            "img" => Self::Image,
            "video" => Self::Video,
            "audio" => Self::Audio,
            "slideshow" => Self::Slideshow,
            "table" => Self::Table,
            "tr" => Self::TableRow,
            "td" => Self::Cell,
            "details" => Self::Details,
            "attached" => Self::AttachedMedia,
            "document" => Self::Document,
            _ => return None,
        })
    }
    pub(crate) fn schema(self) -> Schema {
        match self {
            Self::Caption => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "credit",
                    typ: "inline",
                    optional: false,
                }],
            },
            Self::Link => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "href",
                    typ: "text",
                    optional: false,
                }],
            },
            Self::AnchorLink => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "name",
                    typ: "text",
                    optional: false,
                }],
            },
            Self::Reference => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "name",
                    typ: "text",
                    optional: false,
                }],
            },
            Self::ReferenceLink => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "name",
                    typ: "text",
                    optional: false,
                }],
            },
            Self::Bold => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Italic => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Marked => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Underline => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Strikethrough => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Spoiler => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Subscript => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Superscript => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::DateTime => Schema {
                unnamed: "integer",
                expansion: 0,
                fields: &[Field {
                    name: "timezone",
                    typ: "text",
                    optional: false,
                }],
            },
            Self::Code => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[],
            },
            Self::Math => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[],
            },
            Self::Icon => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[Field {
                    name: "alt",
                    typ: "text",
                    optional: true,
                }],
            },
            Self::Paragraph => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[],
            },
            Self::Header => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "s",
                    typ: "integer",
                    optional: false,
                }],
            },
            Self::Preformatted => Schema {
                unnamed: "text",
                expansion: 1,
                fields: &[Field {
                    name: "language",
                    typ: "text",
                    optional: true,
                }],
            },
            Self::MathBlock => Schema {
                unnamed: "text",
                expansion: 1,
                fields: &[],
            },
            Self::Anchor => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[],
            },
            Self::Divider => Schema {
                unnamed: "none",
                expansion: 0,
                fields: &[],
            },
            Self::Blockquote => Schema {
                unnamed: "block",
                expansion: 2,
                fields: &[Field {
                    name: "credit",
                    typ: "inline",
                    optional: false,
                }],
            },
            Self::PullQuote => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[Field {
                    name: "credit",
                    typ: "inline",
                    optional: false,
                }],
            },
            Self::Collage => Schema {
                unnamed: "media",
                expansion: 1,
                fields: &[Field {
                    name: "caption",
                    typ: "caption",
                    optional: true,
                }],
            },
            Self::List => Schema {
                unnamed: "li",
                expansion: 1,
                fields: &[],
            },
            Self::ListItem => Schema {
                unnamed: "block",
                expansion: 2,
                fields: &[
                    Field {
                        name: "type",
                        typ: "text",
                        optional: true,
                    },
                    Field {
                        name: "order",
                        typ: "integer",
                        optional: true,
                    },
                    Field {
                        name: "checked",
                        typ: "bool",
                        optional: true,
                    },
                ],
            },
            Self::Map => Schema {
                unnamed: "none",
                expansion: 0,
                fields: &[
                    Field {
                        name: "lat",
                        typ: "number",
                        optional: false,
                    },
                    Field {
                        name: "lon",
                        typ: "number",
                        optional: false,
                    },
                    Field {
                        name: "zoom",
                        typ: "integer",
                        optional: true,
                    },
                    Field {
                        name: "caption",
                        typ: "caption",
                        optional: true,
                    },
                ],
            },
            Self::Image => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[
                    Field {
                        name: "caption",
                        typ: "caption",
                        optional: true,
                    },
                    Field {
                        name: "has_spoiler",
                        typ: "bool",
                        optional: false,
                    },
                ],
            },
            Self::Video => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[
                    Field {
                        name: "caption",
                        typ: "caption",
                        optional: true,
                    },
                    Field {
                        name: "has_spoiler",
                        typ: "bool",
                        optional: false,
                    },
                    Field {
                        name: "loop",
                        typ: "bool",
                        optional: false,
                    },
                    Field {
                        name: "preview",
                        typ: "text",
                        optional: false,
                    },
                ],
            },
            Self::Audio => Schema {
                unnamed: "text",
                expansion: 0,
                fields: &[Field {
                    name: "caption",
                    typ: "caption",
                    optional: true,
                }],
            },
            Self::Slideshow => Schema {
                unnamed: "media",
                expansion: 1,
                fields: &[Field {
                    name: "caption",
                    typ: "caption",
                    optional: true,
                }],
            },
            Self::Table => Schema {
                unnamed: "tr",
                expansion: 1,
                fields: &[
                    Field {
                        name: "caption",
                        typ: "inline",
                        optional: false,
                    },
                    Field {
                        name: "bordered",
                        typ: "bool",
                        optional: false,
                    },
                    Field {
                        name: "striped",
                        typ: "bool",
                        optional: false,
                    },
                ],
            },
            Self::TableRow => Schema {
                unnamed: "td",
                expansion: 0,
                fields: &[],
            },
            Self::Cell => Schema {
                unnamed: "inline",
                expansion: 0,
                fields: &[
                    Field {
                        name: "header",
                        typ: "bool",
                        optional: false,
                    },
                    Field {
                        name: "colspan",
                        typ: "integer",
                        optional: true,
                    },
                    Field {
                        name: "rowspan",
                        typ: "integer",
                        optional: true,
                    },
                    Field {
                        name: "align",
                        typ: "text",
                        optional: true,
                    },
                    Field {
                        name: "valign",
                        typ: "text",
                        optional: true,
                    },
                ],
            },
            Self::Details => Schema {
                unnamed: "block",
                expansion: 2,
                fields: &[
                    Field {
                        name: "summary",
                        typ: "inline",
                        optional: false,
                    },
                    Field {
                        name: "open",
                        typ: "bool",
                        optional: false,
                    },
                ],
            },
            Self::AttachedMedia => Schema {
                unnamed: "block",
                expansion: 1,
                fields: &[Field {
                    name: "hash",
                    typ: "text",
                    optional: false,
                }],
            },
            Self::Document => Schema {
                unnamed: "block",
                expansion: 1,
                fields: &[
                    Field {
                        name: "url",
                        typ: "text",
                        optional: false,
                    },
                    Field {
                        name: "title",
                        typ: "text",
                        optional: false,
                    },
                    Field {
                        name: "description",
                        typ: "text",
                        optional: true,
                    },
                    Field {
                        name: "author_name",
                        typ: "text",
                        optional: true,
                    },
                    Field {
                        name: "author_url",
                        typ: "text",
                        optional: true,
                    },
                    Field {
                        name: "image_url",
                        typ: "text",
                        optional: true,
                    },
                    Field {
                        name: "attached_media",
                        typ: "attached",
                        optional: false,
                    },
                ],
            },
        }
    }
    pub fn is_inline(self) -> bool {
        matches!(
            self,
            Self::Link
                | Self::AnchorLink
                | Self::Reference
                | Self::ReferenceLink
                | Self::Bold
                | Self::Italic
                | Self::Marked
                | Self::Underline
                | Self::Strikethrough
                | Self::Spoiler
                | Self::Subscript
                | Self::Superscript
                | Self::DateTime
                | Self::Code
                | Self::Math
                | Self::Icon
        )
    }
    pub fn is_block(self) -> bool {
        !self.is_inline()
            && !matches!(
                self,
                Self::Caption | Self::Document | Self::AttachedMedia | Self::ListItem
            )
    }
}
pub(crate) struct Schema {
    pub unnamed: &'static str,
    pub expansion: u8,
    pub fields: &'static [Field],
}
pub(crate) struct Field {
    pub name: &'static str,
    pub typ: &'static str,
    pub optional: bool,
}

/// A node allowed in rich inline content.
#[derive(Clone, Debug, PartialEq)]
pub struct Inline(Node);
/// A node allowed in block content.
#[derive(Clone, Debug, PartialEq)]
pub struct Block(Node);
/// An image or video allowed in collages and slideshows.
#[derive(Clone, Debug, PartialEq)]
pub struct Media(Node);
impl Inline {
    pub fn as_node(&self) -> &Node {
        &self.0
    }
    pub fn into_node(self) -> Node {
        self.0
    }
}
impl From<Inline> for Node {
    fn from(v: Inline) -> Node {
        v.0
    }
}
impl TryFrom<Node> for Inline {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        if matches!(&node, Node::Text(_) | Node::Unknown(_))
            || matches!(&node,Node::Element(e) if e.kind.is_inline())
        {
            Ok(Self(node))
        } else {
            Err(Error("expected inline node".into()))
        }
    }
}
impl Block {
    pub fn as_node(&self) -> &Node {
        &self.0
    }
    pub fn into_node(self) -> Node {
        self.0
    }
}
impl From<Block> for Node {
    fn from(v: Block) -> Node {
        v.0
    }
}
impl TryFrom<Node> for Block {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        if matches!(&node, Node::Unknown(_))
            || matches!(&node,Node::Element(e) if e.kind.is_block())
        {
            Ok(Self(node))
        } else {
            Err(Error("expected block node".into()))
        }
    }
}
impl Media {
    pub fn as_node(&self) -> &Node {
        &self.0
    }
    pub fn into_node(self) -> Node {
        self.0
    }
}
impl From<Media> for Node {
    fn from(v: Media) -> Node {
        v.0
    }
}
impl TryFrom<Node> for Media {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        if matches!(&node,Node::Element(e) if matches!(e.kind,Kind::Image|Kind::Video)) {
            Ok(Self(node))
        } else {
            Err(Error("expected media node".into()))
        }
    }
}
impl From<&str> for Inline {
    fn from(v: &str) -> Self {
        Self(Node::Text(v.into()))
    }
}
impl From<String> for Inline {
    fn from(v: String) -> Self {
        Self(Node::Text(v))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableCellAlign {
    Left,
    Center,
    Right,
}
impl TableCellAlign {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}
impl std::str::FromStr for TableCellAlign {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            _ => Err(Error("invalid alignment".into())),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableCellValign {
    Top,
    Middle,
    Bottom,
}
impl TableCellValign {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Middle => "middle",
            Self::Bottom => "bottom",
        }
    }
}
impl std::str::FromStr for TableCellValign {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "top" => Ok(Self::Top),
            "middle" => Ok(Self::Middle),
            "bottom" => Ok(Self::Bottom),
            _ => Err(Error("invalid alignment".into())),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Caption {
    pub credit: Vec<Inline>,
    pub text: Vec<Inline>,
}
impl From<Caption> for Node {
    fn from(value: Caption) -> Node {
        let mut e = Element::new(Kind::Caption);
        {
            let v = value.credit;
            e.fields.push((
                "credit".into(),
                Value::Nodes(v.into_iter().map(Node::from).collect()),
            ));
        }
        e.children = value.text.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Caption {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected caption node".into()));
        };
        if e.kind != Kind::Caption {
            return Err(Error("expected caption node".into()));
        };
        Ok(Self {
            credit: e
                .nodes("credit")
                .to_vec()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
            text: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Link {
    pub href: String,
    pub children: Vec<Inline>,
}
impl From<Link> for Node {
    fn from(value: Link) -> Node {
        let mut e = Element::new(Kind::Link);
        {
            let v = value.href;
            e.fields.push(("href".into(), Value::Text(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Link {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected a node".into()));
        };
        if e.kind != Kind::Link {
            return Err(Error("expected a node".into()));
        };
        Ok(Self {
            href: e.string("href").to_string(),
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Link> for Inline {
    fn from(v: Link) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnchorLink {
    pub anchor_name: String,
    pub children: Vec<Inline>,
}
impl From<AnchorLink> for Node {
    fn from(value: AnchorLink) -> Node {
        let mut e = Element::new(Kind::AnchorLink);
        {
            let v = value.anchor_name;
            e.fields.push(("name".into(), Value::Text(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for AnchorLink {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected anchor-link node".into()));
        };
        if e.kind != Kind::AnchorLink {
            return Err(Error("expected anchor-link node".into()));
        };
        Ok(Self {
            anchor_name: e.string("name").to_string(),
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<AnchorLink> for Inline {
    fn from(v: AnchorLink) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reference {
    pub name: String,
    pub children: Vec<Inline>,
}
impl From<Reference> for Node {
    fn from(value: Reference) -> Node {
        let mut e = Element::new(Kind::Reference);
        {
            let v = value.name;
            e.fields.push(("name".into(), Value::Text(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Reference {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected ref node".into()));
        };
        if e.kind != Kind::Reference {
            return Err(Error("expected ref node".into()));
        };
        Ok(Self {
            name: e.string("name").to_string(),
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Reference> for Inline {
    fn from(v: Reference) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReferenceLink {
    pub reference_name: String,
    pub children: Vec<Inline>,
}
impl From<ReferenceLink> for Node {
    fn from(value: ReferenceLink) -> Node {
        let mut e = Element::new(Kind::ReferenceLink);
        {
            let v = value.reference_name;
            e.fields.push(("name".into(), Value::Text(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for ReferenceLink {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected ref-link node".into()));
        };
        if e.kind != Kind::ReferenceLink {
            return Err(Error("expected ref-link node".into()));
        };
        Ok(Self {
            reference_name: e.string("name").to_string(),
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<ReferenceLink> for Inline {
    fn from(v: ReferenceLink) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bold {
    pub children: Vec<Inline>,
}
impl From<Bold> for Node {
    fn from(value: Bold) -> Node {
        let mut e = Element::new(Kind::Bold);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Bold {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected b node".into()));
        };
        if e.kind != Kind::Bold {
            return Err(Error("expected b node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Bold> for Inline {
    fn from(v: Bold) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Italic {
    pub children: Vec<Inline>,
}
impl From<Italic> for Node {
    fn from(value: Italic) -> Node {
        let mut e = Element::new(Kind::Italic);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Italic {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected i node".into()));
        };
        if e.kind != Kind::Italic {
            return Err(Error("expected i node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Italic> for Inline {
    fn from(v: Italic) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Marked {
    pub children: Vec<Inline>,
}
impl From<Marked> for Node {
    fn from(value: Marked) -> Node {
        let mut e = Element::new(Kind::Marked);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Marked {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected m node".into()));
        };
        if e.kind != Kind::Marked {
            return Err(Error("expected m node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Marked> for Inline {
    fn from(v: Marked) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Underline {
    pub children: Vec<Inline>,
}
impl From<Underline> for Node {
    fn from(value: Underline) -> Node {
        let mut e = Element::new(Kind::Underline);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Underline {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected u node".into()));
        };
        if e.kind != Kind::Underline {
            return Err(Error("expected u node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Underline> for Inline {
    fn from(v: Underline) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Strikethrough {
    pub children: Vec<Inline>,
}
impl From<Strikethrough> for Node {
    fn from(value: Strikethrough) -> Node {
        let mut e = Element::new(Kind::Strikethrough);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Strikethrough {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected s node".into()));
        };
        if e.kind != Kind::Strikethrough {
            return Err(Error("expected s node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Strikethrough> for Inline {
    fn from(v: Strikethrough) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Spoiler {
    pub children: Vec<Inline>,
}
impl From<Spoiler> for Node {
    fn from(value: Spoiler) -> Node {
        let mut e = Element::new(Kind::Spoiler);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Spoiler {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected spoiler node".into()));
        };
        if e.kind != Kind::Spoiler {
            return Err(Error("expected spoiler node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Spoiler> for Inline {
    fn from(v: Spoiler) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Subscript {
    pub children: Vec<Inline>,
}
impl From<Subscript> for Node {
    fn from(value: Subscript) -> Node {
        let mut e = Element::new(Kind::Subscript);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Subscript {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected sub node".into()));
        };
        if e.kind != Kind::Subscript {
            return Err(Error("expected sub node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Subscript> for Inline {
    fn from(v: Subscript) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Superscript {
    pub children: Vec<Inline>,
}
impl From<Superscript> for Node {
    fn from(value: Superscript) -> Node {
        let mut e = Element::new(Kind::Superscript);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Superscript {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected sup node".into()));
        };
        if e.kind != Kind::Superscript {
            return Err(Error("expected sup node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Superscript> for Inline {
    fn from(v: Superscript) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DateTime {
    pub timezone: String,
    pub unix: i64,
}
impl From<DateTime> for Node {
    fn from(value: DateTime) -> Node {
        let mut e = Element::new(Kind::DateTime);
        {
            let v = value.timezone;
            e.fields.push(("timezone".into(), Value::Text(v)));
        }
        e.text = value.unix.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for DateTime {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected datetime node".into()));
        };
        if e.kind != Kind::DateTime {
            return Err(Error("expected datetime node".into()));
        };
        Ok(Self {
            timezone: e.string("timezone").to_string(),
            unix: e
                .text
                .parse()
                .map_err(|_| Error("invalid integer".into()))?,
        })
    }
}
impl From<DateTime> for Inline {
    fn from(v: DateTime) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Code {
    pub text: String,
}
impl From<Code> for Node {
    fn from(value: Code) -> Node {
        let mut e = Element::new(Kind::Code);
        e.text = value.text.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Code {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected code node".into()));
        };
        if e.kind != Kind::Code {
            return Err(Error("expected code node".into()));
        };
        Ok(Self {
            text: e.text.clone(),
        })
    }
}
impl From<Code> for Inline {
    fn from(v: Code) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Math {
    pub expression: String,
}
impl From<Math> for Node {
    fn from(value: Math) -> Node {
        let mut e = Element::new(Kind::Math);
        e.text = value.expression.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Math {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected math node".into()));
        };
        if e.kind != Kind::Math {
            return Err(Error("expected math node".into()));
        };
        Ok(Self {
            expression: e.text.clone(),
        })
    }
}
impl From<Math> for Inline {
    fn from(v: Math) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Icon {
    pub alternative_text: Option<String>,
    pub src: String,
}
impl From<Icon> for Node {
    fn from(value: Icon) -> Node {
        let mut e = Element::new(Kind::Icon);
        if let Some(v) = value.alternative_text {
            e.fields.push(("alt".into(), Value::Text(v)));
        }
        e.text = value.src.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Icon {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected icon node".into()));
        };
        if e.kind != Kind::Icon {
            return Err(Error("expected icon node".into()));
        };
        Ok(Self {
            alternative_text: if e.field("alt").is_some() {
                Some(e.string("alt").to_string())
            } else {
                None
            },
            src: e.text.clone(),
        })
    }
}
impl From<Icon> for Inline {
    fn from(v: Icon) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
/// A paragraph accepts inline content only.
/// ```compile_fail
/// use tmark::{Image, Paragraph};
/// let paragraph = Paragraph { children: vec![Image::default().into()] };
/// ```
pub struct Paragraph {
    pub children: Vec<Inline>,
}
impl From<Paragraph> for Node {
    fn from(value: Paragraph) -> Node {
        let mut e = Element::new(Kind::Paragraph);
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Paragraph {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected p node".into()));
        };
        if e.kind != Kind::Paragraph {
            return Err(Error("expected p node".into()));
        };
        Ok(Self {
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Paragraph> for Block {
    fn from(v: Paragraph) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Header {
    pub size: i64,
    pub children: Vec<Inline>,
}
impl From<Header> for Node {
    fn from(value: Header) -> Node {
        let mut e = Element::new(Kind::Header);
        {
            let v = value.size;
            e.fields.push(("s".into(), Value::Number(v.to_string())));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Header {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected h node".into()));
        };
        if e.kind != Kind::Header {
            return Err(Error("expected h node".into()));
        };
        Ok(Self {
            size: if e.field("s").is_none() {
                0
            } else {
                e.string("s")
                    .parse()
                    .map_err(|_| Error("invalid integer".into()))?
            },
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Header> for Block {
    fn from(v: Header) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Preformatted {
    pub language: Option<String>,
    pub text: String,
}
impl From<Preformatted> for Node {
    fn from(value: Preformatted) -> Node {
        let mut e = Element::new(Kind::Preformatted);
        if let Some(v) = value.language {
            e.fields.push(("language".into(), Value::Text(v)));
        }
        e.text = value.text.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Preformatted {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected pre node".into()));
        };
        if e.kind != Kind::Preformatted {
            return Err(Error("expected pre node".into()));
        };
        Ok(Self {
            language: if e.field("language").is_some() {
                Some(e.string("language").to_string())
            } else {
                None
            },
            text: e.text.clone(),
        })
    }
}
impl From<Preformatted> for Block {
    fn from(v: Preformatted) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MathBlock {
    pub expression: String,
}
impl From<MathBlock> for Node {
    fn from(value: MathBlock) -> Node {
        let mut e = Element::new(Kind::MathBlock);
        e.text = value.expression.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for MathBlock {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected math-block node".into()));
        };
        if e.kind != Kind::MathBlock {
            return Err(Error("expected math-block node".into()));
        };
        Ok(Self {
            expression: e.text.clone(),
        })
    }
}
impl From<MathBlock> for Block {
    fn from(v: MathBlock) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Anchor {
    pub name: String,
}
impl From<Anchor> for Node {
    fn from(value: Anchor) -> Node {
        let mut e = Element::new(Kind::Anchor);
        e.text = value.name.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Anchor {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected anchor node".into()));
        };
        if e.kind != Kind::Anchor {
            return Err(Error("expected anchor node".into()));
        };
        Ok(Self {
            name: e.text.clone(),
        })
    }
}
impl From<Anchor> for Block {
    fn from(v: Anchor) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Divider {}
impl From<Divider> for Node {
    fn from(_: Divider) -> Node {
        let e = Element::new(Kind::Divider);
        Node::Element(e)
    }
}
impl TryFrom<Node> for Divider {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected hr node".into()));
        };
        if e.kind != Kind::Divider {
            return Err(Error("expected hr node".into()));
        };
        Ok(Self {})
    }
}
impl From<Divider> for Block {
    fn from(v: Divider) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Blockquote {
    pub credit: Vec<Inline>,
    pub blocks: Vec<Block>,
}
impl From<Blockquote> for Node {
    fn from(value: Blockquote) -> Node {
        let mut e = Element::new(Kind::Blockquote);
        {
            let v = value.credit;
            e.fields.push((
                "credit".into(),
                Value::Nodes(v.into_iter().map(Node::from).collect()),
            ));
        }
        e.children = value.blocks.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Blockquote {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected q node".into()));
        };
        if e.kind != Kind::Blockquote {
            return Err(Error("expected q node".into()));
        };
        Ok(Self {
            credit: e
                .nodes("credit")
                .to_vec()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
            blocks: e
                .children
                .clone()
                .into_iter()
                .map(Block::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Blockquote> for Block {
    fn from(v: Blockquote) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PullQuote {
    pub credit: Vec<Inline>,
    pub text: Vec<Inline>,
}
impl From<PullQuote> for Node {
    fn from(value: PullQuote) -> Node {
        let mut e = Element::new(Kind::PullQuote);
        {
            let v = value.credit;
            e.fields.push((
                "credit".into(),
                Value::Nodes(v.into_iter().map(Node::from).collect()),
            ));
        }
        e.children = value.text.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for PullQuote {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected as node".into()));
        };
        if e.kind != Kind::PullQuote {
            return Err(Error("expected as node".into()));
        };
        Ok(Self {
            credit: e
                .nodes("credit")
                .to_vec()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
            text: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<PullQuote> for Block {
    fn from(v: PullQuote) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Collage {
    pub caption: Option<Caption>,
    pub children: Vec<Media>,
}
impl From<Collage> for Node {
    fn from(value: Collage) -> Node {
        let mut e = Element::new(Kind::Collage);
        if let Some(v) = value.caption {
            e.fields.push(("caption".into(), Value::Caption(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Collage {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected collage node".into()));
        };
        if e.kind != Kind::Collage {
            return Err(Error("expected collage node".into()));
        };
        Ok(Self {
            caption: if e.field("caption").is_some() {
                Some(e.caption().cloned().unwrap_or_default())
            } else {
                None
            },
            children: e
                .children
                .clone()
                .into_iter()
                .map(Media::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Collage> for Block {
    fn from(v: Collage) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
/// A list contains concrete list items.
/// ```compile_fail
/// use tmark::{List, Paragraph};
/// let list = List { items: vec![Paragraph::default()] };
/// ```
pub struct List {
    pub items: Vec<ListItem>,
}
impl From<List> for Node {
    fn from(value: List) -> Node {
        let mut e = Element::new(Kind::List);
        e.children = value.items.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for List {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected list node".into()));
        };
        if e.kind != Kind::List {
            return Err(Error("expected list node".into()));
        };
        Ok(Self {
            items: e
                .children
                .clone()
                .into_iter()
                .map(ListItem::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<List> for Block {
    fn from(v: List) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListItem {
    pub r#type: Option<String>,
    pub order: Option<i64>,
    pub checked: Option<bool>,
    pub children: Vec<Block>,
}
impl From<ListItem> for Node {
    fn from(value: ListItem) -> Node {
        let mut e = Element::new(Kind::ListItem);
        if let Some(v) = value.r#type {
            e.fields.push(("type".into(), Value::Text(v)));
        }
        if let Some(v) = value.order {
            e.fields
                .push(("order".into(), Value::Number(v.to_string())));
        }
        if let Some(v) = value.checked {
            e.fields.push(("checked".into(), Value::Bool(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for ListItem {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected li node".into()));
        };
        if e.kind != Kind::ListItem {
            return Err(Error("expected li node".into()));
        };
        Ok(Self {
            r#type: if e.field("type").is_some() {
                Some(e.string("type").to_string())
            } else {
                None
            },
            order: if e.field("order").is_some() {
                Some(if e.field("order").is_none() {
                    0
                } else {
                    e.string("order")
                        .parse()
                        .map_err(|_| Error("invalid integer".into()))?
                })
            } else {
                None
            },
            checked: if e.field("checked").is_some() {
                Some(e.boolean("checked"))
            } else {
                None
            },
            children: e
                .children
                .clone()
                .into_iter()
                .map(Block::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Map {
    pub lat: f64,
    pub lon: f64,
    pub zoom: Option<i64>,
    pub caption: Option<Caption>,
}
impl From<Map> for Node {
    fn from(value: Map) -> Node {
        let mut e = Element::new(Kind::Map);
        {
            let v = value.lat;
            e.fields.push((
                "lat".into(),
                Value::Number(if v == 0.0 { "0".into() } else { v.to_string() }),
            ));
        }
        {
            let v = value.lon;
            e.fields.push((
                "lon".into(),
                Value::Number(if v == 0.0 { "0".into() } else { v.to_string() }),
            ));
        }
        if let Some(v) = value.zoom {
            e.fields.push(("zoom".into(), Value::Number(v.to_string())));
        }
        if let Some(v) = value.caption {
            e.fields.push(("caption".into(), Value::Caption(v)));
        }
        Node::Element(e)
    }
}
impl TryFrom<Node> for Map {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected map node".into()));
        };
        if e.kind != Kind::Map {
            return Err(Error("expected map node".into()));
        };
        Ok(Self {
            lat: if e.field("lat").is_none() {
                0.0
            } else {
                e.string("lat")
                    .parse()
                    .map_err(|_| Error("invalid number".into()))?
            },
            lon: if e.field("lon").is_none() {
                0.0
            } else {
                e.string("lon")
                    .parse()
                    .map_err(|_| Error("invalid number".into()))?
            },
            zoom: if e.field("zoom").is_some() {
                Some(if e.field("zoom").is_none() {
                    0
                } else {
                    e.string("zoom")
                        .parse()
                        .map_err(|_| Error("invalid integer".into()))?
                })
            } else {
                None
            },
            caption: if e.field("caption").is_some() {
                Some(e.caption().cloned().unwrap_or_default())
            } else {
                None
            },
        })
    }
}
impl From<Map> for Block {
    fn from(v: Map) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Image {
    pub caption: Option<Caption>,
    pub has_spoiler: bool,
    pub src: String,
}
impl From<Image> for Node {
    fn from(value: Image) -> Node {
        let mut e = Element::new(Kind::Image);
        if let Some(v) = value.caption {
            e.fields.push(("caption".into(), Value::Caption(v)));
        }
        {
            let v = value.has_spoiler;
            e.fields.push(("has_spoiler".into(), Value::Bool(v)));
        }
        e.text = value.src.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Image {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected img node".into()));
        };
        if e.kind != Kind::Image {
            return Err(Error("expected img node".into()));
        };
        Ok(Self {
            caption: if e.field("caption").is_some() {
                Some(e.caption().cloned().unwrap_or_default())
            } else {
                None
            },
            has_spoiler: e.boolean("has_spoiler"),
            src: e.text.clone(),
        })
    }
}
impl From<Image> for Block {
    fn from(v: Image) -> Self {
        Self(Node::from(v))
    }
}
impl From<Image> for Media {
    fn from(v: Image) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Video {
    pub caption: Option<Caption>,
    pub has_spoiler: bool,
    pub r#loop: bool,
    pub preview: String,
    pub src: String,
}
impl From<Video> for Node {
    fn from(value: Video) -> Node {
        let mut e = Element::new(Kind::Video);
        if let Some(v) = value.caption {
            e.fields.push(("caption".into(), Value::Caption(v)));
        }
        {
            let v = value.has_spoiler;
            e.fields.push(("has_spoiler".into(), Value::Bool(v)));
        }
        {
            let v = value.r#loop;
            e.fields.push(("loop".into(), Value::Bool(v)));
        }
        {
            let v = value.preview;
            e.fields.push(("preview".into(), Value::Text(v)));
        }
        e.text = value.src.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Video {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected video node".into()));
        };
        if e.kind != Kind::Video {
            return Err(Error("expected video node".into()));
        };
        Ok(Self {
            caption: if e.field("caption").is_some() {
                Some(e.caption().cloned().unwrap_or_default())
            } else {
                None
            },
            has_spoiler: e.boolean("has_spoiler"),
            r#loop: e.boolean("loop"),
            preview: e.string("preview").to_string(),
            src: e.text.clone(),
        })
    }
}
impl From<Video> for Block {
    fn from(v: Video) -> Self {
        Self(Node::from(v))
    }
}
impl From<Video> for Media {
    fn from(v: Video) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Audio {
    pub caption: Option<Caption>,
    pub src: String,
}
impl From<Audio> for Node {
    fn from(value: Audio) -> Node {
        let mut e = Element::new(Kind::Audio);
        if let Some(v) = value.caption {
            e.fields.push(("caption".into(), Value::Caption(v)));
        }
        e.text = value.src.to_string();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Audio {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected audio node".into()));
        };
        if e.kind != Kind::Audio {
            return Err(Error("expected audio node".into()));
        };
        Ok(Self {
            caption: if e.field("caption").is_some() {
                Some(e.caption().cloned().unwrap_or_default())
            } else {
                None
            },
            src: e.text.clone(),
        })
    }
}
impl From<Audio> for Block {
    fn from(v: Audio) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Slideshow {
    pub caption: Option<Caption>,
    pub children: Vec<Media>,
}
impl From<Slideshow> for Node {
    fn from(value: Slideshow) -> Node {
        let mut e = Element::new(Kind::Slideshow);
        if let Some(v) = value.caption {
            e.fields.push(("caption".into(), Value::Caption(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Slideshow {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected slideshow node".into()));
        };
        if e.kind != Kind::Slideshow {
            return Err(Error("expected slideshow node".into()));
        };
        Ok(Self {
            caption: if e.field("caption").is_some() {
                Some(e.caption().cloned().unwrap_or_default())
            } else {
                None
            },
            children: e
                .children
                .clone()
                .into_iter()
                .map(Media::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Slideshow> for Block {
    fn from(v: Slideshow) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub caption: Vec<Inline>,
    pub bordered: bool,
    pub striped: bool,
    pub rows: Vec<TableRow>,
}
impl From<Table> for Node {
    fn from(value: Table) -> Node {
        let mut e = Element::new(Kind::Table);
        {
            let v = value.caption;
            e.fields.push((
                "caption".into(),
                Value::Nodes(v.into_iter().map(Node::from).collect()),
            ));
        }
        {
            let v = value.bordered;
            e.fields.push(("bordered".into(), Value::Bool(v)));
        }
        {
            let v = value.striped;
            e.fields.push(("striped".into(), Value::Bool(v)));
        }
        e.children = value.rows.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Table {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected table node".into()));
        };
        if e.kind != Kind::Table {
            return Err(Error("expected table node".into()));
        };
        Ok(Self {
            caption: e
                .nodes("caption")
                .to_vec()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
            bordered: e.boolean("bordered"),
            striped: e.boolean("striped"),
            rows: e
                .children
                .clone()
                .into_iter()
                .map(TableRow::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Table> for Block {
    fn from(v: Table) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableRow {
    pub cells: Vec<Cell>,
}
impl From<TableRow> for Node {
    fn from(value: TableRow) -> Node {
        let mut e = Element::new(Kind::TableRow);
        e.children = value.cells.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for TableRow {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected tr node".into()));
        };
        if e.kind != Kind::TableRow {
            return Err(Error("expected tr node".into()));
        };
        Ok(Self {
            cells: e
                .children
                .clone()
                .into_iter()
                .map(Cell::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<TableRow> for Block {
    fn from(v: TableRow) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub is_header: bool,
    pub colspan: Option<i64>,
    pub rowspan: Option<i64>,
    pub align: Option<TableCellAlign>,
    pub valign: Option<TableCellValign>,
    pub children: Vec<Inline>,
}
impl From<Cell> for Node {
    fn from(value: Cell) -> Node {
        let mut e = Element::new(Kind::Cell);
        {
            let v = value.is_header;
            e.fields.push(("header".into(), Value::Bool(v)));
        }
        if let Some(v) = value.colspan {
            e.fields
                .push(("colspan".into(), Value::Number(v.to_string())));
        }
        if let Some(v) = value.rowspan {
            e.fields
                .push(("rowspan".into(), Value::Number(v.to_string())));
        }
        if let Some(v) = value.align {
            e.fields
                .push(("align".into(), Value::Text(v.as_str().into())));
        }
        if let Some(v) = value.valign {
            e.fields
                .push(("valign".into(), Value::Text(v.as_str().into())));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Cell {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected td node".into()));
        };
        if e.kind != Kind::Cell {
            return Err(Error("expected td node".into()));
        };
        Ok(Self {
            is_header: e.boolean("header"),
            colspan: if e.field("colspan").is_some() {
                Some(if e.field("colspan").is_none() {
                    0
                } else {
                    e.string("colspan")
                        .parse()
                        .map_err(|_| Error("invalid integer".into()))?
                })
            } else {
                None
            },
            rowspan: if e.field("rowspan").is_some() {
                Some(if e.field("rowspan").is_none() {
                    0
                } else {
                    e.string("rowspan")
                        .parse()
                        .map_err(|_| Error("invalid integer".into()))?
                })
            } else {
                None
            },
            align: if e.field("align").is_some() {
                Some(e.string("align").parse()?)
            } else {
                None
            },
            valign: if e.field("valign").is_some() {
                Some(e.string("valign").parse()?)
            } else {
                None
            },
            children: e
                .children
                .clone()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Cell> for Block {
    fn from(v: Cell) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Details {
    pub summary: Vec<Inline>,
    pub is_open: bool,
    pub children: Vec<Block>,
}
impl From<Details> for Node {
    fn from(value: Details) -> Node {
        let mut e = Element::new(Kind::Details);
        {
            let v = value.summary;
            e.fields.push((
                "summary".into(),
                Value::Nodes(v.into_iter().map(Node::from).collect()),
            ));
        }
        {
            let v = value.is_open;
            e.fields.push(("open".into(), Value::Bool(v)));
        }
        e.children = value.children.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Details {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected details node".into()));
        };
        if e.kind != Kind::Details {
            return Err(Error("expected details node".into()));
        };
        Ok(Self {
            summary: e
                .nodes("summary")
                .to_vec()
                .into_iter()
                .map(Inline::try_from)
                .collect::<Result<_, _>>()?,
            is_open: e.boolean("open"),
            children: e
                .children
                .clone()
                .into_iter()
                .map(Block::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl From<Details> for Block {
    fn from(v: Details) -> Self {
        Self(Node::from(v))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttachedMedia {
    pub hash: String,
    pub content: Vec<Block>,
}
impl From<AttachedMedia> for Node {
    fn from(value: AttachedMedia) -> Node {
        let mut e = Element::new(Kind::AttachedMedia);
        {
            let v = value.hash;
            e.fields.push(("hash".into(), Value::Text(v)));
        }
        e.children = value.content.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for AttachedMedia {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected attached node".into()));
        };
        if e.kind != Kind::AttachedMedia {
            return Err(Error("expected attached node".into()));
        };
        Ok(Self {
            hash: e.string("hash").to_string(),
            content: e
                .children
                .clone()
                .into_iter()
                .map(Block::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub image_url: Option<String>,
    pub attached_media: Vec<AttachedMedia>,
    pub content: Vec<Block>,
}
impl From<Document> for Node {
    fn from(value: Document) -> Node {
        let mut e = Element::new(Kind::Document);
        {
            let v = value.url;
            e.fields.push(("url".into(), Value::Text(v)));
        }
        {
            let v = value.title;
            e.fields.push(("title".into(), Value::Text(v)));
        }
        if let Some(v) = value.description {
            e.fields.push(("description".into(), Value::Text(v)));
        }
        if let Some(v) = value.author_name {
            e.fields.push(("author_name".into(), Value::Text(v)));
        }
        if let Some(v) = value.author_url {
            e.fields.push(("author_url".into(), Value::Text(v)));
        }
        if let Some(v) = value.image_url {
            e.fields.push(("image_url".into(), Value::Text(v)));
        }
        {
            let v = value.attached_media;
            e.fields.push(("attached_media".into(), Value::Attached(v)));
        }
        e.children = value.content.into_iter().map(Node::from).collect();
        Node::Element(e)
    }
}
impl TryFrom<Node> for Document {
    type Error = Error;
    fn try_from(node: Node) -> Result<Self, Error> {
        serialize(&node)?;
        let Node::Element(e) = node else {
            return Err(Error("expected document node".into()));
        };
        if e.kind != Kind::Document {
            return Err(Error("expected document node".into()));
        };
        Ok(Self {
            url: e.string("url").to_string(),
            title: e.string("title").to_string(),
            description: if e.field("description").is_some() {
                Some(e.string("description").to_string())
            } else {
                None
            },
            author_name: if e.field("author_name").is_some() {
                Some(e.string("author_name").to_string())
            } else {
                None
            },
            author_url: if e.field("author_url").is_some() {
                Some(e.string("author_url").to_string())
            } else {
                None
            },
            image_url: if e.field("image_url").is_some() {
                Some(e.string("image_url").to_string())
            } else {
                None
            },
            attached_media: match e.field("attached_media") {
                Some(Value::Attached(v)) => v.clone(),
                _ => vec![],
            },
            content: e
                .children
                .clone()
                .into_iter()
                .map(Block::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Rich inline content.
pub type RichText = Vec<Inline>;
/// Rich block content.
pub type RichBlocks = Vec<Block>;
