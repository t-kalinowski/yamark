use std::path::Path;

use yamark::core::FormatOptions;
use yamark::workspace::{format_source_for_path, project_source_to_yaml};

#[test]
fn strict_json_projects_ordered_duplicate_members_and_decoded_strings() {
    let input = r#"{"first":1,"emoji":"\uD83D\uDE00","slash":"a\/b","a":-0,"a":1e+02,"big":123456789012345678901234567890}"#;
    let options = FormatOptions {
        line_width: 40,
        ..FormatOptions::default()
    };

    let rendered =
        project_source_to_yaml(Path::new("input.json"), input.to_owned(), options, None).unwrap();

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
fn json_lines_projects_each_physical_record_as_a_yaml_document() {
    let rendered = project_source_to_yaml(
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
    let error = project_source_to_yaml(
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
    let error = project_source_to_yaml(
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
fn to_yaml_rejects_native_source_types() {
    let input = "#   Title ##\n\nText that stays short.\n".to_owned();
    let options = FormatOptions::default();
    let error = project_source_to_yaml(Path::new("input.md"), input, options, None).unwrap_err();

    assert_eq!(
        error.diagnostic.message,
        "JSON-to-YAML projection requires a .json, .jsonc, .json5, .jsonl, or .ndjson source path"
    );
    assert_eq!(
        error.diagnostic.path.as_deref(),
        Some(Path::new("input.md"))
    );
}

#[test]
fn jsonc_projection_preserves_comments_without_reinterpreting_string_contents() {
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
        project_source_to_yaml(Path::new("input.jsonc"), input.to_owned(), options, None).unwrap();

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
        let rendered = project_source_to_yaml(
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
    let rendered = project_source_to_yaml(
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
        let error = project_source_to_yaml(
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
fn jsonc_projection_preserves_document_array_and_empty_container_comments() {
    let input = r#"// document
[
  /* first item */ {"x": 1},
  2, // second item
  /* before empty values */ [],
  {} /* empty object */
]
// end
"#;

    let rendered = project_source_to_yaml(
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
    let rendered = project_source_to_yaml(
        Path::new("input.jsonc"),
        "{\n  // fmt: skip file\n  \"a\": 1\n}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "# [jsonc] fmt: skip file\na: 1\n");
}

#[test]
fn json5_normalizes_its_strings_identifiers_and_numbers() {
    let input = "{\n\
  // JSON5 values\n\
  foo:1,\n\
  single: 'don\\'t',\n\
  apostrophe: \"don\\'t\",\n\
  letter: \"\\a\",\n\
  underscore: \"\\_\",\n\
  emoji: \"\\uD83D\\uDE00\",\n\
  hex: 0X2A,\n\
  infinity: Infinity,\n\
  negativeInfinity: -Infinity,\n\
  nan: NaN,\n\
  escaped: {\\u0061: \"\\x41\"},\n\
  leading: .5,\n\
  trailing: 5.,\n\
  positive: +1,\n\
  continued: 'one \\\nline',\n\
}\n";

    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        input.to_owned(),
        FormatOptions {
            line_width: 40,
            ..FormatOptions::default()
        },
        None,
    )
    .unwrap();

    for expected in [
        "# JSON5 values",
        "foo: 1",
        "single: don't",
        "apostrophe: don't",
        "letter: a",
        "underscore: _",
        "emoji: 😀",
        "hex: 0x2A",
        "infinity: .inf",
        "negativeInfinity: -.inf",
        "nan: .nan",
        "a: A",
        "leading: 0.5",
        "trailing: 5.0",
        "positive: 1",
        "continued: one line",
    ] {
        assert!(rendered.output.contains(expected), "missing {expected:?}");
    }
}

#[test]
fn json5_supports_reserved_and_unicode_identifier_names() {
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        "{true: 1, null: 2, Infinity: 3, NaN: 4, café: 5, \\u0061: 6}\n".to_owned(),
        FormatOptions {
            line_width: 30,
            ..FormatOptions::default()
        },
        None,
    )
    .unwrap();

    for expected in [
        "\"true\": 1",
        "\"null\": 2",
        "Infinity: 3",
        "NaN: 4",
        "café: 5",
        "a: 6",
    ] {
        assert!(rendered.output.contains(expected), "missing {expected:?}");
    }
}

#[test]
fn json5_supports_raw_json5_line_separators_inside_strings() {
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        "{paragraph: 'a\u{2028}b', line: 'c\u{2029}d'}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert!(rendered.output.contains("a\\Lb"));
    assert!(rendered.output.contains("c\\Pd"));
}

#[test]
fn json5_normalized_separators_keep_original_diagnostic_positions() {
    let error = project_source_to_yaml(
        Path::new("input.json5"),
        "{first:'a\u{2028}b',second:'c\u{2029}d',broken:}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap_err();

    assert_eq!(error.diagnostic.line, 3);
    assert_eq!(error.diagnostic.column, 11);
}

#[test]
fn json5_rejects_nonstandard_numeric_escapes_and_invalid_identifiers() {
    for input in [
        "{value: '\\1'}",
        "{value: '\\01'}",
        "{a\\u002D: 1}",
        "{value: '\\uD83D'}",
        "{value:\u{0085}1}",
    ] {
        let error = project_source_to_yaml(
            Path::new("input.json5"),
            input.to_owned(),
            FormatOptions::default(),
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.diagnostic.path.as_deref(),
            Some(Path::new("input.json5"))
        );
        assert!(error.diagnostic.message.starts_with("invalid JSON5:"));
    }
}

#[test]
fn json5_preserves_duplicate_keys_and_raw_number_spelling() {
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        "{a: -0, a: 1.20e+3, huge: 0x123456789ABCDEF, negativeHex: -0X2A, negativeHuge: -0x123456789ABCDEF123456789ABCDEF}\n"
            .to_owned(),
        FormatOptions {
            line_width: 200,
            ..FormatOptions::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(
        rendered.output,
        "{a: -0, a: 1.20e+3, huge: 0x123456789ABCDEF, negativeHex: -42, negativeHuge: -94522879700260683142460330790866415}\n"
    );
}

#[test]
fn json5_handles_each_previously_ambiguous_comment_position() {
    for (input, expected_comment) in [
        ("{a:1 // note\n}\n", "# note"),
        ("{a:1, // trailing\n}\n", "# trailing"),
        ("{a:1 // note: hi\n}\n", "# note: hi"),
        ("{a:1 /* block note */}\n", "# block note"),
        ("{a /* key note */:1}\n", "# key note"),
        ("{a: /* value note */ 1}\n", "# value note"),
    ] {
        let rendered = project_source_to_yaml(
            Path::new("input.json5"),
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
fn json5_preserves_comments_between_a_unary_sign_and_its_operand() {
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        "{value: -/* units */1}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "# units\nvalue: -1\n");
}

#[test]
fn json_family_nesting_limit_ignores_strings_and_jsonc_comments() {
    let delimiters = "[{".repeat(300);
    for (extension, input) in [
        ("json", format!("{{\"text\":\"{delimiters}\"}}\n")),
        (
            "jsonc",
            format!("{{/* {delimiters} */\n// {delimiters}\n\"text\":\"{delimiters}\"}}\n"),
        ),
    ] {
        project_source_to_yaml(
            Path::new(&format!("input.{extension}")),
            input,
            FormatOptions::default(),
            None,
        )
        .unwrap();
    }
}

#[test]
fn json5_comments_cannot_become_yamark_format_directives() {
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        "{// fmt: skip file\na: 1}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "# [json5] fmt: skip file\na: 1\n");
}

#[test]
fn json5_supports_each_string_line_continuation() {
    let input = "{lf:'a\\\nb',cr:'a\\\rb',crlf:'a\\\r\nb',ls:'a\\\u{2028}b',ps:'a\\\u{2029}b'}\n";
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        input.to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(
        rendered.output,
        "{lf: ab, cr: ab, crlf: ab, ls: ab, ps: ab}\n"
    );
}

#[test]
fn json5_line_comments_accept_each_json5_line_terminator() {
    let rendered = project_source_to_yaml(
        Path::new("input.json5"),
        "{// before ls\u{2028}a:1,// before ps\u{2029}b:2}\n".to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    assert_eq!(rendered.output, "# before ls\na: 1\n# before ps\nb: 2\n");
}

#[test]
fn jsonc_comments_accept_cr_and_unicode_line_breaks() {
    let rendered = project_source_to_yaml(
        Path::new("input.jsonc"),
        "{// line comment\r\"a\":1,/* cr\rafter cr\u{2028}after ls\u{2029}after ps */\"b\":2}\n"
            .to_owned(),
        FormatOptions::default(),
        None,
    )
    .unwrap();

    for line in [
        "# line comment",
        "# cr",
        "# after cr",
        "# after ls",
        "# after ps",
    ] {
        assert!(rendered.output.contains(line), "missing {line:?}");
    }
    assert!(rendered.output.contains("a: 1"));
    assert!(rendered.output.contains("b: 2"));
}

#[test]
fn jsonc_unicode_comment_breaks_keep_source_diagnostic_columns() {
    for (input, column) in [
        ("{// comment\u{2028}\"a\":}\n", 5),
        ("{/* before\u{2029}after */\"a\":}\n", 13),
    ] {
        let error = project_source_to_yaml(
            Path::new("input.jsonc"),
            input.to_owned(),
            FormatOptions::default(),
            None,
        )
        .unwrap_err();

        assert_eq!(error.diagnostic.line, 2);
        assert_eq!(error.diagnostic.column, column);
    }
}

#[test]
fn json_family_comments_escape_yaml_unsafe_characters() {
    let unsafe_characters = "\u{007f}\u{0080}\u{0085}\u{009f}\u{fffe}\u{ffff}";
    for extension in ["jsonc", "json5"] {
        let input = format!("{{// before{unsafe_characters}after\n\"a\":1}}\n");
        let rendered = project_source_to_yaml(
            Path::new(&format!("input.{extension}")),
            input,
            FormatOptions::default(),
            None,
        )
        .unwrap();

        assert_eq!(
            rendered.output,
            "# before\\x7F\\x80\\x85\\x9F\\uFFFE\\uFFFFafter\na: 1\n"
        );
    }
}

#[test]
fn json_family_strings_escape_yaml_forbidden_characters() {
    for extension in ["json", "jsonc", "json5"] {
        let rendered = project_source_to_yaml(
            Path::new(&format!("input.{extension}")),
            "{\"v\":\"\\u0085\\u007F\\u0080\\u009F\\uFFFE\\uFFFF\"}\n".to_owned(),
            FormatOptions::default(),
            None,
        )
        .unwrap();

        assert_eq!(
            rendered.output,
            "{v: \"\\N\\x7F\\x80\\x9F\\uFFFE\\uFFFF\"}\n"
        );
    }
}

#[test]
fn json_family_preserves_comments_inside_empty_containers() {
    for extension in ["jsonc", "json5"] {
        let input = if extension == "jsonc" {
            "{\"a\": {/* empty object */}, \"b\": [/* empty array */]}\n"
        } else {
            "{a: {/* empty object */}, b: [/* empty array */]}\n"
        };
        let rendered = project_source_to_yaml(
            Path::new(&format!("input.{extension}")),
            input.to_owned(),
            FormatOptions::default(),
            None,
        )
        .unwrap();

        assert_eq!(
            rendered.output,
            "# empty object\na: {}\n# empty array\nb: []\n"
        );
    }
}
