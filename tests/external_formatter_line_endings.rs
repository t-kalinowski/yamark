use assert_cmd::Command;

const CONFIG: &str = "tests/fixtures/external_formatter_line_endings.toml";

fn assert_format(source: &str, expected: &str, path: &str) {
    for input in [source, expected] {
        Command::cargo_bin("yamark")
            .unwrap()
            .args(["format", "--stdin-file-path", path, "--config", CONFIG])
            .write_stdin(input)
            .assert()
            .success()
            .stderr("")
            .stdout(expected.to_owned());
    }
}

#[test]
fn external_formatter_keeps_fences_and_yaml_scalars_line_delimited() {
    for newline in ["\r\n", "\n", "\r"] {
        for (source, path) in [
            ("```python\nx=1\nx\n```\n\nAfter.\n", "input.md"),
            (
                "```{python}\n#| label: example\nx=1\nx\n```\n\nAfter.\n",
                "input.qmd",
            ),
            (
                "# fmt: python\ncode: |\n  x=1\n  x\nother: 1\n",
                "input.yaml",
            ),
        ] {
            let source = source.replace('\n', newline);
            let expected = source.replace("x=1", "x = 1");
            assert_format(&source, &expected, path);
        }
    }
}

#[test]
fn external_formatter_line_ending_fixtures_keep_exact_bytes() {
    for (source, path) in [
        (
            include_str!("fixtures/external_formatter_crlf.md"),
            "input.md",
        ),
        (
            include_str!("fixtures/external_formatter_crlf.yaml"),
            "input.yaml",
        ),
        (
            include_str!("fixtures/external_formatter_cr.md"),
            "input.md",
        ),
    ] {
        assert_format(source, source, path);
    }
}
