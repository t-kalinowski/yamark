use yamark::config::Config;
use yamark::core::document::{DocumentKind, FormatOptions, MarkdownWrap};
use yamark::core::parser::parse_source;
use yamark::core::source::{SourceBuffer, Span};
use yamark::plugins::PluginRegistry;

#[test]
fn prepared_markdown_with_inline_source_offsets_emits_repeatedly() {
    let cases = [
        (
            MarkdownWrap::Column(22),
            "Plain text stays byte-for-byte.\n\n> Nested `code  span` and [link](target).\n\nChanged   gap [link](  target  ) outside.\n",
            "Plain text stays\nbyte-for-byte.\n\n> Nested `code  span`\n> and [link](target).\n\nChanged gap\n[link](target)\noutside.\n",
        ),
        (
            MarkdownWrap::Sentence,
            "Before  `raw  _text_` after _word_ [link](  target  ). Next sentence.\n\n<!-- fmt: wrap=sentence canonical=true scope=file -->\n",
            "Before `raw  _text_` after *word* [link](target).\nNext sentence.\n\n<!-- fmt: wrap=sentence canonical=true scope=file -->\n",
        ),
        (
            MarkdownWrap::None,
            "Simple body stays exact.\n\n> Exact text stays here.\n\n> - Plain `literal  spaces` and [link](target).\n>   More   text [changed](  path  ).\n",
            "Simple body stays exact.\n\n> Exact text stays here.\n\n> - Plain `literal  spaces` and [link](target). More text [changed](path).\n",
        ),
    ];
    let config = Config::default();
    let plugins = PluginRegistry::default();
    for (wrap, input, expected) in cases {
        let options = FormatOptions {
            markdown_wrap: wrap,
            skip_embedded_formatters: true,
            ..FormatOptions::default()
        };
        let source = SourceBuffer::new(input.to_owned());
        let parsed = parse_source(
            source,
            Span::new(0, input.len()),
            DocumentKind::Markdown,
            options,
            &config,
        )
        .unwrap();
        let prepared = parsed.finalize(options);
        assert_eq!(prepared.emit(&plugins).unwrap(), expected);
        assert_eq!(prepared.emit(&plugins).unwrap(), expected);

        let source = SourceBuffer::new(expected.to_owned());
        let second_pass = parse_source(
            source,
            Span::new(0, expected.len()),
            DocumentKind::Markdown,
            options,
            &config,
        )
        .unwrap()
        .finalize(options);
        assert_eq!(second_pass.emit(&plugins).unwrap(), expected);
    }
}
