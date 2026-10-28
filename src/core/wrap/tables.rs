//! Pipe, simple, multiline, and grid table layout. Source column widths remain
//! distinct from rendered widths because Pandoc uses both.

use super::{
    InlineScan, canonicalize_inline, display_width, final_newline, markdown_line_bodies,
    newline_for_join, normalize_spaces_preserving_protected_spans,
    protected_spacing_span_can_start, protected_spacing_span_end, strip_final_newline,
};
use crate::core::document::{FormatOptions, MarkdownTableWidths};
use std::borrow::Cow;

pub fn format_markdown_table(source: &str, options: FormatOptions) -> String {
    let newline = final_newline(source);
    let join_newline = newline_for_join(newline, options);
    let mut rows = markdown_line_bodies(source)
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
    if rows.iter().any(|row| row.len() != alignments.len()) {
        return source.to_owned();
    }
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
    if options.markdown_table_widths == MarkdownTableWidths::Preserve {
        // Pandoc uses delimiter lengths for relative widths and total source
        // line lengths to decide whether pipe cells wrap. Retain both, even
        // when rows were not originally aligned with one another.
        let mut out = String::new();
        let mut formatted_rows = rows.iter();
        for (index, line) in markdown_line_bodies(source).enumerate() {
            if index == 1 {
                out.push_str(line);
            } else {
                let row = formatted_rows.next().expect("table row");
                let mut cursor = 0;
                for ((range, cell), alignment) in
                    pipe_cell_ranges(line).iter().zip(row).zip(&alignments)
                {
                    out.push_str(&line[cursor..range.start]);
                    let width = display_width(&line[range.clone()]);
                    if display_width(cell) > width {
                        return source.to_owned();
                    }
                    let padding = width - display_width(cell);
                    let left = match alignment {
                        Alignment::None | Alignment::Left => padding.min(1),
                        Alignment::Right => padding.saturating_sub(1),
                        Alignment::Center => padding / 2,
                    };
                    out.push_str(&" ".repeat(left));
                    out.push_str(cell);
                    out.push_str(&" ".repeat(padding - left));
                    cursor = range.end;
                }
                out.push_str(&line[cursor..]);
            }
            out.push_str(join_newline);
        }
        if newline.is_empty() {
            trim_trailing_line_ending(&mut out);
        }
        return out;
    }
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
    let expanded = expand_pandoc_table_tabs(source);
    let table_source = expanded.as_ref();
    if let Some(formatted) = format_markdown_multiline_table(table_source, options) {
        return formatted;
    }
    if let Some(formatted) = format_markdown_grid_table(table_source, options) {
        return formatted;
    }

    let lines = markdown_line_bodies(table_source).collect::<Vec<_>>();
    if lines.len() < 2 {
        return source.to_owned();
    }
    let Some(columns) = pandoc_separator_columns(lines[1]) else {
        return source.to_owned();
    };
    format_pandoc_table_lines(table_source, &lines, columns, 0..1, options)
}

fn expand_pandoc_table_tabs(source: &str) -> Cow<'_, str> {
    if !source.contains('\t') {
        return Cow::Borrowed(source);
    }
    // Match Pandoc's tabFilter with its default four-character tab stops.
    // It counts characters here, then splits table cells by display width.
    let mut out = String::with_capacity(source.len());
    let mut column = 0usize;
    for ch in source.chars() {
        match ch {
            '\t' => {
                let padding = 4 - column % 4;
                out.push_str(&" ".repeat(padding));
                column += padding;
            }
            '\r' | '\n' => {
                out.push(ch);
                column = 0;
            }
            _ => {
                out.push(ch);
                column += 1;
            }
        }
    }
    Cow::Owned(out)
}

fn format_markdown_multiline_table(source: &str, options: FormatOptions) -> Option<String> {
    let lines = markdown_line_bodies(source).collect::<Vec<_>>();
    if lines.len() < 3 {
        return None;
    }
    let (separator, header) = if pandoc_separator_token_line(lines[0]) {
        let separators = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| pandoc_separator_token_line(line).then_some(index))
            .collect::<Vec<_>>();
        match separators.as_slice() {
            [0, _] => (0, 1..2), // Headerless: the first body line sets alignment.
            [0, separator, _] => (*separator, 1..*separator),
            _ => return None,
        }
    } else if pandoc_continuous_multiline_bound(lines[0])
        && pandoc_continuous_multiline_bound(lines[lines.len() - 1])
    {
        let separator = lines[1..lines.len() - 1]
            .iter()
            .position(|line| pandoc_separator_token_line(line))?
            + 1;
        (separator, 1..separator)
    } else {
        return None;
    };
    let columns = pandoc_separator_columns(lines[separator])?;
    Some(format_pandoc_table_lines(
        source, &lines, columns, header, options,
    ))
}

