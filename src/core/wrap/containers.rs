//! Container drafts retain the logical body and prefix policy separately so
//! wrapping and nested formatting cannot change list or quote membership.

use super::plan::{Condition, Draft, FragmentInput, Prefix};
use super::{
    MarkdownMultilineInlineScan, atx_heading_start, final_newline, markdown_block_start_line,
    markdown_html_block_start, markdown_line_bodies, markdown_reference_definition_start,
    setext_underline_line, strip_final_newline,
};
use crate::core::document::FormatOptions;
use crate::core::lines::{TextLine as MarkdownLine, text_lines as markdown_lines};
use crate::core::markdown_marker::markdown_list_marker;

pub fn format_markdown_list(source: &str, options: FormatOptions) -> String {
    prepare_markdown_list(source)
        .and_then(|mut draft| draft.resolve(source, options))
        .map(|plan| plan.emit(source))
        .unwrap_or_else(|| source.to_owned())
}

pub(crate) fn prepare_markdown_list(source: &str) -> Option<Draft> {
    let lines = markdown_lines(source).collect::<Vec<_>>();
    if list_needs_rich_format(&lines) {
        prepare_rich_markdown_list(source, &lines)
    } else {
        prepare_simple_markdown_list(source, &lines)
    }
}

fn prepare_simple_markdown_list(source: &str, lines: &[MarkdownLine<'_>]) -> Option<Draft> {
    let mut out = Draft::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        let body = line.body;
        let newline = line.newline;
        if body.trim().is_empty() {
            out.push_str(source, body);
            out.push_str(source, newline);
            index += 1;
            continue;
        }
        let indent_len = body.bytes().take_while(|byte| *byte == b' ').count();
        let indent = &body[..indent_len];
        let trimmed = body[indent_len..].trim_start();
        let (marker, rest) = list_marker(trimmed)?;
        let marker = if matches!(marker, "*" | "+") {
            "-"
        } else {
            marker
        };
        let item_body = rest.trim_start();
        let (first_prefix, first_content) =
            if let Some((checkbox, content)) = task_list_checkbox(item_body) {
                (format!("{indent}{marker} {checkbox} "), content)
            } else {
                (format!("{indent}{marker} "), item_body)
            };
        let continuation_prefix = " ".repeat(first_prefix.chars().count());
        let source_prefix_len = body.len() - first_content.len();
        index += 1;
        let mut paragraph_prefix = first_prefix;
        let mut paragraph_content = first_content;
        let mut paragraph_newline = newline;
        loop {
            let mut pieces = Vec::new();
            let mut nested_quote_start = None;
            let mut saw_nested_quote = false;
            pieces.push((paragraph_content, paragraph_newline));
            while index < lines.len() {
                let continuation_body = lines[index].body;
                if continuation_body.trim().is_empty() {
                    break;
                }
                let continuation_indent = continuation_body
                    .bytes()
                    .take_while(|byte| *byte == b' ')
                    .count();
                let continuation_trimmed = continuation_body[continuation_indent..].trim_start();
                if list_marker(continuation_trimmed).is_some() {
                    break;
                }
                if continuation_indent < continuation_prefix.len()
                    || continuation_indent >= continuation_prefix.len() + 4
                {
                    return None;
                }
                if continuation_trimmed.starts_with('>') {
                    if continuation_indent != continuation_prefix.len() {
                        return None;
                    }
                    saw_nested_quote = true;
                    nested_quote_start.get_or_insert(index);
                    index += 1;
                    continue;
                }
                if saw_nested_quote {
                    return None;
                }
                pieces.push((
                    &continuation_body[source_prefix_len.min(continuation_indent)..],
                    lines[index].newline,
                ));
                index += 1;
            }
            out.append(prepare_prefixed_markdown_lines(
                source,
                &pieces,
                &paragraph_prefix,
                &continuation_prefix,
                false,
            ));
            if let Some(start) = nested_quote_start {
                let first = lines[start].full;
                let last = lines[index - 1].full;
                let start = first.as_ptr() as usize - source.as_ptr() as usize;
                let end = last.as_ptr() as usize - source.as_ptr() as usize + last.len();
                let nested_quote = &source[start..end];
                out.push_scoped(
                    source,
                    nested_quote,
                    prepare_markdown_blockquote(nested_quote)?,
                );
            }
            if index >= lines.len() || !lines[index].body.trim().is_empty() {
                break;
            }

            let blank_start = index;
            while index < lines.len() && lines[index].body.trim().is_empty() {
                index += 1;
            }
            if index >= lines.len() {
                for blank in &lines[blank_start..index] {
                    out.push_str(source, blank.full);
                }
                break;
            }

            let continuation_body = lines[index].body;
            let continuation_indent = continuation_body
                .bytes()
                .take_while(|byte| *byte == b' ')
                .count();
            let continuation_trimmed = continuation_body[continuation_indent..].trim_start();
            if list_marker(continuation_trimmed).is_some()
                || continuation_indent < continuation_prefix.len()
                || continuation_indent >= continuation_prefix.len() + 4
            {
                index = blank_start;
                break;
            }

            for blank in &lines[blank_start..index] {
                out.push_str(source, blank.full);
            }
            paragraph_prefix = continuation_prefix.clone();
            paragraph_content = continuation_trimmed;
            paragraph_newline = lines[index].newline;
            index += 1;
        }
    }
    Some(out)
}

