use yamark::config::Config;
use yamark::core::document::{DocumentKind, FormatOptions};
use yamark::core::emit::emit_document;
use yamark::core::parser::parse_source;
use yamark::core::source::{MAX_SOURCE_SPAN_OFFSET, SourceBuffer, Span};
use yamark::core::yaml_model::{
    YamlAstKind, YamlBlockChomp, YamlEmitPlan, YamlRenderedKind, YamlScalar, YamlScalarStyle,
    YamlTriviaKind,
};
use yamark::plugins::PluginRegistry;

#[test]
fn parse_source_rejects_ranges_that_exceed_compact_source_span_limit() {
    let source = SourceBuffer::new(String::new());
    let err = parse_source(
        &source,
        Span::new(0, u32::MAX as usize + 1),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap_err();

    assert_eq!(
        err.diagnostic.message,
        format!("source input exceeds supported maximum of {MAX_SOURCE_SPAN_OFFSET} bytes")
    );
}

#[test]
fn multiline_json_flow_mapping_builds_yaml_ast() {
    let input = "\
{
  \"a\" : [ 1, 2 ],
  \"b\" : { \"c\" : true }
}
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();

    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 2);

    let YamlAstKind::Scalar(key) = &ast.node(mapping.pairs[0].key).kind else {
        panic!("expected scalar key");
    };
    assert_eq!(key.style, YamlScalarStyle::DoubleQuoted);

    let first_value = mapping.pairs[0].value.unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(first_value).kind else {
        panic!("expected first value flow sequence");
    };
    assert_eq!(sequence.entries.len(), 2);

    let second_value = mapping.pairs[1].value.unwrap();
    let YamlAstKind::FlowMapping(nested) = &ast.node(second_value).kind else {
        panic!("expected second value flow mapping");
    };
    assert_eq!(nested.pairs.len(), 1);
}

#[test]
fn flow_plain_scalars_keep_colons_that_are_not_value_indicators() {
    let input = "[http://example.com, ns:tag, a:b:c]\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(root).kind else {
        panic!("expected root flow sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.entries.len(), 3);

    for (entry, expected) in sequence
        .entries
        .iter()
        .zip(["http://example.com", "ns:tag", "a:b:c"])
    {
        let YamlAstKind::Scalar(scalar) = &ast.node(*entry).kind else {
            panic!("expected scalar entry, got {:?}", ast.node(*entry).kind);
        };
        assert_eq!(source.slice(scalar.value), expected);
    }
}

#[test]
fn flow_mapping_plain_keys_can_contain_non_indicator_colons() {
    let input = "{http://example.com: website, ns:tag: value, \"json\":true}\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 3);

    for (pair, expected_key, expected_value) in [
        (&mapping.pairs[0], "http://example.com", "website"),
        (&mapping.pairs[1], "ns:tag", "value"),
        (&mapping.pairs[2], "\"json\"", "true"),
    ] {
        let YamlAstKind::Scalar(key) = &ast.node(pair.key).kind else {
            panic!("expected scalar key, got {:?}", ast.node(pair.key).kind);
        };
        assert_eq!(source.slice(key.value), expected_key);

        let value = pair.value.unwrap();
        let YamlAstKind::Scalar(value) = &ast.node(value).kind else {
            panic!("expected scalar value, got {:?}", ast.node(value).kind);
        };
        assert_eq!(source.slice(value.value), expected_value);
    }
}

#[test]
fn block_plain_scalars_own_their_continuation_lines() {
    let input = "\
description: first line
  second line
title:
  alpha
  beta
next: value
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 1);
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 3);

    let description = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(description) = &ast.node(description).kind else {
        panic!(
            "expected description scalar, got {:?}",
            ast.node(description).kind
        );
    };
    assert_eq!(source.slice(description.value), "first line\n  second line");

    let title = mapping.pairs[1].value.unwrap();
    let YamlAstKind::Scalar(title) = &ast.node(title).kind else {
        panic!("expected title scalar, got {:?}", ast.node(title).kind);
    };
    assert_eq!(source.slice(title.value), "alpha\n  beta");
    assert_eq!(
        source.slice(ast.node(mapping.pairs[1].value.unwrap()).span),
        "  alpha\n  beta\n"
    );
}

