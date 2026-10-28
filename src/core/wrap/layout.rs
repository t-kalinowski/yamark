//! Measured-token line layout and repairs that prevent a wrap from introducing
//! block syntax or changing inline delimiter meaning.

use super::plan::{PlannedLine, Text};
use super::syntax::{
    InlineScan, LinkTarget, balanced_brace_end, delimiter_run_len_at, emphasis_span_at,
    emphasis_span_at_with_math, emphasis_span_end, escaped_at, find_balanced_square_close,
    find_simple_destination_close, link_or_bracket_token_end, parse_simple_link_target,
    reference_bracket_token_at, render_link_target, split_link_target_lines,
};
use super::{
    MarkdownHardBreakMarker, balanced_brace_span_end, commonmark_autolink_span_end,
    inline_html_tag_has_quoted_gt, inline_html_tag_span_end, literal_block_marker_escape,
    markdown_block_start_line, normalize_attribute_after, normalize_heading_attribute_block,
    normalize_link_label, protected_inline_token_end, token_may_start_markdown_block,
};
use crate::core::document::MarkdownWrap;
use crate::core::source::{SourceSpan, Span};
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct InlineToken<'a> {
    text: Cow<'a, str>,
    width: usize,
    preserve_math_closers: bool,
}

impl<'a> InlineToken<'a> {
    fn owned(text: String, preserve_math_closers: bool) -> Self {
        let width = token_width(&text);
        Self {
            text: Cow::Owned(text),
            width,
            preserve_math_closers,
        }
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn escape_marker(&mut self, offset: usize) {
        self.text.to_mut().insert(offset, '\\');
        self.width += 1;
    }

    fn remove_literal_marker_escape(&mut self, preserve_asterisk_escapes: bool) {
        if preserve_asterisk_escapes && self.text == r"\*" {
            return;
        }
        let Some(offset) = self.text.find('\\') else {
            return;
        };
        let mut candidate = self.text.to_string();
        candidate.remove(offset);
        candidate.push(' ');
        if literal_block_marker_escape(
            &candidate,
            self.preserve_math_closers,
            preserve_asterisk_escapes,
        ) == Some(offset)
        {
            candidate.pop();
            *self = Self::owned(candidate, self.preserve_math_closers);
        }
    }
}

/// A borrowed view of measured tokens. Layout reads it without rebuilding a
/// document-sized vector of temporary inline tokens.
#[derive(Clone, Copy)]
pub(super) struct TokenSlice<'a> {
    pub(super) text: &'a str,
    pub(super) measured: &'a [(SourceSpan, usize)],
}

impl<'a> TokenSlice<'a> {
    fn len(self) -> usize {
        self.measured.len()
    }

    fn slice(self, range: std::ops::Range<usize>) -> Self {
        Self {
            text: self.text,
            measured: &self.measured[range],
        }
    }

    fn iter(self, unmatched_math_opener: Option<usize>) -> impl Iterator<Item = InlineToken<'a>> {
        self.measured.iter().map(move |(span, width)| InlineToken {
            text: Cow::Borrowed(span.span().slice(self.text)),
            width: *width,
            preserve_math_closers: unmatched_math_opener.is_some_and(|open| open < span.start()),
        })
    }
}

fn write_sentence_token_lines(
    out: &mut TokenLineWriter<'_, '_>,
    tokens: TokenSlice<'_>,
    suffix: &'static str,
) -> Option<()> {
    let unmatched_math_opener = out.unmatched_math_opener;
    let preserve_asterisk_escapes = out.preserve_asterisk_escapes;
    write_sentence_token_slices(
        tokens,
        suffix,
        unmatched_math_opener,
        preserve_asterisk_escapes,
        |sentence, sentence_suffix, escape| {
            let mut tokens = sentence.iter(unmatched_math_opener);
            let mut first = tokens.next()?;
            if let Some(offset) = escape {
                first.escape_marker(offset);
            }
            out.record_tokens(
                std::iter::once(first.text()).chain(
                    sentence.measured[1..]
                        .iter()
                        .map(|(span, _)| span.span().slice(sentence.text)),
                ),
                sentence_suffix,
            )
        },
    )
}

