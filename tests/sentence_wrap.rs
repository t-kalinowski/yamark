use std::path::Path;

use yamark::core::document::{FormatOptions, MarkdownWrap};
use yamark::workspace::format_source_for_path;

#[test]
fn sentence_wrapping_preserves_prose_before_block_markers() {
    for wrap in [MarkdownWrap::Sentence, MarkdownWrap::SentenceAndColumn(72)] {
        let options = FormatOptions {
            markdown_wrap: wrap,
            ..FormatOptions::default()
        };
        for marker in [
            "-", "+", "*", "(b)", "(ii)", "2.", "2)", "b.", "ii.", ">", "#",
        ] {
            let prose = format!("First sentence. {marker} More prose.");
            for source in [
                format!("{prose}\n"),
                format!("Intro line\n{prose}\n"),
                format!("- {prose}\n"),
                format!("- Parent\n  - {prose}\n"),
                format!("> {prose}\n"),
                format!("[^note]: {prose}\n"),
            ] {
                for newline in ["\n", "\r\n", "\r"] {
                    let source = source.replace('\n', newline);
                    let output = format_source_for_path(
                        Path::new("input.md"),
                        source.clone(),
                        options,
                        None,
                    )
                    .unwrap()
                    .output;
                    assert_eq!(output, source, "{wrap:?}, {source:?}");
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
