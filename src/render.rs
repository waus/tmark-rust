use crate::{Caption, Element, Kind, Node};
use chrono::{FixedOffset, TimeZone, Utc};

#[derive(Clone, Copy, Debug, Default)]
pub struct MarkdownOptions {
    pub html_less: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalOptions {
    pub plain_text: bool,
    pub reveal_spoilers: bool,
}

fn html_escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '"' => "&#34;".into(),
            '\'' => "&#39;".into(),
            _ => c.to_string(),
        })
        .collect()
}
fn html_attr(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn markdown_escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c == '\n' {
            out.push_str("  \n");
        } else {
            if "\\*_[]<>#&`~!-+=.)".contains(c) {
                out.push('\\');
            }
            out.push(c);
        }
    }
    out
}
fn url_escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\\' => "%5C".into(),
            ' ' => "%20".into(),
            '\t' => "%09".into(),
            '\n' => "%0A".into(),
            '\r' => "%0D".into(),
            '(' => "%28".into(),
            ')' => "%29".into(),
            '<' => "%3C".into(),
            '>' => "%3E".into(),
            '"' => "%22".into(),
            '&' => "&amp;".into(),
            _ => c.to_string(),
        })
        .collect()
}
fn level(e: &Element) -> usize {
    match e.string("s").parse::<usize>().unwrap_or(4) {
        n @ 1..=6 => n,
        _ => 4,
    }
}
fn element(n: &Node) -> Option<&Element> {
    if let Node::Element(e) = n {
        Some(e)
    } else {
        None
    }
}
fn is_kind(n: &Node, kind: Kind) -> bool {
    element(n).is_some_and(|e| e.kind == kind)
}
fn number(e: &Element, field: &str) -> i64 {
    e.string(field).parse().unwrap_or(0)
}
fn ordered(e: &Element) -> bool {
    e.children
        .iter()
        .filter_map(element)
        .any(|i| i.field("order").is_some())
}
fn fixed_offset(name: &str) -> Option<FixedOffset> {
    let (text, utc_prefix) = match name.strip_prefix("UTC") {
        Some(text) => (text, true),
        None => (name, false),
    };
    let sign = match text.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let (hours, minutes) = match text[1..].split_once(':') {
        Some((hours, minutes)) if minutes.len() == 2 => (hours, minutes),
        None if utc_prefix => (&text[1..], "00"),
        _ => return None,
    };
    if !(hours.len() == 2 || utc_prefix && hours.len() == 1)
        || !hours
            .bytes()
            .chain(minutes.bytes())
            .all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    FixedOffset::east_opt(sign * (hours * 60 + minutes) * 60)
}
fn date(e: &Element) -> Option<String> {
    let name = e.string("timezone");
    let timestamp = Utc.timestamp_opt(e.text.parse::<i64>().ok()?, 0).single()?;
    if name.starts_with(['+', '-']) || name.starts_with("UTC+") || name.starts_with("UTC-") {
        let offset = fixed_offset(name)?;
        let label = if offset.local_minus_utc() == 0 {
            "UTC".into()
        } else {
            format!("UTC{offset}")
        };
        return Some(format!(
            "{} {label}",
            timestamp.with_timezone(&offset).format("%Y-%m-%d %H:%M:%S")
        ));
    }
    if name.starts_with("GMT+") || name.starts_with("GMT-") {
        return None;
    }
    let timezone = match name {
        "" | "UTC" | "GMT" | "Z" => chrono_tz::UTC,
        _ => name.parse::<chrono_tz::Tz>().ok()?,
    };
    Some(
        timestamp
            .with_timezone(&timezone)
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string(),
    )
}

