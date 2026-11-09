//! Container drafts retain the logical body and prefix policy separately so
//! wrapping and nested formatting cannot change list or quote membership.

use super::plan::{Condition, Draft, FragmentInput, Prefix};
use super::{
    MarkdownMultilineInlineScan, atx_heading_start, final_newline, markdown_block_start_line,
    markdown_html_block_start, markdown_line_bodies, markdown_reference_definition_start,
    setext_underline_line, strip_final_newline,
};
use crate::core::document::FormatOptions;
use crate::core::lines::text_lines as markdown_lines;
use crate::core::markdown_marker::markdown_list_marker;

pub fn format_markdown_list(source: &str, options: FormatOptions) -> String {
    prepare_markdown_list(source)
        .and_then(|mut draft| draft.resolve(source, options))
        .map(|plan| plan.emit(source))
        .unwrap_or_else(|| source.to_owned())
}

pub(crate) fn prepare_markdown_list(source: &str) -> Option<Draft> {
    let structure = crate::core::markdown::ListStructure::recognize(source)?;
    prepare_markdown_list_structure(source, &structure)
}

/// Layout consumes authored item and block boundaries; it never reclassifies
/// lines using the prefixes that it has just chosen for output.
pub(crate) fn prepare_markdown_list_structure(
    source: &str,
    structure: &crate::core::markdown::ListStructure,
) -> Option<Draft> {
    use crate::core::markdown::{ListBlockKind, leading_list_spaces};

    let mut out = Draft::new();
    for block in &structure.blocks {
        let item = &structure.items[block.item];
        match block.kind {
            ListBlockKind::Blank(span) => out.push_str(source, span.span().slice(source)),
            ListBlockKind::Paragraph {
                source: span,
                content_start,
                first,
            } => {
                if first
                    && let Some(parent) = item.parent
                    && ListPrefix::new(source, &structure.items[parent])
                        .makes_child_code(item.indent)
                {
                    return None;
                }
                let prefix = ListPrefix::new(source, item);
                let mut first_prefix = format!("{}{} ", " ".repeat(item.indent), prefix.marker);
                if let Some(checkbox) = item.checkbox {
                    first_prefix.push_str(checkbox.span().slice(source));
                    first_prefix.push(' ');
                }
                let continuation_prefix = " ".repeat(first_prefix.len());
                let pieces = markdown_lines(span.span().slice(source))
                    .enumerate()
                    .map(|(index, line)| {
                        let cut = if index == 0 {
                            content_start - span.start()
                        } else {
                            item.content_indent.min(leading_list_spaces(line.body))
                        };
                        (&line.body[cut..], line.newline)
                    })
                    .collect::<Vec<_>>();
                out.append(prepare_prefixed_markdown_lines(
                    source,
                    &pieces,
                    if first {
                        &first_prefix
                    } else {
                        &continuation_prefix
                    },
                    &continuation_prefix,
                    false,
                ));
            }
            ListBlockKind::Child {
                source: span,
                indent,
            } => {
                let prefix = ListPrefix::new(source, item);
                let opener = &span.span().slice(source)[indent..];
                if prefix.makes_child_code(indent) || prefix.misaligns_div(opener, indent) {
                    return None;
                }
                let mut nested = String::with_capacity(span.len());
                for line in markdown_lines(span.span().slice(source)) {
                    if !line.body.trim().is_empty() {
                        // Recognition proved the authored ASCII prefix exists.
                        nested.push_str(&line.body[indent..]);
                    }
                    nested.push_str(line.newline);
                }
                out.child(
                    FragmentInput::Buffer(std::sync::Arc::new(
                        crate::core::source::SourceBuffer::nested(nested),
                    )),
                    Prefix::Indent(indent),
                    indent,
                );
            }
        }
    }
    Some(out)
}

/// A marker's output geometry is layout policy. In particular, checkbox text
/// affects prose alignment but does not change this structural content column.
struct ListPrefix<'a> {
    marker: &'a str,
    content_indent: usize,
}

