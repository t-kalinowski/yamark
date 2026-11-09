//! Authored list structure, independent of wrapping and output prefixes.
//!
//! A list is an ordered sequence of blocks owned by explicit items. Spans use
//! coordinates in the enclosing list's source slice. Keeping paragraph and
//! child boundaries here lets layout change marker spelling without interpreting
//! the resulting indentation as input syntax. The document parser still owns
//! the conservative outer boundary and unsupported-Markdown checks.

use crate::core::lines::{TextLine, text_lines};
use crate::core::markdown_marker::markdown_list_marker;
use crate::core::source::{SourceSpan, Span};
use crate::core::wrap::MarkdownMultilineInlineScan;

#[derive(Debug, Clone)]
pub(crate) struct ListStructure {
    pub(crate) items: Vec<ListItem>,
    pub(crate) blocks: Vec<ListBlock>,
}

#[derive(Debug, Clone)]
pub(crate) struct ListItem {
    pub(crate) parent: Option<usize>,
    pub(crate) marker: SourceSpan,
    pub(crate) checkbox: Option<SourceSpan>,
    pub(crate) indent: usize,
    /// Authored byte offset of prose, including marker padding and checkbox.
    pub(crate) content_indent: usize,
    /// The source marker's minimum continuation column, before optional padding.
    pub(super) continuation_min: usize,
    /// Child blocks belong to the list marker, not to its optional checkbox.
    marker_content_indent: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct ListBlock {
    pub(crate) item: usize,
    pub(crate) kind: ListBlockKind,
}

#[derive(Debug, Clone)]
pub(crate) enum ListBlockKind {
    Paragraph {
        source: SourceSpan,
        /// Absolute byte offset in the list slice; always a character boundary.
        content_start: usize,
        first: bool,
    },
    Blank(SourceSpan),
    Child {
        source: SourceSpan,
        indent: usize,
    },
}

impl ListBlock {
    pub(super) fn source(&self) -> SourceSpan {
        match self.kind {
            ListBlockKind::Paragraph { source, .. } | ListBlockKind::Child { source, .. } => source,
            ListBlockKind::Blank(source) => source,
        }
    }
}

impl ListStructure {
    pub(crate) fn recognize(source: &str) -> Option<Self> {
        let lines = text_lines(source).collect::<Vec<_>>();
        let mut structure = Self {
            items: Vec::new(),
            blocks: Vec::new(),
        };
        let mut inlines = MarkdownMultilineInlineScan::new(source);
        let mut current = None;
        let mut index = 0;
        while index < lines.len() {
            let line = lines[index];
            if line.body.trim().is_empty() {
                let start = index;
                while index < lines.len() && lines[index].body.trim().is_empty() {
                    index += 1;
                }
                structure.blocks.push(ListBlock {
                    item: current?,
                    kind: ListBlockKind::Blank(line_span(&lines, start, index)),
                });
                continue;
            }

            let indent = leading_spaces(line.body);
            if index > 0
                && lines[index - 1].body.trim().is_empty()
                && current.is_some_and(|id| indent >= structure.items[id].marker_content_indent + 4)
            {
                // A checkbox is inline content. Its wrapping alignment cannot
                // raise the structural indentation of the enclosing item. After
                // a blank line, four additional spaces can start indented code,
                // including text that looks like another marker or a fence.
                return None;
            }
            let content = &line.body[indent..];
            let inline = inlines.covers(line.body_start + indent);
            let marker = (!inline).then(|| markdown_list_marker(content)).flatten();
            let first = if let Some((marker, rest)) = marker {
                let mut parent = current;
                while let Some(id) = parent {
                    let item: &ListItem = &structure.items[id];
                    if indent >= item.marker_content_indent {
                        break;
                    }
                    // A shifted sibling can become a child after marker
                    // normalization. Preserve this existing ambiguity boundary.
                    if indent > item.indent {
                        return None;
                    }
                    parent = item.parent;
                }
                let marker_start = line.body_start + indent;
                let marker_content_indent = line.body.len() - rest.trim_start().len();
                let body = rest.trim_start();
                let checkbox = task_checkbox(body).map(|checkbox| {
                    SourceSpan::new(Span::new(
                        line.body_start + marker_content_indent,
                        line.body_start + marker_content_indent + checkbox.len(),
                    ))
                });
                let prose = checkbox.map_or(body, |span| body[span.len()..].trim_start());
                let id = structure.items.len();
                structure.items.push(ListItem {
                    parent,
                    marker: SourceSpan::new(Span::new(marker_start, marker_start + marker.len())),
                    checkbox,
                    indent,
                    content_indent: line.body.len() - prose.len(),
                    continuation_min: indent
                        + marker.len()
                        + 1
                        + checkbox.map_or(0, |span| span.len() + 1),
                    marker_content_indent,
                });
                current = Some(id);
                true
            } else {
                false
            };
            let item_id = current?;
            let item = &structure.items[item_id];

            if !first && !inline && crate::core::wrap::atx_heading_start(content) {
                // Child headings are an existing preservation boundary. Decline
                // during recognition instead of retaining the rest of a large
                // unsupported list before the document capability check runs.
                return None;
            }
            if !first && !inline && rich_child_start(content) {
                if indent < item.marker_content_indent
                    || content.starts_with('>') && indent != item.continuation_min
                {
                    return None;
                }
                let end = child_end(&lines, index, indent);
                if lines[index..end]
                    .iter()
                    .any(|line| !line.body.trim().is_empty() && leading_spaces(line.body) < indent)
                {
                    return None;
                }
                let span = line_span(&lines, index, end);
                structure.blocks.push(ListBlock {
                    item: item_id,
                    kind: ListBlockKind::Child {
                        source: span,
                        indent,
                    },
                });
                inlines.skip_to(span.end());
                index = end;
                continue;
            }

            // A separate prose block after a nested item remains ambiguous in
            // the supported dialect, whether it resumes that item or an ancestor.
            // Recording ownership does not silently widen this existing boundary.
            if !first && (item.parent.is_some() || !continuation_supported(item, indent)) {
                return None;
            }
            let start = index;
            let content_start = line.body_start + if first { item.content_indent } else { indent };
            index += 1;
            while index < lines.len() && !lines[index].body.trim().is_empty() {
                let candidate = lines[index];
                let indent = leading_spaces(candidate.body);
                let body = &candidate.body[indent..];
                let inline = inlines.covers(candidate.body_start + indent);
                if !inline && crate::core::wrap::atx_heading_start(body) {
                    return None;
                }
                if !inline && (markdown_list_marker(body).is_some() || rich_child_start(body)) {
                    break;
                }
                if !continuation_supported(item, indent) {
                    return None;
                }
                index += 1;
            }
            structure.blocks.push(ListBlock {
                item: item_id,
                kind: ListBlockKind::Paragraph {
                    source: line_span(&lines, start, index),
                    content_start,
                    first,
                },
            });
        }
        (!structure.items.is_empty()).then_some(structure)
    }
}

fn continuation_supported(item: &ListItem, indent: usize) -> bool {
    // Retain the supported continuation window. Deeper text can be indented
    // code; its interpretation is not inferred from a normalized output prefix.
    indent >= item.continuation_min && indent < item.continuation_min + 4
}

pub(crate) fn leading_spaces(text: &str) -> usize {
    text.bytes().take_while(|byte| *byte == b' ').count()
}

fn line_span(lines: &[TextLine<'_>], start: usize, end: usize) -> SourceSpan {
    let last = lines[end - 1];
    SourceSpan::new(Span::new(
        lines[start].body_start,
        last.body_start + last.full.len(),
    ))
}

fn task_checkbox(body: &str) -> Option<&str> {
    let bytes = body.as_bytes();
    (bytes.len() >= 3
        && bytes[0] == b'['
        && matches!(bytes[1], b' ' | b'x' | b'X')
        && bytes[2] == b']'
        && bytes.get(3).is_none_or(u8::is_ascii_whitespace))
    .then(|| &body[..3])
}

fn rich_child_start(text: &str) -> bool {
    text.starts_with('|')
        || super::grid_table_border(text).is_some()
        || text.starts_with("```")
        || text.starts_with("~~~")
        || text.starts_with("$$")
        || text.starts_with("\\[")
        || text.starts_with("\\begin{")
        || text.starts_with(":::")
        || text.starts_with("{{<")
        || text.starts_with("{{%")
        || text.starts_with('>')
}

fn child_end(lines: &[TextLine<'_>], start: usize, indent: usize) -> usize {
    let opener = &lines[start].body[indent..];
    if let Some(fence) = super::fences::code_fence_at(opener) {
        for (index, line) in lines.iter().enumerate().skip(start + 1) {
            let spaces = leading_spaces(line.body);
            if spaces >= indent {
                let trimmed = line.body[spaces..].trim_end();
                let len = trimmed
                    .bytes()
                    .take_while(|byte| *byte == fence.marker)
                    .count();
                if len >= fence.len && trimmed[len..].trim().is_empty() {
                    return index + 1;
                }
            }
        }
        return lines.len();
    }
    if let Some(marker_len) = super::quarto_div_opening(opener) {
        // A div can contain blank lines. Its body belongs to the div, not to
        // the list item. An unclosed div extends to the end of the list slice.
        let body = &lines[start + 1..];
        return super::find_quarto_div_closing(
            body.iter()
                .map(|line| line.body.get(indent..).unwrap_or("")),
            marker_len,
        )
        .map_or(lines.len(), |offset| start + 2 + offset);
    }
    let mut end = start + 1;
    while end < lines.len()
        && !lines[end].body.trim().is_empty()
        && leading_spaces(lines[end].body) >= indent
    {
        end += 1;
    }
    end
}
