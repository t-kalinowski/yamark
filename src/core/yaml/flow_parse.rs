//! Recognition and parsing of YAML flow collections.

use super::scalar::{empty_scalar_semantic_with_tag, scalar_semantic_with_tag_and_style};
use super::syntax::{
    alias_scalar, comment_can_start, flow_collection_block_from_value,
    flow_colon_is_value_indicator, line_flow_value_start, quoted_scalar_close, scalar_metadata,
    scalar_style,
};
use crate::core::directives::{StateId, TemplateDelimiter};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAlias, YamlAstKind, YamlAstNode, YamlDocumentAst, YamlEmitPlan, YamlFlowMapping,
    YamlFlowPair, YamlFlowSequence, YamlNodeId, YamlRenderedKind, YamlScalar, YamlScalarStyle,
    YamlTrivia, YamlTriviaKind,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn parse_flow_collection(
    source: &SourceBuffer,
    ast: &mut YamlDocumentAst,
    collection_nodes: &mut Vec<YamlNodeId>,
    state: StateId,
    template_delimiters: &[TemplateDelimiter],
    template_spans_possible: bool,
    line_span: Span,
    value: Span,
    collection: Span,
    trailing_comment: Option<Span>,
) -> Option<YamlNodeId> {
    let text = source.slice(collection);
    let (trimmed_start, trimmed_end) = trim_ascii_range(text, 0, text.len())?;
    let collection = Span::new(
        collection.start + trimmed_start,
        collection.start + trimmed_end,
    );
    let first = source.slice(collection).as_bytes().first().copied()?;
    if !matches!(first, b'[' | b'{') {
        return None;
    }
    let metadata = scalar_metadata(source, value);

    collection_nodes.clear();
    let checkpoint = ast.nodes.len();
    let parsed = {
        let mut parser = FlowParser {
            text: source.slice(collection),
            base: collection.start,
            pos: 0,
            ast,
            collection_nodes,
            state,
            template_delimiters,
            template_spans_possible,
            inner_trivia: Vec::new(),
        };
        let id = parser.parse_value();
        parser.skip_ws();
        id.filter(|_| parser.pos == parser.text.len())
            .map(|id| (id, !parser.inner_trivia.is_empty()))
    };
    let Some((id, has_inner_trivia)) = parsed else {
        ast.nodes.truncate(checkpoint);
        collection_nodes.clear();
        return None;
    };

    let node = ast.node_mut(id);
    node.span = SourceSpan::new(line_span);
    match &mut node.kind {
        YamlAstKind::FlowSequence(sequence) => {
            sequence.value = SourceSpan::new(value);
            sequence.tag = metadata.tag.map(SourceSpan::new);
            sequence.anchor = metadata.anchor.map(SourceSpan::new);
            sequence.trailing_comment = trailing_comment.map(SourceSpan::new);
            sequence.has_inner_trivia = has_inner_trivia;
        }
        YamlAstKind::FlowMapping(mapping) => {
            mapping.value = SourceSpan::new(value);
            mapping.tag = metadata.tag.map(SourceSpan::new);
            mapping.anchor = metadata.anchor.map(SourceSpan::new);
            mapping.trailing_comment = trailing_comment.map(SourceSpan::new);
            mapping.has_inner_trivia = has_inner_trivia;
        }
        _ => {
            ast.nodes.truncate(checkpoint);
            collection_nodes.clear();
            return None;
        }
    }
    Some(id)
}

pub(super) struct RecognizedFlowCollection {
    pub(super) value: Span,
    ast: YamlDocumentAst,
    collection_nodes: Vec<YamlNodeId>,
    root: YamlNodeId,
}

impl RecognizedFlowCollection {
    pub(super) fn append_to(
        mut self,
        ast: &mut YamlDocumentAst,
        collection_nodes: &mut Vec<YamlNodeId>,
        state: StateId,
    ) -> YamlNodeId {
        let offset = ast.nodes.len();
        let relocate = |id: YamlNodeId| YamlNodeId::new(offset + id.index());
        for node in &mut self.ast.nodes {
            node.state = state;
            match &mut node.kind {
                YamlAstKind::FlowSequence(sequence) => {
                    for entry in &mut sequence.entries {
                        *entry = relocate(*entry);
                    }
                }
                YamlAstKind::FlowMapping(mapping) => {
                    for pair in &mut mapping.pairs {
                        pair.key = relocate(pair.key);
                        pair.value = pair.value.map(relocate);
                    }
                }
                YamlAstKind::Scalar(_) | YamlAstKind::Alias(_) => {}
                _ => unreachable!("flow parsing produces only flow collections and scalar nodes"),
            }
        }
        if ast.nodes.is_empty() {
            ast.nodes = self.ast.nodes;
        } else {
            ast.nodes.extend(self.ast.nodes);
        }
        collection_nodes.clear();
        collection_nodes.extend(self.collection_nodes.into_iter().map(relocate));
        relocate(self.root)
    }
}

