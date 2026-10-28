use super::support::run_stdin;

#[test]
fn yaml_utf8_bom_is_preserved() {
    let input = "\u{feff}name:    value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");

    let input = "\u{feff}value\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
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
fn yaml_escaped_newline_quoted_scalar_with_leading_spaces_uses_literal_block() {
    let input = r#"result:
  content:
    - type: text
      text: "\n\x20*** caught segfault ***\n\
        address 0x0, cause 'invalid permissions'\n\n\
        Traceback:\n\x201: tools::pskill(Sys.getpid(), signal = 11L)\n\n\
        Possible actions:\n\
        1: abort (with core dump, if enabled)\n\
        2: normal R exit\n\
        3: exit R without saving workspace\n\
        4: exit R saving workspace\n\
        Selection: R is aborting now ...\n\
        worker sideband read failed: worker sideband closed"
  isError: true
"#;
    let expected = r#"result:
  content:
    - type: text
      text: |2-

         *** caught segfault ***
        address 0x0, cause 'invalid permissions'

        Traceback:
         1: tools::pskill(Sys.getpid(), signal = 11L)

        Possible actions:
        1: abort (with core dump, if enabled)
        2: normal R exit
        3: exit R without saving workspace
        4: exit R saving workspace
        Selection: R is aborting now ...
        worker sideband read failed: worker sideband closed
  isError: true
"#;
    let (status, stdout, stderr) = run_stdin(
        &["format", "--verify", "--stdin-file-path", "input.yaml"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_literal_block_conversion_uses_indent_indicator_for_leading_spaces() {
    let input = r#"mixed: "\u0020\u0020first\nsecond"
both: "\u0020\u0020first\n\u0020\u0020second"
escape_heavy: "\u0020\u0020{\"python\":\"print(\u0027hello\u0027)\"}"
nested:
  - value: "\u0020\u0020first\nsecond"
"#;
    let expected = r#"mixed: |2-
    first
  second
both: |2-
    first
    second
escape_heavy: |2-
    {"python":"print('hello')"}
nested:
  - value: |2-
        first
      second
"#;
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
fn yaml_newline_only_quoted_scalar_remains_quoted() {
    let input = "value: \"\\n\"\nafter: after\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_root_block_scalars_preserve_exact_content() {
    for input in [
        "|-\n  #\n  value\n",
        "|-\n  value \n",
        "!generated |-\n  value\t\n",
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_block_scalar_removes_indentation_from_blank_lines() {
    let input = concat!("text: |-\n", "  \n", "  [running]\n");
    let expected = concat!("text: |-\n", "\n", "  [running]\n");
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_nondefault_indent_reindents_nested_block_scalar_body() {
    let input = "\
outer:
  value: |-
    line
";
    let expected = "\
outer:
    value: |-
        line
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn yaml_nondefault_indent_reindents_multiline_plain_scalar() {
    let input_lf = "\
outer:
  value: first line
    second line
    third line
  after: x
";
    let expected_lf = "\
outer:
    value: first line
      second line
      third line
    after: x
";
    for newline in ["\n", "\r\n", "\r"] {
        let input = input_lf.replace('\n', newline);
        let expected = expected_lf.replace('\n', newline);
        let (status, stdout, stderr) = run_stdin(
            &[
                "format",
                "--stdin-file-path",
                "input.yaml",
                "--indent-width",
                "4",
            ],
            &input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_nondefault_indent_reindents_multiline_scalar_variants() {
    let cases = [
        (
            "outer:\n  tagged: !tag first line\n    second line\n  anchored: &value first line\n    second line\n  after: x\n",
            "outer:\n    tagged: !tag first line\n      second line\n    anchored: &value first line\n      second line\n    after: x\n",
        ),
        (
            "outer:\n  value: first line\n    \n    third line\n  after: x\n",
            "outer:\n    value: first line\n      \n      third line\n    after: x\n",
        ),
        (
            "outer:\n  ? key\n  : first line\n    second line\n  after: x\n",
            "outer:\n    ? key\n    : first line\n      second line\n    after: x\n",
        ),
        (
            "outer:\n  - value: first line\n      second line\n      third line\n    after: x\n",
            "outer:\n    - value: first line\n        second line\n        third line\n      after: x\n",
        ),
    ];
    for (input, expected) in cases {
        let (status, stdout, stderr) = run_stdin(
            &[
                "format",
                "--stdin-file-path",
                "input.yaml",
                "--indent-width",
                "4",
            ],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_block_plain_scalar_moved_inline_preserves_continuation_indent() {
    for (input, expected) in [
        (
            "Warning:\n  This is an error message\n  for the log file\n",
            "Warning: This is an error message\n  for the log file\n",
        ),
        (
            "comments:\n    Late afternoon is best.\n    Backup contact is Nancy\n    Billsmer @ 338-4338.\n",
            "comments: Late afternoon is best.\n  Backup contact is Nancy\n  Billsmer @ 338-4338.\n",
        ),
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_nondefault_indent_preserves_space_only_block_scalar_content() {
    let input = concat!("- |-\n", "  first\n", "   \n", "  last\n", "- after\n");
    let expected = concat!(
        "- |-\n",
        "    first\n",
        "     \n",
        "    last\n",
        "- after\n",
    );
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn yaml_bom_prefixed_opaque_root_stops_at_next_document() {
    for (input, expected) in [
        (
            "\u{feff}- one\n---\nname:    value\n",
            "\u{feff}- one\n---\nname: value\n",
        ),
        (
            "\u{feff}|-\n  first\n---\nname:    value\n",
            "\u{feff}|-\n  first\n---\nname: value\n",
        ),
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_later_document_line_leading_fe_ff_is_preserved() {
    let input = "first:    one\n---\n\u{feff}second:    two\nafter: y\n";
    let expected = "first: one\n---\n\u{feff}second:    two\nafter: y\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_nondefault_indent_reindents_cr_only_block_scalar_body() {
    let input = "outer:\r  value: |-\r  \r    first\r    second\r  after: x\r";
    let expected = "outer:\r    value: |-\r\r        first\r        second\r    after: x\r";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
fn yaml_nondefault_indent_preserves_explicit_block_indent_and_space_content() {
    let input = concat!(
        "outer:\n",
        "  - value: |2-\n",
        "      first\n",
        "       \n",
        "      last\n",
        "    after: x\n",
    );
    let expected = concat!(
        "outer:\n",
        "    - value: |2-\n",
        "        first\n",
        "         \n",
        "        last\n",
        "      after: x\n",
    );
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
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
empty: !!null ~
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
fn yaml_plain_core_booleans_normalize_and_tilde_null_is_preserved() {
    let input = "\
flag: TRUE
disabled: False
empty: ~
items: [ TRUE , Null ]
";
    let expected = "\
flag: true
disabled: false
empty: ~
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
empty: !!null ~
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
fn yaml_root_document_marker_strings_remain_quoted() {
    for input in ["\"---\"\n", "\"--- x\"\n", "\"...\"\n", "\"... x\"\n"] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }

    let input = "--- \"---\"\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "--- ---\n");
    assert_eq!(stderr, "");
}

#[test]
fn yaml_strings_ending_in_colon_remain_quoted() {
    for input in [
        "- \"foo:\"\n",
        "value: \"foo:\"\n",
        "items: [\"foo:\"]\n",
        "metadata: {value: \"foo:\"}\n",
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_non_ascii_whitespace_is_scalar_content() {
    let nbsp = '\u{00a0}';
    let input = format!("trailing: foo{nbsp}\nleading: {nbsp}foo\ncore_like: {nbsp}null{nbsp}\n");
    let expected =
        format!("trailing: 'foo{nbsp}'\nleading: '{nbsp}foo'\ncore_like: '{nbsp}null{nbsp}'\n");
    let (status, stdout, stderr) =
        run_stdin(&["format", "--stdin-file-path", "input.yaml"], &input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_escaped_non_printable_strings_remain_quoted() {
    for input in [
        "value: \"\\uFEFF\"\n",
        "value: \"\\uFFFE\"\n",
        "value: \"\\uFFFF\"\n",
    ] {
        let (status, stdout, stderr) =
            run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
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
fn yaml_indicator_prefixed_strings_stay_plain_when_safe() {
    let cases = [
        (
            "extends: :workspace\nname: app\n",
            "{extends: :workspace, name: app}\n",
        ),
        (
            "[:workspace, ?query, -option]\n",
            "[:workspace, ?query, -option]\n",
        ),
        (
            r#"[":workspace", '?query', "-option"]
"#,
            "[:workspace, ?query, -option]\n",
        ),
        (
            r#"["\x3aworkspace", "\x3fquery", "\x2doption"]
"#,
            "[:workspace, ?query, -option]\n",
        ),
        (
            "{\":workspace\": value, \"?query\": result}\n",
            "{:workspace: value, ?query: result}\n",
        ),
        (
            "[\"\\x3f\", \"\\x2d\", \"\\x3a value\", \"\\x3f value\", \"\\x2d value\"]\n",
            "[\"?\", \"-\", \": value\", \"? value\", \"- value\"]\n",
        ),
        (
            r#"[":", "?", "-", ": value", "? value", "- value", ":[value]", "?[value]", "-[value]"]
"#,
            "[\":\", \"?\", \"-\", \": value\", \"? value\", \"- value\", \":[value]\", \"?[value]\", \"-[value]\"]\n",
        ),
    ];
    let args = [
        "format",
        "--stdin-file-path",
        "input.yaml",
        "--compact",
        "--line-width",
        "120",
        "--verify",
    ];
    for (input, expected) in cases {
        for source in [input, expected] {
            let (status, stdout, stderr) = run_stdin(&args, source);
            assert_eq!(status, 0, "{source}: {stderr}");
            assert_eq!(stdout, expected, "{source}");
            assert_eq!(stderr, "");
        }
    }
}

#[test]
fn yaml_indicator_prefixed_escaped_strings_use_unquoted_width() {
    let input = "[\"\\x3aworkspace\"]\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "12",
            "--verify",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "[:workspace]\n");
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
fn yaml_literal_block_conversion_uses_configured_indent_indicator() {
    let input = "outer:\n  text: \"\\x20first\\nsecond\"\n";
    let expected = "outer:\n    text: |4-\n         first\n        second\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--indent-width",
            "4",
            "--verify",
            "--stdin-file-path",
            "input.yaml",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}
