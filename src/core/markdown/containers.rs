//! List, blockquote, footnote, and definition-list boundaries. Ambiguous child
//! layouts stay intact so recognition cannot change container membership.

use super::fences::{code_fence_at, find_code_fence_closing, find_code_fence_closing_at_indent};
use super::{
    grid_table_separator, heading_ranges, markdown_indented_code_at, raw_sensitive_at,
    raw_sensitive_end, setext_heading_at,
};
use crate::core::markdown_marker::markdown_list_marker_len;
use crate::core::source::{SourceBuffer, Span};

pub(super) fn list_item_at(text: &str) -> bool {
    let trimmed = text.trim_start_matches(' ');
    let indent = text.len() - trimmed.len();
    if indent > 3 {
        return false;
    }
    markdown_list_marker_len(trimmed).is_some()
}

pub(super) struct ListBlock {
    pub(super) end: usize,
    pub(super) preserve: bool,
}

pub(super) fn list_block_end(source: &SourceBuffer, start: usize, end: usize) -> ListBlock {
    let first = source.line_text(start);
    let mut base_indent = first.len() - first.trim_start().len();
    let mut item_content_indent = list_marker_content_indent(first);
    let mut block_openers = Vec::new();
    let mut line = start + 1;
    while line < end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            let Some(next) = next_nonblank_line(source, line + 1, end) else {
                line = end;
                break;
            };
            if blank_line_continues_list_item(source.line_text(next), item_content_indent) {
                line = next;
                continue;
            }
            break;
        }
        if list_item_at(text) {
            let indent = text.len() - text.trim_start().len();
            // Nested items do not change where the outer item can resume
            // after a blank line. Only a sibling starts a new boundary.
            if item_content_indent.is_some_and(|content_indent| indent < content_indent) {
                // Keep the list's lowest indentation for lazy continuations.
                base_indent = base_indent.min(indent);
                item_content_indent = list_marker_content_indent(text);
            }
            line += 1;
            continue;
        }
        let indent = text.len() - text.trim_start().len();
        if indent > base_indent {
            if item_content_indent.is_some_and(|content_indent| indent < content_indent)
                && !markdown_indented_code_at(text)
                && !grid_table_separator(text)
                && (code_fence_at(text).is_some() || raw_sensitive_at(text))
            {
                block_openers.push(line);
            }
            line += 1;
            continue;
        }
        // Low-indent prose can lazily continue the item's paragraph. Keep
        // scanning so a later indented paragraph remains inside the item;
        // unsupported continuation layouts are preserved by list planning.
        // Keep existing blockquote boundaries and mixed raw-region handling;
        // their recognition is separate from this lazy-prose preservation rule.
        if !block_openers.is_empty()
            || blockquote_at(text)
            || raw_sensitive_at(text)
            || code_fence_at(text).is_some()
        {
            break;
        }
        line += 1;
    }
    if block_openers.is_empty() {
        return ListBlock {
            end: line,
            preserve: false,
        };
    }

    // Preserve mixed list/raw regions together: normalizing a list marker can
    // change the following block's membership. Consume the region we scanned
    // so the next list does not scan the same suffix again.
    let block_start = source.lines[start].full.start();
    let block = source.slice(Span::new(block_start, source.lines[line - 1].full.end()));
    let mut inlines = crate::core::wrap::MarkdownMultilineInlineScan::new(block);
    let mut raw_end = start;
    let mut preserve = false;
    for opener in block_openers {
        if opener < raw_end {
            continue;
        }
        let offset = source.lines[opener].text.start() - block_start;
        if inlines.covers(offset) {
            continue;
        }

        preserve = true;
        // Inline-looking markers were checked within the tentative region.
        // Only a raw block can extend it to protect an unindented body.
        raw_end = if let Some(fence) = code_fence_at(source.line_text(opener)) {
            find_code_fence_closing(source, opener + 1, end, fence)
                .map_or(end, |closing| closing + 1)
        } else {
            raw_sensitive_end(source, opener, end)
        };
        if raw_end >= line {
            line = raw_end;
            break;
        }
        inlines.skip_to(source.lines[raw_end].full.start() - block_start);
    }
    ListBlock {
        end: line,
        preserve,
    }
}

fn next_nonblank_line(source: &SourceBuffer, mut line: usize, end: usize) -> Option<usize> {
    while line < end {
        if !source.line_text(line).trim().is_empty() {
            return Some(line);
        }
        line += 1;
    }
    None
}

fn blank_line_continues_list_item(text: &str, item_content_indent: Option<usize>) -> bool {
    let Some(item_content_indent) = item_content_indent else {
        return false;
    };
    let indent = text.len() - text.trim_start().len();
    indent >= item_content_indent
}