impl<'a> ListPrefix<'a> {
    fn new(source: &'a str, item: &crate::core::markdown::ListItem) -> Self {
        let authored = item.marker.span().slice(source);
        let marker = if matches!(authored, "*" | "+") {
            "-"
        } else {
            authored
        };
        Self {
            marker,
            content_indent: item.indent + marker.len() + 1,
        }
    }

    fn makes_child_code(&self, child_indent: usize) -> bool {
        // Child prefixes currently retain authored indentation. Narrowing the
        // parent marker must not move such a child across the four-space code
        // boundary. Preserve the list until a separate reindent policy exists.
        child_indent >= self.content_indent + 4
    }

    fn misaligns_div(&self, opener: &str, child_indent: usize) -> bool {
        // Pandoc recognizes a fenced div only at the item's content column.
        // A retained div prefix cannot follow a narrowed marker.
        opener.starts_with(":::") && child_indent != self.content_indent
    }
}

pub fn format_markdown_definition_list(source: &str, options: FormatOptions) -> String {
    prepare_markdown_definition_list(source)
        .and_then(|mut draft| draft.resolve(source, options))
        .map(|plan| plan.emit(source))
        .unwrap_or_else(|| source.to_owned())
}

pub(crate) fn prepare_markdown_definition_list(source: &str) -> Option<Draft> {
    let lines = markdown_lines(source).collect::<Vec<_>>();
    let mut out = Draft::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        let body = line.body;
        let newline = line.newline;
        let Some((prefix, content)) = definition_marker_parts(body) else {
            out.push_str(source, body.trim_end());
            out.push_str(source, newline);
            index += 1;
            continue;
        };
        let continuation_prefix = definition_continuation_indent(&prefix);
        let source_prefix_len = body.len() - content.len();
        let mut pieces = vec![(content.trim_start(), newline)];
        index += 1;
        while index < lines.len() {
            let continuation = lines[index].body;
            if continuation.trim().is_empty() || definition_marker_parts(continuation).is_some() {
                break;
            }
            let indent = continuation.len() - continuation.trim_start().len();
            if indent < 4 {
                break;
            }
            // Eligibility follows the parser's byte-based whitespace count.
            // Tabs can supply the four-byte structural minimum; beyond that,
            // strip only authored spaces, leaving literal tabs as content.
            let spaces = continuation
                .bytes()
                .take_while(|byte| *byte == b' ')
                .count();
            let prefix_len = continuation.floor_char_boundary(source_prefix_len.min(spaces.max(4)));
            pieces.push((&continuation[prefix_len..], lines[index].newline));
            index += 1;
        }
        out.append(prepare_prefixed_markdown_lines(
            source,
            &pieces,
            &prefix,
            &continuation_prefix,
            false,
        ));
    }
    Some(out)
}

pub fn format_markdown_blockquote(source: &str, options: FormatOptions) -> String {
    prepare_markdown_blockquote(source)
        .and_then(|mut draft| draft.resolve(source, options))
        .map(|plan| plan.emit(source))
        .unwrap_or_else(|| source.to_owned())
}

pub(crate) fn prepare_markdown_blockquote(source: &str) -> Option<Draft> {
    Some(Draft::fallback(
        prepare_simple_markdown_blockquote(source),
        prepare_rich_markdown_blockquote(source),
    ))
}

fn prepare_simple_markdown_blockquote(source: &str) -> Option<Draft> {
    if has_nested_blockquote_marker(source) {
        return prepare_nested_blockquote_markers(source);
    }
    if blockquote_needs_rich_format(source) {
        return None;
    }
    let mut indent = None::<&str>;
    let mut pieces = Vec::new();
    let mut saw_empty_quote_line = false;
    for line in markdown_lines(source) {
        let body = line.body;
        let line_newline = line.newline;
        let indent_len = body.bytes().take_while(|byte| *byte == b' ').count();
        let line_indent = &body[..indent_len];
        if let Some(indent) = indent {
            if indent != line_indent {
                return None;
            }
        } else {
            indent = Some(line_indent);
        }
        let trimmed = body[indent_len..].trim_start();
        let rest = trimmed.strip_prefix('>')?;
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        if rest.is_empty() {
            saw_empty_quote_line = true;
        } else if saw_empty_quote_line {
            return None;
        } else {
            pieces.push((rest, line_newline));
        }
    }
    let indent = indent.unwrap_or("");
    if saw_empty_quote_line {
        return None;
    }
    let prefix = format!("{indent}> ");
    Some(prepare_prefixed_markdown_lines(
        source, &pieces, &prefix, &prefix, false,
    ))
}

