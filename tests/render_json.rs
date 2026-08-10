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
