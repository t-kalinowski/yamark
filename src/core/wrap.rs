use crate::core::document::{FormatOptions, MarkdownWrap};
use crate::core::markdown_marker::markdown_list_marker;
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

pub fn format_markdown_paragraph(source: &str, options: FormatOptions) -> String {
    try_format_markdown_paragraph(source, options).unwrap_or_else(|| source.to_owned())
}

pub(crate) fn markdown_paragraph_format_supported(source: &str, options: FormatOptions) -> bool {
    try_format_markdown_paragraph(source, options).is_some()
}

fn try_format_markdown_paragraph(source: &str, options: FormatOptions) -> Option<String> {
    let (body, newline) = strip_final_newline(source);
    if footnote_definition(body) {
        return if options.markdown_format_footnotes {
            format_markdown_footnote(source, options)
        } else {
            Some(source.to_owned())
        };
    }
    if matches!(options.markdown_wrap, MarkdownWrap::None)
        && contains_existing_split_link_destination(body)
    {
        inline_tokens(body)?;
        let normalized = normalize_supported_links_and_images(source);
        return if options.markdown_canonical {
            Some(canonicalize_inline(&normalized))
        } else {
            Some(normalized.into_owned())
        };
    }
    if has_hard_break(body) {
        if matches!(options.markdown_wrap, MarkdownWrap::None) {
            inline_tokens(body)?;
            let normalized = normalize_inline_whitespace_preserving_lines(source);
            let normalized = normalize_supported_links_and_images(&normalized);
            return if options.markdown_canonical {
                Some(canonicalize_inline(&normalized))
            } else {
                Some(normalized.into_owned())
            };
        }
        return format_markdown_hard_break_paragraph(body, newline, options);
    }
    if matches!(options.markdown_wrap, MarkdownWrap::None) {
        inline_tokens(body)?;
        let normalized = normalize_inline_whitespace_preserving_lines(source);
        let normalized = normalize_supported_links_and_images(&normalized);
        return if options.markdown_canonical {
            Some(canonicalize_inline(&normalized))
        } else {
            Some(normalized.into_owned())
        };
    }
    let normalized_spaces = normalize_spaces_preserving_protected_spans(body);
    if normalized_spaces.is_empty() {
        return Some(source.to_owned());
    }
    let normalized_links = normalize_supported_links_and_images(&normalized_spaces);
    let text = if options.markdown_canonical {
        Cow::Owned(canonicalize_inline(&normalized_links))
    } else {
        normalized_links
    };
    let tokens = inline_tokens(&text)?;
    let join_newline = newline_for_join(newline, options);
    let mut out = String::with_capacity(text.len());
    let mut writer = TokenLineWriter::separated(&mut out, join_newline);
    match options.markdown_wrap {
        MarkdownWrap::None => writer.write_token_slice(&tokens, "")?,
        MarkdownWrap::Paragraph => writer.write_token_slice(&tokens, "")?,
        MarkdownWrap::Sentence => write_sentence_token_lines(&mut writer, &tokens, "")?,
        MarkdownWrap::Column => {
            let mut writer = writer.with_block_start_check();
            write_wrapped_tokens(
                &mut writer,
                &tokens,
                options.markdown_wrap_at_column.max(1),
                "",
            )?;
        }
    }
    out.push_str(newline);
    escape_first_markdown_block_start(&mut out);
    if single_line_body(body) && formatted_introduces_markdown_block_start(&out) {
        return None;
    }
    Some(out)
}

fn contains_existing_split_link_destination(source: &str) -> bool {
    for (index, _) in source.match_indices("](") {
        let destination_start = index + 2;
        let Some(destination_close) = find_simple_destination_close(source, destination_start)
        else {
            continue;
        };
        if source[destination_start..destination_close].contains(['\n', '\r']) {
            return true;
        }
    }
    false
}

pub fn normalize_heading_content(source: &str) -> String {
    let text = source.trim();
    let Some((body, attr)) = split_trailing_attribute(text) else {
        return text.to_owned();
    };
    let Some(attr) = normalize_heading_attribute_block(attr) else {
        return text.to_owned();
    };
    let body = trim_heading_closing_hashes(body);
    if body.trim().is_empty() {
        attr
    } else {
        format!("{} {attr}", body.trim_end())
    }
}

fn trim_heading_closing_hashes(source: &str) -> &str {
    let text = source.trim_end();
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

pub fn format_markdown_table(source: &str, options: FormatOptions) -> String {
    let newline = final_newline(source);
    let join_newline = newline_for_join(newline, options);
    let mut rows = markdown_line_bodies(source)
        .into_iter()
        .map(split_pipe_row)
        .collect::<Vec<_>>();
    if rows.len() < 2 || rows.iter().any(Vec::is_empty) {
        return source.to_owned();
    }
    let delimiter = rows.remove(1);
    let alignments = delimiter
        .iter()
        .map(|cell| table_alignment(cell))
        .collect::<Vec<_>>();
    let columns = rows
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .max(alignments.len());
    if columns == 0 {
        return source.to_owned();
    }
    for row in &mut rows {
        row.resize(columns, String::new());
    }
    canonicalize_table_rows(&mut rows, options);
    let mut widths = vec![0usize; columns];
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(display_width(cell));
        }
    }
    if !options.markdown_compact_tables {
        for (index, width) in widths.iter_mut().enumerate() {
            let alignment = alignments.get(index).copied().unwrap_or(Alignment::None);
            *width = (*width).max(delimiter_min_width(alignment));
        }
    }
    let mut out = render_table(
        &rows,
        &alignments,
        &widths,
        options.markdown_compact_tables,
        join_newline,
    );
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    out
}

pub fn format_markdown_pandoc_table(source: &str, options: FormatOptions) -> String {
    if let Some(formatted) = format_markdown_multiline_table(source, options) {
        return formatted;
    }
    if let Some(formatted) = format_markdown_grid_table(source, options) {
        return formatted;
    }

    let newline = final_newline(source);
    let join_newline = newline_for_join(newline, options);
    let lines = markdown_line_bodies(source);
    if lines.len() < 2 {
        return source.to_owned();
    }
    let Some(columns) = pandoc_separator_columns(lines[1]) else {
        return source.to_owned();
    };
    let has_closing_separator = lines.len() > 2
        && pandoc_separator_columns(lines[lines.len() - 1])
            .is_some_and(|closing| closing.len() == columns.len());
    let row_end = if has_closing_separator {
        lines.len() - 1
    } else {
        lines.len()
    };
    let mut rows = Vec::with_capacity(row_end.saturating_sub(1));
    for (index, line) in lines.iter().enumerate() {
        if index == 1 {
            continue;
        }
        if index >= row_end {
            break;
        }
        let Some(cells) = pandoc_table_cells(line, &columns) else {
            return source.to_owned();
        };
        rows.push(cells);
    }
    if rows.is_empty() {
        return source.to_owned();
    }
    canonicalize_table_rows(&mut rows, options);
    let mut widths = vec![0usize; columns.len()];
    for (index, column) in columns.iter().enumerate() {
        widths[index] = widths[index].max(column.min_width);
    }
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(display_width(cell));
        }
    }

    let mut out = String::new();
    if let Some((header, body)) = rows.split_first() {
        emit_pandoc_table_row(&mut out, header, &widths, join_newline);
        emit_pandoc_delimiter_row(&mut out, &columns, &widths, join_newline);
        for row in body {
            emit_pandoc_table_row(&mut out, row, &widths, join_newline);
        }
        if has_closing_separator {
            emit_pandoc_delimiter_row(&mut out, &columns, &widths, join_newline);
        }
    }
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    out
}

fn format_markdown_multiline_table(source: &str, options: FormatOptions) -> Option<String> {
    let newline = final_newline(source);
    let join_newline = newline_for_join(newline, options);
    let lines = markdown_line_bodies(source);
    if lines.len() < 4 {
        return None;
    }

    let (columns, header_lines, body_lines) = if pandoc_separator_token_line(lines[0]) {
        let columns = pandoc_separator_columns(lines[0])?;
        let separators = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| pandoc_separator_token_line(line).then_some(index))
            .collect::<Vec<_>>();
        if separators.len() != 3 || separators[0] != 0 {
            return None;
        }
        (
            columns,
            &lines[separators[0] + 1..separators[1]],
            &lines[separators[1] + 1..separators[2]],
        )
    } else if pandoc_continuous_multiline_bound(lines[0])
        && pandoc_continuous_multiline_bound(lines[lines.len() - 1])
    {
        let separator = lines[1..lines.len() - 1]
            .iter()
            .position(|line| pandoc_separator_token_line(line))?
            + 1;
        let columns = pandoc_separator_columns(lines[separator])?;
        (
            columns,
            &lines[1..separator],
            &lines[separator + 1..lines.len() - 1],
        )
    } else {
        return None;
    };

    let mut header_rows = pandoc_multiline_rows(header_lines, &columns)?;
    let mut body_rows = pandoc_multiline_rows(body_lines, &columns)?;
    if !header_rows.iter().any(Option::is_some) || !body_rows.iter().any(Option::is_some) {
        return None;
    }
    canonicalize_optional_table_rows(&mut header_rows, options);
    canonicalize_optional_table_rows(&mut body_rows, options);

    let mut widths = vec![0usize; columns.len()];
    for row in header_rows.iter().chain(body_rows.iter()).flatten() {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(display_width(cell));
        }
    }
    for (index, column) in columns.iter().enumerate() {
        widths[index] = widths[index].max(column.min_width);
    }

    let mut out = String::new();
    emit_pandoc_delimiter_row(&mut out, &columns, &widths, join_newline);
    for row in &header_rows {
        match row {
            Some(row) => emit_pandoc_table_row(&mut out, row, &widths, join_newline),
            None => out.push_str(join_newline),
        }
    }
    emit_pandoc_delimiter_row(&mut out, &columns, &widths, join_newline);
    for row in &body_rows {
        match row {
            Some(row) => emit_pandoc_table_row(&mut out, row, &widths, join_newline),
            None => out.push_str(join_newline),
        }
    }
    emit_pandoc_delimiter_row(&mut out, &columns, &widths, join_newline);
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    Some(out)
}

