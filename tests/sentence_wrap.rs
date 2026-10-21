use std::path::Path;

use yamark::core::document::{FormatOptions, MarkdownWrap};
use yamark::workspace::format_source_for_path;

#[test]
fn sentence_wrapping_skips_only_unsafe_boundaries() {
    for wrap in [MarkdownWrap::Sentence, MarkdownWrap::SentenceAndColumn(72)] {
        let options = FormatOptions {
            markdown_wrap: wrap,
            ..FormatOptions::default()
        };
        for marker in [
            "-", "+", "*", "(b)", "(ii)", "2.", "2)", "b.", "ii.", ">", "#",
        ] {
            let prose =
                format!("Opening sentence. First sentence. {marker} More prose. Last sentence.");
            let middle = if marker.ends_with('.') {
                format!("First sentence. {marker}\nMore prose.")
            } else {
                format!("First sentence. {marker} More prose.")
            };
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
                }
            }
        }
    }
}

#[test]
fn sentence_transcripts_keep_hard_breaks_and_narrow_columns_idempotent() {
    for (case, wrap) in [
        (
            include_str!("cases/markdown_sentence_block_starts.case"),
            MarkdownWrap::Sentence,
        ),
        (
            include_str!("cases/markdown_sentence_column_unsafe_breaks.case"),
            MarkdownWrap::SentenceAndColumn(12),
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
