use super::support::run_stdin;
use std::fs;
use tempfile::tempdir;

#[test]
fn markdown_tables_lists_blockquotes_and_fences() {
    let input = "\
| a | long |
|---|:---:|
| 1 | two |

*  item with    extra spaces
>quote    text

```yaml
a:    b
```

```yaml fmt: skip
a:    b
```
";
    let expected = "\
| a   | long  |
| --- | :---: |
| 1   |  two  |

- item with extra spaces
> quote text

```yaml
a: b
```

```yaml fmt: skip
a:    b
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
fn markdown_pipe_table_fit_width_aligns_delimiter_cells() {
    let input = "\
| name | value |
| --- | --- |
| short | one |
| long name | two |
";
    let expected = "\
| name      | value |
| --------- | ----- |
| short     | one   |
| long name | two   |
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pipe_table_fit_width_aligns_right_and_center_data_cells() {
    let input = "\
| item | count | state |
| :--- | ---: | :---: |
| a | 1 | ok |
| longer | 1000 | yes |
";
    let expected = "\
| item   | count | state |
| :----- | ----: | :---: |
| a      |     1 |  ok   |
| longer |  1000 |  yes  |
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pipe_table_cells_apply_canonical_inline_formatting() {
    let input = "\
| name | value |
| --- | --- |
| _x_ | __y__ |
";
    let expected = "\
| name | value |
| ---- | ----- |
| *x*  | **y** |
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--canonical"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_definition_lists_with_unsupported_continuations_are_preserved() {
    let input = "\
Term
: definition    first line
    ```r
    x <- 1
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
fn markdown_pipe_tables_preserve_escaped_pipes_and_align_when_wide() {
    let input = "\
| item | value |
| --- | --- |
| a\\|b | 文 |

| key | description |
| --- | --- |
| first | this cell is intentionally long enough to exceed the fit to width threshold when the table formatter would otherwise pad the shorter header column |
";
    let expected = "\
| item | value |
| ---- | ----- |
| a\\|b | 文    |

| key   | description                                                                                                                                        |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| first | this cell is intentionally long enough to exceed the fit to width threshold when the table formatter would otherwise pad the shorter header column |
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pipe_tables_use_unicode_display_width() {
    let input = "\
| a | b |
| --- | --- |
| e\u{301}e\u{301}e\u{301} | x |
| bbbb | y |
";
    let expected = "\
| a    | b   |
| ---- | --- |
| e\u{301}e\u{301}e\u{301}  | x   |
| bbbb | y   |
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pipe_tables_preserve_crlf_line_endings() {
    let input = "| a | long |\r\n|---|:---:|\r\n| 1 | two |\r\n";
    let expected = "| a   | long  |\r\n| --- | :---: |\r\n| 1   |  two  |\r\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pipe_tables_preserve_cr_line_endings() {
    let input = "| a | long |\r|---|:---:|\r| 1 | two |\r";
    let expected = "| a   | long  |\r| --- | :---: |\r| 1   |  two  |\r";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "none"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pipe_tables_ignore_normal_compact_options() {
    let input = "\
| a | long |
|---|:---:|
| 1 | two |
";
    let expected = "\
| a   | long  |
| --- | :---: |
| 1   |  two  |
";

    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--compact"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");

    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "[format]\ncompact = true\n").unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
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
fn markdown_gfm_pipe_tables_allow_body_rows_without_leading_pipes() {
    let input = "\
| name | value |
------ | -----
short | one
long name | two
";
    let expected = "\
| name      | value |
| --------- | ----- |
| short     | one   |
| long name | two   |
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_preserves_inconsistent_pipe_tables() {
    let input = "| A |\n| --- |\n| x | y |\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_tables_handle_incomplete_grid_markers() {
    for input in ["+\n---  ---\nx    y\n", "+---+---+\n|\n+---+---+\n"] {
        let (status, _, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
        assert_eq!(status, 0, "{stderr}");
    }
}

#[test]
fn markdown_pandoc_grid_tables_preserve_unrepresentable_rows() {
    let input = "+---+---+\n| too wide | B |\n+===+===+\n| x | y |\n+---+---+\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn markdown_pandoc_headerless_tables_keep_their_borders_and_following_blocks() {
    for newline in ["\n", "\r\n"] {
        for following in [
            "",
            "Table: keep    caption spacing\n\nFirst sentence. Second sentence.\n",
            ": keep    caption spacing\n\nFirst sentence. Second sentence.\n",
            "#   Following heading\n\nFirst sentence. Second sentence.\n",
        ] {
            for body in ["a  b    c\n", "a  b    c\n        d\n\ne       f\n"] {
                let input = format!("------  ------\n{body}------  ------\n{following}")
                    .replace('\n', newline);
                let expected = input
                    .replace("a  b    c", "a b     c")
                    .replace("#   Following heading", "# Following heading")
                    .replace(
                        "First sentence. Second sentence.",
                        &format!("First sentence.{newline}Second sentence."),
                    );
                let args = [
                    "format",
                    "--table-widths",
                    "preserve",
                    "--stdin-file-path",
                    "input.qmd",
                    "--wrap",
                    "sentence",
                ];
                let (status, stdout, stderr) = run_stdin(&args, &input);
                assert_eq!(status, 0, "{stderr}");
                assert_eq!(stdout, expected, "input: {input:?}");
                let (status, second, stderr) = run_stdin(&args, &stdout);
                assert_eq!(status, 0, "{stderr}");
                assert_eq!(second, stdout, "table formatting must be idempotent");
            }
        }
    }
}

#[test]
fn markdown_pandoc_tables_expand_tabs_before_splitting_cells() {
    let cases = [
        ("A\tB\n---\t---\nx\ty\n", "A   B\n--- ---\nx   y\n"),
        ("A\tB\n--- ---\nx\ty\n", "A   B\n--- ---\nx   y\n"),
        (
            "A\t\tB\n---\t\t---\nx\t\ty\n",
            "A       B\n---     ---\nx       y\n",
        ),
        ("---\t---\na\tb\n---\t---\n", "--- ---\na   b\n--- ---\n"),
        (
            "------------\nA\tB\n---\t---\nx\ty\n\na\tb\n------------\n",
            "------------\nA   B\n--- ---\nx   y\n\na   b\n------------\n",
        ),
        (
            "+-------+-------+\n| A\t\t| B\t\t|\n+=======+=======+\n| x\t\t| y\t\t|\n+-------+-------+\n",
            "+-------+-------+\n| A     | B     |\n+=======+=======+\n| x     | y     |\n+-------+-------+\n",
        ),
    ];
    for (input, expected) in cases {
        for newline in ["\n", "\r\n"] {
            let input = input.replace('\n', newline);
            let expected = expected.replace('\n', newline);
            let args = [
                "format",
                "--table-widths",
                "preserve",
                "--stdin-file-path",
                "input.qmd",
            ];
            let (status, stdout, stderr) = run_stdin(&args, &input);
            assert_eq!(status, 0, "{stderr}");
            assert_eq!(stdout, expected, "input: {input:?}");
            let (status, second, stderr) = run_stdin(&args, &stdout);
            assert_eq!(status, 0, "{stderr}");
            assert_eq!(second, stdout, "table formatting must be idempotent");
        }
    }
}

#[test]
fn markdown_pandoc_tables_preserve_alignment_and_widths() {
    let cases = [
        (
            "A             B\n-----  ------------\nx      y\n",
            "A           B\n-----  ------------\nx           y\n",
        ),
        (
            "+----------+------------------+\n| A        | B                |\n+==========+==================+\n| x  one   | y  two           |\n+----------+------------------+\n",
            "+----------+------------------+\n| A        | B                |\n+==========+==================+\n| x one    | y two            |\n+----------+------------------+\n",
        ),
        (
            "Before.\n\n----------  ----------\nA           B\n----------  ----------\nx  one      y  two\n----------  ----------\n",
            "Before.\n\n----------  ----------\nA           B\n----------  ----------\nx one       y two\n----------  ----------\n",
        ),
    ];
    for (input, expected) in cases {
        for width in ["20", "72"] {
            for newline in ["\n", "\r\n"] {
                let input = input.replace('\n', newline);
                let expected = expected.replace('\n', newline);
                let args = [
                    "format",
                    "--table-widths",
                    "preserve",
                    "--stdin-file-path",
                    "input.qmd",
                    "--wrap",
                    width,
                ];
                let (status, stdout, stderr) = run_stdin(&args, &input);
                assert_eq!(status, 0, "{stderr}");
                assert_eq!(stdout, expected);
                let (status, second, stderr) = run_stdin(&args, &stdout);
                assert_eq!(status, 0, "{stderr}");
                assert_eq!(second, stdout, "table formatting must be idempotent");
            }
        }
    }
}

#[test]
fn markdown_pandoc_simple_tables_and_definition_lists_are_formatted() {
    let input = "\
Name        Value
----------  -----
short       one
long name   two

Term
:   definition with    spacing

<section>
#   Keep ##
</section>
";
    let expected = "\
Name       Value
---------  -----
short      one
long name  two

Term
: definition with spacing

<section>
#   Keep ##
</section>
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
}

#[test]
fn markdown_pandoc_table_cells_apply_canonical_inline_formatting() {
    let input = "\
Name        Value
----------  -----
_x_         __y__

+------+-----+
| Name | Value |
+======+=====+
| _x_ | __y__ |
+------+-----+

----------  ----------
Name        Value
----------  ----------
_x_         __y__
----------  ----------
";
    let expected = "\
Name   Value
-----  -----
*x*    **y**

+------+-------+
| Name | Value |
+======+=======+
| *x*  | **y** |
+------+-------+

-----  ------
Name   Value
-----  ------
*x*    **y**
-----  ------
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--canonical"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_simple_tables_format_optional_closing_separator() {
    let input = "\
Name        Value
----------  -----
short       one
long name   two
----------  -----
";
    let expected = "\
Name       Value
---------  -----
short      one
long name  two
---------  -----
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_table_captions_stop_table_parsing() {
    let input = "\
Name        Value
----------  -----
short       one
long name   two
Table: keep    caption spacing

Term
:   definition with    spacing
";
    let expected = "\
Name       Value
---------  -----
short      one
long name  two
Table: keep    caption spacing

Term
: definition with spacing
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

    let input = "\
Name        Value
----------  -----
short       one
long name   two
: keep    caption spacing
";
    let expected = "\
Name       Value
---------  -----
short      one
long name  two
: keep    caption spacing
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
}

#[test]
fn markdown_definition_list_continuations_are_wrapped_under_definition_indent() {
    let input = "\
Term
:   first line has    spacing
    continuation line has many words that should wrap
";
    let expected = "\
Term
: first line has spacing
  continuation line has
  many words that should
  wrap
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
fn markdown_definition_list_keeps_indented_paragraph_after_blank() {
    let input = "\
Term
: definition one

  second paragraph
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.qmd"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_grid_tables_are_normalized() {
    let input = "\
+------+-----+
| Name | Value |
+======+=====+
| a | one |
+------+-----+
| longer | two |
+------+-----+
";
    let expected = "\
+--------+-------+
| Name   | Value |
+========+=======+
| a      | one   |
+--------+-------+
| longer | two   |
+--------+-------+
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_grid_tables_with_multiline_rows_are_normalized() {
    let input = "\
+------+-----+
| Name | Value |
|      | Extra |
+======+=====+
| a | one |
|   | two |
+------+-----+
";
    let expected = "\
+------+-------+
| Name | Value |
|      | Extra |
+======+=======+
| a    | one   |
|      | two   |
+------+-------+
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_multiline_tables_are_normalized() {
    let input = "\
----------  ----------
Name        Value
----------  ----------
short       one

long name   two
----------  ----------
";
    let expected = "\
---------  ------
Name       Value
---------  ------
short      one

long name  two
---------  ------
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_pandoc_multiline_tables_with_continuous_bounds_are_normalized() {
    let input = "\
-------------------------------------------------------------
Centered   Default           Right Left
Header     Aligned         Aligned Aligned
---------  -------  -------------- -------------------------
First      row                12.0 Example of a row that
                                   spans multiple lines.

Second     row                 5.0 Another row.
-------------------------------------------------------------
";
    let expected = "\
-------------------------------------------------
Centered  Default    Right  Left
Header    Aligned  Aligned  Aligned
--------  -------  -------  ---------------------
First     row         12.0  Example of a row that
                            spans multiple lines.

Second    row          5.0  Another row.
-------------------------------------------------
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}
