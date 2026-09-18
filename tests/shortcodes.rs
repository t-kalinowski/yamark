use assert_cmd::Command;

fn format(source: &str, wrap: &str) -> String {
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args([
            "format",
            "--stdin-file-path",
            "input.md",
            "--canonical",
            "--wrap",
            wrap,
        ])
        .write_stdin(source)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn shortcode_bytes_survive_line_endings_eof_and_nested_documents() {
    for newline in ["\n", "\r\n", "\r"] {
        for wrap in ["none", "paragraph", "sentence", "20"] {
            for token in [
                "{{< call   arg=\"é >}} \\\" still quoted\"  >}}",
                "{{% /call   %}}",
                "{{< call / >}}",
                "{{< data message=\"\n<!-- fmt: off -->\n<!-- fmt: on -->\n\" >}}",
                "{{% call\narg='\\\' %}} still   quoted'  \t\nraw=`\\`\n%}}",
                "{{< missing\nKeep   this.  \t\n\n<!-- fmt: on -->\nStill   opaque.  ",
                "{{% call arg=\"unterminated\n%}}\n<!-- fmt: skip file -->\nKeep   this.  ",
            ] {
                let token = token.replace('\n', newline);
                for prefix in ["", "\u{feff}"] {
                    for suffix in ["", newline] {
                        let source = format!("{prefix}{token}{suffix}");
                        let output = format(&source, wrap);
                        assert_eq!(output, source, "wrap={wrap}");
                        assert_eq!(format(&output, wrap), output, "second pass");
                    }
                }
                for (opening, closing) in [
                    ("::: {.callout-note}", ":::"),
                    ("```markdown", "```"),
                    ("::: {.outer}\n```markdown", "```\n:::"),
                ] {
                    let opening = opening.replace('\n', newline);
                    let closing = closing.replace('\n', newline);
                    let source = format!("{opening}{newline}{token}{newline}{closing}{newline}");
                    let output = format(&source, wrap);
                    assert_eq!(output, source, "wrap={wrap}");
                    assert_eq!(format(&output, wrap), output, "second pass");
                }
            }
        }
    }
}

#[test]
fn shortcode_transcript_outputs_are_idempotent() {
    for case in [
        include_str!("cases/markdown_shortcode_independent_tokens.case"),
        include_str!("cases/markdown_shortcode_multiline_tokens.case"),
        include_str!("cases/markdown_shortcode_directives.case"),
        include_str!("cases/markdown_shortcode_disabled_indented_code.case"),
        include_str!("cases/markdown_shortcode_disabled_html_math.case"),
        include_str!("cases/markdown_shortcode_disabled_raw_blocks.case"),
        include_str!("cases/markdown_shortcode_bom_angle.case"),
        include_str!("cases/markdown_shortcode_bom_percent.case"),
        include_str!("cases/markdown_shortcode_nested_skip_file.case"),
        include_str!("cases/markdown_shortcode_unterminated_token.case"),
        include_str!("cases/markdown_shortcode_unterminated_fragments.case"),
        include_str!("cases/markdown_shortcode_placement_controls.case"),
        include_str!("cases/markdown_shortcode_quoted_closing_delimiter.case"),
        include_str!("cases/markdown_div_shortcode_math.case"),
    ] {
        let args = case
            .split("-- args\n")
            .nth(1)
            .unwrap()
            .split("\n-- stdin")
            .next()
            .unwrap();
        let expected = case
            .split("-- stdout\n")
            .nth(1)
            .unwrap()
            .split("-- stderr\n")
            .next()
            .unwrap();
        let output = Command::cargo_bin("yamark")
            .unwrap()
            .args(args.split_whitespace())
            .write_stdin(expected)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        assert_eq!(output.stdout, expected.as_bytes(), "second pass: {args}");
    }
}