#[test]
fn scalar_value_text_borrows_from_source_lifetime() {
    fn value_text<'src>(source: &'src SourceBuffer, scalar: &YamlScalar<'src>) -> &'src str {
        scalar.value.as_str(source)
    }

    let input = "\
description: first line
  second line
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    let value = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(scalar) = &ast.node(value).kind else {
        panic!("expected scalar, got {:?}", ast.node(value).kind);
    };

    assert_eq!(value_text(&source, scalar), "first line\n  second line");
}

#[test]
fn planned_yaml_flow_block_output_is_deferred_until_emit() {
    let input = "items: [one, two, three]\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions {
            line_width: 12,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();

    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping");
    };
    let value = mapping.pairs[0].value.unwrap();
    let value_node = ast.node(value);
    assert!(
        matches!(
            value_node.emit,
            YamlEmitPlan::Rendered(YamlRenderedKind::BlockFlowCollection)
        ),
        "expected planned block flow collection, got {:?}",
        value_node.emit
    );

    let output = emit_document(
        &source,
        &document,
        FormatOptions {
            line_width: 12,
            ..FormatOptions::default()
        },
        &PluginRegistry::default(),
    )
    .unwrap();

    assert_eq!(output, "items:\n  - one\n  - two\n  - three\n");
}

#[test]
fn planned_yaml_compact_collection_output_is_deferred_until_emit() {
    let input = "items:\n  - one\n  - two\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions {
            yaml_compact: true,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();

    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping");
    };
    let value = mapping.pairs[0].value.unwrap();
    assert!(
        matches!(
            ast.node(value).emit,
            YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
        ),
        "expected deferred compact collection, got {:?}",
        ast.node(value).emit
    );

    let output = emit_document(
        &source,
        &document,
        FormatOptions {
            yaml_compact: true,
            ..FormatOptions::default()
        },
        &PluginRegistry::default(),
    )
    .unwrap();

    assert_eq!(output, "items: [one, two]\n");
}

#[test]
fn folded_block_scalar_rewrap_is_stored_in_yaml_emit_plan() {
    let input = "\
body: >
  one two three four five six
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions {
            prose_width: 12,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping");
    };
    let value = mapping.pairs[0].value.unwrap();
    let value_node = ast.node(value);
    assert!(
        matches!(
            value_node.emit,
            YamlEmitPlan::Rendered(YamlRenderedKind::Scalar)
        ),
        "expected folded scalar rewrap to be planned, got {:?}",
        value_node.emit
    );
}

#[test]
fn standard_yaml_directives_are_attached_as_trivia() {
    let input = "\
%YAML 1.2
%TAG !e! tag:example.com,2026:
---
\"key\": value
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 1);
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    assert_eq!(
        ast.node(root)
            .leading_trivia
            .iter()
            .map(|trivia| trivia.kind)
            .collect::<Vec<_>>(),
        vec![
            YamlTriviaKind::Directive,
            YamlTriviaKind::Directive,
            YamlTriviaKind::DocumentMarker,
        ]
    );
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(source.slice(mapping.pairs[0].key), "\"key\"");
}

#[test]
fn quoted_and_flow_block_keys_build_mapping_ast() {
    let input = "\
\"quoted:key\": value
[flow, key]: sequence key
{flow: map}: mapping key
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 3);
    assert_eq!(source.slice(mapping.pairs[0].key), "\"quoted:key\"");
    assert_eq!(source.slice(mapping.pairs[1].key), "[flow, key]");
    assert_eq!(source.slice(mapping.pairs[2].key), "{flow: map}");
}