fn pandoc_multiline_rows(
    lines: &[&str],
    columns: &[PandocColumn],
) -> Option<Vec<Option<Vec<String>>>> {
    let mut rows = Vec::<Option<Vec<String>>>::new();
    for line in lines {
        if line.trim().is_empty() {
            rows.push(None);
            continue;
        }
        let cells = pandoc_table_cells(line, columns)?;
        push_pandoc_multiline_row(&mut rows, cells)?;
    }
    Some(rows)
}

fn push_pandoc_multiline_row(
    rows: &mut Vec<Option<Vec<String>>>,
    cells: Vec<String>,
) -> Option<()> {
    let first_nonempty = cells.iter().position(|cell| !cell.is_empty())?;
    if first_nonempty > 0
        && let Some(previous) = rows.last_mut().and_then(Option::as_mut)
    {
        for (index, cell) in cells.into_iter().enumerate().skip(first_nonempty) {
            if cell.is_empty() {
                continue;
            }
            if !previous[index].is_empty() {
                previous[index].push(' ');
            }
            previous[index].push_str(&cell);
        }
        return Some(());
    }
    rows.push(Some(cells));
    Some(())
}

fn pandoc_continuous_multiline_bound(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|ch| matches!(ch, '-' | '='))
}

fn pandoc_separator_token_line(line: &str) -> bool {
    let columns = line.split_whitespace().collect::<Vec<_>>();
    columns.len() >= 2 && columns.iter().all(|column| pandoc_separator_token(column))
}

fn format_markdown_grid_table(source: &str, options: FormatOptions) -> Option<String> {
    let newline = final_newline(source);
    let join_newline = newline_for_join(newline, options);
    let lines = markdown_line_bodies(source);
    if lines.len() < 3 {
        return None;
    }
    let column_count = grid_border_columns(lines[0])?;
    let mut sections = Vec::<GridSection>::new();
    let mut index = 1usize;
    while index < lines.len() {
        let mut rows = Vec::new();
        loop {
            if index >= lines.len() {
                return None;
            }
            if let Some(border) = grid_border_kind(lines[index], column_count) {
                if rows.is_empty() {
                    return None;
                }
                sections.push(GridSection {
                    rows,
                    border_after: border,
                });
                index += 1;
                break;
            }
            let mut row = grid_row_cells(lines[index], column_count)?;
            canonicalize_table_row(&mut row, options);
            rows.push(row);
            index += 1;
        }
    }
    if sections.is_empty() {
        return None;
    }

    let mut widths = vec![0usize; column_count];
    for section in &sections {
        for row in &section.rows {
            for (column, cell) in row.iter().enumerate() {
                widths[column] = widths[column].max(display_width(cell));
            }
        }
    }

    let mut out = String::new();
    emit_grid_border(&mut out, &widths, GridBorderKind::Normal, join_newline);
    for section in &sections {
        for row in &section.rows {
            emit_grid_row(&mut out, row, &widths, join_newline);
        }
        emit_grid_border(&mut out, &widths, section.border_after, join_newline);
    }
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    Some(out)
}

#[derive(Debug)]
struct GridSection {
    rows: Vec<Vec<String>>,
    border_after: GridBorderKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridBorderKind {
    Normal,
    Strong,
}

fn grid_border_columns(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    if !trimmed.starts_with('+') || !trimmed.ends_with('+') {
        return None;
    }
    let mut columns = 0usize;
    for part in trimmed.split('+').skip(1) {
        if part.is_empty() {
            continue;
        }
        if !part.chars().all(|ch| matches!(ch, '-' | '=')) {
            return None;
        }
        columns += 1;
    }
    (columns > 0).then_some(columns)
}

fn grid_border_kind(line: &str, column_count: usize) -> Option<GridBorderKind> {
    (grid_border_columns(line)? == column_count).then_some(())?;
    if line.contains('=') {
        Some(GridBorderKind::Strong)
    } else {
        Some(GridBorderKind::Normal)
    }
}

fn grid_row_cells(line: &str, column_count: usize) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }
    let cells = trimmed[1..trimmed.len() - 1]
        .split('|')
        .map(|cell| normalize_spaces(cell.trim()))
        .collect::<Vec<_>>();
    (cells.len() == column_count).then_some(cells)
}

fn emit_grid_border(out: &mut String, widths: &[usize], kind: GridBorderKind, newline: &str) {
    let marker = match kind {
        GridBorderKind::Normal => '-',
        GridBorderKind::Strong => '=',
    };
    out.push('+');
    for width in widths {
        for _ in 0..width + 2 {
            out.push(marker);
        }
        out.push('+');
    }
    out.push_str(newline);
}

fn emit_grid_row(out: &mut String, row: &[String], widths: &[usize], newline: &str) {
    out.push('|');
    for (cell, width) in row.iter().zip(widths) {
        out.push(' ');
        out.push_str(cell);
        out.push_str(&" ".repeat(width.saturating_sub(display_width(cell))));
        out.push(' ');
        out.push('|');
    }
    out.push_str(newline);
}

pub fn format_markdown_list(source: &str, options: FormatOptions) -> String {
    try_format_markdown_list(source, options).unwrap_or_else(|| source.to_owned())
}

pub(crate) fn markdown_list_format_supported(source: &str, options: FormatOptions) -> bool {
    try_format_markdown_list(source, options).is_some()
}

fn try_format_markdown_list(source: &str, options: FormatOptions) -> Option<String> {
    let lines = markdown_lines(source);
    if list_needs_rich_format(&lines) {
        try_format_rich_markdown_list(&lines, options)
    } else {
        try_format_simple_markdown_list(&lines, options)
    }
}

fn try_format_simple_markdown_list(
    lines: &[MarkdownLine<'_>],
    options: FormatOptions,
) -> Option<String> {
    let mut out = String::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        let body = line.body;
        let newline = line.newline;
        if body.trim().is_empty() {
            out.push_str(body);
            out.push_str(newline);
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
        index += 1;
        let mut paragraph_prefix = first_prefix;
        let mut paragraph_content = first_content;
        let mut paragraph_newline = newline;
        loop {
            let mut pieces = Vec::new();
            let mut nested_quote = String::new();
            let mut saw_nested_quote = false;
            if !paragraph_content.trim().is_empty() {
                pieces.push(paragraph_content);
            }
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
                    nested_quote.push_str(lines[index].full);
                    index += 1;
                    continue;
                }
                if saw_nested_quote {
                    return None;
                }
                pieces.push(continuation_trimmed);
                index += 1;
            }
            let segments = markdown_hard_break_segments(pieces.iter().copied());
            if segments.is_empty() {
                out.push_str(paragraph_prefix.trim_end());
                out.push_str(paragraph_newline);
            } else {
                out.push_str(&format_prefixed_markdown_segments(
                    &segments,
                    &paragraph_prefix,
                    &continuation_prefix,
                    paragraph_newline,
                    options,
                )?);
            }
            if !nested_quote.is_empty() {
                out.push_str(&try_format_markdown_blockquote(&nested_quote, options)?);
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
                    out.push_str(blank.full);
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
                out.push_str(blank.full);
            }
            paragraph_prefix = continuation_prefix.clone();
            paragraph_content = continuation_trimmed;
            paragraph_newline = lines[index].newline;
            index += 1;
        }
    }
    Some(out)
}

