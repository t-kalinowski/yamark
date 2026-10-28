use super::support::run_stdin;

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
fn verify_is_opt_in_for_new_yaml_front_matter() {
    let input = "***\nkey: value\n\n---\nbody\n";
    let expected = "---\nkey: value\n\n---\nbody\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");

    let (status, stdout, stderr) = run_stdin(
        &["format", "--verify", "--stdin-file-path", "input.md"],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("formatter changed the YAML value"),
        "{stderr}"
    );
}

#[test]
fn markdown_front_matter_uses_yaml_scalar_safety() {
    let cases = [
        (
            "---\n\"---\"\n---\n\nBody.\n",
            "---\n\"---\"\n---\n\nBody.\n",
        ),
        (
            "---\nmixed: \"\\u0020\\u0020first\\nsecond\"\ncolon: \"foo:\"\n---\n\nBody.\n",
            "---\nmixed: |2-\n    first\n  second\ncolon: \"foo:\"\n---\n\nBody.\n",
        ),
    ];
    for (input, expected) in cases {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.md"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn markdown_front_matter_preserves_block_scalar_trailing_spaces() {
    let input = concat!(
        "---\n",
        "value: |\n",
        "  first \n",
        "   second \n",
        "  third\n",
        "---\n",
        "\n",
        "Body.\n",
    );
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.md"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}
