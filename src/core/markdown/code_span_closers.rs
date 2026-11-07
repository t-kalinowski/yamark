//! Pandoc reads inline content before a setext underline, so a code span
//! opened on a heading line and closed later in the paragraph spans the
//! underline, and the line is not a heading.
//!
//! This is checked only for a line starting with a fence-like run of three or
//! more backticks, such as a fence opener Pandoc rejects for a non-ASCII space
//! in its info string. A code span crossing the underline from elsewhere in a
//! line, or `$` math doing the same, is a known boundary: deciding where Pandoc
//! opens a literal there needs its inline parser (link destinations,
//! autolinks, raw HTML), so such a line is still treated as a heading.
//!
//! A code span closes at the next run of exactly as many backticks; an
//! unmatched run is a literal backtick, and Pandoc retries one backtick later.
//! Closers are indexed once per buffer, so checking every heading line in a
//! run of adjacent headings stays linear.

use crate::core::source::SourceBuffer;

#[derive(Debug, Clone)]
pub(crate) struct CodeSpanClosers {
    /// Maximal backtick runs as `(length, start)`, sorted.
    backtick_runs: Vec<(usize, usize)>,
    blank_lines: Vec<usize>,
}

impl CodeSpanClosers {
    fn new(source: &SourceBuffer) -> Self {
        let bytes = source.as_str().as_bytes();
        let mut backtick_runs = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            let run = bytes[index..].iter().take_while(|b| **b == b'`').count();
            if run > 0 {
                backtick_runs.push((run, index));
            }
            index += run.max(1);
        }
        backtick_runs.sort_unstable();
        Self {
            backtick_runs,
            blank_lines: (0..source.lines.len())
                .filter(|line| source.line_text(*line).trim_ascii().is_empty())
                .collect(),
        }
    }

    fn backtick_run_from(&self, length: usize, from: usize) -> Option<usize> {
        let index = self
            .backtick_runs
            .partition_point(|run| *run < (length, from));
        self.backtick_runs
            .get(index)
            .filter(|run| run.0 == length)
            .map(|run| run.1)
    }
}

pub(super) fn code_span_crosses_line_end(source: &SourceBuffer, line: usize, end: usize) -> bool {
    let text = source.line_text(line);
    if !text.trim_start_matches(' ').starts_with("```") {
        return false;
    }
    let closers = source
        .markdown_code_span_closers
        .get_or_init(|| Box::new(CodeSpanClosers::new(source)));
    let line_start = source.lines[line].text.start();
    let line_end = source.lines[line].text.end();
    let blank = closers.blank_lines[closers
        .blank_lines
        .partition_point(|blank| *blank < line + 2)..]
        .first()
        .copied()
        .unwrap_or(usize::MAX);
    let paragraph_end = match blank.min(end) {
        last if last < source.lines.len() => source.lines[last].full.start(),
        _ => source.as_str().len(),
    };
    let bytes = text.as_bytes();
    let mut index = 0;
    // End of the backtick run being retried, so a long unmatched run is not
    // recounted for each backtick.
    let mut run_end = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                index += 1 + text[index + 1..].chars().next().map_or(0, char::len_utf8);
            }
            b'`' => {
                if run_end <= index {
                    run_end = index + bytes[index..].iter().take_while(|b| **b == b'`').count();
                }
                let run = run_end - index;
                match closers.backtick_run_from(run, line_start + index + run) {
                    Some(close) if close < line_end => index = close - line_start + run,
                    Some(close) if close < paragraph_end => return true,
                    _ => index += 1,
                }
            }
            _ => index += text[index..].chars().next().unwrap().len_utf8(),
        }
    }
    false
}
