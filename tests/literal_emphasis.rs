use assert_cmd::Command;

#[test]
fn literal_emphasis_cases_are_exact_and_idempotent() {
    for case in [
        include_str!("cases/markdown_heading_emphasis_around_literals.case"),
        include_str!("cases/markdown_literal_emphasis_canonical.case"),
        include_str!("cases/markdown_literal_emphasis_escapes.case"),
        include_str!("cases/markdown_literal_emphasis_boundaries.case"),
        include_str!("cases/markdown_literal_emphasis_wrap.case"),
    ] {
        let (args, rest) = case
            .strip_prefix("-- args\n")
            .unwrap()
            .split_once("-- stdin\n")
            .unwrap();
        let (source, rest) = rest.split_once("-- stdout\n").unwrap();
        let (expected, _) = rest.split_once("-- stderr\n").unwrap();
        for input in [source, expected] {
            Command::cargo_bin("yamark")
                .unwrap()
                .args(args.split_whitespace())
                .write_stdin(input)
                .assert()
                .success()
                .stderr("")
                .stdout(expected);
        }
    }
}

#[test]
fn noncanonical_literal_contents_and_surrounding_emphasis_stay_exact() {
    let source = "# _Before `_a_ *b* [x]( url )` after_\n\n\
                  Lead `_a_ *b*` and $a \\$ __b__ **c**$ after _outside_ [real](  target  ).\n";
    let expected = "# _Before `_a_ *b* [x]( url )` after_\n\n\
                    Lead `_a_ *b*` and $a \\$ __b__ **c**$ after _outside_ [real](target).\n";
    for input in [source, expected] {
        Command::cargo_bin("yamark")
            .unwrap()
            .args(["format", "--stdin-file-path", "input.md", "--wrap", "none"])
            .write_stdin(input)
            .assert()
            .success()
            .stderr("")
            .stdout(expected);
    }
}
