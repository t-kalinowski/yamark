//! Inline token boundaries and shared delimiter scans. Matching stays separate
//! from normalization and line layout, including the existing conservative grammar.

use super::templates::{TemplateBoundary, inline_template_at};
#[cfg(test)]
use super::tests;
use super::{
    commonmark_autolink_span_end, inline_html_tag_span_end, normalize_attribute_after,
    normalize_link_label, paired_inline_html_span_end, protected_inline_token_end,
};
use crate::core::directives::TemplateDelimiter;
use crate::core::source::Span;
use std::collections::HashMap;

pub(super) fn inline_token_end(scan: &mut InlineScan<'_>, start: usize) -> Option<usize> {
    let text = scan.text;
    // Inline syntax protects its contents; only surrounding whitespace creates
    // a prose wrapping boundary.
    let mut end = start;
    loop {
        end = inline_token_fragment_end(scan, end)?;
        if end == text.len()
            || text[end..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_whitespace())
        {
            return Some(end);
        }
    }
}

fn inline_token_fragment_end(scan: &mut InlineScan<'_>, start: usize) -> Option<usize> {
    let position = start + scan.literal_offset;
    let next_literal = scan.literals.partition_point(|span| span.start < position);
    let next_literal = scan.literals.get(next_literal);
    if let Some(span) = next_literal
        && span.start == position
    {
        return Some(span.end - scan.literal_offset);
    }
    let text = scan.text;
    let rest = &text[start..];
    if let Some(end) = strikethrough_span_end(text, start) {
        return Some(end);
    }
    if let Some(end) = commonmark_autolink_span_end(text, start) {
        return Some(inline_span_end_with_trailing_punctuation(text, end));
    }
    if let Some(end) = paired_inline_html_span_end(text, start) {
        return Some(end);
    }
    if rest.starts_with('`') {
        return inline_code_span_end(text, start);
    }
    if rest.starts_with('$') && !escaped_at(text, start) {
        return inline_math_span_end(text, start);
    }
    if rest.starts_with('\\') && rest[1..].chars().next().is_some_and(char::is_alphabetic) {
        return latex_command_token_end(text, start);
    }
    if let Some(end) = emphasis_span_end(scan, start) {
        return Some(end);
    }
    if rest.starts_with("![") || rest.starts_with('[') {
        return link_or_bracket_token_end(scan, start);
    }
    if rest.starts_with('<')
        && let Some(end) = inline_html_tag_span_end(text, start)
    {
        return Some(end);
    }
    if (rest.starts_with("{{<") || rest.starts_with("{{%"))
        && let Some(close) = rest.find("}}")
    {
        return Some(start + close + 2);
    }
    if rest.starts_with('{')
        && let Some(close) = balanced_brace_end(rest)
    {
        return Some(start + close);
    }

    let next_literal = next_literal.map_or(text.len(), |span| span.start - scan.literal_offset);
    let mut end = start;
    while end < text.len() {
        let slice = &text[end..];
        let ch = slice.chars().next()?;
        if ch.is_ascii_whitespace() {
            break;
        }
        if end != start
            && (end == next_literal
                || slice.starts_with('`')
                || slice.starts_with('$')
                || slice.starts_with("![")
                || slice.starts_with('[')
                || slice.starts_with('<')
                || slice.starts_with("{{<")
                || slice.starts_with("{{%"))
        {
            break;
        }
        end += ch.len_utf8();
    }
    while end < text.len() {
        if escaped_at(text, end) || !reference_bracket_token_at(scan, end) {
            break;
        }
        let Some(reference_end) = link_or_bracket_token_end(scan, end) else {
            break;
        };
        end = inline_span_end_with_trailing_punctuation(text, reference_end);
    }
    (end > start).then_some(end)
}

pub(super) fn strikethrough_span_end(text: &str, start: usize) -> Option<usize> {
    if escaped_at(text, start) || !text[start..].starts_with("~~") {
        return None;
    }
    let close = text[start + 2..].find("~~")?;
    let content = &text[start + 2..start + 2 + close];
    if content.is_empty() || content.trim().is_empty() || content.contains(['\n', '\r']) {
        return None;
    }
    Some(inline_span_end_with_trailing_punctuation(
        text,
        start + 2 + close + 2,
    ))
}

