//! Shared inline normalization and paragraph preparation. Container drafts,
//! table layout, token recognition, and measured-token layout have separate
//! owners; retained plans connect preparation to emission.

mod containers;
mod inline;
mod layout;
mod plan;
mod syntax;
mod tables;
mod templates;
#[cfg(test)]
mod tests;

use crate::core::directives::TemplateDelimiter;
use crate::core::document::{FormatOptions, MarkdownWrap};
use crate::core::lines::text_lines as markdown_lines;
use crate::core::markdown_marker::{markdown_list_marker, ordered_alpha_marker};
use crate::core::source::{SourceSpan, Span};
use containers::{footnote_definition, prepare_markdown_footnote};
pub use containers::{
    format_markdown_blockquote, format_markdown_definition_list, format_markdown_list,
};
pub(crate) use containers::{
    prepare_markdown_blockquote, prepare_markdown_definition_list, prepare_markdown_list,
};
use inline::{Gaps, InlineSource};
use layout::{
    TokenLineWriter, TokenSlice, is_pandoc_attribute_content, token_width,
    write_markdown_token_lines,
};
use plan::Condition;
pub(crate) use plan::{Draft, Fragment, Plan};
use std::borrow::Cow;
pub(crate) use syntax::markdown_reflow_changes_raw_semantics;
use syntax::{
    InlineScan, balanced_brace_end, delimiter_run_len_at, emphasis_span_at, emphasis_span_end,
    escaped_at, find_balanced_square_close, find_simple_destination_close, inline_code_span_end,
    inline_math_span_end, inline_token_end, latex_command_token_end, link_or_bracket_span_end,
    link_or_bracket_token_end, parse_simple_link_target, raw_semantics_protected_token_end,
    render_existing_split_link_target, render_link_target, split_trailing_attribute,
    strikethrough_span_end,
};
pub use tables::{format_markdown_pandoc_table, format_markdown_table};
pub(crate) use templates::ProseTemplateSpans;
use templates::{TemplateBoundary, inline_template_at};
use unicode_width::UnicodeWidthStr;

pub fn format_markdown_paragraph(source: &str, options: FormatOptions) -> String {
    prepare_markdown_paragraph(source)
        .and_then(|mut draft| draft.resolve(source, options))
        .map(|plan| plan.emit(source))
        .unwrap_or_else(|| source.to_owned())
}

pub(crate) fn prepare_markdown_paragraph(source: &str) -> Option<Draft> {
    let (body, newline) = strip_final_newline(source);
    if footnote_definition(body) {
        return Some(Draft::choice(
            Condition::FormatFootnotes,
            prepare_markdown_footnote(source),
            Some(Draft::text(source, source)),
        ));
    }
    let mut draft = Draft::inline(source, body, "", "", newline, true, false, false, true);
    draft.paragraph(single_line_body(body));
    Some(draft)
}

pub(crate) fn markdown_multiline_link_destination_spans(
    source: &str,
) -> impl Iterator<Item = std::ops::Range<usize>> + '_ {
    source.match_indices("](").filter_map(|(index, _)| {
        let destination_start = index + 2;
        let destination_close = find_simple_destination_close(source, destination_start)?;
        source[destination_start..destination_close]
            .contains(['\n', '\r'])
            .then_some(destination_start..destination_close)
    })
}

pub(crate) fn markdown_multiline_literal_spans(
    source: &str,
) -> impl Iterator<Item = std::ops::Range<usize>> + '_ {
    let mut scan = MarkdownMultilineInlineScan::new(source);
    std::iter::from_fn(move || scan.next_literal_before(source.len()))
}

// Query in source order and skip confirmed raw blocks so their delimiters
// cannot pair with inline content in later list items.
pub(crate) struct MarkdownMultilineInlineScan<'a> {
    scan: InlineScan<'a>,
    index: usize,
    literal_end: usize,
    link_index: usize,
    link_end: usize,
}

