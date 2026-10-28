use super::support::run_stdin;

#[test]
fn embedded_source_skip_file_short_circuits_later_invalid_fmt_comments() {
    let cases = [
        (
            "input.py",
            "\
# fmt: skip file
# fmt: upper
# #   Keep ##
",
        ),
        (
            "input.r",
            "\
# fmt: skip file
# fmt: upper
# #   Keep ##
",
        ),
    ];

    for (path, input) in cases {
        let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", path], input);
        assert_eq!(status, 0, "{path}: {stderr}");
        assert_eq!(stdout, input, "{path}");
        assert_eq!(stderr, "", "{path}");
    }
}

#[test]
fn embedded_python_comment_markdown_target() {
    let input = "\
# fmt: markdown
# #   Title ##
# This is __strong__. This is _emphasis_.
";
    let expected = "\
# fmt: markdown
# # Title
# This is **strong**.
# This is *emphasis*.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.py",
            "--wrap",
            "sentence",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_source_fmt_off_ignores_directives_until_on() {
    let cases = [("input.py", "x = 1\n"), ("input.R", "x <- 1\n")];

    for (path, code) in cases {
        let input = format!(
            "\
# fmt: off
# fmt: markdown
{code}# fmt: upper
# fmt: on
# fmt: markdown
# #   Title ##
"
        );
        let expected = format!(
            "\
# fmt: off
# fmt: markdown
{code}# fmt: upper
# fmt: on
# fmt: markdown
# # Title
"
        );
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "none"],
            &input,
        );
        assert_eq!(status, 0, "{path}: {stderr}");
        assert_eq!(stdout, expected, "{path}");
        assert_eq!(stderr, "", "{path}");
    }
}

