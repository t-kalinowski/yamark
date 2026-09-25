use assert_cmd::Command;

fn format(source: &str, wrap: &str, canonical: bool, path: &str) -> String {
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

fn assert_format(source: &str, expected: &str, wrap: &str, canonical: bool) {
    let output = format(source, wrap, canonical, "input.md");
    assert_eq!(
        output, expected,
        "wrap={wrap}, canonical={canonical}\n{source}"
    );
    assert_eq!(
        format(&output, wrap, canonical, "input.md"),
        expected,
        "second pass"
    );
}

#[test]
fn complete_expressions_keep_exact_bytes_under_paragraph_options() {
    for expression in [
        "{{ 'foo' }}",
        "{{ \"foo\" }}",
        "{{ }}",
        "{{ 1foo }}",
        "{{ foo. }}",
        "{{ .foo }}",
        "{{ foo..bar }}",
        "{{ foo . bar }}",
        "{{ foo() }}",
        "{{ foo[0] }}",
        "{{ foo | filter }}",
        "{{ foo + bar }}",
        "{{- foo -}}",
        "{{{ foo }}}",
        "{{ foo {{ bar }} }}",
        "{{ foo { bar } }}",
        "{{ foo }} {% other %}",
        "{{ foo }} {# other #}",
        "{{ foo }} <% other %>",
        "prefix{{ foo }}",
        "{{ foo }}suffix",
        "({{ foo }})",
        "{{ foo }}/tail",
        r"{{ foo \}}",
        "{{ foo }} and {{ '{{' }}",
        "{{foo}}",
        "{{ foo }}",
        "{{   user.name   }}",
        "{{\t_user9.Name_2\t }}",
        "{{ _ }}",
        "{{a.b.c}}",
        "{{ foo }} and {{bar}}",
        "{{ foo }} {{ bar }}",
        "{{ foo }} and `{{ complex | filter }}`",
    ] {
        for wrap in ["none", "paragraph", "sentence", "240", "sentence:240"] {
            for canonical in [false, true] {
                let source = format!(
                    "First\nsentence with {expression} and [real](  target  ). Second\nsentence with _outside_.\n"
                );
                let outside = if canonical { "*outside*" } else { "_outside_" };
                let expected = match wrap {
                    "none" => format!(
                        "First\nsentence with {expression} and [real](target). Second\nsentence with {outside}.\n"
                    ),
                    "sentence" | "sentence:240" => format!(
                        "First sentence with {expression} and [real](target).\nSecond sentence with {outside}.\n"
                    ),
                    _ => format!(
                        "First sentence with {expression} and [real](target). Second sentence with {outside}.\n"
                    ),
                };
                assert_format(&source, &expected, wrap, canonical);
            }
        }
    }
}

#[test]
fn unsupported_expressions_and_contexts_keep_baseline_preservation() {
    for body in [
        "{{ foo }} {{< ref target >}}",
        "{{ foo }} {{% ref target %}}",
        r"\{{ foo }}",
        "{{ foo }} stray }}",
        "{{ foo }} and {{ unmatched",
        "`{{ foo` }}",
        "{{ foo `bar }}`",
        "{{ foo }} and `{{ other` outside }}",
        "*{{ foo }}*",
        "_{{ foo }}_",
        "~~{{ foo }}~~",
        "$ {{ foo }} $",
        "[{{ foo }}](target)",
        "[real](target){key=\"{{ foo }}\"}",
        "{{ foo }} < value",
        "{{ foo }} <kbd>text</kbd>",
        "{{ foo }} <!-- x -->",
        "{{ foo }} <https://example.com>",
        "{{ foo }} *<kbd>text</kbd>*",
        "{{ foo\n.bar }}",
        "{{ foo }} and `multi\nline`",
        "{{ foo }} and $multi\nline$",
        "{{ foo }} and *multi\nline*",
        "{{ foo }} and [real](multi\nline)",
        "{{ foo }} and \\LaTeX",
        "{{ foo }} and ``code`",
        "{{ foo }} and `code``",
        "{{ foo }} and $a \\$ b$",
        "{{ foo }} and $$a$$",
    ] {
        let source = format!("Before\nwith {body} after [real](  target  ) _outside_.\n");
        for wrap in ["none", "paragraph", "sentence", "16", "sentence:16"] {
            for canonical in [false, true] {
                assert_format(&source, &source, wrap, canonical);
            }
        }
    }
}

#[test]
fn source_and_planned_template_only_lines_follow_wrapping_mode() {
    for (wrap, expected) in [
        ("none", "Before\n{{ foo }}\nafter [real](target).\n"),
        ("paragraph", "Before {{ foo }} after [real](target).\n"),
        ("sentence", "Before {{ foo }} after [real](target).\n"),
        ("12", "Before\n{{ foo }}\nafter\n[real](\n  target\n).\n"),
        (
            "sentence:12",
            "Before\n{{ foo }}\nafter\n[real](\n  target\n).\n",
        ),
    ] {
        assert_format(
            "Before\n{{ foo }}\nafter [real](  target  ).\n",
            expected,
            wrap,
            false,
        );
    }
    for wrap in ["36", "sentence:36"] {
        assert_format(
            "Before {{   user.name   }} after\nsome more prose with [real](  t  ).\n",
            "Before {{   user.name   }} after\nsome more prose with [real](t).\n",
            wrap,
            false,
        );
    }
}

#[test]
fn punctuation_and_paragraph_boundaries_are_inline_placements() {
    for punctuation in [".", ",", ";", ":", "!", "?"] {
        let source = format!("Before\nwith {{{{ foo }}}}{punctuation} After\nprose.\n");
        let expected = format!("Before with {{{{ foo }}}}{punctuation} After prose.\n");
        assert_format(&source, &expected, "paragraph", true);
    }
    assert_format(
        "{{ foo }} starts\nprose ending with {{ bar }}\n",
        "{{ foo }} starts prose ending with {{ bar }}\n",
        "paragraph",
        false,
    );
}

#[test]
fn delimiter_configuration_and_late_policy_use_original_source() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    let input = dir.path().join("input.md");
    for configuration in [
        "[template]\nadd_delimiters = [{open='[[', close=']]'}]\n",
        "[template]\nreplace_delimiters = [{open='[[', close=']]'}]\n",
        "[template]\nreplace_delimiters = []\n",
        "[template]\nreplace_delimiters = [{open='{{', close='}}'}]\n",
    ] {
        std::fs::write(&config, configuration).unwrap();
        for body in [
            "{{   foo   }}",
            "{{ foo }} `[[ other ]]`",
            "{{ foo }} [[ other ]]",
        ] {
            // A late file directive activates [[ even for an initially explicit target.
            let late = "\n<!-- fmt: template.delimiters \"[[\" \"]]\" scope=file -->\n";
            let target = "<!-- fmt: wrap=sentence scope=next -->\n";
            let source = format!("{target}First\nwith {body}. Second\nsentence.\n{late}");
            let expected = format!("{target}First with {body}.\nSecond sentence.\n{late}");
            let first = format(&source, "sentence", false, input.to_str().unwrap());
            assert_eq!(first, expected, "{configuration}");
            assert_eq!(
                format(&first, "sentence", false, input.to_str().unwrap()),
                expected
            );
        }
        let late = "\n<!-- fmt: template.delimiters \"{{\" \"}}\" scope=file -->\n";
        let source = format!("First\nwith {{{{   foo   }}}}. Second\nsentence.\n{late}");
        let expected = format!("First with {{{{   foo   }}}}.\nSecond sentence.\n{late}");
        let first = format(&source, "sentence", true, input.to_str().unwrap());
        assert_eq!(first, expected);
        assert_eq!(
            format(&first, "sentence", true, input.to_str().unwrap()),
            expected
        );

        // Balanced braces remain opaque ordinary text when their template
        // delimiter is disabled; narrow layouts now agree in both cases.
        let source = "Before {{   foo   }} after.\n";
        let expected = "Before\n{{   foo   }}\nafter.\n";
        let first = format(source, "8", false, input.to_str().unwrap());
        assert_eq!(first, expected);
        assert_eq!(
            format(&first, "8", false, input.to_str().unwrap()),
            expected
        );
    }
}

