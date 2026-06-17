use std::path::Path;
use yamark::config::Config;
use yamark::core::document::{
    DocumentKind, EmitPlan, FileKind, FormatOptions, MarkdownNodeKind, NodeKind,
};
use yamark::core::emit::emit_document;
use yamark::core::parser::{format_source_report, parse_source};
use yamark::core::source::{SourceBuffer, SourceSpan, Span};
use yamark::plugins::PluginRegistry;
use yamark::plugins::{ExternalFormatter, ExternalFormatterMode};
use yamark::workspace::format_source_for_path;

#[test]
fn markdown_format_blocks_store_deferred_plans_after_file_scope_patches() {
    let input = "This is __strong__ text.\n\n<!-- fmt: canonical=true scope=file -->\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Markdown,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();

    let paragraph = document
        .nodes
        .iter()
        .find(|node| source.slice(node.span).starts_with("This is"))
        .unwrap();

    let EmitPlan::MarkdownParagraph = &paragraph.emit else {
        panic!("expected markdown paragraph plan, got {:?}", paragraph.emit);
    };
    let output = emit_document(
        &source,
        &document,
        FormatOptions::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    assert_eq!(
        output,
        "This is **strong** text.\n\n<!-- fmt: canonical=true scope=file -->\n"
    );
}

#[test]
fn markdown_format_blocks_store_specific_deferred_emit_variants() {
    let input = "\
This is __strong__ text.

| a |
| - |
| b |

- list item with several words

> quote with several words
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Markdown,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();

    let planned = document
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Markdown(MarkdownNodeKind::Paragraph) => {
                let EmitPlan::MarkdownParagraph = &node.emit else {
                    return None;
                };
                Some("paragraph")
            }
            NodeKind::Markdown(MarkdownNodeKind::GfmPipeTable) => {
                let EmitPlan::MarkdownTable = &node.emit else {
                    return None;
                };
                Some("table")
            }
            NodeKind::Markdown(MarkdownNodeKind::List) => {
                let EmitPlan::MarkdownList = &node.emit else {
                    return None;
                };
                Some("list")
            }
            NodeKind::Markdown(MarkdownNodeKind::Blockquote) => {
                let EmitPlan::MarkdownBlockquote = &node.emit else {
                    return None;
                };
                Some("blockquote")
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(planned, vec!["paragraph", "table", "list", "blockquote"]);

    let output = emit_document(
        &source,
        &document,
        FormatOptions::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    assert_eq!(
        output,
        "\
This is __strong__ text.

| a   |
| --- |
| b   |

- list item with several words

> quote with several words
"
    );
}

#[test]
fn embedded_markdown_string_emit_consumes_nested_document_plan() {
    let input = "# fmt: markdown\nDOC = \"\"\"\n#   Title ##\n\"\"\"\n";
    let source = SourceBuffer::new(input.to_owned());
    let mut document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Python,
        FormatOptions {
            markdown_wrap: yamark::core::document::MarkdownWrap::None,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();

    let nested = document
        .nodes
        .iter()
        .find_map(|node| match node.emit {
            EmitPlan::EmbeddedMarkdownString { nested, .. } => Some(nested),
            _ => None,
        })
        .unwrap();
    document.nested[nested].skip_file = true;

    let output = emit_document(
        &source,
        &document,
        FormatOptions::default(),
        &PluginRegistry::default(),
    )
    .unwrap();

    assert_eq!(output, input);
}

#[test]
fn embedded_source_fragments_are_source_lifetime_spans() {
    fn fragment<'src>(source: &'src SourceBuffer, span: SourceSpan<'src>) -> &'src str {
        span.as_str(source)
    }

    let input = "# fmt: markdown\nDOC = \"\"\"\n  #   Title ##\n  \"\"\"\n";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Python,
        FormatOptions {
            markdown_wrap: yamark::core::document::MarkdownWrap::None,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();

    let (indent, closing_indent) = document
        .nodes
        .iter()
        .find_map(|node| match &node.emit {
            EmitPlan::EmbeddedMarkdownString {
                indent,
                closing_indent,
                ..
            } => Some((*indent, *closing_indent)),
            _ => None,
        })
        .unwrap();

    assert_eq!(fragment(&source, indent), "  ");
    assert_eq!(fragment(&source, closing_indent), "  ");
}

#[test]
fn workspace_format_source_for_path_consumes_owned_input() {
    let input = "items: [one, two]\n".to_owned();
    let formatted = format_source_for_path(
        Path::new("input.yaml"),
        input,
        FormatOptions {
            line_width: 12,
            ..FormatOptions::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(formatted.output, "items:\n  - one\n  - two\n");
    assert!(formatted.changed);
}

#[test]
fn source_file_scope_markdown_patch_does_not_reparse_prior_targets() {
    let input = "\
# fmt: markdown
# This is __before__.
x = 1
# fmt: markdown canonical=true scope=file
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Python,
        FormatOptions {
            markdown_wrap: yamark::core::document::MarkdownWrap::None,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();

    assert_eq!(document.nested.len(), 1);

    let output = emit_document(
        &source,
        &document,
        FormatOptions::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    assert_eq!(
        output,
        "\
# fmt: markdown
# This is **before**.
x = 1
# fmt: markdown canonical=true scope=file
"
    );
}

#[test]
fn markdown_scanner_records_specific_supported_block_kinds() {
    let input = "\
Setext title
============

| a |
| - |
| b |

Term
: Definition

[^note]: Footnote body.
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Markdown,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();

    let kinds = document
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Markdown(kind) if *kind != MarkdownNodeKind::Blank => Some(kind.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        vec![
            MarkdownNodeKind::SetextHeading,
            MarkdownNodeKind::GfmPipeTable,
            MarkdownNodeKind::DefinitionList,
            MarkdownNodeKind::FootnoteDefinition,
        ]
    );
}

#[test]
fn markdown_scanner_records_ordinary_html_comments_distinctly() {
    let input = "\
<!-- ordinary
html comment -->

Paragraph.
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Markdown,
        FormatOptions::default(),
        &Config::default(),
    )
    .unwrap();

    let kinds = document
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Markdown(kind) if *kind != MarkdownNodeKind::Blank => Some(kind.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        vec![MarkdownNodeKind::HtmlComment, MarkdownNodeKind::Paragraph]
    );
}

#[test]
fn markdown_scanner_preserves_mixed_case_paired_html_blocks_only_until_closing_tag() {
    let input = "\
<Section>
Keep    this raw.
</Section>

This paragraph should wrap to sentence. This one too.
";
    let source = SourceBuffer::new(input.to_owned());
    let document = parse_source(
        &source,
        Span::new(0, input.len()),
        DocumentKind::Markdown,
        FormatOptions {
            markdown_wrap: yamark::core::document::MarkdownWrap::Sentence,
            ..FormatOptions::default()
        },
        &Config::default(),
    )
    .unwrap();

    let kinds = document
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Markdown(kind) if *kind != MarkdownNodeKind::Blank => Some(kind.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![MarkdownNodeKind::Raw, MarkdownNodeKind::Paragraph]
    );

    let output = emit_document(
        &source,
        &document,
        FormatOptions::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    assert_eq!(
        output,
        "\
<Section>
Keep    this raw.
</Section>

This paragraph should wrap to sentence.
This one too.
"
    );
}

#[test]
fn markdown_code_fences_infer_quarto_and_myst_executable_languages() {
    let input = "\
```{python echo=false}
x=1
```

```{r}
x=1
```

``` {ojs}
viewof x = Inputs.text()
```

```{code-cell} python
:tags: [hide-input]
x=1
```

``` {#cell .python lst-cap=\"Cell\"}
x=1
```

```{code-cell} ipython3
x=1
```
";
    let mut config = Config::default();
    config.embedded_formatters.insert(
        "python".to_owned(),
        ExternalFormatter {
            command: vec![
                "python3".to_owned(),
                "-c".to_owned(),
                "import sys; sys.stdout.write(sys.stdin.read().replace('x=1', 'x = 1'))".to_owned(),
            ],
            path_suffix: ".py".to_owned(),
            mode: ExternalFormatterMode::Raw,
        },
    );
    config.embedded_formatters.insert(
        "r".to_owned(),
        ExternalFormatter {
            command: vec![
                "python3".to_owned(),
                "-c".to_owned(),
                "import sys; sys.stdout.write(sys.stdin.read().replace('x=1', 'x = 1'))".to_owned(),
            ],
            path_suffix: ".R".to_owned(),
            mode: ExternalFormatterMode::Raw,
        },
    );
    let plugins = PluginRegistry::from_config(&config);

    let output = format_source_report(
        FileKind::Markdown,
        input.to_owned(),
        FormatOptions::default(),
        &config,
        &plugins,
    )
    .unwrap()
    .output;

    assert_eq!(
        output,
        "\
```{python echo=false}
x = 1
```

```{r}
x = 1
```

```{ojs}
viewof x = Inputs.text()
```

```{code-cell} python
:tags: [hide-input]
x = 1
```

```{#cell .python lst-cap=\"Cell\"}
x = 1
```

```{code-cell} ipython3
x = 1
```
"
    );
}
