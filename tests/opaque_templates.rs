use assert_cmd::Command;

fn format(source: &str, path: &str, wrap: &str, canonical: bool) -> String {
    let mut command = Command::cargo_bin("yamark").unwrap();
    command.args(["format", "--stdin-file-path", path, "--wrap", wrap]);
    if canonical {
        command.arg("--canonical");
    }
    let output = command.write_stdin(source).output().unwrap();
    assert!(output.status.success(), "{source}\n{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

fn assert_format(source: &str, expected: &str, path: &str, wrap: &str, canonical: bool) {
    assert_eq!(format(source, path, wrap, canonical), expected, "{source}");
    assert_eq!(
        format(expected, path, wrap, canonical),
        expected,
        "second pass"
    );
}

#[test]
fn transcripts_are_exact_and_idempotent() {
    for case in [
        include_str!("cases/markdown_opaque_template_placement.case"),
        include_str!("cases/markdown_opaque_template_overlapping.case"),
        include_str!("cases/markdown_opaque_template_backtick.case"),
        include_str!("cases/markdown_opaque_template_comments.case"),
        include_str!("cases/markdown_opaque_template_late_raw.case"),
        include_str!("cases/markdown_opaque_template_none.case"),
        include_str!("cases/markdown_opaque_template_preservation.case"),
        include_str!("cases/markdown_opaque_template_boundaries.case"),
        include_str!("cases/markdown_opaque_template_brackets.case"),
        include_str!("cases/markdown_opaque_template_configured_atomic.case"),
        include_str!("cases/markdown_opaque_template_width.case"),
        include_str!("cases/markdown_opaque_template_contents.case"),
        include_str!("cases/markdown_opaque_template_containers.case"),
        include_str!("cases/markdown_opaque_template_hard_breaks.case"),
        include_str!("cases/markdown_opaque_template_configured.case"),
        include_str!("cases/markdown_template_bare_complex.case"),
        include_str!("cases/markdown_template_bare_narrow.case"),
        include_str!("cases/markdown_template_bare_standalone.case"),
        include_str!("cases/markdown_template_container_continuations.case"),
        include_str!("cases/markdown_template_literal_closer.case"),
        include_str!("cases/markdown_template_multiline_word.case"),
        include_str!("cases/markdown_template_multiline_boundaries.case"),
        include_str!("cases/markdown_template_shortcode_overwide.case"),
        include_str!("cases/markdown_template_shortcode_standalone.case"),
        include_str!("cases/markdown_template_shortcode_words.case"),
        include_str!("cases/markdown_template_stray_closers.case"),
        include_str!("cases/markdown_template_autolink.case"),
        include_str!("cases/markdown_template_html_boundary.case"),
    ] {
        let (args, rest) = case
            .strip_prefix("-- args\n")
            .unwrap()
            .split_once("-- stdin\n")
            .unwrap();
        let (source, rest) = rest.split_once("-- stdout\n").unwrap();
        let (expected, _) = rest.split_once("-- stderr\n").unwrap();
        for input in [source, expected] {
            Command::cargo_bin("yamark")
                .unwrap()
                .args(args.split_whitespace())
                .write_stdin(input)
                .assert()
                .success()
                .stderr("")
                .stdout(expected);
        }
    }
}

#[test]
fn tokens_are_opaque_under_every_wrapping_mode() {
    for token in [
        r#"{# don't   change [x](  url  ) _this_ { #}"#,
        r#"{# "quoted-looking closer #}"#,
        r#"{{ render("keep   this") }}"#,
        r#"{{< raw caption="keep   _this_" >}}"#,
        r#"{{% /greeting.inline %}}"#,
        r#"{{ render("unterminated }}"#,
        r#"{{ render({nested: 1) }}"#,
        r#"{{ render("[x](  url  ) _keep_ `code` $math$") }}"#,
        r#"{{ render("<b>  </b>", "x\ty", "café　東京") }}"#,
        r#"{{ render([x](  url  ), _keep_, **this**) }}"#,
    ] {
        for wrap in [
            "none",
            "paragraph",
            "sentence",
            "240",
            "sentence:240",
            "8",
            "sentence:8",
        ] {
            for canonical in [false, true] {
                let source = format!("Before\n{token}\nafter _word_. Next\nline.\n");
                let word = if canonical { "*word*" } else { "_word_" };
                let expected = match wrap {
                    "none" => format!("Before\n{token}\nafter {word}. Next\nline.\n"),
                    "sentence" | "sentence:240" => {
                        format!("Before {token} after {word}.\nNext line.\n")
                    }
                    "8" | "sentence:8" => format!("Before\n{token}\nafter\n{word}.\nNext\nline.\n"),
                    _ => format!("Before {token} after {word}. Next line.\n"),
                };
                assert_format(&source, &expected, "input.md", wrap, canonical);
            }
        }
    }
}

#[test]
fn explicit_preservation_and_malformed_boundaries_remain_exact() {
    let body = "Before\n{{ render(\"keep   this\") }}\nafter [real](  target  ) _word_.\n";
    for (prefix, suffix) in [
        ("<!-- fmt: skip -->\n", ""),
        ("<!-- fmt: skip file -->\n", ""),
        ("<!-- fmt: off -->\n", "<!-- fmt: on -->\n"),
    ] {
        let source = format!("{prefix}{body}{suffix}");
        assert_format(&source, &source, "input.md", "8", true);
    }
    for token in [
        "{{ missing",
        "{{ good }} and {{ missing",
        "{{ good }} and $$ambiguous$$",
        "{{ good }} and < unsupported",
    ] {
        let source = format!("Before\n{token} after [real](  target  ) _word_.\n");
        for wrap in ["none", "paragraph", "sentence", "8"] {
            assert_format(&source, &source, "input.md", wrap, true);
        }
    }
}

#[test]
fn delimiters_and_directive_scopes_protect_tokens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.md");
    let config = dir.path().join("yamark.toml");
    for (open, close) in [
        ("<<", ">>"),
        ("[[", "]]"),
        ("%%", "%%"),
        ("«", "»"),
        ("~~", "~~"),
        ("`%", "%>"),
    ] {
        for action in ["add_delimiters", "replace_delimiters"] {
            std::fs::write(
                &config,
                format!("[template]\n{action} = [{{open='{open}', close='{close}'}}]\n"),
            )
            .unwrap();
            let token = format!(
                "{open} render(\"{close}\", \"_raw_ [x](  url  )\", {{a: {{b: 1}}}}) {close}"
            );
            for (source, expected) in [
                (
                    format!("Before\n{token}\nafter _word_.\n"),
                    format!("Before {token} after *word*.\n"),
                ),
                (
                    format!("- Before\n  {token}\n  after _word_.\n"),
                    format!("- Before {token} after *word*.\n"),
                ),
                (
                    format!("> Before\n> {token}\n> after _word_.\n"),
                    format!("> Before {token} after *word*.\n"),
                ),
            ] {
                assert_format(&source, &expected, path.to_str().unwrap(), "sentence", true);
            }
        }
    }
    std::fs::write(&config, "[template]\nreplace_delimiters = []\n").unwrap();
    for scope in ["next", "from-here", "file"] {
        let directive = format!("<!-- fmt: template.delimiters \"<<\" \">>\" scope={scope} -->\n");
        let source =
            format!("{directive}Before\n<< render(\">>\", \"keep   this\") >>\nafter _word_.\n");
        let expected =
            format!("{directive}Before << render(\">>\", \"keep   this\") >> after *word*.\n");
        assert_format(&source, &expected, path.to_str().unwrap(), "sentence", true);
    }
    let source = "Before\n<< render(\">>\", \"keep   this\") >>\nafter _word_.\n\n<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\n";
    let expected = "Before << render(\">>\", \"keep   this\") >> after *word*.\n\n<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\n";
    assert_format(source, expected, path.to_str().unwrap(), "sentence", true);
}

#[test]
fn supported_containers_and_fragments_share_the_formatter() {
    for (prefix, continuation, suffix) in [
        ("- ", "  ", ""),
        ("* ", "  ", ""),
        ("1. ", "   ", ""),
        ("> ", "> ", ""),
        ("> > ", "> > ", ""),
        ("> - ", ">   ", ""),
        ("Term\n:   ", "    ", ""),
        ("::: note\n", "", ":::\n"),
        ("```markdown\n", "", "```\n"),
    ] {
        let token = "{{ render(\"keep   this\") }}";
        let source =
            format!("{prefix}Before\n{continuation}{token}\n{continuation}after _word_.\n{suffix}");
        let prefix = match prefix {
            "* " => "- ",
            "Term\n:   " => "Term\n: ",
            _ => prefix,
        };
        let expected = format!("{prefix}Before {token} after *word*.\n{suffix}");
        assert_format(&source, &expected, "input.md", "sentence", true);
    }
}

#[test]
fn configured_tokens_remain_atomic_in_nested_containers_and_link_layout() {
    let source = "<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\n> - Before\n>   << render(\"_raw_ [x](  url  )\") >>\n>   after _word_.\n";
    let expected = "<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\n> - Before << render(\"_raw_ [x](  url  )\") >> after *word*.\n";
    assert_format(source, expected, "input.md", "sentence", true);
    let source = "<!-- fmt: template.delimiters \"[\" \")\" scope=next -->\nBefore [a long title](target) after.\n";
    let expected = "<!-- fmt: template.delimiters \"[\" \")\" scope=next -->\nBefore\n[a long title](target)\nafter.\n";
    assert_format(source, expected, "input.md", "8", true);
}

#[test]
fn late_delimiters_reconsider_raw_semantics_inside_the_expression() {
    let source = "Before\n<< render(\">>\", \"a　b\", \"\\LaTeX\") >>\nafter _word_.\n\n<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\n";
    let expected = "Before << render(\">>\", \"a　b\", \"\\LaTeX\") >> after *word*.\n\n<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\n";
    assert_format(source, expected, "input.md", "sentence", true);
}

#[test]
fn multiline_configured_delimiters_do_not_expand_inline_support() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.md");
    std::fs::write(
        dir.path().join("yamark.toml"),
        "[template]\nreplace_delimiters = [{open=\"<<\\n\", close=\">>\"}]\n",
    )
    .unwrap();
    let source = "Before\n<<\nkeep   this >> after _word_.\n";
    assert_format(source, source, path.to_str().unwrap(), "sentence", true);
}
