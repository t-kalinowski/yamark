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
    Column(usize),
    SentenceAndColumn(usize),
}

impl MarkdownWrap {
    const PARSE_ERROR: &'static str = concat!(
        "wrap must be none, paragraph, sentence, a positive integer, ",
        "or sentence:<positive integer>"
    );

    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "none" => Ok(Self::None),
            "paragraph" => Ok(Self::Paragraph),
            "sentence" => Ok(Self::Sentence),
            value => {
                if let Some(value) = value.strip_prefix("sentence:") {
                    return parse_wrap_width(value)
                        .map(Self::SentenceAndColumn)
                        .ok_or(Self::PARSE_ERROR);
                }
                parse_wrap_width(value)
                    .map(Self::Column)
                    .ok_or(Self::PARSE_ERROR)
            }
        }
    }

    pub fn column_width(self) -> Option<usize> {
        let width = match self {
            Self::Column(width) | Self::SentenceAndColumn(width) => width,
            Self::None | Self::Paragraph | Self::Sentence => return None,
        };
        assert!(width > 0, "Markdown wrap column width must be positive");
        Some(width)
    }

    pub fn breaks_at_sentences(self) -> bool {
        matches!(self, Self::Sentence | Self::SentenceAndColumn(_))
    }

    pub fn with_reduced_column_width(self, prefix_width: usize) -> Self {
        match self {
            Self::Column(width) => {
                assert!(width > 0, "Markdown wrap column width must be positive");
                Self::Column(width.saturating_sub(prefix_width).max(1))
            }
            Self::SentenceAndColumn(width) => {
                assert!(width > 0, "Markdown wrap column width must be positive");
                Self::SentenceAndColumn(width.saturating_sub(prefix_width).max(1))
            }
            Self::None | Self::Paragraph | Self::Sentence => self,
        }
    }
}

impl std::str::FromStr for MarkdownWrap {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

fn parse_wrap_width(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|width| *width > 0)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MarkdownTableWidths {
    #[default]
    Fit,
    Preserve,
}

impl MarkdownTableWidths {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "fit" => Ok(Self::Fit),
            "preserve" => Ok(Self::Preserve),
            _ => Err("table-widths must be fit or preserve"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub line_width: usize,
    pub prose_width: usize,
    pub indent_width: usize,
    pub markdown_compact_tables: bool,
    pub yaml_compact: bool,
    pub markdown_wrap: MarkdownWrap,
    pub markdown_table_widths: MarkdownTableWidths,
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
            markdown_wrap: MarkdownWrap::Column(72),
            markdown_table_widths: MarkdownTableWidths::Fit,
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
pub struct Document {
    pub kind: DocumentKind,
    pub range: Span,
    pub source: Option<SourceBuffer>,
    pub nodes: Vec<Node>,
    pub nested: Vec<Document>,
    pub states: DirectiveStateTable,
    pub yaml: Option<YamlDocumentAst>,
    // Indexed by `nodes`; public EmitPlan variants keep their existing shape.
    pub(crate) markdown: crate::core::markdown::MarkdownPlans,
    pub trace: DocumentTrace,
    pub options: FormatOptions,
    pub skip_file: bool,
}

impl Document {
    pub fn new(kind: DocumentKind, range: Span) -> Self {
        Self {
            kind,
            range,
            source: None,
            nodes: Vec::new(),
            nested: Vec::new(),
            states: DirectiveStateTable::new(),
            yaml: None,
            markdown: crate::core::markdown::MarkdownPlans::default(),
            trace: DocumentTrace::default(),
            options: FormatOptions::default(),
            skip_file: false,
        }
    }

    pub fn state(&self, id: StateId) -> &DirectiveState {
        self.states.get(id)
    }

    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node);
        self.markdown.push_node();
    }

    pub fn push_nested(&mut self, document: Document) -> usize {
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
}

/// A parsed tree and the source its spans refer to. Inspection does not grant
/// mutable access to the tree: directives are applied by the parsers, and the
/// supported skip operation below does not invalidate retained recognition.
#[derive(Debug)]
pub struct ParsedDocument {
    source: SourceBuffer,
    document: Document,
    mode: DocumentEmitMode,
}

impl ParsedDocument {
    pub(crate) fn new(source: SourceBuffer, document: Document, mode: DocumentEmitMode) -> Self {
        Self {
            source,
            document,
            mode,
        }
    }

