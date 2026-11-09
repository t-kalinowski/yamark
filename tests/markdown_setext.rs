use std::path::Path;

use yamark::core::document::FormatOptions;
use yamark::workspace::format_source_for_path;

#[test]
fn setext_hash_scanner_handles_long_brace_lines() {
    for count in [32_000, 128_000] {
        let prefix = "{ ".repeat(count);
        let source = format!("{prefix}Title # {{#id}}\n---\n");
        let expected = format!("## {prefix}Title \\# {{#id}}\n");
        for input in [source, expected.clone()] {
            let output = format_source_for_path(
                Path::new("input.md"),
                input,
                FormatOptions::default(),
                None,
            )
            .unwrap()
            .output;
            assert_eq!(output, expected);
        }
    }
}

#[test]
fn setext_indentation_keeps_bom_and_line_endings() {
    for newline in ["\n", "\r\n", "\r"] {
        for bom in ["", "\u{feff}"] {
            let source = format!("{bom}   Heading{newline}---{newline}");
            let expected = format!("{bom}## Heading{newline}");
            for input in [source, expected.clone()] {
                let output = format_source_for_path(
                    Path::new("input.md"),
                    input,
                    FormatOptions::default(),
                    None,
                )
                .unwrap()
                .output;
                assert_eq!(output, expected);
            }
        }
    }
}
