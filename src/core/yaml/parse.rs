//! Block mappings, sequences, scalar nodes, and source-preserving parse boundaries.

use super::YamlParser;
use super::block_scalar::yaml_tag_is_markdown;
use super::flow_emit::source_may_contain_template_span;
use super::flow_parse::{parse_flow_collection, recognize_flow_collection};
use super::scalar::scalar_semantic_with_tag;
use super::syntax::{
    BlockScalar, FlowCollapseHint, FlowCollapseRest, FlowCollectionBlock, FlowCollectionScan,
    PlainScalarContinuation, PlainScalarExtension, QuotedScalarBlock, alias_scalar,
    block_scalar_at, block_scalar_at_value, block_scalar_style, compact_nested_document_index,
    compact_sequence_marker_at, document_marker_line_info, explicit_indicator_at,
    explicit_key_line, explicit_value_line, flow_collapse_hint_from_standalone_opener,
    flow_collapse_hint_from_value, flow_collection_block_from_value,
    flow_collection_directive_target, indentation, line_has_flow_collapse_hint_value,
    mapping_colon_at, mapping_colon_from, plain_scalar_continuation_line,
    quoted_scalar_block_from_value, scalar_content_flow_collection_target, scalar_metadata,
    scalar_style, sequence_line, skip_ascii_whitespace, split_line_value_comment,
    tab_in_indentation, tab_indented_block_at, trim_span_ascii, unsupported_flow_block_at,
    yaml_error_at, yaml_scan_has_active_fmt_directive,
};
use crate::config::Config;
use crate::core::directives::{DirectiveState, DirectiveTargetKind, StateId};
use crate::core::source::{SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAlias, YamlAstKind, YamlAstNode, YamlMapping, YamlMappingPair, YamlNodeId, YamlOpaque,
    YamlOpaqueReason, YamlScalar, YamlScalarSemantic, YamlScalarStyle, YamlSequence,
    YamlSequenceItem, YamlTrivia, YamlTriviaKind,
};
use crate::core::yaml_scan::scan_yaml_lines;
use crate::diagnostic::{Result, YamarkError};

fn trivia_has_document_marker(trivia: &[YamlTrivia]) -> bool {
    trivia
        .iter()
        .any(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)
}

pub(super) fn config_for_directive_state(config: &Config, state: &DirectiveState) -> Config {
    let mut config = config.clone();
    config.template_delimiters = state.template_delimiters.clone();
    config
}

impl<'src, 'cfg> YamlParser<'src, 'cfg> {
    pub(super) fn parse_block(
        &mut self,
        indent: usize,
        leading: Vec<YamlTrivia>,
        container_is_target: bool,
    ) -> Result<Option<YamlNodeId>> {
        if self.line >= self.end {
            self.unread_trivia(leading);
            return Ok(None);
        }

        if let Some(content_start) = self.document_marker_inline_content_start(&leading) {
            return self
                .parse_document_marker_inline_content(leading, content_start)
                .map(Some);
        }

        let text = self.source.line_text(self.line);
        if tab_in_indentation(text).is_some() {
            return self.parse_tab_indented_opaque(leading).map(Some);
        }
        let actual_indent = indentation(text);
        if actual_indent < indent {
            self.unread_trivia(leading);
            return Ok(None);
        }

        if text.starts_with('\u{feff}') {
            // A line-leading U+FEFF can be parsed as scalar or key content. Treating
            // its UTF-8 bytes as indentation can drop or reinterpret that content.
            return self.parse_bom_prefixed_opaque(leading).map(Some);
        }

        if let Some(hint) =
            flow_collapse_hint_from_standalone_opener(self.source, self.line, self.end, indent)
        {
            return self
                .parse_flow_collapse_hint_collection(indent, leading, hint, container_is_target)
                .map(Some);
        }

        if !line_has_flow_collapse_hint_value(self.source, self.line, self.end) {
            self.recognized_flow = recognize_flow_collection(self.source, self.line, self.end);
            if self.recognized_flow.is_none()
                && let Some(block) = unsupported_flow_block_at(self.source, self.line, self.end)
            {
                return self.parse_opaque_flow(block, leading);
            }
        }

        if sequence_line(text, actual_indent).is_some() {
            return self.parse_sequence(actual_indent, leading).map(Some);
        }

        if explicit_key_line(text, actual_indent).is_some() {
            return self
                .parse_explicit_mapping(actual_indent, leading, container_is_target)
                .map(Some);
        }

        if mapping_colon_at(text, actual_indent).is_some() {
            return self
                .parse_mapping(actual_indent, leading, container_is_target)
                .map(Some);
        }

        if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
            return self.parse_block_scalar(block).map(Some);
        }