impl<'a> MarkdownMultilineInlineScan<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            scan: InlineScan::new(source),
            index: 0,
            literal_end: 0,
            link_index: 0,
            link_end: 0,
        }
    }

    pub(crate) fn covers(&mut self, offset: usize) -> bool {
        assert!(offset <= self.scan.text.len());
        if offset < self.literal_end {
            return true;
        }
        while let Some(span) = self.next_literal_before(offset) {
            if span.contains(&offset) {
                self.literal_end = span.end;
                return true;
            }
        }

        let source = self.scan.text;
        if offset < self.link_end {
            return true;
        }
        while self.link_index + 2 < offset {
            let index = self.link_index;
            self.link_index += 1;
            if source.as_bytes()[index..].starts_with(b"](") {
                let start = index + 2;
                if let Some(end) = find_simple_destination_close(source, start)
                    && source[start..end].contains(['\n', '\r'])
                {
                    self.link_end = self.link_end.max(end);
                    if offset < self.link_end {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(crate) fn skip_to(&mut self, offset: usize) {
        assert!(offset <= self.scan.text.len());
        self.index = offset;
        self.literal_end = offset;
        self.link_index = offset;
        self.link_end = offset;
    }

    fn next_literal_before(&mut self, limit: usize) -> Option<std::ops::Range<usize>> {
        let source = self.scan.text;
        while self.index < limit {
            let byte = source.as_bytes()[self.index];
            if escaped_at(source, self.index) {
                // An escaped run cannot become an opener partway through it.
                self.index += if byte == b'`' {
                    delimiter_run_len_at(source, self.index, b'`')
                } else {
                    source[self.index..].chars().next().unwrap().len_utf8()
                };
                continue;
            }
            if let Some(end) = self.scan.literal_span_end(self.index) {
                let start = self.index;
                self.index = end;
                if byte == b'`' {
                    self.index += delimiter_run_len_at(source, self.index, b'`');
                }
                if source[start..end].contains(['\n', '\r']) {
                    return Some(start..end);
                }
            } else if byte == b'`' {
                self.index += delimiter_run_len_at(source, self.index, b'`');
            } else if let Some(end) = raw_semantics_protected_token_end(&mut self.scan, self.index)
            {
                self.index = end;
            } else {
                self.index += source[self.index..]
                    .chars()
                    .next()
                    .expect("source character")
                    .len_utf8();
            }
        }
        None
    }
}

pub fn normalize_heading_content(source: &str) -> String {
    let text = source.trim_ascii();
    let Some((body, attr)) = split_trailing_attribute(text) else {
        return normalize_spaces_preserving_protected_spans(text).into_owned();
    };
    if trailing_attribute_belongs_to_inline(body) {
        return normalize_spaces_preserving_protected_spans(text).into_owned();
    }
    let Some(attr) = normalize_heading_attribute_block(attr) else {
        return normalize_spaces_preserving_protected_spans(text).into_owned();
    };
    let body = trim_heading_closing_hashes(body);
    let body = normalize_spaces_preserving_protected_spans(body.trim_ascii());
    if body.is_empty() {
        attr
    } else {
        format!("{body} {attr}")
    }
}

fn trailing_attribute_belongs_to_inline(body: &str) -> bool {
    let mut scan = InlineScan::new(body);

    if body.ends_with(char::is_whitespace) {
        return false;
    }
    body.char_indices().any(|(start, _)| {
        link_or_bracket_token_end(&mut scan, start) == Some(body.len())
            || !escaped_at(body, start) && inline_code_span_end(body, start) == Some(body.len())
    })
}

fn trim_heading_closing_hashes(source: &str) -> &str {
    let text = source.trim_ascii_end();
    let bytes = text.as_bytes();
    let mut hash_start = bytes.len();
    while hash_start > 0 && bytes[hash_start - 1] == b'#' {
        hash_start -= 1;
    }
    if hash_start == 0 {
        return "";
    }
    if hash_start == bytes.len() || !matches!(bytes[hash_start - 1], b' ' | b'\t') {
        return text;
    }
    let mut content_end = hash_start - 1;
    while content_end > 0 && matches!(bytes[content_end - 1], b' ' | b'\t') {
        content_end -= 1;
    }
    &text[..content_end]
}

pub fn format_markdown_fragment(source: &str, options: FormatOptions) -> String {
    Fragment::plan(source.to_owned(), options).emit()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownHardBreakMarker {
    CompactBackslash,
    SpaceBackslash,
}

impl MarkdownHardBreakMarker {
    fn suffix(self) -> &'static str {
        match self {
            Self::CompactBackslash => "\\",
            Self::SpaceBackslash => " \\",
        }
    }
}

fn markdown_hard_break_line_content(line: &str) -> (&str, Option<MarkdownHardBreakMarker>) {
    if let Some(content) = line.strip_suffix('\\') {
        let marker = if content.ends_with(' ') {
            MarkdownHardBreakMarker::SpaceBackslash
        } else {
            MarkdownHardBreakMarker::CompactBackslash
        };
        return (content.trim_end_matches([' ', '\t']), Some(marker));
    }
    let trimmed = line.trim_end_matches([' ', '\t']);
    let trailing_spaces = line
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b' ')
        .count();
    if trailing_spaces >= 2 && !trimmed.is_empty() {
        (trimmed, Some(MarkdownHardBreakMarker::SpaceBackslash))
    } else {
        (line.trim(), None)
    }
}

fn has_hard_break(source: &str) -> bool {
    markdown_line_bodies(source).any(|line| line.ends_with("  ") || line.ends_with('\\'))
}

fn single_line_body(source: &str) -> bool {
    !source.contains('\n') && !source.contains('\r')
}

fn markdown_block_start_line(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indent = line.len() - trimmed.len();
    indent <= 3
        && (atx_heading_start(trimmed)
            || thematic_break_line(trimmed)
            || setext_underline_line(trimmed)
            || trimmed.starts_with('>')
            || trimmed.starts_with("```")
            || trimmed.starts_with("~~~")
            || trimmed.starts_with("$$")
            || trimmed.starts_with("\\[")
            || trimmed.starts_with("\\begin{")
            || trimmed.starts_with('|')
            || trimmed.starts_with("Table:")
            || trimmed.starts_with(':')
            || trimmed.starts_with('~')
                && trimmed
                    .as_bytes()
                    .get(1)
                    .is_some_and(u8::is_ascii_whitespace)
            || trimmed.starts_with("<!--")
            || markdown_html_block_start_line(trimmed)
            || markdown_reference_definition_start(trimmed)
            || markdown_list_marker(trimmed).is_some())
}

pub(crate) fn markdown_reference_definition_start(text: &str) -> bool {
    // Parsing and wrap safety must agree: the colon follows the label's
    // matching bracket, not a later `]:` in ordinary prose.
    text.starts_with('[')
        && find_balanced_square_close(text, 1)
            .is_some_and(|close| text.as_bytes().get(close + 1) == Some(&b':'))
}

fn setext_underline_line(line: &str) -> bool {
    let line = line.trim_end_matches([' ', '\t']);
    matches!(line.as_bytes().first(), Some(b'-' | b'='))
        && line.bytes().all(|byte| byte == line.as_bytes()[0])
}

// ASCII words can start blocks only as list markers or Table: captions. Use
// the shared marker grammar so ordinary words ending in a period do not send
// every growing line through block recognition. Other prefixes still do.
fn token_may_start_markdown_block(token: &str) -> bool {
    let token = token.trim_start_matches([' ', '\t']);
    let word_len = token.bytes().take_while(u8::is_ascii_alphabetic).count();
    word_len == 0
        || token.starts_with("Table:")
        || token
            .as_bytes()
            .get(word_len)
            .is_some_and(|byte| matches!(byte, b'.' | b')'))
            && ordered_alpha_marker(&token[..word_len])
}

/// These markers are literal punctuation in prose. Escaping HTML, math, or
/// other inline constructs can change their meaning, so they stay joined.
fn literal_block_marker_escape(
    line: &str,
    preserve_math_closers: bool,
    preserve_asterisk_escapes: bool,
) -> Option<usize> {
    if atx_heading_start(line) || line.starts_with('>') {
        return Some(0);
    }
    let (marker, _) = markdown_list_marker(line)?;
    if preserve_math_closers && marker.ends_with(')') || preserve_asterisk_escapes && marker == "*"
    {
        return None;
    }
    // Escaping an opening parenthesis could create Pandoc inline math.
    Some(
        if marker.starts_with('(') || marker.as_bytes()[0].is_ascii_alphanumeric() {
            marker.len() - 1
        } else {
            0
        },
    )
}

pub(crate) fn markdown_html_block_start_line(trimmed: &str) -> bool {
    markdown_html_block_start(trimmed, true)
}

fn markdown_html_block_start(trimmed: &str, paragraph_start: bool) -> bool {
    if trimmed.starts_with("<!") || trimmed.starts_with("<?") {
        return true;
    }
    let Some((name, tag_end)) = markdown_html_tag_name(trimmed) else {
        return false;
    };
    commonmark_block_html_tag(name) || paragraph_start && trimmed[tag_end..].trim_ascii().is_empty()
}

fn markdown_html_tag_name(source: &str) -> Option<(&str, usize)> {
    if !inline_html_tag_at(source, 0) {
        return None;
    }
    let mut name_start = 1;
    if source.as_bytes().get(name_start) == Some(&b'/') {
        name_start += 1;
    }
    let mut name_end = name_start;
    while source
        .as_bytes()
        .get(name_end)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    {
        name_end += 1;
    }
    let tag_end = source.find('>')? + 1;
    Some((&source[name_start..name_end], tag_end))
}

fn commonmark_block_html_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "address"
            | "article"
            | "aside"
            | "base"
            | "basefont"
            | "blockquote"
            | "body"
            | "caption"
            | "center"
            | "col"
            | "colgroup"
            | "dd"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "frame"
            | "frameset"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "header"
            | "hr"
            | "html"
            | "iframe"
            | "legend"
            | "li"
            | "link"
            | "main"
            | "menu"
            | "menuitem"
            | "nav"
            | "noframes"
            | "ol"
            | "optgroup"
            | "option"
            | "p"
            | "param"
            | "pre"
            | "script"
            | "search"
            | "section"
            | "style"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "textarea"
            | "tfoot"
            | "th"
            | "thead"
            | "title"
            | "tr"
            | "track"
            | "ul"
    )
}

fn atx_heading_start(trimmed: &str) -> bool {
    let depth = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6).contains(&depth)
        && trimmed
            .as_bytes()
            .get(depth)
            .is_none_or(u8::is_ascii_whitespace)
}

