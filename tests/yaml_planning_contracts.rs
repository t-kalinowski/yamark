//! Public contracts at the YAML syntax, directive, and layout boundaries.
//!
//! These are characterization tests: moving planning out of the parser must
//! not change which source belongs to a node or when directive policy applies.

use yamark::config::Config;
use yamark::core::document::{DocumentKind, FileKind, FormatOptions, MarkdownWrap, ParsedDocument};
use yamark::core::emit::emit_document;
use yamark::core::parser::{format_source, format_source_report_with_trace, parse_source};
use yamark::core::source::{SourceBuffer, Span};
use yamark::core::yaml_model::{
    YamlAstKind, YamlDocumentAst, YamlNodeId, YamlScalar, YamlScalarSemantic, YamlSequence,
    YamlTrivia,
};
use yamark::plugins::PluginRegistry;

fn parse(input: &str, kind: DocumentKind, options: FormatOptions) -> ParsedDocument {
    parse_source(
        SourceBuffer::new(input.to_owned()),
        Span::new(0, input.len()),
        kind,
        options,
        &Config::default(),
    )
    .unwrap_or_else(|error| panic!("{error}\ninput:\n{input}"))
}

fn format(input: &str, kind: FileKind, options: FormatOptions) -> String {
    format_source(
        kind,
        input.to_owned(),
        options,
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap_or_else(|error| panic!("{error}\ninput:\n{input}"))
}

fn sequence(ast: &YamlDocumentAst, id: YamlNodeId) -> &YamlSequence {
    match &ast.node(id).kind {
        YamlAstKind::Sequence(sequence) => sequence,
        kind => panic!("expected block sequence, got {kind:?}"),
    }
}

fn scalar(ast: &YamlDocumentAst, id: YamlNodeId) -> &YamlScalar {
    match &ast.node(id).kind {
        YamlAstKind::Scalar(scalar) => scalar,
        kind => panic!("expected scalar, got {kind:?}"),
    }
}

#[test]
fn ordinary_and_compact_sequence_entries_keep_value_and_sibling_boundaries() {
    // Each first item exercises a different value recognizer. The second item
    // and the outer sibling prove that recognition stops at the right boundary.
    let cases = [
        ("plain", "first", "scalar"),
        ("quoted", "\"first\n    second\"", "scalar"),
        ("flow", "[first, second]", "flow sequence"),
        ("block scalar", "|-\n    first\n    second", "scalar"),
        ("explicit mapping", "? first\n    : second", "mapping"),
        ("inline mapping", "first: second", "mapping"),
        ("nested sequence", "- first\n    - second", "sequence"),
        ("properties", "&saved\n    'first'", "scalar"),
        ("empty", "", "scalar"),
    ];

    for (name, value, expected_kind) in cases {
        for prefix in ["-\n  - ", "- - "] {
            let input = format!("{prefix}{value}\n  - inner_tail\n- outer_tail\n");
            let document = parse(&input, DocumentKind::Yaml, FormatOptions::default());
            let source = document.source();
            let ast = document.yaml.as_ref().unwrap();
            assert_eq!(ast.roots.len(), 1, "{name}: {input}");
            let outer_id = ast.roots[0].node.unwrap();
            assert!(
                matches!(ast.node(outer_id).kind, YamlAstKind::Sequence(_)),
                "{name}: expected outer sequence, got {:?}\n{input}",
                ast.node(outer_id).kind
            );
            let outer = sequence(ast, outer_id);
            assert_eq!(outer.items.len(), 2, "{name}: {input}");
            let inner_id = outer.items[0].value.unwrap();
            assert!(
                matches!(ast.node(inner_id).kind, YamlAstKind::Sequence(_)),
                "{name}: expected inner sequence, got {:?}\n{input}",
                ast.node(inner_id).kind
            );
            let inner = sequence(ast, inner_id);
            assert_eq!(inner.items.len(), 2, "{name}: {input}");
            assert_eq!(inner.indent, 2, "{name}: {input}");
            assert_eq!(source.slice(inner.items[0].marker), "-");
            assert_eq!(source.slice(inner.items[1].line), "  - inner_tail\n");
            assert_eq!(
                source.slice(scalar(ast, inner.items[1].value.unwrap()).value),
                "inner_tail",
                "{name}: {input}"
            );
            assert_eq!(
                source.slice(scalar(ast, outer.items[1].value.unwrap()).value),
                "outer_tail",
                "{name}: {input}"
            );

            let value = ast.node(inner.items[0].value.unwrap());
            let actual_kind = match &value.kind {
                YamlAstKind::Scalar(_) => "scalar",
                YamlAstKind::Sequence(_) => "sequence",
                YamlAstKind::Mapping(_) => "mapping",
                YamlAstKind::FlowSequence(_) => "flow sequence",
                kind => panic!("{name}: unexpected value kind {kind:?}"),
            };
            assert_eq!(actual_kind, expected_kind, "{name}: {input}");
            assert!(
                value.span.end() <= inner.items[1].line.start(),
                "{name}: {input}"
            );
        }
    }
}

#[test]
fn ordinary_and_compact_sequences_recognize_a_later_flow_collapse_hint() {
    // A hint in the first compact entry is currently rejected by root-flow
    // preflight. A later entry reaches the shared sequence value recognizer.
    let expected = "-\n  - warmup\n  - [first, second]\n  - inner_tail\n- outer_tail\n";
    for prefix in ["-\n  - warmup\n", "- - warmup\n"] {
        let input =
            format!("{prefix}  - [\n    - first\n    - second\n  - inner_tail\n- outer_tail\n");
        let document = parse(&input, DocumentKind::Yaml, FormatOptions::default());
        let source = document.source();
        let ast = document.yaml.as_ref().unwrap();
        let outer = sequence(ast, ast.roots[0].node.unwrap());
        let inner = sequence(ast, outer.items[0].value.unwrap());
        assert_eq!(inner.items.len(), 3);
        let hinted = sequence(ast, inner.items[1].value.unwrap());
        assert_eq!(hinted.items.len(), 2);
        assert_eq!(source.slice(hinted.flow_collapse_hint.unwrap()), "[");
        assert_eq!(
            source.slice(scalar(ast, hinted.items[0].value.unwrap()).value),
            "first"
        );
        assert_eq!(
            source.slice(scalar(ast, hinted.items[1].value.unwrap()).value),
            "second"
        );
        assert_eq!(
            format(&input, FileKind::Yaml, FormatOptions::default()),
            expected
        );
    }
}

#[test]
fn ordinary_and_compact_sequences_keep_distinct_first_item_trivia() {
    let ordinary = "# outer\n-\n  # inner\n  - one\n  # between\n  - two\n# outer tail\n- three\n";
    let compact = "# outer\n- - one\n  # between\n  - two\n# outer tail\n- three\n";
    for (input, expected_inner_leading) in [(ordinary, vec!["  # inner\n"]), (compact, Vec::new())]
    {
        let document = parse(input, DocumentKind::Yaml, FormatOptions::default());
        let source = document.source();
        let ast = document.yaml.as_ref().unwrap();
        let leading_text = |trivia: &[YamlTrivia]| {
            trivia
                .iter()
                .map(|trivia| source.slice(trivia.span))
                .collect::<Vec<_>>()
        };
        let outer_id = ast.roots[0].node.unwrap();
        let outer = sequence(ast, outer_id);
        let inner_id = outer.items[0].value.unwrap();
        let inner = sequence(ast, inner_id);

        assert_eq!(
            leading_text(&ast.node(outer_id).leading_trivia),
            ["# outer\n"]
        );
        assert_eq!(leading_text(&outer.items[0].leading_trivia), ["# outer\n"]);
        assert_eq!(
            leading_text(&ast.node(inner_id).leading_trivia),
            expected_inner_leading
        );
        assert_eq!(
            leading_text(&inner.items[0].leading_trivia),
            expected_inner_leading
        );
        assert_eq!(
            leading_text(&inner.items[1].leading_trivia),
            ["  # between\n"]
        );
        assert_eq!(
            leading_text(&outer.items[1].leading_trivia),
            ["# outer tail\n"]
        );
        let expected_start = if input == ordinary {
            inner.items[0].line.start()
        } else {
            inner.items[0].marker.start()
        };
        assert_eq!(ast.node(inner_id).span.start(), expected_start);
    }
}

#[test]
fn sequence_parent_properties_and_comments_have_separate_source_spans() {
    for prefix in ["-\n  - ", "- - "] {
        let input = format!(
            "{prefix}!note &saved # parent\n    'Café value'\n  - [one,two]\n- outer_tail\n"
        );
        let document = parse(&input, DocumentKind::Yaml, FormatOptions::default());
        let source = document.source();
        let ast = document.yaml.as_ref().unwrap();
        let outer = sequence(ast, ast.roots[0].node.unwrap());
        let inner = sequence(ast, outer.items[0].value.unwrap());
        let item = &inner.items[0];
        let value = scalar(ast, item.value.unwrap());

        assert_eq!(source.slice(value.tag.unwrap()), "!note");
        assert_eq!(source.slice(value.anchor.unwrap()), "&saved");
        assert_eq!(source.slice(item.trailing_comment.unwrap()), "# parent");
        assert_eq!(source.slice(value.value), "'Café value'");
        assert!(value.value.start() >= item.line.end());
        assert!(value.tag.unwrap().end() <= item.line.end());
        assert!(value.anchor.unwrap().end() <= item.line.end());
        assert!(value.trailing_comment.is_none());
    }
}

#[test]
fn empty_sequence_entries_stop_before_document_markers() {
    let input = "- - &empty\n  -\n--- # next\n[one,two]\n... # end\n";
    let document = parse(input, DocumentKind::Yaml, FormatOptions::default());
    let source = document.source();
    let ast = document.yaml.as_ref().unwrap();
    assert_eq!(ast.roots.len(), 2);
    let outer = sequence(ast, ast.roots[0].node.unwrap());
    let inner = sequence(ast, outer.items[0].value.unwrap());
    assert_eq!(inner.items.len(), 2);
    let anchored = scalar(ast, inner.items[0].value.unwrap());
    assert!(anchored.value.is_empty());
    assert_eq!(source.slice(anchored.anchor.unwrap()), "&empty");
    let empty = scalar(ast, inner.items[1].value.unwrap());
    assert!(empty.value.is_empty());
    assert_eq!(empty.semantic, YamlScalarSemantic::Null);
    assert_eq!(
        source.slice(ast.roots[1].start_marker.unwrap()),
        "--- # next\n"
    );
    assert_eq!(
        source.slice(ast.roots[1].end_marker.unwrap()),
        "... # end\n"
    );
}

#[test]
fn late_file_policy_preserves_parent_headers_and_explicitly_skipped_bytes() {
    let input = "\
- - !note &saved # parent
    'Café value'
  - [one,two]
# fmt: skip
- !markdown |
  Keep __raw__  bytes.  \n\
# fmt: markdown canonical=true scope=file
# fmt: compact scope=file
";
    let expected = "\
- - !note &saved # parent
    'Café value'
  - [one, two]
# fmt: skip
- !markdown |
  Keep __raw__  bytes.  \n\
# fmt: markdown canonical=true scope=file
# fmt: compact scope=file
";
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::None,
        ..FormatOptions::default()
    };
    let output = format(input, FileKind::Yaml, options);
    assert_eq!(output, expected);
    assert_eq!(format(&output, FileKind::Yaml, options), output);
}

