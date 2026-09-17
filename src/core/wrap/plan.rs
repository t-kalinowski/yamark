//! Retained results of the existing paragraph/container helpers, not a new
//! Markdown grammar. Drafts own normalization alternatives, token spans and
//! widths, container boundaries, and prefix policy. Resolution records concrete
//! line breaks and child plans; emission only executes those records.
//!
//! Recognition and the existing line-safety/link-splitting scans remain in
//! planning. A changed child policy can require parsing its retained logical
//! buffer again: explicit target rejection and front-matter handling depend on
//! the child's initial options. This happens before execution, never while
//! restoring container prefixes. Child output is not cached, so skip flags and
//! external formatter execution still belong to document emission.

use super::*;
use crate::core::document::{Document, EmitPlan};
use crate::core::source::{SourceBuffer, SourceSpan, Span};

/// Text borrowed from the block's source, or a normalization that needs storage.
#[derive(Debug, Clone)]
pub(crate) enum Text {
    Source(SourceSpan),
    Owned(std::sync::Arc<str>),
}

impl Text {
    pub(super) fn retain(source: &str, text: &str) -> Self {
        if text.is_empty() {
            return Self::Source(SourceSpan::new(Span::empty(0)));
        }
        if text == source {
            return Self::Source(SourceSpan::new(Span::new(0, source.len())));
        }
        let start = (text.as_ptr() as usize).checked_sub(source.as_ptr() as usize);
        if let Some(start) = start.filter(|start| *start + text.len() <= source.len()) {
            Self::Source(SourceSpan::new(Span::new(start, start + text.len())))
        } else {
            Self::Owned(text.into())
        }
    }