fn write_sentence_token_slices<'token>(
    tokens: TokenSlice<'token>,
    suffix: &'static str,
    unmatched_math_opener: Option<usize>,
    preserve_asterisk_escapes: bool,
    mut write: impl FnMut(TokenSlice<'token>, &'static str, Option<usize>) -> Option<()>,
) -> Option<()> {
    let mut safety_line = String::new();
    let mut pending = None::<(usize, usize, Option<usize>)>;
    let mut push_sentence = |from: usize, to: usize| {
        let mut escape = None;
        if let Some((previous_from, previous_to, previous_escape)) = pending {
            let start = tokens.measured[from].0.start();
            let end = tokens.measured[to - 1].0.end();
            safety_line.clear();
            safety_line.push_str(&tokens.text[start..end]);
            // A sentence may be just an ordered marker, such as `2.`. Include
            // the whitespace required by the shared list-marker recognizer.
            safety_line.push(' ');
            escape = literal_block_marker_escape(
                &safety_line,
                unmatched_math_opener.is_some_and(|open| open < start),
                preserve_asterisk_escapes,
            );
            if markdown_block_start_line(&safety_line) && escape.is_none() {
                pending = Some((previous_from, to, previous_escape));
                return Some(());
            }
            write(
                tokens.slice(previous_from..previous_to),
                "",
                previous_escape,
            )?;
        }
        pending = Some((from, to, escape));
        Some(())
    };
    let mut start = 0usize;
    for (index, token) in tokens.iter(unmatched_math_opener).enumerate() {
        if !token_ends_sentence(token.text()) || token_is_sentence_abbreviation(token.text()) {
            continue;
        }
        // A marker such as `2.` at a new sentence boundary will have its
        // period escaped. Keep the following prose in that same sentence.
        if index == start
            && start > 0
            && literal_block_marker_escape(
                &tokens.text[tokens.measured[index].0.start()..],
                token.preserve_math_closers,
                preserve_asterisk_escapes,
            )
            .is_some_and(|offset| offset + 1 == token.text().len())
        {
            continue;
        }
        push_sentence(start, index + 1)?;
        start = index + 1;
    }
    if start < tokens.len() {
        push_sentence(start, tokens.len())?;
    }
    if let Some((from, to, escape)) = pending {
        write(tokens.slice(from..to), suffix, escape)?;
    }
    Some(())
}

fn token_ends_sentence(token: &str) -> bool {
    let token = token.trim_end_matches(['"', '\'', ')', ']', '}']);
    token
        .char_indices()
        .next_back()
        .is_some_and(|(index, ch)| matches!(ch, '.' | '!' | '?') && !escaped_at(token, index))
}

fn token_is_sentence_abbreviation(token: &str) -> bool {
    ascii_ends_with_ignore_case(token, "e.g.") || ascii_ends_with_ignore_case(token, "i.e.")
}

fn ascii_ends_with_ignore_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

#[derive(Debug, Clone, Default)]
struct TokenLine<'a> {
    tokens: Vec<InlineToken<'a>>,
    width: usize,
}

impl<'a> TokenLine<'a> {
    fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn can_fit(&self, token: &InlineToken<'_>, width: usize) -> bool {
        self.is_empty() || self.width + 1 + token.width <= width
    }

    fn push(&mut self, token: InlineToken<'a>) {
        if !self.tokens.is_empty() {
            self.width += 1;
        }
        self.width += token.width;
        self.tokens.push(token);
    }

    fn append(&mut self, other: TokenLine<'a>) {
        for token in other.tokens {
            self.push(token);
        }
    }

    fn extend_from_slice(&mut self, tokens: &[InlineToken<'a>]) {
        for token in tokens {
            self.push(token.clone());
        }
    }

    fn split_off(&mut self, at: usize) -> TokenLine<'a> {
        let moved = self.tokens.split_off(at);
        self.width = token_slice_width(&self.tokens);
        TokenLine {
            width: token_slice_width(&moved),
            tokens: moved,
        }
    }

    fn suffix_width(&self, at: usize) -> usize {
        token_slice_width(&self.tokens[at..])
    }

    fn markdown_block_start(&self, scratch: &mut String, include_marker_only: bool) -> bool {
        token_slice_markdown_block_start(&self.tokens, scratch, include_marker_only)
    }
}