#[test]
fn target_skip_frontmatter_and_child_documents_keep_their_policies() {
    for (prefix, suffix, wrap) in [
        ("<!-- fmt: wrap=sentence scope=next -->\n", "", "none"),
        ("", "\n<!-- fmt: wrap=sentence scope=file -->\n", "none"),
        (
            "---\neditor_options:\n  markdown:\n    wrap: sentence\n---\n",
            "",
            "none",
        ),
        ("::: {.callout-note}\n", ":::\n", "sentence"),
        ("```markdown\n", "```\n", "sentence"),
        (
            "<!-- fmt: template.delimiters \"[[\" \"]]\" scope=from-here -->\n",
            "",
            "sentence",
        ),
    ] {
        let source = format!("{prefix}First\nwith {{{{ foo }}}}. Second\nsentence.\n{suffix}");
        let expected = format!("{prefix}First with {{{{ foo }}}}.\nSecond sentence.\n{suffix}");
        assert_format(&source, &expected, wrap, false);
    }
    for skip in ["<!-- fmt: skip -->\n", "<!-- fmt: skip file -->\n"] {
        let source = format!("{skip}First\nwith {{{{ foo }}}}. Second\nsentence.\n");
        assert_format(&source, &source, "sentence", true);
    }
    for body in [
        "Before {{ foo }} < value.\n",
        "Before {{ render(\"unclosed) }} after.\n",
    ] {
        let source = format!("<!-- fmt: wrap=8 scope=next -->\n{body}");
        let output = Command::cargo_bin("yamark")
            .unwrap()
            .args(["format", "--stdin-file-path", "input.md"])
            .write_stdin(source.as_str())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"input.md:2:1: error: fmt: markdown targets an unsupported Markdown block\n"
        );
        let skipped = format!("<!-- fmt: skip file -->\n{source}");
        assert_format(&skipped, &skipped, "sentence", false);
    }
}