#[test]
fn late_file_policies_reach_prior_scalar_and_nested_markdown() {
    let input = "\
template: << keep   spacing >>
body: !markdown |
  Keep __strong__ and `literal  spacing`.
items:
  - one
  - two
# fmt: template.delimiters \"<<\" \">>\" scope=file
# fmt: markdown canonical=true scope=file
# fmt: compact scope=file
";
    let expected = "\
template: << keep   spacing >>
body: !markdown |
  Keep **strong** and `literal  spacing`.
items: [one, two]
# fmt: template.delimiters \"<<\" \">>\" scope=file
# fmt: markdown canonical=true scope=file
# fmt: compact scope=file
";
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::None,
        ..FormatOptions::default()
    };
    let document = parse(input, DocumentKind::Yaml, options);
    let ast = document.yaml.as_ref().unwrap();
    let template = ast
        .nodes
        .iter()
        .find(|node| {
            matches!(&node.kind, YamlAstKind::Scalar(value)
            if document.source().slice(value.value) == "<< keep   spacing >>")
        })
        .unwrap();
    assert!(
        document
            .state(template.state)
            .template_delimiters
            .iter()
            .any(|delimiter| delimiter.open == "<<" && delimiter.close == ">>")
    );
    assert_eq!(
        emit_document(&document.finalize(options), &PluginRegistry::default()).unwrap(),
        expected
    );
    assert_eq!(format(input, FileKind::Yaml, options), expected);
    assert_eq!(format(expected, FileKind::Yaml, options), expected);
}

