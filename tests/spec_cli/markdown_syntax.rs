use super::support::run_stdin;
use std::fs;
use tempfile::tempdir;

#[test]
fn markdown_canonical_preserves_unsupported_underscore_constructs() {
    let input = "\
This _literal marker stays.

This __strong marker stays.

This literal_ marker stays.

This literal__ marker stays.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_citations_are_supported_inline_tokens() {
    let input = "\
Alpha beta [see @doe2020] gamma delta epsilon.

Alpha beta [-@doe2020] gamma delta epsilon.
";
    let expected = "\
Alpha beta [see @doe2020]
gamma delta epsilon.

Alpha beta [-@doe2020]
gamma delta epsilon.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "25"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_preserves_non_ascii_whitespace_before_block_markers() {
    let input = "\u{00a0}# heading\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_block_attribute_lines_with_key_value_are_preserved() {
    let input = "\
Paragraph before with enough words.
{width=50%}
Following paragraph with enough words.
";
    let expected = "\
Paragraph before with enough
words.
{width=50%}
Following paragraph with
enough words.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "30"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_setext_headings_accept_single_character_underlines() {
    let input = "\
Title
=

Subtitle
-
";
    let expected = "\
# Title

## Subtitle
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
fn markdown_template_spans_preserve_headings() {
    let input = "\
#   Title {{ keep   spacing }}   ##

Setext {{ keep   spacing }}
====

#   Normal ##
";
    let expected = "\
#   Title {{ keep   spacing }}   ##

Setext {{ keep   spacing }}
====

# Normal
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
fn markdown_canonical_preserves_intraword_underscores_and_code_spans() {
    let input = "This keeps foo_bar_baz and `code_value` but changes _emphasis_.\n";
    let expected = "This keeps foo_bar_baz and `code_value` but changes *emphasis*.\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_canonical_preserves_non_emphasis_inline_tokens() {
    let input = "Keep [x](/_path_) ![alt](/_img_) <https://example.com/_id_> {{< relref \"_slug_\" >}} but change _emphasis_.\n";
    let expected = "Keep [x](/_path_) ![alt](/_img_) <https://example.com/_id_> {{< relref \"_slug_\" >}} but change *emphasis*.\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_canonical_preserves_unsupported_inline_spans_in_headings() {
    let input = "# ~~_keep_~~ and _change_\n";
    let expected = "# ~~_keep_~~ and *change*\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_canonical_preserves_reference_style_links_in_headings() {
    let input = "# [__a__][id] and __x__\n";
    let expected = "# [__a__][id] and **x**\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_canonical_preserves_paired_inline_html_content_in_headings() {
    let input = "# <span>_x_</span> and _y_\n";
    let expected = "# <span>_x_</span> and *y*\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--canonical",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_reference_style_links_preserve_source_spelling() {
    let input = "This [![ alt text ](img.png)][fixture-docs] keeps    spacing.\n";
    let expected = "This [![ alt text ](img.png)][fixture-docs] keeps spacing.\n";
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

    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_inline_validation_ignores_code_span_content() {
    let input = "This  has `code <x>` and  spaces.\n";
    let expected = "This has `code <x>` and spaces.\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "40"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_output_trims_trailing_whitespace_and_ends_with_newline() {
    let input = "First line \t\n\n```text\nkept? \t\n```\nlast line\t";
    let expected = "First line\n\n```text\nkept?\n```\nlast line\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_structured_blocks_preserve_unsupported_inline_source() {
    let input = "\
- This ~~struck text keeps    spacing~~.

> This ~~struck text keeps    spacing~~.

Term
: This ~~struck text keeps    spacing~~.
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
fn markdown_hugo_shortcodes_are_not_generic_template_spans() {
    let input = "Before {{< ref \"target\" >}} after    text.\n";
    let expected = "Before {{< ref \"target\" >}} after text.\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_heading_attributes_normalize_without_preceding_space() {
    let input = "## Title{.a  .b }\n";
    let expected = "## Title {.a .b}\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_preserves_quarto_inline_attribute_at_heading_end() {
    let input = concat!(
        "# [text]{.class}\n",
        "# `code`{.class}\n",
        "# [link](dest){.class}\n",
        "# ![alt](img){.class}\n",
        "# [@cite]{.class}\n",
    );
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_preserves_quarto_inline_html_at_paragraph_start() {
    let input = "<span>raw</span> text\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_preserves_quarto_unicode_whitespace() {
    for input in [
        "alpha\u{a0}beta\n",
        "# alpha\u{a0}\n",
        "- \u{a0}alpha beta\n",
        "> \u{a0}alpha beta\n",
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_preserves_quarto_tab_indented_code() {
    for input in [
        "\tcode\n",
        "\tTitle\n\t====\n",
        "\t---\n",
        "\t***\n",
        "\t___\n",
        "\t> code\n",
        " \t> code\n",
        "  \t> code\n",
        "\t- item\n",
        " \t- item\n",
        "  \t- item\n",
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_preserves_quarto_raw_tex_separator() {
    for input in [
        "before \\LaTeX\nafter\n",
        "before words \\LaTeX after words\n",
        "- before \\LaTeX\n  after\n",
        "> before \\LaTeX\n> after\n",
        "Text.[^x]\n\n[^x]: before \\LaTeX\n  after\n",
    ] {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.qmd", "--wrap", "20"],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_heading_closing_hashes_are_removed_before_attributes() {
    let input = "# Title ### {#id}\n";
    let expected = "# Title {#id}\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_heading_closing_hashes_can_leave_empty_headings() {
    let input = "# #\n### ###\n";
    let expected = "#\n###\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_heading_closing_hashes_before_attributes_can_leave_empty_headings() {
    let input = "# # {#id}\n### ### {#id}\n";
    let expected = "# {#id}\n### {#id}\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_simple_link_and_image_labels_are_normalized() {
    let input = "\
A [ label   text ](dest) and ![ alt   text ](img.png){ fig-alt=\"A   figure\" }.
";
    let expected = "\
A [label text](dest) and ![alt text](img.png){fig-alt=\"A figure\"}.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn markdown_quoted_link_titles_are_supported() {
    let input = "\
A [  label  ](https://example.com \"a title\") after.
";
    let expected = "\
A [label](https://example.com \"a title\") after.
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
fn markdown_parenthesized_link_titles_are_supported() {
    let input = "\
A [  label  ](https://example.com (a title)) after.
";
    let expected = "\
A [label](https://example.com (a title)) after.
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
fn markdown_links_with_escaped_targets_are_preserved() {
    let input = "\
A [  label  ](a\\)b) and [  title  ](dest \"a \\\"title\\\"\") after.
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
fn markdown_spaced_image_fig_alt_attribute_is_normalized_for_pandoc() {
    let input = "\
![](mcp-repl-useR-lightning-assets/chat-scenario.png) { fig-alt=\"A chat interface where a user asks an agent to explore a dataset and build a first model. The agent responds by running several bash tool calls with Rscript.\" }
";
    let expected = "\
![](mcp-repl-useR-lightning-assets/chat-scenario.png){
  fig-alt=\"A chat interface where a user asks an agent to explore a dataset and build a
first model. The agent responds by running several bash tool calls with Rscript.\"
}
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "88"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_long_fig_alt_attributes_preserve_document_line_endings() {
    for newline in ["\r\n", "\r"] {
        let input = format!(
            "![](quarto.png){{fig-alt=\"Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu\"}}{newline}"
        );
        let expected = format!(
            "![](quarto.png){{{newline}  fig-alt=\"Alpha beta gamma delta epsilon zeta{newline}eta theta iota kappa lambda mu\"{newline}}}{newline}"
        );
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.md", "--wrap", "48"],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_quarto_divs_format_supported_children() {
    let input = "\
::: {.callout-note}
#   Keep ##
text   with   spacing
:::

#   Change ##
";
    let expected = "\
::: {.callout-note}
# Keep
text with spacing
:::

# Change
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.qmd",
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
fn markdown_long_links_split_destination_onto_own_line() {
    let input = "\
A [compact label](https://example.com/a/very/long/destination/path) after.
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
fn markdown_generated_split_links_keep_surrounding_paragraph_text() {
    let input = "\
See [the documentation](https://example.com/really/really/really/really/long/path \"A helpful title\") for details.
";
    let expected = "\
See
[the documentation](https://example.com/really/really/really/really/long/path \"A helpful title\")
for details.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "40"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_generated_split_links_split_title_from_destination_when_needed() {
    let input = "\
See [docs](https://e.co/path \"short title\") after.
";
    let expected = "\
See
[docs](https://e.co/path \"short title\")
after.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "32"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_existing_indented_split_links_preserve_split_style() {
    let input = "\
A [compact label](
  https://example.com/a/very/long/destination/path
) after.
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
fn markdown_existing_split_links_normalize_label_without_changing_split_style() {
    let input = "\
A [ label   text ](
https://example.com/a/very/long/destination/path
) after.
";
    let expected = "\
A [label text](
https://example.com/a/very/long/destination/path
) after.
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
fn markdown_links_support_nested_image_labels_when_simple() {
    let input = "\
A [ ![ alt   text ](img.png)   label ](dest) after.
";
    let expected = "\
A [![alt text](img.png) label](dest) after.
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
fn markdown_ambiguous_nested_container_continuations_are_preserved() {
    let input = "\
- foo
  - bar

    is this a top level code quote or a 2nd paragraph on the previous list?

* > * this is a list (quote (list))
    > is this prose or is this a 2nd blockquote ?
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "28"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn unmatched_markdown_code_fence_is_preserved_as_raw_source() {
    let input = "\
```yaml
a:    b
# still inside the unmatched fence
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_code_fence_info_removes_duplicate_bare_language_after_class() {
    let input = "\
```{.yaml yaml}
a:    1
```
";
    let expected = "\
```{.yaml}
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
fn markdown_code_fence_info_normalizes_attribute_language_forms() {
    let input = "\
``` {.haskell .numberLines}
main = pure ()
```

```haskell {.haskell .numberLines #sort}
main = pure ()
```
";
    let expected = "\
```{.haskell .numberLines}
main = pure ()
```

```haskell {.numberLines #sort}
main = pure ()
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
fn markdown_risky_blocks_are_preserved() {
    let input = concat!(
        "    code   keeps   spacing\n\n",
        "$$\n",
        "x   +   y\n",
        "$$\n\n",
        "<!-- ordinary\n",
        "html comment -->\n\n",
        "{{< shortcode param=\"keep   spacing\" >}}\n\n",
        "| pandoc line   block\n",
        "| keeps   spacing\n\n",
        "[ref]: http://example.com/a   \"title\"\n",
    );
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
fn markdown_display_math_with_inline_delimiters_is_preserved() {
    let input = "\
$$ E = mc^2 + very_long_symbol
more math content
$$
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "12"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_single_line_raw_tex_preserves_only_that_line() {
    let input = "\
\\begin{note} keep    raw \\end{note}
This paragraph should wrap to sentence. This one too.
";
    let expected = "\
\\begin{note} keep    raw \\end{note}
This paragraph should wrap to sentence.
This one too.
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
fn markdown_hugo_shortcode_tokens_leave_the_body_as_markdown() {
    let input = "\
{{< notice >}}
This    Markdown body formats normally.
{{< /notice >}}
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
    assert_eq!(stdout, input.replace("This    Markdown", "This Markdown"));
    assert_eq!(stderr, "");
}

#[test]
fn markdown_inline_footnote_refs_stay_attached_to_preceding_punctuation() {
    let input = "\
This has a footnote.[^id]

[^id]: preserved
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "none",
            "--preserve-footnotes",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_preserves_prose_that_looks_like_unsupported_ordered_markers() {
    let input = "\
And. this is a paragraph that should stay as prose.

(abc) this is also prose that should not be a list.
";
    let expected = "\
And. this is a
paragraph that
should stay as
prose.

(abc) this is also
prose that should
not be a list.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn markdown_quarto_chunk_headers_are_promoted_to_option_lines() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.r]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.stdin.buffer.read())\", \"{path}\"], path_suffix = \".R\" }
",
    )
    .unwrap();

    let input = "\
```{r, echo=FALSE, fig.width=8}
1 + 1
```
";
    let expected = "\
```{r}
#| echo: false
#| fig.width: 8
1 + 1
```
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.qmd",
            "--wrap",
            "24",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_quarto_chunk_header_promotion_preserves_document_line_endings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.r]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.stdin.buffer.read())\", \"{path}\"], path_suffix = \".R\" }
",
    )
    .unwrap();

    for newline in ["\r\n", "\r"] {
        let input = format!("```{{r, echo=FALSE, fig.width=8}}{newline}1 + 1{newline}```{newline}");
        let expected = format!(
            "```{{r}}{newline}#| echo: false{newline}#| fig.width: 8{newline}1 + 1{newline}```{newline}"
        );
        let (status, stdout, stderr) = run_stdin(
            &[
                "format",
                "--stdin-file-path",
                "input.qmd",
                "--wrap",
                "24",
                "--config",
                config.to_str().unwrap(),
            ],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn bom_crlf_and_preserve_footnotes_are_preserved() {
    let input = "\u{feff}#   Title ##\r\n\r\n[^a]: keep   spacing. Keep one line.\r\n";
    let expected = "\u{feff}# Title\r\n\r\n[^a]: keep   spacing. Keep one line.\r\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
            "--preserve-footnotes",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn unknown_markdown_code_fences_are_preserved() {
    let input = "\
```unknown    {#cell}
body
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