        if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            actual_indent,
            self.source.lines[self.line].full.start(),
        )
        .and_then(FlowCollectionScan::complete)
        {
            return self.parse_flow_collection_block(block, leading).map(Some);
        }

        self.parse_plain_scalar_line(leading).map(Some)
    }

    fn document_marker_inline_content_start(&self, leading: &[YamlTrivia]) -> Option<usize> {
        let marker_trivia = leading
            .iter()
            .rev()
            .find(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)?;
        let line = self.source.lines.get(self.line)?;
        if marker_trivia.span.start() != line.full.start() {
            return None;
        }
        let marker = document_marker_line_info(self.source.line_text(self.line))?;
        let content_start = marker.inline_content_start?;
        (marker_trivia.span.end() == line.text.start() + content_start).then_some(content_start)
    }

    fn parse_document_marker_inline_content(
        &mut self,
        leading: Vec<YamlTrivia>,
        content_start: usize,
    ) -> Result<YamlNodeId> {
        if let Some(scan) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            content_start,
            self.source.lines[self.line].text.start() + content_start,
        ) {
            match scan {
                FlowCollectionScan::Complete(block) => {
                    return self.parse_flow_collection_block(block, leading);
                }
                FlowCollectionScan::Incomplete => {
                    let line = self.source.lines[self.line];
                    return self.parse_opaque_flow_node(
                        Span::new(line.text.start() + content_start, line.full.end()),
                        leading,
                    );
                }
            }
        }

        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let line_value = split_line_value_comment(text, content_start, line.text.start());
        self.reject_populated_same_line_yaml_directive(line_value)?;
        self.line += 1;
        let id = self.push_scalar_line_with_comment(
            Span::new(line.text.start() + content_start, line.full.end()),
            line_value.value,
            line_value.trailing_comment,
            PlainScalarContinuation::None,
        )?;
        self.ast.node_mut(id).leading_trivia = leading.into();
        Ok(id)
    }

    pub(super) fn parse_mapping(
        &mut self,
        indent: usize,
        first_leading: Vec<YamlTrivia>,
        container_is_target: bool,
    ) -> Result<YamlNodeId> {
        let state = if container_is_target {
            self.engine
                .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping)
        } else {
            self.engine.state_for_node(&mut self.doc, false)
        };
        let mut pairs = Vec::new();
        let node_leading = first_leading.clone();
        let mut leading = first_leading;
        let mut start = self.source.lines[self.line].full.start();
        let mut end = start;

        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                self.unread_trivia(leading);
                break;
            }
            let actual_indent = indentation(text);
            let explicit = explicit_key_line(text, indent).is_some();
            let implicit = mapping_colon_at(text, indent).is_some();
            if actual_indent != indent || (!explicit && !implicit) {
                self.unread_trivia(leading);
                break;
            }

            if pairs.is_empty() && leading.is_empty() {
                start = line.full.start();
            }

            let (pair, pair_end) = if explicit {
                self.parse_explicit_mapping_pair_at(indent, std::mem::take(&mut leading))?
            } else {
                self.parse_mapping_pair_at(indent, std::mem::take(&mut leading))?
            };
            end = pair_end;
            pairs.push(pair);

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading) || self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if tab_in_indentation(self.source.line_text(self.line)).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            node_leading,
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(id)
    }

    fn parse_explicit_mapping(
        &mut self,
        indent: usize,
        first_leading: Vec<YamlTrivia>,
        container_is_target: bool,
    ) -> Result<YamlNodeId> {
        let state = if container_is_target {
            self.engine
                .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping)
        } else {
            self.engine.state_for_node(&mut self.doc, false)
        };
        let node_leading = first_leading.clone();
        let mut leading = first_leading;
        let start = self.source.lines[self.line].full.start();
        let mut end = start;
        let mut pairs = Vec::new();

        while self.line < self.end {
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                self.unread_trivia(leading);
                break;
            }
            let explicit = explicit_key_line(text, indent).is_some();
            let implicit = mapping_colon_at(text, indent).is_some();
            if indentation(text) != indent || (!explicit && !implicit) {
                self.unread_trivia(leading);
                break;
            }

            let (pair, pair_end) = if explicit {
                self.parse_explicit_mapping_pair_at(indent, std::mem::take(&mut leading))?
            } else {
                self.parse_mapping_pair_at(indent, std::mem::take(&mut leading))?
            };
            end = pair_end;
            pairs.push(pair);

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading) || self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if tab_in_indentation(self.source.line_text(self.line)).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            node_leading,
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(id)
    }

    fn parse_explicit_mapping_pair_at(
        &mut self,
        indent: usize,
        leading: Vec<YamlTrivia>,
    ) -> Result<(YamlMappingPair, usize)> {
        let key_line = self.source.lines[self.line];
        let key_text = self.source.line_text(self.line);
        let marker =
            explicit_indicator_at(key_text, indent, b'?').expect("explicit mapping key exists");
        let key_start = skip_ascii_whitespace(key_text, marker + 1);
        let (key, key_node) = self.parse_explicit_key_node(indent, key_start)?;
        let mut between_trivia = self.take_leading_trivia()?;
        if self.line >= self.end {
            self.unread_trivia(std::mem::take(&mut between_trivia));
            let pair_end = key_node
                .map(|id| self.ast.node(id).span.end())
                .unwrap_or(key_line.full.end())
                .max(key_line.full.end());
            let value = self.push_empty_scalar(Span::empty(pair_end), pair_end)?;
            return Ok((
                YamlMappingPair {
                    leading_trivia: leading.into(),
                    key: SourceSpan::new(key),
                    key_node,
                    colon: SourceSpan::empty(pair_end),
                    line: SourceSpan::new(key_line.full.into()),
                    source: SourceSpan::new(Span::new(key_line.full.start(), pair_end)),
                    explicit: true,
                    trailing_comment: None,
                    value: Some(value),
                },
                pair_end,
            ));
        }

        let value_line = self.source.lines[self.line];
        let value_text = self.source.line_text(self.line);
        let Some(colon) = explicit_value_line(value_text, indent) else {
            self.unread_trivia(std::mem::take(&mut between_trivia));
            let pair_end = key_node
                .map(|id| self.ast.node(id).span.end())
                .unwrap_or(key_line.full.end())
                .max(key_line.full.end());
            let value = self.push_empty_scalar(Span::empty(pair_end), pair_end)?;
            return Ok((
                YamlMappingPair {
                    leading_trivia: leading.into(),
                    key: SourceSpan::new(key),
                    key_node,
                    colon: SourceSpan::empty(pair_end),
                    line: SourceSpan::new(key_line.full.into()),
                    source: SourceSpan::new(Span::new(key_line.full.start(), pair_end)),
                    explicit: true,
                    trailing_comment: None,
                    value: Some(value),
                },
                pair_end,
            ));
        };
        let colon_span = Span::new(
            value_line.text.start() + colon,
            value_line.text.start() + colon + 1,
        );
        let value_start = skip_ascii_whitespace(value_text, colon + 1);
        let line_value = split_line_value_comment(value_text, value_start, value_line.text.start());
        self.reject_populated_same_line_yaml_directive(line_value)?;

        let mut pair_trailing_comment = None;
        let end;
        let value = if let Some(block) = quoted_scalar_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            value_line.full.start(),
        ) {
            let value = self.parse_quoted_scalar_block(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            value_line.full.start(),
        )
        .and_then(FlowCollectionScan::complete)
        {
            let value = self.parse_flow_collection_block(block, Vec::new())?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(hint) =
            flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
        {
            let value =
                self.parse_flow_collapse_hint_collection(indent, Vec::new(), hint, false)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) =
            block_scalar_at_value(self.source, self.line, self.end, value_start, indent)
        {
            let value = self.parse_block_scalar(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if compact_sequence_marker_at(value_text, value_start).is_some() {
            let value = self.parse_compact_sequence(value_start)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(value) = self.parse_sequence_item_inline_mapping(indent, value_start)? {
            end = self.ast.node(value).span.end();
            Some(value)
        } else {
            let metadata = scalar_metadata(self.source, line_value.value);
            let value_is_empty = self.source.slice(line_value.value).trim_ascii().is_empty();
            let property_only_value = !value_is_empty
                && (metadata.tag.is_some() || metadata.anchor.is_some())
                && self.source.slice(metadata.content).trim_ascii().is_empty();
            self.line += 1;
            if value_is_empty || property_only_value {
                pair_trailing_comment = line_value.trailing_comment;
                let inline_directive =
                    self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                let nested_leading = self.take_leading_trivia()?;
                let nested_is_target = inline_directive
                    || leading
                        .iter()
                        .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                let collapse_hint = nested_leading.is_empty().then(|| {
                    flow_collapse_hint_from_standalone_opener(
                        self.source,
                        self.line,
                        self.end,
                        indent,
                    )
                });
                let indentless_sequence = self.line < self.end
                    && sequence_line(self.source.line_text(self.line), indent).is_some();
                if let Some(Some(hint)) = collapse_hint {
                    let nested = self.parse_flow_collapse_hint_collection(
                        indent,
                        nested_leading,
                        hint,
                        nested_is_target,
                    )?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.next_line_has_tab_indentation() {
                    let nested = self.parse_tab_indented_opaque(nested_leading)?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.line < self.end
                    && (indentation(self.source.line_text(self.line)) > indent
                        || indentless_sequence)
                {
                    let nested_indent = if indentless_sequence {
                        indent
                    } else {
                        indent + 1
                    };
                    let nested =
                        self.parse_block(nested_indent, nested_leading, nested_is_target)?;
                    if let Some(nested) = nested {
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                    } else {
                        end = value_line.full.end();
                    }
                    nested
                } else {
                    self.unread_trivia(nested_leading);
                    if property_only_value {
                        let scalar = self.push_scalar_line_with_comment(
                            value_line.full.into(),
                            line_value.value,
                            line_value.trailing_comment,
                            PlainScalarContinuation::None,
                        )?;
                        end = self.ast.node(scalar).span.end();
                        Some(scalar)
                    } else {
                        let scalar =
                            self.push_empty_scalar(value_line.full.into(), value_line.text.end())?;
                        end = value_line.full.end();
                        Some(scalar)
                    }
                }
            } else {
                let scalar = self.push_scalar_line_with_comment(
                    value_line.full.into(),
                    line_value.value,
                    line_value.trailing_comment,
                    PlainScalarContinuation::Inline {
                        parent_indent: indent,
                    },
                )?;
                end = self.ast.node(scalar).span.end();
                Some(scalar)
            }
        };

        Ok((
            YamlMappingPair {
                leading_trivia: leading.into(),
                key: SourceSpan::new(key),
                key_node,
                colon: SourceSpan::new(colon_span),
                line: SourceSpan::new(value_line.full.into()),
                source: SourceSpan::new(Span::new(key_line.full.start(), end)),
                explicit: true,
                trailing_comment: pair_trailing_comment.map(SourceSpan::new),
                value,
            },
            end,
        ))
    }

    fn parse_explicit_key_node(
        &mut self,
        indent: usize,
        key_start: usize,
    ) -> Result<(Span, Option<YamlNodeId>)> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let key_value = split_line_value_comment(text, key_start, line.text.start());
        self.reject_same_line_yaml_directive(key_value.trailing_comment)?;
        let key = trim_span_ascii(self.source, key_value.value);

        if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
            let key_node = self.parse_block_scalar(block)?;
            self.validate_non_value_directive_target(key_node)?;
            let key = self.ast.node(key_node).span.span();
            return Ok((key, Some(key_node)));
        }

        if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            key_start,
            line.text.start() + key_start,
        )
        .and_then(FlowCollectionScan::complete)
        {
            let key_node = self.parse_flow_collection_block(block, Vec::new())?;
            let key = match &self.ast.node(key_node).kind {
                YamlAstKind::FlowSequence(sequence) => sequence.value.span(),
                YamlAstKind::FlowMapping(mapping) => mapping.value.span(),
                _ => self.ast.node(key_node).span.span(),
            };
            return Ok((key, Some(key_node)));
        }

        if compact_sequence_marker_at(text, key_start).is_some() {
            let key_node = self.parse_compact_sequence(key_start)?;
            let key = self.ast.node(key_node).span.span();
            return Ok((key, Some(key_node)));
        }

        if mapping_colon_from(text, key_start).is_some() {
            let key_node = self
                .parse_sequence_item_inline_mapping(indent, key_start)?
                .expect("mapping colon produced a mapping key node");
            let key = self.ast.node(key_node).span.span();
            return Ok((key, Some(key_node)));
        }

        if !key.is_empty() {
            self.line += 1;
            return Ok((key, self.push_mapping_key_node(key)));
        }

        self.line += 1;
        let nested_leading = self.take_leading_trivia()?;
        if self.line < self.end && indentation(self.source.line_text(self.line)) > indent {
            let key_node = self.parse_block(indent + 1, nested_leading, false)?;
            if let Some(key_node) = key_node {
                let key = self.ast.node(key_node).span.span();
                return Ok((key, Some(key_node)));
            }
        } else {
            self.unread_trivia(nested_leading);
        }

        Ok((key, None))
    }

    fn parse_mapping_pair_at(
        &mut self,
        key_start_column: usize,
        leading: Vec<YamlTrivia>,
    ) -> Result<(YamlMappingPair, usize)> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let colon = mapping_colon_from(text, key_start_column).expect("mapping pair exists");
        let key_start = line.text.start() + key_start_column;
        let key_end = line.text.start()
            + key_start_column
            + text[key_start_column..colon].trim_ascii_end().len();
        let key = Span::new(key_start, key_end);
        let key_node = self.push_mapping_key_node(key);
        let colon_span = Span::new(line.text.start() + colon, line.text.start() + colon + 1);
        let value_start = colon
            + 1
            + text[colon + 1..]
                .bytes()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
        let line_value = split_line_value_comment(text, value_start, line.text.start());
        self.reject_populated_same_line_yaml_directive(line_value)?;

        let mut pair_trailing_comment = None;
        let end;
        let value = if let Some(block) = quoted_scalar_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            line.full.start(),
        ) {
            let value = self.parse_quoted_scalar_block(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            line.full.start(),
        )
        .and_then(FlowCollectionScan::complete)
        {
            let value = self.parse_flow_collection_block(block, Vec::new())?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(hint) =
            flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
        {
            let value = self.parse_flow_collapse_hint_collection(
                key_start_column,
                Vec::new(),
                hint,
                false,
            )?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = block_scalar_at_value(
            self.source,
            self.line,
            self.end,
            value_start,
            key_start_column,
        ) {
            let value = self.parse_block_scalar(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else {
            let metadata = scalar_metadata(self.source, line_value.value);
            let value_is_empty = self.source.slice(line_value.value).trim_ascii().is_empty();
            let property_only_value = !value_is_empty
                && (metadata.tag.is_some() || metadata.anchor.is_some())
                && self.source.slice(metadata.content).trim_ascii().is_empty();
            self.line += 1;
            if value_is_empty || property_only_value {
                pair_trailing_comment = line_value.trailing_comment;
                let inline_directive =
                    self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                let nested_leading = self.take_leading_trivia()?;
                let nested_is_target = inline_directive
                    || leading
                        .iter()
                        .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                let collapse_hint = nested_leading.is_empty().then(|| {
                    flow_collapse_hint_from_standalone_opener(
                        self.source,
                        self.line,
                        self.end,
                        key_start_column,
                    )
                });
                let indentless_sequence = self.line < self.end
                    && sequence_line(self.source.line_text(self.line), key_start_column).is_some();
                if let Some(Some(hint)) = collapse_hint {
                    let nested = self.parse_flow_collapse_hint_collection(
                        key_start_column,
                        nested_leading,
                        hint,
                        nested_is_target,
                    )?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.next_line_has_tab_indentation() {
                    let nested = self.parse_tab_indented_opaque(nested_leading)?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.line < self.end
                    && (indentation(self.source.line_text(self.line)) > key_start_column
                        || indentless_sequence)
                {
                    let nested_indent = if indentless_sequence {
                        key_start_column
                    } else {
                        key_start_column + 1
                    };
                    let nested =
                        self.parse_block(nested_indent, nested_leading, nested_is_target)?;
                    if let Some(nested) = nested {
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                    } else {
                        end = line.full.end();
                    }
                    nested
                } else {
                    self.unread_trivia(nested_leading);
                    if property_only_value {
                        let scalar = self.push_scalar_line_with_comment(
                            line.full.into(),
                            line_value.value,
                            line_value.trailing_comment,
                            PlainScalarContinuation::None,
                        )?;
                        end = self.ast.node(scalar).span.end();
                        Some(scalar)
                    } else {
                        let scalar = self.push_empty_scalar(line.full.into(), line.text.end())?;
                        end = line.full.end();
                        Some(scalar)
                    }
                }
            } else {
                let scalar = self.push_scalar_line_with_comment(
                    line.full.into(),
                    line_value.value,
                    line_value.trailing_comment,
                    PlainScalarContinuation::Inline {
                        parent_indent: key_start_column,
                    },
                )?;
                end = self.ast.node(scalar).span.end();
                Some(scalar)
            }
        };

        Ok((
            YamlMappingPair {
                leading_trivia: leading.into(),
                key: SourceSpan::new(key),
                key_node,
                colon: SourceSpan::new(colon_span),
                line: SourceSpan::new(line.full.into()),
                source: SourceSpan::new(Span::new(line.full.start(), end)),
                explicit: false,
                trailing_comment: pair_trailing_comment.map(SourceSpan::new),
                value,
            },
            end,
        ))
    }

    fn attach_collection_properties(
        &mut self,
        node: YamlNodeId,
        tag: Option<Span>,
        anchor: Option<Span>,
    ) {
        let tag = tag.map(SourceSpan::new);
        let anchor = anchor.map(SourceSpan::new);
        match &mut self.ast.node_mut(node).kind {
            YamlAstKind::Mapping(mapping) => {
                mapping.tag = tag;
                mapping.anchor = anchor;
            }
            YamlAstKind::Sequence(sequence) => {
                sequence.tag = tag;
                sequence.anchor = anchor;
            }
            YamlAstKind::FlowMapping(mapping) => {
                mapping.tag = tag;
                mapping.anchor = anchor;
            }
            YamlAstKind::FlowSequence(sequence) => {
                sequence.tag = tag;
                sequence.anchor = anchor;
            }
            _ => {}
        }
    }

    fn push_mapping_key_node(&mut self, key: Span) -> Option<YamlNodeId> {
        let key = trim_span_ascii(self.source, key);
        if key.is_empty() {
            return None;
        }

        let state = self.engine.state_for_node(&mut self.doc, false);
        let metadata = scalar_metadata(self.source, key);
        let template_spans_possible = self.template_spans_possible_for_state(state);
        if let Some(id) = parse_flow_collection(
            self.source,
            &mut self.ast,
            &mut self.flow_collection_nodes,
            state,
            &self.doc.state(state).template_delimiters,
            template_spans_possible,
            key,
            key,
            metadata.content,
            None,
        ) {
            self.plan_parsed_yaml_flow_nodes();
            return Some(id);
        }

        let source = self.source.slice(metadata.content).trim_ascii();
        if alias_scalar(source) {
            return Some(self.ast.push_node(YamlAstNode::semantic(
                YamlAstKind::Alias(YamlAlias {
                    value: SourceSpan::new(metadata.content),
                    trailing_comment: None,
                }),
                key,
                Vec::new(),
                state,
            )));
        }

        Some(self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: scalar_style(source),
                semantic: scalar_semantic_with_tag(
                    source,
                    metadata.tag.map(|tag| self.source.slice(tag)),
                ),
                value: SourceSpan::new(key),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: None,
            }),
            key,
            Vec::new(),
            state,
        )))
    }

    pub(super) fn parse_sequence(
        &mut self,
        indent: usize,
        first_leading: Vec<YamlTrivia>,
    ) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlSequence);
        let mut items = Vec::new();
        let node_leading = first_leading.clone();
        let mut leading = first_leading;
        let start = self.source.lines[self.line].full.start();
        let mut end = start;

        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                self.unread_trivia(leading);
                break;
            }
            let actual_indent = indentation(text);
            let Some(marker_local) = sequence_line(text, indent) else {
                self.unread_trivia(leading);
                break;
            };
            if actual_indent != indent {
                self.unread_trivia(leading);
                break;
            }

            let marker = Span::new(
                line.text.start() + marker_local,
                line.text.start() + marker_local + 1,
            );
            let value_start = marker_local
                + 1
                + text[marker_local + 1..]
                    .bytes()
                    .take_while(|byte| byte.is_ascii_whitespace())
                    .count();
            let line_value = split_line_value_comment(text, value_start, line.text.start());
            self.reject_populated_same_line_yaml_directive(line_value)?;

            let mut item_trailing_comment = None;
            let mut value_on_marker_line = false;
            let value = if let Some(block) = quoted_scalar_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            ) {
                let value = self.parse_quoted_scalar_block(block)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) = flow_collection_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            )
            .and_then(FlowCollectionScan::complete)
            {
                let value = self.parse_flow_collection_block(block, Vec::new())?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(hint) =
                flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
            {
                let value =
                    self.parse_flow_collapse_hint_collection(indent, Vec::new(), hint, false)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) =
                block_scalar_at_value(self.source, self.line, self.end, value_start, indent)
            {
                let value = self.parse_block_scalar(block)?;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_explicit_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if compact_sequence_marker_at(text, value_start).is_some() {
                let value = self.parse_compact_sequence(value_start)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_inline_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else {
                let metadata = scalar_metadata(self.source, line_value.value);
                let value_is_empty = self.source.slice(line_value.value).trim_ascii().is_empty();
                let property_only_value = !value_is_empty
                    && (metadata.tag.is_some() || metadata.anchor.is_some())
                    && self.source.slice(metadata.content).trim_ascii().is_empty();
                self.line += 1;
                if value_is_empty || property_only_value {
                    item_trailing_comment = line_value.trailing_comment;
                    let inline_directive =
                        self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                    let nested_leading = self.take_leading_trivia()?;
                    let nested_is_target = inline_directive
                        || leading
                            .iter()
                            .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                    if self.next_line_has_tab_indentation() {
                        let nested = self.parse_tab_indented_opaque(nested_leading)?;
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                        Some(nested)
                    } else if self.line < self.end
                        && indentation(self.source.line_text(self.line)) > indent
                    {
                        let nested =
                            self.parse_block(indent + 1, nested_leading, nested_is_target)?;
                        if let Some(nested) = nested {
                            if property_only_value {
                                self.attach_collection_properties(
                                    nested,
                                    metadata.tag,
                                    metadata.anchor,
                                );
                            }
                            end = self.ast.node(nested).span.end();
                        } else {
                            end = line.full.end();
                        }
                        nested
                    } else {
                        self.unread_trivia(nested_leading);
                        if property_only_value {
                            let scalar = self.push_scalar_line_with_comment(
                                line.full.into(),
                                line_value.value,
                                line_value.trailing_comment,
                                PlainScalarContinuation::None,
                            )?;
                            end = self.ast.node(scalar).span.end();
                            Some(scalar)
                        } else {
                            let scalar =
                                self.push_empty_scalar(line.full.into(), line.text.end())?;
                            end = line.full.end();
                            Some(scalar)
                        }
                    }
                } else {
                    let scalar = self.push_scalar_line_with_comment(
                        line.full.into(),
                        line_value.value,
                        line_value.trailing_comment,
                        PlainScalarContinuation::Inline {
                            parent_indent: indent,
                        },
                    )?;
                    end = self.ast.node(scalar).span.end();
                    Some(scalar)
                }
            };

            items.push(YamlSequenceItem {
                leading_trivia: std::mem::take(&mut leading).into(),
                marker: SourceSpan::new(marker),
                line: SourceSpan::new(line.full.into()),
                value_on_marker_line,
                trailing_comment: item_trailing_comment.map(SourceSpan::new),
                value,
            });

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading)
                || leading
                    .iter()
                    .any(|trivia| trivia.kind == YamlTriviaKind::Directive)
            {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if tab_in_indentation(self.source.line_text(self.line)).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Sequence(YamlSequence {
                indent,
                items,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            node_leading,
            state,
        ));
        self.plan_yaml_sequence_emit(id);
        Ok(id)
    }

    fn parse_compact_sequence(&mut self, indent: usize) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlSequence);
        let start = self.source.lines[self.line].text.start() + indent;
        let mut end = start;
        let mut items = Vec::new();
        let mut leading = Vec::new();
        let mut first = true;

        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            let marker_local = if first {
                let Some(marker) = compact_sequence_marker_at(text, indent) else {
                    break;
                };
                marker
            } else {
                let actual_indent = indentation(text);
                let Some(marker) = sequence_line(text, indent) else {
                    self.unread_trivia(leading);
                    break;
                };
                if actual_indent != indent {
                    self.unread_trivia(leading);
                    break;
                }
                marker
            };
            first = false;

            let marker = Span::new(
                line.text.start() + marker_local,
                line.text.start() + marker_local + 1,
            );
            let value_start = marker_local
                + 1
                + text[marker_local + 1..]
                    .bytes()
                    .take_while(|byte| byte.is_ascii_whitespace())
                    .count();
            let line_value = split_line_value_comment(text, value_start, line.text.start());
            self.reject_populated_same_line_yaml_directive(line_value)?;

            let mut item_trailing_comment = None;
            let mut value_on_marker_line = false;
            let value = if let Some(block) = quoted_scalar_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            ) {
                let value = self.parse_quoted_scalar_block(block)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) = flow_collection_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            )
            .and_then(FlowCollectionScan::complete)
            {
                let value = self.parse_flow_collection_block(block, Vec::new())?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(hint) =
                flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
            {
                let value =
                    self.parse_flow_collapse_hint_collection(indent, Vec::new(), hint, false)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) =
                block_scalar_at_value(self.source, self.line, self.end, value_start, indent)
            {
                let value = self.parse_block_scalar(block)?;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_explicit_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if compact_sequence_marker_at(text, value_start).is_some() {
                let value = self.parse_compact_sequence(value_start)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_inline_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else {
                let metadata = scalar_metadata(self.source, line_value.value);
                let value_is_empty = self.source.slice(line_value.value).trim_ascii().is_empty();
                let property_only_value = !value_is_empty
                    && (metadata.tag.is_some() || metadata.anchor.is_some())
                    && self.source.slice(metadata.content).trim_ascii().is_empty();
                self.line += 1;
                if value_is_empty || property_only_value {
                    item_trailing_comment = line_value.trailing_comment;
                    let inline_directive =
                        self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                    let nested_leading = self.take_leading_trivia()?;
                    let nested_is_target = inline_directive
                        || leading
                            .iter()
                            .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                    if self.next_line_has_tab_indentation() {
                        let nested = self.parse_tab_indented_opaque(nested_leading)?;
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                        Some(nested)
                    } else if self.line < self.end
                        && indentation(self.source.line_text(self.line)) > indent
                    {
                        let nested =
                            self.parse_block(indent + 1, nested_leading, nested_is_target)?;
                        if let Some(nested) = nested {
                            if property_only_value {
                                self.attach_collection_properties(
                                    nested,
                                    metadata.tag,
                                    metadata.anchor,
                                );
                            }
                            end = self.ast.node(nested).span.end();
                        } else {
                            end = line.full.end();
                        }
                        nested
                    } else {
                        self.unread_trivia(nested_leading);
                        if property_only_value {
                            let scalar = self.push_scalar_line_with_comment(
                                line.full.into(),
                                line_value.value,
                                line_value.trailing_comment,
                                PlainScalarContinuation::None,
                            )?;
                            end = self.ast.node(scalar).span.end();
                            Some(scalar)
                        } else {
                            let scalar =
                                self.push_empty_scalar(line.full.into(), line.text.end())?;
                            end = line.full.end();
                            Some(scalar)
                        }
                    }
                } else {
                    let scalar = self.push_scalar_line_with_comment(
                        line.full.into(),
                        line_value.value,
                        line_value.trailing_comment,
                        PlainScalarContinuation::Inline {
                            parent_indent: indent,
                        },
                    )?;
                    end = self.ast.node(scalar).span.end();
                    Some(scalar)
                }
            };

            items.push(YamlSequenceItem {
                leading_trivia: std::mem::take(&mut leading).into(),
                marker: SourceSpan::new(marker),
                line: SourceSpan::new(line.full.into()),
                value_on_marker_line,
                trailing_comment: item_trailing_comment.map(SourceSpan::new),
                value,
            });

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading)
                || leading
                    .iter()
                    .any(|trivia| trivia.kind == YamlTriviaKind::Directive)
            {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Sequence(YamlSequence {
                indent,
                items,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            Vec::new(),
            state,
        ));
        self.plan_yaml_sequence_emit(id);
        Ok(id)
    }

    fn parse_sequence_item_inline_mapping(
        &mut self,
        sequence_indent: usize,
        key_start_column: usize,
    ) -> Result<Option<YamlNodeId>> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        if mapping_colon_from(text, key_start_column).is_none() {
            return Ok(None);
        }

        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping);
        let start = line.text.start() + key_start_column;
        let mut pairs = Vec::new();
        let (first_pair, mut end) = self.parse_mapping_pair_at(key_start_column, Vec::new())?;
        pairs.push(first_pair);

        let mut leading = self.take_leading_trivia()?;
        let mut continuation_indent = None;
        while self.line < self.end {
            if trivia_has_document_marker(&leading) {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let text = self.source.line_text(self.line);
            let actual_indent = indentation(text);
            if actual_indent <= sequence_indent || sequence_line(text, sequence_indent).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let pair_indent = *continuation_indent.get_or_insert(actual_indent);
            if actual_indent != pair_indent || mapping_colon_at(text, pair_indent).is_none() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let (pair, pair_end) =
                self.parse_mapping_pair_at(pair_indent, std::mem::take(&mut leading))?;
            end = pair_end;
            pairs.push(pair);
            leading = self.take_leading_trivia()?;
        }
        if self.line >= self.end {
            self.unread_trivia(leading);
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent: key_start_column,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            Vec::new(),
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(Some(id))
    }

    fn parse_sequence_item_explicit_mapping(
        &mut self,
        sequence_indent: usize,
        key_marker_column: usize,
    ) -> Result<Option<YamlNodeId>> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        if explicit_indicator_at(text, key_marker_column, b'?').is_none() {
            return Ok(None);
        }

        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping);
        let start = line.full.start();
        let mut pairs = Vec::new();
        let (first_pair, mut end) =
            self.parse_explicit_mapping_pair_at(key_marker_column, Vec::new())?;
        pairs.push(first_pair);

        let mut leading = self.take_leading_trivia()?;
        let mut continuation_indent = None;
        while self.line < self.end {
            if trivia_has_document_marker(&leading) {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let text = self.source.line_text(self.line);
            let actual_indent = indentation(text);
            if actual_indent <= sequence_indent || sequence_line(text, sequence_indent).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let pair_indent = *continuation_indent.get_or_insert(actual_indent);
            if actual_indent != pair_indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let (pair, pair_end) = if explicit_key_line(text, pair_indent).is_some() {
                self.parse_explicit_mapping_pair_at(pair_indent, std::mem::take(&mut leading))?
            } else if mapping_colon_at(text, pair_indent).is_some() {
                self.parse_mapping_pair_at(pair_indent, std::mem::take(&mut leading))?
            } else {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            };
            end = pair_end;
            pairs.push(pair);
            leading = self.take_leading_trivia()?;
        }
        if self.line >= self.end {
            self.unread_trivia(leading);
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent: key_marker_column,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            Vec::new(),
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(Some(id))
    }

    fn parse_plain_scalar_line(&mut self, leading: Vec<YamlTrivia>) -> Result<YamlNodeId> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let value_start = indentation(text);
        if let Some(block) = quoted_scalar_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            line.full.start(),
        ) {
            let id = self.parse_quoted_scalar_block(block)?;
            self.ast.node_mut(id).leading_trivia = leading.into();
            return Ok(id);
        }

        let line_value = split_line_value_comment(text, value_start, line.text.start());
        self.line += 1;
        let id = self.push_scalar_line_with_comment(
            line.full.into(),
            line_value.value,
            line_value.trailing_comment,
            PlainScalarContinuation::Block {
                indent: value_start,
            },
        )?;
        self.ast.node_mut(id).leading_trivia = leading.into();
        Ok(id)
    }

    fn parse_flow_collapse_hint_collection(
        &mut self,
        parent_indent: usize,
        leading: Vec<YamlTrivia>,
        hint: FlowCollapseHint,
        container_is_target: bool,
    ) -> Result<YamlNodeId> {
        let opener_line = self.source.lines[self.line];
        let id = match hint.rest {
            FlowCollapseRest::Empty => {
                self.line += 1;
                let nested_leading = self.take_leading_trivia()?;
                let text = self.source.line_text(self.line);
                let indent = indentation(text);
                self.parse_block(indent, nested_leading, container_is_target)?
                    .expect("prechecked flow collapse hint has a block collection target")
            }
            FlowCollapseRest::InlineMapping { key_start } => self
                .parse_sequence_item_inline_mapping(parent_indent, key_start)?
                .expect("prechecked flow collapse hint has an inline mapping target"),
        };
        self.mark_flow_collapse_hint(id, opener_line.full.start(), hint.opener);
        self.ast.node_mut(id).leading_trivia = leading.into();
        Ok(id)
    }

    fn mark_flow_collapse_hint(&mut self, id: YamlNodeId, source_start: usize, opener: Span) {
        let node = self.ast.node_mut(id);
        node.span.set_start(source_start);
        match &mut node.kind {
            YamlAstKind::Sequence(sequence) => {
                sequence.flow_collapse_hint = Some(SourceSpan::new(opener));
            }
            YamlAstKind::Mapping(mapping) => {
                mapping.flow_collapse_hint = Some(SourceSpan::new(opener));
            }
            _ => {}
        }
    }

    fn parse_opaque_flow(
        &mut self,
        block: Span,
        leading: Vec<YamlTrivia>,
    ) -> Result<Option<YamlNodeId>> {
        self.parse_opaque_flow_node(block, leading).map(Some)
    }

    fn parse_tab_indented_opaque(&mut self, leading: Vec<YamlTrivia>) -> Result<YamlNodeId> {
        let line = self.source.lines[self.line];
        let block = tab_indented_block_at(self.source, self.line, self.end);
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlUnsupported);
        let state_value = self.doc.state(state);
        if state_value.markdown_target
            || state_value.embedded_formatter.is_some()
            || state_value.table_compact.is_some()
        {
            return Err(yaml_error_at(
                self.source,
                line.text.start()
                    + tab_in_indentation(self.source.line_text(self.line)).unwrap_or(0),
                "tabs in YAML indentation are unsupported",
            ));
        }
        self.line = self.source.line_at_byte(block.end.saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Opaque(YamlOpaque {
                reason: YamlOpaqueReason::UnsupportedLine,
            }),
            block,
            leading,
            state,
        )))
    }

    fn parse_bom_prefixed_opaque(&mut self, leading: Vec<YamlTrivia>) -> Result<YamlNodeId> {
        let line = self.source.lines[self.line];
        let end_line = (self.line + 1..self.end)
            .find(|line| {
                let text = self.source.line_text(*line);
                indentation(text) == 0 && document_marker_line_info(text).is_some()
            })
            .unwrap_or(self.end);
        let block = Span::new(
            line.full.start(),
            self.source.lines[end_line - 1].full.end(),
        );
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlUnsupported);
        let state_value = self.doc.state(state);
        let mut scan = scan_yaml_lines(self.source, block);
        if let Some(first) = scan.lines.first_mut()
            && self.source.slice(first.content).starts_with('\u{feff}')
        {
            first.indent = first.indent.saturating_sub('\u{feff}'.len_utf8());
        }
        if state_value.markdown_target
            || state_value.embedded_formatter.is_some()
            || state_value.table_compact.is_some()
            || yaml_scan_has_active_fmt_directive(self.source, &scan)
        {
            return Err(yaml_error_at(
                self.source,
                line.text.start(),
                "formatting this BOM-prefixed YAML root is unsupported",
            ));
        }
        self.line = end_line;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Opaque(YamlOpaque {
                reason: YamlOpaqueReason::UnsupportedLine,
            }),
            block,
            leading,
            state,
        )))
    }

    fn next_line_has_tab_indentation(&self) -> bool {
        self.line < self.end && tab_in_indentation(self.source.line_text(self.line)).is_some()
    }

    fn parse_opaque_flow_node(
        &mut self,
        block: Span,
        leading: Vec<YamlTrivia>,
    ) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlUnsupported);
        let state_value = self.doc.state(state);
        if state_value.markdown_target
            || state_value.embedded_formatter.is_some()
            || state_value.table_compact.is_some()
        {
            let line = self.source.lines[self.line];
            return Err(yaml_error_at(
                self.source,
                line.text.start(),
                "fmt directive targets unsupported YAML flow syntax",
            ));
        }
        self.line = self.source.line_at_byte(block.end.saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Opaque(YamlOpaque {
                reason: YamlOpaqueReason::UnsupportedFlow,
            }),
            block,
            leading,
            state,
        )))
    }

    fn parse_quoted_scalar_block(&mut self, block: QuotedScalarBlock) -> Result<YamlNodeId> {
        self.reject_same_line_yaml_directive(block.trailing_comment)?;
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        self.validate_inline_scalar_directive_target(state, block.value)?;
        let metadata = scalar_metadata(self.source, block.value);
        let source = self.source.slice(metadata.content).trim_ascii();
        self.line = self.source.line_at_byte(block.full.end().saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: scalar_style(source),
                semantic: scalar_semantic_with_tag(
                    source,
                    metadata.tag.map(|tag| self.source.slice(tag)),
                ),
                value: SourceSpan::new(block.value),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: block.trailing_comment.map(SourceSpan::new),
            }),
            block.full,
            Vec::new(),
            state,
        )))
    }

    fn parse_flow_collection_block(
        &mut self,
        block: FlowCollectionBlock,
        leading: Vec<YamlTrivia>,
    ) -> Result<YamlNodeId> {
        self.reject_same_line_yaml_directive(block.trailing_comment)?;

        let target = flow_collection_directive_target(self.source, block.collection);
        let needs_rollback = self.engine.has_pending_target_directives();
        let engine_checkpoint = needs_rollback.then(|| self.engine.clone());
        let states_checkpoint = needs_rollback.then(|| self.doc.states.clone());
        let state = self.engine.state_for_yaml_node(&mut self.doc, target);
        let checkpoint = self.ast.nodes.len();
        let template_spans_possible = self.template_spans_possible_for_state(state);
        let Some(id) = self.parse_recognized_flow_collection(block, state, template_spans_possible)
        else {
            if let Some(engine) = engine_checkpoint {
                self.engine = engine;
            }
            if let Some(states) = states_checkpoint {
                self.doc.states = states;
            }
            return self.parse_opaque_flow_node(block.span, leading);
        };
        if let Err(err) = self.validate_flow_collection_directive_target(state, block.value) {
            self.ast.nodes.truncate(checkpoint);
            self.flow_collection_nodes.clear();
            return Err(err);
        }
        self.ast.node_mut(id).leading_trivia = leading.into();
        self.plan_parsed_yaml_flow_nodes();
        self.line = self.source.line_at_byte(block.span.end.saturating_sub(1)) + 1;
        Ok(id)
    }

    fn parse_recognized_flow_collection(
        &mut self,
        block: FlowCollectionBlock,
        state: StateId,
        template_spans_possible: bool,
    ) -> Option<YamlNodeId> {
        let delimiters = &self.doc.state(state).template_delimiters;
        if let Some(recognized) = self
            .recognized_flow
            .take_if(|parsed| parsed.value == block.value)
        {
            // Recognition uses YAML syntax alone. Active template delimiters can
            // change token boundaries, so those values still need the configured parser.
            if !template_spans_possible
                || !source_may_contain_template_span(
                    self.source.slice(block.collection),
                    delimiters,
                )
            {
                let id =
                    recognized.append_to(&mut self.ast, &mut self.flow_collection_nodes, state);
                self.ast.node_mut(id).span = SourceSpan::new(block.span);
                return Some(id);
            }
        }
        parse_flow_collection(
            self.source,
            &mut self.ast,
            &mut self.flow_collection_nodes,
            state,
            delimiters,
            template_spans_possible,
            block.span,
            block.value,
            block.collection,
            block.trailing_comment,
        )
    }

    fn parse_block_scalar(&mut self, block: BlockScalar) -> Result<YamlNodeId> {
        self.reject_same_line_yaml_directive(block.trailing_comment)?;
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        let state_value = self.doc.state(state).clone();
        let metadata = scalar_metadata(self.source, block.value);
        let tagged_markdown = metadata
            .tag
            .is_some_and(|tag| yaml_tag_is_markdown(self.source.slice(tag)));
        let style = block_scalar_style(self.source.slice(block.header));
        if state_value.embedded_formatter.is_some() && style != YamlScalarStyle::LiteralBlock {
            return Err(self.embedded_formatter_target_error(state, block.value));
        }
        let nested = if state_value.markdown_target || tagged_markdown {
            let nested = if self.plan_emits {
                crate::core::markdown::parse_markdown(
                    self.source,
                    block.body,
                    state_value.markdown_options(self.options),
                    self.config,
                )?
            } else {
                crate::core::markdown::parse_markdown_for_concrete_validation(
                    self.source,
                    block.body,
                    state_value.markdown_options(self.options),
                    self.config,
                )?
            };
            Some(self.doc.push_nested(nested))
        } else {
            None
        };
        let nested = nested.map(compact_nested_document_index);
        self.line = self.source.line_at_byte(block.full.end().saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style,
                semantic: YamlScalarSemantic::String,
                value: SourceSpan::new(block.value),
                header: Some(SourceSpan::new(block.header)),
                block_header: Some(block.header_info),
                body: Some(SourceSpan::new(block.body)),
                nested,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: block.trailing_comment.map(SourceSpan::new),
            }),
            block.full,
            Vec::new(),
            state,
        )))
    }

    fn push_empty_scalar(&mut self, span: Span, at: usize) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        self.validate_inline_scalar_directive_target(state, Span::empty(at))?;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: YamlScalarStyle::Plain,
                semantic: YamlScalarSemantic::Null,
                value: SourceSpan::empty(at),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: None,
                anchor: None,
                trailing_comment: None,
            }),
            span,
            Vec::new(),
            state,
        )))
    }

    fn push_scalar_line_with_comment(
        &mut self,
        mut span: Span,
        mut value: Span,
        trailing_comment: Option<Span>,
        continuation: PlainScalarContinuation,
    ) -> Result<YamlNodeId> {
        let mut metadata = scalar_metadata(self.source, value);
        if !self.source.slice(metadata.content).trim_ascii().is_empty() {
            self.reject_same_line_yaml_directive(trailing_comment)?;
        }
        if scalar_content_flow_collection_target(self.source, metadata.content).is_some() {
            return self.parse_flow_collection_block(
                FlowCollectionBlock {
                    span,
                    value,
                    collection: metadata.content,
                    trailing_comment,
                },
                Vec::new(),
            );
        }
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        self.validate_inline_scalar_directive_target(state, value)?;
        if metadata.content.is_empty() && (metadata.tag.is_some() || metadata.anchor.is_some()) {
            value = metadata.content;
        }
        let source = self.source.slice(metadata.content).trim_ascii();
        if alias_scalar(source) {
            self.validate_non_scalar_value_directive_target(state, metadata.content)?;
            return Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
                YamlAstKind::Alias(YamlAlias {
                    value: SourceSpan::new(metadata.content),
                    trailing_comment: trailing_comment.map(SourceSpan::new),
                }),
                span,
                Vec::new(),
                state,
            )));
        }
        if trailing_comment.is_none()
            && scalar_style(source) == YamlScalarStyle::Plain
            && !source.is_empty()
            && let Some(extension) = self.take_plain_scalar_continuation(continuation)
        {
            span.end = extension.span_end;
            value.end = extension.value_end;
            metadata.content.end = extension.value_end;
        }
        let source = self.source.slice(metadata.content).trim_ascii();
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: scalar_style(source),
                semantic: scalar_semantic_with_tag(
                    source,
                    metadata.tag.map(|tag| self.source.slice(tag)),
                ),
                value: SourceSpan::new(value),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: trailing_comment.map(SourceSpan::new),
            }),
            span,
            Vec::new(),
            state,
        )))
    }

    fn take_plain_scalar_continuation(
        &mut self,
        continuation: PlainScalarContinuation,
    ) -> Option<PlainScalarExtension> {
        if matches!(continuation, PlainScalarContinuation::None) {
            return None;
        }

        let initial_line = self.line;
        let mut scan = self.line;
        let mut final_line = self.line;
        let mut span_end = None;
        let mut value_end = None;

        while scan < self.end {
            let line = self.source.lines[scan];
            let text = self.source.line_text(scan);
            if text.trim_ascii().is_empty() {
                scan += 1;
                continue;
            }
            if !plain_scalar_continuation_line(text, continuation) {
                break;
            }

            final_line = scan + 1;
            span_end = Some(line.full.end());
            value_end = Some(line.text.end());
            scan += 1;
        }

        if final_line == initial_line {
            return None;
        }

        self.line = final_line;
        Some(PlainScalarExtension {
            span_end: span_end.expect("continuation has a final line"),
            value_end: value_end.expect("continuation has a final line"),
        })
    }

    fn validate_inline_scalar_directive_target(&self, state: StateId, span: Span) -> Result<()> {
        let state_value = self.doc.state(state);
        if state_value.embedded_formatter.is_some() {
            return Err(self.embedded_formatter_target_error(state, span));
        }
        if state_value.markdown_target {
            return Ok(());
        }
        Ok(())
    }

    fn validate_flow_collection_directive_target(&self, state: StateId, span: Span) -> Result<()> {
        let state_value = self.doc.state(state);
        if state_value.embedded_formatter.is_some() {
            return Err(self.embedded_formatter_target_error(state, span));
        }
        if state_value.markdown_target {
            return Err(yaml_error_at(
                self.source,
                span.start,
                "fmt: markdown targets a scalar value",
            ));
        }
        Ok(())
    }

    fn validate_non_value_directive_target(&self, node: YamlNodeId) -> Result<()> {
        let state = self.ast.node(node).state;
        self.validate_non_scalar_value_directive_target(state, self.ast.node(node).span.span())
    }

    fn validate_non_scalar_value_directive_target(&self, state: StateId, span: Span) -> Result<()> {
        let state_value = self.doc.state(state);
        if state_value.embedded_formatter.is_some() {
            return Err(self.embedded_formatter_target_error(state, span));
        }
        if state_value.markdown_target {
            return Err(yaml_error_at(
                self.source,
                span.start,
                "fmt: markdown targets a scalar value",
            ));
        }
        Ok(())
    }

    fn embedded_formatter_target_error(&self, state: StateId, span: Span) -> YamarkError {
        let name = self
            .doc
            .state(state)
            .embedded_formatter
            .as_deref()
            .unwrap_or("embedded formatter");
        yaml_error_at(
            self.source,
            span.start,
            format!("fmt: {name} targets a literal block scalar"),
        )
    }
}