fn thematic_break_line(trimmed: &str) -> bool {
    let mut marker = None;
    let mut count = 0usize;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            continue;
        }
        if !matches!(ch, '-' | '*' | '_') {
            return false;
        }
        if let Some(marker) = marker {
            if marker != ch {
                return false;
            }
        } else {
            marker = Some(ch);
        }
        count += 1;
    }
    count >= 3
}

fn newline_for_join(final_newline: &str, options: FormatOptions) -> &str {
    match final_newline {
        "\r\n" => "\r\n",
        "\r" => "\r",
        "\n" => "\n",
        _ => options.default_line_ending,
    }
}

pub fn canonicalize_inline(source: &str) -> String {
    canonicalize_recognized_inline(source, &[], 0)
}

fn canonicalize_recognized_inline(source: &str, literals: &[Span], offset: usize) -> String {
    let mut scan = InlineScan::new(source);
    scan.literals = literals;
    scan.literal_offset = offset;

    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    while index < source.len() {
        let rest = &source[index..];
        if let Some(end) = scan.literal_span_end(index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if rest.starts_with("~~") && !escaped_at(source, index) {
            let Some(close) = rest[2..].find("~~") else {
                out.push_str(rest);
                break;
            };
            let end = index + 2 + close + 2;
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if rest.starts_with('`') && escaped_at(source, index) {
            // An escaped run cannot start a literal partway through the run.
            let end = index + delimiter_run_len_at(source, index, b'`');
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = paired_inline_html_span_end(source, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = protected_inline_token_end(&mut scan, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        let ch = rest.chars().next().expect("index is on a char boundary");
        if ch == '_'
            && !escaped_at(source, index)
            && let Some(span) = emphasis_span_at(&mut scan, index)
        {
            let delimiter = "*".repeat(span.run);
            out.push_str(&delimiter);
            out.push_str(&canonicalize_recognized_inline(
                &source[index + span.run..span.close],
                literals,
                offset + index + span.run,
            ));
            out.push_str(&delimiter);
            out.push_str(&source[span.close + span.run..span.end]);
            index = span.end;
            continue;
        }
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

fn paired_inline_html_span_end(source: &str, index: usize) -> Option<usize> {
    if escaped_at(source, index) {
        return None;
    }
    if commonmark_autolink_span_end(source, index).is_some() {
        return None;
    }
    let rest = &source[index..];
    if !rest.starts_with('<')
        || rest.starts_with("</")
        || rest.starts_with("<!--")
        || rest.starts_with("<!")
        || rest.starts_with("<?")
    {
        return None;
    }
    let mut chars = rest[1..].char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    let mut tag_end = 1 + first.len_utf8();
    for (offset, ch) in chars {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            tag_end = 1 + offset + ch.len_utf8();
            continue;
        }
        break;
    }
    let tag = &rest[1..tag_end];
    let open_end = rest.find('>')? + 1;
    if rest[..open_end].trim_end().ends_with("/>") {
        return None;
    }
    html_closing_tag_end(source, index + open_end, tag)
}

fn html_closing_tag_end(source: &str, start: usize, tag: &str) -> Option<usize> {
    let mut search_start = start;
    while search_start < source.len() {
        let relative = source[search_start..].find("</")?;
        let candidate_start = search_start + relative + 2;
        let candidate = &source[candidate_start..];
        let name_len = candidate
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
            .count();
        let name = &candidate[..name_len];
        let rest = candidate[name_len..].trim_start();
        if !name.is_empty() && name.eq_ignore_ascii_case(tag) && rest.starts_with('>') {
            return Some(candidate_start + name_len + candidate[name_len..].find('>')? + 1);
        }
        search_start = candidate_start + name_len;
    }
    None
}

fn protected_inline_token_end(scan: &mut InlineScan<'_>, index: usize) -> Option<usize> {
    let source = scan.text;
    let rest = &source[index..];
    if escaped_at(source, index) {
        return None;
    }
    if let Some(end) = reference_style_link_span_end(scan, index) {
        return Some(end);
    }
    if rest.starts_with("![") || rest.starts_with('[') {
        return link_or_bracket_token_end(scan, index);
    }
    if rest.starts_with('<') {
        return rest.find('>').map(|close| index + close + 1);
    }
    if (rest.starts_with("{{<") || rest.starts_with("{{%"))
        && let Some(close) = rest.find("}}")
    {
        return Some(index + close + 2);
    }
    if rest.starts_with('{')
        && let Some(end) = balanced_brace_end(rest)
    {
        return Some(index + end);
    }
    if rest.starts_with('\\') && rest[1..].chars().next().is_some_and(char::is_alphabetic) {
        return latex_command_token_end(source, index);
    }
    None
}

pub(crate) fn commonmark_autolink_span_end(source: &str, index: usize) -> Option<usize> {
    if escaped_at(source, index) || !source[index..].starts_with('<') {
        return None;
    }
    let inner_start = index + '<'.len_utf8();
    let close = inner_start + source[inner_start..].find('>')?;
    let inner = &source[inner_start..close];
    (commonmark_uri_autolink(inner) || commonmark_email_autolink(inner)).then_some(close + 1)
}

fn commonmark_uri_autolink(inner: &str) -> bool {
    let Some(colon) = inner.find(':') else {
        return false;
    };
    let scheme = &inner[..colon];
    if !(2..=32).contains(&scheme.len()) {
        return false;
    }
    let mut scheme_chars = scheme.chars();
    if !scheme_chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        return false;
    }
    if !scheme_chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '+' | '-')) {
        return false;
    }
    inner[colon + 1..]
        .chars()
        .all(|ch| !ch.is_ascii_control() && !ch.is_ascii_whitespace() && !matches!(ch, '<' | '>'))
}

fn commonmark_email_autolink(inner: &str) -> bool {
    let Some((local, domain)) = inner.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && local.chars().all(commonmark_email_local_char)
        && domain.split('.').all(commonmark_email_domain_label)
}

fn commonmark_email_local_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '/'
                | '='
                | '?'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
                | '-'
                | '.'
        )
}

fn commonmark_email_domain_label(label: &str) -> bool {
    let len = label.len();
    (1..=63).contains(&len)
        && label
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        && label
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
        && label
            .chars()
            .next_back()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
}

fn reference_style_link_span_end(scan: &mut InlineScan<'_>, index: usize) -> Option<usize> {
    let source = scan.text;
    let label_start = if source[index..].starts_with("![") {
        index + 2
    } else if source[index..].starts_with('[') {
        index + 1
    } else {
        return None;
    };
    let label_close = scan.square_close(label_start)?;
    let reference_open = label_close + 1;
    if source.as_bytes().get(reference_open) != Some(&b'[') {
        return None;
    }
    let reference_close = scan.square_close(reference_open + 1)?;
    Some(reference_close + 1)
}

fn final_newline(source: &str) -> &str {
    if source.ends_with("\r\n") {
        "\r\n"
    } else if source.ends_with('\n') {
        "\n"
    } else if source.ends_with('\r') {
        "\r"
    } else {
        ""
    }
}

fn normalize_spaces(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut pending_space = false;
    for ch in source.chars() {
        if ch.is_ascii_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            out.push(ch);
            pending_space = false;
        }
    }
    out
}

