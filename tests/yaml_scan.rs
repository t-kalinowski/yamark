use yamark::core::source::{SourceBuffer, Span};
use yamark::core::yaml_scan::{
    YamlIndicatorKind, YamlLineKind, YamlPropertyTokenKind, YamlScalarTokenKind, scan_yaml_lines,
};

#[test]
fn yaml_line_scanner_records_ranges_kinds_indicators_and_scalars() {
    let input = "---\nitems:\n  - \"one\"\n# fmt: skip\n...\n";
    let source = SourceBuffer::new(input.to_owned());
    let scan = scan_yaml_lines(&source, Span::new(0, input.len()));

    assert_eq!(scan.source_scans, 1);
    assert_eq!(scan.lines.len(), 5);
    assert_eq!(scan.lines[0].kind, YamlLineKind::DocumentMarker);
    assert_eq!(scan.lines[1].kind, YamlLineKind::Other);
    assert_eq!(scan.lines[2].indent, 2);
    assert_eq!(scan.lines[3].kind, YamlLineKind::Directive);
    assert_eq!(scan.lines[4].kind, YamlLineKind::DocumentMarker);

    assert_eq!(
        scan.lines[1]
            .indicators
            .iter()
            .map(|indicator| indicator.kind)
            .collect::<Vec<_>>(),
        vec![YamlIndicatorKind::MappingValue]
    );
    assert_eq!(
        scan.lines[2].indicators[0].kind,
        YamlIndicatorKind::SequenceEntry
    );
    assert_eq!(
        scan.lines[2].scalar.as_ref().map(|scalar| scalar.kind),
        Some(YamlScalarTokenKind::DoubleQuoted)
    );
    assert_eq!(source.slice(scan.lines[2].scalar.unwrap().span), "\"one\"");
}

#[test]
fn yaml_line_scanner_classifies_markers_with_trailing_content() {
    let input = "--- # document\nvalue\n... # end\n";
    let source = SourceBuffer::new(input.to_owned());
    let scan = scan_yaml_lines(&source, Span::new(0, input.len()));

    assert_eq!(scan.lines[0].kind, YamlLineKind::DocumentMarker);
    assert_eq!(scan.lines[1].kind, YamlLineKind::Other);
    assert_eq!(scan.lines[2].kind, YamlLineKind::DocumentMarker);
}

#[test]
fn yaml_line_scanner_classifies_standard_yaml_directives() {
    let input = "%YAML 1.2\n%TAG !e! tag:example.com,2026:\n---\nkey: value\n";
    let source = SourceBuffer::new(input.to_owned());
    let scan = scan_yaml_lines(&source, Span::new(0, input.len()));

    assert_eq!(scan.lines[0].kind, YamlLineKind::Directive);
    assert_eq!(scan.lines[1].kind, YamlLineKind::Directive);
    assert_eq!(scan.lines[2].kind, YamlLineKind::DocumentMarker);
    assert_eq!(scan.lines[3].kind, YamlLineKind::Other);
}

#[test]
fn yaml_line_scanner_records_tag_anchor_and_alias_property_tokens() {
    let input = "name: &name !markdown value\nalias: *name\n";
    let source = SourceBuffer::new(input.to_owned());
    let scan = scan_yaml_lines(&source, Span::new(0, input.len()));

    assert_eq!(
        scan.lines[0]
            .properties
            .iter()
            .map(|property| (property.kind, source.slice(property.span)))
            .collect::<Vec<_>>(),
        vec![
            (YamlPropertyTokenKind::Anchor, "&name"),
            (YamlPropertyTokenKind::Tag, "!markdown"),
        ]
    );
    assert_eq!(
        scan.lines[0]
            .scalar
            .as_ref()
            .map(|scalar| { (scalar.kind, source.slice(scalar.span),) }),
        Some((YamlScalarTokenKind::Plain, "value"))
    );
    assert_eq!(
        scan.lines[1]
            .properties
            .iter()
            .map(|property| (property.kind, source.slice(property.span)))
            .collect::<Vec<_>>(),
        vec![(YamlPropertyTokenKind::Alias, "*name")]
    );
    assert_eq!(scan.lines[1].scalar, None);
}
