use super::support::run_stdin;

#[test]
fn markdown_lists_and_blockquotes_reflow_multiline_paragraphs() {
    let input = "\
- This item has many words
  that should be treated as a single paragraph for wrapping.

> This quote has many words
> that should be treated as a single paragraph for wrapping.
";
    let expected = "\
- This item has many words that should
  be treated as a single paragraph for
  wrapping.

> This quote has many words that should
> be treated as a single paragraph for
> wrapping.
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
fn markdown_preserves_quarto_blockquoted_indented_code() {
    for input in [
        ">     code\n",
        "> >     code\n",
        ">>     code\n",
        ">   \tcode\n",
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_footnotes_and_heading_attributes_are_formatted() {
    let input = "\
#   Title   { #id   .class }   ##

[^note]: This is a footnote with many words to wrap.
";
    let expected = "\
# Title {#id .class}

[^note]: This is a
  footnote with many
  words to wrap.
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
fn markdown_multiline_footnotes_are_wrapped_as_footnote_bodies() {
    let input = "\
[^note]: This is a footnote with
    continuation    words that should wrap.
";
    let expected = "\
[^note]: This is a footnote
  with continuation words that
  should wrap.
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
fn markdown_footnote_block_bodies_are_formatted_recursively() {
    let input = "\
[^note]:
    #   Nested title ##
";
    let expected = "\
[^note]:
    # Nested title
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
fn markdown_footnotes_with_multiple_paragraphs_are_wrapped_as_block_bodies() {
    let input = "\
[^note]: First paragraph has many words that should wrap.

    Second paragraph has many words that should wrap.

    Third paragraph has many words that should wrap.
";
    let expected = "\
[^note]:
    First paragraph has many
    words that should wrap.

    Second paragraph has
    many words that should
    wrap.

    Third paragraph has many
    words that should wrap.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "28"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_lists_and_blockquotes_wrap_under_their_markers() {
    let input = "\
- first item has many words that should wrap under marker
> quoted text has many words that should wrap
";
    let expected = "\
- first item has
  many words that
  should wrap under
  marker
> quoted text has
> many words that
> should wrap
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
fn markdown_task_lists_wrap_under_checkbox_content() {
    let input = "\
- [ ] first item has many words that should wrap under the checkbox marker
- [x] done item has many words that should wrap under the checkbox marker
- [X] also done has many words that should wrap under the checkbox marker
";
    let expected = "\
- [ ] first item has many words
      that should wrap under the
      checkbox marker
- [x] done item has many words
      that should wrap under the
      checkbox marker
- [X] also done has many words
      that should wrap under the
      checkbox marker
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
fn markdown_lists_wrap_items_with_nested_lists() {
    let input = "\
- parent item has many words that should wrap before nested children
  - nested item has many words that should wrap under nested marker
- sibling item has many words that should wrap too
";
    let expected = "\
- parent item has many words that
  should wrap before nested children
  - nested item has many words that
    should wrap under nested marker
- sibling item has many words that
  should wrap too
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "36"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_sentence_column_wraps_continuations_in_lists_with_nested_lists() {
    let input = "\
- parent item.
  This continuation sentence has many words that should wrap.
  - nested item has many words that should wrap too.
";
    let expected = "\
- parent item.
  This continuation sentence has many
  words that should wrap.
  - nested item has many words that
    should wrap too.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence:40",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_simple_nested_blockquotes_wrap_under_nested_markers() {
    let input = ">> nested quote has many words that should keep quote depth\n";
    let expected = "\
> > nested quote has
> > many words that
> > should keep quote
> > depth
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "22"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_simple_nested_blockquotes_normalize_markers() {
    let input = "\
>>nested quote    text
> > already    nested
";
    let expected = "\
> > nested quote text
> > already nested
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
fn markdown_blockquotes_with_multiple_paragraphs_wrap_inside_quote_markers() {
    let input = "\
> First paragraph has many words that should wrap.
>
> Second paragraph has many words that should wrap.
>
> Third paragraph has many words that should wrap.
";
    let expected = "\
> First paragraph has many
> words that should wrap.
>
> Second paragraph has many
> words that should wrap.
>
> Third paragraph has many
> words that should wrap.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "28"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_list_continuation_lines_stay_inside_items() {
    let input = "\
- first item has many words
  continuation    paragraph has many words
- second item
";
    let expected = "\
- first item has many words
  continuation paragraph has
  many words
- second item
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
fn markdown_list_continuation_lines_aligned_with_wide_marker_gap_are_wrapped() {
    let input = "\
-   alpha beta
    gamma delta epsilon

3.  alpha beta
    gamma delta
    epsilon zeta
";
    let expected = "\
- alpha beta gamma
  delta epsilon

3. alpha beta gamma
   delta epsilon
   zeta
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
fn markdown_list_second_paragraph_shares_item_indent() {
    let input = "\
-   First paragraph has many words that should wrap under the marker.

    Second   paragraph has many words that should share the same list item indent.
-   Next item stays here.
";
    let expected = "\
- First paragraph has many
  words that should wrap
  under the marker.

  Second paragraph has many
  words that should share
  the same list item indent.
- Next item stays here.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "28"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_lists_with_multiple_paragraphs_share_item_indent() {
    let input = "\
- First paragraph has many words that should wrap.

  Second paragraph has many words that should wrap.

  Third paragraph has many words that should wrap.
- Next item stays here.
";
    let expected = "\
- First paragraph has many
  words that should wrap.

  Second paragraph has many
  words that should wrap.

  Third paragraph has many
  words that should wrap.
- Next item stays here.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "28"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_lists_preserve_unsupported_child_blocks() {
    let input = "\
- item with code
    x   =   1
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
fn markdown_lists_format_reachable_child_code_fences() {
    let input = "\
- item
-
  ```python
  x   =   1
  ```
";
    let expected = "\
- item
-
  ```python
  x = 1
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
fn markdown_lists_format_simple_nested_blockquotes() {
    let input = "\
- item
  > nested quote has many words
";
    let expected = "\
- item
  > nested quote has
  > many words
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
fn markdown_blockquotes_format_reachable_child_code_fences() {
    let input = "\
> quoted intro
> ```python
> x   =   1
> ```
";
    let expected = "\
> quoted intro
> ```python
> x = 1
> ```
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
fn markdown_blockquotes_preserve_nested_lists_when_not_safely_wrappable() {
    let input = "\
> - item
>   - nested
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
fn markdown_alphabetic_roman_and_parenthesized_ordered_lists_are_formatted() {
    let input = "\
a.  alphabetic    item
IV)  roman    item
(1)  parenthesized    item
(a)  parenthesized    alphabetic    item
";
    let expected = "\
a. alphabetic item
IV) roman item
(1) parenthesized item
(a) parenthesized alphabetic item
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn markdown_lists_preserve_cr_line_endings() {
    let input = "*  first    item\r*  second    item\r";
    let expected = "- first item\r- second item\r";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}