fn normalize_spaces_preserving_protected_spans(source: &str) -> Cow<'_, str> {
    let mut scan = InlineScan::new(source);

    if inline_spacing_is_already_normalized(source) {
        return Cow::Borrowed(source);
    }
    let mut out = String::with_capacity(source.len());
    let mut pending_space = false;
    let mut index = 0usize;
    while index < source.len() {
        if let Some(end) = protected_spacing_span_end(&mut scan, index) {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        let ch = source[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        if ch.is_ascii_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            out.push(ch);
            pending_space = false;
        }
        index += ch.len_utf8();
    }
    Cow::Owned(out)
}

fn inline_spacing_is_already_normalized(source: &str) -> bool {
    if source.is_empty() {
        return true;
    }
    let bytes = source.as_bytes();
    if matches!(bytes.first(), Some(b' ' | b'\t' | b'\r' | b'\n'))
        || matches!(bytes.last(), Some(b' ' | b'\t' | b'\r' | b'\n'))
    {
        return false;
    }
    !bytes
        .iter()
        .any(|byte| matches!(*byte, b'\t' | b'\r' | b'\n'))
        && !bytes.windows(2).any(|window| window == b"  ")
}

fn protected_spacing_span_end(scan: &mut InlineScan<'_>, index: usize) -> Option<usize> {
    let source = scan.text;
    reference_style_link_span_end(scan, index)
        .or_else(|| balanced_brace_span_end(source, index))
        .or_else(|| strikethrough_span_end(source, index))
        .or_else(|| commonmark_autolink_span_end(source, index))
        .or_else(|| paired_inline_html_span_end(source, index))
        .or_else(|| inline_html_tag_span_end(source, index))
        .or_else(|| inline_code_span_end(source, index))
        .or_else(|| inline_math_span_end(source, index))
        .or_else(|| emphasis_span_end(scan, index))
        .or_else(|| latex_command_token_end(source, index))
}

