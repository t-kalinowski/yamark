use std::path::Path;

use yamark::core::document::{FormatOptions, MarkdownWrap};
use yamark::workspace::format_source_for_path;

#[test]
fn sentence_wrapping_escapes_literal_block_markers() {
    for wrap in [MarkdownWrap::Sentence, MarkdownWrap::SentenceAndColumn(72)] {
        let options = FormatOptions {
            markdown_wrap: wrap,
            ..FormatOptions::default()
        };
        for (marker, escaped) in [
            ("-", r"\-"),
            ("+", r"\+"),
            ("*", r"\*"),
            ("(b)", r"(b\)"),
            ("(ii)", r"(ii\)"),
            ("2.", r"2\."),
            ("2)", r"2\)"),
            ("b.", r"b\."),
            ("ii.", r"ii\."),
            (">", r"\>"),
            ("#", r"\#"),
        ] {
            let prose =
                format!("Opening sentence. First sentence. {marker} More prose. Last sentence.");
            let middle = format!("First sentence.\n{escaped} More prose.");
            let wrapped = format!("Opening sentence.\n{middle}\nLast sentence.\n");
            for (source, expected) in [
                (format!("{prose}\n"), wrapped.clone()),
                (
                    format!("Intro line\n{prose}\n"),
                    format!("Intro line {wrapped}"),
                ),
                (
                    format!("- {prose}\n"),
                    format!("- {}\n", wrapped.trim_end().replace('\n', "\n  ")),
                ),
                (
                    format!("- Parent\n  - {prose}\n"),
                    format!(
                        "- Parent\n  - {}\n",
                        wrapped.trim_end().replace('\n', "\n    ")
                    ),
                ),
                (
                    format!("> {prose}\n"),
                    format!("> {}\n", wrapped.trim_end().replace('\n', "\n> ")),
                ),
                (
                    format!("[^note]: {prose}\n"),
                    format!("[^note]: {}\n", wrapped.trim_end().replace('\n', "\n  ")),
                ),
            ] {
                for newline in ["\n", "\r\n", "\r"] {
                    let source = source.replace('\n', newline);
                    let expected = expected.replace('\n', newline);
                    let output = format_source_for_path(
                        Path::new("input.md"),
                        source.clone(),
                        options,
                        None,
                    )
                    .unwrap()
                    .output;
                    assert_eq!(output, expected, "{wrap:?}, {source:?}");
                    let second = format_source_for_path(
                        Path::new("input.md"),
                        output.clone(),
                        options,
                        None,
                    )
                    .unwrap()
                    .output;
                    assert_eq!(second, output, "second pass: {wrap:?}, {source:?}");
                    for width in [8, 12, 20, 32, 72] {
                        let column_options = FormatOptions {
                            markdown_wrap: MarkdownWrap::Column(width),
                            ..options
                        };
                        let column = format_source_for_path(
                            Path::new("input.md"),
                            source.clone(),
                            column_options,
                            None,
                        )
                        .unwrap()
                        .output;
                        let sentence = format_source_for_path(
                            Path::new("input.md"),
                            column.clone(),
                            options,
                            None,
                        )
                        .unwrap()
                        .output;
                        assert!(sentence.contains(escaped), "escape missing: {sentence:?}");
                        let restored = format_source_for_path(
                            Path::new("input.md"),
                            sentence,
                            column_options,
                            None,
                        )
                        .unwrap()
                        .output;
                        assert_eq!(
                            restored, column,
                            "column roundtrip: {width}, {wrap:?}, {source:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn sentence_transcripts_keep_hard_breaks_and_narrow_columns_idempotent() {
    for (case, wrap) in [
        (
            include_str!("cases/markdown_container_raw_html_boundaries.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_container_literal_block_markers.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_empty_link_destinations.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_angle_link_destinations.case"),
            MarkdownWrap::SentenceAndColumn(12),
        ),
        (
            include_str!("cases/markdown_column_preserved_html_literals.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_column_math_sensitive_emphasis.case"),
            MarkdownWrap::Column(24),
        ),
        (
            include_str!("cases/markdown_column_quoted_html_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_column_final_marker_escapes.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_column_emphasis_math_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_column_hard_break_marker_escapes.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_column_hard_break_asterisk_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_column_trailing_asterisk_repairs.case"),
            MarkdownWrap::Column(23),
        ),
        (
            include_str!("cases/markdown_column_nested_headings.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_sentence_column_nested_headings.case"),
            MarkdownWrap::SentenceAndColumn(12),
        ),
        (
            include_str!("cases/markdown_sentence_definition_markers.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_reference_boundaries.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_column_opaque_html_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_sentence_column_emphasis_repairs.case"),
            MarkdownWrap::SentenceAndColumn(72),
        ),
        (
            include_str!("cases/markdown_column_emphasis_repairs.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_column_raw_tex_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_column_raw_tex_escapes.case"),
            MarkdownWrap::SentenceAndColumn(12),
        ),
        (
            include_str!("cases/markdown_sentence_shortcode_math_opener.case"),
            MarkdownWrap::SentenceAndColumn(12),
        ),
        (
            include_str!("cases/markdown_sentence_asterisk_boundaries.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_column_asterisk_escapes.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_sentence_column_math_contents.case"),
            MarkdownWrap::SentenceAndColumn(12),
        ),
        (
            include_str!("cases/markdown_sentence_setext_candidates.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_setext_candidates.case"),
            MarkdownWrap::SentenceAndColumn(72),
        ),
        (
            include_str!("cases/markdown_column_backslash_math_escapes.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_column_inline_html_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_sentence_parenthesized_marker_escapes.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_parenthesized_marker_escapes.case"),
            MarkdownWrap::SentenceAndColumn(72),
        ),
        (
            include_str!("cases/markdown_sentence_column_block_candidates.case"),
            MarkdownWrap::SentenceAndColumn(32),
        ),
        (
            include_str!("cases/markdown_sentence_block_starts.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_column_unsafe_breaks.case"),
            MarkdownWrap::SentenceAndColumn(12),
        ),
        (
            include_str!("cases/markdown_sentence_meaning_sensitive.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_escaped_punctuation.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_column_escaped_punctuation.case"),
            MarkdownWrap::SentenceAndColumn(10),
        ),
        (
            include_str!("cases/markdown_column_sentence_escapes.case"),
            MarkdownWrap::Column(72),
        ),
        (
            include_str!("cases/markdown_column_literal_marker_escapes.case"),
            MarkdownWrap::Column(12),
        ),
        (
            include_str!("cases/markdown_column_ordered_marker_escapes.case"),
            MarkdownWrap::Column(8),
        ),
        (
            include_str!("cases/markdown_column_escape_width.case"),
            MarkdownWrap::Column(10),
        ),
    ] {
        let (_, case) = case.split_once("-- stdin\n").unwrap();
        let (input, case) = case.split_once("-- stdout\n").unwrap();
        let (expected, _) = case.split_once("-- stderr\n").unwrap();
        for newline in ["\n", "\r\n", "\r"] {
            let expected = expected.replace('\n', newline);
            for source in [input.replace('\n', newline), expected.clone()] {
                let output = format_source_for_path(
                    Path::new("input.md"),
                    source.clone(),
                    FormatOptions {
                        markdown_wrap: wrap,
                        ..FormatOptions::default()
                    },
                    None,
                )
                .unwrap()
                .output;
                assert_eq!(output, expected, "{wrap:?}, {source:?}");
            }
        }
    }
}
