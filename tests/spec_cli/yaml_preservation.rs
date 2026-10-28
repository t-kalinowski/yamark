use super::support::{run_stdin, yamark};
use std::fs;
use tempfile::tempdir;

#[test]
fn unsupported_flow_values_preserve_container_prefixes() {
    for suffix in [
        "sequence:\n  - [it's quoted]\n",
        "sequence:\n  -\n    [it's quoted]\n",
        "nested:\n  [it's quoted]\n",
        "? explicit\n: [it's quoted]\n",
    ] {
        let input = format!("top: [a,b]\n{suffix}");
        let expected = format!("top: [a, b]\n{suffix}");
        let args = ["format", "--stdin-file-path", "input.yaml", "--verify"];
        let (status, output, stderr) = run_stdin(&args, &input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(output, expected);
        let (status, repeated, stderr) = run_stdin(&args, &output);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(repeated, output);
    }
}

#[test]
fn yaml_unsupported_sequence_mapping_with_multiline_quote_is_preserved() {
    let inputs = [
        r#"outer:
  - ? key: "\n\x20first\n\
      Traceback:\n\x201: frame"
    flow: [a,b]
"#,
        r#"outer:
  - ?key: "\n\x20first\n\
      Traceback:\n\x201: frame"
    flow: [a,b]
"#,
        r#"outer:
  - &anchor key : "\n\x20first\n\
      Traceback:\n\x201: frame"
    flow: [a,b]
"#,
        r#"outer:
  - !tag key : "\n\x20first\n\
      Traceback:\n\x201: frame"
    flow: [a,b]
"#,
    ];
    for input in inputs {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--verify", "--stdin-file-path", "input.yaml"],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_unsupported_root_mapping_with_multiline_quote_is_preserved() {
    let inputs = [
        r#"&anchor key: "\n\x20first\n\
  Traceback:\n\x201: frame"
flow: [a,b]
"#,
        r#"!tag key: "\n\x20first\n\
  Traceback:\n\x201: frame"
flow: [a,b]
"#,
        r#"*alias: "\n\x20first\n\
  Traceback:\n\x201: frame"
flow: [a,b]
"#,
        r#"? key: "\n\x20first\n\
  Traceback:\n\x201: frame"
flow: [a,b]
"#,
    ];
    for input in inputs {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--verify", "--stdin-file-path", "input.yaml"],
            input,
        );
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_unsupported_implicit_block_scalar_indent_is_preserved() {
    let input = concat!(
        "text: |-\n",
        "  \n",
        "   *** caught segfault ***\n",
        "  address 0x0\n",
    );
    let configurations: &[&[&str]] = &[&[], &["--indent-width", "4"]];
    for configuration in configurations {
        let mut args = vec![
            "format",
            "--skip-embedded-formatters",
            "--verify",
            "--stdin-file-path",
            "input.yaml",
        ];
        args.extend_from_slice(configuration);
        let (status, stdout, stderr) = run_stdin(&args, input);
        assert_eq!(status, 0, "{stderr}");
        assert_eq!(stdout, input);
        assert_eq!(stderr, "");
    }
}

#[test]
fn yaml_unsupported_leading_block_scalar_blank_indent_is_preserved() {
    let input = concat!("text: |-\n", "    \n", "  content\n");
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
    assert_eq!(stdout, input);
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
fn yaml_diagnostics_skip_output_reparse_by_default() {
    let input = "name:   yamark\n";
    let expected = "name: yamark\n";
    let (status, stdout, stderr) = run_stdin(
        &["format", "--diagnostics", "--stdin-file-path", "input.yaml"],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert!(stderr.contains("source_scans=1 parse_passes=1"), "{stderr}");
}

#[test]
fn yaml_diagnostics_report_verified_output_reparse() {
    let input = "name:   yamark\n";
    let expected = "name: yamark\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--verify",
            "--diagnostics",
            "--stdin-file-path",
            "input.yaml",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert!(stderr.contains("source_scans=1 parse_passes=2"), "{stderr}");
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
fn yaml_verify_formats_markdown_scalar_with_nested_yaml_directive() {
    let input = "\
# fmt: markdown
body: |
  ```yaml
  %YAML 1.2
  ---
  key:    value
  ```
";
    let expected = "\
# fmt: markdown
body: |
  ```yaml
  %YAML 1.2
  ---
  key: value
  ```
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
fn yaml_verify_rejects_non_printable_unicode() {
    for codepoint in [0xfeff, 0xfffe, 0xffff] {
        let dir = tempdir().unwrap();
        let config = dir.path().join("yamark.toml");
        fs::write(
            &config,
            format!(
                r#"
[embedded.invalid]
formatter = {{ command = ["python", "-c", "import sys; sys.stdout.buffer.write(chr({codepoint}).encode())", "{{path}}"], path_suffix = ".txt" }}
"#
            ),
        )
        .unwrap();

        let input = "# fmt: invalid\nbody: |\n  valid\n";
        let (status, stdout, stderr) = run_stdin(
            &[
                "format",
                "--verify",
                "--stdin-file-path",
                "input.yaml",
                "--config",
                config.to_str().unwrap(),
            ],
            input,
        );
        assert_eq!(status, 1);
        assert_eq!(stdout, "");
        assert!(
            stderr.contains("formatter produced invalid YAML"),
            "{stderr}"
        );
    }
}

#[test]
fn yaml_verify_rejects_over_indented_leading_blank_line() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        r#"
[embedded.invalid]
formatter = { command = ["python", "-c", "import sys; sys.stdout.buffer.write(b'   \\na\\n')", "{path}"], path_suffix = ".txt" }
"#,
    )
    .unwrap();

    let input = "# fmt: invalid\nbody: |\n  valid\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--verify",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("formatter produced invalid YAML"),
        "{stderr}"
    );

    let path = dir.path().join("input.yaml");
    fs::write(&path, input).unwrap();
    let output = yamark()
        .args([
            "format",
            "--verify",
            "--config",
            config.to_str().unwrap(),
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}
