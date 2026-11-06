use std::time::{Duration, Instant};

use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions, MarkdownWrap};
use yamark::core::parser::format_source;
use yamark::plugins::PluginRegistry;

fn measure_format(input: String) -> Duration {
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::Sentence,
        skip_embedded_formatters: true,
        ..FormatOptions::default()
    };
    let started = Instant::now();
    format_source(
        FileKind::Markdown,
        input,
        options,
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    started.elapsed()
}

#[test]
fn adjacent_setext_headings_format_near_linearly() {
    let small = measure_format("Heading text\n---\n".repeat(1_000));
    let large = measure_format("Heading text\n---\n".repeat(4_000));

    assert!(
        large.as_nanos() <= small.as_nanos().max(1) * 8,
        "adjacent setext headings should format near-linearly: \
         1000 headings took {small:?}, 4000 headings took {large:?}"
    );
}