fn prepare_rich_markdown_list(source: &str, lines: &[MarkdownLine<'_>]) -> Option<Draft> {
    let mut out = Draft::new();
    let mut index = 0usize;
    let mut changed = false;
    while index < lines.len() {
        if let Some(indent) = rich_list_child_indent(lines[index].body) {
            let end = rich_list_child_block_end(lines, index, indent);
            let nested = strip_rich_list_child_indent(&lines[index..end], indent)?;
            out.child(
                FragmentInput::Buffer(std::sync::Arc::new(crate::core::source::SourceBuffer::new(
                    nested,
                ))),
                Prefix::Indent(indent),
                indent,
            );
            changed = true;
            index = end;
            continue;
        }
        let start = index;
        index += 1;
        while index < lines.len() && rich_list_child_indent(lines[index].body).is_none() {
            index += 1;
        }
        out.append(prepare_simple_markdown_list(source, &lines[start..index])?);
    }
    changed.then_some(out)
}

fn list_needs_rich_format(lines: &[MarkdownLine<'_>]) -> bool {
    lines
        .iter()
        .any(|line| rich_list_child_indent(line.body).is_some())
}

fn rich_list_child_indent(body: &str) -> Option<usize> {
    let indent = body.bytes().take_while(|byte| *byte == b' ').count();
    if indent == 0 || body.trim().is_empty() {
        return None;
    }
    let trimmed = body[indent..].trim_start();
    rich_child_block_start(trimmed).then_some(indent)
}

fn rich_list_child_block_end(lines: &[MarkdownLine<'_>], start: usize, indent: usize) -> usize {
    if code_fence_line(lines[start].body[indent..].trim_start()) {
        return rich_list_code_fence_end(lines, start, indent);
    }
    let mut index = start + 1;
    while index < lines.len() {
        let body = lines[index].body;
        if body.trim().is_empty() {
            break;
        }
        let line_indent = body.bytes().take_while(|byte| *byte == b' ').count();
        if line_indent < indent {
            break;
        }
        index += 1;
    }
    index
}

fn rich_list_code_fence_end(lines: &[MarkdownLine<'_>], start: usize, indent: usize) -> usize {
    let opener = lines[start].body[indent..].trim_start();
    let marker = opener.as_bytes().first().copied().unwrap_or(b'`');
    let marker_len = opener.bytes().take_while(|byte| *byte == marker).count();
    let mut index = start + 1;
    while index < lines.len() {
        let body = lines[index].body;
        let line_indent = body.bytes().take_while(|byte| *byte == b' ').count();
        if line_indent >= indent {
            let trimmed = body[line_indent..].trim();
            let close_len = trimmed.bytes().take_while(|byte| *byte == marker).count();
            if close_len >= marker_len && trimmed[close_len..].trim().is_empty() {
                return index + 1;
            }
        }
        index += 1;
    }
    lines.len()
}

fn strip_rich_list_child_indent(lines: &[MarkdownLine<'_>], indent: usize) -> Option<String> {
    let mut nested = String::new();
    for line in lines {
        if line.body.trim().is_empty() {
            nested.push_str(line.newline);
            continue;
        }
        let line_indent = line.body.bytes().take_while(|byte| *byte == b' ').count();
        if line_indent < indent {
            return None;
        }
        nested.push_str(&line.body[indent..]);
        nested.push_str(line.newline);
    }
    Some(nested)
}

fn code_fence_line(trimmed: &str) -> bool {
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn task_list_checkbox(body: &str) -> Option<(&str, &str)> {
    let bytes = body.as_bytes();
    if bytes.len() < 3
        || bytes[0] != b'['
        || !matches!(bytes[1], b' ' | b'x' | b'X')
        || bytes[2] != b']'
    {
        return None;
    }
    if bytes.get(3).is_some_and(|byte| !byte.is_ascii_whitespace()) {
        return None;
    }
    Some((&body[..3], body[3..].trim_start()))
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
        FragmentInput::Buffer(std::sync::Arc::new(crate::core::source::SourceBuffer::new(
            nested,
        ))),
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
        FragmentInput::Buffer(std::sync::Arc::new(crate::core::source::SourceBuffer::new(
            nested,
        ))),
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

fn list_marker(trimmed: &str) -> Option<(&str, &str)> {
    markdown_list_marker(trimmed)
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