pub(super) fn inline_code_span_end(text: &str, start: usize) -> Option<usize> {
    if !text[start..].starts_with('`') {
        return None;
    }
    let rest = &text[start..];
    let tick_count = rest.bytes().take_while(|byte| *byte == b'`').count();
    let marker = &rest[..tick_count];
    let close = rest[tick_count..].find(marker)?;
    Some(start + tick_count + close + tick_count)
}

pub(super) fn inline_math_span_end(text: &str, start: usize) -> Option<usize> {
    if escaped_at(text, start) || !text[start..].starts_with('$') {
        return None;
    }
    let marker = if text[start..].starts_with("$$") {
        "$$"
    } else {
        "$"
    };
    let content_start = start + marker.len();
    let close = text[content_start..]
        .match_indices(marker)
        .map(|(offset, _)| content_start + offset)
        .find(|close| !escaped_at(text, *close))?;
    Some(close + marker.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EmphasisSpan {
    pub(super) close: usize,
    pub(super) end: usize,
    pub(super) run: usize,
}

pub(super) fn emphasis_span_end(scan: &mut InlineScan<'_>, start: usize) -> Option<usize> {
    emphasis_span_at(scan, start).map(|span| span.end)
}

pub(super) fn emphasis_span_at(scan: &mut InlineScan<'_>, start: usize) -> Option<EmphasisSpan> {
    emphasis_span_at_with_math(scan, start, true)
}

pub(super) fn emphasis_span_at_with_math(
    scan: &mut InlineScan<'_>,
    start: usize,
    backslash_math: bool,
) -> Option<EmphasisSpan> {
    let text = scan.text;
    if escaped_at(text, start) {
        return None;
    }
    let marker = text.as_bytes().get(start).copied()?;
    if !matches!(marker, b'*' | b'_') {
        return None;
    }
    let run = delimiter_run_len_at(text, start, marker);
    if run == 0
        || text
            .as_bytes()
            .get(start + run)
            .is_none_or(u8::is_ascii_whitespace)
    {
        return None;
    }
    if marker == b'_' && underscore_is_intraword(text, start, run) {
        return None;
    }
    let delimiter = &text[start..start + run];
    let mut search = start + run;
    let mut opaque_end = search;
    while search < text.len() {
        let byte = text.as_bytes()[search];
        // Keep matched math opaque while searching for the emphasis closer,
        // including Pandoc's backslash math.
        if search >= opaque_end && (matches!(byte, b'`' | b'$') || backslash_math && byte == b'\\')
        {
            if let Some(end) = scan.literal_span_end(search) {
                search = end;
                continue;
            }
            if byte == b'`' && escaped_at(text, search) {
                search += delimiter_run_len_at(text, search, b'`');
                continue;
            }
        }
        // Existing nonliteral ranges cannot open code/math spans. Retain the
        // old emphasis delimiter search within them; changing how emphasis
        // interacts with links, HTML/raw angles or attributes is separate work.
        if search >= opaque_end
            && matches!(byte, b'[' | b'!' | b'<' | b'{' | b'\\')
            && let Some(end) = paired_inline_html_span_end(text, search)
                .or_else(|| protected_inline_token_end(scan, search))
        {
            opaque_end = end;
        }
        if byte != marker || !text[search..].starts_with(delimiter) {
            // Only ASCII candidates above are sliced, so bytewise advancement
            // also skips ordinary UTF-8 prose without decoding it.
            search += 1;
            continue;
        }
        let close = search;
        if close <= start + run
            || text[..close]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace)
            || escaped_at(text, close)
            || text
                .as_bytes()
                .get(close.saturating_sub(1))
                .is_some_and(|byte| close > 0 && *byte == marker)
            || delimiter_run_len_at(text, close, marker) != run
            || text
                .as_bytes()
                .get(close + run)
                .is_some_and(|byte| *byte == marker)
            || (marker == b'_' && underscore_is_intraword(text, close, run))
        {
            search = close + run;
            continue;
        }
        // Nonempty contents ending in non-whitespace cannot be all whitespace.
        // Check line breaks only after the boundary tests. If one is present,
        // every later closing candidate contains it too, so this search is done.
        if emphasis_contents_have_line_break(&text[start + run..close]) {
            return None;
        }
        return Some(EmphasisSpan {
            close,
            end: inline_span_end_with_trailing_punctuation(text, close + run),
            run,
        });
    }
    None
}

fn emphasis_contents_have_line_break(contents: &str) -> bool {
    #[cfg(test)]
    tests::EMPHASIS_CONTENT_BYTES.with(|bytes| bytes.set(bytes.get() + contents.len()));
    contents.contains(['\n', '\r'])
}

pub(super) fn delimiter_run_len_at(text: &str, index: usize, marker: u8) -> usize {
    text[index..]
        .bytes()
        .take_while(|byte| *byte == marker)
        .count()
}

fn inline_span_end_with_trailing_punctuation(text: &str, mut end: usize) -> usize {
    while end < text.len() {
        let ch = text[end..]
            .chars()
            .next()
            .expect("end is on a char boundary");
        if !matches!(ch, '.' | ',' | ';' | ':' | '!' | '?') {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

pub(super) fn latex_command_token_end(text: &str, start: usize) -> Option<usize> {
    if !text[start..].starts_with('\\')
        || !text[start + '\\'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(char::is_alphabetic)
    {
        return None;
    }
    let mut end = start + '\\'.len_utf8();
    while end < text.len() {
        let ch = text[end..].chars().next()?;
        if !ch.is_alphabetic() {
            break;
        }
        end += ch.len_utf8();
    }
    while text[end..].starts_with('{') {
        let brace_end = balanced_brace_end(&text[end..])?;
        end += brace_end;
    }
    Some(end)
}

pub(crate) fn markdown_reflow_changes_raw_semantics(
    source: &str,
    delimiters: &[TemplateDelimiter],
) -> bool {
    let mut scan = InlineScan::new(source);

    if memchr::memchr(b'\\', source.as_bytes()).is_none() && source.is_ascii() {
        return false;
    }

    let bytes = source.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match inline_template_at(&mut scan, index, delimiters) {
            TemplateBoundary::Complete(end) => {
                index = end;
                continue;
            }
            TemplateBoundary::Incomplete => return true,
            TemplateBoundary::Absent => {}
        }
        let byte = bytes[index];
        if byte.is_ascii()
            && !matches!(byte, b'\\' | b'`' | b'$' | b'~' | b'<' | b'[' | b'!' | b'{')
        {
            index += 1;
            continue;
        }
        if byte.is_ascii() {
            if escaped_at(source, index) {
                index += 1;
                continue;
            }
            if byte == b'\\'
                && let Some(end) = latex_command_token_end(source, index)
            {
                if !source[index..end].contains('{') {
                    return true;
                }
                index = end;
                continue;
            }
            if let Some(end) = raw_semantics_protected_token_end(&mut scan, index) {
                index = end;
                continue;
            }
        }
        let ch = source[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        if ch.is_whitespace() && !ch.is_ascii_whitespace() {
            return true;
        }
        index += ch.len_utf8();
    }
    false
}

pub(super) fn raw_semantics_protected_token_end(
    scan: &mut InlineScan<'_>,
    index: usize,
) -> Option<usize> {
    let text = scan.text;
    let rest = &text[index..];
    match rest.as_bytes().first()? {
        b'`' => {
            let tick_count = rest.bytes().take_while(|byte| *byte == b'`').count();
            let marker = &rest[..tick_count];
            let close = rest[tick_count..].find(marker)?;
            Some(index + tick_count + close + tick_count)
        }
        b'$' => find_unescaped(rest, 1, '$').map(|close| index + close + 1),
        b'~' => strikethrough_span_end(text, index),
        b'<' => commonmark_autolink_span_end(text, index)
            .or_else(|| paired_inline_html_span_end(text, index))
            .or_else(|| inline_html_tag_span_end(text, index))
            .or_else(|| rest.find('>').map(|close| index + close + 1)),
        b'[' => link_or_bracket_token_end(scan, index),
        b'!' if rest.as_bytes().get(1) == Some(&b'[') => link_or_bracket_token_end(scan, index),
        b'{' if rest.starts_with("{{<") || rest.starts_with("{{%") => {
            rest.find("}}").map(|close| index + close + 2)
        }
        b'{' => balanced_brace_end(rest).map(|close| index + close),
        _ => None,
    }
}

pub(super) fn link_or_bracket_token_end(scan: &mut InlineScan<'_>, start: usize) -> Option<usize> {
    link_or_bracket_span_end(scan, start, true)
}

pub(super) fn link_or_bracket_span_end(
    scan: &mut InlineScan<'_>,
    start: usize,
    validate_label: bool,
) -> Option<usize> {
    let text = scan.text;
    let image = text[start..].starts_with("![");
    if !image && text.as_bytes().get(start) != Some(&b'[') {
        return None;
    }
    let label_start = if image { start + 2 } else { start + 1 };
    let label_close = scan.square_close(label_start)?;
    let after_label = label_close + 1;
    if text.as_bytes().get(after_label) == Some(&b'(') {
        if validate_label {
            normalize_link_label(&text[label_start..label_close], !image)?;
        }
        let destination_close = find_simple_destination_close(text, after_label + 1)?;
        parse_simple_link_target(&text[after_label + 1..destination_close])?;
        let mut end = destination_close + 1;
        if let Some((attribute_end, _)) = normalize_attribute_after(text, end) {
            end = attribute_end;
        }
        return Some(inline_span_end_with_trailing_punctuation(text, end));
    }
    if !image
        && let Some(end) = reference_style_link_end(scan, label_start, label_close, after_label)
    {
        return Some(inline_span_end_with_trailing_punctuation(text, end));
    }
    if !image && shortcut_reference_link_token(&text[label_start..label_close]) {
        return Some(inline_span_end_with_trailing_punctuation(text, after_label));
    }
    if text[start..].starts_with("[^")
        || !image && pandoc_citation_label(&text[label_start..label_close])
    {
        return Some(after_label);
    }
    None
}

fn shortcut_reference_link_token(label: &str) -> bool {
    !label.is_empty() && !label.contains(['\n', '\r'])
}

fn reference_style_link_end(
    scan: &mut InlineScan<'_>,
    label_start: usize,
    label_close: usize,
    reference_open: usize,
) -> Option<usize> {
    let text = scan.text;
    if text.as_bytes().get(reference_open) != Some(&b'[')
        || text[label_start..label_close].contains(['\n', '\r'])
    {
        return None;
    }
    let reference_close = scan.square_close(reference_open + 1)?;
    if text[reference_open + 1..reference_close].contains(['\n', '\r']) {
        return None;
    }
    Some(reference_close + 1)
}

pub(super) fn reference_bracket_token_at(scan: &mut InlineScan<'_>, start: usize) -> bool {
    let text = scan.text;
    if text[start..].starts_with("[^") {
        return true;
    }
    if !text[start..].starts_with('[') || text[start..].starts_with("![") {
        return false;
    }
    let Some(label_close) = scan.square_close(start + 1) else {
        return false;
    };
    pandoc_citation_label(&text[start + 1..label_close])
}

fn pandoc_citation_label(label: &str) -> bool {
    let mut saw_citation = false;
    for part in label.split(';') {
        let part = part.trim();
        if part.is_empty() {
            return false;
        }
        if pandoc_citation_part_has_marker(part) {
            saw_citation = true;
        } else {
            return false;
        }
    }
    saw_citation
}

fn pandoc_citation_part_has_marker(part: &str) -> bool {
    let mut index = 0usize;
    while index < part.len() {
        let rest = &part[index..];
        if rest.starts_with("-@")
            && !escaped_at(part, index)
            && citation_marker_boundary(part, index)
            && citation_key_end(part, index + 2).is_some_and(|end| end > index + 2)
        {
            return true;
        }
        if rest.starts_with('@')
            && !escaped_at(part, index)
            && citation_marker_boundary(part, index)
            && citation_key_end(part, index + 1).is_some_and(|end| end > index + 1)
        {
            return true;
        }
        let ch = rest.chars().next().expect("index is on a char boundary");
        index += ch.len_utf8();
    }
    false
}

fn citation_marker_boundary(text: &str, index: usize) -> bool {
    previous_char(text, index).is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '['))
}

fn citation_key_end(text: &str, mut index: usize) -> Option<usize> {
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if ch.is_whitespace() || matches!(ch, ',' | ';' | '[' | ']' | '(' | ')') {
            break;
        }
        index += ch.len_utf8();
    }
    Some(index)
}

// Ordinary matched labels need no cache. After a failed search, index the
// remaining bracket pairs once so later openers do not rescan the same suffix.
pub(super) struct InlineScan<'a> {
    pub(super) text: &'a str,
    cached_from: Option<usize>,
    square_closes: HashMap<usize, usize>,
    code_ends: Option<HashMap<usize, usize>>,
    parenthesized_math_closes: Option<Vec<usize>>,
    pub(super) literals: &'a [Span],
    pub(super) literal_offset: usize,
    pub(super) template_comment_closes: Option<Vec<(usize, usize)>>,
}

impl<'a> InlineScan<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self {
            text,
            cached_from: None,
            square_closes: HashMap::new(),
            code_ends: None,
            parenthesized_math_closes: None,
            literals: &[],
            literal_offset: 0,
            template_comment_closes: None,
        }
    }

    pub(super) fn literal_span_end(&mut self, start: usize) -> Option<usize> {
        if !self.literals.is_empty() {
            let position = start + self.literal_offset;
            let index = self.literals.partition_point(|span| span.start < position);
            if let Some(span) = self
                .literals
                .get(index)
                .filter(|span| span.start == position)
            {
                return Some(span.end - self.literal_offset);
            }
        }
        match self.text.as_bytes().get(start)? {
            b'`' if !escaped_at(self.text, start) => self.code_span_end(start),
            b'$' => inline_math_span_end(self.text, start),
            b'\\' if self.text[start..].starts_with("\\(") && !escaped_at(self.text, start) => {
                // Preserve possible Pandoc math as one literal. Index closers
                // once so unmatched openers do not repeatedly scan the suffix.
                let closes = self.parenthesized_math_closes.get_or_insert_with(|| {
                    self.text
                        .match_indices("\\)")
                        .map(|(index, _)| index)
                        .filter(|index| !escaped_at(self.text, *index))
                        .collect()
                });
                let next = closes.partition_point(|close| *close < start + 2);
                closes.get(next).map(|close| close + 2)
            }
            _ => None,
        }
    }

    pub(super) fn code_span_end(&mut self, start: usize) -> Option<usize> {
        if !self.text[start..].starts_with('`') {
            return None;
        }
        if let Some(ends) = &self.code_ends {
            return ends.get(&start).copied();
        }
        if let Some(end) = inline_code_span_end(self.text, start) {
            return Some(end);
        }
        // A failed opener must not make every later backtick rescan the suffix.
        // Cache the same marker searches as inline_code_span_end, including
        // suffixes of unmatched runs accepted by canonicalization. Include
        // earlier runs because a failed emphasis lookahead can return to them.
        // This caches existing recognition; it does not tighten its grammar.
        let mut ends = HashMap::new();
        let mut next = vec![None];
        let bytes = self.text.as_bytes();
        let mut index = bytes.len();
        while index > 0 {
            if bytes[index - 1] != b'`' {
                index -= 1;
                continue;
            }
            let end = index;
            while index > 0 && bytes[index - 1] == b'`' {
                index -= 1;
            }
            let run = end - index;
            if next.len() <= run {
                next.resize(run + 1, None);
            }
            for (length, &close) in next.iter().enumerate().take(run + 1).skip(1) {
                if let Some(close) = close {
                    ends.insert(end - length, close + length);
                }
            }
            next[1..=run].fill(Some(index));
        }
        self.code_ends = Some(ends);
        None
    }

    pub(super) fn square_close(&mut self, label_start: usize) -> Option<usize> {
        if self.cached_from.is_some_and(|start| label_start >= start) {
            return self.square_closes.get(&label_start).copied();
        }
        if let Some(close) = find_balanced_square_close(self.text, label_start) {
            return Some(close);
        }
        let mut closes = Vec::new();
        for (offset, ch) in self.text[label_start..].char_indices().rev() {
            let index = label_start + offset;
            if ch == '[' {
                if let Some(&close) = closes.last() {
                    self.square_closes.insert(index + 1, close);
                }
                if !escaped_at(self.text, index) {
                    closes.pop();
                }
            } else if ch == ']' && !escaped_at(self.text, index) {
                closes.push(index);
            }
        }
        self.cached_from = Some(label_start);
        None
    }
}