pub(super) fn recognize_flow_collection(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> Option<RecognizedFlowCollection> {
    let value_start = line_flow_value_start(source.line_text(line))?;
    let block = flow_collection_block_from_value(
        source,
        line,
        end,
        value_start,
        source.lines[line].full.start(),
    )?
    .complete()?;
    let mut ast = YamlDocumentAst::new(block.value);
    let mut collection_nodes = Vec::new();
    let root = parse_flow_collection(
        source,
        &mut ast,
        &mut collection_nodes,
        StateId(0),
        &[],
        false,
        block.span,
        block.value,
        block.collection,
        block.trailing_comment,
    )?;
    Some(RecognizedFlowCollection {
        value: block.value,
        ast,
        collection_nodes,
        root,
    })
}

struct FlowParser<'src, 'ast, 'cfg> {
    text: &'src str,
    base: usize,
    pos: usize,
    ast: &'ast mut YamlDocumentAst,
    collection_nodes: &'ast mut Vec<YamlNodeId>,
    state: StateId,
    template_delimiters: &'cfg [TemplateDelimiter],
    template_spans_possible: bool,
    inner_trivia: Vec<YamlTrivia>,
}

#[derive(Debug, Clone, Copy)]
struct FlowNodeProperties {
    start: usize,
    tag: Option<Span>,
    anchor: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
enum FlowImplicitMappingStart {
    Reparse,
    PlainKey { end: usize },
}

impl FlowNodeProperties {
    fn empty_at(pos: usize) -> Self {
        Self {
            start: pos,
            tag: None,
            anchor: None,
        }
    }
}

impl<'src, 'ast, 'cfg> FlowParser<'src, 'ast, 'cfg> {
    fn parse_value(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        let properties = self.parse_node_properties();
        if self.value_is_empty_node() {
            return Some(self.push_empty_flow_scalar(properties));
        }
        let id = match self.peek_byte()? {
            b'[' => self.parse_sequence(),
            b'{' => self.parse_mapping(),
            _ => self.parse_scalar_until(b",]}", properties),
        }?;
        self.apply_node_properties(id, properties);
        Some(id)
    }

    fn parse_sequence_entry(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        if let Some(start) = self.consume_explicit_key_indicator() {
            self.parse_sequence_entry_mapping(start)
        } else if self.sequence_entry_starts_with_collection() {
            self.parse_sequence_entry_collection()
        } else if let Some(mapping_start) = self.sequence_entry_implicit_mapping_start() {
            match mapping_start {
                FlowImplicitMappingStart::Reparse => self.parse_implicit_mapping(),
                FlowImplicitMappingStart::PlainKey { end } => {
                    self.parse_plain_implicit_mapping(end)
                }
            }
        } else {
            self.parse_value()
        }
    }