#[test]
fn anchors_before_flow_collections_keep_the_collection_structural() {
    let input = "\
defaults: &defaults {\"enabled\": true}
typed: !flags [1, 2]
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };

    let defaults = mapping.pairs[0].value.unwrap();
    let YamlAstKind::FlowMapping(defaults) = &ast.node(defaults).kind else {
        panic!(
            "expected anchored flow mapping, got {:?}",
            ast.node(defaults).kind
        );
    };
    assert_eq!(source.slice(defaults.anchor.unwrap()), "&defaults");
    assert_eq!(defaults.pairs.len(), 1);

    let typed = mapping.pairs[1].value.unwrap();
    let YamlAstKind::FlowSequence(typed) = &ast.node(typed).kind else {
        panic!(
            "expected tagged flow sequence, got {:?}",
            ast.node(typed).kind
        );
    };
    assert_eq!(source.slice(typed.tag.unwrap()), "!flags");
    assert_eq!(typed.entries.len(), 2);
}

#[test]
fn aliases_are_ast_nodes() {
    let input = "\
defaults: &defaults {\"enabled\": true}
use: *defaults
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    let alias = mapping.pairs[1].value.unwrap();
    let YamlAstKind::Alias(alias) = &ast.node(alias).kind else {
        panic!("expected alias, got {:?}", ast.node(alias).kind);
    };
    assert_eq!(source.slice(alias.value), "*defaults");
}

#[test]
fn comments_inside_flow_collections_keep_structural_ast() {
    let input = "\
{
  # leading entry
  \"a\": 1,
  \"b\": [
    2, # two
    3,
  ],
}
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert!(mapping.has_inner_trivia);
    assert_eq!(mapping.pairs.len(), 2);

    let nested = mapping.pairs[1].value.unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(nested).kind else {
        panic!(
            "expected nested flow sequence, got {:?}",
            ast.node(nested).kind
        );
    };
    assert!(sequence.has_inner_trivia);
    assert_eq!(sequence.entries.len(), 2);
}

#[test]
fn tags_and_anchors_before_block_collections_attach_to_the_collection() {
    let input = "\
root: !settings &defaults
  enabled: true
items: !items
  - one
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };

    let nested_mapping = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Mapping(nested_mapping) = &ast.node(nested_mapping).kind else {
        panic!(
            "expected tagged block mapping, got {:?}",
            ast.node(nested_mapping).kind
        );
    };
    assert_eq!(source.slice(nested_mapping.tag.unwrap()), "!settings");
    assert_eq!(source.slice(nested_mapping.anchor.unwrap()), "&defaults");

    let nested_sequence = mapping.pairs[1].value.unwrap();
    let YamlAstKind::Sequence(nested_sequence) = &ast.node(nested_sequence).kind else {
        panic!(
            "expected tagged block sequence, got {:?}",
            ast.node(nested_sequence).kind
        );
    };
    assert_eq!(source.slice(nested_sequence.tag.unwrap()), "!items");
}

#[test]
fn indentless_sequence_is_mapping_value() {
    let input = "\
items:
- one
- two
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 1);
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    let value = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Sequence(sequence) = &ast.node(value).kind else {
        panic!(
            "expected mapping value sequence, got {:?}",
            ast.node(value).kind
        );
    };
    assert_eq!(sequence.items.len(), 2);
}

#[test]
fn explicit_block_keys_build_mapping_ast() {
    let input = "\
? [red, blue]
: purple
? {left: right}
: mirror
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 2);
    assert!(mapping.pairs[0].explicit);
    assert!(mapping.pairs[1].explicit);
    assert_eq!(source.slice(mapping.pairs[0].key), "[red, blue]");
    assert_eq!(source.slice(mapping.pairs[1].key), "{left: right}");
    assert_eq!(
        source.slice(mapping.pairs[0].source),
        "? [red, blue]\n: purple\n"
    );
}