#[test]
fn embedded_source_file_scope_markdown_patches_prior_marked_targets_only() {
    let input = "\
# fmt: markdown
# This is __before__.
DOC = \"\"\"
This is __not__ marked.
\"\"\"
# fmt: markdown canonical=true scope=file
x = 1
# fmt: markdown
# This is __after__.
";
    let expected = "\
# fmt: markdown
# This is **before**.
DOC = \"\"\"
This is __not__ marked.
\"\"\"
# fmt: markdown canonical=true scope=file
x = 1
# fmt: markdown
# This is **after**.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_python_markdown_targets_support_cr_only_line_endings() {
    let input = "# fmt: markdown\rDOC = \"\"\"\r#   Title ##\r\"\"\"\r";
    let expected = "# fmt: markdown\rDOC = \"\"\"\r# Title\r\"\"\"\r";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_python_comment_markdown_restores_prefixes_for_cr_only_lines() {
    let input = "# fmt: markdown\r# one two three four\r# five six seven eight\r";
    let expected = "# fmt: markdown\r# one two\r# three four\r# five six\r# seven eight\r";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "14"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_comment_markdown_wrap_width_accounts_for_prefix() {
    let input = "\
# fmt: markdown
    # one two three four
";
    let expected = "\
# fmt: markdown
    # one two
    # three four
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "18"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_comment_sentence_column_wrap_accounts_for_prefix() {
    let input = "\
# fmt: markdown
    # One short. This second sentence has more words.
";
    let expected = "\
# fmt: markdown
    # One short.
    # This second
    # sentence has
    # more words.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.py",
            "--wrap",
            "sentence:18",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_comment_markdown_wrap_width_uses_visual_prefix_width() {
    let wide_space = "\u{3000}";
    let input = format!(
        "\
# fmt: markdown
{wide_space}# abc de f
"
    );
    let expected = format!(
        "\
# fmt: markdown
{wide_space}# abc de
{wide_space}# f
"
    );
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "11"],
        &input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_template_delimiter_directive_isolated_defaults_to_from_here() {
    let python = "\
# fmt: template.delimiters \"<<\" \">>\"

# fmt: markdown
FIRST = \"\"\"
#   <<one>> ##
\"\"\"

# fmt: markdown
SECOND = \"\"\"
#   <<two>> ##
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        python,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, python);
    assert_eq!(stderr, "");

    let r = "\
# fmt: template.delimiters \"<<\" \">>\"

# fmt: markdown
first <- \"
#   <<one>> ##
\"

# fmt: markdown
second <- \"
#   <<two>> ##
\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.R", "--wrap", "none"],
        r,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, r);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_source_pending_markdown_directive_without_target_errors() {
    let input = "# fmt: markdown\nx = 1\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("fmt: markdown has no target"), "{stderr}");
}

#[test]
fn embedded_source_markdown_directive_targets_next_string_literal_after_code() {
    let cases = [
        (
            "input.py",
            "\
# fmt: markdown
value = render(
    normalize(
        prefix,
        \"\"\"A    python markdown paragraph.
\"\"\"
    )
)
x={\"a\":1,\"b\":2}
",
            "\
# fmt: markdown
value = render(
    normalize(
        prefix,
        \"\"\"A python markdown paragraph.
\"\"\"
    )
)
x={\"a\":1,\"b\":2}
",
        ),
        (
            "input.r",
            "\
# fmt: markdown
value <- render(
    normalize(
        prefix,
        \"A    R markdown paragraph.
\"
    )
)
x <- list(a=1,b=2)
",
            "\
# fmt: markdown
value <- render(
    normalize(
        prefix,
        \"A R markdown paragraph.
\"
    )
)
x <- list(a=1,b=2)
",
        ),
    ];

    for (path, input, expected) in cases {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "120"],
            input,
        );
        assert_eq!(status, 0, "{path}: {stderr}");
        assert_eq!(stdout, expected, "{path}");
        assert_eq!(stderr, "", "{path}");
    }
}

#[test]
fn embedded_source_markdown_directive_errors_when_code_precedes_target() {
    let input = "\
# fmt: markdown
x = 1
# #   Title ##
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("fmt: markdown has no target"), "{stderr}");
}

#[test]
fn embedded_source_markdown_directive_ignores_quote_delimiters_in_trailing_comments() {
    let cases = [
        (
            "input.py",
            "\
# fmt: markdown
x = 1 # \"\"\"
#   Title ##
\"\"\"
",
        ),
        (
            "input.R",
            "\
# fmt: markdown
x <- 1 # \"
#   Title ##
\"
",
        ),
        (
            "input.R",
            "\
# fmt: markdown
x <- 1 # r\"(
#   Title ##
)\"
",
        ),
    ];
    for (path, input) in cases {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "none"],
            input,
        );
        assert_eq!(status, 1, "{path}: {stderr}");
        assert_eq!(stdout, "", "{path}");
        assert!(
            stderr.contains("fmt: markdown has no target"),
            "{path}: {stderr}"
        );
    }
}

#[test]
fn embedded_source_markdown_directive_errors_on_unsupported_comment_target() {
    for path in ["input.py", "input.R"] {
        let input = "\
# fmt: markdown
# first paragraph line
## unsupported mixed prefix
";
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "none"],
            input,
        );
        assert_eq!(status, 1, "{path}: {stderr}");
        assert_eq!(stdout, "", "{path}");
        assert!(
            stderr.contains("fmt: markdown has no target"),
            "{path}: {stderr}"
        );
    }
}

