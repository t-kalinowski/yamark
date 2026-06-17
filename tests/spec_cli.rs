use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command as ProcessCommand, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::Command;
use tempfile::tempdir;

fn yamark() -> Command {
    Command::cargo_bin("yamark").unwrap()
}

fn run_stdin(args: &[&str], stdin: &str) -> (i32, String, String) {
    let output = yamark().args(args).write_stdin(stdin).output().unwrap();
    (
        output.status.code().unwrap_or(1),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[cfg(unix)]
fn fake_ruff_path_env(dir: &Path) -> std::ffi::OsString {
    use std::os::unix::fs::PermissionsExt;

    let ruff = dir.join("ruff");
    fs::write(
        &ruff,
        r#"#!/usr/bin/env python3
import json
import sys

if sys.argv[1:3] != ["format", "--stdin-filename"] or sys.argv[4:] != ["-"]:
    sys.stderr.write(f"unexpected argv: {sys.argv[1:]!r}\n")
    sys.exit(2)

if not sys.argv[3].endswith(".ipynb"):
    sys.stderr.write(f"expected ipynb path, got {sys.argv[3]}\n")
    sys.exit(2)

try:
    notebook = json.load(sys.stdin)
except Exception as err:
    sys.stderr.write(f"stdin was not notebook json: {err}\n")
    sys.exit(2)

cells = notebook.get("cells")
if not isinstance(cells, list) or len(cells) != 1:
    sys.stderr.write("expected exactly one notebook cell\n")
    sys.exit(2)

cell = cells[0]
if cell.get("cell_type") != "code":
    sys.stderr.write("expected a code cell\n")
    sys.exit(2)

source = cell.get("source")
if isinstance(source, list):
    source = "".join(source)
elif not isinstance(source, str):
    sys.stderr.write("expected string or array cell source\n")
    sys.exit(2)

if source.startswith("%%tab pytorch\n"):
    formatted = source
else:
    formatted = source.replace("x= 1", "x = 1")

if "%load_ext d2lbook.tab" in source:
    cell["source"] = formatted.splitlines(True)
else:
    cell["source"] = formatted

json.dump(notebook, sys.stdout)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&ruff).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&ruff, permissions).unwrap();

    let mut path_entries = vec![dir.to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(path_entries).unwrap()
}

#[cfg(unix)]
fn failing_ruff_path_env(dir: &Path) -> std::ffi::OsString {
    use std::os::unix::fs::PermissionsExt;

    let ruff = dir.join("ruff");
    fs::write(
        &ruff,
        r#"#!/bin/sh
cat >/dev/null
printf 'error: Failed to parse input.md.embedded.2.ipynb:7:9: Unexpected indentation\n' >&2
printf '> 7 |     x=1\n' >&2
printf '    |         ^\n' >&2
exit 2
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&ruff).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&ruff, permissions).unwrap();

    let mut path_entries = vec![dir.to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(path_entries).unwrap()
}

#[test]
fn format_rejects_unsupported_fast_option() {
    let (status, stdout, stderr) = run_stdin(
        &["format", "--fast", "--stdin-file-path", "input.md"],
        "# Title\n",
    );
    assert_eq!(status, 2);
    assert_eq!(stdout, "");
    assert!(stderr.contains("unexpected argument '--fast'"), "{stderr}");
}

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
fn markdown_paragraphs_that_normalize_to_block_starts_are_escaped() {
    let input = "\u{00a0}# heading\n";
    let expected = "\\# heading\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_preserves_paragraphs_when_quote_marker_would_start_line() {
    let input = "preamble > marker remains prose after wrapping.\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "9"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn markdown_wrapping_preserves_paragraphs_when_unordered_marker_would_start_line() {
    for marker in ["+ ", "- ", "* "] {
        let input = format!("preamble {marker}marker remains prose after wrapping.\n");
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.md", "--wrap", "9"],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_wrapping_preserves_paragraphs_when_ordered_marker_would_start_line() {
    for marker in ["1. ", "1) ", "a. ", "i. ", "(a) "] {
        let input = format!("preamble {marker}marker remains prose after wrapping.\n");
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", "input.md", "--wrap", "10"],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
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
fn markdown_hugo_shortcodes_are_not_generic_template_spans() {
    let input = "Before {{< ref \"target\" >}} after    text.\n";
    let expected = "Before {{< ref \"target\" >}} after text.\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
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
fn markdown_heading_attributes_normalize_without_preceding_space() {
    let input = "## Title{.a  .b }\n";
    let expected = "## Title {.a .b}\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
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
See [the documentation](
  https://example.com/really/really/really/really/long/path
  \"A helpful title\"
) for details.
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
See [docs](
  https://e.co/path
  \"short title\"
) after.
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

    Second paragraph has many
    words that should wrap.

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
Name  Value
----  -----
*x*   **y**

+------+-------+
| Name | Value |
+======+=======+
| *x*  | **y** |
+------+-------+

----  -----
Name  Value
----  -----
*x*   **y**
----  -----
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
fn markdown_hugo_shortcode_blocks_are_preserved_as_raw_blocks() {
    let input = "\
{{< notice >}}
This    Markdown body should stay untouched.
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
    assert_eq!(stdout, input);
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
---------  -----
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
--------  -------  -------  -------------------------------------------
Centered  Default  Right    Left
Header    Aligned  Aligned  Aligned
--------  -------  -------  -------------------------------------------
First     row      12.0     Example of a row that spans multiple lines.

Second    row      5.0      Another row.
--------  -------  -------  -------------------------------------------
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
fn markdown_link_definitions_and_inline_html_are_preserved() {
    let input = "\
[ref]: https://example.com
  \"title   spacing\"

Use <span>keep    spacing</span> here.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.md", "--wrap", "20"],
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

#[test]
fn skipped_external_code_fence_preserves_entire_target_span() {
    let input = "\
```{.python .python #cell}
print('x')
```
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--skip-embedded-formatters",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn missing_optional_external_code_fence_preserves_entire_target_span() {
    let input = "\
```{.python .python #cell}
print('x')
```
";
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .env("PATH", "")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.md:2:1: note: missing optional embedded formatter `ruff`; preserved source"
    ));
}

#[test]
fn missing_configured_ruff_and_air_shorthands_are_diagnostics() {
    for (name, shorthand, diagnostic) in [
        (
            "custom-python",
            "ruff",
            "missing optional embedded formatter `ruff`",
        ),
        (
            "custom-r",
            "air",
            "missing optional embedded formatter `air`",
        ),
    ] {
        let dir = tempdir().unwrap();
        let config = dir.path().join("yamark.toml");
        fs::write(
            &config,
            format!("[embedded.{name}]\nformatter = \"{shorthand}\"\n"),
        )
        .unwrap();
        let input = format!("```{name}\nprint('x')\n```\n");
        let output = yamark()
            .args([
                "format",
                "--diagnostics",
                "--stdin-file-path",
                "input.md",
                "--config",
                config.to_str().unwrap(),
            ])
            .env("PATH", "")
            .write_stdin(input.as_str())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains(diagnostic)
        );
    }
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

#[test]
fn markdown_quarto_chunk_headers_are_promoted_to_option_lines() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.r]
formatter = { command = [\"/bin/sh\", \"-c\", \"cat\", \"{path}\"], path_suffix = \".R\" }
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
formatter = { command = [\"/bin/sh\", \"-c\", \"cat\", \"{path}\"], path_suffix = \".R\" }
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
fn stdin_encoding_errors_are_targeted() {
    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args(["format", "--stdin-file-path", "bad.md"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&[0xff, 0xfe, 0x00, 0x00])
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("bad.md:1:1: error: unsupported encoding: UTF-16 BOM")
    );

    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args(["format", "--stdin-file-path", "bad.md"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(&[0xff]).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("bad.md:1:1: error: invalid UTF-8")
    );
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
            "fmt: wrap must be none, paragraph, sentence, or a positive integer",
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
fn frontmatter_wrapper_markers_are_normalized() {
    let input = "\
---   
title:    Test
---   
Body.
";
    let expected = "\
---
title: Test
---
Body.
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
fn frontmatter_closing_marker_ignores_indented_block_scalar_lines() {
    let input = "\
---
description: |
  ---
  body
---
#   Title ##
";
    let expected = "\
---
description: |
  ---
  body
---
# Title
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn frontmatter_template_delimiter_directive_infers_from_here_at_range_start() {
    let input = "\
---
# fmt: template.delimiters \"<<\" \">>\"

title: \"<< keep   this >>\"
---
Body.
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
fn frontmatter_options_drive_markdown_body() {
    let input = "\
---
title:    Test
editor_options:
  markdown:
    wrap: sentence
    canonical: true
---
This is __strong__. This is _emphasis_.
";
    let expected = "\
---
title: Test
editor_options:
  markdown:
    wrap: sentence
    canonical: true
---
This is **strong**.
This is *emphasis*.
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
fn cli_preserve_footnotes_overrides_frontmatter_footnote_formatting() {
    let input = "\
---
editor_options:
  markdown:
    footnotes: wrap
---
[^note]: keep   spacing. Keep one line.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "20",
            "--preserve-footnotes",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn frontmatter_options_drive_recursive_code_fence_markdown() {
    let input = "\
---
editor_options:
  markdown:
    wrap: sentence
    canonical: true
---
```markdown
This is __strong__. This is _emphasis_.
```

```yaml
body: !markdown |
  This is __strong__. This is _emphasis_.
```
";
    let expected = "\
---
editor_options:
  markdown:
    wrap: sentence
    canonical: true
---
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
fn frontmatter_markdown_options_only_use_supported_paths() {
    let input = "\
---
params:
  wrap: sentence
---
This is one sentence. This is another sentence.
";
    let expected = "\
---
params:
  wrap: sentence
---
This is one sentence. This is another sentence.
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
fn frontmatter_editor_options_markdown_overrides_editor_markdown() {
    let input = "\
---
editor_options:
  markdown:
    wrap: none
editor:
  markdown:
    wrap: sentence
---
This is one sentence. This is another sentence.
";
    let expected = "\
---
editor_options:
  markdown:
    wrap: none
editor:
  markdown:
    wrap: sentence
---
This is one sentence. This is another sentence.
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
fn frontmatter_editor_options_markdown_does_not_merge_editor_markdown_keys() {
    let input = "\
---
editor_options:
  markdown:
    wrap: none
editor:
  markdown:
    canonical: true
---
This is __strong__.
";
    let expected = "\
---
editor_options:
  markdown:
    wrap: none
editor:
  markdown:
    canonical: true
---
This is __strong__.
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
fn frontmatter_skip_file_preserves_entire_markdown_document() {
    let input = "\
---
# fmt: skip file
title:    Test
---
#   Keep ##
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn frontmatter_skip_file_short_circuits_later_invalid_markdown_directives() {
    let input = "\
---
# fmt: skip file
title:    Test
---
<!-- fmt: unknown -->
#   Keep ##
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn frontmatter_template_span_preserves_only_unsafe_yaml_scalar() {
    let input = "\
---
title:    Test
template: \"{{ keep   this }}\"
tags: [docs,frontmatter]
---
#   Body ##
";
    let expected = "\
---
title: Test
template: \"{{ keep   this }}\"
tags: [docs, frontmatter]
---
# Body
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
fn yaml_markdown_template_flow_and_table_directives() {
    let input = "\
# fmt: markdown wrap=none
body: |
  #   Title ##
  text

# fmt: template.delimiters \"<<\" \">>\"
value: \"<< keep   this >>\"

# fmt: table
- {name: a, type: int}
- {name: long_name, type: string}

flow: [ one ,  two]
map: { a : b , long : value }
";
    let expected = "\
# fmt: markdown wrap=none
body: |
  # Title
  text

# fmt: template.delimiters \"<<\" \">>\"
value: \"<< keep   this >>\"

# fmt: table
- {name: a,         type: int}
- {name: long_name, type: string}

flow: [one, two]
map: {a: b, long: value}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_indent_width_applies_to_nested_sequence_expansion() {
    let input = "\
root:
  items: [one, two, three]
";
    let expected = "\
root:
    items:
        - one
        - two
        - three
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "14",
            "--indent-width",
            "4",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_explicit_mapping_values_are_formatted_and_targetable() {
    let input = "\
? key
: [ one ,  two]
";
    let expected = "\
? key
: [one, two]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");

    let input = "\
# fmt: markdown
? key
: |
  #   Title ##
";
    let expected = "\
# fmt: markdown
? key
: |
  # Title
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_template_span_preserves_only_unsafe_scalar() {
    let input = "\
title:    Test
template: \"{{ keep   this }}\"
tags: [docs,yaml]
";
    let expected = "\
title: Test
template: \"{{ keep   this }}\"
tags: [docs, yaml]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_hugo_shortcode_like_scalars_are_template_spans() {
    let input = "template: \"{{< keep   this >}}\"\ntags: [docs,yaml]\n";
    let expected = "template: \"{{< keep   this >}}\"\ntags: [docs, yaml]\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_template_delimiter_directives_accept_spaces_and_escapes() {
    let input = "\
# fmt: template.delimiters \"<< open\" \"close >>\"
value: \"<< open keep   this close >>\"
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");

    let input = r#"# fmt: template.delimiters "[\"" "\"]"
value: '[" keep   this "]'
"#;
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_utf8_bom_is_preserved() {
    let input = "\u{feff}name:    value\n";
    let expected = "\u{feff}name: value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");

    let input = "\u{feff}value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_markdown_directive_rejects_flow_collection_targets() {
    let input = "\
# fmt: markdown
items: [one, two]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("fmt: markdown targets a scalar value"),
        "{stderr}"
    );
}

#[test]
fn yaml_markdown_directive_rejects_alias_targets() {
    let input = "\
anchor: &anchor value
# fmt: markdown
body: *anchor
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("fmt: markdown targets a scalar value"),
        "{stderr}"
    );
}

#[test]
fn yaml_markdown_directive_rejects_explicit_block_scalar_keys() {
    let input = "\
# fmt: markdown
? |
  key
: value
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("fmt: markdown targets a scalar value"),
        "{stderr}"
    );
}

#[test]
fn yaml_embedded_formatter_directive_rejects_explicit_block_scalar_keys() {
    let input = "\
# fmt: json
? |
  key
: value
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("fmt: json targets a literal block scalar"),
        "{stderr}"
    );
}

#[test]
fn yaml_markdown_tag_formats_block_scalar() {
    let input = "\
body: !markdown |
  #   Title ##
  This is __strong__. This is _emphasis_.
";
    let expected = "\
body: !markdown |
  # Title
  This is **strong**. This is *emphasis*.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn yaml_md_tag_formats_block_scalar() {
    let input = "\
body: !md |
  #   Title ##
  This is __strong__. This is _emphasis_.
";
    let expected = "\
body: !md |
  # Title
  This is **strong**. This is *emphasis*.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn yaml_markdown_scalar_template_delimiter_directive_infers_from_here_at_range_start() {
    let input = "\
body: !markdown |
  <!-- fmt: template.delimiters \"<<\" \">>\" -->

  This << keep   spacing >> paragraph.

  Another << keep   spacing >> paragraph.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn yaml_block_scalar_header_comments_are_preserved() {
    let input = "\
body: | # markdown
  # Title
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_nested_markdown_block_scalar_header_comments_are_preserved() {
    let input = "\
# fmt: markdown
body: | # markdown
  #   Title ##
";
    let expected = "\
# fmt: markdown
body: | # markdown
  # Title
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--wrap",
            "none",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_markdown_block_scalars_reindent_formatted_paragraphs() {
    let input = "\
body: !markdown |
  This   paragraph has enough words to wrap across lines and keep YAML indentation.
";
    let expected = "\
body: !markdown |
  This paragraph has enough
  words to wrap across lines
  and keep YAML indentation.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--wrap", "28"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_markdown_folded_scalars_are_emitted_as_literal_blocks() {
    let input = "\
body: !markdown >
  #   Title ##
  This is __strong__.

# fmt: markdown wrap=none
other: >
  A paragraph.
";
    let expected = "\
body: !markdown |
  # Title
  This is __strong__.

# fmt: markdown wrap=none
other: |
  A paragraph.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--wrap",
            "none",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_file_scope_directives_patch_prior_ast_nodes() {
    let input = "\
body: !markdown |
  << keep   spacing >>
items:
  - one
  - two

# fmt: template.delimiters \"<<\" \">>\" scope=file
# fmt: compact scope=file
";
    let expected = "\
body: !markdown |
  << keep   spacing >>
items: [one, two]

# fmt: template.delimiters \"<<\" \">>\" scope=file
# fmt: compact scope=file
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_flow_collections_with_active_template_spans_are_preserved() {
    let input = "\
# fmt: template.delimiters \"<<\" \">>\" scope=file
a: [<< a,b >>]
";
    let expected = input;
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_flow_collections_are_parsed_before_formatting() {
    let input = "\
flow: [ one , [ two , three ], { name : \"a, b\", tags: [x, y] } ] # keep
map: { a : [ one , two ], b : { c : d } }

# fmt: table
- {name: a, desc: \"one, two\"}
- {name: longer, desc: z}

# fmt: table
- {name: a, type: int, default: 1}
- {name: long_name, type: string, default: two}
";
    let expected = "\
flow: [one, [two, three], {name: \"a, b\", tags: [x, y]}] # keep
map: {a: [one, two], b: {c: d}}

# fmt: table
- {name: a,      desc: \"one, two\"}
- {name: longer, desc: z}

# fmt: table
- {name: a,         type: int,    default: 1}
- {name: long_name, type: string, default: two}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_table_directives_align_supported_row_comments() {
    let input = "\
# fmt: table
- {name: a, type: int} # first
- {name: long_name, type: string} # second
";
    let expected = "\
# fmt: table
- {name: a,         type: int   } # first
- {name: long_name, type: string} # second
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_explicit_flow_pairs_preserve_source_spelling() {
    let input = "\
map: {? explicit: entry}
seq: [? key: value]
";
    let expected = input;
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_sequence_item_explicit_mapping_with_flow_key_preserves_structure() {
    let input = "\
- ? [a,b]
  : [c,d]
";
    let expected = "\
- ? [a,b]
  : [c, d]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_flow_sequence_implicit_collection_keys_preserve_structure() {
    let input = "\
items: [{a: b}: c, [x, y]: z]
";
    let expected = input;
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_property_only_nodes_and_commented_markers_preserve_source_spelling() {
    let input = "\
--- # document
root: !!str
seq:
  - &empty
flow: {foo: !!str, anchored: &empty}
... # end
";
    let expected = input;
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_core_collection_tags_are_removed_when_syntax_implies_type() {
    let input = "\
items: !!seq [ one , two ]
mapping: !!map { a : b }
block_seq: !!seq
  - one
  - two
block_map: !!map
  a: b
anchored: &items !!seq [ one ]
";
    let expected = "\
items: [one, two]
mapping: {a: b}
block_seq:
  - one
  - two
block_map:
  a: b
anchored: &items [one]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_custom_collection_tag_and_anchor_order_is_preserved() {
    let input = "\
flow: &defaults !settings { name : yamark }
block: &flags !settings
  enabled: true
";
    let expected = "\
flow: &defaults !settings { name : yamark }
block: &flags !settings
  enabled: true
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_long_safe_prose_scalars_are_folded_and_wrapped() {
    let input = "\
description: This is a long prose scalar with enough words to fold into a block scalar safely
";
    let expected = "\
description: >-
  This is a long prose scalar with enough
  words to fold into a block scalar safely
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "42",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_long_safe_quoted_prose_scalars_are_folded_and_wrapped() {
    let input = "\
description: \"This is a long prose scalar with enough words to fold into a block scalar safely\"
";
    let expected = "\
description: >-
  This is a long prose scalar with enough
  words to fold into a block scalar safely
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "42",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_sequence_item_mapping_prose_scalars_fold_with_valid_body_indent() {
    let input = "\
- description: \"This is a long prose scalar with enough words to fold into a block scalar safely\"
";
    let expected = "\
- description: >-
    This is a long prose scalar with enough
    words to fold into a block scalar safely
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "42",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_sequence_item_mapping_plain_prose_scalars_fold_with_valid_body_indent() {
    let input = "\
- description: This is a long prose scalar with enough words to fold into a block scalar safely
";
    let expected = "\
- description: >-
    This is a long prose scalar with enough
    words to fold into a block scalar safely
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "42",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_escaped_newline_quoted_scalars_emit_literal_blocks() {
    let input = "text: \"line one\\nline two\"\n";
    let expected = "\
text: |-
  line one
  line two
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_escaped_final_newline_quoted_scalars_emit_clipped_literal_blocks() {
    let input = "text: \"line\\n\"\n";
    let expected = "\
text: |
  line
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_existing_folded_prose_scalars_are_rewrapped() {
    let input = "\
description: >-
  This is a folded scalar with enough words to rewrap into several shorter lines
";
    let expected = "\
description: >-
  This is a folded scalar with enough
  words to rewrap into several shorter
  lines
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "38",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_short_folded_prose_scalars_are_rewrapped_only_in_canonical_mode() {
    let input = "\
description: >
  Manage a simple todo
  list.
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");

    let expected = "\
description: >
  Manage a simple todo list.
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--canonical"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_explicit_core_scalar_tags_normalize_quoted_values_when_safe() {
    let input = "\
flag: !!bool \"TRUE\"
empty: !!null \"~\"
count: !!int \"42\"
ratio: !!float \"1.5\"
name: !!str \"true\"
";
    let expected = "\
flag: !!bool true
empty: !!null null
count: !!int 42
ratio: !!float 1.5
name: !!str 'true'
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_explicit_string_tagged_quoted_prose_scalars_are_folded_and_wrapped() {
    let input = "\
description: !!str \"This is a long prose scalar with enough words to fold into a block scalar safely\"
";
    let expected = "\
description: !!str >-
  This is a long prose scalar with enough
  words to fold into a block scalar safely
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "42",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_plain_core_booleans_and_nulls_are_normalized() {
    let input = "\
flag: TRUE
disabled: False
empty: ~
items: [ TRUE , Null ]
";
    let expected = "\
flag: true
disabled: false
empty: null
items: [true, null]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_explicit_core_scalar_tags_are_normalized_when_safe() {
    let input = "\
flag: !!bool TRUE
empty: !!null ~
name: !!str true
tabbed: !!str a\tb
items: [!!bool FALSE, !!null Null, !!str false]
";
    let expected = "\
flag: !!bool true
empty: !!null null
name: !!str 'true'
tabbed: !!str \"a\\tb\"
items: [!!bool false, !!null null, !!str 'false']
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_unsafe_plain_strings_are_quoted_in_block_context() {
    let input = "key:  a\tb\n";
    let expected = "key: \"a\\tb\"\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_document_markers_with_inline_content_parse_the_inline_root() {
    let input = "\
--- !!map
name:    value
--- [ one , two ]
";
    let expected = "\
--- !!map
name: value
--- [one, two]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_diagnostics_report_fast_path_trace_counters() {
    let input = "name: yamark\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--diagnostics", "--stdin-file-path", "input.yaml"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert!(
        stderr.contains(
            "input.yaml:1:1: note: yaml trace: source_scans=1 parse_passes=1 source_lines=1 yaml_scanned_lines=1 yaml_semantic_nodes=3 planned_rendered_scalars=2 planned_rendered_flow_collections=0 planned_rendered_block_flow_collections=0 emitted_bytes=13 emitted_nodes=2"
        ),
        "{stderr}"
    );
}

#[test]
fn yaml_multiline_flow_collections_are_structural() {
    let input = "\
# fmt: table
- {
  name: a,
  desc: \"one, two\"
}
- {
  name: longer,
  desc: z
}
";
    let expected = "\
# fmt: table
- {name: a,      desc: \"one, two\"}
- {name: longer, desc: z}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_block_mapping_sequence_items_are_structural() {
    let input = "\
# fmt: compact table
- name: a
  type: int
- name: long_name
  type: string

items:
  - name: keep
    value: [ one , two]

# fmt: table
- name: preserve
  type: block
after: done
";
    let expected = "\
# fmt: compact table
- {name: a,         type: int}
- {name: long_name, type: string}

items:
  - name: keep
    value: [one, two]

# fmt: table
- name: preserve
  type: block
after: done
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_table_preserves_block_mappings_with_explicit_keys() {
    let input = "\
# fmt: compact table
- ? name
  : a
  type: int
- name: long_name
  type: string
";
    let expected = "\
# fmt: compact table
- ? name
  : a
  type: int
- name: long_name
  type: string
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_respects_line_width() {
    let input = "\
package:
  name: yamark
  language: rust
  license: MIT
";
    let expected = input;
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--compact",
            "--line-width",
            "50",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_collapses_sequence_item_collections_when_they_fit() {
    let input = "\
items:
  -
    name: yamark
    language: rust
";
    let expected = "\
items: [{name: yamark, language: rust}]
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--compact"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_expands_sequence_item_collections_when_sequence_is_too_wide() {
    let input = "\
items:
  -
    name: yamark
    language: rust
  -
    name: yamark
    language: rust
  -
    name: yamark
    language: rust
";
    let expected = "\
items:
  - {name: yamark, language: rust}
  - {name: yamark, language: rust}
  - {name: yamark, language: rust}
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--compact"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_directive_collapses_next_collection_target() {
    let input = "\
root:
  # fmt: compact
  nested:
    name: yamark
    language: rust
  keep:
    name: unchanged
    language: yaml
";
    let expected = "\
root:
  # fmt: compact
  nested: {name: yamark, language: rust}
  keep:
    name: unchanged
    language: yaml
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_directive_accepts_existing_flow_collection_target() {
    let input = "\
# fmt: compact
flow: [ one ,  two]
";
    let expected = "\
# fmt: compact
flow: [one, two]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_unmatched_flow_sequence_opener_at_mapping_value_start_collapses_block_sequence() {
    let input = "\
tags: [
  - llm
  - authoring
  - formats
";
    let expected = "\
tags: [llm, authoring, formats]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_detached_unmatched_flow_opener_after_empty_mapping_value_collapses_child_collection() {
    let input = "\
tags:
[
  - llm
  - authoring
  - formats
";
    let expected = "\
tags: [llm, authoring, formats]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_unmatched_flow_mapping_opener_at_sequence_item_start_collapses_block_mapping_row() {
    let input = "\
items:
  - {name: a
    kind: scalar
  - {name: longer
    kind: sequence
";
    let expected = "\
items:
  - {name: a, kind: scalar}
  - {name: longer, kind: sequence}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_newline_inside_flow_mapping_expands_it_to_block_style() {
    let input = "\
- {a: b,
   c: d}
";
    let expected = "\
- a: b
  c: d
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_newline_inside_flow_sequence_expands_it_to_block_style() {
    let input = "\
items: [one,
  two,
  [three, four]]
";
    let expected = "\
items:
  - one
  - two
  - [three, four]
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_flow_collections_expand_when_they_exceed_line_width() {
    let input = "\
items: [one, two, three, four]
config: {name: yamark, language: rust}
";
    let expected = "\
items:
  - one
  - two
  - three
  - four
config:
  name: yamark
  language: rust
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "18",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_nested_flow_collections_expand_when_outer_expansion_leaves_wide_lines() {
    let input = "\
config: {a: [alpha, beta, gamma]}
";
    let expected = "\
config:
  a:
    - alpha
    - beta
    - gamma
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "24",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_deep_nested_flow_collections_expand_when_lines_stay_wide() {
    let input = "\
config: {outer: {alpha: [one, two, three], beta: [four, five, six]}}
items: [[alpha, beta, gamma], [delta, epsilon, zeta]]
";
    let expected = "\
config:
  outer:
    alpha:
      - one
      - two
      - three
    beta:
      - four
      - five
      - six
items:
  -
    - alpha
    - beta
    - gamma
  -
    - delta
    - epsilon
    - zeta
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "22",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_root_flow_collection_expands_when_it_exceeds_line_width() {
    let input = "[one, two, three, four]\n";
    let expected = "\
- one
- two
- three
- four
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "10",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_sequence_entry_flow_collection_expands_when_it_exceeds_line_width() {
    let input = "\
items:
  - [one, two, three, four]
";
    let expected = "\
items:
  -
    - one
    - two
    - three
    - four
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "16",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_tab_indentation_without_directives_is_preserved() {
    let input = "\tname: value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_tab_indentation_with_fmt_text_in_scalar_is_preserved() {
    let input = "root:\n\tchild: \"this fmt: text is data\"\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_tab_indentation_with_fmt_comment_in_block_scalar_is_preserved() {
    let input = "root: |\n\t# fmt: markdown\n\t#   Title ##\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_tab_indentation_with_preservation_directive_is_preserved() {
    let input = "# fmt: skip\nroot:\n\tchild: value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_tab_indentation_rejects_active_scalar_directive_target() {
    let input = "# fmt: markdown\nkey:\n\tchild: value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("tabs in YAML indentation are unsupported"),
        "{stderr}"
    );
}

#[test]
fn yaml_tab_indentation_after_unrelated_file_scope_directive_is_preserved() {
    let input = "\
# fmt: compact scope=file
items:
  - a
  - b
\troot:
\t  child: value
";
    let expected = "\
# fmt: compact scope=file
items: [a, b]
\troot:
\t  child: value
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_skip_file_short_circuits_later_invalid_leading_fmt_comments() {
    let input = "\
# fmt: skip file
# fmt: unknown
a:    b
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_directive_errors_are_reported() {
    let cases = [
        ("# fmt:\na: b\n", "invalid fmt directive"),
        ("# fmt: on\na: b\n", "fmt: on without active fmt: off"),
        (
            "# fmt: off\n# fmt: off\na:    b\n# fmt: on\n",
            "nested fmt: off",
        ),
        ("# fmt: off\na:    b\n", "unterminated fmt: off"),
        ("# fmt: skip\n", "fmt: skip has no target"),
        (
            "# fmt: skip scope=from-here\na: b\n",
            "fmt: skip does not support scope=from-here",
        ),
        (
            "# fmt: template.delimiters \"<<\"\na: b\n",
            "fmt: template.delimiters requires exactly two quoted delimiter strings",
        ),
        (
            "# fmt: template.delimiters \"<<\" \">>\"\n# ordinary\nvalue: << keep >>\n",
            "fmt: template.delimiters needs explicit scope",
        ),
        (
            "# fmt: markdown scope=file\nbody: text\n",
            "fmt: markdown with scope=file requires at least one option",
        ),
        (
            "# fmt: json\nbody: inline\n",
            "fmt: json targets a literal block scalar",
        ),
        (
            "# fmt: custom\nbody: |\n  inline\n",
            "unknown embedded formatter: custom",
        ),
        (
            "# fmt: json\nbody: >\n  inline\n",
            "fmt: json targets a literal block scalar",
        ),
        (
            "name: value # fmt: skip\nnext: value\n",
            "fmt directive is not supported in this position",
        ),
        (
            "- value # fmt: skip\n- next\n",
            "fmt directive is not supported in this position",
        ),
        (
            "name: value # fmt: wrap=0\n",
            "fmt: wrap must be none, paragraph, sentence, or a positive integer",
        ),
    ];

    for (input, message) in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 1, "{input}");
        assert_eq!(stdout, "");
        assert!(stderr.contains(message), "{stderr}");
    }
}

#[test]
fn yaml_same_line_table_directive_targets_empty_parent_sequence() {
    let input = "\
items: # fmt: table
  - {name: a, type: int}
  - {name: long_name, type: string}
";
    let expected = "\
items: # fmt: table
  - {name: a,         type: int}
  - {name: long_name, type: string}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_quotes_plain_scalars_that_are_unsafe_in_flow_context() {
    let input = "\
item:
  text: a, b
  label: colon: value
  hash: a
";
    let expected = "\
item: {text: 'a, b', label: 'colon: value', hash: a}
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--compact"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_compact_quotes_plain_scalars_with_control_characters() {
    let input = "item:\n  text: !!str a\u{0007}b\n";
    let expected = "item: {text: !!str \"a\\ab\"}\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--compact"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_inline_markdown_scalar_is_converted_to_literal_block() {
    let input = "\
# fmt: markdown wrap=sentence canonical=true
body: This is __strong__. This is _emphasis_.
";
    let expected = "\
# fmt: markdown wrap=sentence canonical=true
body: |
  This is **strong**.
  This is *emphasis*.
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn yaml_inline_markdown_scalar_uses_dominant_line_ending_without_final_newline() {
    let input = "# fmt: markdown wrap=none\r\nbody: This is __strong__ text.";
    let expected = "# fmt: markdown wrap=none\r\nbody: |\r\n  This is __strong__ text.\r\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_broad_markdown_scope_sets_options_without_marking_unmarked_scalars() {
    let input = "\
# fmt: markdown canonical=true scope=file
plain: \"This is __not__ markdown.\"
# fmt: markdown
body: \"This is __strong__.\"
";
    let expected = "\
# fmt: markdown canonical=true scope=file
plain: This is __not__ markdown.
# fmt: markdown
body: |
  This is **strong**.
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_option_only_markdown_directive_defaults_to_next_scope() {
    let input = "\
# fmt: canonical=true
first: !markdown \"This is __strong__.\"
second: !markdown \"This is __strong__.\"
";
    let expected = "\
# fmt: canonical=true
first: !markdown |
  This is **strong**.
second: !markdown |
  This is __strong__.
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--wrap",
            "none",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_file_scope_markdown_options_patch_prior_markdown_scalars() {
    let input = "\
body: !markdown |
  This is __strong__ before the directive.
# fmt: markdown
other: |
  This is __marked__ before the directive.

# fmt: markdown canonical=true scope=file
";
    let expected = "\
body: !markdown |
  This is **strong** before the directive.
# fmt: markdown
other: |
  This is **marked** before the directive.

# fmt: markdown canonical=true scope=file
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--wrap",
            "none",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_inline_markdown_scalars_use_decoded_scalar_values() {
    let input = "\
# fmt: markdown wrap=none canonical=true
body: \"This is __strong__ text.\"
empty: !markdown \"\"
";
    let expected = "\
# fmt: markdown wrap=none canonical=true
body: |
  This is **strong** text.
empty: !markdown \"\"
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_empty_markdown_scalars_emit_empty_string() {
    let input = "\
# fmt: markdown
body:
empty: !markdown |
tag_only: !markdown
";
    let expected = "\
# fmt: markdown
body: \"\"
empty: !markdown \"\"
tag_only: !markdown \"\"
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_generated_quotes_prefer_fewer_escapes() {
    let input = "\
values:
  - a'b, c
  - a\"b, c
";
    let expected = "\
values: [\"a'b, c\", 'a\"b, c']
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--stdin-file-path", "input.yaml", "--compact"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_table_directives_wait_for_sequence_targets() {
    let input = "\
# fmt: table
note:    keep
rows:
  - {name: a, type: int}
  - {name: long_name, type: string}
";
    let expected = "\
# fmt: table
note: keep
rows:
  - {name: a,         type: int}
  - {name: long_name, type: string}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_table_directive_flow_sequence_targets_are_preserved() {
    let input = "\
# fmt: table
[{name: a,type: int}]
";
    let expected = input;
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_table_directives_without_sequence_targets_error() {
    let cases = [
        ("# fmt: table\nname: value\n", "fmt: table has no target"),
        (
            "# fmt: compact table\nname: value\n",
            "fmt: compact table has no target",
        ),
    ];
    for (input, message) in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 1, "{input}");
        assert_eq!(stdout, "");
        assert!(stderr.contains(message), "{stderr}");
    }
}

#[test]
fn yaml_skip_preserves_nested_target_node_with_leading_trivia() {
    let input = "\
# fmt: skip
root:
  name:    keep
  tags: [keep,spacing]
other:    change
";
    let expected = "\
# fmt: skip
root:
  name:    keep
  tags: [keep,spacing]
other: change
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_inline_comments_are_metadata_not_scalar_content() {
    let input = "\
# fmt: skip
root: # keep root with child
  child:    value
other: [ one ,  two ] # keep flow comment
items:
- item # keep scalar comment
- # keep empty item comment
";
    let expected = "\
# fmt: skip
root: # keep root with child
  child:    value
other: [one, two] # keep flow comment
items:
  - item # keep scalar comment
  - # keep empty item comment
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_unsupported_flow_syntax_is_preserved_without_directives() {
    let input = "\
flow: {
  a:    b, # comment
  c:    d
}
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_unsupported_suite_syntax_is_preserved_without_directives() {
    let cases = [
        "%FOO  bar baz # Should be ignored\n---\n\"foo\"\n",
        "--- |-\n ab\n \n...\n",
        "key ends with two colons::: value\n",
        "- bla\"keks: foo\n- bla]keks: foo\n",
        "key: &anchor\n !!map\n  a: b\n",
        "a: &:@*!$\"<foo>: scalar a\nb: *:@*!$\"<foo>:\n",
        "'implicit block key' : [\n  'implicit flow key' : value,\n ]\n",
        "- sun: yellow\n- ? earth: blue\n  : moon: white\n",
    ];

    for input in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{input}: {stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn paths_modes_config_and_skips_are_deterministic() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("yamark.toml"),
        "[format]\nmarkdown_horizontal_rule = \"***\"\n",
    )
    .unwrap();
    fs::write(root.join("a.md"), "#   Title ##\n\n---\n").unwrap();
    fs::write(root.join("ignored.txt"), "#   Title ##\n").unwrap();

    let output = yamark()
        .args([
            "format",
            root.join("a.md").to_str().unwrap(),
            root.join("ignored.txt").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 files scanned, 1 formatted, 0 unchanged, 1 skipped, 0 failed\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("a.md")).unwrap(),
        "# Title\n\n***\n"
    );

    fs::write(root.join("a.md"), "#   Title ##\n").unwrap();
    let output = yamark()
        .args(["format", "--check", root.join("a.md").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );

    let output = yamark()
        .args(["format", "--diff", root.join("a.md").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("@@ -1 +1 @@\n-#   Title ##\n+# Title\n")
    );
}

#[test]
fn format_skips_cmd_files() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("build.cmd");
    let input = "Title\n=====\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 0 unchanged, 1 skipped, 0 failed\n"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_prints_contextual_unified_hunks() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("input.yaml");
    let input = "\
top: one
a: 1
b: 2
c: 3
d: 4
e: 5
f: 6
items: [a,b]
";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", "--diff", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--- "));
    assert!(stdout.contains("+++ "));
    assert!(
        stdout.contains("@@ -5,4 +5,4 @@\n d: 4\n e: 5\n f: 6\n-items: [a,b]\n+items: [a, b]\n"),
        "{stdout}"
    );
    assert!(!stdout.contains(" top: one\n a: 1\n b: 2\n"), "{stdout}");
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_reports_missing_final_newline_changes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("input.md");
    let input = "# Title";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", "--diff", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("@@ -1 +1 @@\n-# Title\n\\ No newline at end of file\n+# Title\n"),
        "{stdout}"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_handles_large_inputs_without_quadratic_memory() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.yaml");
    let mut input = String::new();
    for i in 0..12_000 {
        input.push_str(&format!("key_{i}: value\n"));
    }
    input.push_str("items: [a,b]\n");
    for i in 12_000..24_000 {
        input.push_str(&format!("key_{i}: value\n"));
    }
    fs::write(&path, &input).unwrap();

    let output = yamark()
        .args(["format", "--diff", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("-items: [a,b]\n+items: [a, b]\n"),
        "{stdout}"
    );
    assert!(
        !stdout.contains(" key_0: value\n key_1: value\n"),
        "{stdout}"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_resyncs_after_large_fallback_insertions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.yaml");
    let mut input = String::new();
    let items = (0..100)
        .map(|i| format!("item_{i}"))
        .collect::<Vec<_>>()
        .join(",");
    input.push_str("items: [");
    input.push_str(&items);
    input.push_str("]\n");
    for i in 0..3_000 {
        input.push_str(&format!("key_{i}: value\n"));
    }
    input.push_str("second: [c,d]\n");
    fs::write(&path, &input).unwrap();

    let output = yamark()
        .args([
            "format",
            "--diff",
            "--line-width",
            "40",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let hunk_count = stdout.matches("@@ ").count();
    assert_eq!(hunk_count, 2);
    assert!(stdout.contains("-items: [item_0,item_1,"));
    assert!(stdout.contains("+items:\n+  - item_0\n+  - item_1\n"));
    assert!(stdout.contains("-second: [c,d]\n+second: [c, d]\n"));
    assert!(!stdout.contains("key_1500: value\n"));
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_handles_repeated_large_fallback_syncs_without_hanging() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.yaml");
    let mut input = String::new();
    let items = (0..128)
        .map(|i| format!("item_{i}"))
        .collect::<Vec<_>>()
        .join(",");
    for block in 0..400 {
        input.push_str(&format!("items_{block}: [{items}]\n"));
        input.push_str(&format!("key_{block}: value\n"));
    }
    fs::write(&path, &input).unwrap();

    let stdout_path = dir.path().join("diff.out");
    let stdout_file = fs::File::create(&stdout_path).unwrap();
    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args([
            "format",
            "--diff",
            "--line-width",
            "40",
            path.to_str().unwrap(),
        ])
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(2) {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("diff mode hung on repeated large fallback syncs");
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = fs::read_to_string(stdout_path).unwrap();
    assert!(stdout.contains("-items_0: [item_0,item_1,"));
    assert!(stdout.contains("+items_0:\n+  - item_0\n+  - item_1\n"));
    assert!(stdout.contains("-items_399: [item_0,item_1,"));
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[cfg(unix)]
#[test]
fn external_formatter_handles_large_stdin_stdout_without_deadlock() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let formatter = dir.path().join("echo-stdin");
    fs::write(&formatter, "#!/bin/sh\ncat\n").unwrap();
    let mut permissions = fs::metadata(&formatter).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&formatter, permissions).unwrap();

    fs::write(
        dir.path().join("yamark.toml"),
        format!(
            "[embedded.echo]\nformatter = {{ command = [\"{}\", \"{{path}}\"], path_suffix = \".txt\" }}\n",
            formatter.display()
        ),
    )
    .unwrap();
    let path = dir.path().join("large.md");
    let payload = "a".repeat(4 * 1024 * 1024);
    let input = format!("```echo\n{payload}\n```\n");
    fs::write(&path, &input).unwrap();

    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args(["format", path.to_str().unwrap()])
        .current_dir(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(5) {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("external formatter deadlocked on large stdin/stdout");
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn git_filter_formats_only_markdown() {
    let (status, stdout, stderr) = run_stdin(
        &["git-filter", "clean", "--stdin-filename", "input.md"],
        "One sentence. Two sentences.\n",
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "One sentence.\nTwo sentences.\n");

    let (status, stdout, stderr) = run_stdin(
        &["git-filter", "clean", "--stdin-filename", "table.md"],
        "| a | long |\n|---|:---:|\n| 1 | two |\n",
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "| a | long |\n| --- | :---: |\n| 1 | two |\n");

    let (status, stdout, stderr) = run_stdin(
        &["git-filter", "clean", "--stdin-filename", "input.txt"],
        "#   Title ##\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("unsupported Git filter path"));
}

#[test]
fn invalid_config_keys_fail() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "[format]\nunknown = true\n").unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains(&format!("{}:1:1", config.display())),
        "{stderr}"
    );
    assert!(stderr.contains("unknown format config key: format.unknown"));
}

#[test]
fn expanded_embedded_formatter_requires_path_suffix() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[embedded.custom]\nformatter = { command = [\"tool\", \"{path}\"] }\n",
    )
    .unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("missing embedded formatter path_suffix"));
}

#[test]
fn embedded_formatter_entries_require_formatter_key() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[embedded.custom]\ncommand = [\"tool\", \"{path}\"]\n",
    )
    .unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("embedded formatter entries must contain formatter"),
        "{stderr}"
    );
}

#[test]
fn embedded_config_must_be_a_table() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "embedded = true\n").unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("embedded config must be a table"),
        "{stderr}"
    );
}

#[test]
fn template_delimiter_entries_reject_unknown_fields() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[template]\nadd_delimiters = [{ open = \"<<\", close = \">>\", extra = true }]\n",
    )
    .unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("unknown template delimiter key: template.add_delimiters.extra"),
        "{stderr}"
    );
}

#[test]
fn path_template_delimiter_entries_reject_unknown_fields() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\", extra = true }]\n",
    )
    .unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "docs/input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("unknown template delimiter key: paths.docs.template.add_delimiters.extra"),
        "{stderr}"
    );
}

#[test]
fn bom_prefixed_first_line_directives_apply() {
    let cases = [
        (
            "input.yaml",
            "\u{feff}# fmt: skip file\na:    b\n",
            "\u{feff}# fmt: skip file\na:    b\n",
        ),
        (
            "input.md",
            "\u{feff}<!-- fmt: skip file -->\n#   Title ##\n",
            "\u{feff}<!-- fmt: skip file -->\n#   Title ##\n",
        ),
        (
            "input.py",
            "\u{feff}# fmt: markdown\n# #   Title ##\n",
            "\u{feff}# fmt: markdown\n# # Title\n",
        ),
        (
            "input.r",
            "\u{feff}# fmt: markdown\n# #   Title ##\n",
            "\u{feff}# fmt: markdown\n# # Title\n",
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
    }
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
fn source_files_reject_bare_external_formatter_directives() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.upper]
formatter = { command = [\"/bin/sh\", \"-c\", \"tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: upper\nDOC = \"\"\"\nabc\n\"\"\"\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.py",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("invalid fmt directive: upper"), "{stderr}");
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
fn file_contents_are_valid_utf8() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.md");
    fs::write(&path, [0xff, 0xfe, 0x00, 0x00]).unwrap();
    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unsupported encoding: UTF-16 BOM")
    );
}

#[test]
fn explicit_symlink_targets_are_deduplicated_by_canonical_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    let link = dir.path().join("link.md");
    fs::write(&path, "#   Title ##\n").unwrap();
    make_symlink(&path, &link);

    let output = yamark()
        .args(["format", path.to_str().unwrap(), link.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
}

#[cfg(unix)]
#[test]
fn explicit_symlink_directories_are_walked() {
    let dir = tempdir().unwrap();
    let target_dir = dir.path().join("target");
    let link_dir = dir.path().join("link");
    fs::create_dir(&target_dir).unwrap();
    let path = target_dir.join("a.md");
    fs::write(&path, "#   Title ##\n").unwrap();
    std::os::unix::fs::symlink(&target_dir, &link_dir).unwrap();

    let output = yamark()
        .args(["format", link_dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), "# Title\n");
}

#[cfg(unix)]
#[test]
fn explicit_parent_symlink_directory_walk_does_not_recurse_through_nested_symlink() {
    let dir = tempdir().unwrap();
    let parent = dir.path().join("parent");
    let child = parent.join("child");
    let up = child.join("up");
    fs::create_dir(&parent).unwrap();
    fs::create_dir(&child).unwrap();
    fs::write(parent.join("parent.md"), "#   Parent ##\n").unwrap();
    fs::write(child.join("child.md"), "#   Child ##\n").unwrap();
    std::os::unix::fs::symlink("..", &up).unwrap();

    let output = yamark()
        .args(["format", up.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 files scanned, 2 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
    assert_eq!(
        fs::read_to_string(parent.join("parent.md")).unwrap(),
        "# Parent\n"
    );
    assert_eq!(
        fs::read_to_string(child.join("child.md")).unwrap(),
        "# Child\n"
    );
}

#[test]
fn directory_diagnostics_are_deterministic_for_multiple_yaml_failures() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.yaml");
    let b = dir.path().join("b.yaml");
    fs::write(&a, "# fmt: markdown\nitems: [one, two]\n").unwrap();
    fs::write(&b, "# fmt: markdown\nitems: [one, two]\n").unwrap();

    let output = yamark()
        .args(["format", "--diagnostics", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        "2 files scanned, 0 formatted, 0 unchanged, 0 skipped, 2 failed\n"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    let first = stderr.find(&format!("{}:2:8", a.display())).unwrap();
    let second = stderr.find(&format!("{}:2:8", b.display())).unwrap();
    assert!(first < second, "{stderr}");
}

#[test]
fn path_specific_template_config_preserves_matching_blocks() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This << keep   this >> paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 1 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn path_specific_template_config_applies_during_relative_directory_scan() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This << keep   this >> paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "."])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 files scanned, 0 formatted, 1 unchanged, 1 skipped, 0 failed\n"
    );
}

#[test]
fn directory_formatting_starts_before_later_config_discovery_fails() {
    let dir = tempdir().unwrap();
    let marker = dir.path().join("formatter-ran");
    let formatter = dir.path().join("formatter.sh");
    fs::write(&formatter, "#!/bin/sh\ntouch \"$1\"\ncat\n").unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        format!(
            "[embedded.txt]\nformatter = {{ command = [\"/bin/sh\", {:?}, {:?}, \"{{path}}\"], path_suffix = \".txt\" }}\n",
            formatter.to_str().unwrap(),
            marker.to_str().unwrap(),
        ),
    )
    .unwrap();
    fs::write(dir.path().join("a.md"), "```txt\nabc\n```\n").unwrap();

    let later = dir.path().join("z");
    fs::create_dir(&later).unwrap();
    fs::write(later.join("yamark.toml"), "[format]\nunknown = true\n").unwrap();
    fs::write(later.join("b.md"), "# Title\n").unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "."])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unknown format config key: format.unknown")
    );
    assert!(marker.is_file());
}

#[test]
fn discovered_nested_config_path_configs_match_relative_to_config_dir() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("sub");
    let docs = subdir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(
        subdir.join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"%%\", close = \"%%\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This %% keep   this %% paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "sub/docs/a.md"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 1 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn explicit_config_path_configs_match_relative_to_config_dir() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("configs");
    let docs = dir.path().join("docs");
    fs::create_dir(&config_dir).unwrap();
    fs::create_dir(&docs).unwrap();
    fs::write(
        config_dir.join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"%%\", close = \"%%\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    fs::write(&path, "This %% keep   this %% paragraph.\n").unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "--config", "configs/yamark.toml", "docs/a.md"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "This %% keep this %% paragraph.\n"
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn relative_explicit_config_applies_path_config_to_absolute_paths() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This << keep   this >> paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "--config", "yamark.toml", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 1 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn path_specific_embedded_markdown_template_config_preserves_targets() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".embedded_markdown.template]\nadd_delimiters = [{ open = \"[[\", close = \"]]\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.py");
    let input = "# fmt: markdown\n# This [[ keep   spacing ]] paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
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
fn non_spec_hard_break_space_config_is_rejected() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[format]\npreserve_markdown_hard_break_spaces = true\n",
    )
    .unwrap();
    let input = "line with hard break  \nnext line\n";
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
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("unknown format config key: format.preserve_markdown_hard_break_spaces")
    );
}

#[test]
fn explicit_embedded_formatters_run_for_yaml_scalars_and_code_fences() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.upper]
formatter = { command = [\"/bin/sh\", \"-c\", \"tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }

[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: upper\nbody: |\n  abc\n";
    let expected = "# fmt: upper\nbody: |\n  ABC\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);

    let input = "```txt\nabc\n```\n";
    let expected = "```txt\nABC\n```\n";
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

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
            "--skip-embedded-formatters",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn yaml_embedded_formatter_directive_accepts_explicit_scope_next() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt scope=next\nbody: |\n  abc\n";
    let expected = "# fmt: txt scope=next\nbody: |\n  ABC\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn external_yaml_scalar_formatter_output_is_reindented() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf 'x\\n'\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |\n  y\n";
    let expected = "# fmt: txt\nbody: |\n  x\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn external_yaml_scalar_formatter_preserves_block_header_comments() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf 'x\\n'\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: | # markdown\n  y\n";
    let expected = "# fmt: txt\nbody: | # markdown\n  x\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn external_yaml_scalar_formatter_output_updates_strip_chomp_for_final_newline() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf 'x\\n'\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |-\n  y\n";
    let expected = "# fmt: txt\nbody: |\n  x\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn external_yaml_scalar_formatter_output_uses_explicit_block_indent_indicator() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf 'x\\n'\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |4\nnext: 1\n";
    let expected = "# fmt: txt\nbody: |4\n    x\nnext: 1\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn external_formatter_output_without_trailing_newline_stays_line_delimited() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf X\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |\n  y\nnext: 1\n";
    let expected = "# fmt: txt\nbody: |\n  X\nnext: 1\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);

    let input = "```txt\nabc\n```\n";
    let expected = "```txt\nX\n```\n";
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
}

#[test]
fn external_yaml_scalar_formatter_receives_dedented_block_content() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"sed 's/^/seen:/'\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |\n    value\n";
    let expected = "# fmt: txt\nbody: |\n    seen:value\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn external_yaml_scalar_formatter_preserves_renderer_preamble() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf '%s\\n' \\\"$0\\\"; tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "\
# fmt: txt
script: |
  #| echo: false
  #@ trace
  abc
";
    let expected = "\
# fmt: txt
script: |
  #| echo: false
  #@ trace
  input.yaml.embedded.3.txt
  ABC
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn skipped_external_yaml_scalar_preserves_entire_target_span() {
    let input = "# fmt: json\nbody:    |\n  {\"name\":\"yamark\"}\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--skip-embedded-formatters",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn missing_optional_external_yaml_scalar_preserves_entire_target_span() {
    let input = "# fmt: json\nbody:    |\n  {\"name\":\"yamark\"}\n";
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.yaml"])
        .env("PATH", "")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.yaml:3:1: note: missing optional embedded formatter `prettier`; preserved source"
    ));
}

#[test]
fn configured_prettier_shorthand_is_optional_when_missing() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[embedded.custom-json]\nformatter = \"prettier-json\"\n",
    )
    .unwrap();
    let input = "# fmt: custom-json\nbody:    |\n  {\"name\":\"yamark\"}\n";

    let output = yamark()
        .args([
            "format",
            "--diagnostics",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ])
        .env("PATH", "")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.yaml:3:1: note: missing optional embedded formatter `prettier`; preserved source"
    ));
}

#[test]
fn external_formatter_receives_source_based_virtual_path_and_not_renderer_preamble() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf '%s\\n' \\\"$0\\\"; tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "```txt\n#| label: fig\nabc\n```\n";
    let expected = "```txt\n#| label: fig\ninput.md.embedded.2.txt\nABC\n```\n";
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
}

#[test]
fn external_formatter_preamble_split_supports_cr_only_code_fences() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf '%s\\n' \\\"$0\\\"; tr a-z A-Z\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "```txt\r#| label: fig\rabc\r```\r";
    let expected = "```txt\r#| label: fig\rinput.md.embedded.2.txt\nABC\r```\r";
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
}

#[test]
fn code_fence_formatter_output_cannot_close_the_original_fence() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf '```\\\\n'\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("formatted code fence would contain closing fence"));
}

#[cfg(unix)]
#[test]
fn code_fence_formatter_output_cannot_close_original_fence_with_cr_only_lines() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let formatter = dir.path().join("formatter");
    fs::write(&formatter, "#!/bin/sh\nprintf 'ok\\r```\\r'\n").unwrap();
    let mut permissions = fs::metadata(&formatter).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&formatter, permissions).unwrap();

    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        format!(
            "\
[embedded.txt]
formatter = {{ command = [{:?}, \"{{path}}\"], path_suffix = \".txt\" }}
",
            formatter.to_str().unwrap()
        ),
    )
    .unwrap();

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("formatted code fence would contain closing fence"));
}

#[cfg(unix)]
#[test]
fn default_embedded_formatter_mappings_run_for_yaml_scalars_and_code_fences() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let prettier = dir.path().join("prettier");
    fs::write(&prettier, "#!/bin/sh\ntr a-z A-Z\n").unwrap();
    let mut permissions = fs::metadata(&prettier).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&prettier, permissions).unwrap();
    let mut path_entries = vec![dir.path().to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(path_entries).unwrap();

    let output = yamark()
        .args(["format", "--stdin-file-path", "input.yaml"])
        .env("PATH", &path)
        .write_stdin("# fmt: json\nbody: |\n  {\"name\":\"yamark\"}\n")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "# fmt: json\nbody: |\n  {\"NAME\":\"YAMARK\"}\n"
    );

    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin("```json\n{\"name\":\"yamark\"}\n```\n")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "```json\n{\"NAME\":\"YAMARK\"}\n```\n"
    );
}

#[cfg(unix)]
#[test]
fn python_code_fence_with_line_magic_formats_through_ruff_notebook_cell() {
    let dir = tempdir().unwrap();
    let path = fake_ruff_path_env(dir.path());

    let input = "\
```python
%load_ext d2lbook.tab
x= 1
```
";
    let expected = "\
```python
%load_ext d2lbook.tab
x = 1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn python_code_fence_with_cell_magic_is_preserved_by_ruff_notebook_cell() {
    let dir = tempdir().unwrap();
    let path = fake_ruff_path_env(dir.path());

    let input = "\
```python
%%tab pytorch
x= 1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn ordinary_python_code_fence_still_formats_through_ruff() {
    let dir = tempdir().unwrap();
    let path = fake_ruff_path_env(dir.path());

    let input = "\
```python
x= 1
```
";
    let expected = "\
```python
x = 1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn failing_optional_embedded_formatter_preserves_code_fence_silently() {
    let dir = tempdir().unwrap();
    let path = failing_ruff_path_env(dir.path());

    let input = "\
```python
    x=1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn failing_optional_embedded_formatter_reports_diagnostic_when_requested() {
    let dir = tempdir().unwrap();
    let path = failing_ruff_path_env(dir.path());

    let input = "\
```python
    x=1
```
";
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("input.md:2:1: note: embedded formatter `ruff` failed at formatter input 7:9; left chunk unchanged"),
        "{stderr}"
    );
    assert!(!stderr.contains("x=1"), "{stderr}");
}

#[test]
fn missing_optional_formatter_reports_diagnostic_notes_when_requested() {
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .env("PATH", "")
        .write_stdin("```python\nprint('x')\n```\n")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "```python\nprint('x')\n```\n"
    );
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.md:2:1: note: missing optional embedded formatter `ruff`; preserved source"
    ));
}

#[test]
fn embedded_formatter_stderr_on_success_is_an_error() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"printf formatted; printf warning >&2\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("embedded formatter wrote to stderr: warning"));
}

#[cfg(unix)]
#[test]
fn embedded_formatter_nonzero_with_empty_stderr_reports_context() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"/bin/sh\", \"-c\", \"exit 7\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("embedded formatter failed"), "{stderr}");
    assert!(stderr.contains("/bin/sh"), "{stderr}");
    assert!(stderr.contains("exit status 7"), "{stderr}");
    assert!(stderr.contains("input.md.embedded.2.txt"), "{stderr}");
}

#[cfg(unix)]
fn make_symlink(path: &Path, link: &Path) {
    std::os::unix::fs::symlink(path, link).unwrap();
}

#[cfg(windows)]
fn make_symlink(path: &Path, link: &Path) {
    std::os::windows::fs::symlink_file(path, link).unwrap();
}