#[test]
fn flow_mapping_keys_can_be_flow_nodes() {
    let input = "{[red, blue]: purple, {left: right}: mirror}\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 2);
    let YamlAstKind::FlowSequence(sequence_key) = &ast.node(mapping.pairs[0].key).kind else {
        panic!(
            "expected flow sequence key, got {:?}",
            ast.node(mapping.pairs[0].key).kind
        );
    };
    assert_eq!(sequence_key.entries.len(), 2);
    let YamlAstKind::FlowMapping(mapping_key) = &ast.node(mapping.pairs[1].key).kind else {
        panic!(
            "expected flow mapping key, got {:?}",
            ast.node(mapping.pairs[1].key).kind
        );
    };
    assert_eq!(mapping_key.pairs.len(), 1);
}

#[test]
fn flow_sequence_entries_can_be_mappings() {
    let input = "[a: b, {c: d}]\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(root).kind else {
        panic!("expected root flow sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.entries.len(), 2);
    let YamlAstKind::FlowMapping(entry_mapping) = &ast.node(sequence.entries[0]).kind else {
        panic!(
            "expected first sequence entry to be a mapping, got {:?}",
            ast.node(sequence.entries[0]).kind
        );
    };
    assert!(!entry_mapping.braced);
    assert_eq!(entry_mapping.pairs.len(), 1);
    let YamlAstKind::FlowMapping(braced_mapping) = &ast.node(sequence.entries[1]).kind else {
        panic!(
            "expected second sequence entry to be a mapping, got {:?}",
            ast.node(sequence.entries[1]).kind
        );
    };
    assert!(braced_mapping.braced);
}

#[test]
fn compact_nested_block_sequences_build_sequence_ast() {
    let input = "\
- - one
  - two
- three
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Sequence(sequence) = &ast.node(root).kind else {
        panic!("expected root sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.items.len(), 2);

    let nested = sequence.items[0].value.unwrap();
    let YamlAstKind::Sequence(nested_sequence) = &ast.node(nested).kind else {
        panic!(
            "expected first item to be a nested sequence, got {:?}",
            ast.node(nested).kind
        );
    };
    assert_eq!(nested_sequence.items.len(), 2);

    let first_nested_value = nested_sequence.items[0].value.unwrap();
    let YamlAstKind::Scalar(first_nested_scalar) = &ast.node(first_nested_value).kind else {
        panic!(
            "expected nested sequence item to be a scalar, got {:?}",
            ast.node(first_nested_value).kind
        );
    };
    assert_eq!(source.slice(first_nested_scalar.value).trim(), "one");
}

#[test]
fn nested_flow_node_properties_are_attached_to_nodes() {
    let input = "[!thing &name \"value\", &defaults {enabled: true}, !seq [1]]\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(root).kind else {
        panic!("expected root flow sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.entries.len(), 3);

    let YamlAstKind::Scalar(scalar) = &ast.node(sequence.entries[0]).kind else {
        panic!(
            "expected first flow entry to be a scalar, got {:?}",
            ast.node(sequence.entries[0]).kind
        );
    };
    assert_eq!(source.slice(scalar.tag.unwrap()), "!thing");
    assert_eq!(source.slice(scalar.anchor.unwrap()), "&name");
    assert_eq!(scalar.style, YamlScalarStyle::DoubleQuoted);

    let YamlAstKind::FlowMapping(mapping) = &ast.node(sequence.entries[1]).kind else {
        panic!(
            "expected second flow entry to be a mapping, got {:?}",
            ast.node(sequence.entries[1]).kind
        );
    };
    assert_eq!(source.slice(mapping.anchor.unwrap()), "&defaults");

    let YamlAstKind::FlowSequence(nested_sequence) = &ast.node(sequence.entries[2]).kind else {
        panic!(
            "expected third flow entry to be a sequence, got {:?}",
            ast.node(sequence.entries[2]).kind
        );
    };
    assert_eq!(source.slice(nested_sequence.tag.unwrap()), "!seq");
}

#[test]
fn flow_comments_are_attached_as_trivia() {
    let input = "\
{
  # before a
  a: 1,
  # before b
  b: 2,
}
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(
        mapping
            .inner_trivia
            .iter()
            .map(|trivia| trivia.kind)
            .collect::<Vec<_>>(),
        vec![YamlTriviaKind::Comment, YamlTriviaKind::Comment]
    );
}

#[test]
fn block_mapping_keys_are_ast_nodes() {
    let input = "\
plain: value
\"quoted\": value
[flow, key]: value
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };

    let YamlAstKind::Scalar(plain_key) = &ast.node(mapping.pairs[0].key_node.unwrap()).kind else {
        panic!("expected plain key scalar");
    };
    assert_eq!(plain_key.style, YamlScalarStyle::Plain);

    let YamlAstKind::Scalar(quoted_key) = &ast.node(mapping.pairs[1].key_node.unwrap()).kind else {
        panic!("expected quoted key scalar");
    };
    assert_eq!(quoted_key.style, YamlScalarStyle::DoubleQuoted);

    let YamlAstKind::FlowSequence(flow_key) = &ast.node(mapping.pairs[2].key_node.unwrap()).kind
    else {
        panic!("expected flow key sequence");
    };
    assert_eq!(flow_key.entries.len(), 2);
}

#[test]
fn explicit_block_keys_can_be_block_sequences() {
    let input = "\
? - Detroit Tigers
  - Chicago Cubs
: 2001-07-23
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 1);
    assert!(mapping.pairs[0].explicit);

    let key = mapping.pairs[0].key_node.unwrap();
    let YamlAstKind::Sequence(sequence) = &ast.node(key).kind else {
        panic!("expected block sequence key, got {:?}", ast.node(key).kind);
    };
    assert_eq!(sequence.items.len(), 2);
}

#[test]
fn quoted_scalars_can_span_lines() {
    let input = "\
title: \"first
  second\"
items:
  - 'one
    two'
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 2);

    let title = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(title) = &ast.node(title).kind else {
        panic!("expected title scalar, got {:?}", ast.node(title).kind);
    };
    assert_eq!(title.style, YamlScalarStyle::DoubleQuoted);
    assert_eq!(source.slice(title.value), "\"first\n  second\"");

    let items = mapping.pairs[1].value.unwrap();
    let YamlAstKind::Sequence(sequence) = &ast.node(items).kind else {
        panic!("expected items sequence, got {:?}", ast.node(items).kind);
    };
    let item = sequence.items[0].value.unwrap();
    let YamlAstKind::Scalar(item) = &ast.node(item).kind else {
        panic!(
            "expected sequence item scalar, got {:?}",
            ast.node(item).kind
        );
    };
    assert_eq!(item.style, YamlScalarStyle::SingleQuoted);
    assert_eq!(source.slice(item.value), "'one\n    two'");
}