fn protected_spacing_span_can_start(source: &str, index: usize) -> bool {
    if escaped_at(source, index) {
        return false;
    }
    source[index..].chars().next().is_some_and(|ch| {
        matches!(
            ch,
            '[' | '!' | '{' | '`' | '$' | '~' | '<' | '\\' | '*' | '_'
        )
    })
}

fn balanced_brace_span_end(source: &str, index: usize) -> Option<usize> {
    if escaped_at(source, index) || !source[index..].starts_with('{') {
        return None;
    }
    balanced_brace_end(&source[index..]).map(|end| index + end)
}

fn simple_inline_tokens_supported(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'`' | b'$' | b'~' | b'<' | b'\\' | b'[' | b'{' | b'*' | b'_' => return false,
            b'!' if bytes.get(index + 1) == Some(&b'[') => return false,
            _ => index += 1,
        }
    }
    true
}

fn simple_inline_tokens(text: &str) -> Vec<(SourceSpan, usize)> {
    text.split_ascii_whitespace()
        .map(|token| {
            let start = token.as_ptr() as usize - text.as_ptr() as usize;
            (
                SourceSpan::new(Span::new(start, start + token.len())),
                token_width(token),
            )
        })
        .collect()
}

fn multiline_brace_span_at(text: &str, index: usize) -> bool {
    if escaped_at(text, index) || !text[index..].starts_with('{') {
        return false;
    }
    let Some(end) = balanced_brace_end(&text[index..]) else {
        return false;
    };
    // Multiline brace spans can be Pandoc or Quarto attributes; leave them raw.
    text[index..index + end].contains(['\n', '\r'])
}