#[test]
fn outer_markdown_policy_reaches_yaml_inside_an_owned_markdown_scalar() {
    let input = "\
````yaml
outer: !markdown |
  ```yaml
  inner: !markdown |
    Keep __strong__ and `literal  bytes`.
  ```
````

<!-- fmt: canonical=true scope=file -->
";
    let expected = "\
````yaml
outer: !markdown |
  ```yaml
  inner: !markdown |
    Keep **strong** and `literal  bytes`.
  ```
````

<!-- fmt: canonical=true scope=file -->
";
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::None,
        ..FormatOptions::default()
    };
    let output = format(input, FileKind::Markdown, options);
    assert_eq!(output, expected);
    assert_eq!(format(&output, FileKind::Markdown, options), output);
}

#[test]
fn late_layout_policy_reuses_the_original_yaml_ingestion() {
    let input = "items:\n  - one\n  - two\n# fmt: compact scope=file\n";
    let options = FormatOptions::default();
    let report = format_source_report_with_trace(
        FileKind::Yaml,
        input.to_owned(),
        options,
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    assert_eq!(
        report.output,
        "items: [one, two]\n# fmt: compact scope=file\n"
    );
    assert_eq!(report.output, format(input, FileKind::Yaml, options));
    let trace = report.trace.unwrap();
    assert_eq!(trace.source_scans, 1);
    assert_eq!(trace.parse_passes, 1);
}

#[test]
fn invalid_same_line_directives_keep_original_sequence_coordinates() {
    for (input, line, column) in [
        ("- value # fmt: markdown\n", 1, 9),
        ("- - value # fmt: markdown\n", 1, 11),
        ("-\n  - value # fmt: markdown\n", 2, 11),
    ] {
        let error = parse_source(
            SourceBuffer::new(input.to_owned()),
            Span::new(0, input.len()),
            DocumentKind::Yaml,
            FormatOptions::default(),
            &Config::default(),
        )
        .unwrap_err();
        assert_eq!(
            error.diagnostic.message,
            "fmt directive is not supported in this position"
        );
        assert_eq!(error.diagnostic.line, line);
        assert_eq!(error.diagnostic.column, column);
    }
}
