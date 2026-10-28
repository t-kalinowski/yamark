//! Front-matter boundaries and supported Markdown options read from its YAML AST.

use crate::core::directives::DirectiveDelta;
use crate::core::document::{Document, MarkdownTableWidths, MarkdownWrap};
use crate::core::source::SourceBuffer;
use crate::core::yaml_model::{YamlAstKind, YamlDocumentAst, YamlNodeId, YamlScalar};

pub(super) fn front_matter_opening(text: &str) -> bool {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    front_matter_marker_line(text)
}

pub(super) fn find_front_matter_closing(
    source: &SourceBuffer,
    mut i: usize,
    end: usize,
) -> Option<usize> {
    while i < end {
        if front_matter_marker_line(source.line_text(i)) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn front_matter_marker_line(text: &str) -> bool {
    text.trim_end_matches([' ', '\t']) == "---"
}

pub(super) fn front_matter_markdown_delta(
    source: &SourceBuffer,
    document: &Document,
) -> Option<DirectiveDelta> {
    let ast = document.yaml.as_ref()?;
    if let Some(editor_options) =
        front_matter_markdown_delta_at_path(source, ast, &["editor_options", "markdown"])
    {
        return directive_delta_has_markdown_options(&editor_options).then_some(editor_options);
    }
    let delta = front_matter_markdown_delta_at_path(source, ast, &["editor", "markdown"])?;
    directive_delta_has_markdown_options(&delta).then_some(delta)
}

fn front_matter_markdown_delta_at_path(
    source: &SourceBuffer,
    ast: &YamlDocumentAst,
    path: &[&str],
) -> Option<DirectiveDelta> {
    let root = ast.roots.iter().find_map(|root| root.node)?;
    let node = yaml_mapping_value_at_path(source, ast, root, path)?;
    let mut delta = DirectiveDelta::default();
    for (key, value) in yaml_mapping_scalar_pairs(source, ast, node) {
        match key.as_str() {
            "wrap" => apply_front_matter_wrap(&mut delta, &value),
            "table-widths" => delta.markdown_table_widths = MarkdownTableWidths::parse(&value).ok(),
            "canonical" => delta.markdown_canonical = parse_front_matter_bool(&value),
            "footnotes" => {
                delta.markdown_format_footnotes = match value.as_str() {
                    "wrap" | "format" | "true" | "yes" | "1" => Some(true),
                    "preserve" | "none" | "false" | "no" | "0" => Some(false),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    Some(delta)
}

fn yaml_mapping_value_at_path(
    source: &SourceBuffer,
    ast: &YamlDocumentAst,
    mut node: YamlNodeId,
    path: &[&str],
) -> Option<YamlNodeId> {
    for part in path {
        node = yaml_mapping_value_for_key(source, ast, node, part)?;
    }
    Some(node)
}

fn yaml_mapping_value_for_key(
    source: &SourceBuffer,
    ast: &YamlDocumentAst,
    node: YamlNodeId,
    key: &str,
) -> Option<YamlNodeId> {
    match &ast.node(node).kind {
        YamlAstKind::Mapping(mapping) => {
            mapping
                .pairs
                .iter()
                .rev()
                .find(|pair| yaml_block_mapping_key(source, ast, pair).as_deref() == Some(key))?
                .value
        }
        YamlAstKind::FlowMapping(mapping) => {
            mapping
                .pairs
                .iter()
                .rev()
                .find(|pair| yaml_node_scalar_text(source, ast, pair.key).as_deref() == Some(key))?
                .value
        }
        _ => None,
    }
}

fn yaml_mapping_scalar_pairs(
    source: &SourceBuffer,
    ast: &YamlDocumentAst,
    node: YamlNodeId,
) -> Vec<(String, String)> {
    match &ast.node(node).kind {
        YamlAstKind::Mapping(mapping) => mapping
            .pairs
            .iter()
            .filter_map(|pair| {
                let key = yaml_block_mapping_key(source, ast, pair)?;
                let value = pair.value?;
                let value = yaml_node_scalar_text(source, ast, value)?;
                Some((key, value))
            })
            .collect(),
        YamlAstKind::FlowMapping(mapping) => mapping
            .pairs
            .iter()
            .filter_map(|pair| {
                let key = yaml_node_scalar_text(source, ast, pair.key)?;
                let value = pair.value?;
                let value = yaml_node_scalar_text(source, ast, value)?;
                Some((key, value))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn yaml_block_mapping_key<'a>(
    source: &'a SourceBuffer,
    ast: &'a YamlDocumentAst,
    pair: &crate::core::yaml_model::YamlMappingPair,
) -> Option<String> {
    if let Some(key) = pair.key_node {
        return yaml_node_scalar_text(source, ast, key);
    }
    Some(front_matter_unquoted_text(source.slice(pair.key).trim()).to_owned())
}

fn yaml_node_scalar_text(
    source: &SourceBuffer,
    ast: &YamlDocumentAst,
    node: YamlNodeId,
) -> Option<String> {
    let YamlAstKind::Scalar(scalar) = &ast.node(node).kind else {
        return None;
    };
    Some(front_matter_scalar_value(source, scalar))
}

fn front_matter_scalar_value(source: &SourceBuffer, scalar: &YamlScalar) -> String {
    let value = source.slice(scalar.value).trim();
    front_matter_unquoted_text(value).trim().to_owned()
}

fn front_matter_unquoted_text(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn apply_front_matter_wrap(delta: &mut DirectiveDelta, value: &str) {
    delta.markdown_wrap = MarkdownWrap::parse(value).ok();
}

fn parse_front_matter_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

fn directive_delta_has_markdown_options(delta: &DirectiveDelta) -> bool {
    delta.markdown_wrap.is_some()
        || delta.markdown_table_widths.is_some()
        || delta.markdown_canonical.is_some()
        || delta.markdown_format_footnotes.is_some()
}
