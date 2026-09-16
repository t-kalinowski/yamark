//! Retained results of the existing paragraph/container helpers, not a new
//! Markdown grammar. Drafts own normalization alternatives, token spans and
//! widths, container boundaries, and prefix policy. Resolution records concrete
//! line breaks and child documents; emission only executes those records.
//!
//! Recognition and the existing line-safety/link-splitting scans remain in
//! planning. A changed child policy can require parsing its retained logical
//! buffer again: explicit target rejection and front-matter handling depend on
//! the child's initial options. This happens before execution, never while
//! restoring container prefixes. Child output is not cached, so skip flags and
//! external formatter execution still belong to document emission.

use super::*;
use crate::core::document::Document;
use crate::core::source::{SourceBuffer, SourceSpan, Span};

/// Text borrowed from the block's source, or a normalization that needs storage.
#[derive(Debug, Clone)]
pub(crate) enum Text {
    Source(SourceSpan),
    Owned(std::sync::Arc<str>),
}

impl Text {
    pub(super) fn retain(source: &str, text: &str) -> Self {
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
    pub first_prefix: Box<str>,
    pub continuation_prefix: Box<str>,
    pub newline: Box<str>,
    pub terminated: bool,
    pub escape_first: bool,
}

#[derive(Debug, Clone)]
pub(super) struct PlannedLine {
    pub tokens: std::ops::Range<usize>,
    pub suffix: Box<str>,
}

#[derive(Debug, Clone)]
pub(super) enum Prefix {
    Indent(usize),
    Quote(Box<str>),
    Footnote { indent: Box<str>, newline: Box<str> },
}

/// Logical container text and its document have the same explicit owner.
/// Fragment parsing intentionally uses the historical default configuration.
#[derive(Debug, Clone)]
pub(crate) struct Fragment {
    source: std::sync::Arc<SourceBuffer>,
    document: Option<Box<Document>>,
    options: FormatOptions,
}

impl Fragment {
    pub(crate) fn plan(source: String, options: FormatOptions) -> Self {
        Self::from_buffer(std::sync::Arc::new(SourceBuffer::new(source)), options)
    }