#[test]
fn block_scalar_header_comments_do_not_hide_block_scalars() {
    let input = "\
body: | # markdown
  line one
  line two
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 1);

    let body = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(body) = &ast.node(body).kind else {
        panic!("expected block scalar, got {:?}", ast.node(body).kind);
    };
    assert_eq!(body.style, YamlScalarStyle::LiteralBlock);
    assert_eq!(source.slice(body.trailing_comment.unwrap()), "# markdown");
    assert_eq!(source.slice(body.body.unwrap()), "  line one\n  line two\n");
}

#[test]
fn block_scalar_headers_record_indent_and_chomping_indicators() {
    let input = "\
body: |4-
    text
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };

    let body = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(body) = &ast.node(body).kind else {
        panic!("expected block scalar, got {:?}", ast.node(body).kind);
    };
    let header = body.block_header.unwrap();
    assert_eq!(header.indent, Some(4));
    assert_eq!(header.chomp, YamlBlockChomp::Strip);
}

#[test]
fn blank_lines_inside_flow_collections_are_trivia() {
    let input = "\
[
  1,

  2,
]
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(root).kind else {
        panic!("expected root flow sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.entries.len(), 2);
    assert_eq!(
        sequence
            .inner_trivia
            .iter()
            .map(|trivia| trivia.kind)
            .collect::<Vec<_>>(),
        vec![YamlTriviaKind::Blank]
    );
}

#[test]
fn flow_mapping_explicit_and_empty_entries_build_ast() {
    let input = "\
{
  ? explicit: entry,
  implicit: entry,
  http://foo.com,
  omitted value:,
  : omitted key,
  ?
}
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 6);
    assert!(mapping.pairs[0].explicit);
    assert_eq!(source.slice(mapping.pairs[0].source), "? explicit: entry");
    assert!(!mapping.pairs[1].explicit);

    let YamlAstKind::Scalar(explicit_key) = &ast.node(mapping.pairs[0].key).kind else {
        panic!("expected explicit key scalar");
    };
    assert_eq!(source.slice(explicit_key.value), "explicit");

    let YamlAstKind::Scalar(key_only_value) = &ast.node(mapping.pairs[2].value.unwrap()).kind
    else {
        panic!("expected omitted value scalar");
    };
    assert!(key_only_value.value.is_empty());

    let YamlAstKind::Scalar(empty_key) = &ast.node(mapping.pairs[4].key).kind else {
        panic!("expected empty key scalar");
    };
    assert!(empty_key.value.is_empty());

    let YamlAstKind::Scalar(empty_explicit_key) = &ast.node(mapping.pairs[5].key).kind else {
        panic!("expected empty explicit key scalar");
    };
    assert!(empty_explicit_key.value.is_empty());
    let YamlAstKind::Scalar(empty_explicit_value) = &ast.node(mapping.pairs[5].value.unwrap()).kind
    else {
        panic!("expected empty explicit value scalar");
    };
    assert!(empty_explicit_value.value.is_empty());
}