    pub fn source(&self) -> &SourceBuffer {
        &self.source
    }

    pub fn skip_nested(&mut self, nested: usize) {
        self.document.nested[nested].skip_file = true;
    }

    /// Transfer the tree into its execution phase after options have settled.
    pub fn finalize(self, options: FormatOptions) -> PreparedDocument {
        let tree = PreparedTree::new(&self.source, self.document, options, self.mode);
        PreparedDocument {
            source: self.source,
            tree,
        }
    }
}

impl std::ops::Deref for ParsedDocument {
    type Target = Document;

    fn deref(&self) -> &Document {
        &self.document
    }
}

/// Finalized plans cannot be separated from their source or supplied new options
/// at execution. The owned buffers move through this boundary without cloning.
#[derive(Debug)]
pub struct PreparedDocument {
    source: SourceBuffer,
    tree: PreparedTree,
}

impl PreparedDocument {
    pub fn emit(
        &self,
        plugins: &crate::plugins::PluginRegistry,
    ) -> crate::diagnostic::Result<String> {
        crate::core::emit::emit_document(self, plugins)
    }

    pub(crate) fn source(&self) -> &SourceBuffer {
        &self.source
    }

    pub(crate) fn tree(&self) -> &PreparedTree {
        &self.tree
    }

    pub(crate) fn into_parts(self) -> (SourceBuffer, Document) {
        (self.source, self.tree.into_document())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DocumentEmitMode {
    Document,
    Markdown,
    Yaml,
}

/// A finalized tree may use the enclosing document's source or a fragment's
/// shared logical buffer. Both owners keep the pair private during execution.
#[derive(Debug, Clone)]
pub(crate) struct PreparedTree {
    document: Document,
    options: FormatOptions,
    mode: DocumentEmitMode,
}

impl PreparedTree {
    pub(crate) fn new(
        source: &SourceBuffer,
        mut document: Document,
        mut options: FormatOptions,
        mode: DocumentEmitMode,
    ) -> Self {
        if matches!(mode, DocumentEmitMode::Yaml)
            && !matches!(
                source.dominant_line_ending,
                crate::core::source::LineEnding::None
            )
        {
            options.default_line_ending = source.dominant_line_ending.as_str();
        }
        if !document.skip_file {
            crate::core::markdown::finalize_document(
                source,
                &mut document,
                options,
                !matches!(mode, DocumentEmitMode::Yaml),
            );
        }
        crate::core::markdown::release_drafts(&mut document);
        Self {
            document,
            options,
            mode,
        }
    }

    pub(crate) fn document(&self) -> &Document {
        &self.document
    }

    pub(crate) fn options(&self) -> FormatOptions {
        self.options
    }

    pub(crate) fn mode(&self) -> DocumentEmitMode {
        self.mode
    }

    pub(crate) fn into_document(self) -> Document {
        self.document
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
pub struct Node {
    pub kind: NodeKind,
    pub span: Span,
    pub state: StateId,
    pub emit: EmitPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceText {
    Span(SourceSpan),
    Owned(Box<str>),
}

impl SourceText {
    pub(crate) fn span(span: Span) -> Self {
        Self::Span(SourceSpan::new(span))
    }

    pub(crate) fn owned(text: String) -> Self {
        Self::Owned(text.into_boxed_str())
    }

    pub fn as_str<'a>(&'a self, source: &'a SourceBuffer) -> &'a str {
        match self {
            Self::Span(span) => span.as_str(source),
            Self::Owned(text) => text.as_ref(),
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
pub enum EmitPlan {
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
    MarkdownShortcode {
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
        indent: SourceSpan,
        closing_indent: SourceSpan,
    },
    EmbeddedMarkdownComment {
        prefix: SourceText,
        nested: usize,
    },
    EmbeddedYamlComment {
        prefix: SourceText,
        nested: usize,
    },
    ExternalPlugin {
        name: Box<str>,
        body: Span,
        string_indent: Option<SourceSpan>,
        normalized_opening: Option<Box<str>>,
        fence_safety: Option<CodeFenceSafety>,
    },
}