// Interpret punctuation only in prose, reusing the existing opaque boundaries.
// Text inside emphasis, HTML elements and ordinary braces remains visible.
fn inline_wrap_safety(text: &str, literals: &[Span]) -> Option<(Option<usize>, bool)> {
    let check_math = text.contains("\\(");
    let check_asterisks = text.match_indices('*').nth(1).is_some();
    let check_html = text.contains("<!")
        || text.contains("<?")
        || text.contains('<') && text.contains(['"', '\'']);
    if !check_math && !check_asterisks && !check_html {
        return Some((None, false));
    }
    let mut scan = InlineScan::new(text);
    let mut index = 0;
    let mut unmatched_math_opener = None;
    let mut asterisks = 0;
    let mut emphasis_end = 0;
    while let Some(offset) = text[index..].find(['`', '$', '\\', '<', '[', '!', '{', '*']) {
        if !check_html
            && (!check_math || unmatched_math_opener.is_some())
            && (!check_asterisks || asterisks >= 2)
        {
            break;
        }
        index += offset;
        let literal = literals.partition_point(|span| span.end <= index);
        let literal = literals
            .get(literal)
            .filter(|span| span.start <= index)
            .copied()
            .or_else(|| {
                scan.literal_span_end(index)
                    .map(|end| Span::new(index, end))
            });
        if let Some(span) = literal {
            // Math syntax is prose in some supported readers. Its asterisks
            // may pair with prose outside it even though layout keeps it whole.
            if text[span.start..].starts_with('$') || text[span.start..].starts_with("\\(") {
                asterisks += text[span.start..span.end]
                    .match_indices('*')
                    .filter(|(offset, _)| !escaped_at(text, span.start + offset))
                    .take(2 - asterisks)
                    .count();
            }
            index = span.end;
        } else if check_html
            && !escaped_at(text, index)
            && (text[index..].starts_with("<!")
                || text[index..].starts_with("<?")
                || inline_html_tag_has_quoted_gt(text, index))
        {
            // Preserve raw HTML openers, including incomplete or prose uses,
            // and ambiguous tags with a quoted `>`. Their contents cannot
            // safely use Markdown marker unescaping.
            return None;
        } else if !escaped_at(text, index)
            && let Some(end) = commonmark_autolink_span_end(text, index)
                .or_else(|| inline_html_tag_span_end(text, index))
                .or_else(|| match text.as_bytes()[index] {
                    b'[' | b'!'
                        if text[index..].starts_with('[') || text[index..].starts_with("![") =>
                    {
                        let label_start = index + 1 + usize::from(text.as_bytes()[index] == b'!');
                        let end = scan.square_close(label_start)? + 1;
                        if let Some(attribute_end) = balanced_brace_span_end(text, end)
                            && normalize_heading_attribute_block(&text[end..attribute_end])
                                .is_some()
                        {
                            Some(attribute_end)
                        } else if text.as_bytes().get(end) == Some(&b'(')
                            || reference_bracket_token_at(&mut scan, index)
                        {
                            link_or_bracket_token_end(&mut scan, index)
                        } else {
                            // Reference-shaped brackets may be ordinary prose.
                            // Without resolved definitions, scan their contents.
                            None
                        }
                    }
                    b'\\' => protected_inline_token_end(&mut scan, index),
                    b'{' if text[index..].starts_with("{{<")
                        || text[index..].starts_with("{{%") =>
                    {
                        protected_inline_token_end(&mut scan, index)
                    }
                    _ => None,
                })
        {
            index = end;
        } else if text[index..].starts_with("\\(") && !escaped_at(text, index) {
            unmatched_math_opener.get_or_insert(index);
            index += 2;
        } else if text.as_bytes()[index] == b'`' {
            index += delimiter_run_len_at(text, index, b'`');
        } else {
            // Ignore simple matched emphasis, but keep scanning its contents
            // for math and HTML. A preceding loose star can capture a closer
            // after reflow; nested stars also need the conservative guard.
            if asterisks == 0
                && index >= emphasis_end
                && text.as_bytes()[index] == b'*'
                && let Some(span) = emphasis_span_at(&mut scan, index)
                && !text[index + span.run..span.close].contains('*')
            {
                emphasis_end = span.end;
            }
            // Count spaced stars too: Pandoc can pair them after column reflow
            // puts a newline after one. Both modes need the same roundtrip guard.
            if text.as_bytes()[index] == b'*'
                && index >= emphasis_end
                && (!escaped_at(text, index)
                    // Column reflow may unescape a standalone `\*`, including
                    // one before a compact hard break. Count it in both modes
                    // for roundtrips, but not punctuation such as `\*.`.
                    || (index == 1 || text.as_bytes()[index - 2].is_ascii_whitespace())
                        && (text.as_bytes().get(index + 1).is_none_or(u8::is_ascii_whitespace)
                            || text[index + 1..]
                                .strip_prefix('\\')
                                .is_some_and(|rest| rest.starts_with(['\r', '\n']))))
            {
                asterisks = (asterisks + 1).min(2);
            }
            index += 1;
        }
    }
    Some((unmatched_math_opener, asterisks >= 2))
}

pub(super) struct TokenLineWriter<'out, 'prefix> {
    out: &'out mut Vec<PlannedLine>,
    tokens: &'out mut Vec<Text>,
    text: &'prefix str,
    literals: &'prefix [Span],
    first_prefix: &'prefix str,
    continuation_prefix: &'prefix str,
    unmatched_math_opener: Option<usize>,
    preserve_asterisk_escapes: bool,
    check_block_starts: bool,
    safety_line: String,
    line_count: usize,
}

