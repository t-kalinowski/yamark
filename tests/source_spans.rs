use yamark::config::Config;
use yamark::core::document::{DocumentKind, FormatOptions};
use yamark::core::emit::emit_document;
use yamark::core::parser::parse_source;
use yamark::core::source::{SourceBuffer, Span};
use yamark::plugins::PluginRegistry;

#[test]
fn parsed_documents_can_move_with_their_source_buffer() {
    let options = FormatOptions::default();
    let document = {
        let source = SourceBuffer::new("name: 'Café'\n".to_owned());
        let range = Span::new(0, source.as_str().len());
        parse_source(
            source,
            range,
            DocumentKind::Yaml,
            options,
            &Config::default(),
        )
        .unwrap()
    };
    assert_eq!(
        emit_document(&document.finalize(options), &PluginRegistry::default()).unwrap(),
        "name: Café\n"
    );
}

#[test]
fn span_access_borrows_the_supplied_buffer() {
    let span = SourceBuffer::new("Café".to_owned()).lines[0].text;
    let replacement = SourceBuffer::new("Renée".to_owned());
    assert_eq!(span.as_str(&replacement), "René");
}

#[test]
fn span_access_checks_bounds_and_utf8_boundaries() {
    let span = SourceBuffer::new("a".to_owned()).lines[0].text;
    for text in ["", "é"] {
        let source = SourceBuffer::new(text.to_owned());
        assert!(std::panic::catch_unwind(|| span.as_str(&source)).is_err());
    }
}
