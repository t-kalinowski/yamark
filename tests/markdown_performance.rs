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

fn assert_near_linear(heading: &str) {
    let small = measure_format(heading.repeat(1_000));
    let large = measure_format(heading.repeat(4_000));

    assert!(
        large.as_nanos() <= small.as_nanos().max(1) * 8,
        "adjacent setext headings {heading:?} should format near-linearly: \
         1000 headings took {small:?}, 4000 headings took {large:?}"
    );
}

#[test]
fn adjacent_setext_headings_format_near_linearly() {
    assert_near_linear("Heading text\n---\n");
}

#[test]
fn adjacent_setext_headings_with_unclosed_literals_format_near_linearly() {
    assert_near_linear("Price $5\n---\n");
    assert_near_linear("Paren \\(\n---\n");
    assert_near_linear("Tick ``a\n---\nTick `b\n---\n");
}
