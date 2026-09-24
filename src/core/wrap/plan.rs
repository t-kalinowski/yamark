//! Retained results of the existing paragraph/container helpers, not a new
//! Markdown grammar. Drafts retain source boundaries and container prefix
//! policy. Resolution normalizes for the effective options and measures tokens
//! only for wrapping. Emission executes the retained lines and literal ranges.
//!
//! Recognition and the existing line-safety/link-splitting scans remain in
//! planning. A changed child policy can require parsing its retained logical
//! buffer again: explicit target rejection and front-matter handling depend on
//! the child's initial options. This happens before execution, never while
//! restoring container prefixes. Child output is not cached, so skip flags and
//! external formatter execution still belong to document emission.

use super::*;
use crate::core::document::{DocumentEmitMode, EmitPlan, PreparedTree};
use crate::core::emit::EmittedText;
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
    Unwrapped(UnwrappedPlan),
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
    literals: Box<[Span]>,
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
pub(super) struct UnwrappedPlan {
    text: Text,
    literals: Box<[Span]>,
    first_prefix: Text,
    continuation_prefix: Text,
    final_newline: &'static str,
    escape_first: bool,
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
/// Fragment parsing inherits template delimiters; other configuration keeps
/// the historical defaults.
#[derive(Debug, Clone)]
pub(crate) struct Fragment {
    source: std::sync::Arc<SourceBuffer>,
    body: Option<FragmentBody>,
    options: FormatOptions,
    template_delimiters: Vec<TemplateDelimiter>,
}

#[derive(Debug, Clone)]
enum FragmentBody {
    Plan(Box<Plan>),
    Document(Box<PreparedTree>),
}

impl FragmentBody {
    fn retain(source: &SourceBuffer, tree: PreparedTree) -> Self {
        let document = tree.document();
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
            return Self::Document(Box::new(tree));
        }
        let options = tree.options();
        let mut document = tree.into_document();
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
        Self::from_buffer(
            std::sync::Arc::new(SourceBuffer::new(source)),
            options,
            &crate::config::Config::default().template_delimiters,
        )
    }

    fn from_buffer(
        source: std::sync::Arc<SourceBuffer>,
        options: FormatOptions,
        delimiters: &[TemplateDelimiter],
    ) -> Self {
        let config = crate::config::Config {
            template_delimiters: delimiters.to_vec(),
            ..crate::config::Config::default()
        };
        let range = Span::new(0, source.as_str().len());
        let body = crate::core::markdown::parse_markdown(&source, range, options, &config)
            .ok()
            .map(|document| {
                let tree =
                    PreparedTree::new(&source, document, options, DocumentEmitMode::Document);
                FragmentBody::retain(&source, tree)
            });
        Self {
            source,
            body,
            options,
            template_delimiters: delimiters.to_vec(),
        }
    }

