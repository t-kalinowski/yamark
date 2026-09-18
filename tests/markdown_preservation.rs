use assert_cmd::Command;

fn format(source: &str) -> String {
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args([
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "sentence",
        ])
        .write_stdin(source)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn preserved_transcripts_are_idempotent() {
    for case in [
        include_str!("cases/markdown_preserved_nodes.case"),
        include_str!("cases/markdown_preserved_nested_fragments.case"),
        include_str!("cases/markdown_preserved_off.case"),
        include_str!("cases/markdown_preserved_cleanup_controls.case"),
    ] {
        let expected = case
            .split("-- stdout\n")
            .nth(1)
            .unwrap()
            .split("-- stderr\n")
            .next()
            .unwrap();
        assert_eq!(format(expected), expected, "second pass");
    }
}

#[test]
fn explicit_preservation_keeps_line_endings_and_eof_bytes() {
    for newline in ["\n", "\r\n", "\r"] {
        for directive in ["skip", "off", "skip file"] {
            let preserved = format!("<!-- fmt: {directive} -->{newline}Keep   é prose.  \t");
            for suffix in ["", newline] {
                let source = format!("{preserved}{suffix}");
                assert_eq!(format(&source), source, "{directive}, {newline:?}");
                assert_eq!(format(&format(&source)), source, "second pass");
            }
        }

        let source = "::: {.outer}\n```markdown\nBefore   child. \n\n<!-- fmt: skip -->\nKeep   é prose.  \t\n\nAfter   child. \n```\n:::\n"
            .replace('\n', newline);
        let expected = source
            .replace("Before   child. ", "Before child.")
            .replace("After   child. ", "After child.");
        assert_eq!(format(&source), expected, "nested {newline:?}");
        assert_eq!(format(&expected), expected, "second pass");
    }

    let source = "<!-- fmt: skip file -->\r\nKeep   CRLF.  \t\nKeep   LF.  \rKeep   CR.  ";
    assert_eq!(format(source), source, "mixed line endings");
    assert_eq!(format(&format(source)), source, "second pass");

    // Ordinary EOF cleanup remains active after the last protected range.
    assert_eq!(
        format("<!-- fmt: skip -->\nKeep   prose.  \t\n\nFormat   outside.  \t"),
        "<!-- fmt: skip -->\nKeep   prose.  \t\n\nFormat outside.\n"
    );
}