fn format_pandoc_table_lines(
    source: &str,
    lines: &[&str],
    mut columns: Vec<PandocColumn>,
    header: std::ops::Range<usize>,
    options: FormatOptions,
) -> String {
    // Pandoc derives alignment from the shortest nonempty header line in each
    // column. Delimiter lengths and gaps also encode multiline column widths.
    let header_cells = lines[header.clone()]
        .iter()
        .map(|line| pandoc_table_cells(line, &columns))
        .collect::<Vec<_>>();
    for (index, column) in columns.iter_mut().enumerate() {
        let shortest = header_cells
            .iter()
            .map(|row| row[index].trim_end())
            .filter(|cell| !cell.is_empty())
            .min_by_key(|cell| cell.chars().count());
        column.alignment = shortest.map_or(Alignment::None, |cell| {
            match (cell.starts_with(' '), display_width(cell) < column.width) {
                (true, true) => Alignment::Center,
                (true, false) => Alignment::Right,
                (false, true) => Alignment::Left,
                (false, false) => Alignment::None,
            }
        });
    }

    let rows = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if line.trim().is_empty()
                || pandoc_separator_token_line(line)
                || pandoc_continuous_multiline_bound(line)
            {
                return Vec::new();
            }
            let cells = pandoc_table_cells(line, &columns);
            let mut row = cells
                .iter()
                .map(|cell| normalize_spaces_preserving_protected_spans(cell.trim()).into_owned())
                .collect::<Vec<_>>();
            canonicalize_table_row(&mut row, options);
            if options.markdown_table_widths == MarkdownTableWidths::Preserve
                && header.contains(&index)
            {
                for ((cell, original), column) in row.iter_mut().zip(&cells).zip(&columns) {
                    let width = display_width(cell);
                    let fits_alignment = match column.alignment {
                        Alignment::None => width >= column.width,
                        Alignment::Left | Alignment::Right => width < column.width,
                        Alignment::Center => width + 1 < column.width,
                    };
                    if !fits_alignment {
                        // Internal padding can make a header flush with its
                        // delimiter. Keep that padding when it sets alignment.
                        *cell = if options.markdown_canonical {
                            canonicalize_inline(original.trim())
                        } else {
                            original.trim().to_owned()
                        };
                    }
                }
            }
            row
        })
        .collect::<Vec<_>>();
    if options.markdown_table_widths == MarkdownTableWidths::Fit {
        fit_pandoc_columns(&mut columns, &rows, header);
    }

    let newline = final_newline(source);
    let join_newline = newline_for_join(newline, options);
    let mut out = String::new();
    for (line, row) in lines.iter().zip(&rows) {
        if !row.is_empty() {
            emit_pandoc_table_row(&mut out, row, &columns);
        } else if options.markdown_table_widths == MarkdownTableWidths::Preserve
            || line.trim().is_empty()
        {
            out.push_str(line);
        } else {
            let marker = if line.contains('=') { '=' } else { '-' };
            if pandoc_continuous_multiline_bound(line) {
                let last = columns.last().expect("table has columns");
                out.push_str(&marker.to_string().repeat(last.start + last.width));
            } else {
                let separators = columns
                    .iter()
                    .map(|column| marker.to_string().repeat(column.width))
                    .collect::<Vec<_>>();
                emit_pandoc_table_row(&mut out, &separators, &columns);
            }
        }
        out.push_str(join_newline);
    }
    if newline.is_empty() {
        trim_trailing_line_ending(&mut out);
    }
    out
}

