use std::path::Path;

use crate::core::directives::{DirectiveDelta, DirectiveState, DirectiveStateTable, StateId};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::YamlDocumentAst;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Markdown,
    Yaml,
    Python,
    R,
    Unsupported,
}

impl FileKind {
    pub fn for_path(path: &Path) -> Self {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            return Self::Unsupported;
        };
        let extension = extension.to_ascii_lowercase();
        match extension.as_str() {
            "md" | "qmd" | "rmd" => Self::Markdown,
            "yaml" | "yml" => Self::Yaml,
            "py" => Self::Python,
            "r" => Self::R,
            _ => Self::Unsupported,
        }
    }

    pub fn is_supported(self) -> bool {
        !matches!(self, Self::Unsupported)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Markdown,
    Yaml,
    Python,
    R,
}

impl DocumentKind {
    pub fn from_file_kind(kind: FileKind) -> Option<Self> {
        match kind {
            FileKind::Markdown => Some(Self::Markdown),
            FileKind::Yaml => Some(Self::Yaml),
            FileKind::Python => Some(Self::Python),
            FileKind::R => Some(Self::R),
            FileKind::Unsupported => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownWrap {
    None,
    Paragraph,
    Sentence,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub line_width: usize,
    pub prose_width: usize,
    pub indent_width: usize,
    pub markdown_compact_tables: bool,
    pub yaml_compact: bool,
    pub markdown_wrap: MarkdownWrap,
    pub markdown_wrap_at_column: usize,
    pub markdown_canonical: bool,
    pub markdown_format_footnotes: bool,
    pub markdown_preserve_footnotes: bool,
    pub markdown_horizontal_rule: &'static str,
    pub default_line_ending: &'static str,
    pub respect_frontmatter_markdown_options: bool,
    pub skip_embedded_formatters: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            line_width: 80,
            prose_width: 72,
            indent_width: 2,
            markdown_compact_tables: false,
            yaml_compact: false,
            markdown_wrap: MarkdownWrap::Column,
            markdown_wrap_at_column: 72,
            markdown_canonical: false,
            markdown_format_footnotes: true,
            markdown_preserve_footnotes: false,
            markdown_horizontal_rule: "---",
            default_line_ending: "\n",
            respect_frontmatter_markdown_options: true,
            skip_embedded_formatters: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document<'src> {
    pub kind: DocumentKind,
    pub range: Span,
    pub source: Option<SourceBuffer>,
    pub nodes: Vec<Node<'src>>,
    pub nested: Vec<Document<'src>>,
    pub states: DirectiveStateTable,
    pub yaml: Option<YamlDocumentAst<'src>>,
    pub trace: DocumentTrace,
    pub options: FormatOptions,
    pub skip_file: bool,
}

impl<'src> Document<'src> {
    pub fn new(kind: DocumentKind, range: Span) -> Self {
        Self {
            kind,
            range,
            source: None,
            nodes: Vec::new(),
            nested: Vec::new(),
            states: DirectiveStateTable::new(),
            yaml: None,
            trace: DocumentTrace::default(),
            options: FormatOptions::default(),
            skip_file: false,
        }
    }

    pub fn state(&self, id: StateId) -> &DirectiveState {
        self.states.get(id)
    }

    pub fn push_node(&mut self, node: Node<'src>) {
        self.nodes.push(node);
    }

    pub fn push_nested(&mut self, document: Document<'src>) -> usize {
        let id = self.nested.len();
        self.nested.push(document);
        id
    }

    pub fn patch_all_states(&mut self, delta: DirectiveDelta) {
        let mut patched_ids = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let mut state = self.states.get(node.state).clone();
            delta.apply_to(&mut state);
            patched_ids.push(self.states.intern(state));
        }
        for (node, state) in self.nodes.iter_mut().zip(patched_ids) {
            node.state = state;
        }
    }

    pub(crate) fn retag_source_lifetime<'dst>(self) -> Document<'dst> {
        Document {
            kind: self.kind,
            range: self.range,
            source: self.source,
            nodes: self
                .nodes
                .into_iter()
                .map(Node::retag_source_lifetime)
                .collect(),
            nested: self
                .nested
                .into_iter()
                .map(Document::retag_source_lifetime)
                .collect(),
            states: self.states,
            yaml: self.yaml.map(YamlDocumentAst::retag_source_lifetime),
            trace: self.trace,
            options: self.options,
            skip_file: self.skip_file,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DocumentTrace {
    pub source_scans: usize,
    pub parse_passes: usize,
    pub yaml_scanned_lines: usize,
    pub yaml_semantic_nodes: usize,
    pub planned_rendered_scalars: usize,
    pub planned_rendered_flow_collections: usize,
    pub planned_rendered_block_flow_collections: usize,
}

#[derive(Debug, Clone)]
pub struct Node<'src> {
    pub kind: NodeKind,
    pub span: Span,
    pub state: StateId,
    pub emit: EmitPlan<'src>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceText<'src> {
    Span(SourceSpan<'src>),
    Owned(Box<str>),
}

impl<'src> SourceText<'src> {
    pub(crate) fn span(span: Span) -> Self {
        Self::Span(SourceSpan::new(span))
    }

    pub(crate) fn owned(text: String) -> Self {
        Self::Owned(text.into_boxed_str())
    }

    pub fn as_str<'a>(&'a self, source: &'src SourceBuffer) -> &'a str
    where
        'src: 'a,
    {
        match self {
            Self::Span(span) => span.as_str(source),
            Self::Owned(text) => text.as_ref(),
        }
    }

    pub(crate) fn retag_source_lifetime<'dst>(self) -> SourceText<'dst> {
        match self {
            Self::Span(span) => SourceText::Span(span.retag()),
            Self::Owned(text) => SourceText::Owned(text),
        }
    }
}

impl<'src> Node<'src> {
    pub(crate) fn retag_source_lifetime<'dst>(self) -> Node<'dst> {
        Node {
            kind: self.kind,
            span: self.span,
            state: self.state,
            emit: self.emit.retag_source_lifetime(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Markdown(MarkdownNodeKind),
    Yaml(YamlNodeKind),
    Source(SourceNodeKind),
    Trivia,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownNodeKind {
    FrontMatter,
    Blank,
    Directive,
    Heading,
    SetextHeading,
    ThematicBreak,
    Paragraph,
    Table,
    GfmPipeTable,
    PandocTable,
    List,
    DefinitionList,
    FootnoteDefinition,
    ReferenceDefinition,
    Blockquote,
    CodeFence,
    QuartoDiv,
    Shortcode,
    DisplayMath,
    HtmlComment,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlNodeKind {
    Document,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceNodeKind {
    Comment,
    Directive,
    StringLiteral,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeFenceSafety {
    pub marker: char,
    pub min_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitPlan<'src> {
    Copy,
    Preserve,
    MarkdownHeading {
        marker: Span,
        content: Span,
    },
    MarkdownSetextHeading {
        content: Span,
        depth: usize,
    },
    MarkdownThematicBreak,
    MarkdownParagraph,
    MarkdownTable,
    MarkdownPandocTable,
    MarkdownList,
    MarkdownDefinitionList,
    MarkdownBlockquote,
    MarkdownFrontMatter {
        opening: Span,
        closing: Span,
        nested: usize,
    },
    MarkdownCodeFence {
        opening: Span,
        normalized_opening: Option<Box<str>>,
        closing: Span,
        nested: Option<usize>,
        safety: CodeFenceSafety,
        supported: bool,
    },
    MarkdownDiv {
        opening: Span,
        closing: Span,
        nested: usize,
    },
    MarkdownOpaque,
    YamlDocument,
    EmbeddedMarkdownString {
        opening: Span,
        body: Span,
        closing: Span,
        nested: usize,
        indent: SourceSpan<'src>,
        closing_indent: SourceSpan<'src>,
    },
    EmbeddedMarkdownComment {
        prefix: SourceText<'src>,
        nested: usize,
    },
    EmbeddedYamlComment {
        prefix: SourceText<'src>,
        nested: usize,
    },
    ExternalPlugin {
        name: Box<str>,
        body: Span,
        string_indent: Option<SourceSpan<'src>>,
        normalized_opening: Option<Box<str>>,
        fence_safety: Option<CodeFenceSafety>,
    },
}

impl<'src> EmitPlan<'src> {
    pub(crate) fn retag_source_lifetime<'dst>(self) -> EmitPlan<'dst> {
        match self {
            Self::Copy => EmitPlan::Copy,
            Self::Preserve => EmitPlan::Preserve,
            Self::MarkdownHeading { marker, content } => {
                EmitPlan::MarkdownHeading { marker, content }
            }
            Self::MarkdownSetextHeading { content, depth } => {
                EmitPlan::MarkdownSetextHeading { content, depth }
            }
            Self::MarkdownThematicBreak => EmitPlan::MarkdownThematicBreak,
            Self::MarkdownParagraph => EmitPlan::MarkdownParagraph,
            Self::MarkdownTable => EmitPlan::MarkdownTable,
            Self::MarkdownPandocTable => EmitPlan::MarkdownPandocTable,
            Self::MarkdownList => EmitPlan::MarkdownList,
            Self::MarkdownDefinitionList => EmitPlan::MarkdownDefinitionList,
            Self::MarkdownBlockquote => EmitPlan::MarkdownBlockquote,
            Self::MarkdownFrontMatter {
                opening,
                closing,
                nested,
            } => EmitPlan::MarkdownFrontMatter {
                opening,
                closing,
                nested,
            },
            Self::MarkdownCodeFence {
                opening,
                normalized_opening,
                closing,
                nested,
                safety,
                supported,
            } => EmitPlan::MarkdownCodeFence {
                opening,
                normalized_opening,
                closing,
                nested,
                safety,
                supported,
            },
            Self::MarkdownDiv {
                opening,
                closing,
                nested,
            } => EmitPlan::MarkdownDiv {
                opening,
                closing,
                nested,
            },
            Self::MarkdownOpaque => EmitPlan::MarkdownOpaque,
            Self::YamlDocument => EmitPlan::YamlDocument,
            Self::EmbeddedMarkdownString {
                opening,
                body,
                closing,
                nested,
                indent,
                closing_indent,
            } => EmitPlan::EmbeddedMarkdownString {
                opening,
                body,
                closing,
                nested,
                indent: indent.retag(),
                closing_indent: closing_indent.retag(),
            },
            Self::EmbeddedMarkdownComment { prefix, nested } => EmitPlan::EmbeddedMarkdownComment {
                prefix: prefix.retag_source_lifetime(),
                nested,
            },
            Self::EmbeddedYamlComment { prefix, nested } => EmitPlan::EmbeddedYamlComment {
                prefix: prefix.retag_source_lifetime(),
                nested,
            },
            Self::ExternalPlugin {
                name,
                body,
                string_indent,
                normalized_opening,
                fence_safety,
            } => EmitPlan::ExternalPlugin {
                name,
                body,
                string_indent: string_indent.map(SourceSpan::retag),
                normalized_opening,
                fence_safety,
            },
        }
    }
}
