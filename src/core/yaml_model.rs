use std::cell::Cell;
use std::num::NonZeroU32;

use crate::core::directives::StateId;
use crate::core::source::{SourceSpan, Span};

const YAML_WIDTH_CACHE_EMPTY: u32 = u32::MAX;
const YAML_WIDTH_CACHE_NONE: u32 = u32::MAX - 1;
const YAML_SOURCE_INDENT_CACHE_EMPTY: u32 = u32::MAX;

#[derive(Debug, Clone)]
pub struct YamlDocumentAst {
    pub range: SourceSpan,
    pub roots: Vec<YamlRoot>,
    pub nodes: Vec<YamlAstNode>,
    pub trailing_trivia: Vec<YamlTrivia>,
}

impl YamlDocumentAst {
    pub fn new(range: Span) -> Self {
        Self {
            range: SourceSpan::new(range),
            roots: Vec::new(),
            nodes: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    pub fn push_node(&mut self, node: YamlAstNode) -> YamlNodeId {
        let id = YamlNodeId::new(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn node(&self, id: YamlNodeId) -> &YamlAstNode {
        &self.nodes[id.index()]
    }

    pub fn node_mut(&mut self, id: YamlNodeId) -> &mut YamlAstNode {
        &mut self.nodes[id.index()]
    }
}

#[derive(Debug, Clone)]
pub struct YamlRoot {
    pub node: Option<YamlNodeId>,
    pub start_marker: Option<SourceSpan>,
    pub end_marker: Option<SourceSpan>,
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
pub struct YamlAstNode {
    pub kind: YamlAstKind,
    pub span: SourceSpan,
    pub leading_trivia: Box<[YamlTrivia]>,
    pub state: StateId,
    pub emit: YamlEmitPlan,
    pub must_preserve_source: Option<bool>,
    inline_width: Cell<u32>,
    flow_inline_width: Cell<u32>,
    source_indent: Cell<u32>,
}

impl YamlAstNode {
    pub fn semantic(
        kind: YamlAstKind,
        span: Span,
        leading_trivia: Vec<YamlTrivia>,
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
pub enum YamlAstKind {
    Empty,
    Scalar(YamlScalar),
    Sequence(YamlSequence),
    Mapping(YamlMapping),
    FlowSequence(YamlFlowSequence),
    FlowMapping(YamlFlowMapping),
    Alias(YamlAlias),
    Opaque(YamlOpaque),
}

#[derive(Debug, Clone)]
pub struct YamlScalar {
    pub style: YamlScalarStyle,
    pub semantic: YamlScalarSemantic,
    pub value: SourceSpan,
    pub header: Option<SourceSpan>,
    pub block_header: Option<YamlBlockScalarHeader>,
    pub body: Option<SourceSpan>,
    pub nested: Option<u32>,
    pub tag: Option<SourceSpan>,
    pub anchor: Option<SourceSpan>,
    pub trailing_comment: Option<SourceSpan>,
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
    pub base_indent: usize,
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
pub struct YamlSequence {
    pub indent: usize,
    pub items: Vec<YamlSequenceItem>,
    pub tag: Option<SourceSpan>,
    pub anchor: Option<SourceSpan>,
    pub flow_collapse_hint: Option<SourceSpan>,
}

#[derive(Debug, Clone)]
pub struct YamlSequenceItem {
    pub leading_trivia: Box<[YamlTrivia]>,
    pub marker: SourceSpan,
    pub line: SourceSpan,
    pub value_on_marker_line: bool,
    pub trailing_comment: Option<SourceSpan>,
    pub value: Option<YamlNodeId>,
}

#[derive(Debug, Clone)]
pub struct YamlMapping {
    pub indent: usize,
    pub pairs: Vec<YamlMappingPair>,
    pub tag: Option<SourceSpan>,
    pub anchor: Option<SourceSpan>,
    pub flow_collapse_hint: Option<SourceSpan>,
}

#[derive(Debug, Clone)]
pub struct YamlMappingPair {
    pub leading_trivia: Box<[YamlTrivia]>,
    pub key: SourceSpan,
    pub key_node: Option<YamlNodeId>,
    pub colon: SourceSpan,
    pub line: SourceSpan,
    pub source: SourceSpan,
    pub explicit: bool,
    pub trailing_comment: Option<SourceSpan>,
    pub value: Option<YamlNodeId>,
}

#[derive(Debug, Clone)]
pub struct YamlFlowSequence {
    pub value: SourceSpan,
    pub entries: Box<[YamlNodeId]>,
    pub tag: Option<SourceSpan>,
    pub anchor: Option<SourceSpan>,
    pub trailing_comment: Option<SourceSpan>,
    pub has_inner_trivia: bool,
    pub inner_trivia: Box<[YamlTrivia]>,
}

#[derive(Debug, Clone)]
pub struct YamlFlowMapping {
    pub value: SourceSpan,
    pub pairs: Box<[YamlFlowPair]>,
    pub braced: bool,
    pub tag: Option<SourceSpan>,
    pub anchor: Option<SourceSpan>,
    pub trailing_comment: Option<SourceSpan>,
    pub has_inner_trivia: bool,
    pub inner_trivia: Box<[YamlTrivia]>,
}

#[derive(Debug, Clone)]
pub struct YamlFlowPair {
    pub key: YamlNodeId,
    pub value: Option<YamlNodeId>,
    pub explicit: bool,
    pub source: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct YamlAlias {
    pub value: SourceSpan,
    pub trailing_comment: Option<SourceSpan>,
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
pub struct YamlTrivia {
    pub kind: YamlTriviaKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlTriviaKind {
    Blank,
    Comment,
    Directive,
    DocumentMarker,
}