#[test]
fn heading_guards_and_explicit_hard_breaks_keep_their_boundaries() {
    let source = "# Before {{ foo }} after [real](  target  ) _outside_.\n";
    assert_format(source, source, "sentence", true);
    for prefix in ["- ", "> "] {
        let source = format!("{prefix}Before {{{{ foo }}}} after [real](  target  ) _outside_.\n");
        let expected = format!("{prefix}Before {{{{ foo }}}} after [real](target) *outside*.\n");
        assert_format(&source, &expected, "sentence", true);
    }
    assert_format(
        "Before\n{{ foo }}  \nafter [real](  target  ).\n",
        "Before {{ foo }} \\\nafter [real](target).\n",
        "sentence",
        false,
    );
}

#[test]
fn late_policy_changes_reconsider_placement_without_changing_target_precedence() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("yamark.toml"),
        "[template]\nreplace_delimiters = []\n",
    )
    .unwrap();
    let path = dir.path().join("input.md");
    let source = "<!-- fmt: wrap=8 scope=next -->\nBefore {{ foo }} after.\n\n<!-- fmt: template.delimiters \"{{\" \"}}\" scope=file -->\n";
    let first = format(source, "8", false, path.to_str().unwrap());
    let expected = "<!-- fmt: wrap=8 scope=next -->\nBefore\n{{ foo }}\nafter.\n\n<!-- fmt: template.delimiters \"{{\" \"}}\" scope=file -->\n";
    assert_eq!(first, expected);
    assert_eq!(format(&first, "8", false, path.to_str().unwrap()), expected);

    let source = "Before\nwith {{ foo }} after [real](  target  ).\n\n<!-- fmt: wrap=sentence scope=file -->\n";
    let expected =
        "Before with {{ foo }} after [real](target).\n\n<!-- fmt: wrap=sentence scope=file -->\n";
    assert_format(source, expected, "8", false);
}

#[test]
fn hosts_keep_their_bare_template_guards() {
    for (path, source) in [
        (
            "input.yaml",
            "body: !markdown |\n  Before\n  with {{ foo }}. After\n  sentence.\n",
        ),
        (
            "input.yaml",
            "body: !markdown \"Before with {{ foo }}. After sentence.\"\n",
        ),
        (
            "input.py",
            "# fmt: markdown\ntext = \"\"\"\nBefore\nwith {{ foo }}. After\nsentence.\n\"\"\"\n",
        ),
        (
            "input.R",
            "# fmt: markdown\nx <- r\"(\nBefore\nwith {{ foo }}. After\nsentence.\n)\"\n",
        ),
    ] {
        let first = format(source, "sentence", false, path);
        assert_eq!(first, source);
        assert_eq!(format(&first, "sentence", false, path), source);
    }
}