fn caption_nodes(nodes: &[crate::Inline]) -> Vec<Node> {
    nodes.iter().map(|n| n.as_node().clone()).collect()
}
#[derive(Clone, Copy)]
enum RenderMode {
    Html,
    Markdown(MarkdownOptions),
    Terminal(TerminalOptions),
}
fn validate_root(node: &Node, mode: RenderMode) -> Result<(), crate::Error> {
    if let Some(e) = element(node).filter(|e| e.kind == Kind::Document) {
        for child in &e.children {
            validate(child, true, mode)?;
        }
        Ok(())
    } else {
        validate(node, true, mode)
    }
}
// Only fallbacks serialize nodes. Ordinary text may contain controls that the
// terminal renderer safely removes even though the wire format rejects them.
fn validate(node: &Node, block: bool, mode: RenderMode) -> Result<(), crate::Error> {
    let Some(e) = element(node) else {
        if block || matches!(node, Node::Unknown(_)) {
            crate::serialize(node)?;
        }
        return Ok(());
    };
    if let RenderMode::Terminal(options) = mode {
        if !options.reveal_spoilers
            && ((!block && e.kind == Kind::Spoiler)
                || (block
                    && matches!(e.kind, Kind::Image | Kind::Video)
                    && e.boolean("has_spoiler")))
        {
            return Ok(());
        }
    }
    if let RenderMode::Markdown(options) = mode {
        if block
            && !options.html_less
            && matches!(e.kind, Kind::Image | Kind::Video)
            && e.boolean("has_spoiler")
        {
            return validate(node, block, RenderMode::Html);
        }
        if e.kind == Kind::Table
            && !options.html_less
            && e.children.iter().filter_map(element).any(|r| {
                r.children
                    .iter()
                    .filter_map(element)
                    .any(|c| number(c, "colspan") > 1 || number(c, "rowspan") > 1)
            })
        {
            return validate(node, block, RenderMode::Html);
        }
    }
    let supported = if block {
        matches!(
            e.kind,
            Kind::Paragraph
                | Kind::Header
                | Kind::Blockquote
                | Kind::PullQuote
                | Kind::List
                | Kind::Divider
                | Kind::Preformatted
                | Kind::Anchor
                | Kind::Image
                | Kind::Video
                | Kind::Audio
                | Kind::Collage
                | Kind::Slideshow
                | Kind::Table
                | Kind::Details
        ) || (!matches!(mode, RenderMode::Html) && e.kind == Kind::MathBlock)
            || (matches!(mode, RenderMode::Terminal(_)) && e.kind == Kind::Map)
    } else {
        matches!(
            e.kind,
            Kind::Link
                | Kind::AnchorLink
                | Kind::Reference
                | Kind::ReferenceLink
                | Kind::Bold
                | Kind::Italic
                | Kind::Marked
                | Kind::Underline
                | Kind::Strikethrough
                | Kind::Spoiler
                | Kind::Subscript
                | Kind::Superscript
                | Kind::Code
                | Kind::Icon
        ) || (e.kind == Kind::DateTime && date(e).is_some())
            || (!matches!(mode, RenderMode::Html) && e.kind == Kind::Math)
    };
    if !supported {
        crate::serialize(node)?;
        return Ok(());
    }
    if e.kind == Kind::Map
        && ["lat", "lon"].iter().any(|field| {
            e.string(field)
                .parse::<f64>()
                .is_ok_and(|value| !value.is_finite())
        })
    {
        return Err(crate::Error("non-finite map coordinate".into()));
    }
    match e.kind {
        Kind::List => {
            for item in e.children.iter().filter_map(element) {
                for child in &item.children {
                    validate(child, true, mode)?;
                }
            }
        }
        Kind::Table => {
            for row in e.children.iter().filter_map(element) {
                for cell in row.children.iter().filter_map(element) {
                    for child in &cell.children {
                        validate(child, false, mode)?;
                    }
                }
            }
        }
        _ => {
            for child in &e.children {
                validate(
                    child,
                    matches!(
                        e.kind,
                        Kind::Blockquote | Kind::Details | Kind::Collage | Kind::Slideshow
                    ),
                    mode,
                )?;
            }
        }
    }
    for (name, value) in &e.fields {
        match value {
            crate::Value::Nodes(nodes) => {
                let mode = if name == "summary"
                    && matches!(
                        mode,
                        RenderMode::Markdown(MarkdownOptions { html_less: false })
                    ) {
                    RenderMode::Html
                } else {
                    mode
                };
                for child in nodes {
                    validate(child, false, mode)?;
                }
            }
            crate::Value::Caption(c) => {
                for child in c.text.iter().chain(&c.credit) {
                    validate(child.as_node(), false, mode)?;
                }
            }
            _ => (),
        }
    }
    Ok(())
}
fn fallback(n: &Node) -> String {
    crate::serialize(n).expect("valid renderer node")
}
fn wrapped(value: String, opening: &str, closing: &str) -> String {
    let text = value.trim();
    if text.is_empty() {
        return value;
    }
    let start = value.find(text).unwrap();
    format!(
        "{}{}{}{}{}",
        &value[..start],
        opening,
        text,
        closing,
        &value[start + text.len()..]
    )
}
fn fence(text: &str, initial: usize) -> String {
    let mut value = "`".repeat(initial);
    while text.contains(&value) {
        value.push('`');
    }
    value
}
fn inline_code(text: &str, html_less: bool) -> String {
    if text.is_empty() {
        return if html_less {
            String::new()
        } else {
            "<code></code>".into()
        };
    }
    let f = fence(text, 1);
    if text.starts_with('`')
        || text.ends_with('`')
        || (text.starts_with(' ') && text.ends_with(' ') && !text.trim_matches(' ').is_empty())
    {
        format!("{f} {text} {f}")
    } else {
        format!("{f}{text}{f}")
    }
}
fn preformatted(text: &str, language: &str) -> String {
    let f = fence(text, 3);
    format!(
        "{f}{}\n{text}{}{f}\n",
        language
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('`', ""),
        if text.ends_with('\n') { "" } else { "\n" }
    )
}