    pub(super) fn get<'a>(&'a self, source: &'a str) -> &'a str {
        match self {
            Self::Source(span) => span.span().slice(source),
            Self::Owned(text) => text,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub(super) items: Vec<Item>,
    trim_final_newline: bool,
}

#[derive(Debug, Clone)]
pub(super) enum Item {
    Text(Text),
    Inline(InlinePlan),
    Scoped {
        text: Text,
        plan: Box<Plan>,
    },
    Child {
        fragment: Box<Fragment>,
        prefix: Prefix,
    },
}

#[derive(Debug, Clone)]
pub(super) struct InlinePlan {
    pub text: Text,
    pub lines: Vec<PlannedLine>,
    pub tokens: Vec<Text>,
    pub first_prefix: Text,
    pub continuation_prefix: Text,
    pub newline: &'static str,
    pub terminated: bool,
    pub final_newline: &'static str,
    pub escape_first: bool,
}

#[derive(Debug, Clone)]
pub(super) struct PlannedLine {
    pub tokens: std::ops::Range<usize>,
    pub suffix: Option<MarkdownHardBreakMarker>,
    pub block_start_checked: bool,
}

#[derive(Debug, Clone)]
pub(super) enum Prefix {
    Indent(usize),
    Quote(Box<str>),
    Footnote { indent: Box<str>, newline: Box<str> },
}

/// Logical container text and its execution plan have the same explicit owner.
/// Fragment parsing intentionally uses the historical default configuration.
#[derive(Debug, Clone)]
pub(crate) struct Fragment {
    source: std::sync::Arc<SourceBuffer>,
    body: Option<FragmentBody>,
    options: FormatOptions,
}

#[derive(Debug, Clone)]
enum FragmentBody {
    Plan(Box<Plan>),
    Document(Box<Document>),
}

impl FragmentBody {
    fn retain(source: &SourceBuffer, mut document: Document, options: FormatOptions) -> Self {
        // Private block sequences need only their existing plans and source
        // spans. Documents with other execution work (including skip flags,
        // nested host documents, or plugins) keep normal document emission.
        if document.skip_file
            || document.source.is_some()
            || !document.nodes.iter().all(|node| {
                matches!(
                    node.emit,
                    EmitPlan::Copy
                        | EmitPlan::Preserve
                        | EmitPlan::MarkdownOpaque
                        | EmitPlan::MarkdownHeading { .. }
                        | EmitPlan::MarkdownSetextHeading { .. }
                        | EmitPlan::MarkdownParagraph
                        | EmitPlan::MarkdownList
                        | EmitPlan::MarkdownDefinitionList
                        | EmitPlan::MarkdownBlockquote
                        | EmitPlan::MarkdownTable
                        | EmitPlan::MarkdownPandocTable
                )
            })
        {
            return Self::Document(Box::new(document));
        }
        if let [node] = document.nodes.as_slice()
            && node.span == document.range
            && !document.state(node.state).preserve
            && let Some(plan) = document.markdown.take_fragment_block(0)
        {
            return Self::Plan(Box::new(plan));
        }
        let options = crate::core::markdown::document_emit_options(source, &document, options);
        // Copied nodes coalesce into gaps between block plans. In particular,
        // a preserved fragment needs one span, not one slot per parsed node.
        let capacity = document
            .nodes
            .len()
            .min(2 * document.markdown.block_count() + 1);
        let mut plan = Plan {
            items: Vec::with_capacity(capacity),
            trim_final_newline: false,
        };
        let mut cursor = document.range.start;
        let mut index = 0;
        while index < document.nodes.len() {
            let node = &document.nodes[index];
            if let Some(end) =
                crate::core::emit::paragraph_separator_blank_run_end(&document, index)
            {
                if cursor < node.span.start {
                    plan.push_str(
                        source.as_str(),
                        source.slice(Span::new(cursor, node.span.start)),
                    );
                }
                let newline = crate::core::emit::line_ending_for_span(source, node.span);
                plan.push_str(source.as_str(), newline_for_join(newline, options));
                cursor = document.nodes[end - 1].span.end;
                index = end;
                continue;
            }
            if !document.state(node.state).preserve
                && let Some(block) = document.markdown.take_fragment_block(index)
            {
                if cursor < node.span.start {
                    plan.push_str(
                        source.as_str(),
                        source.slice(Span::new(cursor, node.span.start)),
                    );
                }
                plan.items.push(Item::Scoped {
                    text: Text::Source(SourceSpan::new(node.span)),
                    plan: Box::new(block),
                });
                cursor = node.span.end;
            }
            index += 1;
        }
        if cursor < document.range.end {
            plan.push_str(
                source.as_str(),
                source.slice(Span::new(cursor, document.range.end)),
            );
        }
        Self::Plan(Box::new(plan))
    }
}

impl Fragment {
    pub(crate) fn plan(source: String, options: FormatOptions) -> Self {
        Self::from_buffer(std::sync::Arc::new(SourceBuffer::new(source)), options)
    }

    fn from_buffer(source: std::sync::Arc<SourceBuffer>, options: FormatOptions) -> Self {
        let range = Span::new(0, source.as_str().len());
        let body = crate::core::markdown::parse_markdown_retained(
            &source,
            range,
            options,
            &crate::config::Config::default(),
        )
        .ok()
        .map(|mut document| {
            crate::core::markdown::finalize_fragment_document(&source, &mut document, options);
            FragmentBody::retain(&source, document, options)
        });
        Self {
            source,
            body,
            options,
        }
    }

    pub(crate) fn options(&self) -> FormatOptions {
        self.options
    }

    pub(crate) fn resolve_options(&mut self, options: FormatOptions) {
        if self.options != options {
            *self = Self::from_buffer(self.source.clone(), options);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.source.as_str().is_empty()
    }

    pub(crate) fn emit(&self) -> String {
        self.body
            .as_ref()
            .and_then(|body| match body {
                // Like the historical fragment emitter, this does not apply
                // top-level Markdown line cleanup to copied or opaque text.
                FragmentBody::Plan(plan) => Some(plan.emit(self.source.as_str())),
                FragmentBody::Document(document) => crate::core::emit::emit_planned_document(
                    &self.source,
                    document,
                    self.options,
                    &crate::plugins::PluginRegistry::default(),
                    false,
                )
                .ok(),
            })
            .unwrap_or_else(|| self.source.as_str().to_owned())
    }
}

impl Plan {
    pub(super) fn new() -> Self {
        Self {
            items: Vec::with_capacity(1),
            trim_final_newline: false,
        }
    }

    // Heading and table algorithms are unchanged; their normalization is
    // finalized here so recursive emission cannot re-enter inline recognition.
    pub(crate) fn normalized_block(source: &str, text: &str) -> Self {
        let mut plan = Self::new();
        plan.push_str(source, text);
        plan
    }

    pub(super) fn push_str(&mut self, source: &str, text: &str) {
        if !text.is_empty() {
            self.items.push(Item::Text(Text::retain(source, text)));
        }
    }

    /// The old safety check observes laid-out lines, including newlines inside
    /// protected tokens. Keep that policy in planning without rendering a block
    /// merely to decide whether it is supported.
    pub(super) fn plan_paragraph_block_starts(
        &mut self,
        source: &str,
        single_line: bool,
    ) -> Option<()> {
        let mut first = true;
        let mut candidate = String::new();
        for item in &mut self.items {
            let Item::Inline(inline) = item else { continue };
            let text = inline.text.get(source);
            for line in &inline.lines {
                let pieces = &inline.tokens[line.tokens.clone()];
                // Column layout already checked these joined tokens. Preserve
                // the additional physical-line check for multiline tokens and
                // lines whose hard-break suffix can affect recognition.
                if !first
                    && line.block_start_checked
                    && pieces.iter().all(|piece| {
                        memchr::memchr2(b'\n', b'\r', piece.get(text).as_bytes()).is_none()
                    })
                {
                    continue;
                }
                let line_text = if let [piece] = pieces
                    && line.suffix.is_none()
                {
                    piece.get(text)
                } else {
                    candidate.clear();
                    for (index, piece) in pieces.iter().enumerate() {
                        if index > 0 {
                            candidate.push(' ');
                        }
                        candidate.push_str(piece.get(text));
                    }
                    candidate.push_str(line.suffix.map_or("", MarkdownHardBreakMarker::suffix));
                    &candidate
                };
                for body in markdown_line_bodies(line_text) {
                    if first {
                        inline.escape_first = markdown_block_start_line(body);
                        if !single_line {
                            return Some(());
                        }
                        first = false;
                    } else if single_line && markdown_block_start_line(body) {
                        return None;
                    }
                }
            }
        }
        Some(())
    }

    pub(crate) fn emit(&self, source: &str) -> String {
        let mut output = String::with_capacity(source.len());
        self.emit_into(source, &mut output);
        output
    }

    fn emit_into(&self, source: &str, output: &mut String) {
        let start = output.len();
        for item in &self.items {
            match item {
                Item::Text(text) => output.push_str(text.get(source)),
                Item::Inline(inline) => inline.emit(source, output),
                Item::Scoped { text, plan } => plan.emit_into(text.get(source), output),
                Item::Child { fragment, prefix } => {
                    let text = fragment.emit();
                    for line in markdown_lines(&text) {
                        match prefix {
                            Prefix::Indent(indent) => {
                                if !line.body.is_empty() {
                                    output.extend(std::iter::repeat_n(' ', *indent));
                                }
                            }
                            Prefix::Quote(indent) => {
                                output.push_str(indent);
                                output.push('>');
                                if !line.body.is_empty() {
                                    output.push(' ');
                                }
                            }
                            Prefix::Footnote { indent, .. } => {
                                if !line.body.is_empty() {
                                    output.push_str(indent);
                                }
                            }
                        }
                        output.push_str(line.body);
                        if let Prefix::Footnote { newline, .. } = prefix
                            && !line.body.is_empty()
                            && line.newline.is_empty()
                        {
                            output.push_str(newline);
                        } else {
                            output.push_str(line.newline);
                        }
                    }
                }
            }
        }
        if self.trim_final_newline {
            let newline_len = strip_final_newline(&output[start..]).1.len();
            output.truncate(output.len() - newline_len);
        }
    }
}

impl InlinePlan {
    fn emit(&self, source: &str, output: &mut String) {
        let text = self.text.get(source);
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 && !self.terminated {
                output.push_str(self.newline);
            }
            output.push_str(if index == 0 {
                self.first_prefix.get(source)
            } else {
                self.continuation_prefix.get(source)
            });
            if index == 0 && self.escape_first {
                output.push('\\');
            }
            for (index, token) in self.tokens[line.tokens.clone()].iter().enumerate() {
                if index > 0 {
                    output.push(' ');
                }
                output.push_str(token.get(text));
            }
            output.push_str(line.suffix.map_or("", MarkdownHardBreakMarker::suffix));
            if self.terminated {
                output.push_str(self.newline);
            }
        }
        output.push_str(self.final_newline);
    }
}
/// Option-independent results of recognition and inline normalization. Resolution
/// performs layout after file-scoped directives have established effective options.
#[derive(Debug, Clone)]
pub(crate) struct Draft {
    body: DraftBody,
    trim_final_newline: bool,
    paragraph_single_line: Option<bool>,
}

#[derive(Debug, Clone)]
enum DraftBody {
    Items(Vec<DraftItem>),
    Choice(Condition, Box<[Option<Draft>; 2]>),
    Fallback(Box<[Option<Draft>; 2]>),
}

#[derive(Debug, Clone)]
enum DraftItem {
    Unwrapped {
        validation: Text,
        spaces: Text,
        // Unresolved, unsupported, or retained normalization, respectively.
        normalized: Option<Option<NormalizedText>>,
    },
    JoinNewline(Box<str>),
    Text(Text),
    Inline(InlineDraft),
    Scoped {
        text: Text,
        draft: Box<Draft>,
    },
    Child {
        input: FragmentInput,
        prefix: Prefix,
        reduction: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Condition {
    WrapNone,
    FormatFootnotes,
}

#[derive(Debug, Clone)]
pub(super) enum FragmentInput {
    Buffer(std::sync::Arc<SourceBuffer>),
    Lines { lines: Vec<Text>, newline: Box<str> },
}

#[derive(Debug, Clone)]
struct InlineDraft {
    plain: TokenInput,
    canonical: Option<Box<TokenInput>>,
    first_prefix: Text,
    continuation_prefix: Text,
    newline: &'static str,
    default_newline: bool,
    terminated: bool,
    suffix: &'static str,
    paragraph: bool,
    normalized: bool,
}

#[derive(Debug, Clone)]
struct NormalizedText {
    plain: Text,
    canonical: Option<Text>,
}

#[derive(Debug, Clone)]
struct TokenInput {
    text: Text,
    // Unmeasured, unsupported, or retained measurements for the selected policy.
    tokens: Option<Option<Vec<(SourceSpan, usize)>>>,
}

impl TokenInput {
    fn prepare(source: &str, text: &str) -> Self {
        Self {
            text: Text::retain(source, text),
            tokens: None,
        }
    }
}

impl Draft {
    pub(super) fn new() -> Self {
        Self {
            body: DraftBody::Items(Vec::with_capacity(1)),
            trim_final_newline: false,
            paragraph_single_line: None,
        }
    }

    fn items_mut(&mut self) -> &mut Vec<DraftItem> {
        let DraftBody::Items(items) = &mut self.body else {
            unreachable!("only segment sequences are appended")
        };
        items
    }

    pub(super) fn text(source: &str, text: &str) -> Self {
        let mut draft = Self::new();
        draft.push_str(source, text);
        draft
    }

    pub(super) fn unwrapped(source: &str, validation: &str, spaces: &str) -> Self {
        let mut draft = Self::new();
        draft.items_mut().push(DraftItem::Unwrapped {
            validation: Text::retain(source, validation),
            spaces: Text::retain(source, spaces),
            normalized: None,
        });
        draft
    }

    pub(super) fn push_str(&mut self, source: &str, text: &str) {
        if !text.is_empty() {
            self.items_mut()
                .push(DraftItem::Text(Text::retain(source, text)));
        }
    }

    pub(super) fn append(&mut self, mut draft: Self) {
        assert!(!draft.trim_final_newline && draft.paragraph_single_line.is_none());
        self.items_mut().append(draft.items_mut());
    }

    pub(super) fn push_scoped(&mut self, source: &str, text: &str, draft: Self) {
        self.items_mut().push(DraftItem::Scoped {
            text: Text::retain(source, text),
            draft: Box::new(draft),
        });
    }

    pub(super) fn trim_final_newline(&mut self) {
        self.trim_final_newline = true;
    }

    pub(super) fn push_join_newline(&mut self, newline: &str) {
        self.items_mut()
            .push(DraftItem::JoinNewline(newline.into()));
    }

    pub(super) fn paragraph(&mut self, single_line: bool) {
        self.paragraph_single_line = Some(single_line);
    }

    pub(super) fn choice(condition: Condition, yes: Option<Self>, no: Option<Self>) -> Self {
        Self {
            body: DraftBody::Choice(condition, Box::new([yes, no])),
            trim_final_newline: false,
            paragraph_single_line: None,
        }
    }

    pub(super) fn fallback(first: Option<Self>, second: Option<Self>) -> Self {
        Self {
            body: DraftBody::Fallback(Box::new([first, second])),
            trim_final_newline: false,
            paragraph_single_line: None,
        }
    }

    pub(super) fn child(&mut self, input: FragmentInput, prefix: Prefix, reduction: usize) {
        self.items_mut().push(DraftItem::Child {
            input,
            prefix,
            reduction,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn inline(
        source: &str,
        text: &str,
        first_prefix: &str,
        continuation_prefix: &str,
        newline: &str,
        default_newline: bool,
        terminated: bool,
        suffix: &'static str,
        paragraph: bool,
    ) -> Self {
        let mut draft = Self::new();
        draft.items_mut().push(DraftItem::Inline(InlineDraft {
            plain: TokenInput::prepare(source, text),
            canonical: None,
            first_prefix: Text::retain(source, first_prefix),
            continuation_prefix: Text::retain(source, continuation_prefix),
            newline: match newline {
                "\r\n" => "\r\n",
                "\r" => "\r",
                "\n" => "\n",
                "" => "",
                _ => unreachable!("line ending"),
            },
            default_newline,
            terminated,
            suffix,
            paragraph,
            normalized: !paragraph,
        }));
        draft
    }

    pub(crate) fn resolve(&mut self, source: &str, options: FormatOptions) -> Option<Plan> {
        let items = match &mut self.body {
            DraftBody::Choice(condition, branches) => {
                let chosen = match condition {
                    Condition::WrapNone => matches!(options.markdown_wrap, MarkdownWrap::None),
                    Condition::FormatFootnotes => options.markdown_format_footnotes,
                };
                let [yes, no] = branches.as_mut();
                return if chosen { yes } else { no }
                    .as_mut()?
                    .resolve(source, options);
            }
            DraftBody::Fallback(branches) => {
                let [first, second] = branches.as_mut();
                return first
                    .as_mut()
                    .and_then(|draft| draft.resolve(source, options))
                    .or_else(|| {
                        second
                            .as_mut()
                            .and_then(|draft| draft.resolve(source, options))
                    });
            }
            DraftBody::Items(items) => items,
        };
        // Each retained draft item produces at most one execution item. Reserve
        // its known size directly instead of growing oversized enum storage.
        let mut plan = Plan {
            items: Vec::with_capacity(items.len()),
            trim_final_newline: false,
        };
        for item in items {
            match item {
                DraftItem::Unwrapped {
                    validation,
                    spaces,
                    normalized,
                } => {
                    if normalized.is_none() {
                        *normalized = Some(inline_tokens(validation.get(source)).map(|_| {
                            let raw = spaces.get(source);
                            let spaces =
                                if contains_existing_split_link_destination(validation.get(source))
                                    || markdown_lines(raw)
                                        .all(|line| !line.body.ends_with([' ', '\t']))
                                {
                                    Cow::Borrowed(raw)
                                } else {
                                    Cow::Owned(normalize_inline_whitespace_preserving_lines(raw))
                                };
                            let text = normalize_supported_links_and_images(&spaces);
                            NormalizedText {
                                plain: Text::retain(source, &text),
                                canonical: None,
                            }
                        }));
                    }
                    let normalized = normalized.as_mut()?.as_mut()?;
                    if options.markdown_canonical && normalized.canonical.is_none() {
                        let text = normalized.plain.get(source);
                        if text.contains('_') {
                            let canonical = canonicalize_inline(text);
                            if canonical != text {
                                normalized.canonical = Some(Text::retain(source, &canonical));
                            }
                        }
                    }
                    let text = if options.markdown_canonical {
                        normalized.canonical.as_ref().unwrap_or(&normalized.plain)
                    } else {
                        &normalized.plain
                    };
                    plan.items.push(Item::Text(text.clone()));
                }
                DraftItem::JoinNewline(newline) => {
                    plan.push_str(source, newline_for_join(newline, options))
                }
                DraftItem::Text(text) => plan.items.push(Item::Text(text.clone())),
                DraftItem::Inline(inline) => {
                    if !inline.normalized {
                        let spaces = normalize_spaces_preserving_protected_spans(
                            inline.plain.text.get(source),
                        );
                        let text = normalize_supported_links_and_images(&spaces);
                        inline.plain.text = Text::retain(source, &text);
                        inline.normalized = true;
                    }
                    if inline.paragraph && inline.plain.text.get(source).is_empty() {
                        return Some(Plan::normalized_block(source, source));
                    }
                    if options.markdown_canonical && inline.canonical.is_none() {
                        let text = inline.plain.text.get(source);
                        if text.contains('_') {
                            let canonical = canonicalize_inline(text);
                            if canonical != text {
                                inline.canonical =
                                    Some(Box::new(TokenInput::prepare(source, &canonical)));
                            }
                        }
                    }
                    let input = if options.markdown_canonical {
                        inline.canonical.as_deref_mut().unwrap_or(&mut inline.plain)
                    } else {
                        &mut inline.plain
                    };
                    let text = input.text.get(source);
                    if input.tokens.is_none() {
                        input.tokens = Some(inline_tokens(text));
                    }
                    let tokens = TokenSlice {
                        text,
                        measured: input.tokens.as_ref()?.as_ref()?,
                    };
                    let mut lines = Vec::new();
                    let mut planned_tokens = Vec::new();
                    let mut writer = TokenLineWriter::new(
                        &mut lines,
                        &mut planned_tokens,
                        text,
                        inline.first_prefix.get(source),
                        inline.continuation_prefix.get(source),
                    );
                    write_markdown_token_lines(
                        &mut writer,
                        tokens,
                        options.markdown_wrap,
                        inline.suffix,
                    )?;
                    plan.items.push(Item::Inline(InlinePlan {
                        text: input.text.clone(),
                        lines,
                        tokens: planned_tokens,
                        first_prefix: inline.first_prefix.clone(),
                        continuation_prefix: inline.continuation_prefix.clone(),
                        newline: if inline.default_newline {
                            newline_for_join(inline.newline, options)
                        } else {
                            inline.newline
                        },
                        terminated: inline.terminated,
                        final_newline: if inline.paragraph { inline.newline } else { "" },
                        escape_first: false,
                    }));
                }
                DraftItem::Scoped { text, draft } => {
                    let nested = draft.resolve(text.get(source), options)?;
                    plan.items.push(Item::Scoped {
                        text: text.clone(),
                        plan: Box::new(nested),
                    });
                }
                DraftItem::Child {
                    input,
                    prefix,
                    reduction,
                } => {
                    let mut child_options = options;
                    child_options.markdown_wrap =
                        options.markdown_wrap.with_reduced_column_width(*reduction);
                    let buffer = match input {
                        FragmentInput::Buffer(buffer) => buffer.clone(),
                        FragmentInput::Lines { lines, newline } => {
                            let newline = newline_for_join(newline, options);
                            let mut text = String::new();
                            for line in lines {
                                text.push_str(line.get(source));
                                text.push_str(newline);
                            }
                            std::sync::Arc::new(SourceBuffer::new(text))
                        }
                    };
                    let fragment = Box::new(Fragment::from_buffer(buffer, child_options));
                    let mut prefix = prefix.clone();
                    if let Prefix::Footnote { newline, .. } = &mut prefix {
                        *newline = newline_for_join(newline, options).into();
                    }
                    plan.items.push(Item::Child { fragment, prefix });
                }
            }
        }
        plan.trim_final_newline = self.trim_final_newline;
        if let Some(single_line) = self.paragraph_single_line {
            plan.plan_paragraph_block_starts(source, single_line)?;
        }
        Some(plan)
    }
}
