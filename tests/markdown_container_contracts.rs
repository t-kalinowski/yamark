//! Public contracts for container refactors. Exact output demonstrates useful
//! formatting; idempotence alone would also accept a formatter that copied input.

use yamark::config::Config;
use yamark::core::document::{
    DocumentKind, FileKind, FormatOptions, MarkdownNodeKind, MarkdownWrap, NodeKind,
};
use yamark::core::emit::emit_document;
use yamark::core::parser::{format_source, parse_source};
use yamark::core::source::{SourceBuffer, Span};
use yamark::plugins::PluginRegistry;

fn format(input: &str, wrap: MarkdownWrap) -> String {
    format_source(
        FileKind::Markdown,
        input.to_owned(),
        FormatOptions {
            markdown_wrap: wrap,
            skip_embedded_formatters: true,
            ..FormatOptions::default()
        },
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap()
}

fn assert_formatted(input: &str, expected: &str, wrap: MarkdownWrap) {
    assert_ne!(
        input, expected,
        "a positive formatting case must change input"
    );
    assert_eq!(format(input, wrap), expected, "input: {input:?}");
    assert_eq!(
        format(expected, wrap),
        expected,
        "second pass: {expected:?}"
    );
}

#[test]
fn nested_containers_format_prose_without_changing_inline_literal_bytes() {
    let cases = [
        (
            "First   `keep  é` sentence. Second   sentence.\n",
            "First `keep  é` sentence.\nSecond sentence.\n",
        ),
        (
            "- First   `keep  é` sentence. Second   sentence.\n",
            "- First `keep  é` sentence.\n  Second sentence.\n",
        ),
        (
            "> First   `keep  é` sentence. Second   sentence.\n",
            "> First `keep  é` sentence.\n> Second sentence.\n",
        ),
        (
            "- Parent.\n  - First   `keep  é` sentence. Second   sentence.\n",
            "- Parent.\n  - First `keep  é` sentence.\n    Second sentence.\n",
        ),
        (
            "> > First   `keep  é` sentence. Second   sentence.\n",
            "> > First `keep  é` sentence.\n> > Second sentence.\n",
        ),
    ];
    for (input, expected) in cases {
        for newline in ["\n", "\r\n"] {
            assert_formatted(
                &input.replace('\n', newline),
                &expected.replace('\n', newline),
                MarkdownWrap::Sentence,
            );
        }
    }
}

#[test]
fn container_source_spans_remain_absolute_inside_a_document_subrange() {
    let prefix = "Outside café.\n\n";
    let body = "- Café   first.\n  Continued   words.\n\n> Quoted   first.\n> Continued   words.\n";
    let suffix = "\nOutside suffix.\n";
    let source = SourceBuffer::new(format!("{prefix}{body}{suffix}"));
    let range = Span::new(prefix.len(), prefix.len() + body.len());
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::Paragraph,
        ..FormatOptions::default()
    };
    let document = parse_source(
        source,
        range,
        DocumentKind::Markdown,
        options,
        &Config::default(),
    )
    .unwrap();

    assert_eq!(document.range, range);
    let mut previous_end = range.start;
    let mut kinds = Vec::new();
    for node in &document.nodes {
        assert!(node.span.start >= previous_end, "source spans overlap");
        assert!(
            node.span.end <= range.end,
            "node escapes the requested range"
        );
        let original = document.source().slice(node.span);
        assert!(!original.contains("Outside"));
        if let NodeKind::Markdown(kind) = &node.kind
            && *kind != MarkdownNodeKind::Blank
        {
            kinds.push(kind.clone());
        }
        previous_end = node.span.end;
    }
    assert_eq!(
        kinds,
        vec![MarkdownNodeKind::List, MarkdownNodeKind::Blockquote]
    );
    assert_eq!(
        emit_document(&document.finalize(options), &PluginRegistry::default()).unwrap(),
        "- Café first. Continued words.\n\n> Quoted first. Continued words.\n"
    );
}

#[test]
fn late_file_policy_reaches_nested_container_prose() {
    let input = "- Before __strong__. Second sentence.\n  - Nested _words_. Another sentence.\n\n> Quoted __words__. Another sentence.\n\n<!-- fmt: canonical=true wrap=sentence scope=file -->\n";
    let expected = "- Before **strong**.\n  Second sentence.\n  - Nested *words*.\n    Another sentence.\n\n> Quoted **words**.\n> Another sentence.\n\n<!-- fmt: canonical=true wrap=sentence scope=file -->\n";
    assert_formatted(input, expected, MarkdownWrap::None);
}

#[test]
fn opaque_child_fences_keep_bytes_while_list_prose_formats() {
    let input = "- First   sentence. Second   sentence.\n\n  ```text\n  keep  é  bytes\n  # not a heading\n  ```\n\n- Other   sentence. Next   sentence.\n";
    let expected = "- First sentence.\n  Second sentence.\n\n  ```text\n  keep  é  bytes\n  # not a heading\n  ```\n\n- Other sentence.\n  Next sentence.\n";
    assert_formatted(input, expected, MarkdownWrap::Sentence);
}

#[test]
fn explicit_skip_preserves_container_bytes_without_swallowing_following_prose() {
    let input = "<!-- fmt: skip -->\n- Keep   `literal  é`. Another   sentence.\n  - Nested   words.\n\nOutside   first. Outside   second.\n";
    let expected = "<!-- fmt: skip -->\n- Keep   `literal  é`. Another   sentence.\n  - Nested   words.\n\nOutside first.\nOutside second.\n";
    assert_formatted(input, expected, MarkdownWrap::Sentence);
}
