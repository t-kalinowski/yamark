use assert_cmd::Command;

fn format(source: &str, wrap: &str, canonical: bool) -> String {
    let mut command = Command::cargo_bin("yamark").unwrap();
    command.args(["format", "--stdin-file-path", "input.md", "--wrap", wrap]);
    if canonical {
        command.arg("--canonical");
    }
    let output = command.write_stdin(source).output().unwrap();
    assert!(output.status.success(), "{source}\n{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

fn assert_format(source: &str, expected: &str, wrap: &str, canonical: bool) {
    let output = format(source, wrap, canonical);
    assert_eq!(
        output, expected,
        "wrap={wrap}, canonical={canonical}\n{source}"
    );
    assert_eq!(format(&output, wrap, canonical), expected, "second pass");
}

#[test]
fn complete_code_templates_use_normal_paragraph_options() {
    for (directive, literal) in [
        ("", "`${{ foo }} [x](  url  ) _literal_`"),
        ("", "`{% foo %} [x](  url  ) _literal_`"),
        ("", "`{# foo #} [x](  url  ) _literal_`"),
        ("", "`<% foo %> [x](  url  ) _literal_`"),
        ("", "`{{ one }} {{ two }}` and `{{ three }}`"),
        ("", "``{{ foo }} ` [x](  url  ) _literal_``"),
        ("", "```{{ foo }} `` [x](  url  ) _literal_```"),
        (
            "<!-- fmt: template.delimiters \"[[\" \"]]\" scope=file -->\n",
            "`[[ foo ]] [x](  url  ) _literal_`",
        ),
        (
            "<!-- fmt: template.delimiters \"%%\" \"%%\" scope=file -->\n",
            "`%% foo %% [x](  url  ) _literal_`",
        ),
    ] {
        for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
            for canonical in [false, true] {
                let source = format!(
                    "{directive}Before\nwith {literal}. After\n[real](  target  ) _outside_.\n"
                );
                let outside = if canonical { "*outside*" } else { "_outside_" };
                let expected = match wrap {
                    "none" => {
                        format!(
                            "{directive}Before\nwith {literal}. After\n[real](target) {outside}.\n"
                        )
                    }
                    "sentence" | "sentence:240" => {
                        format!(
                            "{directive}Before with {literal}.\nAfter [real](target) {outside}.\n"
                        )
                    }
                    _ => format!(
                        "{directive}Before with {literal}. After [real](target) {outside}.\n"
                    ),
                };
                assert_format(&source, &expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn overwide_code_templates_remain_atomic() {
    for literal in [
        "`{{ foo }} [x](  long-target  )`",
        "``<% foo %> ![x](  long-target  ) ` ``",
    ] {
        for wrap in ["16", "sentence:16"] {
            for canonical in [false, true] {
                assert_format(
                    &format!("Before {literal} after [r](  t  ).\n"),
                    &format!("Before\n{literal}\nafter [r](t).\n"),
                    wrap,
                    canonical,
                );
            }
        }
    }
}

#[test]
fn ambiguous_or_non_code_templates_keep_the_original_paragraph() {
    for body in [
        "`{{ foo }}` and {{ outside }}",
        "`{{ foo` }}",
        "{{ foo `bar }}`",
        "`{{ foo` and `bar }}`",
        "`{{ \"}}\"` outside }}",
        "`{{ foo }}` and {{ unmatched",
        "`{{ foo }}` and a stray }}",
        r"\`{{ foo }}\`",
        "`{{ foo }}",
        "``{{ foo }}`",
        "`{{ foo }}``",
        "``{{ foo }}```",
        "*with `{{ foo }}` inside*",
        "_with `{{ foo }}` inside_",
        "~~with `{{ foo }}` inside~~",
        "$`{{ foo }}`$",
        "[with `{{ foo }}`](target)",
        "[link](`{{foo}}`)",
        "[link](target){key=\"`{{ foo }}`\"}",
        "{key=\"`{{ foo }}`\"}",
        "`{{ foo }}` < value",
        "`{{ foo }}` <kbd>text</kbd>",
        "`{{ foo }}` <!-- comment -->",
        "`{{ foo }}` <http://example.com>",
        "`{{ foo }}` *<kbd>text</kbd>*",
        "`{{ foo\nbar }}`",
        "`{{ foo }}` and `multiline\ncode`",
        "`{{ foo }}` and \\LaTeX",
    ] {
        for wrap in ["none", "paragraph", "sentence", "16", "sentence:16"] {
            for canonical in [false, true] {
                let source = format!("Before\n{body} after [real](  target  ) _outside_.\n");
                let expected = if body == "`{{ foo }}` and {{ outside }}" {
                    if matches!(wrap, "16" | "sentence:16") {
                        let outside = if canonical { "*outside*" } else { "_outside_" };
                        let expected = format!(
                            "Before\n`{{{{ foo }}}}` and\n{{{{ outside }}}}\nafter\n[real](target)\n{outside}.\n"
                        );
                        assert_format(&source, &expected, wrap, canonical);
                        continue;
                    }
                    let outside = if canonical { "*outside*" } else { "_outside_" };
                    let separator = if wrap == "none" { "\n" } else { " " };
                    format!("Before{separator}{body} after [real](target) {outside}.\n")
                } else {
                    source.clone()
                };
                assert_format(&source, &expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn late_delimiters_recheck_original_code_ranges_and_explicit_targets() {
    let late = "\n<!-- fmt: template.delimiters \"[[\" \"]]\" scope=file -->\n";
    for target in ["", "<!-- fmt: wrap=sentence scope=next -->\n"] {
        for (body, eligible) in [
            ("`[[ foo ]]`", true),
            ("`{{ foo }}` and `[[ bar ]]`", true),
            ("`{{ foo }}` and [[ outside ]]", true),
            ("`[[ foo` and `bar ]]`", false),
        ] {
            let source = format!("{target}Before\n{body}. After\nsentence.\n{late}");
            let expected = if eligible {
                format!("{target}Before {body}.\nAfter sentence.\n{late}")
            } else {
                source.clone()
            };
            // On the second pass the file directive is still encountered late.
            assert_format(&source, &expected, "sentence", false);
        }
    }
}

#[test]
fn scoped_delimiters_and_skip_keep_their_boundaries() {
    let source = "<!-- fmt: template.delimiters \"[[\" \"]]\" scope=from-here -->\n\
Before\n`[[ foo ]]`. After\nsentence.\n\n\
<!-- fmt: skip -->\nBefore\n`{{ foo }}`. After\nsentence.\n\n\
Following\nparagraph. Next\nsentence.\n";
    let expected = "<!-- fmt: template.delimiters \"[[\" \"]]\" scope=from-here -->\n\
Before `[[ foo ]]`.\nAfter sentence.\n\n\
<!-- fmt: skip -->\nBefore\n`{{ foo }}`. After\nsentence.\n\n\
Following paragraph.\nNext sentence.\n";
    assert_format(source, expected, "sentence", false);
}

#[test]
fn containers_reflow_while_heading_guards_remain() {
    for prefix in ["# ", "- ", "> ", "[^note]: "] {
        let source = format!(
            "{prefix}Before `{{{{ foo }}}}` after [real](  target  ).\n\nFollowing\nparagraph. Next\nsentence.\n"
        );
        let expected = format!(
            "{prefix}Before `{{{{ foo }}}}` after [real](  target  ).\n\nFollowing paragraph.\nNext sentence.\n"
        );
        let expected = if matches!(prefix, "- " | "> " | "[^note]: ") {
            expected.replace("[real](  target  )", "[real](target)")
        } else {
            expected
        };
        assert_format(&source, &expected, "sentence", true);
    }
}

#[test]
fn escaped_angles_and_consumed_links_do_not_open_code() {
    for link in [r"\< value", "[a](  <target>  )", "[a](  `target  )"] {
        let normalized = link.replace("(  ", "(").replace("  )", ")");
        assert_format(
            &format!("Before\n`{{{{ foo }}}}` with {link}. After\nsentence.\n"),
            &format!("Before `{{{{ foo }}}}` with {normalized}.\nAfter sentence.\n"),
            "sentence",
            false,
        );
    }
}

#[test]
fn frontmatter_options_and_existing_child_documents_use_the_same_planner() {
    for (prefix, suffix) in [
        (
            "---\neditor_options:\n  markdown:\n    wrap: sentence\n---\n",
            "",
        ),
        ("::: {.callout-note}\n", ":::\n"),
        ("```markdown\n", "```\n"),
    ] {
        let source = format!("{prefix}Before\nwith `{{{{ foo }}}}`. After\nsentence.\n{suffix}");
        let expected = format!("{prefix}Before with `{{{{ foo }}}}`.\nAfter sentence.\n{suffix}");
        let wrap = if prefix.starts_with("---") {
            "none"
        } else {
            "sentence"
        };
        assert_format(&source, &expected, wrap, false);
    }
}

#[test]
fn host_guards_still_preserve_template_bearing_markdown() {
    for (path, source) in [
        (
            "input.yaml",
            "body: !markdown |\n  Before\n  with `{{ foo }}`. After\n  sentence.\n",
        ),
        (
            "input.yaml",
            "body: !markdown \"Before with `{{ foo }}`. After sentence.\"\n",
        ),
        (
            "input.py",
            "# fmt: markdown\ntext = \"\"\"\nBefore\nwith `{{ foo }}`. After\nsentence.\n\"\"\"\n",
        ),
        (
            "input.py",
            "# fmt: markdown\n# Before\n# with `{{ foo }}`. After\n# sentence.\n",
        ),
        (
            "input.R",
            "# fmt: markdown\nx <- r\"(\nBefore\nwith `{{ foo }}`. After\nsentence.\n)\"\n",
        ),
    ] {
        let mut input = source.to_owned();
        for _ in 0..2 {
            let output = Command::cargo_bin("yamark")
                .unwrap()
                .args(["format", "--stdin-file-path", path, "--wrap", "sentence"])
                .write_stdin(input.as_str())
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output.stderr);
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
            assert_eq!(output.stdout, source.as_bytes());
            input = String::from_utf8(output.stdout).unwrap();
        }
    }
}

#[test]
fn explicit_targets_keep_other_rejections_and_skip_precedence() {
    for body in [
        "Before `{{ foo }}` < value.\n",
        "Before `{{ foo` }}.\n",
        "Before {{ foo }}.\n",
        "Before \\LaTeX.\n",
    ] {
        let source = format!("<!-- fmt: wrap=sentence scope=next -->\n{body}");
        let output = Command::cargo_bin("yamark")
            .unwrap()
            .args(["format", "--stdin-file-path", "input.md"])
            .write_stdin(source.as_str())
            .output()
            .unwrap();
        if body == "Before {{ foo }}.\n" {
            // This target was rejected solely for its simple bare expression.
            assert_eq!(output.status.code(), Some(0));
            assert_eq!(output.stdout, source.as_bytes());
            assert!(output.stderr.is_empty());
        } else {
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                b"input.md:2:1: error: fmt: markdown targets an unsupported Markdown block\n"
            );
        }
        let skipped = format!("<!-- fmt: skip file -->\n{source}");
        assert_format(&skipped, &skipped, "sentence", false);
    }
}

#[test]
fn code_templates_respect_explicit_hard_breaks() {
    let source = "Before\n`{{ foo }}`  \nafter [real](  target  ).\n";
    for (wrap, expected) in [
        ("none", "Before\n`{{ foo }}` \\\nafter [real](target).\n"),
        ("sentence", "Before `{{ foo }}` \\\nafter [real](target).\n"),
    ] {
        assert_format(source, expected, wrap, false);
    }
}

#[test]
fn ambiguous_math_boundaries_do_not_enable_template_reflow() {
    for body in [
        r"`{{ foo }}` and $a \$ [x](  url  ) _math_$",
        "$$`{{ foo }}`$$",
    ] {
        let source = format!("Before\nwith {body}. After\nsentence.\n");
        for wrap in ["none", "paragraph", "sentence", "24", "sentence:24"] {
            for canonical in [false, true] {
                assert_format(&source, &source, wrap, canonical);
            }
        }
    }
}

#[test]
fn configured_delimiters_and_replacements_keep_their_meaning() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    let input = dir.path().join("input.md");
    for (configuration, body, eligible) in [
        (
            "[template]\nadd_delimiters = [{open='[[', close=']]'}]\n",
            "`[[ foo ]]`",
            true,
        ),
        (
            "[template]\nadd_delimiters = [{open='[[', close=']]'}]\n",
            "`{{ foo }}` [[ outside ]]",
            true,
        ),
        (
            "[template]\nreplace_delimiters = [{open='[[', close=']]'}]\n",
            "`[[ foo ]]` and {{ ordinary }}",
            true,
        ),
        (
            "[template]\nreplace_delimiters = []\n",
            "{{ ordinary }}",
            true,
        ),
    ] {
        std::fs::write(&config, configuration).unwrap();
        let source = format!("Before\nwith {body}. After\nsentence.\n");
        let expected = if eligible {
            format!("Before with {body}.\nAfter sentence.\n")
        } else {
            source.clone()
        };
        let mut current = source;
        for _ in 0..2 {
            let output = Command::cargo_bin("yamark")
                .unwrap()
                .args([
                    "format",
                    "--stdin-file-path",
                    input.to_str().unwrap(),
                    "--wrap",
                    "sentence",
                ])
                .write_stdin(current.as_str())
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output.stderr);
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
            current = String::from_utf8(output.stdout).unwrap();
            assert_eq!(current, expected);
        }
    }
}
