use super::support::run_stdin;

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
fn yaml_bom_prefixed_opaque_root_rejects_same_line_directives() {
    let input = "\u{feff}- value # fmt: skip\n- next\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("formatting this BOM-prefixed YAML root is unsupported"),
        "{stderr}"
    );
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
fn yaml_tab_indentation_without_directives_is_preserved() {
    let input = "\tname: value\n";
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
            "fmt: wrap must be none, paragraph, sentence, a positive integer, or sentence:<positive integer>",
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