impl<'out, 'prefix> TokenLineWriter<'out, 'prefix> {
    pub(super) fn new(
        out: &'out mut Vec<PlannedLine>,
        tokens: &'out mut Vec<Text>,
        text: &'prefix str,
        literals: &'prefix [Span],
        first_prefix: &'prefix str,
        continuation_prefix: &'prefix str,
    ) -> Option<Self> {
        let (unmatched_math_opener, preserve_asterisk_escapes) =
            inline_wrap_safety(text, literals)?;
        Some(Self {
            out,
            tokens,
            text,
            literals,
            first_prefix,
            continuation_prefix,
            unmatched_math_opener,
            preserve_asterisk_escapes,
            check_block_starts: false,
            safety_line: String::new(),
            line_count: 0,
        })
    }

    fn overlaps_literal(&self, token: &str) -> bool {
        let start = token.as_ptr() as usize - self.text.as_ptr() as usize;
        let end = start + token.len();
        let index = self.literals.partition_point(|span| span.end <= start);
        self.literals
            .get(index)
            .is_some_and(|span| span.start < end)
    }

    fn enable_block_start_check(&mut self) {
        self.check_block_starts = true;
    }

    fn write_line(&mut self, line: &TokenLine<'_>, suffix: &'static str) -> Option<()> {
        self.write_token_slice(&line.tokens, suffix)
    }

    fn write_token_slice(
        &mut self,
        tokens: &[InlineToken<'_>],
        suffix: &'static str,
    ) -> Option<()> {
        if self.check_block_starts
            && self.line_count > 0
            && token_slice_markdown_block_start(tokens, &mut self.safety_line, false)
        {
            return None;
        }
        self.record_tokens(tokens.iter().map(InlineToken::text), suffix)
    }

    fn write_retained_slice(&mut self, tokens: TokenSlice<'_>, suffix: &'static str) -> Option<()> {
        debug_assert!(!self.check_block_starts);
        self.record_tokens(
            tokens
                .measured
                .iter()
                .map(|(span, _)| span.span().slice(tokens.text)),
            suffix,
        )
    }

    fn record_tokens<'a>(
        &mut self,
        tokens: impl Iterator<Item = &'a str>,
        suffix: &'static str,
    ) -> Option<()> {
        let start = self.tokens.len();
        for token in tokens {
            let piece = Text::retain(self.text, token);
            // A laid-out line usually occupies one contiguous normalized span.
            // Keep separate pieces only where joining or a rewrite requires it.
            if self.tokens.len() > start
                && let Text::Source(next) = &piece
                && let Some(Text::Source(previous)) = self.tokens.last_mut()
                && previous.end() + 1 == next.start()
                && self.text.as_bytes()[previous.end()] == b' '
            {
                *previous = SourceSpan::new(Span::new(previous.start(), next.end()));
            } else {
                self.tokens.push(piece);
            }
        }
        let suffix = match suffix {
            "" => None,
            "\\" => Some(MarkdownHardBreakMarker::CompactBackslash),
            " \\" => Some(MarkdownHardBreakMarker::SpaceBackslash),
            _ => unreachable!("planned hard break"),
        };
        self.out.push(PlannedLine {
            tokens: start..self.tokens.len(),
            suffix,
            block_start_checked: self.check_block_starts && self.line_count > 0 && suffix.is_none(),
        });
        self.line_count += 1;
        Some(())
    }
}

fn asterisk_run(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte == b'*')
}

fn trailing_asterisk_may_open(text: &str) -> bool {
    if !text.ends_with('*') || escaped_at(text, text.len() - 1) {
        return false;
    }
    // A closer must be valid with and without backslash math. Otherwise a
    // newline can turn its literal star into an opener in Pandoc Markdown.
    let mut scan = InlineScan::new(text);
    emphasis_span_end(&mut scan, 0) != Some(text.len())
        || text.contains("\\(")
            && emphasis_span_at_with_math(&mut scan, 0, false)
                .is_none_or(|span| span.end != text.len())
}

struct WrapLineBuffer<'a> {
    pending: Option<TokenLine<'a>>,
    current: TokenLine<'a>,
    width: usize,
    continuation_width: usize,
    sentence_wrap: bool,
}

impl<'a> WrapLineBuffer<'a> {
    fn new(first_width: usize, continuation_width: usize, sentence_wrap: bool) -> Self {
        Self {
            pending: None,
            current: TokenLine::default(),
            width: first_width.max(1),
            continuation_width: continuation_width.max(1),
            sentence_wrap,
        }
    }

    fn push_pending(
        &mut self,
        writer: &mut TokenLineWriter<'_, '_>,
        line: TokenLine<'a>,
    ) -> Option<()> {
        if let Some(pending) = self.pending.take() {
            writer.write_line(&pending, "")?;
        }
        self.pending = Some(line);
        Some(())
    }

