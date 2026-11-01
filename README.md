# tmark

A Rust library for tmark compatibility level 1. Build typed documents, parse strict or forward-compatible input, serialize canonical tmark, and render HTML, GitHub Markdown, or terminal output.

```toml
[dependencies]
tmark = "0.1"
```

```rust
use tmark::{from_str, render_html, to_string, Bold, Document, Node, Paragraph};

fn main() -> Result<(), tmark::Error> {
    let document = Document {
        title: "Example".into(),
        content: vec![Paragraph {
            children: vec![
                "Hello ".into(),
                Bold { children: vec!["world".into()] }.into(),
            ],
        }.into()],
        ..Default::default()
    };
    let encoded = to_string(&document)?;
    let parsed: Document = from_str(&encoded)?;
    assert_eq!(parsed.title, "Example");
    assert!(render_html(&Node::from(parsed))?.contains("<strong>world</strong>"));
    Ok(())
}
```

`Inline`, `Block`, and `Media` enforce content categories. Concrete arrays use `ListItem`, `TableRow`, and `Cell`. Optional fields retain the distinction between absence and explicit empty or false values. `Element`, `Kind`, and `Value` provide dynamic access when needed; serialization validates dynamic input.

`from_str::<T>` and `to_string` handle typed nodes, strings, booleans, integer and floating-point scalars, and typed sequences. Implement `Tmark` for application-specific values.

`from_str_soft::<T>` enables forward-compatible parsing for typed values and sequences.

`parse` rejects unknown tags. `parse_soft` preserves unknown nodes verbatim while still rejecting malformed known nodes. `parse_bytes` additionally validates UTF-8. Parsing accepts CRLF and enforces canonical numbers, field uniqueness, field order, child types, semantic constraints, and the 16-level depth limit.

`render_html` and `to_html` produce HTML. `render_markdown` and `to_markdown` accept `MarkdownOptions { html_less: true }` for Markdown without HTML fallbacks. `render_terminal` and `to_terminal` accept `TerminalOptions` to select ANSI styling, plain output, and spoiler visibility. Renderers return `Result<String, Error>`.

Run `cargo test` to check local specification fixtures and rendering snapshots against the reference implementation.

MIT licensed.

To publish a release, run `cargo test`, `cargo package`, and `cargo publish --dry-run` from a clean checkout. Authenticate with `cargo login`, then run `cargo publish` when ready.
