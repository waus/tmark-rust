use std::{collections::BTreeMap, fs, path::Path};
use tmark::{
    parse_soft, render_html, render_markdown, render_terminal, MarkdownOptions, Node,
    TerminalOptions,
};

#[test]
fn matches_go_renderer_fixtures() {
    let expected: BTreeMap<String, BTreeMap<String, String>> =
        serde_json::from_str(include_str!("fixtures/render.json")).unwrap();
    for (name, outputs) in expected {
        let source = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures/valid")
                .join(&name),
        )
        .unwrap();
        let node = parse_soft(&source).unwrap();
        let actual = [
            ("html", render_html(&node).unwrap()),
            (
                "md",
                render_markdown(&node, MarkdownOptions::default()).unwrap(),
            ),
            (
                "md_plain",
                render_markdown(&node, MarkdownOptions { html_less: true }).unwrap(),
            ),
            (
                "terminal",
                render_terminal(&node, TerminalOptions::default()).unwrap(),
            ),
            (
                "terminal_plain",
                render_terminal(
                    &node,
                    TerminalOptions {
                        plain_text: true,
                        reveal_spoilers: false,
                    },
                )
                .unwrap(),
            ),
            (
                "terminal_reveal",
                render_terminal(
                    &node,
                    TerminalOptions {
                        plain_text: false,
                        reveal_spoilers: true,
                    },
                )
                .unwrap(),
            ),
            (
                "terminal_plain_reveal",
                render_terminal(
                    &node,
                    TerminalOptions {
                        plain_text: true,
                        reveal_spoilers: true,
                    },
                )
                .unwrap(),
            ),
        ];
        for (mode, output) in actual {
            assert_eq!(output, outputs[mode], "{name}: {mode}");
        }
    }
}

#[test]
fn malformed_unknown_is_an_error() {
    let node = Node::Unknown("{future;".into());
    assert!(render_html(&node).is_err());
    assert!(render_markdown(&node, MarkdownOptions::default()).is_err());
    assert!(render_terminal(&node, TerminalOptions::default()).is_err());
}

#[test]
fn markdown_whitespace_fences_and_escaping() {
    let source = "{document;\n{p;a{b; bold }{i; italic }{s; old }}\n{p;{code; code } {code;`code`} {code;}}\n{p;1. text\n- item\n+ item\n===\n`code` ~~strike~~ ![image](x) &copy;}\n{math-block;\nx = y\n```\n}\n}";
    let node = parse_soft(source).unwrap();
    assert_eq!(render_markdown(&node, MarkdownOptions::default()).unwrap(), "a **bold**  *italic*  ~~old~~ \n\n`  code  ` `` `code` `` <code></code>\n\n1\\. text  \n\\- item  \n\\+ item  \n\\=\\=\\=  \n\\`code\\` \\~\\~strike\\~\\~ \\!\\[image\\](x\\) \\&copy;\n\n````\nx = y\n```\n````");
}

#[test]
fn nested_styles_restore_parent_and_input_controls_are_removed() {
    let node = parse_soft("{p;{b;a{i;b}c}d}").unwrap();
    assert_eq!(
        render_terminal(&node, TerminalOptions::default()).unwrap(),
        "\x1b[1ma\x1b[3mb\x1b[0m\x1b[1mc\x1b[0md"
    );
    let attack = "safe\x1b]52;c;payload\x07\x1b[2J\r\x08\u{009b}31m";
    let node: Node = tmark::Paragraph {
        children: vec![attack.into()],
    }
    .into();
    let output = render_terminal(
        &node,
        TerminalOptions {
            plain_text: true,
            reveal_spoilers: false,
        },
    )
    .unwrap();
    assert!(!output.chars().any(char::is_control));
    assert_eq!(output, "safe]52;c;payload[2J31m");
}