    fn commit_current(&mut self, writer: &mut TokenLineWriter<'_, '_>) -> Option<()> {
        // Pandoc can pair a trailing literal asterisk across a newline. Keep
        // it with the following text when another asterisk could pair.
        if writer.preserve_asterisk_escapes
            && self
                .current
                .tokens
                .last()
                .is_some_and(|token| trailing_asterisk_may_open(token.text()))
        {
            return Some(());
        }
        // A bare ordered marker can introduce a list when more text follows
        // on the next line.
        self.repair_current_line_if_markdown_block_start(
            &mut writer.safety_line,
            true,
            writer.preserve_asterisk_escapes,
        )?;
        if self.current.is_empty() {
            return Some(());
        }
        let line = std::mem::take(&mut self.current);
        self.push_pending(writer, line)?;
        self.width = self.continuation_width;
        Some(())
    }

    fn finish(mut self, writer: &mut TokenLineWriter<'_, '_>, suffix: &'static str) -> Option<()> {
        // The last token has no next iteration to check a bare marker. Apply
        // the same check to escaped and unescaped inputs, including at EOF.
        // A compact backslash cannot complete a list prefix.
        let include_marker_only = suffix != "\\";
        self.repair_current_line_if_markdown_block_start(
            &mut writer.safety_line,
            include_marker_only,
            writer.preserve_asterisk_escapes,
        )?;
        if !self.current.is_empty() {
            if let Some(pending) = self.pending.take() {
                writer.write_line(&pending, "")?;
            }
            writer.write_line(&self.current, suffix)?;
        } else if let Some(pending) = self.pending.take() {
            writer.write_line(&pending, suffix)?;
        }
        Some(())
    }

    fn push_split_token_lines(
        &mut self,
        writer: &mut TokenLineWriter<'_, '_>,
        split: Vec<String>,
        preserve_math_closers: bool,
    ) -> Option<()> {
        let mut split = split.into_iter();
        let Some(first) = split.next() else {
            return Some(());
        };
        let first = InlineToken::owned(first, preserve_math_closers);
        if self.current.can_fit(&first, self.width) {
            self.current.push(first);
        } else {
            self.commit_current(writer)?;
            self.current.push(first);
        }
        self.repair_current_line_if_markdown_block_start(
            &mut writer.safety_line,
            self.sentence_wrap,
            writer.preserve_asterisk_escapes,
        )?;

        let mut rest = split
            .map(|text| InlineToken::owned(text, preserve_math_closers))
            .collect::<Vec<_>>();
        if rest.is_empty() {
            return Some(());
        }

        self.commit_current(writer)?;
        let last = rest.pop().expect("rest is not empty");
        for token in rest {
            let mut line = TokenLine::default();
            line.push(token);
            self.push_pending(writer, line)?;
        }
        self.current.push(last);

        if self.current.width > self.continuation_width {
            self.commit_current(writer)?;
        }
        self.repair_current_line_if_markdown_block_start(
            &mut writer.safety_line,
            self.sentence_wrap,
            writer.preserve_asterisk_escapes,
        )?;
        Some(())
    }

    fn repair_current_line_if_markdown_block_start(
        &mut self,
        scratch: &mut String,
        include_marker_only: bool,
        preserve_asterisk_escapes: bool,
    ) -> Option<()> {
        if self.pending.is_none()
            || self.current.is_empty()
            || !self
                .current
                .markdown_block_start(scratch, include_marker_only)
        {
            return Some(());
        }
        let escape = literal_block_marker_escape(
            scratch,
            self.current.tokens[0].preserve_math_closers,
            preserve_asterisk_escapes,
        );
        if self.sentence_wrap
            && let Some(offset) = escape
        {
            self.current.tokens[0].escape_marker(offset);
            self.current.width += 1;
            return Some(());
        }
        let previous = self.pending.as_mut()?;
        for split in (1..previous.len()).rev() {
            if preserve_asterisk_escapes
                && trailing_asterisk_may_open(previous.tokens[split - 1].text())
            {
                continue;
            }
            let suffix_width = previous.suffix_width(split);
            let candidate_width = suffix_width + 1 + self.current.width;
            if candidate_width > self.continuation_width && !self.sentence_wrap {
                continue;
            }
            let mut candidate = TokenLine::default();
            candidate.extend_from_slice(&previous.tokens[split..]);
            candidate.extend_from_slice(&self.current.tokens);
            if !candidate.markdown_block_start(scratch, include_marker_only) {
                let mut moved = previous.split_off(split);
                moved.append(std::mem::take(&mut self.current));
                self.current = moved;
                return Some(());
            }
        }
        if let Some(offset) = escape {
            self.current.tokens[0].escape_marker(offset);
            self.current.width += 1;
            return Some(());
        }
        if self.sentence_wrap
            || preserve_asterisk_escapes && asterisk_run(self.current.tokens[0].text())
        {
            // Keep the repaired line open so the marker's following text
            // cannot be stranded across a delimiter-sensitive line break.
            let mut joined = self.pending.take()?;
            joined.append(std::mem::take(&mut self.current));
            self.current = joined;
            return Some(());
        }
        None
    }
}