#[test]
fn flow_tag_only_nodes_are_empty_scalars_with_properties() {
    let input = "{foo: !!str, !!str: bar, anchored: &empty}\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowMapping(mapping) = &ast.node(root).kind else {
        panic!("expected root flow mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 3);

    let YamlAstKind::Scalar(tagged_value) = &ast.node(mapping.pairs[0].value.unwrap()).kind else {
        panic!("expected tagged empty value scalar");
    };
    assert!(tagged_value.value.is_empty());
    assert_eq!(source.slice(tagged_value.tag.unwrap()), "!!str");

    let YamlAstKind::Scalar(tagged_key) = &ast.node(mapping.pairs[1].key).kind else {
        panic!("expected tagged empty key scalar");
    };
    assert!(tagged_key.value.is_empty());
    assert_eq!(source.slice(tagged_key.tag.unwrap()), "!!str");

    let YamlAstKind::Scalar(anchored_value) = &ast.node(mapping.pairs[2].value.unwrap()).kind
    else {
        panic!("expected anchored empty value scalar");
    };
    assert!(anchored_value.value.is_empty());
    assert_eq!(source.slice(anchored_value.anchor.unwrap()), "&empty");
}

#[test]
fn flow_sequence_explicit_single_pair_entries_build_mapping_ast() {
    let input = "\
[
  ? foo
    bar : baz,
  : empty key entry,
  {JSON: like}:adjacent
]
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::FlowSequence(sequence) = &ast.node(root).kind else {
        panic!("expected root flow sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.entries.len(), 3);

    let YamlAstKind::FlowMapping(explicit_entry) = &ast.node(sequence.entries[0]).kind else {
        panic!(
            "expected explicit sequence entry mapping, got {:?}",
            ast.node(sequence.entries[0]).kind
        );
    };
    assert!(!explicit_entry.braced);
    assert_eq!(explicit_entry.pairs.len(), 1);
    assert!(explicit_entry.pairs[0].explicit);

    let YamlAstKind::Scalar(empty_key) = &ast
        .node(match &ast.node(sequence.entries[1]).kind {
            YamlAstKind::FlowMapping(mapping) => mapping.pairs[0].key,
            other => panic!("expected empty-key mapping, got {other:?}"),
        })
        .kind
    else {
        panic!("expected empty key scalar");
    };
    assert!(empty_key.value.is_empty());

    let YamlAstKind::FlowMapping(collection_key_entry) = &ast.node(sequence.entries[2]).kind else {
        panic!(
            "expected collection-key entry mapping, got {:?}",
            ast.node(sequence.entries[2]).kind
        );
    };
    let YamlAstKind::FlowMapping(collection_key) =
        &ast.node(collection_key_entry.pairs[0].key).kind
    else {
        panic!("expected mapping key");
    };
    assert_eq!(collection_key.pairs.len(), 1);
}

#[test]
fn document_markers_with_trailing_comments_are_trivia() {
    let input = "\
--- # document
value
... # end
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 1);
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    assert_eq!(
        ast.node(root)
            .leading_trivia
            .iter()
            .map(|trivia| trivia.kind)
            .collect::<Vec<_>>(),
        vec![YamlTriviaKind::DocumentMarker]
    );
    assert_eq!(
        ast.trailing_trivia
            .iter()
            .map(|trivia| trivia.kind)
            .collect::<Vec<_>>(),
        vec![YamlTriviaKind::DocumentMarker]
    );
    let YamlAstKind::Scalar(scalar) = &ast.node(root).kind else {
        panic!("expected scalar root, got {:?}", ast.node(root).kind);
    };
    assert_eq!(source.slice(scalar.value), "value");
}

#[test]
fn document_stream_roots_record_marker_metadata_and_empty_documents() {
    let input = "\
--- # empty
... # empty end
--- # data
value
... # data end
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();

    assert_eq!(ast.roots.len(), 2);
    let empty = ast.roots[0].node.unwrap();
    assert!(matches!(ast.node(empty).kind, YamlAstKind::Empty));
    assert_eq!(
        source.slice(ast.roots[0].start_marker.unwrap()),
        "--- # empty\n"
    );
    assert_eq!(
        source.slice(ast.roots[0].end_marker.unwrap()),
        "... # empty end\n"
    );

    let value = ast.roots[1].node.unwrap();
    assert!(matches!(ast.node(value).kind, YamlAstKind::Scalar(_)));
    assert_eq!(
        source.slice(ast.roots[1].start_marker.unwrap()),
        "--- # data\n"
    );
    assert_eq!(
        source.slice(ast.roots[1].end_marker.unwrap()),
        "... # data end\n"
    );
}

#[test]
fn block_tag_and_anchor_only_nodes_are_empty_scalars_with_properties() {
    let input = "\
root: !!str
seq:
  - &empty
---
&standalone
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 2);
    let root = ast.roots[0].node.unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };

    let tagged_value = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(tagged_value) = &ast.node(tagged_value).kind else {
        panic!("expected tagged empty scalar");
    };
    assert!(tagged_value.value.is_empty());
    assert_eq!(source.slice(tagged_value.tag.unwrap()), "!!str");

    let seq = mapping.pairs[1].value.unwrap();
    let YamlAstKind::Sequence(seq) = &ast.node(seq).kind else {
        panic!("expected sequence, got {:?}", ast.node(seq).kind);
    };
    let anchored_item = seq.items[0].value.unwrap();
    let YamlAstKind::Scalar(anchored_item) = &ast.node(anchored_item).kind else {
        panic!("expected anchored empty scalar");
    };
    assert!(anchored_item.value.is_empty());
    assert_eq!(source.slice(anchored_item.anchor.unwrap()), "&empty");

    let standalone = ast.roots[1].node.unwrap();
    let YamlAstKind::Scalar(standalone) = &ast.node(standalone).kind else {
        panic!("expected standalone anchored scalar");
    };
    assert!(standalone.value.is_empty());
    assert_eq!(source.slice(standalone.anchor.unwrap()), "&standalone");
}