#[test]
fn python_markdown_string_targets_require_safe_supported_literals() {
    let cases = [
        (
            "# fmt: markdown\ntext = \"\"\"inline\"\"\"\n",
            "fmt: markdown has no target",
        ),
        (
            "# fmt: markdown\ntext = \"\"\"\nbackslash \\\\ stays unsafe\n\"\"\"\n",
            "non-raw Python Markdown strings must not contain backslashes",
        ),
        (
            "# fmt: markdown\ntext = rr\"\"\"\n#   Title ##\n\"\"\"\n",
            "fmt: markdown has no target",
        ),
    ];

    for (input, message) in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.py"], input);
        assert_eq!(status, 1, "{input}");
        assert_eq!(stdout, "");
        assert!(stderr.contains(message), "{stderr}");
    }

    let input = "# fmt: markdown\ntext = f\"\"\"\nThis    has {value}.\n\"\"\"\n";
    let expected = "# fmt: markdown\ntext = f\"\"\"\nThis has {value}.\n\"\"\"\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.py"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn python_f_string_markdown_targets_preserve_only_expressions() {
    let input = "\
# fmt: markdown
text = f\"\"\"
This    has { value   +  one } and some long surrounding prose that should wrap.
\"\"\"
";
    let expected = "\
# fmt: markdown
text = f\"\"\"
This has { value   +  one } and some long
surrounding prose that should wrap.
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "42"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn python_f_string_markdown_expressions_override_generic_template_preservation() {
    let input = "\
# fmt: markdown
text = f\"\"\"
This    has { \"{{ keep   }}\" } and surrounding   text.
\"\"\"
";
    let expected = "\
# fmt: markdown
text = f\"\"\"
This has { \"{{ keep   }}\" } and surrounding text.
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.py",
            "--wrap",
            "sentence",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn embedded_source_directives_inside_unsupported_strings_are_ignored() {
    let python = "\
payload = b\"\"\"
# fmt: markdown
this    should     not trigger
\"\"\"
after = 1
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.py"], python);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, python);
    assert_eq!(stderr, "");

    let r = "\
payload <- r\"tag(
# fmt: markdown
this    should     not trigger
)tag\"
after <- 1
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.r"], r);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, r);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_comment_markdown_blank_lines_do_not_gain_trailing_spaces() {
    let input = "\
# fmt: markdown
# #   Title ##
#
# Body text.
";
    let expected = "\
# fmt: markdown
# # Title
#
# Body text.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_python_markdown_string_targets_are_dedented_and_reindented() {
    let input = "\
# fmt: markdown
DOC = \"\"\"
    #   Title ##
    This is __strong__. This is _emphasis_.
\"\"\"
";
    let expected = "\
# fmt: markdown
DOC = \"\"\"
    # Title
    This is **strong**.
    This is *emphasis*.
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.py",
            "--wrap",
            "sentence",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_python_markdown_string_keeps_closing_delimiter_at_content_indent() {
    let input = "\
# fmt: markdown
DOC = \"\"\"
    #   Title ##
    \"\"\"
";
    let expected = "\
# fmt: markdown
DOC = \"\"\"
    # Title
    \"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_python_markdown_string_preserves_column_zero_closing_delimiter() {
    let input = "\
# fmt: markdown
DOC = \"\"\"
    #   Title ##
\"\"\"
";
    let expected = "\
# fmt: markdown
DOC = \"\"\"
    # Title
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_markdown_string_targets_ignore_delimiter_padding() {
    let cases = [
        (
            "input.py",
            concat!(
                "# fmt: markdown\n",
                "DOC = \"\"\"\n",
                "    #   Title ##\n",
                "    \n",
                "    \"\"\"\n",
            ),
            "\
# fmt: markdown
DOC = \"\"\"
    # Title
    \"\"\"
",
        ),
        (
            "input.r",
            concat!(
                "# fmt: markdown\n",
                "text <- \"\n",
                "    #   Title ##\n",
                "    \n",
                "    \"\n",
            ),
            "\
# fmt: markdown
text <- \"
    # Title
    \"
",
        ),
        (
            "input.r",
            concat!(
                "# fmt: markdown\n",
                "text <- r\"(\n",
                "    #   Title ##\n",
                "    \n",
                "    )\"\n",
            ),
            "\
# fmt: markdown
text <- r\"(
    # Title
    )\"
",
        ),
    ];

    for (path, input, expected) in cases {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "none"],
            input,
        );
        assert_eq!(status, 0, "{path}: {stderr}");
        assert_eq!(stdout, expected, "{path}");
        assert_eq!(stderr, "", "{path}");

        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "none"],
            expected,
        );
        assert_eq!(status, 0, "{path}: {stderr}");
        assert_eq!(stdout, expected, "{path}: not idempotent");
        assert_eq!(stderr, "", "{path}");
    }
}