    fn parse_sequence_entry_collection(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        let key = self.parse_value()?;
        self.skip_ws();
        if self.consume_byte(b':').is_none() {
            return Some(key);
        }
        let value = Some(self.parse_optional_flow_value()?);
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: false,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_sequence(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        self.consume_byte(b'[')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.consume_byte(b']').is_some() {
            let id = self.push_flow_sequence(start, entries);
            self.mark_flow_inner_trivia(id, trivia_start);
            return Some(id);
        }

        loop {
            entries.push(self.parse_sequence_entry()?);
            self.skip_ws();
            if self.consume_byte(b',').is_some() {
                self.skip_ws();
                if self.consume_byte(b']').is_some() {
                    break;
                }
                continue;
            }
            self.consume_byte(b']')?;
            break;
        }

        let id = self.push_flow_sequence(start, entries);
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_mapping(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        self.consume_byte(b'{')?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.consume_byte(b'}').is_some() {
            let id = self.push_flow_mapping(start, pairs, true);
            self.mark_flow_inner_trivia(id, trivia_start);
            return Some(id);
        }

        loop {
            let pair_start = self.pos;
            let explicit = self.consume_explicit_key_indicator().is_some();
            let key = self.parse_flow_key()?;
            self.skip_ws();
            let value = if self.consume_byte(b':').is_some() {
                Some(self.parse_optional_flow_value()?)
            } else {
                Some(self.push_empty_flow_scalar(FlowNodeProperties::empty_at(self.pos)))
            };
            pairs.push(YamlFlowPair {
                key,
                value,
                explicit,
                source: SourceSpan::new(Span::new(self.base + pair_start, self.base + self.pos)),
            });
            self.skip_ws();
            if self.consume_byte(b',').is_some() {
                self.skip_ws();
                if self.consume_byte(b'}').is_some() {
                    break;
                }
                continue;
            }
            self.consume_byte(b'}')?;
            break;
        }

        let id = self.push_flow_mapping(start, pairs, true);
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_implicit_mapping(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        let key = self.parse_flow_key()?;
        self.skip_ws();
        self.consume_byte(b':')?;
        let value = Some(self.parse_optional_flow_value()?);
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: false,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_plain_implicit_mapping(&mut self, key_end: usize) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        self.skip_ws();
        let properties = self.parse_node_properties();
        let key_start = self.pos;
        self.pos = key_end;
        let key = self.push_flow_scalar(key_start, properties)?;
        self.apply_node_properties(key, properties);
        self.skip_ws();
        self.consume_byte(b':')?;
        let value = Some(self.parse_optional_flow_value()?);
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: false,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_sequence_entry_mapping(&mut self, start: usize) -> Option<YamlNodeId> {
        let trivia_start = self.inner_trivia.len();
        let key = self.parse_flow_key()?;
        self.skip_ws();
        let value = if self.consume_byte(b':').is_some() {
            Some(self.parse_optional_flow_value()?)
        } else {
            Some(self.push_empty_flow_scalar(FlowNodeProperties::empty_at(self.pos)))
        };
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: true,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_flow_key(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        let properties = self.parse_node_properties();
        if self.key_is_empty_node() {
            return Some(self.push_empty_flow_scalar(properties));
        }
        let id = match self.peek_byte()? {
            b'[' => self.parse_sequence(),
            b'{' => self.parse_mapping(),
            b'\'' | b'"' => self.parse_scalar_until(b":", properties),
            _ => self.parse_plain_flow_key_scalar(properties),
        }?;
        self.apply_node_properties(id, properties);
        Some(id)
    }

    fn parse_plain_flow_key_scalar(
        &mut self,
        properties: FlowNodeProperties,
    ) -> Option<YamlNodeId> {
        let start = self.pos;
        let (end, _) = self.plain_flow_key_end(start)?;
        self.pos = end;
        self.push_flow_scalar(start, properties)
    }

    fn parse_optional_flow_value(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        if self.value_is_empty_node() {
            Some(self.push_empty_flow_scalar(FlowNodeProperties::empty_at(self.pos)))
        } else {
            self.parse_value()
        }
    }

    fn value_is_empty_node(&self) -> bool {
        matches!(self.peek_byte(), None | Some(b',' | b']' | b'}'))
    }

    fn key_is_empty_node(&self) -> bool {
        matches!(self.peek_byte(), None | Some(b',' | b']' | b'}'))
            || self.peek_byte() == Some(b':')
                && flow_colon_is_value_indicator(self.text, self.pos, false)
    }

    fn sequence_entry_implicit_mapping_start(&self) -> Option<FlowImplicitMappingStart> {
        let start = self.node_properties_end_from(self.pos);
        match self.text.as_bytes().get(start).copied() {
            Some(b':') if flow_colon_is_value_indicator(self.text, start, false) => {
                Some(FlowImplicitMappingStart::Reparse)
            }
            Some(b'[' | b'{') => None,
            Some(quote @ (b'\'' | b'"')) => {
                let close = quoted_scalar_close(self.text, start, quote)?;
                (self.text.as_bytes().get(self.skip_ws_from(close + 1)) == Some(&b':'))
                    .then_some(FlowImplicitMappingStart::Reparse)
            }
            _ => {
                let (end, has_colon) = self.plain_flow_key_end(start)?;
                has_colon.then_some(FlowImplicitMappingStart::PlainKey { end })
            }
        }
    }

    fn sequence_entry_starts_with_collection(&self) -> bool {
        let start = self.node_properties_end_from(self.pos);
        matches!(self.text.as_bytes().get(start), Some(b'[' | b'{'))
    }

    fn explicit_key_indicator_at(&self, pos: usize) -> bool {
        if self.text.as_bytes().get(pos) != Some(&b'?') {
            return false;
        }
        self.text
            .as_bytes()
            .get(pos + 1)
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}'))
    }

    fn consume_explicit_key_indicator(&mut self) -> Option<usize> {
        let start = self.pos;
        if !self.explicit_key_indicator_at(start) {
            return None;
        }
        self.pos += 1;
        Some(start)
    }

    fn parse_scalar_until(
        &mut self,
        delimiters: &[u8],
        properties: FlowNodeProperties,
    ) -> Option<YamlNodeId> {
        let start = self.pos;
        let mut in_single = false;
        let mut in_double = false;
        let mut escaped = false;

        while self.pos < self.text.len() {
            if in_double {
                let byte = self.peek_byte()?;
                self.advance_flow_char()?;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_double = false;
                }
                continue;
            }
            if in_single {
                let byte = self.peek_byte()?;
                self.advance_flow_char()?;
                if byte == b'\'' {
                    if self.peek_byte() == Some(b'\'') {
                        self.pos += 1;
                    } else {
                        in_single = false;
                    }
                }
                continue;
            }

            let byte = self.peek_byte()?;
            if byte == b'\'' {
                in_single = true;
                self.pos += 1;
                continue;
            }
            if byte == b'"' {
                in_double = true;
                self.pos += 1;
                continue;
            }
            if self.template_spans_possible
                && let Some(end) = self.template_span_end_at(self.pos)
            {
                self.pos = end;
                continue;
            }
            if byte == b'#' && comment_can_start(self.text, start, self.pos) {
                self.push_comment_trivia();
                break;
            }
            if delimiters.contains(&byte) {
                break;
            }
            self.advance_flow_char()?;
        }

        if in_single || in_double {
            return None;
        }

        self.push_flow_scalar(start, properties)
    }

    fn push_flow_scalar(
        &mut self,
        start: usize,
        properties: FlowNodeProperties,
    ) -> Option<YamlNodeId> {
        let (trimmed_start, trimmed_end) = trim_ascii_range(self.text, start, self.pos)?;
        let content = Span::new(self.base + trimmed_start, self.base + trimmed_end);
        let span = Span::new(self.base + properties.start, content.end);
        let value = self.source_slice(content);
        if value.as_bytes().first() == Some(&b'*') && alias_scalar(value) {
            return Some(self.ast.push_node(YamlAstNode::semantic(
                YamlAstKind::Alias(YamlAlias {
                    value: SourceSpan::new(content),
                    trailing_comment: None,
                }),
                span,
                Vec::new(),
                self.state,
            )));
        }
        let style = scalar_style(value);
        let tag = properties.tag.map(|tag| self.source_slice(tag));
        let semantic = scalar_semantic_with_tag_and_style(value, tag, style);
        let mut node = YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style,
                semantic,
                value: SourceSpan::new(content),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: properties.tag.map(SourceSpan::new),
                anchor: properties.anchor.map(SourceSpan::new),
                trailing_comment: None,
            }),
            span,
            Vec::new(),
            self.state,
        );
        node.emit = YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        let id = self.ast.push_node(node);
        Some(id)
    }