fn fit_pandoc_columns(
    columns: &mut [PandocColumn],
    rows: &[Vec<String>],
    header: std::ops::Range<usize>,
) {
    let mut start = 0;
    for (index, column) in columns.iter_mut().enumerate() {
        let header_width = rows[header.clone()]
            .iter()
            .filter_map(|row| row.get(index))
            .filter(|cell| !cell.is_empty())
            .min_by_key(|cell| cell.chars().count())
            .map_or(0, |cell| display_width(cell));
        let content_width = rows
            .iter()
            .filter_map(|row| row.get(index))
            .map(|cell| pandoc_source_width(cell).max(display_width(cell)))
            .max()
            .unwrap_or(0);
        column.start = start;
        column.width = match column.alignment {
            // Default alignment requires a header flush with its underline.
            // Wider body text can use the gap before the next column.
            Alignment::None => header_width.max(3),
            Alignment::Left | Alignment::Right => content_width.max(header_width + 1).max(3),
            Alignment::Center => content_width.max(header_width + 2).max(3),
        };
        start += content_width.max(column.width) + 2;
    }
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
    let lines = markdown_line_bodies(source).collect::<Vec<_>>();
    if lines.len() < 3 {
        return None;
    }
    let mut widths = grid_border_widths(lines[0])?;
    let top_alignments = grid_border_alignments(lines[0]);
    let column_count = widths.len();
    let mut sections = Vec::<GridSection>::new();
    let mut index = 1usize;
    while index < lines.len() {
        let mut rows = Vec::new();
        loop {
            if index >= lines.len() {
                return None;
            }
            if let Some(border) = grid_border_kind(lines[index], &widths) {
                if rows.is_empty() {
                    return None;
                }
                sections.push(GridSection {
                    rows,
                    border_after: border,
                    border_alignments: grid_border_alignments(lines[index]),
                });
                index += 1;
                break;
            }
            let mut row = grid_row_cells(lines[index], column_count)?;
            for (cell, width) in row.iter().zip(&widths) {
                if display_width(cell) > *width {
                    // Repairing a row wider than its border would change the
                    // layout. Preserve this malformed table instead.
                    return None;
                }
            }
            canonicalize_table_row(&mut row, options);
            rows.push(row);
            index += 1;
        }
    }
    if sections.is_empty() {
        return None;
    }
    if options.markdown_table_widths == MarkdownTableWidths::Fit {
        widths.fill(3);
        for row in sections.iter().flat_map(|section| &section.rows) {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(display_width(cell) + 2);
            }
        }
    }

    let mut out = String::new();
    emit_grid_border(
        &mut out,
        &widths,
        GridBorderKind::Normal,
        &top_alignments,
        join_newline,
    );
    for section in &sections {
        for row in &section.rows {
            emit_grid_row(&mut out, row, &widths, join_newline);
        }
        emit_grid_border(
            &mut out,
            &widths,
            section.border_after,
            &section.border_alignments,
            join_newline,
        );
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
    border_alignments: Vec<Alignment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridBorderKind {
    Normal,
    Strong,
}

fn grid_border_widths(line: &str) -> Option<Vec<usize>> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('+')?.strip_suffix('+')?;
    let mut widths = Vec::new();
    for part in inner.split('+') {
        let marker = part.trim_matches(':');
        if marker.is_empty() || !marker.chars().all(|ch| matches!(ch, '-' | '=')) {
            return None;
        }
        widths.push(part.len());
    }
    Some(widths)
}

fn grid_border_alignments(line: &str) -> Vec<Alignment> {
    line.trim()
        .trim_matches('+')
        .split('+')
        .map(table_alignment)
        .collect()
}

fn grid_border_kind(line: &str, widths: &[usize]) -> Option<GridBorderKind> {
    (grid_border_widths(line)? == widths).then_some(())?;
    if line.contains('=') {
        Some(GridBorderKind::Strong)
    } else {
        Some(GridBorderKind::Normal)
    }
}

fn grid_row_cells(line: &str, column_count: usize) -> Option<Vec<String>> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|')?.strip_suffix('|')?;
    let cells = inner
        .split('|')
        .map(|cell| normalize_spaces_preserving_protected_spans(cell.trim()).into_owned())
        .collect::<Vec<_>>();
    (cells.len() == column_count).then_some(cells)
}

fn emit_grid_border(
    out: &mut String,
    widths: &[usize],
    kind: GridBorderKind,
    alignments: &[Alignment],
    newline: &str,
) {
    let marker = match kind {
        GridBorderKind::Normal => '-',
        GridBorderKind::Strong => '=',
    };
    out.push('+');
    for (width, alignment) in widths.iter().zip(alignments) {
        let left = matches!(alignment, Alignment::Left | Alignment::Center);
        let right = matches!(alignment, Alignment::Right | Alignment::Center);
        if left {
            out.push(':');
        }
        for _ in 0..width - usize::from(left) - usize::from(right) {
            out.push(marker);
        }
        if right {
            out.push(':');
        }
        out.push('+');
    }
    out.push_str(newline);
}