fn prepare_rich_markdown_blockquote(source: &str) -> Option<Draft> {
    if !blockquote_needs_rich_format(source) {
        return None;
    }
    let mut indent = None::<&str>;
    let mut nested = String::new();
    for line in markdown_lines(source) {
        let body = line.body;
        let indent_len = body.bytes().take_while(|byte| *byte == b' ').count();
        if indent_len > 3 {
            return None;
        }
        let line_indent = &body[..indent_len];
        if let Some(indent) = indent {
            if indent != line_indent {
                return None;
            }
        } else {
            indent = Some(line_indent);
        }
        let rest = body[indent_len..].strip_prefix('>')?;
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        nested.push_str(rest);
        nested.push_str(line.newline);
    }

    let indent = indent.unwrap_or("");
    let quote_prefix_width = indent.chars().count() + "> ".chars().count();
    let mut out = Draft::new();
    out.child(
        FragmentInput::Buffer(std::sync::Arc::new(
            crate::core::source::SourceBuffer::nested(nested),
        )),
        Prefix::Quote(indent.into()),
        quote_prefix_width,
    );
    Some(out)
}

fn blockquote_needs_rich_format(source: &str) -> bool {
    markdown_lines(source).any(|line| {
        let body = line.body;
        let indent_len = body.bytes().take_while(|byte| *byte == b' ').count();
        if indent_len > 3 {
            return false;
        }
        let Some(rest) = body[indent_len..].strip_prefix('>') else {
            return false;
        };
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        let trimmed = rest.trim_start();
        trimmed.is_empty() || rich_child_block_start(trimmed)
    })
}

fn rich_child_block_start(trimmed: &str) -> bool {
    trimmed.starts_with('|')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || trimmed.starts_with("$$")
        || trimmed.starts_with("\\[")
        || trimmed.starts_with("\\begin{")
        || trimmed.starts_with(":::")
        || trimmed.starts_with("{{<")
        || trimmed.starts_with("{{%")
        || markdown_list_marker(trimmed).is_some()
}

fn prepare_nested_blockquote_markers(source: &str) -> Option<Draft> {
    let mut indent = None::<&str>;
    let mut depth = None::<usize>;
    let mut pieces = Vec::new();
    for line in markdown_lines(source) {
        let (line_indent, line_depth, rest) = nested_blockquote_parts(line.body)?;
        if let Some(indent) = indent {
            if indent != line_indent {
                return prepare_varying_nested_quote_lines(source);
            }
        } else {
            indent = Some(line_indent);
        }
        if let Some(depth) = depth {
            if depth != line_depth {
                return prepare_varying_nested_quote_lines(source);
            }
        } else {
            depth = Some(line_depth);
        }
        if rest.trim().is_empty() {
            return prepare_varying_nested_quote_lines(source);
        }
        pieces.push((rest, line.newline));
    }
    let indent = indent.unwrap_or("");
    let depth = depth?;
    let prefix = format!("{indent}{}", "> ".repeat(depth));
    Some(prepare_prefixed_markdown_lines(
        source, &pieces, &prefix, &prefix, true,
    ))
}

fn prepare_varying_nested_quote_lines(source: &str) -> Option<Draft> {
    // Depth changes are separate container scopes. Main supports this path only
    // without reflow; each scope still uses the shared inline preparation.
    let mut out = Draft::new();
    for line in markdown_lines(source) {
        let (indent, depth, rest) = nested_blockquote_parts(line.body)?;
        let prefix = format!("{indent}{}", "> ".repeat(depth));
        out.append(prepare_prefixed_markdown_lines(
            source,
            &[(rest, line.newline)],
            &prefix,
            &prefix,
            true,
        ));
    }
    Some(Draft::choice(Condition::WrapNone, Some(out), None))
}