/// Capability checks consume the same item ownership that layout will use.
/// These conservative checks remain separate from recognition: recognizing a
/// child does not assert that normalizing its surrounding list is safe.
pub(super) fn list_block_supported(
    source: &SourceBuffer,
    start: usize,
    end: usize,
    structure: &super::ListStructure,
) -> bool {
    let first = source.line_text(start);
    let base_indent = super::list::leading_spaces(first);
    let block_start = source.lines[start].full.start();
    let mut block_index = 0;
    let mut multiline_inlines: Option<Vec<std::ops::Range<usize>>> = None;
    let mut code_fence_end = start;
    for line in start + 1..end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            continue;
        }
        let offset = source.lines[line].full.start() - block_start;
        while structure.blocks[block_index].source().end() <= offset {
            block_index += 1;
        }
        let block = &structure.blocks[block_index];
        let item = &structure.items[block.item];
        let item_content_indent = Some(item.content_indent);
        let task_continuation = item
            .checkbox
            .map(|_| (item.continuation_min, item.continuation_min + 4));
        if matches!(
            block.kind,
            super::ListBlockKind::Paragraph { first: true, .. }
        ) && block.source().start() == offset
        {
            continue;
        }
        let indent = super::list::leading_spaces(text);
        if line >= code_fence_end
            && let Some(content_indent) = item_content_indent
            && indent >= content_indent
            && let Some(fence) = text.get(content_indent..).and_then(code_fence_at)
        {
            code_fence_end =
                find_code_fence_closing_at_indent(source, line + 1, end, fence, content_indent)
                    .map_or(end, |closing| closing + 1);
        }
        if line >= code_fence_end
            && item_content_indent.is_some_and(|content_indent| indent >= content_indent)
            && (heading_ranges(source, line).is_some()
                || setext_heading_at(source, line, end).is_some())
        {
            let block_start = source.lines[start].full.start();
            let multiline_inlines = multiline_inlines
                .get_or_insert_with(|| list_multiline_inline_spans(source, start, end));
            let content_offset = source.lines[line].text.start() - block_start + indent;
            let underline_offset = source
                .lines
                .get(line + 1)
                .filter(|_| setext_heading_at(source, line, end).is_some())
                .map(|next| next.text.start() - block_start);
            if !multiline_inlines.iter().any(|span| {
                span.contains(&content_offset)
                    || underline_offset.is_some_and(|offset| span.contains(&offset))
            }) {
                // The simple-list formatter cannot emit child headings.
                return false;
            }
        }
        if blockquote_at(text) {
            let Some(body) = blockquote_plain_body(text) else {
                return false;
            };
            if list_item_at(body) || raw_sensitive_at(body) || code_fence_at(body).is_some() {
                return false;
            }
            continue;
        }
        if list_continuation_matches_item_content(text, indent, item_content_indent) {
            continue;
        }
        if let Some((min, max)) = task_continuation
            && indent >= min
            && indent < max
        {
            continue;
        }
        if indent >= base_indent + 4 || raw_sensitive_at(text) || code_fence_at(text).is_some() {
            if list_rich_child_line(text, item_content_indent) {
                continue;
            }
            if list_item_at(text.trim_start()) {
                return false;
            }
            // Multiline links and literals are rare. Keep this whole-block
            // scan off the ordinary-list hot path and cache it when needed.
            let block_start = source.lines[start].full.start();
            let multiline_inlines = multiline_inlines
                .get_or_insert_with(|| list_multiline_inline_spans(source, start, end));
            // Include openers after the source indentation as well as lines
            // whose indentation is already inside a link or literal.
            let line_offset = source.lines[line].text.start() - block_start;
            let content_offset = line_offset + indent;
            if multiline_inlines
                .iter()
                .any(|span| span.contains(&line_offset) || span.contains(&content_offset))
            {
                continue;
            }
            return false;
        }
    }
    true
}

fn list_multiline_inline_spans(
    source: &SourceBuffer,
    start: usize,
    end: usize,
) -> Vec<std::ops::Range<usize>> {
    let block = source.slice(Span::new(
        source.lines[start].full.start(),
        source.lines[end - 1].full.end(),
    ));
    crate::core::wrap::markdown_multiline_link_destination_spans(block)
        .chain(crate::core::wrap::markdown_multiline_literal_spans(block))
        .collect()
}

fn list_rich_child_line(text: &str, item_content_indent: Option<usize>) -> bool {
    let Some(item_content_indent) = item_content_indent else {
        return false;
    };
    let indent = text.len() - text.trim_start().len();
    if indent < item_content_indent {
        return false;
    }
    let content = &text[indent..];
    code_fence_at(content).is_some() || raw_sensitive_at(content) || rich_child_block_start(content)
}

fn rich_child_block_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('|')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || trimmed.starts_with("$$")
        || trimmed.starts_with("\\[")
        || trimmed.starts_with("\\begin{")
        || trimmed.starts_with(":::")
        || trimmed.starts_with("{{<")
        || trimmed.starts_with("{{%")
        || blockquote_at(trimmed)
        || list_item_at(trimmed)
}

fn list_marker_content_indent(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    let marker_len = markdown_list_marker_len(trimmed)?;
    let after_marker = &trimmed[marker_len..];
    let marker_gap = after_marker.len() - after_marker.trim_start().len();
    // Pandoc permits a paragraph at the source end of an empty marker.
    Some(indent + marker_len + marker_gap)
}