#[test]
fn explicit_block_entries_can_omit_values_and_use_block_values() {
    let input = "\
? explicit key # Empty value
? |
  block key
: - one # Explicit compact
  - two # block value
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 1);
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 2);
    assert!(mapping.pairs[0].explicit);
    assert!(mapping.pairs[1].explicit);

    let first_value = mapping.pairs[0].value.unwrap();
    let YamlAstKind::Scalar(first_value) = &ast.node(first_value).kind else {
        panic!("expected empty scalar value");
    };
    assert!(first_value.value.is_empty());

    let second_key = mapping.pairs[1].key_node.unwrap();
    let YamlAstKind::Scalar(second_key) = &ast.node(second_key).kind else {
        panic!("expected block scalar key");
    };
    assert_eq!(second_key.style, YamlScalarStyle::LiteralBlock);
    assert_eq!(source.slice(second_key.body.unwrap()), "  block key\n");

    let second_value = mapping.pairs[1].value.unwrap();
    let YamlAstKind::Sequence(second_value) = &ast.node(second_value).kind else {
        panic!("expected compact sequence value");
    };
    assert_eq!(second_value.items.len(), 2);
}

#[test]
fn mixed_implicit_and_explicit_block_mapping_entries_share_one_mapping() {
    let input = "\
implicit: one
? explicit
: two
after: three
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 1);
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Mapping(mapping) = &ast.node(root).kind else {
        panic!("expected root mapping, got {:?}", ast.node(root).kind);
    };
    assert_eq!(mapping.pairs.len(), 3);
    assert!(!mapping.pairs[0].explicit);
    assert!(mapping.pairs[1].explicit);
    assert!(!mapping.pairs[2].explicit);
    assert_eq!(source.slice(mapping.pairs[1].source), "? explicit\n: two\n");
}

