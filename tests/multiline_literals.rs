use assert_cmd::Command;

fn assert_format(source: &str, expected: &str, wrap: &str, canonical: bool) {
    for input in [source, expected] {
        let mut command = Command::cargo_bin("yamark").unwrap();
        command.args(["format", "--stdin-file-path", "input.md", "--wrap", wrap]);
        if canonical {
            command.arg("--canonical");
        }
        command
            .write_stdin(input)
            .assert()
            .success()
            .stderr("")
            .stdout(expected.to_owned());
    }
}

#[test]
fn multiline_literals_preserve_bytes_and_format_surrounding_prose() {
    for (source, expected, wrap, canonical) in [
        (
            "Before `first  \n  second [x]( url )` after _outside_. Next sentence.\n",
            "Before `first  \n  second [x]( url )` after *outside*.\nNext sentence.\n",
            "sentence:80",
            true,
        ),
        (
            "Before\n`first\t\n  second [x]( url )` after [real](  target  ).\n",
            "Before `first\t\n  second [x]( url )` after [real](target).\n",
            "paragraph",
            false,
        ),
        (
            "Before  $first  \n  second \\$ [x]( url )$ after _outside_ [real](  target  ).\n",
            "Before  $first  \n  second \\$ [x]( url )$ after *outside* [real](target).\n",
            "none",
            true,
        ),
        (
            "- Before `first  \n    second [x]( url )` after _outside_. Next sentence.\n",
            "- Before `first  \n    second [x]( url )` after *outside*.\n  Next sentence.\n",
            "sentence:80",
            true,
        ),
        (
            "> Before $first  \n>   second \\$ [x]( url )$ after _outside_. Next sentence.\n",
            "> Before $first  \n>   second \\$ [x]( url )$ after *outside*.\n> Next sentence.\n",
            "sentence:80",
            true,
        ),
        (
            "Before `first  \n  second [x]( url )` after [real](  target  ).\n",
            "Before\n`first  \n  second [x]( url )`\nafter [real](target).\n",
            "24",
            false,
        ),
    ] {
        assert_format(source, expected, wrap, canonical);
    }
}

#[test]
fn literal_line_endings_survive_container_prefixes_and_recursive_output() {
    let source = "Before   `first  \r\n  second\t\rthird\\\nlast [x]( url )` after _outside_.\r\n";
    let expected = source
        .replace("Before   ", "Before ")
        .replace("_outside_", "*outside*");
    assert_format(source, &expected, "paragraph", true);
    for newline in ["\n", "\r\n", "\r"] {
        let source = "::: note\n```markdown\nBefore `first  \t\n  second\\\nthird [x]( url )` after _outside_.\n \t\nNext   paragraph.\t\n```\n:::\n".replace('\n', newline);
        let expected = source
            .replace("_outside_", "*outside*")
            .replace(
                &format!("{newline} \t{newline}"),
                &format!("{newline}{newline}"),
            )
            .replace("Next   paragraph.\t", "Next paragraph.");
        assert_format(&source, &expected, "paragraph", true);

        let source = "> > Before `first\t\n> >   second [x]( url )` after _outside_.\n"
            .replace('\n', newline);
        let expected = source.replace("_outside_", "*outside*");
        assert_format(&source, &expected, "none", true);
    }
    for source in [
        "> ```markdown\n> Before `first  \n>   second [x]( url )` after _outside_.\n> ```\n",
        "- Example\n\n  ```markdown\n  Before `first  \n    second [x]( url )` after _outside_.\n  ```\n",
    ] {
        assert_format(
            source,
            &source.replace("_outside_", "*outside*"),
            "paragraph",
            true,
        );
    }
}

#[test]
fn definition_and_footnote_content_keeps_authored_lines() {
    for (source, expected) in [
        (
            "-  Before `first  \n    second [x]( url )` after _outside_.\n",
            "- Before `first  \n   second [x]( url )` after *outside*.\n",
        ),
        (
            "Term\n:   Before `first  \n      second [x]( url )` after _outside_.\n",
            "Term\n: Before `first  \n    second [x]( url )` after *outside*.\n",
        ),
        (
            "Term\n   : Before `first  \n    second [x]( url )` after _outside_. Next sentence.\n",
            "Term\n   : Before `first  \n     second [x]( url )` after *outside*.\n     Next sentence.\n",
        ),
        (
            "Term\n   : Before `first\t\n    \tsecond [x]( url )` after _outside_.\n",
            "Term\n   : Before `first\t\n     \tsecond [x]( url )` after *outside*.\n",
        ),
        (
            "[^note]: Before `first  \n    second [x]( url )` after _outside_.\n",
            "[^note]: Before `first  \n    second [x]( url )` after *outside*.\n",
        ),
        (
            "[^note]: First paragraph.\n\n    Before `first  \n      second [x]( url )` after _outside_.\n",
            "[^note]:\n    First paragraph.\n\n    Before `first  \n      second [x]( url )` after *outside*.\n",
        ),
    ] {
        assert_format(source, expected, "sentence:80", true);
    }
}

#[test]
fn late_options_use_the_original_literal_boundaries() {
    let source = "Before  `first  \n  second [x]( url )` after _outside_ [real](  target  ). Next sentence.\n\n<!-- fmt: wrap=sentence canonical=true scope=file -->\n";
    let expected = "Before `first  \n  second [x]( url )` after *outside* [real](target).\nNext sentence.\n\n<!-- fmt: wrap=sentence canonical=true scope=file -->\n";
    assert_format(source, expected, "none", false);
}