fn inline_html_tag_at(text: &str, index: usize) -> bool {
    if escaped_at(text, index) {
        return false;
    }
    let rest = &text[index..];
    if !rest.starts_with('<') {
        return false;
    }
    let Some(close) = rest.find('>') else {
        return false;
    };
    let mut name_start = '<'.len_utf8();
    if rest.as_bytes().get(name_start) == Some(&b'/') {
        name_start += '/'.len_utf8();
    }
    let Some(first) = rest[name_start..].chars().next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    let mut name_end = name_start;
    while name_end < close {
        let ch = rest[name_end..]
            .chars()
            .next()
            .expect("name_end is on a char boundary");
        if !(ch.is_ascii_alphanumeric() || ch == '-') {
            break;
        }
        name_end += ch.len_utf8();
    }
    if name_end == name_start {
        return false;
    }
    let after_name = &rest[name_end..close];
    !(after_name.starts_with([':', '@']))
        && (after_name.is_empty()
            || after_name.starts_with(char::is_whitespace)
            || after_name.trim() == "/")
}

fn inline_html_tag_span_end(text: &str, index: usize) -> Option<usize> {
    if !inline_html_tag_at(text, index) {
        return None;
    }
    text[index..].find('>').map(|close| index + close + 1)
}

fn inline_html_tag_has_quoted_gt(text: &str, index: usize) -> bool {
    let Some(end) = inline_html_tag_span_end(text, index) else {
        return false;
    };
    // Existing tag recognition stops at the first `>`. An open quote there
    // makes the boundary ambiguous; preserve the paragraph instead of editing
    // attribute contents as prose.
    let mut quote = None;
    for byte in text[index..end].bytes() {
        match (quote, byte) {
            (None, b'\'' | b'"') => quote = Some(byte),
            (Some(open), close) if open == close => quote = None,
            _ => {}
        }
    }
    quote.is_some()
}

