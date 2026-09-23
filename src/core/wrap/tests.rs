use std::cell::Cell;

use crate::config::Config;
use crate::core::document::{FileKind, FormatOptions, MarkdownWrap};
use crate::core::parser::format_source_report;
use crate::plugins::PluginRegistry;

thread_local! {
    pub(super) static EMPHASIS_CONTENT_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[test]
fn emphasis_content_scanning_stays_bounded_when_formatting_markdown() {
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::Sentence,
        ..FormatOptions::default()
    };
    for marker in ['*', '_'] {
        for count in [32, 64, 128] {
            let openers = vec![format!("{marker}x"); count].join(" ");
            for closed in [false, true] {
                let suffix = if closed {
                    format!(" late{marker}")
                } else {
                    String::new()
                };
                let input = format!("Before\nwith `code` {openers}{suffix} after.\n");
                let expected = input.replacen('\n', " ", 1);
                EMPHASIS_CONTENT_BYTES.set(0);
                let formatted = format_source_report(
                    FileKind::Markdown,
                    input.clone(),
                    options,
                    &config,
                    &plugins,
                )
                .unwrap();
                let scanned = EMPHASIS_CONTENT_BYTES.get();
                assert_eq!(formatted.output, expected);
                // Bound growing-content scans, not the remaining quadratic
                // candidate visits. Allow multiple passes through the public API.
                assert!(
                    scanned <= 4 * input.len(),
                    "{marker}, count={count}, closed={closed}: scanned {scanned} content bytes for {} input bytes",
                    input.len()
                );
                if closed {
                    assert!(scanned > 0, "the valid closer must exercise the counter");
                }
            }
        }
    }
}