fn nested_blockquote_parts(body: &str) -> Option<(&str, usize, &str)> {
    let indent_len = body.bytes().take_while(|byte| *byte == b' ').count();
    if indent_len > 3 {
        return None;
    }
    let indent = &body[..indent_len];
    let mut rest = &body[indent_len..];
    let mut depth = 0usize;
    while let Some(after_marker) = rest.strip_prefix('>') {
        depth += 1;
        rest = after_marker.strip_prefix(' ').unwrap_or(after_marker);
        if rest.trim_start().starts_with('>') {
            rest = rest.trim_start();
        }
    }
    (depth > 1).then_some((indent, depth, rest))
}

fn has_nested_blockquote_marker(source: &str) -> bool {
    markdown_line_bodies(source).any(|line| blockquote_marker_depth(line) > 1)
}

fn blockquote_marker_depth(line: &str) -> usize {
    let mut rest = line.trim_start();
    let mut depth = 0usize;
    while let Some(after_marker) = rest.strip_prefix('>') {
        depth += 1;
        rest = after_marker.trim_start();
    }
    depth
}

pub(super) fn footnote_definition(source: &str) -> bool {
    let trimmed = source.trim_start();
    let indent = source.len() - trimmed.len();
    indent <= 3 && trimmed.starts_with("[^") && markdown_reference_definition_start(trimmed)
}

pub(super) fn prepare_markdown_footnote(source: &str) -> Option<Draft> {
    let newline = final_newline(source);
    let mut lines = markdown_lines(source);
    let first = lines.next()?;
    let first_line = first.body;
    let indent = first_line.bytes().take_while(|byte| *byte == b' ').count();
    if indent > 3 {
        return None;
    }
    let rest = &first_line[indent..];
    let label_end = rest.find("]:")? + 2;
    let content_start = label_end
        + rest[label_end..]
            .bytes()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count();
    let label = format!("{}{}", &first_line[..indent], &rest[..label_end]);
    let prefix = format!("{label} ");
    let continuation_prefix = " ".repeat(indent + 2);
    let block_continuation_prefix = " ".repeat(indent + 4);
    let mut pieces = Vec::new();
    let first_content = rest[content_start..].trim_start();
    let mut block_continuation_lines = Vec::new();
    let mut saw_blank_line = false;
    pieces.push((first_content, first.newline));
    for source_line in lines {
        let line = source_line.body;
        if line.trim().is_empty() {
            saw_blank_line = true;
            block_continuation_lines.push(Some(("", source_line.newline)));
            continue;
        }
        if line.bytes().take_while(|byte| *byte == b' ').count() < continuation_prefix.len() {
            return None;
        }
        let continuation = &line[continuation_prefix.len()..];
        pieces.push((continuation, source_line.newline));
        block_continuation_lines.push(
            line.strip_prefix(&block_continuation_prefix)
                .map(|body| (body, source_line.newline)),
        );
    }
    let block_continuation_refs = block_continuation_lines
        .iter()
        .copied()
        .collect::<Option<Vec<_>>>();
    if saw_blank_line
        || footnote_needs_recursive_body(
            first_content.trim_end(),
            block_continuation_refs.as_deref().unwrap_or(&[]),
        )
    {
        let block_continuation_refs = block_continuation_refs?;
        return Some(prepare_markdown_footnote_block(
            source,
            &label,
            (first_content, first.newline),
            &block_continuation_refs,
            &block_continuation_prefix,
            newline,
        ));
    }
    let mut out =
        prepare_prefixed_markdown_lines(source, &pieces, &prefix, &continuation_prefix, false);
    if newline.is_empty() {
        out.trim_final_newline();
    }
    Some(out)
}

fn footnote_needs_recursive_body(first_content: &str, continuation_lines: &[(&str, &str)]) -> bool {
    first_content.is_empty()
        || continuation_lines
            .iter()
            .any(|(body, _)| markdown_block_start_line(body))
}