fn write_wrapped_tokens_with_first_width(
    writer: &mut TokenLineWriter<'_, '_>,
    tokens: TokenSlice<'_>,
    first_width: usize,
    continuation_width: usize,
    suffix: &'static str,
    sentence_wrap: bool,
) -> Option<()> {
    let mut lines = WrapLineBuffer::new(first_width, continuation_width, sentence_wrap);
    for (index, mut token) in tokens.iter(writer.unmatched_math_opener).enumerate() {
        // Sentence layout already checked bare markers after the last token.
        // Column layout needs this check before measuring the next token.
        if !sentence_wrap {
            lines.repair_current_line_if_markdown_block_start(
                &mut writer.safety_line,
                true,
                writer.preserve_asterisk_escapes,
            )?;
        }
        let literal = writer.overlaps_literal(token.text());
        // Column layout may have escaped the previous period. Decide whether
        // it ends a sentence using the text that will actually be written.
        if sentence_wrap
            && lines.current.tokens.last().is_some_and(|previous| {
                token_ends_sentence(previous.text())
                    && !token_is_sentence_abbreviation(previous.text())
            })
        {
            let mut escape = None;
            let safe_break = if token_may_start_markdown_block(token.text()) {
                let start = tokens.measured[index].0.start();
                let end = tokens.measured[index..]
                    .iter()
                    .find(|(span, _)| {
                        let text = span.span().slice(tokens.text);
                        token_ends_sentence(text) && !token_is_sentence_abbreviation(text)
                    })
                    .unwrap_or(&tokens.measured[tokens.len() - 1])
                    .0
                    .end();
                writer.safety_line.clear();
                writer.safety_line.push_str(&tokens.text[start..end]);
                // Recognize a standalone ordered marker at the end, too.
                writer.safety_line.push(' ');
                escape = literal_block_marker_escape(
                    &writer.safety_line,
                    token.preserve_math_closers,
                    writer.preserve_asterisk_escapes,
                );
                !markdown_block_start_line(&writer.safety_line) || escape.is_some()
            } else {
                true
            };
            if safe_break {
                lines.commit_current(writer)?;
                if let Some(offset) = escape {
                    token.escape_marker(offset);
                }
            }
        }
        // Column layout can remove a sentence-start escape once the marker is
        // back inside prose. Keep escapes at the start of a paragraph or hard
        // break, and let the existing column repair avoid new block starts.
        // Reflow can make spaced asterisks form emphasis across a newline.
        // Retain their escapes when another asterisk could pair with them.
        if !sentence_wrap && index > 0 && !literal {
            token.remove_literal_marker_escape(writer.preserve_asterisk_escapes);
        }
        if token.width > lines.width
            && !literal
            && let Some(split) = split_long_link_token(token.text(), lines.width)
        {
            lines.push_split_token_lines(writer, split, token.preserve_math_closers)?;
            lines.width = lines.continuation_width;
            continue;
        }
        if lines.current.can_fit(&token, lines.width) {
            lines.current.push(token);
        } else {
            lines.commit_current(writer)?;
            lines.current.push(token);
        }
        lines.repair_current_line_if_markdown_block_start(
            &mut writer.safety_line,
            sentence_wrap,
            writer.preserve_asterisk_escapes,
        )?;
    }
    lines.finish(writer, suffix)
}

pub(super) fn write_markdown_token_lines(
    writer: &mut TokenLineWriter<'_, '_>,
    tokens: TokenSlice<'_>,
    wrap: MarkdownWrap,
    suffix: &'static str,
) -> Option<()> {
    let column_width = wrap.column_width();
    match wrap {
        MarkdownWrap::None | MarkdownWrap::Paragraph => writer.write_retained_slice(tokens, suffix),
        MarkdownWrap::Sentence => write_sentence_token_lines(writer, tokens, suffix),
        MarkdownWrap::Column(_) | MarkdownWrap::SentenceAndColumn(_) => {
            let width = column_width.expect("column wrap has a width");
            let first_width = width
                .saturating_sub(writer.first_prefix.chars().count())
                .max(1);
            let continuation_width = width
                .saturating_sub(writer.continuation_prefix.chars().count())
                .max(1);
            writer.enable_block_start_check();
            write_wrapped_tokens_with_first_width(
                writer,
                tokens,
                first_width,
                continuation_width,
                suffix,
                matches!(wrap, MarkdownWrap::SentenceAndColumn(_)),
            )
        }
    }
}