#[test]
fn compact_explicit_mapping_entries_in_sequences_build_nested_mappings() {
    let input = "\
- sun: yellow
- ? earth: blue
  : moon: white
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Yaml,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();
    let ast = document.yaml.as_ref().unwrap();
    let root = ast.roots.first().and_then(|root| root.node).unwrap();
    let YamlAstKind::Sequence(sequence) = &ast.node(root).kind else {
        panic!("expected root sequence, got {:?}", ast.node(root).kind);
    };
    assert_eq!(sequence.items.len(), 2);

    let explicit = sequence.items[1].value.unwrap();
    let YamlAstKind::Mapping(explicit) = &ast.node(explicit).kind else {
        panic!(
            "expected explicit entry mapping, got {:?}",
            ast.node(explicit).kind
        );
    };
    assert_eq!(explicit.pairs.len(), 1);
    assert!(explicit.pairs[0].explicit);

    let key = explicit.pairs[0].key_node.unwrap();
    let YamlAstKind::Mapping(key) = &ast.node(key).kind else {
        panic!("expected mapping key, got {:?}", ast.node(key).kind);
    };
    assert_eq!(key.pairs.len(), 1);
    assert_eq!(source.slice(key.pairs[0].key), "earth");

    let value = explicit.pairs[0].value.unwrap();
    let YamlAstKind::Mapping(value) = &ast.node(value).kind else {
        panic!("expected mapping value, got {:?}", ast.node(value).kind);
    };
    assert_eq!(value.pairs.len(), 1);
    assert_eq!(source.slice(value.pairs[0].key), "moon");
}
