use super::support::run_stdin;

#[test]
fn markdown_sentence_column_wrap_is_idempotent_and_preserves_crlf() {
    let input = "Short first. This second sentence contains enough words to wrap across the requested column. Short last.\r\n";
    let expected = "Short first.\r\nThis second sentence contains\r\nenough words to wrap across the\r\nrequested column.\r\nShort last.\r\n";
    let args = [
        "format",
        "--stdin-file-path",
        "input.md",
        "--wrap",
        "sentence:32",
    ];

    let (status, stdout, stderr) = run_stdin(&args, input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");

    let (status, stdout, stderr) = run_stdin(&args, &stdout);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_sentence_column_wrap_rejects_invalid_widths() {
    for wrap in ["sentence:0", "sentence:", "sentence:nope", "sentence:24:32"] {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.md", "--wrap", wrap],
            "One sentence. Another sentence.\n",
        );
        assert_eq!(status, 2, "{wrap}: {stderr}");
        assert_eq!(stdout, "", "{wrap}");
        assert!(
            stderr.contains(
                "wrap must be none, paragraph, sentence, a positive integer, or sentence:<positive integer>"
            ),
            "{wrap}: {stderr}"
        );
    }
}

#[test]
fn markdown_wrapping_escapes_quote_markers_when_no_safe_break_fits() {
    let input = "preamble > marker remains prose after wrapping.\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "9"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(
        stdout,
        "preamble\n\\> marker\nremains\nprose\nafter\nwrapping.\n"
    );
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_escapes_unordered_markers_when_no_safe_break_fits() {
    for marker in ["+ ", "- ", "* "] {
        let input = format!("preamble {marker}marker remains prose after wrapping.\n");
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.md", "--wrap", "9"],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(
            stdout,
            format!("preamble\n\\{marker}marker\nremains\nprose\nafter\nwrapping.\n")
        );
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_wrapping_escapes_ordered_markers_when_no_safe_break_fits() {
    for (marker, escaped_line) in [
        ("1. ", "1\\. marker"),
        ("1) ", "1\\) marker"),
        ("a. ", "a\\. marker"),
        ("i. ", "i\\. marker"),
        ("(a) ", "(a\\)\nmarker"),
    ] {
        let input = format!("preamble {marker}marker remains prose after wrapping.\n");
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.md", "--wrap", "10"],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(
            stdout,
            format!("preamble\n{escaped_line}\nremains\nprose\nafter\nwrapping.\n")
        );
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_wrapping_preserves_inline_tokens_and_unsupported_inline_source() {
    let input = "\
A `code span with spaces` after.

A [label with spaces](dest) after.

This ~~struck text keeps    spacing~~.

This `unterminated code span keeps    spacing.

This [reference][id] keeps    spacing.
";
    let expected = "\
A
`code span with spaces`
after.

A
[label with spaces](
  dest
) after.

This
~~struck text keeps    spacing~~.

This `unterminated code span keeps    spacing.

This
[reference][id]
keeps spacing.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "18"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_wraps_reference_style_links_as_atomic_tokens() {
    let input = "\
This paragraph links to [yamark](https://example.com/yamark) and uses a reference [fixture][fixture-docs] before a footnote.[^long]
";
    let expected = "\
This paragraph links to [yamark](https://example.com/yamark) and uses a
reference [fixture][fixture-docs] before a footnote.[^long]
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "80"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_keeps_latex_commands_with_balanced_braces_together() {
    let input = "This uses \\command{alpha beta gamma} after words.\n";
    let expected = "\
This uses
\\command{alpha beta gamma}
after words.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_keeps_emphasis_spans_together() {
    let input = "This is *very important phrase* after words.\n";
    let expected = "\
This is
*very important phrase*
after words.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "18"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_hard_breaks_inside_structured_blocks_are_normalized() {
    let input = concat!(
        "- first line  \n",
        "  second line\n",
        "\n",
        "> first line  \n",
        "> second line\n",
        "\n",
        "Term\n",
        ": first line  \n",
        "    second line\n",
        "\n",
        "[^note]: first line  \n",
        "    second line\n",
    );
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    let expected = concat!(
        "- first line \\\n",
        "  second line\n",
        "\n",
        "> first line \\\n",
        "> second line\n",
        "\n",
        "Term\n",
        ": first line \\\n",
        "  second line\n",
        "\n",
        "[^note]: first line \\\n",
        "  second line\n",
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_footnote_at_eof_without_final_newline_wraps() {
    let input = "[^long]: This footnote explains that comments were removed because  people used comments to hold parsing directives and enough extra words  to wrap.";
    let expected = concat!(
        "[^long]: This footnote explains that comments were removed because\n",
        "  people used comments to hold parsing directives and enough extra words\n",
        "  to wrap.\n",
    );
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_preserves_paragraphs_when_new_lines_would_start_blocks() {
    let input = "Alpha --- beta\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "5"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_hard_break_spaces_convert_to_backslash_by_default() {
    let input = "line with hard break  \nnext line\n";
    let expected = "line with hard break \\\nnext line\n";
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
fn markdown_hard_break_paragraph_segments_are_wrapped() {
    let input =
        "alpha  \nThis long prose line should wrap after hard break and preserve semantics.\n";
    let expected = "\
alpha \\
This long prose line
should wrap after hard
break and preserve
semantics.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "24"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_hard_break_wrap_none_preserves_soft_line_boundaries() {
    let input = "alpha\nbeta  \ngamma\n";
    let expected = "alpha\nbeta \\\ngamma\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_allows_angle_autolink_continuation_lines() {
    let input = "\
<!-- fmt: wrap=20 scope=next -->
Send mail to <a@example.com> for follow up.
";
    let expected = "\
<!-- fmt: wrap=20 scope=next -->
Send mail to
<a@example.com> for
follow up.
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_keeps_line_start_autolinks_in_prose() {
    let input = "\
First paragraph.
<https://example.com/>.

Contact us.
<a@example.com>.

Second paragraph should wrap into multiple lines now.
";
    let expected = "\
First paragraph.
<https://example.com/>.

Contact us.
<a@example.com>.

Second paragraph
should wrap into
multiple lines now.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wraps_raw_looking_text_inside_protected_inline_spans() {
    let input = "This paragraph includes `C:\\Users\u{a0}name`, [link \\alpha](dest), $\\beta + x$, and <span>HTML\u{a0}text</span> while enough words remain to wrap safely.\n";
    let expected = concat!(
        "This paragraph\n",
        "includes\n",
        "`C:\\Users\u{a0}name`,\n",
        "[link \\alpha](dest),\n",
        "$\\beta + x$, and\n",
        "<span>HTML\u{a0}text</span>\n",
        "while enough words\n",
        "remain to wrap\n",
        "safely.\n",
    );
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.qmd", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_long_fig_alt_attributes_split_and_wrap_when_safe() {
    let input = "\
![](quarto.png){fig-alt=\"Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu\"}
";
    let expected = "\
![](quarto.png){
  fig-alt=\"Alpha beta gamma delta epsilon zeta
eta theta iota kappa lambda mu\"
}
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "48"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_column_wrapping_uses_dominant_crlf_without_final_newline() {
    let input = "Intro line\r\nThis paragraph has enough words to wrap";
    let expected = "Intro line This\r\nparagraph has enough\r\nwords to wrap\r\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_existing_split_links_reflow_when_wrapping() {
    let input = "\
A
[compact label](
https://example.com/a/very/long/destination/path
)
after.
";
    let expected = "\
A [compact label](
  https://example.com/a/very/long/destination/path
) after.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "24"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_inline_code_suffix_stays_attached_when_wrapping() {
    let input = "\
Use `NA`s when values are missing in this paragraph.
";
    let expected = "\
Use `NA`s when
values are missing
in this paragraph.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_link_definitions_are_preserved_and_inline_html_wraps() {
    let input = "\
[ref]: https://example.com
  \"title   spacing\"

Use <span>keep    spacing</span> here.
";
    let expected = "\
[ref]: https://example.com
  \"title   spacing\"

Use
<span>keep    spacing</span>
here.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}
