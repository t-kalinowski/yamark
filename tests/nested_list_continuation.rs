use assert_cmd::Command;

fn format(source: &str, wrap: &str) -> String {
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--stdin-file-path", "input.md", "--wrap", wrap])
        .write_stdin(source)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn outer_list_continuations_keep_line_endings_and_are_idempotent() {
    let case = include_str!("cases/markdown_nested_list_continuation_sentence.case");
    let expected = case
        .split("-- stdout\n")
        .nth(1)
        .unwrap()
        .split("-- stderr\n")
        .next()
        .unwrap();
    for newline in ["\n", "\r\n", "\r"] {
        let expected = expected.replace('\n', newline);
        for wrap in ["sentence", "72", "sentence:32", "paragraph", "none"] {
            assert_eq!(format(&expected, wrap), expected, "{wrap}, {newline:?}");
            assert_eq!(
                format(&format(&expected, wrap), wrap),
                expected,
                "second pass: {wrap}, {newline:?}"
            );
        }
    }
}

#[test]
fn nested_list_boundary_transcript_is_idempotent() {
    let case = include_str!("cases/markdown_nested_list_continuation_boundaries.case");
    let expected = case
        .split("-- stdout\n")
        .nth(1)
        .unwrap()
        .split("-- stderr\n")
        .next()
        .unwrap();
    assert_eq!(format(expected, "sentence"), expected);
}
