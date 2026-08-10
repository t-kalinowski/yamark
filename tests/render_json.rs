use std::path::Path;

use yamark::core::FormatOptions;
use yamark::workspace::{format_source_for_path, render_source_for_path};

#[test]
fn strict_json_renders_ordered_duplicate_members_and_decoded_strings() {
    let input = r#"{"first":1,"emoji":"\uD83D\uDE00","slash":"a\/b","a":-0,"a":1e+02,"big":123456789012345678901234567890}"#;
    let options = FormatOptions {
        line_width: 40,
        ..FormatOptions::default()
    };

    let rendered =
        render_source_for_path(Path::new("input.json"), input.to_owned(), options, None).unwrap();

    assert_eq!(
        rendered.output,
        "first: 1\nemoji: 😀\nslash: a/b\na: -0\na: 1e+02\nbig: 123456789012345678901234567890\n"
    );
    let formatted = format_source_for_path(
        Path::new("output.yaml"),
        rendered.output.clone(),
        options,
        None,
    )
    .unwrap();
    assert!(!formatted.changed);
    assert_eq!(formatted.output, rendered.output);
}

#[test]
fn json_lines_renders_each_physical_record_as_a_yaml_document() {
    let rendered = render_source_for_path(
        Path::new("records.jsonl"),
        "{\"a\":1}\n[2,3]\n4\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "---\n{a: 1}\n---\n[2, 3]\n---\n4\n");
}

#[test]
fn json_lines_rejects_blank_records_at_their_physical_line() {
    let error = render_source_for_path(
        Path::new("records.ndjson"),
        "{\"a\":1}\n\n[2]\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap_err();

    assert_eq!(
        error.diagnostic.path.as_deref(),
        Some(Path::new("records.ndjson"))
    );
    assert_eq!(error.diagnostic.line, 2);
    assert_eq!(error.diagnostic.column, 1);
    assert_eq!(
        error.diagnostic.message,
        "invalid JSON Lines: records cannot be blank"
    );
}

#[test]
fn strict_json_reports_invalid_surrogate_escapes_at_the_source_path() {
    let error = render_source_for_path(
        Path::new("input.json"),
        r#"{"value":"\uD83D"}"#.to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap_err();

    assert_eq!(
        error.diagnostic.path.as_deref(),
        Some(Path::new("input.json"))
    );
    assert_eq!(error.diagnostic.line, 1);
    assert!(error.diagnostic.column > 1);
    assert!(error.diagnostic.message.contains("unpaired high surrogate"));
}

#[test]
fn native_rendering_matches_the_existing_formatter() {
    let input = "#   Title ##\n\nText that stays short.\n".to_owned();
    let options = FormatOptions::default();
    let formatted =
        format_source_for_path(Path::new("input.md"), input.clone(), options, None).unwrap();
    let rendered = render_source_for_path(Path::new("input.md"), input, options, None).unwrap();

    assert_eq!(rendered.output, formatted.output);
    assert_eq!(rendered.diagnostics, formatted.diagnostics);
}

#[test]
fn jsonc_preserves_comments_without_reinterpreting_string_contents() {
    let input = r#"{
  // before a
  "a": 1, // after a
  "url": "https://example.test/a/*b*/", // after url: value
  /* before nested */
  "nested": {"b": 2 /* after b */},
}
"#;
    let options = FormatOptions {
        line_width: 40,
        ..FormatOptions::default()
    };

    let rendered =
        render_source_for_path(Path::new("input.jsonc"), input.to_owned(), options, None).unwrap();

    for comment in [
        "# before a",
        "# after a",
        "# after url: value",
        "# before nested",
        "# after b",
    ] {
        assert!(rendered.output.contains(comment), "missing {comment:?}");
    }
    assert!(rendered.output.contains("url: https://example.test/a/*b*/"));
    assert!(rendered.output.contains("a: 1"));
    assert!(rendered.output.contains("nested:\n  b: 2"));

    let formatted =
        format_source_for_path(Path::new("output.yaml"), rendered.output, options, None).unwrap();
    assert!(!formatted.changed);
}

#[test]
fn jsonc_handles_each_previously_ambiguous_comment_position() {
    for (input, expected_comment) in [
        ("{\"a\":1 // note\n}\n", "# note"),
        ("{\"a\":1, // trailing\n}\n", "# trailing"),
        ("{\"a\":1 // note: hi\n}\n", "# note: hi"),
        ("{\"a\":1 /* block note */}\n", "# block note"),
        ("{\"a\" /* key note */:1}\n", "# key note"),
        ("{\"a\": /* value note */ 1}\n", "# value note"),
    ] {
        let rendered = render_source_for_path(
            Path::new("input.jsonc"),
            input.to_owned(),
            FormatOptions::default(),
            None,
        )
        .unwrap();
        assert!(
            rendered.output.contains(expected_comment),
            "input {input:?} rendered as {:?}",
            rendered.output
        );
        assert!(rendered.output.contains("a: 1"));
    }
}

#[test]
fn jsonc_accepts_trailing_commas_without_other_json5_extensions() {
    let rendered = render_source_for_path(
        Path::new("input.jsonc"),
        "{\"a\": [1, 2,],}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "{a: [1, 2]}\n");
}

#[test]
fn jsonc_rejects_json5_only_extensions() {
    for input in [
        "{a: 1}",
        "{'a': 1}",
        "{\"a\": 0x2A}",
        "{\"a\": +1}",
        "{\"a\": 1 \"b\": 2}",
    ] {
        let error = render_source_for_path(
            Path::new("input.jsonc"),
            input.to_owned(),
            FormatOptions::default(),
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.diagnostic.path.as_deref(),
            Some(Path::new("input.jsonc"))
        );
        assert!(error.diagnostic.message.starts_with("invalid JSONC:"));
    }
}

#[test]
fn jsonc_preserves_document_array_and_empty_container_comments() {
    let input = r#"// document
[
  /* first item */ {"x": 1},
  2, // second item
  /* before empty values */ [],
  {} /* empty object */
]
// end
"#;

    let rendered = render_source_for_path(
        Path::new("input.jsonc"),
        input.to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    for comment in [
        "# document",
        "# first item",
        "# second item",
        "# before empty values",
        "# empty object",
        "# end",
    ] {
        assert!(rendered.output.contains(comment), "missing {comment:?}");
    }
    assert!(rendered.output.contains("-\n  x: 1"));
    assert!(rendered.output.contains("- 2"));
    assert!(rendered.output.contains("- []"));
    assert!(rendered.output.contains("- {}"));
}

#[test]
fn jsonc_comments_cannot_become_yamark_format_directives() {
    let rendered = render_source_for_path(
        Path::new("input.jsonc"),
        "{\n  // fmt: skip file\n  \"a\": 1\n}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "# [jsonc] fmt: skip file\na: 1\n");
}