fn prepare_markdown_footnote_block(
    source: &str,
    label: &str,
    first_line: (&str, &str),
    continuation_lines: &[(&str, &str)],
    continuation_prefix: &str,
    newline: &str,
) -> Draft {
    let mut nested = String::new();
    if !first_line.0.is_empty() {
        nested.push_str(first_line.0);
        nested.push_str(first_line.1);
    }
    for (body, newline) in continuation_lines {
        nested.push_str(body);
        nested.push_str(newline);
    }
    let mut out = Draft::new();
    out.push_str(source, label);
    out.push_join_newline(newline);
    out.child(
        FragmentInput::Buffer(std::sync::Arc::new(
            crate::core::source::SourceBuffer::nested(nested),
        )),
        Prefix::Footnote {
            indent: continuation_prefix.into(),
            newline: newline.into(),
        },
        continuation_prefix.chars().count(),
    );
    if newline.is_empty() {
        out.trim_final_newline();
    }
    out
}

fn definition_marker_parts(body: &str) -> Option<(String, &str)> {
    let trimmed = body.trim_start();
    let indent = body.len() - trimmed.len();
    if indent > 3 {
        return None;
    }
    let marker = *trimmed.as_bytes().first()?;
    if !matches!(marker, b':' | b'~') {
        return None;
    }
    if !trimmed
        .as_bytes()
        .get(1)
        .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    let marker_start = indent;
    let mut content_start = marker_start + 1;
    while body
        .as_bytes()
        .get(content_start)
        .is_some_and(u8::is_ascii_whitespace)
    {
        content_start += 1;
    }
    if content_start == marker_start + 1 {
        content_start += 1;
    }
    Some((
        format!("{}{} ", &body[..indent], marker as char),
        &body[content_start..],
    ))
}

fn definition_continuation_indent(prefix: &str) -> String {
    " ".repeat(prefix.chars().count())
}

fn prepare_prefixed_markdown_lines(
    source: &str,
    lines: &[(&str, &str)],
    first_prefix: &str,
    continuation_prefix: &str,
    preserve_lines: bool,
) -> Draft {
    let mut content = String::with_capacity(
        lines
            .iter()
            .map(|(body, newline)| body.len() + newline.len())
            .sum(),
    );
    for (body, newline) in lines {
        content.push_str(body);
        content.push_str(newline);
    }
    // Preserve authored block boundaries in containers, but allow prose around
    // heading-like text inside multiline literals and links to wrap normally.
    let mut inlines = MarkdownMultilineInlineScan::new(&content);
    let mut offset = 0;
    let mut paragraph_start = true;
    if lines.iter().any(|(body, newline)| {
        let trimmed = body.trim_start();
        let content_offset = offset + body.len() - trimmed.len();
        offset += body.len() + newline.len();
        // Type-7 HTML blocks cannot interrupt an existing paragraph. A lone
        // inline tag on a continuation line must not disable prose wrapping.
        let html_block_start = markdown_html_block_start(trimmed, paragraph_start);
        paragraph_start = trimmed.is_empty();
        (atx_heading_start(trimmed) || setext_underline_line(trimmed) || html_block_start)
            && !inlines.covers(content_offset)
    }) {
        let mut draft = Draft::new();
        let mut prefix = first_prefix;
        for (body, newline) in lines {
            draft.push_str(source, prefix);
            draft.push_str(source, body);
            draft.push_str(source, newline);
            prefix = continuation_prefix;
        }
        return draft;
    }
    let (body, _) = strip_final_newline(&content);
    let newline = lines.first().map_or("", |(_, newline)| *newline);
    if body.trim().is_empty() {
        let mut draft = Draft::text(source, first_prefix.trim_end());
        draft.push_join_newline(newline);
        return draft;
    }
    Draft::inline(
        source,
        body,
        first_prefix,
        continuation_prefix,
        newline,
        true,
        true,
        preserve_lines,
        false,
    )
}
