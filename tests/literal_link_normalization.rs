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
fn template_pairs_keep_the_existing_preservation_policy() {
    for source in [
        "Before `[x](  url  ) {{ value }}` after [r](  t  ) _outside_.\n",
        "- Before $![x](  url  )$ {{ value }} after [r](  t  ).\n",
        "> Before `[x](  url  )` {{ value }} after [r](  t  ).\n",
        "<!-- fmt: template.delimiters \"<<\" \">>\" scope=file -->\nBefore `[x](  url  )` << value >> after [r](  t  ).\n",
    ] {
        for wrap in ["none", "paragraph", "sentence", "16", "sentence:16"] {
            for canonical in [false, true] {
                assert_format(source, source, wrap, canonical);
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
fn existing_angle_spans_do_not_open_literals() {
    for marker in ['`', '$'] {
        for (source, expected) in [
            (
                format!(
                    "Before <kbd title=\"{marker}\">[r](  t  )</kbd> then {marker}tail{marker} after [s](  u  ).\n"
                ),
                format!(
                    "Before <kbd title=\"{marker}\">[r](t)</kbd> then {marker}tail{marker} after [s](u).\n"
                ),
            ),
            (
                format!(
                    "Before <http://a/{marker}> [r](  t  ) then {marker}tail{marker} after [s](  u  ).\n"
                ),
                format!(
                    "Before <http://a/{marker}> [r](t) then {marker}tail{marker} after [s](u).\n"
                ),
            ),
            (
                format!("Before <kbd title=\"{marker}[x](  v  ){marker}\">[r](  t  )</kbd>.\n"),
                format!("Before <kbd title=\"{marker}[x](v){marker}\">[r](t)</kbd>.\n"),
            ),
        ] {
            for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
                for canonical in [false, true] {
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}

#[test]
fn review_examples_retain_baseline_output() {
    for (source, expected) in [
        (
            "a < b <kbd title=\"`\">[r](  t  )</kbd> then `tail`\n",
            "a < b <kbd title=\"`\">[r](t)</kbd> then `tail`\n",
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
                assert_format(source, expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn completed_comments_do_not_open_literals() {
    for marker in ['`', '$'] {
        for (source, expected) in [
            (
                format!(
                    "Before <!-- {marker} --> [r](  t  ) then {marker}tail after [s](  u  ).\n"
                ),
                format!("Before <!-- {marker} --> [r](t) then {marker}tail after [s](u).\n"),
            ),
            (
                format!(
                    "Before <!-- > {marker} --> [r](  t  ) then {marker}tail after [s](  u  ).\n"
                ),
                format!("Before <!-- > {marker} --> [r](t) then {marker}tail after [s](u).\n"),
            ),
            (
                format!(
                    "Before <!-- {marker} --> [r](  t  ) <!-- {marker} --> after [s](  u  ).\n"
                ),
                format!("Before <!-- {marker} --> [r](t) <!-- {marker} --> after [s](u).\n"),
            ),
            // Links inside comments still normalize; literals after them do not.
            (
                format!(
                    "Before <!-- > {marker}[x](  v  ) ![i](  p  ){marker} --> [r](  t  ) then {marker}[code](  c  ){marker}.\n"
                ),
                format!(
                    "Before <!-- > {marker}[x](v) ![i](p){marker} --> [r](t) then {marker}[code](  c  ){marker}.\n"
                ),
            ),
        ] {
            for wrap in ["none", "paragraph", "sentence:240"] {
                for canonical in [false, true] {
                    assert_format(&source, &expected, wrap, canonical);
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
                        "Before <!-- > {marker} --> [r](t) then {marker}tail\nafter [s](u).\n"
                    ),
                    wrap,
                    canonical,
                );
            }
        }
    }
}

#[test]
fn comment_exclusions_follow_source_order_and_require_a_complete_token() {
    for marker in ['`', '$'] {
        for contents in [
            "<!-- > [x](  v  ) -->",
            "<kbd title=\"<!--\">[x](  v  )</kbd> -->",
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
            let expected =
                format!("Before {candidate} {marker}[x](  v  ){marker}{suffix} after [r](t).\n");
            for wrap in ["none", "paragraph"] {
                for canonical in [false, true] {
                    // No completed comment: existing code/math recognition applies.
                    assert_format(&source, &expected, wrap, canonical);
                }
            }
        }
    }
}