fn token_slice_width(tokens: &[InlineToken<'_>]) -> usize {
    tokens.iter().map(|token| token.width).sum::<usize>() + tokens.len().saturating_sub(1)
}

fn token_slice_markdown_block_start(
    tokens: &[InlineToken<'_>],
    scratch: &mut String,
    include_marker_only: bool,
) -> bool {
    if !tokens
        .first()
        .is_some_and(|token| token_may_start_markdown_block(token.text()))
    {
        return false;
    }
    // Layout checks growing candidate lines repeatedly. Reuse their temporary
    // storage while keeping the existing recognition input and rules unchanged.
    scratch.clear();
    write_joined_tokens(scratch, tokens);
    if include_marker_only {
        scratch.push(' ');
    }
    markdown_block_start_line(scratch)
}

fn write_joined_tokens(out: &mut String, tokens: &[InlineToken<'_>]) {
    let Some((first, rest)) = tokens.split_first() else {
        return;
    };
    out.push_str(first.text());
    for token in rest {
        out.push(' ');
        out.push_str(token.text());
    }
}

pub(super) fn token_width(token: &str) -> usize {
    token.chars().count()
}

fn split_long_link_token(token: &str, width: usize) -> Option<Vec<String>> {
    if token.contains(['\n', '\r']) {
        return None;
    }
    if let Some(split) = split_long_image_attribute_token(token, width) {
        return Some(split);
    }
    let link = normalized_split_link_token(token)?;
    let SplitLinkToken {
        link,
        trailing_suffix,
        image,
        label_start,
        label_close,
        destination_open,
        target,
        suffix,
    } = link;
    // Keep empty destinations as `()`. Pandoc treats angle brackets as URL
    // characters when an angle-bracket destination follows a newline.
    if target.title.is_some()
        || target.destination.is_empty()
        || target.destination.starts_with('<')
    {
        return None;
    }

    if !image {
        let label = &link[label_start..label_close];
        if label.starts_with("![")
            && let Some(nested_image) = split_long_link_token(label, width)
        {
            let mut nested_image = nested_image.into_iter();
            let mut lines = Vec::new();
            let first = nested_image.next()?;
            lines.push(format!("{}{first}", &link[..label_start]));
            let mut rest = nested_image.collect::<Vec<_>>();
            let last = rest.pop()?;
            lines.extend(rest);
            let inline_target_close = format!(
                "{last}]({}){}{}",
                render_link_target(target),
                suffix,
                trailing_suffix
            );
            if inline_target_close.chars().count() <= width {
                lines.push(inline_target_close);
                return Some(lines);
            }
            lines.push(format!("{last}]("));
            lines.extend(split_link_target_lines(target, width));
            lines.push(format!("){suffix}{trailing_suffix}"));
            return Some(lines);
        }
    }

    let mut lines = Vec::new();
    lines.push(link[..destination_open + 1].to_owned());
    lines.extend(split_link_target_lines(target, width));
    lines.push(format!("){suffix}{trailing_suffix}"));
    Some(lines)
}

struct SplitLinkToken<'a> {
    link: &'a str,
    trailing_suffix: &'a str,
    image: bool,
    label_start: usize,
    label_close: usize,
    destination_open: usize,
    target: LinkTarget<'a>,
    suffix: &'a str,
}

fn normalized_split_link_token(token: &str) -> Option<SplitLinkToken<'_>> {
    let image = token.starts_with("![");
    if !image && !token.starts_with('[') {
        return None;
    }
    let label_start = if image { 2 } else { 1 };
    let label_close = find_balanced_square_close(token, label_start)?;
    let label = &token[label_start..label_close];
    if normalize_link_label(label, !image)? != label {
        return None;
    }
    let destination_open = label_close + 1;
    if token.as_bytes().get(destination_open) != Some(&b'(') {
        return None;
    }
    let destination_close = find_simple_destination_close(token, destination_open + 1)?;
    let raw_target = &token[destination_open + 1..destination_close];
    let target = parse_simple_link_target(raw_target)?;
    if !link_target_is_normalized(raw_target, target) {
        return None;
    }

    let mut end = destination_close + 1;
    if token.as_bytes().get(end) == Some(&b'{') {
        let (attribute_end, normalized) = normalize_attribute_after(token, end)?;
        if normalized != token[end..attribute_end] {
            return None;
        }
        end = attribute_end;
    }

    let trailing_suffix = &token[end..];
    if !trailing_link_suffix(trailing_suffix) {
        return None;
    }
    let link = &token[..end];
    let suffix = &link[destination_close + 1..];
    Some(SplitLinkToken {
        link,
        trailing_suffix,
        image,
        label_start,
        label_close,
        destination_open,
        target,
        suffix,
    })
}

fn link_target_is_normalized(raw: &str, target: LinkTarget<'_>) -> bool {
    match target.title {
        Some(_) => render_link_target(target) == raw,
        None => target.destination == raw,
    }
}

fn trailing_link_suffix(value: &str) -> bool {
    value
        .trim_start_matches([')', ']', '}'])
        .chars()
        .all(|ch| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?'))
}

fn split_long_image_attribute_token(token: &str, width: usize) -> Option<Vec<String>> {
    let (image, attributes, trailing) = simple_image_attribute_parts(token)?;
    if let Some(attribute_lines) = wrap_fig_alt_attribute_lines(attributes, width) {
        let mut split = Vec::with_capacity(attribute_lines.len() + 2);
        split.push(format!("{image}{{"));
        let mut attribute_lines = attribute_lines.into_iter();
        let first = attribute_lines.next()?;
        split.push(format!("  {first}"));
        split.extend(attribute_lines);
        split.push(format!("}}{trailing}"));
        return Some(split);
    }
    Some(vec![
        format!("{image}{{"),
        format!("  {attributes}"),
        format!("}}{trailing}"),
    ])
}

fn simple_image_attribute_parts(token: &str) -> Option<(&str, &str, &str)> {
    if !token.starts_with("![") {
        return None;
    }
    let label_close = find_balanced_square_close(token, 2)?;
    let destination_open = label_close + 1;
    if token.as_bytes().get(destination_open) != Some(&b'(') {
        return None;
    }
    let destination_close = find_simple_destination_close(token, destination_open + 1)?;
    parse_simple_link_target(&token[destination_open + 1..destination_close])?;
    let attribute_open = destination_close + 1;
    if token.as_bytes().get(attribute_open) != Some(&b'{') {
        return None;
    }
    let attribute_len = balanced_brace_end(&token[attribute_open..])?;
    let attribute_close = attribute_open + attribute_len;
    let trailing = &token[attribute_close..];
    if !trailing
        .chars()
        .all(|ch| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?'))
    {
        return None;
    }
    let attributes = token[attribute_open + 1..attribute_close - 1].trim();
    if !is_pandoc_attribute_content(attributes) {
        return None;
    }
    Some((&token[..attribute_open], attributes, trailing))
}

fn wrap_fig_alt_attribute_lines(attributes: &str, width: usize) -> Option<Vec<String>> {
    const FIG_ALT: &str = "fig-alt=\"";

    if attributes.contains('\n') {
        return None;
    }

    let fig_alt_start = find_fig_alt_attribute(attributes)?;
    let value_start = fig_alt_start + FIG_ALT.len();
    let value = attributes[value_start..].strip_suffix('"')?;
    let words = single_spaced_words(value)?;
    let leading = &attributes[..fig_alt_start];
    if leading.contains(['"', '\'']) {
        return None;
    }
    let prefix = &attributes[..value_start];
    let first_width = width.checked_sub(2 + prefix.chars().count())?;
    let continuation_width = width.checked_sub(1)?;
    let lines = wrap_single_spaced_words(&words, first_width, continuation_width)?;

    if lines.len() <= 1 {
        return None;
    }
    let mut lines = lines.into_iter();
    let mut attribute_lines = Vec::new();
    attribute_lines.push(format!("{prefix}{}", lines.next()?));
    attribute_lines.extend(lines);
    attribute_lines.last_mut()?.push('"');
    Some(attribute_lines)
}

fn find_fig_alt_attribute(attributes: &str) -> Option<usize> {
    const FIG_ALT: &str = "fig-alt=\"";

    let mut cursor = 0usize;
    while let Some(offset) = attributes[cursor..].find(FIG_ALT) {
        let start = cursor + offset;
        if start == 0 || attributes.as_bytes().get(start - 1) == Some(&b' ') {
            return Some(start);
        }
        cursor = start + FIG_ALT.len();
    }
    None
}

fn single_spaced_words(value: &str) -> Option<Vec<&str>> {
    let words = value.split(' ').collect::<Vec<_>>();
    (words.len() > 1 && words.iter().all(|word| !word.is_empty())).then_some(words)
}

fn wrap_single_spaced_words(
    words: &[&str],
    first_width: usize,
    continuation_width: usize,
) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = first_width;

    for word in words {
        let word_width = word.chars().count();
        if word_width > continuation_width {
            return None;
        }
        if current.is_empty() {
            if word_width > current_width {
                return None;
            }
            current.push_str(word);
        } else if current.chars().count() + 1 + word_width <= current_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = (*word).to_owned();
            current_width = continuation_width;
        }
    }

    lines.push(current);
    Some(lines)
}

pub(super) fn is_pandoc_attribute_content(value: &str) -> bool {
    !value.is_empty() && (value.starts_with(['#', '.']) || value.contains('='))
}