    pub(crate) fn resolve_options(&mut self, options: FormatOptions) {
        if self.options != options {
            *self = Self::from_buffer(self.source.clone(), options, &self.template_delimiters);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.source.as_str().is_empty()
    }

    pub(crate) fn emit(&self) -> String {
        self.emit_protected().text
    }

    fn emit_protected(&self) -> EmittedText {
        self.body
            .as_ref()
            .and_then(|body| match body {
                FragmentBody::Plan(plan) => Some(plan.emit_protected(self.source.as_str())),
                FragmentBody::Document(tree) => {
                    crate::core::emit::emit_fragment(&self.source, tree).ok()
                }
            })
            .unwrap_or_else(|| EmittedText {
                text: self.source.as_str().to_owned(),
                verbatim: Vec::new(),
                preserve_eof_at: 0,
            })
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
        self.emit_protected(source).text
    }

    pub(crate) fn emit_protected(&self, source: &str) -> EmittedText {
        let mut output = EmittedText {
            text: String::with_capacity(source.len()),
            verbatim: Vec::new(),
            preserve_eof_at: 0,
        };
        self.emit_into(source, &mut output);
        output
    }

    fn emit_into(&self, source: &str, output: &mut EmittedText) {
        let start = output.text.len();
        for item in &self.items {
            match item {
                Item::Text(text) => output.text.push_str(text.get(source)),
                Item::Inline(inline) => inline.emit(source, output),
                Item::Unwrapped(inline) => {
                    output.text.push_str(inline.first_prefix.get(source));
                    if inline.escape_first {
                        output.text.push('\\');
                    }
                    let text = inline.text.get(source);
                    emit_prefixed_slice(
                        output,
                        text,
                        &inline.literals,
                        Span::new(0, text.len()),
                        inline.continuation_prefix.get(source),
                    );
                    output.text.push_str(inline.final_newline);
                }
                Item::Scoped { text, plan } => plan.emit_into(text.get(source), output),
                Item::Child { fragment, prefix } => {
                    let text = fragment.emit_protected();
                    for line in markdown_lines(&text.text) {
                        match prefix {
                            Prefix::Indent(indent) => {
                                if !line.body.is_empty() {
                                    output.text.extend(std::iter::repeat_n(' ', *indent));
                                }
                            }
                            Prefix::Quote(indent) => {
                                output.text.push_str(indent);
                                output.text.push('>');
                                if !line.body.is_empty() {
                                    output.text.push(' ');
                                }
                            }
                            Prefix::Footnote { indent, .. } => {
                                if !line.body.is_empty() {
                                    output.text.push_str(indent);
                                }
                            }
                        }
                        output.push_slice(
                            &text.text,
                            &text.verbatim,
                            Span::new(line.body_start, line.body_start + line.full.len()),
                        );
                        if text.preserve_eof_at == text.text.len()
                            && line.body_start + line.full.len() == text.text.len()
                        {
                            output.preserve_eof_at = output.text.len();
                        }
                        if let Prefix::Footnote { newline, .. } = prefix
                            && !line.body.is_empty()
                            && line.newline.is_empty()
                        {
                            output.text.push_str(newline);
                        }
                    }
                }
            }
        }
        if self.trim_final_newline {
            let newline_len = strip_final_newline(&output.text[start..]).1.len();
            output.text.truncate(output.text.len() - newline_len);
            output.verbatim.retain_mut(|span| {
                span.end = span.end.min(output.text.len());
                span.start < span.end
            });
        }
    }
}

// Prefix restoration is an output-coordinate operation, not inline recognition.
// Map only the retained literal ranges; editable gaps remain eligible for cleanup.
fn emit_prefixed_slice(
    output: &mut EmittedText,
    text: &str,
    literals: &[Span],
    slice: Span,
    prefix: &str,
) {
    if prefix.is_empty() {
        output.push_slice(text, literals, slice);
        return;
    }
    for (index, line) in markdown_lines(slice.slice(text)).enumerate() {
        if index > 0 {
            output.text.push_str(prefix);
        }
        let start = slice.start + line.body_start;
        output.push_slice(text, literals, Span::new(start, start + line.full.len()));
    }
}

impl InlinePlan {
    fn emit(&self, source: &str, output: &mut EmittedText) {
        let text = self.text.get(source);
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 && !self.terminated {
                output.text.push_str(self.newline);
            }
            output.text.push_str(if index == 0 {
                self.first_prefix.get(source)
            } else {
                self.continuation_prefix.get(source)
            });
            if index == 0 && self.escape_first {
                output.text.push('\\');
            }
            for (index, token) in self.tokens[line.tokens.clone()].iter().enumerate() {
                if index > 0 {
                    output.text.push(' ');
                }
                match token {
                    Text::Source(span) => emit_prefixed_slice(
                        output,
                        text,
                        &self.literals,
                        span.span(),
                        self.continuation_prefix.get(source),
                    ),
                    Text::Owned(text) => output.text.push_str(text),
                }
            }
            output
                .text
                .push_str(line.suffix.map_or("", MarkdownHardBreakMarker::suffix));
            if self.terminated {
                output.text.push_str(self.newline);
            }
        }
        output.text.push_str(self.final_newline);
    }
}

/// Option-independent source and recognition facts. Resolution materializes only
/// the normalization and layout requested by the current effective options.
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
}

