use super::support::run_stdin;

#[test]
fn markdown_directives_and_blocks() {
    let input = "\
<!-- fmt: canonical=true scope=file -->
Title
=====
This is e.g. one sentence. This is __strong__ and _emphasis_!

<!-- fmt: off -->
#   Keep ##
<!-- fmt: on -->
#   Change ##

   * * *
";
    let expected = "\
<!-- fmt: canonical=true scope=file -->
# Title
This is e.g. one sentence.
This is **strong** and *emphasis*!

<!-- fmt: off -->
#   Keep ##
<!-- fmt: on -->
# Change

---
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_skip_at_eof_is_preserved_without_error() {
    let input = "<!-- fmt: skip -->\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_skip_file_short_circuits_later_invalid_fmt_comments() {
    let input = "\
<!-- fmt: skip file -->
<!-- fmt: unknown -->
#   Keep ##
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_off_region_preserves_invalid_fmt_comments_until_on() {
    let input = "\
<!-- fmt: off -->
#   Keep ##
<!-- fmt: unknown -->
<!-- fmt: on scope=next -->
<!-- fmt: on -->
#   Change ##
";
    let expected = "\
<!-- fmt: off -->
#   Keep ##
<!-- fmt: unknown -->
<!-- fmt: on scope=next -->
<!-- fmt: on -->
# Change
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_off_region_without_on_preserves_through_eof() {
    let input = "\
<!-- fmt: off -->
<!-- fmt: on scope=next -->
#   Keep ##
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_file_scope_directives_patch_prior_rendered_blocks() {
    let input = "\
This is __strong__ and _emphasis_ before the directive.

<!-- fmt: canonical=true scope=file -->
";
    let expected = "\
This is **strong** and *emphasis* before the directive.

<!-- fmt: canonical=true scope=file -->
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_file_scope_directives_patch_prior_markdown_code_fences() {
    let input = "\
```markdown
This is __strong__ before the directive.
```

<!-- fmt: canonical=true scope=file -->
";
    let expected = "\
```markdown
This is **strong** before the directive.
```

<!-- fmt: canonical=true scope=file -->
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_file_scope_directives_patch_prior_yaml_code_fence_markdown_scalars() {
    let input = "\
```yaml
body: !markdown |
  This is __strong__ before the directive.
```

<!-- fmt: canonical=true scope=file -->
";
    let expected = "\
```yaml
body: !markdown |
  This is **strong** before the directive.
```

<!-- fmt: canonical=true scope=file -->
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_active_directives_apply_to_nested_code_fences_on_first_parse() {
    let input = "\
<!-- fmt: canonical=true wrap=sentence scope=from-here -->
```markdown
This is __strong__. This is _emphasis_.
```

```yaml
body: !markdown |
  This is __strong__. This is _emphasis_.
```
";
    let expected = "\
<!-- fmt: canonical=true wrap=sentence scope=from-here -->
```markdown
This is **strong**.
This is *emphasis*.
```

```yaml
body: !markdown |
  This is **strong**.
  This is *emphasis*.
```
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_scope_next_directive_applies_to_nested_code_fence_only() {
    let input = "\
<!-- fmt: canonical=true scope=next -->
```markdown
This is __strong__.
```

This is _emphasis_.
";
    let expected = "\
<!-- fmt: canonical=true scope=next -->
```markdown
This is **strong**.
```

This is _emphasis_.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_template_state_applies_inside_recursive_code_fences() {
    let input = "\
<!-- fmt: template.delimiters \"<<\" \">>\" scope=from-here -->
```markdown
#   << keep   this >> ##
```

```yaml
body: !markdown |
  #   << keep   this >> ##
```
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_template_delimiter_directives_accept_spaces_and_escapes() {
    let input = "\
<!-- fmt: template.delimiters \"<< open\" \"close >>\" -->
This << open keep   spacing close >> paragraph.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");

    let input = r#"<!-- fmt: template.delimiters "[\"" "\"]" -->
This [" keep   spacing "] paragraph.
"#;
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_code_fence_local_skip_requires_exact_supported_forms() {
    let input = "\
```yaml fmt: skip-extra
a:    1
```
";
    let expected = "\
```yaml fmt: skip-extra
a: 1
```
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_code_fence_local_skip_accepts_scope_next() {
    let input = "\
```yaml fmt: skip scope=next
a:    1
```
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_next_directive_preserves_local_skip_code_fence_target() {
    let input = "\
<!-- fmt: wrap=sentence scope=next -->
```yaml fmt: skip
a:    1
```
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_directive_errors_are_reported() {
    let cases = [
        ("<!-- fmt: -->\n# Title\n", "invalid fmt directive"),
        (
            "<!-- fmt: skip scope=from-here -->\n# Title\n",
            "fmt: skip does not support scope=from-here",
        ),
        (
            "<!-- fmt: off scope=file -->\n# Title\n",
            "fmt: off does not support scope=file",
        ),
        (
            "<!-- fmt: on scope=next -->\n# Title\n",
            "fmt: on does not support explicit scope",
        ),
        (
            "<!-- fmt: wrap=0 -->\n# Title\n",
            "fmt: wrap must be none, paragraph, sentence, a positive integer, or sentence:<positive integer>",
        ),
        (
            "<!-- fmt: template.delimiters \"<<\" -->\n# Title\n",
            "fmt: template.delimiters requires exactly two quoted delimiter strings",
        ),
        (
            "<!-- fmt: table -->\n| a |\n|---|\n",
            "invalid fmt directive: table",
        ),
        (
            "<!-- fmt: canonical=true scope=next -->\n",
            "fmt: markdown has no target",
        ),
    ];

    for (input, message) in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.md"], input);
        assert_eq!(status, 1, "{input}");
        assert_eq!(stdout, "");
        assert!(stderr.contains(message), "{stderr}");
    }
}

#[test]
fn markdown_next_format_directive_rejects_unsupported_targets() {
    let cases = [
        "\
<!-- fmt: wrap=sentence scope=next -->
<div>
keep    raw
</div>
",
        "\
<!-- fmt: wrap=20 scope=next -->
- This ~~struck text keeps    spacing.
",
        "\
<!-- fmt: wrap=20 scope=next -->
> This ~~struck text keeps    spacing.
",
        "\
<!-- fmt: wrap=20 scope=next -->
Term
: This ~~struck text keeps    spacing.
",
        "\
<!-- fmt: wrap=20 scope=next -->
before \\LaTeX after
",
    ];

    for input in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.md"], input);
        assert_eq!(status, 1, "{input}");
        assert_eq!(stdout, "");
        assert!(
            stderr.contains("fmt: markdown targets an unsupported Markdown block"),
            "{stderr}"
        );
    }
}

#[test]
fn markdown_template_delimiter_directive_infers_from_here_when_isolated() {
    let input = "\
<!-- fmt: template.delimiters \"<<\" \">>\" -->

This << keep   spacing >> paragraph.

Another << keep   spacing >> paragraph.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn markdown_template_delimiter_directive_requires_unambiguous_implicit_scope() {
    let input = "\
Intro paragraph.
<!-- fmt: template.delimiters \"<<\" \">>\" -->

This << keep   spacing >> paragraph.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("fmt: template.delimiters needs explicit scope"),
        "{stderr}"
    );
}

#[test]
fn nested_markdown_template_delimiter_directive_infers_from_here_at_range_start() {
    let input = "\
```markdown
<!-- fmt: template.delimiters \"<<\" \">>\" -->

This << keep   spacing >> paragraph.

Another << keep   spacing >> paragraph.
```
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn unknown_markdown_html_fmt_directives_error() {
    let input = "<!-- fmt: unknown -->\n# Title\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("invalid fmt directive: unknown"),
        "{stderr}"
    );
}