fn list_continuation_matches_item_content(
    text: &str,
    indent: usize,
    item_content_indent: Option<usize>,
) -> bool {
    if item_content_indent != Some(indent) {
        return false;
    }
    let content = &text[indent..];
    !list_item_at(content) && !raw_sensitive_at(content) && code_fence_at(content).is_none()
}

pub(super) fn blockquote_at(text: &str) -> bool {
    let trimmed = text.trim_start_matches(' ');
    text.len() - trimmed.len() <= 3 && trimmed.starts_with('>')
}

pub(super) fn blockquote_block_supported(source: &SourceBuffer, start: usize, end: usize) -> bool {
    for line in start..end {
        if blockquote_plain_body(source.line_text(line)).is_none() {
            return false;
        };
    }
    true
}

fn blockquote_plain_body(text: &str) -> Option<&str> {
    let trimmed = text.trim_ascii_start();
    if text.len() - trimmed.len() > 3 {
        return None;
    }
    let mut body = trimmed.strip_prefix('>')?;
    loop {
        let leading_len = body
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
        if leading_len >= 5 || body[..leading_len].contains('\t') {
            return None;
        }
        body = body.trim_ascii_start();
        let Some(rest) = body.strip_prefix('>') else {
            break;
        };
        body = rest;
    }
    Some(body)
}

pub(super) fn footnote_definition_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    text.len() - trimmed.len() <= 3
        && trimmed.starts_with("[^")
        && crate::core::wrap::markdown_reference_definition_start(trimmed)
}

pub(super) fn footnote_block_end(source: &SourceBuffer, start: usize, end: usize) -> usize {
    let mut line = start + 1;
    while line < end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            let Some(next) = next_nonblank_line(source, line + 1, end) else {
                break;
            };
            let indent = source.line_text(next).len() - source.line_text(next).trim_start().len();
            if indent >= 4 {
                line = next + 1;
                continue;
            }
            break;
        }
        let indent = text.len() - text.trim_start().len();
        if indent >= 2 {
            line += 1;
            continue;
        }
        break;
    }
    line
}

pub(super) fn definition_list_at(source: &SourceBuffer, line: usize, end: usize) -> bool {
    if line + 1 >= end || source.line_text(line).trim().is_empty() {
        return false;
    }
    let marker = source.line_text(line + 1).trim_start();
    let indent = source.line_text(line + 1).len() - marker.len();
    indent <= 3
        && matches!(marker.as_bytes().first(), Some(b':' | b'~'))
        && marker
            .as_bytes()
            .get(1)
            .is_some_and(u8::is_ascii_whitespace)
}

pub(super) fn definition_list_end(source: &SourceBuffer, start: usize, end: usize) -> usize {
    let mut content_indent = definition_marker_content_indent(source.line_text(start + 1))
        .expect("definition_list_at validated the definition marker");
    let mut line = start + 2;
    while line < end {
        if !source.line_text(line).trim().is_empty() {
            line += 1;
            continue;
        }

        let blank_start = line;
        let Some(next) = next_nonblank_line(source, line + 1, end) else {
            return blank_start;
        };
        let next_text = source.line_text(next);
        let next_indent = next_text.len() - next_text.trim_start().len();
        if next_indent >= content_indent {
            line = next + 1;
            continue;
        }
        if let Some(next_content_indent) = definition_marker_content_indent(next_text) {
            content_indent = next_content_indent;
            line = next + 1;
            continue;
        }
        if definition_list_at(source, next, end) {
            content_indent = definition_marker_content_indent(source.line_text(next + 1))
                .expect("definition_list_at validated the definition marker");
            line = next + 2;
            continue;
        }
        return blank_start;
    }
    end
}

pub(super) fn definition_list_supported(source: &SourceBuffer, start: usize, end: usize) -> bool {
    for line in start..end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            continue;
        }
        if let Some(content) = definition_marker_content(text) {
            if raw_sensitive_at(content) || code_fence_at(content).is_some() {
                return false;
            }
            continue;
        }
        let trimmed = text.trim_start();
        let indent = text.len() - trimmed.len();
        if indent >= 4 {
            if raw_sensitive_at(trimmed) || code_fence_at(trimmed).is_some() {
                return false;
            }
            continue;
        }
        if raw_sensitive_at(text) || code_fence_at(text).is_some() {
            return false;
        }
    }
    true
}

fn definition_marker_content(text: &str) -> Option<&str> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    if indent > 3 || !matches!(trimmed.as_bytes().first(), Some(b':' | b'~')) {
        return None;
    }
    if !trimmed
        .as_bytes()
        .get(1)
        .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    Some(trimmed[1..].trim_start())
}

fn definition_marker_content_indent(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    if indent > 3 || !matches!(trimmed.as_bytes().first(), Some(b':' | b'~')) {
        return None;
    }
    let mut content_indent = indent + 1;
    while text
        .as_bytes()
        .get(content_indent)
        .is_some_and(u8::is_ascii_whitespace)
    {
        content_indent += 1;
    }
    (content_indent > indent + 1).then_some(content_indent)
}
