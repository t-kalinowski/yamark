use assert_cmd::Command;

fn format(source: &str, wrap: &str, canonical: bool) -> String {
    let mut command = Command::cargo_bin("yamark").unwrap();
    command.args(["format", "--stdin-file-path", "input.md", "--wrap", wrap]);
    if canonical {
        command.arg("--canonical");
    }
    let output = command
        .write_stdin(source)
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

fn assert_format(source: &str, expected: &str, wrap: &str, canonical: bool) {
    let output = format(source, wrap, canonical);
    assert_eq!(output, expected, "wrap={wrap}, canonical={canonical}");
    assert_eq!(format(&output, wrap, canonical), output, "second pass");
}

#[test]
fn inline_pipeline_preserves_literals_and_normalizes_real_links() {
    for literal in [
        "`[x](< url >)`",
        "`[x](  url  )`",
        "$[x](  url  )$",
        r"$a \$ _literal_ [x](  url  )$",
        r"`  [x]( url ) \\ _literal_  `",
        "``[x]( url ) ` _literal_``",
        "`[x]( url ) {{< include file >}}`",
        "_with `[x]( url )` inside_",
        r"\text{[x](  url  )}",
    ] {
        for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
            for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
                for canonical in [false, true] {
                    let source = format!(
                        "{first}Before\n{continuation}{literal} after [real](  target  ).\n"
                    );
                    let literal = if canonical && literal.starts_with('_') {
                        literal
                            .replace("_with", "*with")
                            .replace("inside_", "inside*")
                    } else {
                        literal.to_owned()
                    };
                    let separator = if wrap == "none" && first.is_empty() {
                        "\n"
                    } else {
                        " "
                    };
                    let expected =
                        format!("{first}Before{separator}{literal} after [real](target).\n");
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn inline_pipeline_preserves_multiline_literal_bytes_in_containers() {
    for literal in [
        "`first  \n  second [x]( url )`",
        "`first\\\n  second [x]( url )`",
        "$first  \n  second \\$ [x]( url )$",
        "`first\t\n  second {{< include file >}}`",
    ] {
        for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> "), ("> > ", "> > ")] {
            for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
                for canonical in [false, true] {
                    let literal = literal.replace('\n', &format!("\n{continuation}"));
                    let source = format!("{first}Before {literal} after _outside_.\n");
                    let outside = if canonical { "*outside*" } else { "_outside_" };
                    let expected = format!("{first}Before {literal} after {outside}.\n");
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn inline_pipeline_keeps_main_template_policy() {
    for source in [
        "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
        "Before `first  \n  {{ value }} second` after _outside_.\n",
        "- {{ render_item() }}\n  Follow-up prose.\n",
        "> {{ render_item() }}\n> Follow-up prose.\n",
        "Before {{ value }}\nmore prose [real]( url ).\n",
    ] {
        for wrap in ["none", "sentence", "paragraph", "20", "sentence:20"] {
            for canonical in [false, true] {
                assert_format(source, source, wrap, canonical);
            }
        }
    }
}

#[test]
fn inline_pipeline_keeps_existing_markup_and_heading_normalization() {
    for (source, expected) in [
        ("# [hello   world](  url  )\n", "# [hello world]( url )\n"),
        (
            "Before <kbd>[x](  url  )</kbd> after.\n",
            "Before <kbd>[x](url)</kbd> after.\n",
        ),
        (
            "Before ~~[x](  url  )~~ after.\n",
            "Before ~~[x](url)~~ after.\n",
        ),
    ] {
        assert_format(source, expected, "sentence", true);
    }
}

#[test]
fn inline_pipeline_preserves_literal_line_endings() {
    for newline in ["\n", "\r\n", "\r"] {
        for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
            let source = format!(
                "{first}Before `first  \t{newline}{continuation}  second\\{newline}{continuation}third` after.{newline}"
            );
            for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
                assert_format(&source, &source, wrap, true);
            }
        }
    }
}

#[test]
fn inline_pipeline_wraps_around_overwide_literals() {
    for literal in [
        "`[x](  long-link-target  )`",
        "$[x](  long-link-target  )$",
        "`[x](  long-link-target  ) {{< include file >}}`",
        "_with `[x](  long-link-target  )` inside_",
        "~~with `[x](  long-link-target  )` inside~~",
    ] {
        for prefix in ["", "- ", "> "] {
            for wrap in ["16", "sentence:16"] {
                let source = format!("{prefix}Before {literal} after [real](  target  ).\n");
                let output = format(&source, wrap, false);
                assert!(output.contains(literal), "{output:?}");
                assert!(!output.contains("(  target  )"), "{output:?}");
                assert_eq!(format(&output, wrap, false), output, "second pass");
            }
        }
    }
}

#[test]
fn inline_pipeline_canonical_emphasis_skips_literal_delimiters() {
    for literal in ["`_a_ [x]( url )`", "$x + _a_ [x]( url )$"] {
        let source = format!("# _Before {literal} after_\n");
        let expected = format!("# *Before {literal} after*\n");
        assert_format(&source, &expected, "sentence", true);
    }
}

#[test]
fn inline_pipeline_preserves_definition_and_footnote_literals() {
    for source in [
        "Term\n: Before `first  \n      second [x]( url )` after _outside_.\n",
        "[^note]: Before `first  \n    second [x]( url )` after _outside_.\n",
    ] {
        let expected = source.replace("_outside_", "*outside*");
        for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
            assert_format(source, &expected, wrap, true);
        }
    }
}

#[test]
fn inline_pipeline_preserves_multiline_literals_nested_in_markup() {
    for (opening, closing) in [("<kbd>", "</kbd>"), ("<kbd><span>", "</span></kbd>")] {
        for literal in [
            "`first  \n  second [x]( url )`",
            "$first\\\n  second [x]( url )$",
            "`first\t\n  second {{< include file >}}`",
        ] {
            for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
                let literal = literal.replace('\n', &format!("\n{continuation}"));
                let source = format!(
                    "{first}Béfore {opening}{literal}{closing} after _outside_ [real](  target  ).\n"
                );
                for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
                    for canonical in [false, true] {
                        let outside = if canonical { "*outside*" } else { "_outside_" };
                        let expected = format!(
                            "{first}Béfore {opening}{literal}{closing} after {outside} [real](target).\n"
                        );
                        assert_format(&source, &expected, wrap, canonical);
                    }
                }
            }
        }
    }
}

#[test]
fn inline_pipeline_normalizes_editable_form_feeds() {
    for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
        for gap in ["\u{000c}", " \u{000c}\t", "\u{000c}\n"] {
            let gap = gap.replace('\n', &format!("\n{continuation}"));
            let source = format!("{first}first{gap}second `a\u{000c}b` $c\u{000c}d$\n");
            for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
                let expected = if wrap == "none" && first.is_empty() {
                    source.clone()
                } else {
                    format!("{first}first second `a\u{000c}b` $c\u{000c}d$\n")
                };
                for canonical in [false, true] {
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }

    // With wrapping disabled, retain form feeds even at physical line ends.
    // Space and backslash hard breaks still receive their existing normalization.
    for (source, preserved, reflowed) in [
        ("first\u{000c}", "first\u{000c}\n", "first\n"),
        (
            "first \u{000c}\t\nsecond\n",
            "first \u{000c}\nsecond\n",
            "first second\n",
        ),
        (
            "first\u{000c}  \nsecond\n",
            "first\u{000c} \\\nsecond\n",
            "first \\\nsecond\n",
        ),
        (
            "first\u{000c}\\\nsecond\n",
            "first\u{000c}\\\nsecond\n",
            "first\\\nsecond\n",
        ),
    ] {
        assert_format(source, preserved, "none", false);
        for wrap in ["paragraph", "sentence", "80", "sentence:80"] {
            assert_format(source, reflowed, wrap, false);
        }
    }
}

#[test]
fn inline_pipeline_canonical_emphasis_respects_escaped_openers() {
    for (source, expected) in [
        (
            "_before \\` literal_ after `code`\n",
            "*before \\` literal* after `code`\n",
        ),
        (
            "_before \\[ literal_ after ](target).\n",
            "*before \\[ literal* after ](target).\n",
        ),
        (
            "_before \\[ literal_ after ] text.\n",
            "*before \\[ literal* after ] text.\n",
        ),
    ] {
        for prefix in ["", "# ", "- ", "> "] {
            for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
                assert_format(
                    &format!("{prefix}{source}"),
                    &format!("{prefix}{expected}"),
                    wrap,
                    true,
                );
            }
        }
    }
}
