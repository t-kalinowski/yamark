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
        "Before `first  \n  {{ value }} second` after _outside_.\n",
        "- {{ render_item() }}\n  Follow-up prose.\n",
        "> {{ render_item() }}\n> Follow-up prose.\n",
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
            "Before <kbd>[x](  url  )</kbd> after.\n",
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
                        let target = if wrap == "none" && first.is_empty() {
                            "[real](  target  )"
                        } else {
                            "[real]( target )"
                        };
                        let expected = format!(
                            "{first}Béfore {opening}{literal}{closing} after {outside} {target}.\n"
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

#[test]
fn inline_pipeline_normalizes_gaps_after_backslashes() {
    for gap in [" ", "   ", "\t", "\u{000c}", " \t\u{000c} "] {
        for count in 1..=3 {
            let slashes = "\\".repeat(count);
            let literal = format!("`[x](  url  ) {slashes}{gap}`");
            for prefix in ["", "- ", "> "] {
                let source = format!(
                    "{prefix}first{slashes}{gap}second {literal} _outside_ [real](  target  ).\n"
                );
                for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
                    for canonical in [false, true] {
                        let gap = if wrap == "none" && prefix.is_empty() {
                            gap
                        } else {
                            " "
                        };
                        let outside = if canonical { "*outside*" } else { "_outside_" };
                        let expected = format!(
                            "{prefix}first{slashes}{gap}second {literal} {outside} [real](target).\n"
                        );
                        assert_format(&source, &expected, wrap, canonical);
                    }
                }
            }
        }
    }
}

#[test]
fn inline_pipeline_keeps_unsupported_whitespace_out_of_reflow() {
    // Rust's ASCII whitespace excludes vertical tabs, as does main's reflow.
    // A backslash cannot turn this whitespace into an escaped text fragment.
    for whitespace in ['\u{000b}', '\u{00a0}', '\u{2003}'] {
        for slashes in ["", "\\", "\\\\"] {
            let source = format!("first{slashes}{whitespace}second _outside_ [real](  url  ).\n");
            for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
                assert_format(&source, &source, wrap, true);
            }
        }
        let source = format!("`first{whitespace}second` _outside_ [real](  url  ).\n");
        let expected = format!("`first{whitespace}second` *outside* [real](url).\n");
        assert_format(&source, &expected, "paragraph", true);
    }
}

#[test]
fn inline_pipeline_normalizes_list_separators_without_trimming_literals() {
    for (first, continuation) in [("- ", "  "), ("1. ", "   "), ("- [x] ", "      ")] {
        for newline in ["\n", "\r\n", "\r"] {
            let literal = format!("`first  {newline}{continuation}  second [x]( url )`");
            for blank in ["  ", "\t "] {
                let source = format!(
                    "{first}Before {literal} after _outside_ [real](  target  ).{newline}\
                     {blank}{newline}{continuation}Next paragraph.{newline}"
                );
                let expected = format!(
                    "{first}Before {literal} after *outside* [real](target).{newline}\
                     {newline}{continuation}Next paragraph.{newline}"
                );
                for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
                    assert_format(&source, &expected, wrap, true);
                }
            }
        }
    }
    for (source, expected) in [
        (
            "> - first `a  b`\n>   \n>   second [real](  target  ).\n",
            "> - first `a  b`\n>\n>   second [real](target).\n",
        ),
        (
            "- first\n \u{00a0} \t\n  second\n",
            "- first\n \u{00a0}\n  second\n",
        ),
    ] {
        assert_format(source, expected, "paragraph", true);
    }
}

#[test]
fn inline_pipeline_preserves_literals_in_recursive_footnotes() {
    for literal in [
        "`first  \n  second\\\nthird [x]( url )`",
        "$first\t\n  second [x]( url )$",
        "`first  \n  second {{< include file >}}`",
    ] {
        for newline in ["\n", "\r\n", "\r"] {
            let literal = literal.replace('\n', &format!("{newline}    "));
            let paragraph = format!("Before {literal} after _outside_ [real](  target  ).");
            let formatted = format!("Before {literal} after *outside* [real](target).");
            for (source, expected) in [
                (
                    format!("[^note]: {paragraph}\n \t\n    Next paragraph.\n"),
                    format!("[^note]:\n    {formatted}\n\n    Next paragraph.\n"),
                ),
                (
                    format!("[^note]: First paragraph.\n\n    {paragraph}\n"),
                    format!("[^note]:\n    First paragraph.\n\n    {formatted}\n"),
                ),
            ] {
                for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
                    assert_format(&source, &expected, wrap, true);
                }
            }
        }
    }
}

#[test]
fn inline_pipeline_normalizes_nested_output_once() {
    for (opening, closing) in [
        ("::: note\n", ":::\n"),
        (":::: outer\n::: inner\n", ":::\n::::\n"),
        ("```markdown\n", "```\n"),
    ] {
        let source = format!(
            "{opening}Before `first  \n  second [x]( url )` after _outside_.\n \t\n\
             Next paragraph.\t\n{closing}"
        );
        let expected = format!(
            "{opening}Before `first  \n  second [x]( url )` after *outside*.\n\n\
             Next paragraph.\n{closing}"
        );
        for wrap in ["none", "paragraph", "sentence", "80", "sentence:80"] {
            assert_format(&source, &expected, wrap, true);
            // The existing block parser does not support tabbed quote prefixes.
            let quoted_source = source
                .replace("\n \t\n", "\n  \n")
                .split_inclusive('\n')
                .map(|line| format!("> {line}"))
                .collect::<String>();
            let quoted_expected = expected
                .split_inclusive('\n')
                .map(|line| {
                    if line == "\n" {
                        ">\n".to_owned()
                    } else {
                        format!("> {line}")
                    }
                })
                .collect::<String>();
            assert_format(&quoted_source, &quoted_expected, wrap, true);
        }
    }
}