#[test]
fn embedded_python_markdown_string_target_accepts_escaped_opening_newline() {
    let input = "# fmt: markdown\nDOC = \"\"\"\\\n    #   Title ##\n\"\"\"\n";
    let expected = "# fmt: markdown\nDOC = \"\"\"\\\n    # Title\n\"\"\"\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_string_markdown_wrap_width_accounts_for_indent() {
    let input = "\
# fmt: markdown
DOC = \"\"\"
    one two three four
\"\"\"
";
    let expected = "\
# fmt: markdown
DOC = \"\"\"
    one two three
    four
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "18"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_string_markdown_wrap_width_uses_visual_indent_width() {
    let wide_space = "\u{3000}";
    let input = format!(
        "\
# fmt: markdown
DOC = \"\"\"
{wide_space}abc de f
\"\"\"
"
    );
    let expected = format!(
        "\
# fmt: markdown
DOC = \"\"\"
{wide_space}abc de
{wide_space}f
\"\"\"
"
    );
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.py", "--wrap", "9"],
        &input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn r_raw_string_can_be_embedded_markdown_target() {
    let input = "\
# fmt: markdown
text <- r\"(
#   Title ##
)\"
";
    let expected = "\
# fmt: markdown
text <- r\"(
# Title
)\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.r", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn r_raw_string_markdown_targets_support_brackets_and_dash_delimiters() {
    let cases = [
        (
            "\
# fmt: markdown
text <- r\"[
#   Title ##
]\"
",
            "\
# fmt: markdown
text <- r\"[
# Title
]\"
",
        ),
        (
            "\
# fmt: markdown
text <- r\"--{
#   Title ##
}--\"
",
            "\
# fmt: markdown
text <- r\"--{
# Title
}--\"
",
        ),
        (
            "\
# fmt: markdown
text <- r'--|
#   Title ##
|--'
",
            "\
# fmt: markdown
text <- r'--|
# Title
|--'
",
        ),
    ];

    for (input, expected) in cases {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.r", "--wrap", "none"],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn embedded_r_markdown_string_keeps_closing_delimiter_at_content_indent() {
    let input = "\
# fmt: markdown
text <- \"
    #   Title ##
    \"
";
    let expected = "\
# fmt: markdown
text <- \"
    # Title
    \"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.r", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn embedded_r_markdown_string_preserves_column_zero_closing_delimiter() {
    let input = "\
# fmt: markdown
text <- \"
    #   Title ##
\"
";
    let expected = "\
# fmt: markdown
text <- \"
    # Title
\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.r", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn r_standard_markdown_string_targets_reject_backslashes() {
    let cases = [
        "# fmt: markdown\ntext <- \"\nline with hard break  \nnext line\n\"\n",
        "\
# fmt: markdown
text <- \"
backslash \\\\ stays unsafe
\"
",
    ];

    for input in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.r"], input);
        assert_eq!(status, 1, "{input}");
        assert_eq!(stdout, "");
        assert!(
            stderr.contains("non-raw R Markdown strings must not contain backslashes"),
            "{stderr}"
        );
    }
}

#[test]
fn verify_reparses_nested_yaml_in_python_markdown() {
    let input = "\
# fmt: markdown
DOC = \"\"\"
```yaml
name:   yamark
```
\"\"\"
";
    let expected = "\
# fmt: markdown
DOC = \"\"\"
```yaml
name: yamark
```
\"\"\"
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--verify", "--stdin-file-path", "input.py"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}