fn normalize_supported_links_and_images(source: &str) -> Cow<'_, str> {
    let mut scan = InlineScan::new(source);

    if !source.as_bytes().contains(&b'[') {
        return Cow::Borrowed(source);
    }
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    while index < source.len() {
        let rest = &source[index..];
        if rest.starts_with('<') && !escaped_at(source, index) {
            // Without interpreting angle syntax, literal boundaries may be
            // ambiguous. Preserve this input instead of rewriting any links,
            // including ones already visited before the angle marker.
            return Cow::Borrowed(source);
        }
        // Visit literals in prose order. A real link below consumes its label,
        // destination and attributes before their delimiters can become openers.
        if rest.starts_with('`') {
            let end = if !escaped_at(source, index)
                && let Some(end) = scan.code_span_end(index)
            {
                end
            } else {
                index
            };
            // Neither a failed/escaped opener nor the remainder of a closing
            // run starts a new literal partway through that backtick run.
            let end = end
                + source[end..]
                    .bytes()
                    .take_while(|byte| *byte == b'`')
                    .count();
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if rest.starts_with('$')
            && let Some(end) = inline_math_span_end(source, index)
        {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = balanced_brace_span_end(source, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = reference_style_link_span_end(&mut scan, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if !escaped_at(source, index)
            && (rest.starts_with("![") || rest.starts_with('['))
            && let Some((end, normalized)) = normalize_link_or_image_at(&mut scan, index)
        {
            out.push_str(&normalized);
            index = end;
            continue;
        }
        let ch = rest.chars().next().expect("index is on a char boundary");
        out.push(ch);
        index += ch.len_utf8();
    }
    Cow::Owned(out)
}

fn normalize_link_or_image_at(scan: &mut InlineScan<'_>, start: usize) -> Option<(usize, String)> {
    let text = scan.text;
    let image = text[start..].starts_with("![");
    let label_start = if image { start + 2 } else { start + 1 };
    let label_close = scan.square_close(label_start)?;
    let after_label = label_close + 1;
    if text.as_bytes().get(after_label) != Some(&b'(') {
        return None;
    }
    let destination_close = find_simple_destination_close(text, after_label + 1)?;
    let label = &text[label_start..label_close];
    let label = normalize_link_label(label, !image)?;
    let raw_target = &text[after_label + 1..destination_close];
    let target = parse_simple_link_target(raw_target)?;
    let target = if raw_target.contains(['\n', '\r']) {
        render_existing_split_link_target(raw_target, target)
    } else {
        render_link_target(target)
    };
    let mut end = destination_close + 1;
    let mut normalized = String::new();
    if image {
        normalized.push('!');
    }
    normalized.push('[');
    normalized.push_str(&label);
    normalized.push_str("](");
    normalized.push_str(&target);
    normalized.push(')');
    let attribute = if image {
        normalize_image_attribute_after(text, end)
    } else {
        normalize_attribute_after(text, end)
    };
    if let Some((attribute_end, attribute)) = attribute {
        normalized.push_str(&attribute);
        end = attribute_end;
    }
    Some((end, normalized))
}

fn normalize_link_label(label: &str, allow_nested_image: bool) -> Option<String> {
    let mut scan = InlineScan::new(label);

    if label.contains(['\n', '\r']) {
        return None;
    }
    let mut out = String::with_capacity(label.len());
    let mut index = 0usize;
    while index < label.len() {
        let rest = &label[index..];
        if !escaped_at(label, index)
            && allow_nested_image
            && rest.starts_with("![")
            && let Some((end, normalized)) = normalize_link_or_image_at(&mut scan, index)
        {
            out.push_str(&normalized);
            index = end;
            continue;
        }
        if let Some(end) = link_label_protected_span_end(&mut scan, index) {
            out.push_str(&label[index..end]);
            index = end;
            continue;
        }
        if unsupported_link_label_inline_at(label, index) {
            return None;
        }
        let ch = rest.chars().next().expect("index is on a char boundary");
        out.push(ch);
        index += ch.len_utf8();
    }
    Some(normalize_spaces_preserving_protected_spans(out.trim()).into_owned())
}

fn link_label_protected_span_end(scan: &mut InlineScan<'_>, index: usize) -> Option<usize> {
    let label = scan.text;
    strikethrough_span_end(label, index)
        .or_else(|| inline_code_span_end(label, index))
        .or_else(|| inline_math_span_end(label, index))
        .or_else(|| paired_inline_html_span_end(label, index))
        .or_else(|| inline_html_tag_span_end(label, index))
        .or_else(|| emphasis_span_end(scan, index))
        .or_else(|| latex_command_token_end(label, index))
}

fn unsupported_link_label_inline_at(label: &str, index: usize) -> bool {
    let rest = &label[index..];
    if escaped_at(label, index) {
        return false;
    }
    if rest.starts_with('\\') {
        return true;
    }
    if rest.starts_with("![") || rest.starts_with(['[', ']']) {
        return true;
    }
    if rest.starts_with('`') || rest.starts_with('$') || rest.starts_with('<') {
        return true;
    }
    false
}

fn normalize_attribute_after(text: &str, start: usize) -> Option<(usize, String)> {
    let rest = &text[start..];
    if !rest.starts_with('{') {
        return None;
    }
    let end = balanced_brace_end(rest)?;
    let attribute = &rest[..end];
    normalize_attribute_block(attribute).map(|normalized| (start + end, normalized))
}

fn normalize_image_attribute_after(text: &str, start: usize) -> Option<(usize, String)> {
    if let Some(attribute) = normalize_attribute_after(text, start) {
        return Some(attribute);
    }

    let mut cursor = start;
    while matches!(text.as_bytes().get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    if cursor == start {
        return None;
    }

    let (end, attribute) = normalize_attribute_after(text, cursor)?;
    let inner = &attribute[1..attribute.len() - 1];
    is_pandoc_attribute_content(inner).then_some((end, attribute))
}

fn normalize_attribute_block(attribute: &str) -> Option<String> {
    if !attribute.starts_with('{') || !attribute.ends_with('}') {
        return None;
    }
    let inner = &attribute[1..attribute.len() - 1];
    if inner.contains(['\n', '\r', '\'']) {
        return None;
    }
    let tokens = attribute_tokens(inner)?;
    if tokens.is_empty() {
        return None;
    }
    Some(format!("{{{}}}", tokens.join(" ")))
}

fn normalize_heading_attribute_block(attribute: &str) -> Option<String> {
    if !attribute.starts_with('{') || !attribute.ends_with('}') {
        return None;
    }
    let inner = &attribute[1..attribute.len() - 1];
    if inner.contains(['\n', '\r', '\'']) {
        return None;
    }
    let tokens = attribute_tokens(inner)?;
    if tokens.is_empty() || !tokens.iter().all(|token| heading_attribute_token(token)) {
        return None;
    }
    Some(format!("{{{}}}", tokens.join(" ")))
}

fn heading_attribute_token(token: &str) -> bool {
    if token == "-" {
        return true;
    }
    if let Some(value) = token.strip_prefix('#').or_else(|| token.strip_prefix('.')) {
        return !value.is_empty() && !value.contains(['{', '}', '"', '\'']);
    }
    let Some((key, value)) = token.split_once('=') else {
        return false;
    };
    !key.is_empty()
        && key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
        && !value.is_empty()
        && !value.contains(['{', '}', '\''])
}

fn attribute_tokens(inner: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_double_quote = false;
    for ch in inner.chars() {
        match ch {
            '"' => {
                in_double_quote = !in_double_quote;
                current.push(ch);
            }
            ch if ch.is_whitespace() && !in_double_quote => {
                if !current.is_empty() {
                    tokens.push(normalize_attribute_token(&current));
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if in_double_quote {
        return None;
    }
    if !current.is_empty() {
        tokens.push(normalize_attribute_token(&current));
    }
    Some(tokens)
}

fn normalize_attribute_token(token: &str) -> String {
    let Some(value) = token
        .strip_prefix("fig-alt=\"")
        .and_then(|value| value.strip_suffix('"'))
    else {
        return token.to_owned();
    };
    format!("fig-alt=\"{}\"", normalize_spaces(value))
}

fn display_width(source: &str) -> usize {
    UnicodeWidthStr::width(source)
}

fn markdown_line_bodies(source: &str) -> impl Iterator<Item = &str> {
    markdown_lines(source).map(|line| line.body)
}

fn strip_final_newline(source: &str) -> (&str, &str) {
    if let Some(body) = source.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = source.strip_suffix('\n') {
        (body, "\n")
    } else if let Some(body) = source.strip_suffix('\r') {
        (body, "\r")
    } else {
        (source, "")
    }
}