#[test]
fn unwrapped_nested_quotes_keep_line_scopes_and_depth_changes() {
    for (source, expected) in [
        (
            "> > Before <tag> value.\n> > Next [r](  t  ) _outside_.\n",
            "> > Before <tag> value.\n> > Next [r](t) *outside*.\n",
        ),
        (
            "> > Before [r](  t  ) _outside_.\n> > > Next [r](  t  ) _outside_.\n",
            "> > Before [r](t) *outside*.\n> > > Next [r](t) *outside*.\n",
        ),
    ] {
        assert_format(source, expected, "none", true);
    }
}

#[test]
fn ordinary_gaps_hard_breaks_and_fallback_keep_their_policy() {
    for (source, expected) in [
        (
            "Before   `first  \n  second` after.  \nNext   line.\t\n",
            "Before `first  \n  second` after. \\\nNext line.\n",
        ),
        (
            "100. Before $first\\\n    second [x]( url )$ after _outside_. Next sentence.\n",
            "100. Before $first\\\n    second [x]( url )$ after _outside_. Next sentence.\n",
        ),
        (
            "Before <kbd>first  \nsecond </kbd> after _outside_.\n",
            "Before <kbd>first \\\nsecond </kbd> after _outside_.\n",
        ),
    ] {
        assert_format(source, expected, "sentence:80", false);
    }
}

#[test]
fn hard_breaks_keep_existing_markup_and_link_normalization_boundaries() {
    assert_format(
        "Before <kbd>first\nsecond</kbd> after.  \nNext [real](  target  ).\n",
        "Before <kbd>first second</kbd> after. \\\nNext [real](target).\n",
        "sentence:80",
        true,
    );
    for (source, expected) in [
        (
            "Before <kbd>first   word  \nsecond  word</kbd> after _outside_.\n",
            "Before <kbd>first word \\\nsecond word</kbd> after\n*outside*.\n",
        ),
        (
            "Before <kbd>word</kbd> after.  \nNext [real](  target  ).\n",
            "Before <kbd>word</kbd> after. \\\nNext [real](target).\n",
        ),
    ] {
        assert_format(source, expected, "30", true);
    }

    // Main's automatic fallback trims trailing spaces in this opaque HTML
    // example. Its second pass consequently has different eligibility; this
    // compatibility control does not give HTML new literal semantics.
    Command::cargo_bin("yamark")
        .unwrap()
        .args([
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            "30",
            "--canonical",
        ])
        .write_stdin("Before <kbd>`first  \nsecond`</kbd> after _outside_.\n")
        .assert()
        .success()
        .stderr("")
        .stdout("Before <kbd>`first\nsecond`</kbd> after _outside_.\n");
}

#[test]
fn marker_only_containers_keep_structural_lines() {
    let source = "-\n\n>\n\n> -\n\n- >\n\n```markdown\n-\n```\n\n- `  `\n\n> $  $\n";
    for wrap in ["sentence", "24", "none", "paragraph"] {
        assert_format(source, source, wrap, false);
    }
    for newline in ["\r\n", "\r"] {
        let source = source.replace('\n', newline);
        assert_format(&source, &source, "sentence", false);
    }
}

#[test]
fn terminal_literals_keep_document_final_newline_policy() {
    for (source, wrap) in [
        ("Before `code`", "sentence"),
        ("Before $math$", "none"),
        ("Before prose", "24"),
        (
            "<!-- fmt: skip -->\nKeep   this\n\nBefore `code`",
            "sentence",
        ),
        ("{{< call >}}\n\nBefore `code`", "sentence"),
        ("```markdown\nBefore `code`\n```", "sentence"),
        ("> ```markdown\n> Before `code`\n> ```", "sentence"),
    ] {
        assert_format(source, &format!("{source}\n"), wrap, false);
    }
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!("Before `first  {newline}  second\t[x]( url )`");
        assert_format(&source, &format!("{source}{newline}"), "paragraph", false);
    }
    assert_format("Before `code`  \t", "Before `code`\n", "none", false);
    for source in [
        "<!-- fmt: off -->\nBefore `code`",
        "<!-- fmt: skip -->\nBefore $math$",
        "Before `code`\n\n{{< call >}}",
    ] {
        assert_format(source, source, "sentence", false);
    }
}

#[test]
fn definition_tabs_match_parser_eligibility_without_trimming_literals() {
    let case = include_str!("cases/markdown_definition_tab_continuations.case");
    let source = case
        .split("-- stdin\n")
        .nth(1)
        .unwrap()
        .split("-- stdout\n")
        .next()
        .unwrap();
    let expected = case
        .split("-- stdout\n")
        .nth(1)
        .unwrap()
        .split("-- stderr\n")
        .next()
        .unwrap();
    assert_format(source, expected, "sentence", true);
    // Main's unsupported Unicode gap remains a fallback. A byte threshold
    // must not become an invalid UTF-8 slice while preparing this content.
    let source = "Term\n   : Before   prose.\n   \u{2003}After   _outside_ [real](  target  ).\n";
    assert_format(source, source, "sentence", true);
}

#[test]
fn list_support_uses_literal_boundaries_with_escapes_and_indented_openers() {
    for (case, wrap) in [
        (
            include_str!("cases/markdown_multiline_literal_list_escapes.case"),
            "sentence",
        ),
        (
            include_str!("cases/markdown_multiline_literal_list_escapes.case"),
            "none",
        ),
        (
            include_str!("cases/markdown_multiline_literal_list_openers.case"),
            "24",
        ),
    ] {
        let source = case
            .split("-- stdin\n")
            .nth(1)
            .unwrap()
            .split("-- stdout\n")
            .next()
            .unwrap();
        let expected = case
            .split("-- stdout\n")
            .nth(1)
            .unwrap()
            .split("-- stderr\n")
            .next()
            .unwrap();
        assert_format(source, expected, wrap, true);
    }
}
