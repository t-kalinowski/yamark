use assert_cmd::Command;

fn format(source: &str, wrap: &str, path: &str) -> String {
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--stdin-file-path", path, "--wrap", wrap])
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
            assert_eq!(
                format(&expected, wrap, "input.md"),
                expected,
                "{wrap}, {newline:?}"
            );
            assert_eq!(
                format(&format(&expected, wrap, "input.md"), wrap, "input.md"),
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
    assert_eq!(format(expected, "sentence", "input.md"), expected);
}

#[test]
fn ambiguous_list_continuations_are_preserved_across_wrap_modes() {
    for case in [
        include_str!("cases/markdown_list_shifted_sibling_lazy_continuation.case"),
        include_str!("cases/markdown_list_empty_item_pandoc_continuation.case"),
    ] {
        let expected = case
            .split("-- stdout\n")
            .nth(1)
            .unwrap()
            .split("-- stderr\n")
            .next()
            .unwrap();
        for path in ["input.md", "input.qmd"] {
            for wrap in ["sentence", "72", "sentence:32", "paragraph", "none"] {
                let output = format(expected, wrap, path);
                assert_eq!(output, expected, "{path}, {wrap}");
                assert_eq!(format(&output, wrap, path), expected, "second pass");
            }
        }
    }
}