    fn push_empty_flow_scalar(&mut self, properties: FlowNodeProperties) -> YamlNodeId {
        let value = Span::empty(self.base + self.pos);
        let span = Span::new(self.base + properties.start, value.end);
        let mut node = YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: YamlScalarStyle::Plain,
                semantic: empty_scalar_semantic_with_tag(
                    properties.tag.map(|tag| self.source_slice(tag)),
                ),
                value: SourceSpan::new(value),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: properties.tag.map(SourceSpan::new),
                anchor: properties.anchor.map(SourceSpan::new),
                trailing_comment: None,
            }),
            span,
            Vec::new(),
            self.state,
        );
        node.emit = YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        self.ast.push_node(node)
    }

    fn push_flow_sequence(&mut self, start: usize, entries: Vec<YamlNodeId>) -> YamlNodeId {
        let value = Span::new(self.base + start, self.base + self.pos);
        let id = self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::FlowSequence(YamlFlowSequence {
                value: SourceSpan::new(value),
                entries: entries.into_boxed_slice(),
                tag: None,
                anchor: None,
                trailing_comment: None,
                has_inner_trivia: false,
                inner_trivia: Box::new([]),
            }),
            value,
            Vec::new(),
            self.state,
        ));
        self.collection_nodes.push(id);
        id
    }

    fn mark_flow_inner_trivia(&mut self, id: YamlNodeId, trivia_start: usize) {
        if self.inner_trivia.len() == trivia_start {
            return;
        }
        let inner_trivia = self.inner_trivia[trivia_start..]
            .to_vec()
            .into_boxed_slice();
        match &mut self.ast.node_mut(id).kind {
            YamlAstKind::FlowSequence(sequence) => {
                sequence.has_inner_trivia = true;
                sequence.inner_trivia = inner_trivia;
            }
            YamlAstKind::FlowMapping(mapping) => {
                mapping.has_inner_trivia = true;
                mapping.inner_trivia = inner_trivia;
            }
            _ => {}
        }
    }

    fn push_flow_mapping(
        &mut self,
        start: usize,
        pairs: Vec<YamlFlowPair>,
        braced: bool,
    ) -> YamlNodeId {
        let value = Span::new(self.base + start, self.base + self.pos);
        let id = self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::FlowMapping(YamlFlowMapping {
                value: SourceSpan::new(value),
                pairs: pairs.into_boxed_slice(),
                braced,
                tag: None,
                anchor: None,
                trailing_comment: None,
                has_inner_trivia: false,
                inner_trivia: Box::new([]),
            }),
            value,
            Vec::new(),
            self.state,
        ));
        self.collection_nodes.push(id);
        id
    }

    fn parse_node_properties(&mut self) -> FlowNodeProperties {
        let start = self.pos;
        if !matches!(self.peek_byte(), Some(b'!' | b'&')) {
            return FlowNodeProperties {
                start,
                tag: None,
                anchor: None,
            };
        }
        let mut tag = None;
        let mut anchor = None;

        loop {
            self.skip_ws();
            let token_start = self.pos;
            let Some(marker @ (b'!' | b'&')) = self.peek_byte() else {
                break;
            };
            while self.text.as_bytes().get(self.pos).is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b']' | b'}' | b':')
            }) {
                self.pos += 1;
            }
            if self.text.as_bytes().get(self.pos).is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b']' | b'}' | b':')
            }) {
                self.pos = token_start;
                break;
            }
            let span = Span::new(self.base + token_start, self.base + self.pos);
            if marker == b'!' {
                tag = Some(span);
            } else {
                anchor = Some(span);
            }
        }

        FlowNodeProperties { start, tag, anchor }
    }

    fn apply_node_properties(&mut self, id: YamlNodeId, properties: FlowNodeProperties) {
        if properties.tag.is_none() && properties.anchor.is_none() {
            return;
        }

        let start = self.base + properties.start;
        let node = self.ast.node_mut(id);
        node.span.set_start(start);
        match &mut node.kind {
            YamlAstKind::Scalar(scalar) => {
                scalar.tag = properties.tag.map(SourceSpan::new);
                scalar.anchor = properties.anchor.map(SourceSpan::new);
            }
            YamlAstKind::FlowSequence(sequence) => {
                sequence.tag = properties.tag.map(SourceSpan::new);
                sequence.anchor = properties.anchor.map(SourceSpan::new);
                sequence.value.set_start(start);
            }
            YamlAstKind::FlowMapping(mapping) => {
                mapping.tag = properties.tag.map(SourceSpan::new);
                mapping.anchor = properties.anchor.map(SourceSpan::new);
                mapping.value.set_start(start);
            }
            _ => {}
        }
    }

    fn source_slice(&self, span: Span) -> &str {
        &self.text[span.start - self.base..span.end - self.base]
    }

    fn template_span_end_at(&self, pos: usize) -> Option<usize> {
        if !self.template_spans_possible {
            return None;
        }
        let byte = *self.text.as_bytes().get(pos)?;
        for delimiter in self.template_delimiters {
            let open = delimiter.open.as_bytes();
            if open.is_empty() || delimiter.close.is_empty() || open[0] != byte {
                continue;
            }
            if !self.text[pos..].starts_with(&delimiter.open) {
                continue;
            }
            let content_start = pos + delimiter.open.len();
            if let Some(close) = self.text[content_start..].find(&delimiter.close) {
                return Some(content_start + close + delimiter.close.len());
            }
        }
        None
    }

    fn skip_ws_from(&self, mut pos: usize) -> usize {
        while self
            .text
            .as_bytes()
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            pos += 1;
        }
        pos
    }

    fn node_properties_end_from(&self, mut pos: usize) -> usize {
        loop {
            pos = self.skip_ws_from(pos);
            let token_start = pos;
            let Some(b'!' | b'&') = self.text.as_bytes().get(pos).copied() else {
                return pos;
            };
            while self.text.as_bytes().get(pos).is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b']' | b'}' | b':')
            }) {
                pos += 1;
            }
            let valid_property = self.text.as_bytes().get(pos).is_none_or(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}' | b':')
            });
            if !valid_property {
                return token_start;
            }
        }
    }

    fn plain_flow_key_end(&self, start: usize) -> Option<(usize, bool)> {
        let mut pos = start;
        let mut flow_depth = 0usize;
        let mut in_single = false;
        let mut in_double = false;
        let mut escaped = false;

        while pos < self.text.len() {
            let ch = self.text[pos..].chars().next()?;
            let ch_len = ch.len_utf8();
            if in_double {
                pos += ch_len;
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_double = false;
                }
                continue;
            }
            if in_single {
                pos += ch_len;
                if ch == '\'' {
                    if self.text[pos..].starts_with('\'') {
                        pos += '\''.len_utf8();
                    } else {
                        in_single = false;
                    }
                }
                continue;
            }

            if self.template_spans_possible
                && let Some(end) = self.template_span_end_at(pos)
            {
                pos = end;
                continue;
            }

            match ch {
                '\'' => in_single = true,
                '"' => in_double = true,
                '#' if flow_depth == 0 && comment_can_start(self.text, start, pos) => break,
                '[' | '{' => flow_depth += 1,
                ']' | '}' if flow_depth > 0 => flow_depth -= 1,
                ':' if flow_depth == 0 && flow_colon_is_value_indicator(self.text, pos, false) => {
                    return Some((pos, true));
                }
                ',' | ']' | '}' if flow_depth == 0 => return Some((pos, false)),
                _ => {}
            }
            pos += ch_len;
        }

        if in_single || in_double || flow_depth > 0 {
            None
        } else {
            Some((pos, false))
        }
    }

    fn skip_ws(&mut self) {
        loop {
            while let Some(byte) = self.text.as_bytes().get(self.pos).copied()
                && byte.is_ascii_whitespace()
            {
                if matches!(byte, b'\r' | b'\n') {
                    let newline_start = self.pos;
                    let newline_end = if byte == b'\r'
                        && self.text.as_bytes().get(self.pos + 1) == Some(&b'\n')
                    {
                        self.pos + 2
                    } else {
                        self.pos + 1
                    };
                    let line_start = self.line_start_before(newline_start);
                    if self.text[line_start..newline_start].trim_ascii().is_empty() {
                        self.inner_trivia.push(YamlTrivia {
                            kind: YamlTriviaKind::Blank,
                            span: SourceSpan::new(Span::new(
                                self.base + line_start,
                                self.base + newline_end,
                            )),
                        });
                    }
                    self.pos = newline_end;
                } else {
                    self.pos += 1;
                }
            }
            if self.peek_byte() == Some(b'#') && comment_can_start(self.text, 0, self.pos) {
                self.push_comment_trivia();
                continue;
            }
            break;
        }
    }

    fn line_start_before(&self, pos: usize) -> usize {
        self.text[..pos]
            .rfind(['\r', '\n'])
            .map(|index| index + 1)
            .unwrap_or(0)
    }

    fn push_comment_trivia(&mut self) {
        let start = self.pos;
        while self
            .text
            .as_bytes()
            .get(self.pos)
            .is_some_and(|byte| !matches!(byte, b'\r' | b'\n'))
        {
            self.pos += 1;
        }
        self.inner_trivia.push(YamlTrivia {
            kind: YamlTriviaKind::Comment,
            span: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
        });
    }

    fn consume_byte(&mut self, byte: u8) -> Option<()> {
        if self.peek_byte() == Some(byte) {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn advance_flow_char(&mut self) -> Option<()> {
        let byte = self.peek_byte()?;
        self.pos += if byte.is_ascii() {
            1
        } else {
            self.text[self.pos..].chars().next()?.len_utf8()
        };
        Some(())
    }
}

fn trim_ascii_range(text: &str, mut start: usize, mut end: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start < end).then_some((start, end))
}