#[test]
fn inline_pipeline_accepts_definition_continuation_indentation() {
    for prefix in [": ", " : ", "  : ", "   : ", "   ~ "] {
        let source = format!("Term\n{prefix}Before\n    after _outside_ [real](  target  ).\n");
        let expected = format!("Term\n{prefix}Before after *outside* [real](target).\n");
        for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
            assert_format(&source, &expected, wrap, true);
        }
    }
    for (prefix, authored_indent, emitted_indent) in [
        (": ", "    ", "    "),
        (" : ", "    ", "    "),
        ("  : ", "    ", "    "),
        ("   : ", "    ", "     "),
        ("   ~ ", "    ", "     "),
        ("   : ", "       ", "       "),
    ] {
        for (opening, closing) in [
            ("`first  ", "second [x]( url )`"),
            ("$first\\", "second [x]( url )$"),
            ("`first\t", "second {{< include file >}}`"),
        ] {
            let source = format!(
                "Term\n{prefix}Before {opening}\n{authored_indent}{closing} after _outside_ [real](  target  ).\n"
            );
            let expected = format!(
                "Term\n{prefix}Before {opening}\n{emitted_indent}{closing} after *outside* [real](target).\n"
            );
            for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
                assert_format(&source, &expected, wrap, true);
            }
        }
    }
    for wrap in ["28", "sentence:28"] {
        assert_format(
            "Term\n   : Before\n    after _outside_ [real](  target  ).\n",
            "Term\n   : Before after *outside*\n     [real](target).\n",
            wrap,
            true,
        );
    }
}

#[test]
fn inline_pipeline_accepts_list_literal_continuation_indentation() {
    for (prefix, authored_indent, emitted_indent) in [
        ("1. ", " ", "   "),
        ("10. ", "  ", "    "),
        ("100. ", "    ", "     "),
        ("1000) ", "    ", "      "),
        ("- [x] ", "    ", "      "),
        ("  100. ", "    ", "       "),
        ("100. ", "         ", "         "),
    ] {
        for (opening, closing) in [
            ("`first  ", "second [x]( url )`"),
            ("$first\\", "second [x]( url )$"),
            ("`first\t", "second {{< include file >}}`"),
        ] {
            for newline in ["\n", "\r\n", "\r"] {
                let source = format!(
                    "{prefix}Before {opening}{newline}{authored_indent}{closing} after _outside_ [real](  target  ).{newline}"
                );
                let expected = format!(
                    "{prefix}Before {opening}{newline}{emitted_indent}{closing} after *outside* [real](target).{newline}"
                );
                for wrap in ["none", "paragraph", "sentence", "120", "sentence:120"] {
                    assert_format(&source, &expected, wrap, true);
                }
            }
        }
    }
    assert_format(
        "::: note\n100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n:::\n",
        "::: note\n100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n:::\n",
        "sentence",
        true,
    );
    for wrap in ["24", "sentence:24"] {
        assert_format(
            "100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
            "100. Before\n     `first  \n     second [x]( url )`\n     after *outside*\n     [real](target).\n",
            wrap,
            true,
        );
    }
    // Keep the existing boundary for lazy prose.
    for source in [
        "- Before\n second _outside_ [real](  target  ).\n",
        "100. Before\n    second _outside_ [real](  target  ).\n",
        "100. Before `code`\n second _outside_ [real](  target  ).\n",
    ] {
        assert_format(source, source, "sentence", true);
    }
}

#[test]
fn inline_pipeline_review_preserves_angle_canonicalization_boundaries() {
    for angle in [
        "<!-- _disabled_ -->",
        "<!DOCTYPE _disabled_>",
        "<?_disabled_?>",
        "<42 _disabled_>",
    ] {
        for prefix in ["", "- ", "> "] {
            let source = format!("{prefix}Before {angle} after _outside_.\n");
            let expected = format!("{prefix}Before {angle} after *outside*.\n");
            for wrap in ["none", "paragraph", "sentence", "80"] {
                assert_format(&source, &expected, wrap, true);
            }
        }
    }
    assert_format(
        "Before \\<!-- _editable_ --> after _outside_.\n",
        "Before \\<!-- *editable* --> after *outside*.\n",
        "paragraph",
        true,
    );
}

#[test]
fn inline_pipeline_review_normalizes_markup_line_gaps() {
    for (content, expected) in [
        ("first  \nsecond ", "first \\\nsecond "),
        ("first \t\nsecond ", "first\nsecond "),
        ("first\\\nsecond ", "first\\\nsecond "),
        (
            "first  \n`literal  \n  bytes` second ",
            "first \\\n`literal  \n  bytes` second ",
        ),
    ] {
        for (opening, closing) in [("<kbd>", "</kbd>"), ("<kbd><span>", "</span></kbd>")] {
            for (prefix, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
                for newline in ["\n", "\r\n", "\r"] {
                    let content = content.replace('\n', &format!("{newline}{continuation}"));
                    let expected = expected.replace('\n', &format!("{newline}{continuation}"));
                    let source = format!(
                        "{prefix}Before {opening}{content}{closing} after _outside_.{newline}"
                    );
                    let expected = format!(
                        "{prefix}Before {opening}{expected}{closing} after *outside*.{newline}"
                    );
                    for wrap in ["none", "paragraph", "sentence", "160"] {
                        assert_format(&source, &expected, wrap, true);
                    }
                }
            }
        }
    }
}