#[derive(Debug, Clone)]
struct InlineDraft {
    source: Text,
    // Unprepared, recognized, or rejected; only the selected layout is prepared.
    recognition: Option<Option<InlineSource>>,
    first_prefix: Text,
    continuation_prefix: Text,
    newline: &'static str,
    default_newline: bool,
    terminated: bool,
    preserve_lines: bool,
    paragraph: bool,
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
        preserve_lines: bool,
        paragraph: bool,
    ) -> Self {
        let mut draft = Self::new();
        draft.items_mut().push(DraftItem::Inline(InlineDraft {
            source: Text::retain(source, text),
            recognition: None,
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
            preserve_lines,
            paragraph,
        }));
        draft
    }

    pub(crate) fn resolve(&mut self, source: &str, options: FormatOptions) -> Option<Plan> {
        self.resolve_with_templates(
            source,
            options,
            &crate::config::Config::default().template_delimiters,
        )
    }

    pub(crate) fn resolve_with_templates(
        &mut self,
        source: &str,
        options: FormatOptions,
        delimiters: &[TemplateDelimiter],
    ) -> Option<Plan> {
        let items = match &mut self.body {
            DraftBody::Choice(condition, branches) => {
                let chosen = match condition {
                    Condition::WrapNone => matches!(options.markdown_wrap, MarkdownWrap::None),
                    Condition::FormatFootnotes => options.markdown_format_footnotes,
                };
                let [yes, no] = branches.as_mut();
                return if chosen { yes } else { no }
                    .as_mut()?
                    .resolve_with_templates(source, options, delimiters);
            }
            DraftBody::Fallback(branches) => {
                let [first, second] = branches.as_mut();
                return first
                    .as_mut()
                    .and_then(|draft| draft.resolve_with_templates(source, options, delimiters))
                    .or_else(|| {
                        second.as_mut().and_then(|draft| {
                            draft.resolve_with_templates(source, options, delimiters)
                        })
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
                DraftItem::JoinNewline(newline) => {
                    plan.push_str(source, newline_for_join(newline, options))
                }
                DraftItem::Text(text) => plan.items.push(Item::Text(text.clone())),
                DraftItem::Inline(inline) => {
                    let unwrapped = matches!(
                        options.markdown_wrap,
                        MarkdownWrap::None | MarkdownWrap::Paragraph
                    );
                    let gaps = if matches!(options.markdown_wrap, MarkdownWrap::None) {
                        if inline.paragraph {
                            Gaps::Preserve
                        } else if inline.preserve_lines {
                            Gaps::Lines
                        } else {
                            Gaps::Reflow
                        }
                    } else {
                        Gaps::Reflow
                    };
                    let input = inline
                        .recognition
                        .get_or_insert_with(|| {
                            InlineSource::recognize_with_templates(
                                inline.source.get(source),
                                delimiters,
                            )
                        })
                        .as_ref()?
                        .normalize(
                            inline.source.get(source),
                            gaps,
                            options.markdown_canonical,
                            !unwrapped,
                        )?;
                    let text = input.text.as_str();
                    if unwrapped {
                        plan.items.push(Item::Unwrapped(UnwrappedPlan {
                            text: Text::retain(source, text),
                            literals: input.literals.into_boxed_slice(),
                            first_prefix: inline.first_prefix.clone(),
                            continuation_prefix: inline.continuation_prefix.clone(),
                            final_newline: if inline.terminated && inline.default_newline {
                                newline_for_join(inline.newline, options)
                            } else {
                                inline.newline
                            },
                            escape_first: !matches!(options.markdown_wrap, MarkdownWrap::None)
                                && inline.paragraph
                                && markdown_block_start_line(text),
                        }));
                        continue;
                    }
                    let mut lines = Vec::new();
                    let mut planned_tokens = Vec::new();
                    let mut writer = TokenLineWriter::new(
                        &mut lines,
                        &mut planned_tokens,
                        text,
                        &input.literals,
                        inline.first_prefix.get(source),
                        inline.continuation_prefix.get(source),
                    );
                    let mut start = 0;
                    for &(end, hard_break) in &input.breaks {
                        write_markdown_token_lines(
                            &mut writer,
                            TokenSlice {
                                text,
                                measured: &input.tokens[start..end],
                            },
                            options.markdown_wrap,
                            hard_break.map_or("", MarkdownHardBreakMarker::suffix),
                        )?;
                        start = end;
                    }
                    plan.items.push(Item::Inline(InlinePlan {
                        text: Text::retain(source, text),
                        literals: input.literals.into_boxed_slice(),
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
                    let nested =
                        draft.resolve_with_templates(text.get(source), options, delimiters)?;
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
                    };
                    let fragment =
                        Box::new(Fragment::from_buffer(buffer, child_options, delimiters));
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
