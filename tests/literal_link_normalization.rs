use assert_cmd::Command;

fn assert_format(source: &str, expected: &str, wrap: &str, canonical: bool) {
    let format = |input: &str| {
        let mut command = Command::cargo_bin("yamark").unwrap();
        command.args(["format", "--stdin-file-path", "input.md", "--wrap", wrap]);
        if canonical {
            command.arg("--canonical");
        }
        let output = command.write_stdin(input).output().unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        String::from_utf8(output.stdout).unwrap()
    };
    let output = format(source);
    assert_eq!(output, expected, "wrap={wrap}, canonical={canonical}");
    assert_eq!(format(&output), expected, "second pass: wrap={wrap}");
}

#[test]
fn link_normalization_preserves_recognized_literals() {
    for literal in [
        "`[x](  url  )`",
        "`![x](  image  )`",
        "``[x](  url  ) ` ![y](  image  )``",
        "```[x](  url  ) `` ![y](  image  )```",
        "$[x](  url  )$",
        "$![x](  image  )$",
        r"$a \$ [x](  url  ) ![y](  image  )$",
        "*with `[x](  url  )` inside*",
        "**with $![x](  image  )$ inside**",
        "_with ``![x](  image  )`` inside_",
    ] {
        for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
            for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
                for canonical in [false, true] {
                    let source = format!(
                        "{first}[before](  a  ) ![before](  b  )\n{continuation}Before {literal} after [real](  target  ) ![real](  image  ) _outside_.\n"
                    );
                    let separator = if wrap == "none" && first.is_empty() {
                        "\n"
                    } else {
                        " "
                    };
                    let literal = if canonical && literal.starts_with('_') {
                        literal.replace('_', "*")
                    } else {
                        literal.to_owned()
                    };
                    let outside = if canonical { "*outside*" } else { "_outside_" };
                    let expected = format!(
                        "{first}[before](a) ![before](b){separator}Before {literal} after [real](target) ![real](image) {outside}.\n"
                    );
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn link_normalization_wraps_around_overwide_literals() {
    for literal in [
        "`[x](  long-link-target  )`",
        "$![x](  long-image-target  )$",
    ] {
        for (first, continuation) in [("", ""), ("- ", "  "), ("> ", "> ")] {
            for wrap in ["16", "sentence:16"] {
                for canonical in [false, true] {
                    let source = format!("{first}Before [b](  s  ). {literal} after [r](  t  ).\n");
                    let expected = format!(
                        "{first}Before [b](s).\n{continuation}{literal}\n{continuation}after [r](t).\n"
                    );
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn link_normalization_keeps_escaped_and_unsupported_controls() {
    for (source, expected) in [
        (
            r"Before \`[x](  url  )\` after [r](  t  ).",
            r"Before \`[x](url)\` after [r](t).",
        ),
        (
            r"Before \$![x](  url  )\$ after [r](  t  ).",
            r"Before \$![x](url)\$ after [r](t).",
        ),
        (
            "Before ``[x](  url  )` after [r](  t  ).",
            "Before ``[x](  url  )` after [r](  t  ).",
        ),
        (
            "Before `[x](  url  )`` after [r](  t  ).",
            "Before `[x](  url  )`` after [r](  t  ).",
        ),
        (
            "Before $[x](  url  ) after [r](  t  ).",
            "Before $[x](  url  ) after [r](  t  ).",
        ),
        (
            "Before [x](  two words  ) after [r](  t  ).",
            "Before [x](  two words  ) after [r](  t  ).",
        ),
        (
            r"Before \``[x](  url  )\`` after [r](  t  ).",
            r"Before \``[x](url)\`` after [r](t).",
        ),
        (
            "Before *``[x](  url  )`* after [r](  t  ).",
            "Before *``[x](url)`* after [r](t).",
        ),
        (
            "Before $$[x](  url  )$$ after [r](  t  ).",
            "Before $$[x](url)$$ after [r](t).",
        ),
    ] {
        for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
            for canonical in [false, true] {
                assert_format(
                    &format!("{source}\n"),
                    &format!("{expected}\n"),
                    wrap,
                    canonical,
                );
            }
        }
    }
}

#[test]
fn link_targets_and_attributes_do_not_open_literals() {
    for marker in ['`', '$'] {
        for image in ["", "!"] {
            for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
                for canonical in [false, true] {
                    let source = format!(
                        "Before {image}[a](  {marker}x  ) [b](  target  ) {image}[c](  {marker}y  ).\n"
                    );
                    let expected = format!(
                        "Before {image}[a]({marker}x) [b](target) {image}[c]({marker}y).\n"
                    );
                    assert_format(&source, &expected, wrap, canonical);
                    let source = format!(
                        "Before {image}[a](  x  ){{key=\"{marker}\"}} [b](  target  ) {image}[c](  y  ){{key=\"{marker}\"}}.\n"
                    );
                    let expected = format!(
                        "Before {image}[a](x){{key=\"{marker}\"}} [b](target) {image}[c](y){{key=\"{marker}\"}}.\n"
                    );
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn template_contents_stay_exact_while_surrounding_links_reflow() {
    for source in [
        "Before `[x](  url  ) {{ value }}` after [r](  t  ) _outside_.\n",
        "- Before $![x](  url  )$ {{ value }} after [r](  t  ).\n",
        "> Before `[x](  url  )` {{ value }} after [r](  t  ).\n",
        "<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\nBefore `[x](  url  )` << value >> after [r](  t  ).\n",
    ] {
        for wrap in ["none", "paragraph", "sentence", "16", "sentence:16"] {
            for canonical in [false, true] {
                let expected = if source.starts_with("Before") {
                    // This ordinary paragraph is now eligible. Its code stays
                    // exact while the real link, emphasis and wrapping format.
                    let outside = if canonical { "*outside*" } else { "_outside_" };
                    if matches!(wrap, "16" | "sentence:16") {
                        format!(
                            "Before\n`[x](  url  ) {{{{ value }}}}`\nafter [r](t)\n{outside}.\n"
                        )
                    } else {
                        format!("Before `[x](  url  ) {{{{ value }}}}` after [r](t) {outside}.\n")
                    }
                } else if matches!(wrap, "16" | "sentence:16") {
                    if source.starts_with("- ") {
                        "- Before\n  $![x](  url  )$\n  {{ value }}\n  after [r](t).\n".to_owned()
                    } else if source.starts_with("> ") {
                        "> Before\n> `[x](  url  )`\n> {{ value }}\n> after [r](t).\n".to_owned()
                    } else {
                        "<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\nBefore\n`[x](  url  )`\n<< value >>\nafter [r](t).\n".to_owned()
                    }
                } else {
                    source.replace("[r](  t  )", "[r](t)")
                };
                assert_format(source, &expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn unmatched_backtick_run_keeps_later_literals_and_links_distinct() {
    let source = "Before *```` [raw](  u  ) ``[x](  v  )``* after [r](  t  ).\n";
    let expected = "Before *```` [raw](u) ``[x](  v  )``* after [r](t).\n";
    for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
        for canonical in [false, true] {
            assert_format(source, expected, wrap, canonical);
        }
    }
}

#[test]
fn angle_inputs_preserve_links_during_normalization() {
    for marker in ['`', '$'] {
        for (source, expected) in [
            (
                format!(
                    "Before <kbd title=\"{marker}\">[r](  t  )</kbd> then {marker}tail{marker} after [s](  u  ).\n"
                ),
                format!(
                    "Before <kbd title=\"{marker}\">[r](  t  )</kbd> then {marker}tail{marker} after [s]( u ).\n"
                ),
            ),
            (
                format!(
                    "Before <http://a/{marker}> [r](  t  ) then {marker}tail{marker} after [s](  u  ).\n"
                ),
                format!(
                    "Before <http://a/{marker}> [r]( t ) then {marker}tail{marker} after [s]( u ).\n"
                ),
            ),
            (
                format!("Before <kbd title=\"{marker}[x](  v  ){marker}\">[r](  t  )</kbd>.\n"),
                format!("Before <kbd title=\"{marker}[x](  v  ){marker}\">[r](  t  )</kbd>.\n"),
            ),
        ] {
            for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
                for canonical in [false, true] {
                    let expected = if wrap == "none" { &source } else { &expected };
                    assert_format(&source, expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn review_examples_preserve_ambiguous_links() {
    for (source, expected) in [
        (
            "Before <kbd title=\"> `\">[r](  t  )</kbd> then `tail` after [s](  u  ).\n",
            "Before <kbd title=\"> `\">[r](  t  )</kbd> then `tail` after [s]( u ).\n",
        ),
        (
            "a < b <kbd title=\"`\">[r](  t  )</kbd> then `tail`\n",
            "a < b <kbd title=\"`\">[r](  t  )</kbd> then `tail`\n",
        ),
        // These exact reviewer inputs already fall back unchanged on main.
        (
            "Before <!-- ` --> [r](  t  ) then `tail` after [s](  u  ).\n",
            "Before <!-- ` --> [r](  t  ) then `tail` after [s](  u  ).\n",
        ),
        (
            "Before <!-- $ --> [r](  t  ) then $tail$ after [s](  u  ).\n",
            "Before <!-- $ --> [r](  t  ) then $tail$ after [s](  u  ).\n",
        ),
    ] {
        for wrap in ["none", "paragraph", "sentence:240"] {
            for canonical in [false, true] {
                let expected = if wrap == "none" { source } else { expected };
                assert_format(source, expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn comment_inputs_preserve_links_during_normalization() {
    for marker in ['`', '$'] {
        for (source, expected) in [
            (
                format!(
                    "Before <!-- {marker} --> [r](  t  ) then {marker}tail after [s](  u  ).\n"
                ),
                format!("Before <!-- {marker} --> [r](  t  ) then {marker}tail after [s]( u ).\n"),
            ),
            (
                format!(
                    "Before <!-- > {marker} --> [r](  t  ) then {marker}tail after [s](  u  ).\n"
                ),
                format!(
                    "Before <!-- > {marker} --> [r](  t  ) then {marker}tail after [s]( u ).\n"
                ),
            ),
            (
                format!(
                    "Before <!-- {marker} --> [r](  t  ) <!-- {marker} --> after [s](  u  ).\n"
                ),
                format!("Before <!-- {marker} --> [r](  t  ) <!-- {marker} --> after [s]( u ).\n"),
            ),
            // Raw '<' skips link normalization for this whole input.
            (
                format!(
                    "Before <!-- > {marker}[x](  v  ) ![i](  p  ){marker} --> [r](  t  ) then {marker}[code](  c  ){marker}.\n"
                ),
                format!(
                    "Before <!-- > {marker}[x](  v  ) ![i](  p  ){marker} --> [r]( t ) then {marker}[code](  c  ){marker}.\n"
                ),
            ),
        ] {
            for wrap in ["none", "paragraph", "sentence:240"] {
                for canonical in [false, true] {
                    let expected = if wrap == "none" { &source } else { &expected };
                    assert_format(&source, expected, wrap, canonical);
                }
            }
        }
        for wrap in ["40", "sentence:40"] {
            for canonical in [false, true] {
                assert_format(
                    &format!(
                        "Before <!-- > {marker} --> [r](  t  ) then {marker}tail after [s](  u  ).\n"
                    ),
                    &format!(
                        "Before <!-- >\n{marker} --> [r](  t  ) then {marker}tail after\n[s]( u ).\n"
                    ),
                    wrap,
                    canonical,
                );
            }
        }
    }
}

#[test]
fn angle_compatibility_respects_literals_and_escapes() {
    for marker in ['`', '$'] {
        for contents in [
            "<!-- > [x](  v  ) -->",
            "<kbd title=\"<!--\">[x](  v  )</kbd> -->",
            "<kbd title=\">\"> [x](  u  )",
        ] {
            let source = format!("Before {marker}{contents}{marker} after [r](  t  ).\n");
            let expected = format!("Before {marker}{contents}{marker} after [r](t).\n");
            for wrap in ["none", "paragraph", "sentence:240"] {
                for canonical in [false, true] {
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
        for (candidate, suffix) in [
            ("<!--".to_owned(), ""),
            ("<!-->".to_owned(), ""),
            ("<!-".to_owned(), " -->"),
            (r"\<!--".to_owned(), " -->"),
            ("<!-- > ".repeat(64).trim_end().to_owned(), ""),
        ] {
            let source = format!(
                "Before {candidate} {marker}[x](  v  ){marker}{suffix} after [r](  t  ).\n"
            );
            // Escaped '<' remains eligible; raw '<' skips link normalization,
            // whether or not any angle/comment closing delimiter is present.
            let real = if candidate == r"\<!--" {
                "[r](t)"
            } else {
                "[r]( t )"
            };
            let expected =
                format!("Before {candidate} {marker}[x](  v  ){marker}{suffix} after {real}.\n");
            for wrap in ["none", "paragraph"] {
                for canonical in [false, true] {
                    let expected = if wrap == "none" && candidate != r"\<!--" {
                        &source
                    } else {
                        &expected
                    };
                    assert_format(&source, expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn declaration_and_backtick_reports_keep_their_baseline_cli_output() {
    for (source, expected) in [
        (
            "Before <!DOCTYPE x \"`\"> [r](  t  ) then `tail` after [s](  u  ).\n",
            "Before <!DOCTYPE x \"`\"> [r](  t  ) then `tail` after [s](  u  ).\n",
        ),
        // Exact closing-run recognition is a pre-existing limitation. The
        // second input forces the cache with an earlier unmatched opener.
        (
            "Before `code`` [x](  url  ) tail` after [r](  t  ).\n",
            "Before `code`` [x](url) tail` after [r](t).\n",
        ),
        (
            "Before *```` unmatched `code`` [x](  url  ) tail`* after [r](  t  ).\n",
            "Before *```` unmatched `code`` [x](url) tail`* after [r](t).\n",
        ),
        (
            "Before *```` unmatched `[x](  url  )`* after [r](  t  ).\n",
            "Before *```` unmatched `[x](  url  )`* after [r](t).\n",
        ),
    ] {
        for wrap in ["none", "paragraph", "sentence:240"] {
            for canonical in [false, true] {
                assert_format(source, expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn raw_angle_spellings_preserve_links_during_normalization() {
    for marker in ['`', '$'] {
        for token in [
            format!("<!DOCTYPE x \"{marker}\">"),
            format!("<?x {marker}?>"),
            format!("<42 {marker}>"),
        ] {
            let source = format!("Before {token} [r](  t  ) then {marker}tail after [s](  u  ).\n");
            let expected = format!("Before {token} [r](  t  ) then {marker}tail after [s]( u ).\n");
            for wrap in ["none", "paragraph", "sentence:240"] {
                for canonical in [false, true] {
                    let expected = if wrap == "none" { &source } else { &expected };
                    assert_format(&source, expected, wrap, canonical);
                }
            }
        }
        for (source, expected) in [
            (
                format!("Before <42 {marker}[x](  u  ){marker}> after [r](  t  ).\n"),
                format!("Before <42 {marker}[x](  u  ){marker}> after [r]( t ).\n"),
            ),
            (
                format!("Before <42> {marker}[x](  u  ){marker} after [r](  t  ).\n"),
                format!("Before <42> {marker}[x](  u  ){marker} after [r]( t ).\n"),
            ),
            (
                format!("Before {marker}<!DOCTYPE x \"[x](  u  )\">{marker} after [r](  t  ).\n"),
                format!("Before {marker}<!DOCTYPE x \"[x](  u  )\">{marker} after [r](t).\n"),
            ),
            (
                format!("Before < x < x {marker}[x](  u  ){marker} after [r](  t  ).\n"),
                format!("Before < x < x {marker}[x](  u  ){marker} after [r]( t ).\n"),
            ),
        ] {
            for wrap in ["none", "paragraph", "sentence:240"] {
                for canonical in [false, true] {
                    let expected = if wrap == "none" && source.starts_with("Before <") {
                        &source
                    } else {
                        &expected
                    };
                    assert_format(&source, expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn raw_angles_preserve_the_entire_normalization_input() {
    // Discard partial normalization before the raw '<'. Prose still formats.
    let source =
        "Before `[keep](  u  )` < x <!-- > ` --> [r](  t  ) then `tail after [s](  u  ).\n";
    let expected =
        "Before `[keep](  u  )` < x <!-- > ` --> [r](  t  ) then `tail after [s]( u ).\n";
    for wrap in ["none", "paragraph", "sentence:240"] {
        for canonical in [false, true] {
            let expected = if wrap == "none" { source } else { expected };
            assert_format(source, expected, wrap, canonical);
            for angle in ["< value", "<kbd>", "<http://x>", "<!--"] {
                let real = if wrap == "none" {
                    "[r](  t  )"
                } else {
                    "[r]( t )"
                };
                assert_format(
                    &format!("Before `[a](  u  )` {angle} then `[b](  v  )` after [r](  t  ).\n"),
                    &format!("Before `[a](  u  )` {angle} then `[b](  v  )` after {real}.\n"),
                    wrap,
                    canonical,
                );
            }
        }
    }
}

#[test]
fn angles_consumed_by_links_and_attributes_keep_literal_protection() {
    for (link, normalized) in [
        ("[a](  <target>  )", "[a](<target>)"),
        ("[a](  target  ){key=\"<\"}", "[a](target){key=\"<\"}"),
        ("[a][<target>]", "[a][<target>]"),
    ] {
        for wrap in ["none", "paragraph", "sentence:240"] {
            for canonical in [false, true] {
                assert_format(
                    &format!("Before `[x](  u  )` {link} then `[y](  v  )` after [r](  t  ).\n"),
                    &format!("Before `[x](  u  )` {normalized} then `[y](  v  )` after [r](t).\n"),
                    wrap,
                    canonical,
                );
            }
        }
    }
}

#[test]
fn raw_angle_preservation_does_not_disable_other_paragraphs() {
    let source =
        "Before < value `[a](  u  )` after [r](  t  ).\n\nBefore `[b](  u  )` after [s](  v  ).\n";
    for wrap in ["none", "paragraph", "sentence:240"] {
        for canonical in [false, true] {
            let real = if wrap == "none" {
                "[r](  t  )"
            } else {
                "[r]( t )"
            };
            let expected = format!(
                "Before < value `[a](  u  )` after {real}.\n\nBefore `[b](  u  )` after [s](v).\n"
            );
            assert_format(source, &expected, wrap, canonical);
        }
    }
}