fn emit_grid_row(out: &mut String, row: &[String], widths: &[usize], newline: &str) {
    out.push('|');
    for (cell, width) in row.iter().zip(widths) {
        let padding = width - display_width(cell);
        // Grid colons carry alignment. Indenting cell text to match that
        // alignment can turn prose into an indented code block in Pandoc.
        let left = padding.min(1);
        out.push_str(&" ".repeat(left));
        out.push_str(cell);
        out.push_str(&" ".repeat(padding - left));
        out.push('|');
    }
    out.push_str(newline);
}

fn split_pipe_row(line: &str) -> Vec<String> {
    pipe_cell_ranges(line)
        .into_iter()
        .map(|range| normalize_spaces_preserving_protected_spans(line[range].trim()).into_owned())
        .collect()
}

fn pipe_cell_ranges(line: &str) -> Vec<std::ops::Range<usize>> {
    let (body, _) = strip_final_newline(line);
    let mut trimmed = body.trim();
    if let Some(rest) = trimmed.strip_prefix('|') {
        trimmed = rest;
    }
    if let Some(rest) = trimmed.strip_suffix('|') {
        trimmed = rest;
    }

    let mut scan = InlineScan::new(trimmed);
    let mut cells = Vec::new();
    let offset = trimmed.as_ptr() as usize - line.as_ptr() as usize;
    let mut start = 0;
    let mut escaped = false;
    let mut index = 0usize;
    while index < trimmed.len() {
        if !escaped
            && protected_spacing_span_can_start(trimmed, index)
            && let Some(end) = protected_spacing_span_end(&mut scan, index)
        {
            index = end;
            continue;
        }
        let ch = trimmed[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        if ch == '|' && !escaped {
            cells.push(offset + start..offset + index);
            index += ch.len_utf8();
            start = index;
            continue;
        }
        escaped = ch == '\\' && !escaped;
        if ch != '\\' {
            escaped = false;
        }
        index += ch.len_utf8();
    }
    cells.push(offset + start..offset + trimmed.len());
    cells
}

fn canonicalize_table_rows(rows: &mut [Vec<String>], options: FormatOptions) {
    if !options.markdown_canonical {
        return;
    }
    for row in rows {
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
    width: usize,
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
            width: token.len(),
            alignment: Alignment::None,
        });
    }
    (columns.len() >= 2).then_some(columns)
}

fn pandoc_separator_token(token: &str) -> bool {
    token.len() >= 3
        && token.chars().all(|ch| matches!(ch, '-' | '=' | ':'))
        && token.chars().any(|ch| matches!(ch, '-' | '='))
}

fn pandoc_table_cells<'a>(line: &'a str, columns: &[PandocColumn]) -> Vec<&'a str> {
    let offsets = columns
        .iter()
        .map(|column| byte_offset_at_display_width(line, column.start))
        .chain(std::iter::once(line.len()))
        .collect::<Vec<_>>();
    offsets
        .windows(2)
        .map(|pair| &line[pair[0]..pair[1]])
        .collect()
}

fn byte_offset_at_display_width(line: &str, width: usize) -> usize {
    let mut current = 0;
    for (offset, ch) in line.char_indices() {
        if current >= width {
            // Match Pandoc's splitAtWidth, including a combining mark at the
            // boundary belonging to the next cell. Consuming it here would
            // move that cell's alignment padding after the mark.
            return offset;
        }
        current += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
    }
    line.len()
}

fn emit_pandoc_table_row(out: &mut String, row: &[String], columns: &[PandocColumn]) {
    let mut position = 0;
    for (cell, column) in row.iter().zip(columns) {
        let width = display_width(cell);
        let padding = column.width.saturating_sub(width);
        let left = match column.alignment {
            Alignment::None | Alignment::Left => 0,
            Alignment::Right => padding,
            Alignment::Center => padding / 2,
        };
        let start = column.start + left;
        out.push_str(&" ".repeat(start.saturating_sub(position)));
        out.push_str(cell);
        // Pandoc splits source columns by character widths, even when a
        // sequence such as a joined emoji renders as a single glyph.
        position = start.max(position) + pandoc_source_width(cell);
    }
}

fn pandoc_source_width(cell: &str) -> usize {
    cell.chars()
        .filter_map(unicode_width::UnicodeWidthChar::width)
        .sum()
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