pub(super) fn find_balanced_square_close(text: &str, mut index: usize) -> Option<usize> {
    let mut depth = 0usize;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if escaped_at(text, index) {
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '[' => depth += 1,
            ']' if depth == 0 => return Some(index),
            ']' => depth -= 1,
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

pub(super) fn find_simple_destination_close(text: &str, mut index: usize) -> Option<usize> {
    let mut in_single = false;
    let mut in_double = false;
    let mut depth = 0usize;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if escaped_at(text, index) {
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '(' if !in_single && !in_double => depth += 1,
            ')' if !in_single && !in_double && depth == 0 => return Some(index),
            ')' if !in_single && !in_double => depth -= 1,
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LinkTarget<'a> {
    pub(super) destination: &'a str,
    pub(super) title: Option<&'a str>,
}

pub(super) fn parse_simple_link_target(target: &str) -> Option<LinkTarget<'_>> {
    let target = target.trim();
    if target.is_empty() {
        return Some(LinkTarget {
            destination: target,
            title: None,
        });
    }
    let Some(split) = target.find(char::is_whitespace) else {
        return simple_link_destination_atom_supported(target).then_some(LinkTarget {
            destination: target,
            title: None,
        });
    };
    let destination = &target[..split];
    if !simple_link_destination_atom_supported(destination) {
        return None;
    }
    let title = target[split..].trim();
    simple_link_title_supported(title).then_some(LinkTarget {
        destination,
        title: Some(title),
    })
}

pub(super) fn render_link_target(target: LinkTarget<'_>) -> String {
    match target.title {
        Some(title) => format!("{} {title}", target.destination),
        None => target.destination.to_owned(),
    }
}

pub(super) fn render_existing_split_link_target(raw: &str, target: LinkTarget<'_>) -> String {
    let leading = raw.len() - raw.trim_start().len();
    let trailing = raw.trim_end().len();
    format!(
        "{}{}{}",
        &raw[..leading],
        render_link_target(target),
        &raw[trailing..]
    )
}

pub(super) fn split_link_target_lines(target: LinkTarget<'_>, width: usize) -> Vec<String> {
    let target_width = width.saturating_sub(2).max(1);
    let lines = match target.title {
        Some(title)
            if render_link_target(target).chars().count() > target_width
                && !target.destination.is_empty() =>
        {
            vec![target.destination.to_owned(), title.to_owned()]
        }
        _ => vec![render_link_target(target)],
    };
    lines.into_iter().map(|line| format!("  {line}")).collect()
}

fn simple_link_destination_atom_supported(destination: &str) -> bool {
    let destination = destination.trim();
    let mut index = 0usize;
    while index < destination.len() {
        let ch = destination[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        if ch == '\\' {
            return false;
        }
        if !escaped_at(destination, index) && matches!(ch, '"' | '\'') {
            return false;
        }
        if ch.is_whitespace() {
            return false;
        }
        index += ch.len_utf8();
    }
    true
}

fn simple_link_title_supported(title: &str) -> bool {
    let open = title.as_bytes().first().copied();
    let Some((open, close)) = (match open {
        Some(b'"') => Some(('"', '"')),
        Some(b'\'') => Some(('\'', '\'')),
        Some(b'(') => Some(('(', ')')),
        _ => None,
    }) else {
        return false;
    };
    if !title.ends_with(close) {
        return false;
    }
    for (index, ch) in title.char_indices() {
        if index == 0 || index + ch.len_utf8() == title.len() {
            continue;
        }
        if matches!(ch, '\n' | '\r') {
            return false;
        }
        if ch == '\\' {
            return false;
        }
        if ch == close || (open == '(' && ch == open) {
            return false;
        }
    }
    true
}

fn find_unescaped(text: &str, mut index: usize, target: char) -> Option<usize> {
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if ch == target && !escaped_at(text, index) {
            return Some(index);
        }
        index += ch.len_utf8();
    }
    None
}

pub(super) fn balanced_brace_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + ch.len_utf8());
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

pub(super) fn split_trailing_attribute(text: &str) -> Option<(&str, &str)> {
    if !text.ends_with('}') {
        return None;
    }
    let open = text.rfind('{')?;
    let before = &text[..open];
    let attr = &text[open..];
    balanced_brace_end(attr).filter(|end| *end == attr.len())?;
    Some((before, attr))
}

pub(super) fn escaped_at(source: &str, index: usize) -> bool {
    let backslashes = source[..index]
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'\\')
        .count();
    backslashes % 2 == 1
}

fn underscore_is_intraword(source: &str, index: usize, run: usize) -> bool {
    previous_char(source, index).is_some_and(|ch| ch.is_alphanumeric())
        && next_char(source, index + run).is_some_and(|ch| ch.is_alphanumeric())
}

fn previous_char(source: &str, index: usize) -> Option<char> {
    source[..index].chars().next_back()
}

fn next_char(source: &str, index: usize) -> Option<char> {
    source[index..].chars().next()
}