#[test]
fn date_timezone_and_invalid_timezone_fallback() {
    for (unix, timezone, expected) in [
        ("0", "UTC", "1970-01-01 00:00:00 UTC"),
        ("0", "America/New_York", "1969-12-31 19:00:00 EST"),
        ("1625097600", "America/New_York", "2021-06-30 20:00:00 EDT"),
        ("-1", "UTC", "1969-12-31 23:59:59 UTC"),
        ("0", "Unknown/Zone", "`{datetime;#timezone{Unknown/Zone}0}`"),
    ] {
        let source = format!("{{p;{{datetime;#timezone{{{timezone}}}{unix}}}}}");
        let node = parse_soft(&source).unwrap();
        assert_eq!(
            render_terminal(
                &node,
                TerminalOptions {
                    plain_text: true,
                    reveal_spoilers: false
                }
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn unsupported_timezones_use_raw_fallback() {
    for zone in [
        "Local",
        "localtime",
        "posixrules",
        "posix/Europe/Moscow",
        "right/UTC",
        "PST",
        "CST",
        "IST",
        "MSK",
        "GMT+2",
        "GMT+02:00",
        "gmt+2",
        "gmt",
        "utc",
        "utc+2",
        "z",
        "+2:00",
        "+0200",
        "+24:00",
        "-24:00",
        "+03:60",
        "UTC+24",
        "UTC+2:3",
        "UTC+002",
        "UTC+",
        "UTC+2:000",
        "UTC+２",
        "UTC+2 ",
        " UTC+2",
        "Unknown/Zone",
    ] {
        let raw = format!("{{datetime;#timezone{{{zone}}}0}}");
        let node = parse_soft(&format!("{{p;{raw}}}")).unwrap();
        assert_eq!(
            render_html(&node).unwrap(),
            format!("<p><code>{raw}</code></p>\n")
        );
        for html_less in [false, true] {
            assert_eq!(
                render_markdown(&node, MarkdownOptions { html_less }).unwrap(),
                format!("`{raw}`")
            );
        }
        for plain_text in [false, true] {
            assert_eq!(
                render_terminal(
                    &node,
                    TerminalOptions {
                        plain_text,
                        reveal_spoilers: false
                    }
                )
                .unwrap(),
                format!("`{raw}`")
            );
        }
    }
    let node = parse_soft("{p;{datetime;0}}").unwrap();
    assert_eq!(
        render_terminal(&node, TerminalOptions::default()).unwrap(),
        "1970-01-01 00:00:00 UTC"
    );
}

#[test]
fn non_finite_map_coordinates_return_errors() {
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        for (lat, lon) in [(value, 0.0), (0.0, value)] {
            let node = Node::from(tmark::Map {
                lat,
                lon,
                ..Default::default()
            });
            assert!(render_terminal(&node, TerminalOptions::default()).is_err());
        }
    }
}

#[test]
fn ordered_lists_continue_past_i64_max() {
    let node =
        parse_soft("{list;{#order{9223372036854775807}{p;first}}{{p;second}}{#order{1}{p;reset}}}")
            .unwrap();
    let expected = "9223372036854775807. first\n9223372036854775808. second\n1. reset";
    assert_eq!(
        render_markdown(&node, MarkdownOptions::default()).unwrap(),
        expected
    );
    assert_eq!(
        render_terminal(&node, TerminalOptions::default()).unwrap(),
        expected
    );
}

#[test]
fn fixed_timezones_render_in_all_modes() {
    for (zone, expected) in [
        ("", "1970-01-01 00:00:00 UTC"),
        ("UTC", "1970-01-01 00:00:00 UTC"),
        ("GMT", "1970-01-01 00:00:00 UTC"),
        ("Z", "1970-01-01 00:00:00 UTC"),
        ("+03:00", "1970-01-01 03:00:00 UTC+03:00"),
        ("-05:30", "1969-12-31 18:30:00 UTC-05:30"),
        ("UTC+2", "1970-01-01 02:00:00 UTC+02:00"),
        ("UTC+02", "1970-01-01 02:00:00 UTC+02:00"),
        ("UTC-3:30", "1969-12-31 20:30:00 UTC-03:30"),
        ("UTC+05:45", "1970-01-01 05:45:00 UTC+05:45"),
        ("+00:00", "1970-01-01 00:00:00 UTC"),
        ("-00:00", "1970-01-01 00:00:00 UTC"),
        ("UTC-0", "1970-01-01 00:00:00 UTC"),
        ("+23:59", "1970-01-01 23:59:00 UTC+23:59"),
        ("-23:59", "1969-12-31 00:01:00 UTC-23:59"),
        ("UTC+23:59", "1970-01-01 23:59:00 UTC+23:59"),
        ("Etc/GMT+2", "1969-12-31 22:00:00 -02"),
    ] {
        let node: Node = tmark::Paragraph {
            children: vec![tmark::DateTime {
                unix: 0,
                timezone: zone.into(),
            }
            .into()],
        }
        .into();
        assert_eq!(
            render_html(&node).unwrap(),
            format!("<p><time data-unix=\"0\" data-timezone=\"{zone}\">{expected}</time></p>\n"),
            "{zone}"
        );
        for html_less in [false, true] {
            assert_eq!(
                render_markdown(&node, MarkdownOptions { html_less }).unwrap(),
                expected.replace('-', "\\-").replace('+', "\\+"),
                "{zone}"
            );
        }
        for plain_text in [false, true] {
            assert_eq!(
                render_terminal(
                    &node,
                    TerminalOptions {
                        plain_text,
                        reveal_spoilers: false
                    }
                )
                .unwrap(),
                expected,
                "{zone}"
            );
        }
    }
    let node = parse_soft("{p;{datetime;#timezone{UTC+2}1625097600}}").unwrap();
    assert_eq!(
        render_terminal(&node, TerminalOptions::default()).unwrap(),
        "2021-07-01 02:00:00 UTC+02:00"
    );
}