fn try_format_rich_markdown_list(
    lines: &[MarkdownLine<'_>],
    options: FormatOptions,
) -> Option<String> {
    let mut out = String::new();
    let mut index = 0usize;
    let mut changed = false;
    while index < lines.len() {
        if let Some(indent) = rich_list_child_indent(lines[index].body) {
            let end = rich_list_child_block_end(lines, index, indent);
            let nested = strip_rich_list_child_indent(&lines[index..end], indent)?;
            let formatted = format_markdown_fragment(&nested, options);
            out.push_str(&reindent_markdown_fragment(&formatted, indent));
            changed = true;
            index = end;
            continue;
        }
        if let Some(formatted) = try_format_single_markdown_list_line(lines[index], options) {
            changed |= formatted != lines[index].full;
            out.push_str(&formatted);
        } else {
            out.push_str(lines[index].full);
        }
        index += 1;
    }
    changed.then_some(out)
}

fn try_format_single_markdown_list_line(
    line: MarkdownLine<'_>,
    options: FormatOptions,
) -> Option<String> {
    let body = line.body;
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
    let (prefix, content) = if let Some((checkbox, content)) = task_list_checkbox(item_body) {
        (format!("{indent}{marker} {checkbox} "), content)
    } else {
        (format!("{indent}{marker} "), item_body)
    };
    let continuation_prefix = " ".repeat(prefix.chars().count());
    let (content, hard_break) = markdown_hard_break_line_content(content);
    let text = normalize_spaces_preserving_protected_spans(content);
    if text.is_empty() {
        Some(format!(
            "{}{newline}",
            prefix.trim_end(),
            newline = line.newline
        ))
    } else {
        let segment = MarkdownHardBreakSegment {
            text: text.into_owned(),
            hard_break,
        };
        format_prefixed_markdown_segment(
            &segment,
            &prefix,
            &continuation_prefix,
            line.newline,
            options,
        )
    }
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

fn reindent_markdown_fragment(fragment: &str, indent: usize) -> String {
    let prefix = " ".repeat(indent);
    let mut out = String::new();
    for line in markdown_lines(fragment) {
        if line.body.is_empty() {
            out.push_str(line.newline);
            continue;
        }
        out.push_str(&prefix);
        out.push_str(line.body);
        out.push_str(line.newline);
    }
    out
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
    try_format_markdown_definition_list(source, options).unwrap_or_else(|| source.to_owned())
}

pub(crate) fn markdown_definition_list_format_supported(
    source: &str,
    options: FormatOptions,
) -> bool {
    try_format_markdown_definition_list(source, options).is_some()
}

fn try_format_markdown_definition_list(source: &str, options: FormatOptions) -> Option<String> {
    let lines = markdown_lines(source);
    let mut out = String::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        let body = line.body;
        let newline = line.newline;
        let Some((prefix, content)) = definition_marker_parts(body) else {
            out.push_str(body.trim_end());
            out.push_str(newline);
            index += 1;
            continue;
        };
        let mut pieces = vec![content.trim_start()];
        index += 1;
        while index < lines.len() {
            let continuation = lines[index].body;
            if continuation.trim().is_empty() || definition_marker_parts(continuation).is_some() {
                break;
            }
            let trimmed = continuation.trim_start();
            let indent = continuation.len() - trimmed.len();
            if indent < 4 {
                break;
            }
            pieces.push(trimmed);
            index += 1;
        }
        let segments = markdown_hard_break_segments(pieces.iter().copied());
        if segments.is_empty() {
            out.push_str(prefix.trim_end());
            out.push_str(newline);
            continue;
        }
        let continuation_prefix = definition_continuation_indent(&prefix);
        out.push_str(&format_prefixed_markdown_segments(
            &segments,
            &prefix,
            &continuation_prefix,
            newline,
            options,
        )?);
    }
    Some(out)
}

pub fn format_markdown_blockquote(source: &str, options: FormatOptions) -> String {
    try_format_markdown_blockquote(source, options).unwrap_or_else(|| source.to_owned())
}

pub(crate) fn markdown_blockquote_format_supported(source: &str, options: FormatOptions) -> bool {
    try_format_markdown_blockquote(source, options).is_some()
}

fn try_format_markdown_blockquote(source: &str, options: FormatOptions) -> Option<String> {
    try_format_simple_markdown_blockquote(source, options)
        .or_else(|| try_format_rich_markdown_blockquote(source, options))
}

fn try_format_simple_markdown_blockquote(source: &str, options: FormatOptions) -> Option<String> {
    if has_nested_blockquote_marker(source) {
        return format_nested_blockquote_markers(source, options);
    }
    if blockquote_needs_rich_format(source) {
        return None;
    }
    let mut indent = None::<&str>;
    let mut newline = "";
    let mut pieces = Vec::new();
    let mut saw_empty_quote_line = false;
    for line in markdown_lines(source) {
        let body = line.body;
        let line_newline = line.newline;
        if newline.is_empty() {
            newline = line_newline;
        }
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
        let rest = rest.trim_start();
        if rest.is_empty() {
            saw_empty_quote_line = true;
        } else if saw_empty_quote_line {
            return None;
        } else {
            pieces.push(rest);
        }
    }
    let indent = indent.unwrap_or("");
    if saw_empty_quote_line {
        return None;
    }
    let prefix = format!("{indent}> ");
    let segments = markdown_hard_break_segments(pieces.iter().copied());
    if segments.is_empty() {
        Some(format!("{indent}>{newline}"))
    } else {
        format_prefixed_markdown_segments(&segments, &prefix, &prefix, newline, options)
    }
}

fn try_format_rich_markdown_blockquote(source: &str, options: FormatOptions) -> Option<String> {
    if !blockquote_needs_rich_format(source) {
        return None;
    }
    let lines = markdown_lines(source);
    let mut indent = None::<&str>;
    let mut nested = String::new();
    for line in &lines {
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
    let mut nested_options = options;
    if matches!(nested_options.markdown_wrap, MarkdownWrap::Column) {
        nested_options.markdown_wrap_at_column = nested_options
            .markdown_wrap_at_column
            .saturating_sub(quote_prefix_width)
            .max(1);
    }
    let formatted = format_markdown_fragment(&nested, nested_options);
    let mut out = String::new();
    for line in markdown_lines(&formatted) {
        out.push_str(indent);
        out.push('>');
        if !line.body.is_empty() {
            out.push(' ');
            out.push_str(line.body);
        }
        out.push_str(line.newline);
    }
    Some(out)
}

fn blockquote_needs_rich_format(source: &str) -> bool {
    markdown_lines(source).into_iter().any(|line| {
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

fn format_nested_blockquote_markers(source: &str, options: FormatOptions) -> Option<String> {
    if matches!(options.markdown_wrap, MarkdownWrap::None) {
        return markdown_lines(source)
            .into_iter()
            .map(|line| {
                let body = line.body;
                let newline = line.newline;
                let (indent, depth, rest) = nested_blockquote_parts(body)?;
                let markers = "> ".repeat(depth);
                let prefix = format!("{indent}{markers}");
                let segments = markdown_hard_break_segments([rest.trim_start()]);
                if segments.is_empty() {
                    Some(format!("{indent}{}{newline}", markers.trim_end()))
                } else {
                    format_prefixed_markdown_segments(&segments, &prefix, &prefix, newline, options)
                }
            })
            .collect();
    }
    let mut indent = None::<&str>;
    let mut depth = None::<usize>;
    let mut newline = "";
    let mut pieces = Vec::new();
    for line in markdown_lines(source) {
        let (line_indent, line_depth, rest) = nested_blockquote_parts(line.body)?;
        if let Some(indent) = indent {
            if indent != line_indent {
                return None;
            }
        } else {
            indent = Some(line_indent);
        }
        if let Some(depth) = depth {
            if depth != line_depth {
                return None;
            }
        } else {
            depth = Some(line_depth);
        }
        if newline.is_empty() {
            newline = line.newline;
        }
        if rest.trim().is_empty() {
            return None;
        }
        pieces.push(rest.trim());
    }
    let indent = indent.unwrap_or("");
    let depth = depth?;
    let prefix = format!("{indent}{}", "> ".repeat(depth));
    let segments = markdown_hard_break_segments(pieces.iter().copied());
    format_prefixed_markdown_segments(&segments, &prefix, &prefix, newline, options)
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
        rest = after_marker.trim_start();
    }
    (depth > 1).then_some((indent, depth, rest))
}

fn has_nested_blockquote_marker(source: &str) -> bool {
    markdown_line_bodies(source)
        .into_iter()
        .any(|line| blockquote_marker_depth(line) > 1)
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

pub fn format_markdown_fragment(source: &str, options: FormatOptions) -> String {
    let buffer = crate::core::source::SourceBuffer::new(source.to_owned());
    let config = crate::config::Config::default();
    let plugins = crate::plugins::PluginRegistry::default();
    let range = crate::core::source::Span::new(0, source.len());
    let Ok(document) = crate::core::markdown::parse_markdown(&buffer, range, options, &config)
    else {
        return source.to_owned();
    };
    crate::core::emit::emit_document(&buffer, &document, options, &plugins)
        .unwrap_or_else(|_| source.to_owned())
}

fn normalize_inline_whitespace_preserving_lines(source: &str) -> String {
    markdown_lines(source)
        .into_iter()
        .map(|line| {
            let body = line.body;
            let newline = line.newline;
            let trimmed = body.trim_end_matches([' ', '\t']);
            let trailing_spaces = body
                .as_bytes()
                .iter()
                .rev()
                .take_while(|byte| **byte == b' ')
                .count();
            if trailing_spaces >= 2 && !trimmed.is_empty() {
                format!("{trimmed} \\{newline}")
            } else {
                format!("{trimmed}{newline}")
            }
        })
        .collect()
}

fn format_markdown_hard_break_paragraph(
    body: &str,
    final_newline: &str,
    options: FormatOptions,
) -> Option<String> {
    let segments = markdown_hard_break_segments(markdown_line_bodies(body));
    if segments.is_empty() {
        return Some(body.to_owned());
    }
    let join_newline = newline_for_join(final_newline, options);
    let mut out = format_prefixed_markdown_segments(&segments, "", "", join_newline, options)?;
    if final_newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    escape_first_markdown_block_start(&mut out);
    if single_line_body(body) && formatted_introduces_markdown_block_start(&out) {
        return None;
    }
    Some(out)
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownHardBreakSegment {
    text: String,
    hard_break: Option<MarkdownHardBreakMarker>,
}

fn markdown_hard_break_segments<'a>(
    lines: impl IntoIterator<Item = &'a str>,
) -> Vec<MarkdownHardBreakSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    for line in lines {
        let (content, hard_break) = markdown_hard_break_line_content(line);
        let content = normalize_spaces_preserving_protected_spans(content);
        if !content.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&content);
        }
        if let Some(marker) = hard_break
            && !current.is_empty()
        {
            segments.push(MarkdownHardBreakSegment {
                text: std::mem::take(&mut current),
                hard_break: Some(marker),
            });
        }
    }
    if !current.is_empty() {
        segments.push(MarkdownHardBreakSegment {
            text: current,
            hard_break: None,
        });
    }
    segments
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
    markdown_line_bodies(source)
        .into_iter()
        .any(|line| line.ends_with("  ") || line.ends_with('\\'))
}

fn single_line_body(source: &str) -> bool {
    !source.contains('\n') && !source.contains('\r')
}

fn formatted_introduces_markdown_block_start(source: &str) -> bool {
    let (body, _) = strip_final_newline(source);
    let lines = markdown_line_bodies(body);
    if lines.is_empty() {
        return false;
    }
    lines.into_iter().skip(1).any(markdown_block_start_line)
}

fn escape_first_markdown_block_start(source: &mut String) {
    let first_line_end = source.find(['\n', '\r']).unwrap_or(source.len());
    let first_line = &source[..first_line_end];
    if !markdown_block_start_line(first_line) {
        return;
    }
    let marker = first_line
        .char_indices()
        .find(|(_, ch)| !matches!(ch, ' ' | '\t'))
        .map(|(index, _)| index)
        .unwrap_or(0);
    source.insert(marker, '\\');
}

fn markdown_block_start_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    indent <= 3
        && (atx_heading_start(trimmed)
            || thematic_break_line(trimmed)
            || trimmed.starts_with('>')
            || trimmed.starts_with("```")
            || trimmed.starts_with("~~~")
            || trimmed.starts_with("$$")
            || trimmed.starts_with("\\[")
            || trimmed.starts_with("\\begin{")
            || trimmed.starts_with('|')
            || trimmed.starts_with("Table:")
            || trimmed.starts_with(':')
            || trimmed.starts_with("<!--")
            || markdown_html_block_start_line(trimmed)
            || trimmed.starts_with("[") && trimmed.contains("]:")
            || markdown_list_marker(trimmed).is_some())
}

fn markdown_html_block_start_line(trimmed: &str) -> bool {
    trimmed.starts_with("<!") || trimmed.starts_with("<?") || inline_html_tag_at(trimmed, 0)
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

fn footnote_definition(source: &str) -> bool {
    let trimmed = source.trim_start();
    let indent = source.len() - trimmed.len();
    indent <= 3 && trimmed.starts_with("[^") && trimmed.contains("]:")
}

fn format_markdown_footnote(source: &str, options: FormatOptions) -> Option<String> {
    let (body, newline) = strip_final_newline(source);
    let lines = markdown_line_bodies(body);
    let first_line = *lines.first()?;
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
    pieces.push(first_content);
    for line in lines.into_iter().skip(1) {
        if line.trim().is_empty() {
            saw_blank_line = true;
            block_continuation_lines.push(Some(""));
            continue;
        }
        if line.bytes().take_while(|byte| *byte == b' ').count() < continuation_prefix.len() {
            return None;
        }
        let continuation = line[continuation_prefix.len()..].trim_start();
        pieces.push(continuation);
        block_continuation_lines.push(
            line.strip_prefix(&block_continuation_prefix)
                .map(str::trim_end),
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
        return Some(format_markdown_footnote_block(
            &label,
            first_content.trim_end(),
            &block_continuation_refs,
            &block_continuation_prefix,
            newline,
            options,
        ));
    }
    let segments = markdown_hard_break_segments(pieces.iter().copied());
    let join_newline = newline_for_join(newline, options);
    let mut out = format_prefixed_markdown_segments(
        &segments,
        &prefix,
        &continuation_prefix,
        join_newline,
        options,
    )?;
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    Some(out)
}

fn footnote_needs_recursive_body(first_content: &str, continuation_lines: &[&str]) -> bool {
    first_content.is_empty()
        || continuation_lines
            .iter()
            .any(|line| markdown_block_start_line(line))
}

fn format_markdown_footnote_block(
    label: &str,
    first_content: &str,
    continuation_lines: &[&str],
    continuation_prefix: &str,
    newline: &str,
    options: FormatOptions,
) -> String {
    let join_newline = newline_for_join(newline, options);
    let mut nested = String::new();
    if !first_content.is_empty() {
        nested.push_str(first_content);
        nested.push_str(join_newline);
    }
    for line in continuation_lines {
        nested.push_str(line);
        nested.push_str(join_newline);
    }

    let formatted = format_markdown_fragment(&nested, options);
    let mut out = String::new();
    out.push_str(label);
    out.push_str(join_newline);
    for line in markdown_lines(&formatted) {
        let body = line.body;
        let line_newline = line.newline;
        if body.is_empty() {
            out.push_str(line_newline);
            continue;
        }
        out.push_str(continuation_prefix);
        out.push_str(body);
        out.push_str(if line_newline.is_empty() {
            join_newline
        } else {
            line_newline
        });
    }
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    out
}

fn newline_for_join(final_newline: &str, options: FormatOptions) -> &str {
    match final_newline {
        "\r\n" => "\r\n",
        "\r" => "\r",
        "\n" => "\n",
        _ => options.default_line_ending,
    }
}

#[derive(Debug, Clone)]
struct InlineToken<'a> {
    text: Cow<'a, str>,
    width: usize,
}

impl<'a> InlineToken<'a> {
    fn borrowed(text: &'a str) -> Self {
        Self {
            text: Cow::Borrowed(text),
            width: token_width(text),
        }
    }

    fn owned(text: String) -> Self {
        let width = token_width(&text);
        Self {
            text: Cow::Owned(text),
            width,
        }
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn push_str(&mut self, suffix: &str) {
        self.text.to_mut().push_str(suffix);
        self.width += token_width(suffix);
    }
}

fn write_sentence_token_lines(
    out: &mut TokenLineWriter<'_, '_>,
    tokens: &[InlineToken<'_>],
    suffix: &str,
) -> Option<()> {
    let mut start = 0usize;
    let mut pending = None::<(usize, usize)>;
    for (index, token) in tokens.iter().enumerate() {
        if token_ends_sentence(token.text()) && !token_is_sentence_abbreviation(token.text()) {
            if let Some((from, to)) = pending.take() {
                out.write_token_slice(&tokens[from..to], "")?;
            }
            pending = Some((start, index + 1));
            start = index + 1;
        }
    }
    if start < tokens.len() {
        if let Some((from, to)) = pending.take() {
            out.write_token_slice(&tokens[from..to], "")?;
        }
        pending = Some((start, tokens.len()));
    }
    if let Some((from, to)) = pending {
        out.write_token_slice(&tokens[from..to], suffix)?;
    }
    Some(())
}

fn token_ends_sentence(token: &str) -> bool {
    token
        .trim_end_matches(['"', '\'', ')', ']', '}'])
        .chars()
        .next_back()
        .is_some_and(|ch| matches!(ch, '.' | '!' | '?'))
}

fn token_is_sentence_abbreviation(token: &str) -> bool {
    ascii_ends_with_ignore_case(token, "e.g.") || ascii_ends_with_ignore_case(token, "i.e.")
}

fn ascii_ends_with_ignore_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

pub fn canonicalize_inline(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    while index < source.len() {
        let rest = &source[index..];
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
        if rest.starts_with('`') {
            let tick_count = rest.bytes().take_while(|byte| *byte == b'`').count();
            let marker = &rest[..tick_count];
            if let Some(close) = rest[tick_count..].find(marker) {
                let end = index + tick_count + close + tick_count;
                out.push_str(&source[index..end]);
                index = end;
                continue;
            }
        }
        if rest.starts_with('$')
            && !escaped_at(source, index)
            && let Some(close) = rest[1..].find('$')
        {
            let end = index + 1 + close + 1;
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = paired_inline_html_span_end(source, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = protected_inline_token_end(source, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        let ch = rest.chars().next().expect("index is on a char boundary");
        if ch == '_'
            && !escaped_at(source, index)
            && let Some(span) = emphasis_span_at(source, index)
        {
            let delimiter = "*".repeat(span.run);
            out.push_str(&delimiter);
            out.push_str(&canonicalize_inline(&source[index + span.run..span.close]));
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

fn protected_inline_token_end(source: &str, index: usize) -> Option<usize> {
    let rest = &source[index..];
    if escaped_at(source, index) {
        return None;
    }
    if let Some(end) = reference_style_link_span_end(source, index) {
        return Some(end);
    }
    if rest.starts_with("![") || rest.starts_with('[') {
        return link_or_bracket_token_end(source, index);
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

fn reference_style_link_span_end(source: &str, index: usize) -> Option<usize> {
    let label_start = if source[index..].starts_with("![") {
        index + 2
    } else if source[index..].starts_with('[') {
        index + 1
    } else {
        return None;
    };
    let label_close = find_balanced_square_close(source, label_start)?;
    let reference_open = label_close + 1;
    if source.as_bytes().get(reference_open) != Some(&b'[') {
        return None;
    }
    let reference_close = find_balanced_square_close(source, reference_open + 1)?;
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

fn split_pipe_row(line: &str) -> Vec<String> {
    let (body, _) = strip_final_newline(line);
    let mut trimmed = body.trim();
    if let Some(rest) = trimmed.strip_prefix('|') {
        trimmed = rest;
    }
    if let Some(rest) = trimmed.strip_suffix('|') {
        trimmed = rest;
    }
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;
    let mut index = 0usize;
    while index < trimmed.len() {
        if !escaped
            && protected_spacing_span_can_start(trimmed, index)
            && let Some(end) = table_cell_protected_span_end(trimmed, index)
        {
            cell.push_str(&trimmed[index..end]);
            index = end;
            continue;
        }
        let ch = trimmed[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        if ch == '|' && !escaped {
            cells.push(normalize_spaces_preserving_protected_spans(cell.trim()).into_owned());
            cell.clear();
            index += ch.len_utf8();
            continue;
        }
        cell.push(ch);
        escaped = ch == '\\' && !escaped;
        if ch != '\\' {
            escaped = false;
        }
        index += ch.len_utf8();
    }
    cells.push(normalize_spaces_preserving_protected_spans(cell.trim()).into_owned());
    cells
}

fn table_cell_protected_span_end(text: &str, index: usize) -> Option<usize> {
    protected_spacing_span_end(text, index)
}

fn canonicalize_table_rows(rows: &mut [Vec<String>], options: FormatOptions) {
    if !options.markdown_canonical {
        return;
    }
    for row in rows {
        canonicalize_table_row(row, options);
    }
}

fn canonicalize_optional_table_rows(rows: &mut [Option<Vec<String>>], options: FormatOptions) {
    if !options.markdown_canonical {
        return;
    }
    for row in rows.iter_mut().flatten() {
        canonicalize_table_row(row, options);
    }
}

fn canonicalize_table_row(row: &mut [String], options: FormatOptions) {
    if !options.markdown_canonical {
        return;
    }
    for cell in row {
        *cell = canonicalize_inline(cell);
    }
}

fn render_table(
    rows: &[Vec<String>],
    alignments: &[Alignment],
    widths: &[usize],
    compact: bool,
    newline: &str,
) -> String {
    let mut out = String::new();
    emit_table_row(&mut out, &rows[0], alignments, widths, compact, newline);
    emit_delimiter_row(&mut out, alignments, widths, compact, newline);
    for row in rows.iter().skip(1) {
        emit_table_row(&mut out, row, alignments, widths, compact, newline);
    }
    out
}

#[derive(Debug, Clone)]
struct PandocColumn {
    start: usize,
    min_width: usize,
    alignment: Alignment,
}

fn pandoc_separator_columns(line: &str) -> Option<Vec<PandocColumn>> {
    let bytes = line.as_bytes();
    let mut columns = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let token = &line[start..index];
        if !pandoc_separator_token(token) {
            return None;
        }
        columns.push(PandocColumn {
            start,
            min_width: 3,
            alignment: table_alignment(token),
        });
    }
    (columns.len() >= 2).then_some(columns)
}

fn pandoc_separator_token(token: &str) -> bool {
    token.len() >= 3
        && token.chars().all(|ch| matches!(ch, '-' | '=' | ':'))
        && token.chars().any(|ch| matches!(ch, '-' | '='))
}

fn pandoc_table_cells(line: &str, columns: &[PandocColumn]) -> Option<Vec<String>> {
    let mut cells = Vec::with_capacity(columns.len());
    for (index, column) in columns.iter().enumerate() {
        let start = clamp_to_char_boundary(line, column.start.min(line.len()));
        let end = columns
            .get(index + 1)
            .map(|next| clamp_to_char_boundary(line, next.start.min(line.len())))
            .unwrap_or(line.len());
        if start > end {
            return None;
        }
        cells.push(normalize_spaces(line.get(start..end)?.trim()));
    }
    Some(cells)
}

fn clamp_to_char_boundary(line: &str, mut index: usize) -> usize {
    while index > 0 && !line.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn emit_pandoc_table_row(out: &mut String, row: &[String], widths: &[usize], newline: &str) {
    for (index, cell) in row.iter().enumerate() {
        if index > 0 {
            out.push_str("  ");
        }
        out.push_str(cell);
        if index + 1 < row.len() {
            let padding = widths[index].saturating_sub(display_width(cell));
            out.push_str(&" ".repeat(padding));
        }
    }
    out.push_str(newline);
}

fn emit_pandoc_delimiter_row(
    out: &mut String,
    columns: &[PandocColumn],
    widths: &[usize],
    newline: &str,
) {
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            out.push_str("  ");
        }
        out.push_str(&pandoc_delimiter_cell(
            widths[index].max(3),
            column.alignment,
        ));
    }
    out.push_str(newline);
}

fn pandoc_delimiter_cell(width: usize, alignment: Alignment) -> String {
    match alignment {
        Alignment::None => "-".repeat(width),
        Alignment::Left => format!(":{}", "-".repeat(width.saturating_sub(1).max(2))),
        Alignment::Right => format!("{}:", "-".repeat(width.saturating_sub(1).max(2))),
        Alignment::Center => {
            let middle = width.saturating_sub(2).max(1);
            format!(":{}:", "-".repeat(middle))
        }
    }
}

fn trim_trailing_line_ending(out: &mut String) {
    if out.ends_with("\r\n") {
        out.truncate(out.len() - 2);
    } else if out.ends_with('\n') || out.ends_with('\r') {
        out.pop();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    None,
    Left,
    Right,
    Center,
}

fn table_alignment(cell: &str) -> Alignment {
    let trimmed = cell.trim();
    let left = trimmed.starts_with(':');
    let right = trimmed.ends_with(':');
    match (left, right) {
        (true, true) => Alignment::Center,
        (true, false) => Alignment::Left,
        (false, true) => Alignment::Right,
        (false, false) => Alignment::None,
    }
}

fn delimiter_min_width(alignment: Alignment) -> usize {
    match alignment {
        Alignment::None => 3,
        Alignment::Left | Alignment::Right => 4,
        Alignment::Center => 5,
    }
}

fn emit_table_row(
    out: &mut String,
    row: &[String],
    alignments: &[Alignment],
    widths: &[usize],
    compact: bool,
    newline: &str,
) {
    out.push('|');
    for (index, cell) in row.iter().enumerate() {
        if compact {
            out.push(' ');
            out.push_str(cell);
            out.push(' ');
            out.push('|');
        } else {
            out.push(' ');
            let alignment = alignments.get(index).copied().unwrap_or(Alignment::None);
            emit_aligned_table_cell(out, cell, widths[index], alignment);
            out.push(' ');
            out.push('|');
        }
    }
    out.push_str(newline);
}

fn emit_aligned_table_cell(out: &mut String, cell: &str, width: usize, alignment: Alignment) {
    let padding = width.saturating_sub(display_width(cell));
    match alignment {
        Alignment::None | Alignment::Left => {
            out.push_str(cell);
            out.push_str(&" ".repeat(padding));
        }
        Alignment::Right => {
            out.push_str(&" ".repeat(padding));
            out.push_str(cell);
        }
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            out.push_str(&" ".repeat(left));
            out.push_str(cell);
            out.push_str(&" ".repeat(right));
        }
    }
}

fn emit_delimiter_row(
    out: &mut String,
    alignments: &[Alignment],
    widths: &[usize],
    compact: bool,
    newline: &str,
) {
    out.push('|');
    for (index, width) in widths.iter().copied().enumerate() {
        let alignment = alignments.get(index).copied().unwrap_or(Alignment::None);
        if !compact {
            out.push(' ');
        }
        match alignment {
            Alignment::None | Alignment::Left => {
                if compact {
                    out.push(' ');
                }
                if matches!(alignment, Alignment::Left) {
                    out.push(':');
                }
                let marker_width = usize::from(matches!(alignment, Alignment::Left));
                let dash_count = if compact {
                    3
                } else {
                    width.saturating_sub(marker_width)
                };
                out.push_str(&"-".repeat(dash_count.max(3)));
                if compact {
                    out.push(' ');
                }
            }
            Alignment::Right => {
                if compact {
                    out.push(' ');
                }
                let dash_count = if compact { 3 } else { width.saturating_sub(1) };
                out.push_str(&"-".repeat(dash_count.max(3)));
                out.push(':');
                if compact {
                    out.push(' ');
                }
            }
            Alignment::Center => {
                if compact {
                    out.push(' ');
                }
                out.push(':');
                let dash_count = if compact { 3 } else { width.saturating_sub(2) };
                out.push_str(&"-".repeat(dash_count.max(3)));
                out.push(':');
                if compact {
                    out.push(' ');
                }
            }
        }
        if !compact {
            out.push(' ');
        }
        out.push('|');
    }
    out.push_str(newline);
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

fn normalize_spaces(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut pending_space = false;
    for ch in source.chars() {
        if ch.is_whitespace() {
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
    if inline_spacing_is_already_normalized(source) {
        return Cow::Borrowed(source);
    }
    let mut out = String::with_capacity(source.len());
    let mut pending_space = false;
    let mut index = 0usize;
    while index < source.len() {
        if let Some(end) = protected_spacing_span_end(source, index) {
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
        if ch.is_whitespace() {
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
    if source.is_empty() || !source.is_ascii() {
        return source.is_empty();
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

fn protected_spacing_span_end(source: &str, index: usize) -> Option<usize> {
    reference_style_link_span_end(source, index)
        .or_else(|| balanced_brace_span_end(source, index))
        .or_else(|| strikethrough_span_end(source, index))
        .or_else(|| commonmark_autolink_span_end(source, index))
        .or_else(|| paired_inline_html_span_end(source, index))
        .or_else(|| inline_html_tag_span_end(source, index))
        .or_else(|| inline_code_span_end(source, index))
        .or_else(|| inline_math_span_end(source, index))
        .or_else(|| emphasis_span_end(source, index))
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

    fn markdown_block_start(&self) -> bool {
        token_slice_markdown_block_start(&self.tokens)
    }
}

#[derive(Debug, Clone, Copy)]
enum TokenLineWriteMode {
    Separated,
    Terminated,
}

struct TokenLineWriter<'out, 'prefix> {
    out: &'out mut String,
    first_prefix: &'prefix str,
    continuation_prefix: &'prefix str,
    newline: &'prefix str,
    mode: TokenLineWriteMode,
    check_block_starts: bool,
    line_count: usize,
}

impl<'out, 'prefix> TokenLineWriter<'out, 'prefix> {
    fn separated(out: &'out mut String, newline: &'prefix str) -> Self {
        Self {
            out,
            first_prefix: "",
            continuation_prefix: "",
            newline,
            mode: TokenLineWriteMode::Separated,
            check_block_starts: false,
            line_count: 0,
        }
    }

    fn terminated(
        out: &'out mut String,
        first_prefix: &'prefix str,
        continuation_prefix: &'prefix str,
        newline: &'prefix str,
    ) -> Self {
        Self {
            out,
            first_prefix,
            continuation_prefix,
            newline,
            mode: TokenLineWriteMode::Terminated,
            check_block_starts: false,
            line_count: 0,
        }
    }

    fn with_block_start_check(mut self) -> Self {
        self.check_block_starts = true;
        self
    }

    fn write_line(&mut self, line: &TokenLine<'_>, suffix: &str) -> Option<()> {
        self.write_token_slice(&line.tokens, suffix)
    }

    fn write_token_slice(&mut self, tokens: &[InlineToken<'_>], suffix: &str) -> Option<()> {
        if self.check_block_starts
            && self.line_count > 0
            && token_slice_markdown_block_start(tokens)
        {
            return None;
        }
        if matches!(self.mode, TokenLineWriteMode::Separated) && self.line_count > 0 {
            self.out.push_str(self.newline);
        }
        if self.line_count == 0 {
            self.out.push_str(self.first_prefix);
        } else {
            self.out.push_str(self.continuation_prefix);
        }
        write_joined_tokens(self.out, tokens);
        self.out.push_str(suffix);
        if matches!(self.mode, TokenLineWriteMode::Terminated) {
            self.out.push_str(self.newline);
        }
        self.line_count += 1;
        Some(())
    }
}

struct WrapLineBuffer<'a> {
    pending: Option<TokenLine<'a>>,
    current: TokenLine<'a>,
    width: usize,
    continuation_width: usize,
}

impl<'a> WrapLineBuffer<'a> {
    fn new(first_width: usize, continuation_width: usize) -> Self {
        Self {
            pending: None,
            current: TokenLine::default(),
            width: first_width.max(1),
            continuation_width: continuation_width.max(1),
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
        if self.current.is_empty() {
            return Some(());
        }
        let line = std::mem::take(&mut self.current);
        self.push_pending(writer, line)?;
        self.width = self.continuation_width;
        Some(())
    }

    fn finish(mut self, writer: &mut TokenLineWriter<'_, '_>, suffix: &str) -> Option<()> {
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
    ) -> Option<()> {
        let mut split = split.into_iter();
        let Some(first) = split.next() else {
            return Some(());
        };
        let first = InlineToken::owned(first);
        if self.current.can_fit(&first, self.width) {
            self.current.push(first);
        } else {
            self.commit_current(writer)?;
            self.current.push(first);
        }
        self.repair_current_line_if_markdown_block_start()?;

        let mut rest = split.map(InlineToken::owned).collect::<Vec<_>>();
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
        self.repair_current_line_if_markdown_block_start()?;
        Some(())
    }

    fn repair_current_line_if_markdown_block_start(&mut self) -> Option<()> {
        if self.pending.is_none() || self.current.is_empty() || !self.current.markdown_block_start()
        {
            return Some(());
        }
        let previous = self.pending.as_mut()?;
        for split in (1..previous.len()).rev() {
            let suffix_width = previous.suffix_width(split);
            let candidate_width = suffix_width + 1 + self.current.width;
            if candidate_width > self.continuation_width {
                continue;
            }
            let mut candidate = TokenLine::default();
            candidate.extend_from_slice(&previous.tokens[split..]);
            candidate.extend_from_slice(&self.current.tokens);
            if !candidate.markdown_block_start() {
                let mut moved = previous.split_off(split);
                moved.append(std::mem::take(&mut self.current));
                self.current = moved;
                return Some(());
            }
        }
        None
    }
}

fn write_wrapped_tokens(
    writer: &mut TokenLineWriter<'_, '_>,
    tokens: &[InlineToken<'_>],
    width: usize,
    suffix: &str,
) -> Option<()> {
    write_wrapped_tokens_with_first_width(writer, tokens, width, width, suffix)
}

fn write_wrapped_tokens_with_first_width(
    writer: &mut TokenLineWriter<'_, '_>,
    tokens: &[InlineToken<'_>],
    first_width: usize,
    continuation_width: usize,
    suffix: &str,
) -> Option<()> {
    let mut lines = WrapLineBuffer::new(first_width, continuation_width);
    for token in tokens {
        if token.width > lines.width
            && let Some(split) = split_long_link_token(token.text(), lines.width)
        {
            lines.push_split_token_lines(writer, split)?;
            lines.width = lines.continuation_width;
            continue;
        }
        if lines.current.can_fit(token, lines.width) {
            lines.current.push(token.clone());
        } else {
            lines.commit_current(writer)?;
            lines.current.push(token.clone());
        }
        lines.repair_current_line_if_markdown_block_start()?;
    }
    lines.finish(writer, suffix)
}

fn token_slice_width(tokens: &[InlineToken<'_>]) -> usize {
    tokens.iter().map(|token| token.width).sum::<usize>() + tokens.len().saturating_sub(1)
}

fn token_slice_markdown_block_start(tokens: &[InlineToken<'_>]) -> bool {
    let mut line = String::new();
    write_joined_tokens(&mut line, tokens);
    markdown_block_start_line(&line)
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

fn token_width(token: &str) -> usize {
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
        trailing_punctuation,
        image,
        label_start,
        label_close,
        destination_open,
        target,
        suffix,
    } = link;

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
                trailing_punctuation
            );
            if inline_target_close.chars().count() <= width {
                lines.push(inline_target_close);
                return Some(lines);
            }
            lines.push(format!("{last}]("));
            lines.extend(split_link_target_lines(target, width));
            lines.push(format!("){suffix}{trailing_punctuation}"));
            return Some(lines);
        }
    }

    let mut lines = Vec::new();
    lines.push(link[..destination_open + 1].to_owned());
    lines.extend(split_link_target_lines(target, width));
    lines.push(format!("){suffix}{trailing_punctuation}"));
    Some(lines)
}

struct SplitLinkToken<'a> {
    link: &'a str,
    trailing_punctuation: &'a str,
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

    let trailing_punctuation = &token[end..];
    if !trailing_link_punctuation(trailing_punctuation) {
        return None;
    }
    let link = &token[..end];
    let suffix = &link[destination_close + 1..];
    Some(SplitLinkToken {
        link,
        trailing_punctuation,
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

fn trailing_link_punctuation(value: &str) -> bool {
    value
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

fn is_pandoc_attribute_content(value: &str) -> bool {
    !value.is_empty() && (value.starts_with(['#', '.']) || value.contains('='))
}

fn format_prefixed_markdown_segments(
    segments: &[MarkdownHardBreakSegment],
    first_prefix: &str,
    continuation_prefix: &str,
    newline: &str,
    options: FormatOptions,
) -> Option<String> {
    let Some((first, rest)) = segments.split_first() else {
        return Some(format!("{first_prefix}{newline}"));
    };
    let mut out = String::new();
    out.push_str(&format_prefixed_markdown_segment(
        first,
        first_prefix,
        continuation_prefix,
        newline,
        options,
    )?);
    for segment in rest {
        out.push_str(&format_prefixed_markdown_segment(
            segment,
            continuation_prefix,
            continuation_prefix,
            newline,
            options,
        )?);
    }
    Some(out)
}

fn format_prefixed_markdown_segment(
    segment: &MarkdownHardBreakSegment,
    first_prefix: &str,
    continuation_prefix: &str,
    newline: &str,
    options: FormatOptions,
) -> Option<String> {
    let text = normalize_supported_links_and_images(&segment.text);
    let text = if options.markdown_canonical {
        Cow::Owned(canonicalize_inline(&text))
    } else {
        text
    };
    let tokens = inline_tokens(&text)?;
    let suffix = segment
        .hard_break
        .map(MarkdownHardBreakMarker::suffix)
        .unwrap_or("");
    let mut out = String::new();
    let mut writer =
        TokenLineWriter::terminated(&mut out, first_prefix, continuation_prefix, newline);
    match options.markdown_wrap {
        MarkdownWrap::None => writer.write_token_slice(&tokens, suffix)?,
        MarkdownWrap::Paragraph => writer.write_token_slice(&tokens, suffix)?,
        MarkdownWrap::Sentence => write_sentence_token_lines(&mut writer, &tokens, suffix)?,
        MarkdownWrap::Column => {
            let first_width = options
                .markdown_wrap_at_column
                .saturating_sub(first_prefix.chars().count())
                .max(1);
            let continuation_width = options
                .markdown_wrap_at_column
                .saturating_sub(continuation_prefix.chars().count())
                .max(1);
            let mut writer = writer.with_block_start_check();
            write_wrapped_tokens_with_first_width(
                &mut writer,
                &tokens,
                first_width,
                continuation_width,
                suffix,
            )?;
        }
    }
    Some(out)
}

fn inline_tokens(text: &str) -> Option<Vec<InlineToken<'_>>> {
    if simple_inline_tokens_supported(text) {
        return Some(simple_inline_tokens(text));
    }
    if contains_unsupported_inline_construct(text) {
        return None;
    }
    let mut tokens = Vec::new();
    let mut pending_prefix = String::new();
    let mut index = 0usize;
    while index < text.len() {
        while index < text.len() {
            let ch = text[index..].chars().next()?;
            if !ch.is_whitespace() {
                break;
            }
            index += ch.len_utf8();
        }
        if index >= text.len() {
            break;
        }

        let end = inline_token_end(text, index)?;
        attach_inline_token(&mut tokens, &mut pending_prefix, &text[index..end]);
        index = end;
    }
    if !pending_prefix.is_empty() {
        if let Some(previous) = tokens.last_mut() {
            previous.push_str(&pending_prefix);
        } else {
            tokens.push(InlineToken::owned(pending_prefix));
        }
    }

    Some(tokens)
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

fn simple_inline_tokens(text: &str) -> Vec<InlineToken<'_>> {
    let mut tokens = Vec::new();
    let mut pending_prefix = String::new();
    for token in text.split_whitespace() {
        attach_inline_token(&mut tokens, &mut pending_prefix, token);
    }
    if !pending_prefix.is_empty() {
        if let Some(previous) = tokens.last_mut() {
            previous.push_str(&pending_prefix);
        } else {
            tokens.push(InlineToken::owned(pending_prefix));
        }
    }
    tokens
}

fn attach_inline_token<'a>(
    tokens: &mut Vec<InlineToken<'a>>,
    pending_prefix: &mut String,
    token: &'a str,
) {
    if token.chars().all(opening_punctuation) {
        pending_prefix.push_str(token);
        return;
    }
    if token.chars().all(closing_punctuation) {
        if let Some(previous) = tokens.last_mut() {
            previous.push_str(token);
        } else {
            pending_prefix.push_str(token);
        }
        return;
    }
    if pending_prefix.is_empty() {
        tokens.push(InlineToken::borrowed(token));
        return;
    }

    let mut owned = String::with_capacity(pending_prefix.len() + token.len());
    owned.push_str(pending_prefix);
    owned.push_str(token);
    pending_prefix.clear();
    tokens.push(InlineToken::owned(owned));
}

fn opening_punctuation(ch: char) -> bool {
    matches!(ch, '(' | '[')
}

fn closing_punctuation(ch: char) -> bool {
    matches!(ch, ')' | ']' | ',' | '.' | ';' | ':' | '!' | '?')
}

fn contains_unsupported_inline_construct(text: &str) -> bool {
    let mut index = 0usize;
    while index < text.len() {
        if let Some(end) = unsupported_scan_protected_token_end(text, index) {
            index = end;
            continue;
        }
        if text[index..].starts_with("~~") && !escaped_at(text, index) {
            return strikethrough_span_end(text, index).is_none();
        }
        let ch = text[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        index += ch.len_utf8();
    }
    false
}

fn unsupported_scan_protected_token_end(text: &str, index: usize) -> Option<usize> {
    if escaped_at(text, index) {
        return None;
    }
    let rest = &text[index..];
    if rest.starts_with('`') {
        let tick_count = rest.bytes().take_while(|byte| *byte == b'`').count();
        let marker = &rest[..tick_count];
        let close = rest[tick_count..].find(marker)?;
        return Some(index + tick_count + close + tick_count);
    }
    if rest.starts_with('$') {
        let close = find_unescaped(rest, 1, '$')?;
        return Some(index + close + 1);
    }
    if let Some(end) = strikethrough_span_end(text, index) {
        return Some(end);
    }
    if let Some(end) = commonmark_autolink_span_end(text, index) {
        return Some(end);
    }
    if let Some(end) = paired_inline_html_span_end(text, index) {
        return Some(end);
    }
    if let Some(end) = inline_html_tag_span_end(text, index) {
        return Some(end);
    }
    if rest.starts_with('\\') && rest[1..].chars().next().is_some_and(char::is_alphabetic) {
        return latex_command_token_end(text, index);
    }
    if rest.starts_with("![") || rest.starts_with('[') {
        return link_or_bracket_token_end(text, index);
    }
    if (rest.starts_with("{{<") || rest.starts_with("{{%"))
        && let Some(close) = rest.find("}}")
    {
        return Some(index + close + 2);
    }
    if rest.starts_with('{')
        && let Some(close) = balanced_brace_end(rest)
    {
        return Some(index + close);
    }
    if rest.starts_with('<') && !inline_html_tag_at(text, index) {
        return rest.find('>').map(|close| index + close + 1);
    }
    None
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

fn normalize_supported_links_and_images(source: &str) -> Cow<'_, str> {
    if !source.as_bytes().contains(&b'[') {
        return Cow::Borrowed(source);
    }
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    while index < source.len() {
        let rest = &source[index..];
        if let Some(end) = balanced_brace_span_end(source, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if let Some(end) = reference_style_link_span_end(source, index) {
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if !escaped_at(source, index)
            && (rest.starts_with("![") || rest.starts_with('['))
            && let Some((end, normalized)) = normalize_link_or_image_at(source, index)
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

fn normalize_link_or_image_at(text: &str, start: usize) -> Option<(usize, String)> {
    let image = text[start..].starts_with("![");
    let label_start = if image { start + 2 } else { start + 1 };
    let label_close = find_balanced_square_close(text, label_start)?;
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
            && let Some((end, normalized)) = normalize_link_or_image_at(label, index)
        {
            out.push_str(&normalized);
            index = end;
            continue;
        }
        if let Some(end) = link_label_protected_span_end(label, index) {
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

fn link_label_protected_span_end(label: &str, index: usize) -> Option<usize> {
    strikethrough_span_end(label, index)
        .or_else(|| inline_code_span_end(label, index))
        .or_else(|| inline_math_span_end(label, index))
        .or_else(|| paired_inline_html_span_end(label, index))
        .or_else(|| inline_html_tag_span_end(label, index))
        .or_else(|| emphasis_span_end(label, index))
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

fn inline_token_end(text: &str, start: usize) -> Option<usize> {
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
        let end = inline_code_span_end(text, start)?;
        return Some(inline_code_span_end_with_attached_suffix(text, end));
    }
    if rest.starts_with('$') && !escaped_at(text, start) {
        return inline_math_span_end(text, start);
    }
    if rest.starts_with('\\') && rest[1..].chars().next().is_some_and(char::is_alphabetic) {
        return latex_command_token_end(text, start);
    }
    if let Some(end) = emphasis_span_end(text, start) {
        return Some(end);
    }
    if rest.starts_with("![") || rest.starts_with('[') {
        return link_or_bracket_token_end(text, start);
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

    let mut end = start;
    while end < text.len() {
        let slice = &text[end..];
        let ch = slice.chars().next()?;
        if ch.is_whitespace() {
            break;
        }
        if end != start
            && (slice.starts_with('`')
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
        if escaped_at(text, end) || !reference_bracket_token_at(text, end) {
            break;
        }
        let Some(reference_end) = link_or_bracket_token_end(text, end) else {
            break;
        };
        end = inline_span_end_with_trailing_punctuation(text, reference_end);
    }
    (end > start).then_some(end)
}

fn strikethrough_span_end(text: &str, start: usize) -> Option<usize> {
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

fn inline_code_span_end(text: &str, start: usize) -> Option<usize> {
    if !text[start..].starts_with('`') {
        return None;
    }
    let rest = &text[start..];
    let tick_count = rest.bytes().take_while(|byte| *byte == b'`').count();
    let marker = &rest[..tick_count];
    let close = rest[tick_count..].find(marker)?;
    Some(start + tick_count + close + tick_count)
}

fn inline_math_span_end(text: &str, start: usize) -> Option<usize> {
    if escaped_at(text, start) || !text[start..].starts_with('$') {
        return None;
    }
    let close = find_unescaped(&text[start..], 1, '$')?;
    Some(start + close + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmphasisSpan {
    close: usize,
    end: usize,
    run: usize,
}

fn emphasis_span_end(text: &str, start: usize) -> Option<usize> {
    emphasis_span_at(text, start).map(|span| span.end)
}

fn emphasis_span_at(text: &str, start: usize) -> Option<EmphasisSpan> {
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
    while search < text.len() {
        let relative = text[search..].find(delimiter)?;
        let close = search + relative;
        let inner = &text[start + run..close];
        if close <= start + run
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
            || inner.contains(['\n', '\r'])
            || inner.trim().is_empty()
            || text[..close]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace)
            || (marker == b'_' && underscore_is_intraword(text, close, run))
        {
            search = close + run;
            continue;
        }
        return Some(EmphasisSpan {
            close,
            end: inline_span_end_with_trailing_punctuation(text, close + run),
            run,
        });
    }
    None
}

fn delimiter_run_len_at(text: &str, index: usize, marker: u8) -> usize {
    text[index..]
        .bytes()
        .take_while(|byte| *byte == marker)
        .count()
}

fn inline_code_span_end_with_attached_suffix(text: &str, mut end: usize) -> usize {
    while end < text.len() {
        let ch = text[end..]
            .chars()
            .next()
            .expect("end is on a char boundary");
        if !ch.is_alphanumeric() {
            break;
        }
        end += ch.len_utf8();
    }
    end
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

fn latex_command_token_end(text: &str, start: usize) -> Option<usize> {
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

fn link_or_bracket_token_end(text: &str, start: usize) -> Option<usize> {
    let image = text[start..].starts_with("![");
    let label_start = if image { start + 2 } else { start + 1 };
    let label_close = find_balanced_square_close(text, label_start)?;
    let after_label = label_close + 1;
    if text.as_bytes().get(after_label) == Some(&b'(') {
        normalize_link_label(&text[label_start..label_close], !image)?;
        let destination_close = find_simple_destination_close(text, after_label + 1)?;
        parse_simple_link_target(&text[after_label + 1..destination_close])?;
        let mut end = destination_close + 1;
        if let Some((attribute_end, _)) = normalize_attribute_after(text, end) {
            end = attribute_end;
        }
        return Some(inline_span_end_with_trailing_punctuation(text, end));
    }
    if !image
        && let Some(end) = reference_style_link_end(text, label_start, label_close, after_label)
    {
        return Some(inline_span_end_with_trailing_punctuation(text, end));
    }
    if !image && shortcut_reference_link_token(&text[label_start..label_close]) {
        return Some(inline_span_end_with_trailing_punctuation(text, after_label));
    }
    if reference_bracket_token_at(text, start) {
        return Some(after_label);
    }
    None
}

fn shortcut_reference_link_token(label: &str) -> bool {
    !label.is_empty() && !label.contains(['\n', '\r'])
}

fn reference_style_link_end(
    text: &str,
    label_start: usize,
    label_close: usize,
    reference_open: usize,
) -> Option<usize> {
    if text.as_bytes().get(reference_open) != Some(&b'[')
        || text[label_start..label_close].contains(['\n', '\r'])
    {
        return None;
    }
    let reference_close = find_balanced_square_close(text, reference_open + 1)?;
    if text[reference_open + 1..reference_close].contains(['\n', '\r']) {
        return None;
    }
    Some(reference_close + 1)
}

fn reference_bracket_token_at(text: &str, start: usize) -> bool {
    if text[start..].starts_with("[^") {
        return true;
    }
    if !text[start..].starts_with('[') || text[start..].starts_with("![") {
        return false;
    }
    let Some(label_close) = find_balanced_square_close(text, start + 1) else {
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

fn find_balanced_square_close(text: &str, mut index: usize) -> Option<usize> {
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

fn find_simple_destination_close(text: &str, mut index: usize) -> Option<usize> {
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
struct LinkTarget<'a> {
    destination: &'a str,
    title: Option<&'a str>,
}

fn parse_simple_link_target(target: &str) -> Option<LinkTarget<'_>> {
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

fn render_link_target(target: LinkTarget<'_>) -> String {
    match target.title {
        Some(title) => format!("{} {title}", target.destination),
        None => target.destination.to_owned(),
    }
}

fn render_existing_split_link_target(raw: &str, target: LinkTarget<'_>) -> String {
    let leading = raw.len() - raw.trim_start().len();
    let trailing = raw.trim_end().len();
    format!(
        "{}{}{}",
        &raw[..leading],
        render_link_target(target),
        &raw[trailing..]
    )
}

fn split_link_target_lines(target: LinkTarget<'_>, width: usize) -> Vec<String> {
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

fn balanced_brace_end(text: &str) -> Option<usize> {
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

fn split_trailing_attribute(text: &str) -> Option<(&str, &str)> {
    if !text.ends_with('}') {
        return None;
    }
    let open = text.rfind('{')?;
    let before = &text[..open];
    let attr = &text[open..];
    balanced_brace_end(attr).filter(|end| *end == attr.len())?;
    Some((before, attr))
}

fn escaped_at(source: &str, index: usize) -> bool {
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

fn display_width(source: &str) -> usize {
    UnicodeWidthStr::width(source)
}

#[derive(Debug, Clone, Copy)]
struct MarkdownLine<'a> {
    full: &'a str,
    body: &'a str,
    newline: &'a str,
}

fn markdown_lines(source: &str) -> Vec<MarkdownLine<'_>> {
    let mut lines = Vec::new();
    let bytes = source.as_bytes();
    let mut start = 0usize;
    while start < source.len() {
        let mut end = start;
        while end < source.len() && !matches!(bytes[end], b'\r' | b'\n') {
            end += 1;
        }
        let (full_end, newline) = if end == source.len() {
            (end, "")
        } else if bytes[end] == b'\r' && end + 1 < source.len() && bytes[end + 1] == b'\n' {
            (end + 2, "\r\n")
        } else if bytes[end] == b'\r' {
            (end + 1, "\r")
        } else {
            (end + 1, "\n")
        };
        lines.push(MarkdownLine {
            full: &source[start..full_end],
            body: &source[start..end],
            newline,
        });
        start = full_end;
    }
    lines
}

fn markdown_line_bodies(source: &str) -> Vec<&str> {
    markdown_lines(source)
        .into_iter()
        .map(|line| line.body)
        .collect()
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
