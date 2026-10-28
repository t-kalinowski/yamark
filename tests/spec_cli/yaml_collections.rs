use super::support::run_stdin;

#[test]
fn yaml_json_lines_inserts_initial_markers_and_preserves_crlf() {
    let input = "{\"id\":1}\r\n{\"id\":2}\r\n";
    let expected = "---\r\n{id: 1}\r\n---\r\n{id: 2}\r\n";
    let args = ["format", "--stdin-file-path", "input.yaml"];

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
fn yaml_escaped_newline_quoted_sequence_scalars_with_colons_emit_literal_blocks() {
    let input = "messages:\n  - \"Lorem ipsum: dolor sit amet\\nConsectetur adipiscing elit\"\n";
    let expected = "\
messages:
  - |-
    Lorem ipsum: dolor sit amet
    Consectetur adipiscing elit
";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--verify", "--stdin-file-path", "input.yaml"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_escaped_newline_quoted_inline_sequence_mapping_uses_literal_block() {
    let input = r#"outer:
  - text: "\n\x20first\n\
      Traceback:\n\x201: frame"
"#;
    let expected = r#"outer:
  - text: |2-

       first
      Traceback:
       1: frame
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
fn yaml_nondefault_indent_reindents_multiline_quoted_sequence_mapping_scalar() {
    let single_quoted_input = "\
outer:
  - value: 'first
      second
      third'
    after: x
";
    let single_quoted_expected = "\
outer:
    - value: 'first
        second
        third'
      after: x
";
    let double_quoted_input = single_quoted_input.replace('\'', "\"");
    let double_quoted_expected = single_quoted_expected.replace('\'', "\"");
    for (input_lf, expected_lf) in [
        (
            single_quoted_input.to_owned(),
            single_quoted_expected.to_owned(),
        ),
        (double_quoted_input, double_quoted_expected),
    ] {
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
}

#[test]
fn yaml_sequence_mapping_block_scalar_stops_before_sibling_pair() {
    let input = "\
- value: |-
    first
    second
  empty: ~
";
    let expected = "\
- value: |-
    first
    second
  empty: ~
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_nested_sequence_folded_scalar_uses_child_indent() {
    let input = "\
- - This is a long prose scalar with enough words to fold into a block scalar safely
";
    let expected = "\
-
  - >-
    This is a long prose scalar
    with enough words to fold into
    a block scalar safely
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--prose-width",
            "30",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_nondefault_indent_keeps_sequence_mapping_keys_aligned() {
    let input = "\
- first: ~
  second: []
";
    let expected = "\
- first: ~
  second: []
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
fn yaml_nondefault_indent_reindents_nested_sequence_block_scalar() {
    let input = "\
- - |-
    first
    second
";
    let expected = "\
-
    - |-
        first
        second
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
fn yaml_bom_prefixed_root_mapping_preserves_block_scalar_body() {
    let input = "\u{feff}value: |-\n  first\n  second\nafter: y\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_bom_prefixed_root_block_scalars_and_sequences_are_preserved() {
    for input in [
        "\u{feff}|-\n  first\n  second\n",
        "\u{feff}- |-\n  first\n- after\n",
        "\u{feff}- - |-\n    first\n  - after\n",
    ] {
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
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_bom_prefixed_flow_and_mapping_roots_are_preserved() {
    let configurations: &[&[&str]] = &[
        &[],
        &["--indent-width", "4"],
        &["--compact"],
        &["--canonical"],
        &["--indent-width", "4", "--compact", "--canonical"],
    ];
    for input in ["\u{feff}[a,b,{x: y}]\n", "\u{feff}a: 1\nb: [2,3]\n"] {
        for configuration in configurations {
            let mut args = vec!["format", "--stdin-file-path", "input.yaml"];
            args.extend_from_slice(configuration);
            let (status, stdout, stderr) = run_stdin(&args, input);
            assert_eq!(status, 0, "{stderr}");
            assert_eq!(stdout, input);
            assert_eq!(stderr, "");
        }
    }
}

#[test]
fn yaml_flow_strings_preserve_boundary_whitespace() {
    let input = "items: [\" foo\", \"foo \"]\n";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
    assert_eq!(stderr, "");
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
fn yaml_nested_compact_sequence_mapping_preserves_structure() {
    let input = "\
- - key: \"value\"
    empty: ~
";
    let expected = "\
-
  - key: value
    empty: ~
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
fn yaml_compact_keeps_single_pair_root_mappings_in_block_style() {
    let cases = [
        ("extends: workspace\n", "extends: workspace\n"),
        ("extends: :workspace\n", "extends: :workspace\n"),
        ("extends: [ one , two ]\n", "extends: [one, two]\n"),
        ("---\nextends: workspace\n", "---\nextends: workspace\n"),
        ("extends: workspace\r\n", "extends: workspace\r\n"),
    ];
    let args = [
        "format",
        "--stdin-file-path",
        "input.yaml",
        "--compact",
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
fn yaml_compact_keeps_single_line_sequence_mappings_in_block_style() {
    let input = "- extends: workspace\n- extends: other\n";
    for indent in ["2", "4"] {
        let (status, stdout, stderr) = run_stdin(
            &[
                "format",
                "--stdin-file-path",
                "input.yaml",
                "--compact",
                "--indent-width",
                indent,
                "--verify",
            ],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_compact_still_collapses_single_pair_mappings_when_it_saves_a_line() {
    let cases = [
        (
            "task:\n  extends: workspace\n",
            "task: {extends: workspace}\n",
        ),
        ("-\n  extends: workspace\n", "- {extends: workspace}\n"),
    ];
    for (input, expected) in cases {
        let (status, stdout, stderr) = run_stdin(
            &[
                "format",
                "--stdin-file-path",
                "input.yaml",
                "--compact",
                "--verify",
            ],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
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
    let (status, stdout, stderr) = run_stdin(
        &["format", "--verify", "--stdin-file-path", "input.yaml"],
        input,
    );
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
fn yaml_multiline_quoted_flow_scalar_expands_without_final_newline() {
    let input = "items: [\"alpha\\nbeta\"]";
    let expected = "items:\n  - |-\n    alpha\n    beta\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--verify",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            "20",
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