    fn from_buffer(source: std::sync::Arc<SourceBuffer>, options: FormatOptions) -> Self {
        let range = Span::new(0, source.as_str().len());
        let document = crate::core::markdown::parse_markdown(
            &source,
            range,
            options,
            &crate::config::Config::default(),
        )
        .ok()
        .map(Box::new);
        Self {
            source,
            document,
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
        self.document
            .as_ref()
            .and_then(|document| {
                crate::core::emit::emit_document(
                    &self.source,
                    document,
                    self.options,
                    &crate::plugins::PluginRegistry::default(),
                )
                .ok()
            })
            .unwrap_or_else(|| self.source.as_str().to_owned())
    }
}

impl Plan {
    pub(super) fn new() -> Self {
        Self {
            items: Vec::new(),
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
        let mut candidate = String::with_capacity(source.len().min(256));
        for item in &mut self.items {
            let Item::Inline(inline) = item else { continue };
            let text = inline.text.get(source);
            for line in &inline.lines {
                candidate.clear();
                for (index, token) in inline.tokens[line.tokens.clone()].iter().enumerate() {
                    if index > 0 {
                        candidate.push(' ');
                    }
                    candidate.push_str(token.get(text));
                }
                candidate.push_str(&line.suffix);
                for body in markdown_line_bodies(&candidate) {
                    if first {
                        inline.escape_first = markdown_block_start_line(body);
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
        for item in &self.items {
            match item {
                Item::Text(text) => output.push_str(text.get(source)),
                Item::Inline(inline) => inline.emit(source, &mut output),
                Item::Scoped { text, plan } => output.push_str(&plan.emit(text.get(source))),
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
            trim_trailing_line_ending(&mut output);
        }
        output
    }
}

impl InlinePlan {
    fn emit(&self, source: &str, output: &mut String) {
        let text = self.text.get(source);
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 && !self.terminated {
                output.push_str(&self.newline);
            }
            output.push_str(if index == 0 {
                &self.first_prefix
            } else {
                &self.continuation_prefix
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
            output.push_str(&line.suffix);
            if self.terminated {
                output.push_str(&self.newline);
            }
        }
    }
}
/// Option-independent results of recognition and inline normalization. Resolution
/// performs layout after file-scoped directives have established effective options.
#[derive(Debug, Clone)]
pub(crate) struct Draft {
    items: Vec<DraftItem>,
    trim_final_newline: bool,
    paragraph_single_line: Option<bool>,
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
    Text {
        plain: Text,
        canonical: Option<Text>,
    },
    Inline(InlineDraft),
    Scoped {
        text: Text,
        draft: Box<Draft>,
    },
    Choice {
        condition: Condition,
        yes: Option<Box<Draft>>,
        no: Option<Box<Draft>>,
    },
    Fallback {
        first: Option<Box<Draft>>,
        second: Option<Box<Draft>>,
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
    canonical: Option<TokenInput>,
    first_prefix: Box<str>,
    continuation_prefix: Box<str>,
    newline: Box<str>,
    default_newline: bool,
    terminated: bool,
    suffix: &'static str,
}

#[derive(Debug, Clone)]
struct NormalizedText {
    plain: Text,
    canonical: Option<Text>,
}

#[derive(Debug, Clone)]
struct TokenInput {
    text: Text,
    tokens: Option<Vec<(SourceSpan, usize)>>,
}

impl TokenInput {
    fn prepare(source: &str, text: &str) -> Self {
        let tokens = inline_tokens(text).map(|tokens| {
            tokens
                .iter()
                .map(|token| {
                    let start = token.text().as_ptr() as usize - text.as_ptr() as usize;
                    (
                        SourceSpan::new(Span::new(start, start + token.text().len())),
                        token.width,
                    )
                })
                .collect()
        });
        Self {
            text: Text::retain(source, text),
            tokens,
        }
    }
}

impl Draft {
    pub(super) fn new() -> Self {
        Self {
            items: Vec::new(),
            trim_final_newline: false,
            paragraph_single_line: None,
        }
    }

    pub(super) fn text(source: &str, text: &str) -> Self {
        let mut draft = Self::new();
        draft.push_str(source, text);
        draft
    }

    pub(super) fn unwrapped(source: &str, validation: &str, spaces: &str) -> Self {
        let mut draft = Self::new();
        draft.items.push(DraftItem::Unwrapped {
            validation: Text::retain(source, validation),
            spaces: Text::retain(source, spaces),
            normalized: None,
        });
        draft
    }

    pub(super) fn push_str(&mut self, source: &str, text: &str) {
        if !text.is_empty() {
            self.items.push(DraftItem::Text {
                plain: Text::retain(source, text),
                canonical: None,
            });
        }
    }

    pub(super) fn append(&mut self, mut draft: Self) {
        assert!(!draft.trim_final_newline && draft.paragraph_single_line.is_none());
        self.items.append(&mut draft.items);
    }

    pub(super) fn push_scoped(&mut self, source: &str, text: &str, draft: Self) {
        self.items.push(DraftItem::Scoped {
            text: Text::retain(source, text),
            draft: Box::new(draft),
        });
    }

    pub(super) fn trim_final_newline(&mut self) {
        self.trim_final_newline = true;
    }

    pub(super) fn push_join_newline(&mut self, newline: &str) {
        self.items.push(DraftItem::JoinNewline(newline.into()));
    }

    pub(super) fn paragraph(&mut self, single_line: bool) {
        self.paragraph_single_line = Some(single_line);
    }

    pub(super) fn choice(condition: Condition, yes: Option<Self>, no: Option<Self>) -> Self {
        let mut draft = Self::new();
        draft.items.push(DraftItem::Choice {
            condition,
            yes: yes.map(Box::new),
            no: no.map(Box::new),
        });
        draft
    }

    pub(super) fn fallback(first: Option<Self>, second: Option<Self>) -> Self {
        let mut draft = Self::new();
        draft.items.push(DraftItem::Fallback {
            first: first.map(Box::new),
            second: second.map(Box::new),
        });
        draft
    }

    pub(super) fn child(&mut self, input: FragmentInput, prefix: Prefix, reduction: usize) {
        self.items.push(DraftItem::Child {
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
    ) -> Self {
        let canonical = text
            .contains('_')
            .then(|| canonicalize_inline(text))
            .filter(|canonical| canonical != text);
        let mut draft = Self::new();
        draft.items.push(DraftItem::Inline(InlineDraft {
            plain: TokenInput::prepare(source, text),
            canonical: canonical
                .as_deref()
                .map(|canonical| TokenInput::prepare(source, canonical)),
            first_prefix: first_prefix.into(),
            continuation_prefix: continuation_prefix.into(),
            newline: newline.into(),
            default_newline,
            terminated,
            suffix,
        }));
        draft
    }

    pub(crate) fn resolve(&mut self, source: &str, options: FormatOptions) -> Option<Plan> {
        if !self.trim_final_newline && self.paragraph_single_line.is_none() {
            match self.items.as_mut_slice() {
                [DraftItem::Choice { condition, yes, no }] => {
                    let chosen = match condition {
                        Condition::WrapNone => matches!(options.markdown_wrap, MarkdownWrap::None),
                        Condition::FormatFootnotes => options.markdown_format_footnotes,
                    };
                    return if chosen { yes } else { no }
                        .as_mut()?
                        .resolve(source, options);
                }
                [DraftItem::Fallback { first, second }] => {
                    return first
                        .as_mut()
                        .and_then(|draft| draft.resolve(source, options))
                        .or_else(|| {
                            second
                                .as_mut()
                                .and_then(|draft| draft.resolve(source, options))
                        });
                }
                _ => {}
            }
        }
        let mut plan = Plan::new();
        for item in &mut self.items {
            match item {
                DraftItem::Unwrapped {
                    validation,
                    spaces,
                    normalized,
                } => {
                    if normalized.is_none() {
                        *normalized = Some(inline_tokens(validation.get(source)).map(|_| {
                            let text = normalize_supported_links_and_images(spaces.get(source));
                            let canonical = text
                                .contains('_')
                                .then(|| canonicalize_inline(&text))
                                .filter(|canonical| canonical != &text);
                            NormalizedText {
                                plain: Text::retain(source, &text),
                                canonical: canonical
                                    .as_deref()
                                    .map(|text| Text::retain(source, text)),
                            }
                        }));
                    }
                    let normalized = normalized.as_ref()?.as_ref()?;
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
                DraftItem::Text { plain, canonical } => {
                    plan.items.push(Item::Text(
                        if options.markdown_canonical {
                            canonical.as_ref().unwrap_or(plain)
                        } else {
                            plain
                        }
                        .clone(),
                    ));
                }
                DraftItem::Inline(inline) => {
                    let input = if options.markdown_canonical {
                        inline.canonical.as_ref().unwrap_or(&inline.plain)
                    } else {
                        &inline.plain
                    };
                    let text = input.text.get(source);
                    let tokens = input
                        .tokens
                        .as_ref()?
                        .iter()
                        .map(|(span, width)| InlineToken {
                            text: Cow::Borrowed(span.span().slice(text)),
                            width: *width,
                        })
                        .collect::<Vec<_>>();
                    let mut lines = Vec::new();
                    let mut planned_tokens = Vec::with_capacity(tokens.len());
                    let mut writer = TokenLineWriter::new(
                        &mut lines,
                        &mut planned_tokens,
                        text,
                        &inline.first_prefix,
                        &inline.continuation_prefix,
                    );
                    write_markdown_token_lines(
                        &mut writer,
                        &tokens,
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
                            newline_for_join(&inline.newline, options).into()
                        } else {
                            inline.newline.clone()
                        },
                        terminated: inline.terminated,
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
                DraftItem::Choice { condition, yes, no } => {
                    let chosen = match condition {
                        Condition::WrapNone => matches!(options.markdown_wrap, MarkdownWrap::None),
                        Condition::FormatFootnotes => options.markdown_format_footnotes,
                    };
                    let draft = if chosen { yes } else { no }.as_mut()?;
                    plan.items.push(Item::Scoped {
                        text: Text::retain(source, source),
                        plan: Box::new(draft.resolve(source, options)?),
                    });
                }
                DraftItem::Fallback { first, second } => {
                    let nested = first
                        .as_mut()
                        .and_then(|draft| draft.resolve(source, options))
                        .or_else(|| {
                            second
                                .as_mut()
                                .and_then(|draft| draft.resolve(source, options))
                        })?;
                    plan.items.push(Item::Scoped {
                        text: Text::retain(source, source),
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