pub fn render_html(node: &Node) -> Result<String, crate::Error> {
    validate_root(node, RenderMode::Html)?;
    Ok(
        if let Some(e) = element(node).filter(|e| e.kind == Kind::Document) {
            html_blocks(&e.children)
        } else {
            html_block(node)
        },
    )
}
pub fn to_html(content: &[Node]) -> Result<String, crate::Error> {
    for node in content {
        validate(node, true, RenderMode::Html)?;
    }
    Ok(html_blocks(content))
}
fn html_blocks(nodes: &[Node]) -> String {
    nodes.iter().map(html_block).collect()
}
fn html_rich(nodes: &[Node]) -> String {
    nodes.iter().map(html_inline).collect()
}
fn html_caption(c: Option<&Caption>) -> String {
    let Some(c) = c else {
        return String::new();
    };
    let mut out = format!("<figcaption>{}", html_rich(&caption_nodes(&c.text)));
    if !c.credit.is_empty() {
        if !c.text.is_empty() {
            out.push_str("<br>\n");
        }
        out.push_str(&format!(
            "<cite>{}</cite>",
            html_rich(&caption_nodes(&c.credit))
        ));
    }
    out.push_str("</figcaption>");
    out
}
fn html_block(node: &Node) -> String {
    let Some(e) = element(node) else {
        return format!("<pre><code>{}</code></pre>\n", html_escape(&fallback(node)));
    };
    let children = &e.children;
    match e.kind {
        Kind::Paragraph => format!("<p>{}</p>\n", html_rich(children)),
        Kind::Header => {
            let l = level(e);
            format!("<h{l}>{}</h{l}>\n", html_rich(children))
        }
        Kind::Blockquote => format!(
            "<blockquote>\n{}{}</blockquote>\n",
            html_blocks(children),
            if e.nodes("credit").is_empty() {
                String::new()
            } else {
                format!("<cite>{}</cite>\n", html_rich(e.nodes("credit")))
            }
        ),
        Kind::PullQuote => format!(
            "<aside class=\"pull-quote\"><blockquote>{}{}</blockquote></aside>\n",
            html_rich(children),
            if e.nodes("credit").is_empty() {
                String::new()
            } else {
                format!("<cite>{}</cite>", html_rich(e.nodes("credit")))
            }
        ),
        Kind::Divider => "<hr>\n".into(),
        Kind::Preformatted => format!(
            "<pre><code{}>{}</code></pre>\n",
            if e.field("language").is_some() {
                format!(" class=\"language-{}\"", html_escape(e.string("language")))
            } else {
                String::new()
            },
            html_escape(&e.text)
        ),
        Kind::Anchor => format!("<a id=\"{}\"></a>\n", html_escape(&e.text)),
        Kind::List => {
            let tag = if ordered(e) { "ol" } else { "ul" };
            let mut out = format!("<{tag}>\n");
            for i in children.iter().filter_map(element) {
                out.push_str("<li");
                if i.field("order").is_some() {
                    out.push_str(&format!(" value=\"{}\"", i.string("order")));
                }
                let typ = i.string("type");
                if typ.len() == 1 && "aAiI1".contains(typ) {
                    out.push_str(&format!(" type=\"{typ}\""));
                }
                out.push('>');
                if i.field("checked").is_some() {
                    out.push_str(&format!(
                        "<input type=\"checkbox\" disabled{}> ",
                        if i.boolean("checked") { " checked" } else { "" }
                    ));
                }
                out.push('\n');
                out.push_str(&html_blocks(&i.children));
                out.push_str("</li>\n");
            }
            out.push_str(&format!("</{tag}>\n"));
            out
        }
        Kind::Image | Kind::Video | Kind::Audio => {
            let mut out = format!(
                "<figure{}>",
                if e.boolean("has_spoiler") {
                    " class=\"tmark-spoiler\" tabindex=\"0\""
                } else {
                    ""
                }
            );
            let src = html_escape(&e.text);
            match e.kind {
                Kind::Image => out.push_str(&format!("<img src=\"{src}\" alt=\"\">")),
                Kind::Video => out.push_str(&format!(
                    "<video controls src=\"{src}\" poster=\"{}\"{}></video>",
                    html_escape(e.string("preview")),
                    if e.boolean("loop") { " loop" } else { "" }
                )),
                _ => out.push_str(&format!("<audio controls src=\"{src}\"></audio>")),
            }
            out.push_str(&html_caption(e.caption()));
            out.push_str("</figure>\n");
            out
        }
        Kind::Collage | Kind::Slideshow => format!(
            "<figure class=\"{}\">\n{}{}</figure>\n",
            if e.kind == Kind::Collage {
                "collage"
            } else {
                "slideshow"
            },
            html_blocks(children),
            html_caption(e.caption())
        ),
        Kind::Details => format!(
            "<details{}>\n<summary>{}</summary>\n{}</details>\n",
            if e.boolean("open") { " open" } else { "" },
            html_rich(e.nodes("summary")),
            html_blocks(children)
        ),
        Kind::Table => {
            let classes = [("bordered", "tmark-bordered"), ("striped", "tmark-striped")]
                .iter()
                .filter(|(key, _)| e.boolean(key))
                .map(|(_, class)| *class)
                .collect::<Vec<_>>()
                .join(" ");
            let mut out = format!(
                "<table{}>\n",
                if classes.is_empty() {
                    String::new()
                } else {
                    format!(" class=\"{classes}\"")
                }
            );
            if !e.nodes("caption").is_empty() {
                out.push_str(&format!(
                    "<caption>{}</caption>\n",
                    html_rich(e.nodes("caption"))
                ));
            }
            for row in children.iter().filter_map(element) {
                out.push_str("<tr>");
                for cell in row.children.iter().filter_map(element) {
                    let tag = if cell.boolean("header") { "th" } else { "td" };
                    out.push_str(&format!("<{tag}"));
                    for key in ["align", "colspan", "rowspan"] {
                        if cell.field(key).is_some() {
                            out.push_str(&format!(" {key}=\"{}\"", html_escape(cell.string(key))));
                        }
                    }
                    out.push_str(&format!(">{}</{tag}>", html_rich(&cell.children)));
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</table>\n");
            out
        }
        _ => format!("<pre><code>{}</code></pre>\n", html_escape(&fallback(node))),
    }
}
fn html_inline(node: &Node) -> String {
    if let Node::Text(text) = node {
        return html_escape(text).replace('\n', "<br>\n");
    }
    let Some(e) = element(node) else {
        return format!("<code>{}</code>", html_escape(&fallback(node)));
    };
    let rich = || html_rich(&e.children);
    match e.kind {
        Kind::Link => format!("<a href=\"{}\">{}</a>", html_escape(e.string("href")), rich()),
        Kind::AnchorLink | Kind::ReferenceLink => format!("<a href=\"#{}\">{}</a>", html_escape(e.string("name")), rich()),
        Kind::Reference => format!("<span id=\"{}\">{}</span>", html_escape(e.string("name")), rich()),
        Kind::Bold | Kind::Italic | Kind::Marked | Kind::Underline | Kind::Strikethrough | Kind::Subscript | Kind::Superscript => {
            let tag = match e.kind { Kind::Bold => "strong", Kind::Italic => "em", Kind::Marked => "mark", Kind::Underline => "u", Kind::Strikethrough => "s", Kind::Subscript => "sub", _ => "sup" };
            format!("<{tag}>{}</{tag}>", rich())
        }
        Kind::Spoiler => format!("<span class=\"spoiler\">{}</span>", rich()),
        Kind::Code => format!("<code>{}</code>", html_escape(&e.text)),
        Kind::Icon => format!("<img src=\"{}\" alt=\"{}\" style=\"height: 1em; width: auto; vertical-align: middle;\">", html_escape(&e.text), html_escape(e.string("alt"))),
        Kind::DateTime => date(e).map(|value| format!("<time data-unix=\"{}\" data-timezone=\"{}\">{}</time>", e.text, html_escape(e.string("timezone")), html_escape(&value))).unwrap_or_else(|| format!("<code>{}</code>", html_escape(&fallback(node)))),
        _ => format!("<code>{}</code>", html_escape(&fallback(node))),
    }
}

pub fn render_markdown(node: &Node, options: MarkdownOptions) -> Result<String, crate::Error> {
    validate_root(node, RenderMode::Markdown(options))?;
    Ok(
        if let Some(e) = element(node).filter(|e| e.kind == Kind::Document) {
            md_blocks(&e.children, options)
        } else {
            md_block(node, options)
        }
        .trim_end_matches('\n')
        .into(),
    )
}
pub fn to_markdown(content: &[Node], options: MarkdownOptions) -> Result<String, crate::Error> {
    for node in content {
        validate(node, true, RenderMode::Markdown(options))?;
    }
    Ok(md_blocks(content, options).trim_end_matches('\n').into())
}
fn md_blocks(nodes: &[Node], o: MarkdownOptions) -> String {
    let mut out = String::new();
    for (i, node) in nodes.iter().enumerate() {
        if i > 0 {
            out.push('\n');
            if is_kind(node, Kind::List) && is_kind(&nodes[i - 1], Kind::List) {
                out.push_str(if o.html_less {
                    "---\n\n"
                } else {
                    "<!-- -->\n\n"
                });
            }
        }
        out.push_str(&md_block(node, o));
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}
fn md_rich(nodes: &[Node], o: MarkdownOptions) -> String {
    nodes.iter().map(|n| md_inline(n, o)).collect()
}
fn md_caption(c: Option<&Caption>, o: MarkdownOptions) -> String {
    let Some(c) = c else {
        return String::new();
    };
    let mut out = format!(
        "\n{}",
        wrapped(md_rich(&caption_nodes(&c.text), o), "_", "_")
    );
    if !c.credit.is_empty() {
        if !c.text.is_empty() {
            out.push_str("  \n");
        }
        out.push_str(&wrapped(md_rich(&caption_nodes(&c.credit), o), "_", "_"));
    }
    out.push('\n');
    out
}
fn md_block(node: &Node, o: MarkdownOptions) -> String {
    let Some(e) = element(node) else {
        return preformatted(&fallback(node), "");
    };
    match e.kind {
        Kind::Paragraph => format!("{}\n", md_rich(&e.children, o)),
        Kind::Header => format!("{} {}\n", "#".repeat(level(e)), md_rich(&e.children, o)),
        Kind::Blockquote | Kind::PullQuote => {
            let mut value = if e.kind == Kind::PullQuote {
                format!("{}\n", md_rich(&e.children, o))
            } else {
                md_blocks(&e.children, o)
            };
            if !e.nodes("credit").is_empty() {
                if !value.is_empty() {
                    value.push('\n');
                }
                value.push_str(&format!("— {}\n", md_rich(e.nodes("credit"), o)));
            }
            value
                .trim_end_matches('\n')
                .split('\n')
                .map(|l| format!("> {l}\n"))
                .collect()
        }
        Kind::Divider => "---\n".into(),
        Kind::Preformatted => preformatted(&e.text, e.string("language")),
        Kind::MathBlock => preformatted(&e.text, ""),
        Kind::Anchor => {
            if o.html_less {
                String::new()
            } else {
                format!("<a id=\"{}\"></a>\n", html_attr(&e.text))
            }
        }
        Kind::List => {
            let mut out = String::new();
            let mut next = 1_i128;
            for item in e.children.iter().filter_map(element) {
                if item.field("order").is_some() {
                    next = i128::from(number(item, "order"));
                }
                let marker = if ordered(e) {
                    let m = format!("{next}. ");
                    next += 1;
                    m
                } else {
                    "- ".into()
                };
                let prefix = format!(
                    "{marker}{}",
                    if item.field("checked").is_some() {
                        if item.boolean("checked") {
                            "[x] "
                        } else {
                            "[ ] "
                        }
                    } else {
                        ""
                    }
                );
                let value = md_blocks(&item.children, o);
                for (i, line) in value.trim_end_matches('\n').split('\n').enumerate() {
                    if i == 0 {
                        out.push_str(&prefix);
                    } else if !line.is_empty() {
                        out.push_str(&" ".repeat(marker.len()));
                    }
                    out.push_str(line);
                    out.push('\n');
                }
            }
            out
        }
        Kind::Image | Kind::Video | Kind::Audio => {
            if e.boolean("has_spoiler") && !o.html_less {
                return html_block(node);
            }
            let label = match e.kind {
                Kind::Image => "![]",
                Kind::Video => "[video]",
                _ => "[audio]",
            };
            format!(
                "{label}({})\n{}",
                url_escape(&e.text),
                md_caption(e.caption(), o)
            )
        }
        Kind::Collage | Kind::Slideshow => {
            let mut out: String = e
                .children
                .iter()
                .map(|n| format!("{}\n", md_block(n, o)))
                .collect();
            out.push_str(&md_caption(e.caption(), o));
            out
        }
        Kind::Details => {
            if o.html_less {
                return format!(
                    "{}{}",
                    if e.nodes("summary").is_empty() {
                        String::new()
                    } else {
                        format!(
                            "{}\n\n",
                            wrapped(md_rich(e.nodes("summary"), o), "**", "**")
                        )
                    },
                    md_blocks(&e.children, o)
                );
            }
            format!(
                "<details{}>\n<summary>{}</summary>\n\n{}\n</details>\n",
                if e.boolean("open") { " open" } else { "" },
                html_rich(e.nodes("summary")),
                md_blocks(&e.children, o)
            )
        }
        Kind::Table => md_table(node, e, o),
        _ => preformatted(&fallback(node), ""),
    }
}
fn md_table(node: &Node, e: &Element, o: MarkdownOptions) -> String {
    let rows: Vec<_> = e.children.iter().filter_map(element).collect();
    if !o.html_less
        && rows.iter().any(|r| {
            r.children
                .iter()
                .filter_map(element)
                .any(|c| number(c, "colspan") > 1 || number(c, "rowspan") > 1)
        })
    {
        return html_block(node);
    }
    let mut out = String::new();
    if !e.nodes("caption").is_empty() {
        out.push_str(&format!("{}\n\n", md_rich(e.nodes("caption"), o)));
    }
    let columns = rows.iter().map(|r| r.children.len()).max().unwrap_or(0);
    if columns == 0 {
        return out;
    }
    let first: Vec<_> = rows[0].children.iter().filter_map(element).collect();
    let has_header = first.iter().any(|c| c.boolean("header"));
    let row = |cells: &[&Element]| {
        let mut value = String::from("|");
        for i in 0..columns {
            let text = cells
                .get(i)
                .map(|c| md_rich(&c.children, o))
                .unwrap_or_default();
            let line_break = if o.html_less { " " } else { "<br>" };
            let text = text.replace("  \n", line_break).replace('\n', line_break);
            value.push_str(&format!(" {} |", text.trim().replace('|', "\\|")));
        }
        value.push('\n');
        value
    };
    out.push_str(&row(if has_header { &first } else { &[] }));
    out.push('|');
    for i in 0..columns {
        out.push_str(
            match first.get(i).map(|c| c.string("align")).unwrap_or("") {
                "left" => " :--- |",
                "center" => " :---: |",
                "right" => " ---: |",
                _ => " --- |",
            },
        );
    }
    out.push('\n');
    for r in rows.iter().skip(usize::from(has_header)) {
        out.push_str(&row(&r
            .children
            .iter()
            .filter_map(element)
            .collect::<Vec<_>>()));
    }
    out
}
fn md_inline(node: &Node, o: MarkdownOptions) -> String {
    if let Node::Text(text) = node {
        return markdown_escape(text);
    }
    let Some(e) = element(node) else {
        return inline_code(&fallback(node), o.html_less);
    };
    let rich = || md_rich(&e.children, o);
    match e.kind {
        Kind::Link => format!("[{}]({})", rich(), url_escape(e.string("href"))),
        Kind::AnchorLink | Kind::ReferenceLink => {
            if o.html_less {
                rich()
            } else {
                format!("[{}](#{})", rich(), url_escape(e.string("name")))
            }
        }
        Kind::Reference => {
            if o.html_less {
                rich()
            } else {
                format!(
                    "<span id=\"{}\">{}</span>",
                    html_attr(e.string("name")),
                    rich()
                )
            }
        }
        Kind::Bold => wrapped(rich(), "**", "**"),
        Kind::Italic => wrapped(rich(), "*", "*"),
        Kind::Strikethrough => wrapped(rich(), "~~", "~~"),
        Kind::Marked | Kind::Underline | Kind::Subscript | Kind::Superscript | Kind::Spoiler => {
            if o.html_less {
                return rich();
            }
            let tag = match e.kind {
                Kind::Marked => "mark",
                Kind::Underline => "u",
                Kind::Subscript => "sub",
                Kind::Superscript => "sup",
                _ => "span",
            };
            format!(
                "<{tag}{}>{}</{tag}>",
                if e.kind == Kind::Spoiler {
                    " class=\"spoiler\""
                } else {
                    ""
                },
                rich()
            )
        }
        Kind::Code | Kind::Math => inline_code(&e.text, o.html_less),
        Kind::Icon => format!(
            "![{}]({})",
            markdown_escape(e.string("alt")),
            url_escape(&e.text)
        ),
        Kind::DateTime => date(e)
            .map(|d| markdown_escape(&d))
            .unwrap_or_else(|| inline_code(&fallback(node), o.html_less)),
        _ => inline_code(&fallback(node), o.html_less),
    }
}

pub fn render_terminal(node: &Node, options: TerminalOptions) -> Result<String, crate::Error> {
    validate_root(node, RenderMode::Terminal(options))?;
    Ok(
        if let Some(e) = element(node).filter(|e| e.kind == Kind::Document) {
            Terminal::new(options).blocks(&e.children)
        } else {
            Terminal::new(options).blocks(std::slice::from_ref(node))
        },
    )
}
pub fn to_terminal(content: &[Node], options: TerminalOptions) -> Result<String, crate::Error> {
    for node in content {
        validate(node, true, RenderMode::Terminal(options))?;
    }
    Ok(Terminal::new(options).blocks(content))
}
fn go_float(value: f64) -> String {
    if value == 0.0 {
        return value.to_string();
    }
    let exponent = value.abs().log10().floor() as i32;
    if (-4..6).contains(&exponent) {
        return value.to_string();
    }
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    format!("{mantissa}e{exponent:+03}")
}
fn terminal_text(text: &str) -> String {
    text.chars()
        .filter_map(|c| match c {
            '\n' => Some("\n".into()),
            '\t' => Some("    ".into()),
            c if c.is_control() => None,
            c => Some(c.to_string()),
        })
        .collect()
}
fn indent(value: &str, first: &str, rest: &str) -> String {
    value
        .split('\n')
        .enumerate()
        .map(|(i, line)| format!("{}{line}", if i == 0 { first } else { rest }))
        .collect::<Vec<_>>()
        .join("\n")
}
struct Terminal {
    options: TerminalOptions,
    style: String,
}
impl Terminal {
    fn new(options: TerminalOptions) -> Self {
        Self {
            options,
            style: String::new(),
        }
    }
    fn rich(&mut self, nodes: &[Node]) -> String {
        nodes.iter().map(|n| self.inline(n)).collect()
    }
    fn styled(&mut self, style: &str, nodes: &[Node]) -> String {
        if self.options.plain_text || nodes.is_empty() {
            return self.rich(nodes);
        }
        let previous = self.style.clone();
        let opening = format!("\x1b[{style}m");
        self.style.push_str(&opening);
        let value = self.rich(nodes);
        self.style = previous;
        format!("{opening}{value}\x1b[0m{}", self.style)
    }
    fn blocks(&mut self, nodes: &[Node]) -> String {
        nodes
            .iter()
            .map(|n| {
                Terminal::new(self.options)
                    .block(n)
                    .trim_end_matches('\n')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    fn caption(&mut self, c: Option<&Caption>) -> String {
        let Some(c) = c else {
            return String::new();
        };
        let mut out = String::new();
        if !c.text.is_empty() {
            out.push_str(&format!(
                "\n{}",
                self.styled("2;3", &caption_nodes(&c.text))
            ));
        }
        if !c.credit.is_empty() {
            out.push_str(&format!(
                "\n— {}",
                self.styled("2", &caption_nodes(&c.credit))
            ));
        }
        out
    }
    fn block(&mut self, node: &Node) -> String {
        let Some(e) = element(node) else {
            return self.code(&fallback(node), None);
        };
        match e.kind {
            Kind::Paragraph => self.rich(&e.children),
            Kind::Header => format!(
                "{} {}",
                "#".repeat(level(e)),
                self.styled("1;36", &e.children)
            ),
            Kind::Divider => "────────────────────────────────".into(),
            Kind::Anchor => String::new(),
            Kind::Blockquote | Kind::PullQuote => {
                let mut value = if e.kind == Kind::PullQuote {
                    self.rich(&e.children)
                } else {
                    self.blocks(&e.children)
                };
                if !e.nodes("credit").is_empty() {
                    value.push_str(&format!("\n\n— {}", self.rich(e.nodes("credit"))));
                }
                indent(&value, "│ ", "│ ")
            }
            Kind::List => {
                let mut items = Vec::new();
                let mut next = 1_i128;
                for item in e.children.iter().filter_map(element) {
                    if item.field("order").is_some() {
                        next = i128::from(number(item, "order"));
                    }
                    let mut marker = if ordered(e) {
                        let marker = format!("{next}. ");
                        next += 1;
                        marker
                    } else {
                        "• ".into()
                    };
                    if item.field("checked").is_some() {
                        marker.push_str(if item.boolean("checked") {
                            "[x] "
                        } else {
                            "[ ] "
                        });
                    }
                    items.push(indent(
                        &self.blocks(&item.children),
                        &marker,
                        &" ".repeat(marker.chars().count()),
                    ));
                }
                items.join("\n")
            }
            Kind::Preformatted => {
                self.code(&e.text, e.field("language").map(|_| e.string("language")))
            }
            Kind::MathBlock => format!("math:\n{}", indent(&terminal_text(&e.text), "  ", "  ")),
            Kind::Details => {
                let mut out = format!("▾ {}", self.styled("1", e.nodes("summary")));
                let body = self.blocks(&e.children);
                if !body.is_empty() {
                    out.push_str(&format!("\n{}", indent(&body, "  ", "  ")));
                }
                out
            }
            Kind::Image | Kind::Video | Kind::Audio => {
                let kind = match e.kind {
                    Kind::Image => "image",
                    Kind::Video => "video",
                    _ => "audio",
                };
                if e.boolean("has_spoiler") && !self.options.reveal_spoilers {
                    return format!("[spoiler: {kind}]");
                }
                format!(
                    "[{kind}] {}{}",
                    terminal_text(&e.text),
                    self.caption(e.caption())
                )
            }
            Kind::Map => {
                let lat = e.string("lat").parse::<f64>().unwrap_or(0.0);
                let lon = e.string("lon").parse::<f64>().unwrap_or(0.0);
                format!(
                    "[map] {lat:.6}, {lon:.6}\nhttps://www.openstreetmap.org/?mlat={}&mlon={}{}",
                    go_float(lat),
                    go_float(lon),
                    self.caption(e.caption())
                )
            }
            Kind::Collage | Kind::Slideshow => {
                let mut out = format!(
                    "[{}]",
                    if e.kind == Kind::Collage {
                        "collage"
                    } else {
                        "slideshow"
                    }
                );
                for (i, node) in e.children.iter().enumerate() {
                    let prefix = format!("{}. ", i + 1);
                    out.push_str(&format!(
                        "\n{}",
                        indent(
                            &Terminal::new(self.options).block(node),
                            &prefix,
                            &" ".repeat(prefix.len())
                        )
                    ));
                }
                out.push_str(&self.caption(e.caption()));
                out
            }
            Kind::Table => self.table(e),
            _ => self.code(&fallback(node), None),
        }
    }
    fn code(&self, text: &str, language: Option<&str>) -> String {
        let text = terminal_text(text);
        format!(
            "┌─ code{}\n{}\n└─",
            language
                .map(|l| format!(" {}", terminal_text(l)))
                .unwrap_or_default(),
            indent(text.strip_suffix('\n').unwrap_or(&text), "│ ", "│ ")
        )
    }
    fn table(&mut self, e: &Element) -> String {
        let rows: Vec<_> = e.children.iter().filter_map(element).collect();
        let mut out = self.styled("1", e.nodes("caption"));
        let mut labels = Vec::new();
        if rows.len() > 1
            && rows[0]
                .children
                .iter()
                .filter_map(element)
                .all(|c| c.boolean("header"))
            && !rows.iter().any(|r| {
                r.children
                    .iter()
                    .filter_map(element)
                    .any(|c| number(c, "colspan") > 1 || number(c, "rowspan") > 1)
            })
        {
            let mut plain = Terminal::new(TerminalOptions {
                plain_text: true,
                ..self.options
            });
            labels = rows[0]
                .children
                .iter()
                .filter_map(element)
                .map(|c| {
                    plain
                        .rich(&c.children)
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect();
        }
        for (i, row) in rows
            .iter()
            .skip(usize::from(!labels.is_empty()))
            .enumerate()
        {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("┌─ row {}", i + 1));
            for (j, cell) in row.children.iter().filter_map(element).enumerate() {
                let label = labels
                    .get(j)
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .unwrap_or_else(|| (j + 1).to_string());
                out.push_str(&format!("\n│ {label}"));
                for (key, tag) in [("colspan", "cols"), ("rowspan", "rows")] {
                    if number(cell, key) > 1 {
                        out.push_str(&format!(" [{tag}={}]", number(cell, key)));
                    }
                }
                let value = if cell.boolean("header") {
                    self.styled("1", &cell.children)
                } else {
                    self.rich(&cell.children)
                };
                out.push_str(&format!(": {}", indent(&value, "", "│    ")));
            }
            out.push_str("\n└─");
        }
        out
    }
    fn inline(&mut self, node: &Node) -> String {
        if let Node::Text(text) = node {
            return terminal_text(text);
        }
        let Some(e) = element(node) else {
            return format!("`{}`", terminal_text(&fallback(node)));
        };
        match e.kind {
            Kind::Bold => self.styled("1", &e.children),
            Kind::Italic => self.styled("3", &e.children),
            Kind::Underline => self.styled("4", &e.children),
            Kind::Strikethrough => self.styled("9", &e.children),
            Kind::Marked => self.styled("30;43", &e.children),
            Kind::Spoiler => {
                if self.options.reveal_spoilers {
                    self.rich(&e.children)
                } else {
                    "[spoiler]".into()
                }
            }
            Kind::Subscript => format!("_({})", self.rich(&e.children)),
            Kind::Superscript => format!("^({})", self.rich(&e.children)),
            Kind::Link => format!(
                "{} <{}>",
                self.styled("4;36", &e.children),
                terminal_text(e.string("href"))
            ),
            Kind::AnchorLink => format!(
                "{} [#{}]",
                self.rich(&e.children),
                terminal_text(e.string("name"))
            ),
            Kind::Reference => self.rich(&e.children),
            Kind::ReferenceLink => format!(
                "{} [ref: {}]",
                self.rich(&e.children),
                terminal_text(e.string("name"))
            ),
            Kind::Code => format!("`{}`", terminal_text(&e.text)),
            Kind::Math => format!("⟦{}⟧", terminal_text(&e.text)),
            Kind::Icon => format!(
                "[{}]",
                if e.field("alt").is_some() {
                    terminal_text(e.string("alt"))
                } else {
                    format!("icon: {}", terminal_text(&e.text))
                }
            ),
            Kind::DateTime => {
                date(e).unwrap_or_else(|| format!("`{}`", terminal_text(&fallback(node))))
            }
            _ => format!("`{}`", terminal_text(&fallback(node))),
        }
    }
}
