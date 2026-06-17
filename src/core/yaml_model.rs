use std::cell::Cell;
use std::num::NonZeroU32;

use crate::core::directives::StateId;
use crate::core::source::{SourceSpan, Span};

const YAML_WIDTH_CACHE_EMPTY: u32 = u32::MAX;
const YAML_WIDTH_CACHE_NONE: u32 = u32::MAX - 1;
const YAML_SOURCE_INDENT_CACHE_EMPTY: u32 = u32::MAX;

#[derive(Debug, Clone)]
pub struct YamlDocumentAst<'src> {
    pub range: SourceSpan<'src>,
    pub roots: Vec<YamlRoot<'src>>,
    pub nodes: Vec<YamlAstNode<'src>>,
    pub trailing_trivia: Vec<YamlTrivia<'src>>,
}

impl<'src> YamlDocumentAst<'src> {
    pub fn new(range: Span) -> Self {
        Self {
            range: SourceSpan::new(range),
            roots: Vec::new(),
            nodes: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    pub fn push_node(&mut self, node: YamlAstNode<'src>) -> YamlNodeId {
        let id = YamlNodeId::new(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn node(&self, id: YamlNodeId) -> &YamlAstNode<'src> {
        &self.nodes[id.index()]
    }

    pub fn node_mut(&mut self, id: YamlNodeId) -> &mut YamlAstNode<'src> {
        &mut self.nodes[id.index()]
    }

    pub(crate) fn retag_source_lifetime<'dst>(self) -> YamlDocumentAst<'dst> {
        YamlDocumentAst {
            range: self.range.retag(),
            roots: self
                .roots
                .into_iter()
                .map(YamlRoot::retag_source_lifetime)
                .collect(),
            nodes: self
                .nodes
                .into_iter()
                .map(YamlAstNode::retag_source_lifetime)
                .collect(),
            trailing_trivia: self
                .trailing_trivia
                .into_iter()
                .map(YamlTrivia::retag_source_lifetime)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlRoot<'src> {
    pub node: Option<YamlNodeId>,
    pub start_marker: Option<SourceSpan<'src>>,
    pub end_marker: Option<SourceSpan<'src>>,
}

impl<'src> YamlRoot<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlRoot<'dst> {
        YamlRoot {
            node: self.node,
            start_marker: self.start_marker.map(SourceSpan::retag),
            end_marker: self.end_marker.map(SourceSpan::retag),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlNodeId(NonZeroU32);

impl YamlNodeId {
    pub fn new(index: usize) -> Self {
        assert!(
            index < u32::MAX as usize,
            "YAML AST node count exceeded u32::MAX"
        );
        let stored = index as u32 + 1;
        Self(NonZeroU32::new(stored).expect("index offset is nonzero"))
    }

    pub fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

#[derive(Debug, Clone)]
pub struct YamlAstNode<'src> {
    pub kind: YamlAstKind<'src>,
    pub span: SourceSpan<'src>,
    pub leading_trivia: Box<[YamlTrivia<'src>]>,
    pub state: StateId,
    pub emit: YamlEmitPlan,
    pub must_preserve_source: Option<bool>,
    inline_width: Cell<u32>,
    flow_inline_width: Cell<u32>,
    source_indent: Cell<u32>,
}

impl<'src> YamlAstNode<'src> {
    pub fn semantic(
        kind: YamlAstKind<'src>,
        span: Span,
        leading_trivia: Vec<YamlTrivia<'src>>,
        state: StateId,
    ) -> Self {
        Self {
            kind,
            span: SourceSpan::new(span),
            leading_trivia: leading_trivia.into_boxed_slice(),
            state,
            emit: YamlEmitPlan::None,
            must_preserve_source: None,
            inline_width: Cell::new(YAML_WIDTH_CACHE_EMPTY),
            flow_inline_width: Cell::new(YAML_WIDTH_CACHE_EMPTY),
            source_indent: Cell::new(YAML_SOURCE_INDENT_CACHE_EMPTY),
        }
    }

    pub fn inline_width(&self) -> Option<Option<usize>> {
        decode_yaml_width_cache(self.inline_width.get())
    }

    pub fn set_inline_width(&self, width: Option<usize>) {
        self.inline_width.set(encode_yaml_width_cache(width));
    }

    pub fn clear_inline_width(&self) {
        self.inline_width.set(YAML_WIDTH_CACHE_EMPTY);
    }

    pub fn flow_inline_width(&self) -> Option<Option<usize>> {
        decode_yaml_width_cache(self.flow_inline_width.get())
    }

    pub fn set_flow_inline_width(&self, width: Option<usize>) {
        self.flow_inline_width.set(encode_yaml_width_cache(width));
    }

    pub fn clear_flow_inline_width(&self) {
        self.flow_inline_width.set(YAML_WIDTH_CACHE_EMPTY);
    }

    pub fn source_indent(&self) -> Option<usize> {
        let indent = self.source_indent.get();
        (indent != YAML_SOURCE_INDENT_CACHE_EMPTY).then_some(indent as usize)
    }

    pub fn set_source_indent(&self, indent: usize) {
        assert!(indent < YAML_SOURCE_INDENT_CACHE_EMPTY as usize);
        self.source_indent.set(indent as u32);
    }

    fn retag_source_lifetime<'dst>(self) -> YamlAstNode<'dst> {
        YamlAstNode {
            kind: self.kind.retag_source_lifetime(),
            span: self.span.retag(),
            leading_trivia: self
                .leading_trivia
                .into_iter()
                .map(YamlTrivia::retag_source_lifetime)
                .collect(),
            state: self.state,
            emit: self.emit,
            must_preserve_source: self.must_preserve_source,
            inline_width: self.inline_width,
            flow_inline_width: self.flow_inline_width,
            source_indent: self.source_indent,
        }
    }
}

fn encode_yaml_width_cache(width: Option<usize>) -> u32 {
    match width {
        Some(width) => {
            assert!(width < YAML_WIDTH_CACHE_NONE as usize);
            width as u32
        }
        None => YAML_WIDTH_CACHE_NONE,
    }
}

fn decode_yaml_width_cache(width: u32) -> Option<Option<usize>> {
    match width {
        YAML_WIDTH_CACHE_EMPTY => None,
        YAML_WIDTH_CACHE_NONE => Some(None),
        width => Some(Some(width as usize)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlEmitPlan {
    None,
    PreserveSource,
    Rendered(YamlRenderedKind),
    NestedMarkdownBlockScalar { nested: u32 },
    ExternalBlockScalar,
}

impl YamlEmitPlan {
    pub fn rendered_shape(kind: YamlRenderedKind) -> Self {
        Self::Rendered(kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlRenderedKind {
    Table,
    CompactCollection,
    FlowCollection,
    BlockFlowCollection,
    EmptyMarkdownScalar,
    InlineMarkdownScalar,
    Scalar,
}

#[derive(Debug, Clone)]
pub enum YamlAstKind<'src> {
    Empty,
    Scalar(YamlScalar<'src>),
    Sequence(YamlSequence<'src>),
    Mapping(YamlMapping<'src>),
    FlowSequence(YamlFlowSequence<'src>),
    FlowMapping(YamlFlowMapping<'src>),
    Alias(YamlAlias<'src>),
    Opaque(YamlOpaque),
}

impl<'src> YamlAstKind<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlAstKind<'dst> {
        match self {
            Self::Empty => YamlAstKind::Empty,
            Self::Scalar(scalar) => YamlAstKind::Scalar(scalar.retag_source_lifetime()),
            Self::Sequence(sequence) => YamlAstKind::Sequence(sequence.retag_source_lifetime()),
            Self::Mapping(mapping) => YamlAstKind::Mapping(mapping.retag_source_lifetime()),
            Self::FlowSequence(sequence) => {
                YamlAstKind::FlowSequence(sequence.retag_source_lifetime())
            }
            Self::FlowMapping(mapping) => YamlAstKind::FlowMapping(mapping.retag_source_lifetime()),
            Self::Alias(alias) => YamlAstKind::Alias(alias.retag_source_lifetime()),
            Self::Opaque(opaque) => YamlAstKind::Opaque(opaque),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlScalar<'src> {
    pub style: YamlScalarStyle,
    pub semantic: YamlScalarSemantic,
    pub value: SourceSpan<'src>,
    pub header: Option<SourceSpan<'src>>,
    pub block_header: Option<YamlBlockScalarHeader>,
    pub body: Option<SourceSpan<'src>>,
    pub nested: Option<u32>,
    pub tag: Option<SourceSpan<'src>>,
    pub anchor: Option<SourceSpan<'src>>,
    pub trailing_comment: Option<SourceSpan<'src>>,
}

impl<'src> YamlScalar<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlScalar<'dst> {
        YamlScalar {
            style: self.style,
            semantic: self.semantic,
            value: self.value.retag(),
            header: self.header.map(SourceSpan::retag),
            block_header: self.block_header,
            body: self.body.map(SourceSpan::retag),
            nested: self.nested,
            tag: self.tag.map(SourceSpan::retag),
            anchor: self.anchor.map(SourceSpan::retag),
            trailing_comment: self.trailing_comment.map(SourceSpan::retag),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlScalarStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    LiteralBlock,
    FoldedBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlBlockScalarHeader {
    pub indent: Option<u8>,
    pub chomp: YamlBlockChomp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlBlockChomp {
    Clip,
    Strip,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlScalarSemantic {
    String,
    Null,
    Boolean,
    Integer,
    Float,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct YamlSequence<'src> {
    pub indent: usize,
    pub items: Vec<YamlSequenceItem<'src>>,
    pub tag: Option<SourceSpan<'src>>,
    pub anchor: Option<SourceSpan<'src>>,
    pub flow_collapse_hint: Option<SourceSpan<'src>>,
}

impl<'src> YamlSequence<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlSequence<'dst> {
        YamlSequence {
            indent: self.indent,
            items: self
                .items
                .into_iter()
                .map(YamlSequenceItem::retag_source_lifetime)
                .collect(),
            tag: self.tag.map(SourceSpan::retag),
            anchor: self.anchor.map(SourceSpan::retag),
            flow_collapse_hint: self.flow_collapse_hint.map(SourceSpan::retag),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlSequenceItem<'src> {
    pub leading_trivia: Box<[YamlTrivia<'src>]>,
    pub marker: SourceSpan<'src>,
    pub line: SourceSpan<'src>,
    pub value_on_marker_line: bool,
    pub trailing_comment: Option<SourceSpan<'src>>,
    pub value: Option<YamlNodeId>,
}

impl<'src> YamlSequenceItem<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlSequenceItem<'dst> {
        YamlSequenceItem {
            leading_trivia: self
                .leading_trivia
                .into_iter()
                .map(YamlTrivia::retag_source_lifetime)
                .collect(),
            marker: self.marker.retag(),
            line: self.line.retag(),
            value_on_marker_line: self.value_on_marker_line,
            trailing_comment: self.trailing_comment.map(SourceSpan::retag),
            value: self.value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlMapping<'src> {
    pub indent: usize,
    pub pairs: Vec<YamlMappingPair<'src>>,
    pub tag: Option<SourceSpan<'src>>,
    pub anchor: Option<SourceSpan<'src>>,
    pub flow_collapse_hint: Option<SourceSpan<'src>>,
}

impl<'src> YamlMapping<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlMapping<'dst> {
        YamlMapping {
            indent: self.indent,
            pairs: self
                .pairs
                .into_iter()
                .map(YamlMappingPair::retag_source_lifetime)
                .collect(),
            tag: self.tag.map(SourceSpan::retag),
            anchor: self.anchor.map(SourceSpan::retag),
            flow_collapse_hint: self.flow_collapse_hint.map(SourceSpan::retag),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlMappingPair<'src> {
    pub leading_trivia: Box<[YamlTrivia<'src>]>,
    pub key: SourceSpan<'src>,
    pub key_node: Option<YamlNodeId>,
    pub colon: SourceSpan<'src>,
    pub line: SourceSpan<'src>,
    pub source: SourceSpan<'src>,
    pub explicit: bool,
    pub trailing_comment: Option<SourceSpan<'src>>,
    pub value: Option<YamlNodeId>,
}

impl<'src> YamlMappingPair<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlMappingPair<'dst> {
        YamlMappingPair {
            leading_trivia: self
                .leading_trivia
                .into_iter()
                .map(YamlTrivia::retag_source_lifetime)
                .collect(),
            key: self.key.retag(),
            key_node: self.key_node,
            colon: self.colon.retag(),
            line: self.line.retag(),
            source: self.source.retag(),
            explicit: self.explicit,
            trailing_comment: self.trailing_comment.map(SourceSpan::retag),
            value: self.value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlFlowSequence<'src> {
    pub value: SourceSpan<'src>,
    pub entries: Box<[YamlNodeId]>,
    pub tag: Option<SourceSpan<'src>>,
    pub anchor: Option<SourceSpan<'src>>,
    pub trailing_comment: Option<SourceSpan<'src>>,
    pub has_inner_trivia: bool,
    pub inner_trivia: Box<[YamlTrivia<'src>]>,
}

impl<'src> YamlFlowSequence<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlFlowSequence<'dst> {
        YamlFlowSequence {
            value: self.value.retag(),
            entries: self.entries,
            tag: self.tag.map(SourceSpan::retag),
            anchor: self.anchor.map(SourceSpan::retag),
            trailing_comment: self.trailing_comment.map(SourceSpan::retag),
            has_inner_trivia: self.has_inner_trivia,
            inner_trivia: self
                .inner_trivia
                .into_iter()
                .map(YamlTrivia::retag_source_lifetime)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlFlowMapping<'src> {
    pub value: SourceSpan<'src>,
    pub pairs: Box<[YamlFlowPair<'src>]>,
    pub braced: bool,
    pub tag: Option<SourceSpan<'src>>,
    pub anchor: Option<SourceSpan<'src>>,
    pub trailing_comment: Option<SourceSpan<'src>>,
    pub has_inner_trivia: bool,
    pub inner_trivia: Box<[YamlTrivia<'src>]>,
}

impl<'src> YamlFlowMapping<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlFlowMapping<'dst> {
        YamlFlowMapping {
            value: self.value.retag(),
            pairs: self
                .pairs
                .into_iter()
                .map(YamlFlowPair::retag_source_lifetime)
                .collect(),
            braced: self.braced,
            tag: self.tag.map(SourceSpan::retag),
            anchor: self.anchor.map(SourceSpan::retag),
            trailing_comment: self.trailing_comment.map(SourceSpan::retag),
            has_inner_trivia: self.has_inner_trivia,
            inner_trivia: self
                .inner_trivia
                .into_iter()
                .map(YamlTrivia::retag_source_lifetime)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlFlowPair<'src> {
    pub key: YamlNodeId,
    pub value: Option<YamlNodeId>,
    pub explicit: bool,
    pub source: SourceSpan<'src>,
}

impl<'src> YamlFlowPair<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlFlowPair<'dst> {
        YamlFlowPair {
            key: self.key,
            value: self.value,
            explicit: self.explicit,
            source: self.source.retag(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlAlias<'src> {
    pub value: SourceSpan<'src>,
    pub trailing_comment: Option<SourceSpan<'src>>,
}

impl<'src> YamlAlias<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlAlias<'dst> {
        YamlAlias {
            value: self.value.retag(),
            trailing_comment: self.trailing_comment.map(SourceSpan::retag),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YamlOpaque {
    pub reason: YamlOpaqueReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlOpaqueReason {
    UnsupportedFlow,
    UnsupportedLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlTrivia<'src> {
    pub kind: YamlTriviaKind,
    pub span: SourceSpan<'src>,
}

impl<'src> YamlTrivia<'src> {
    fn retag_source_lifetime<'dst>(self) -> YamlTrivia<'dst> {
        YamlTrivia {
            kind: self.kind,
            span: self.span.retag(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlTriviaKind {
    Blank,
    Comment,
    Directive,
    DocumentMarker,
}
