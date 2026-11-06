//! Pandoc reads inline content before a setext underline, so a code span or
//! `$` math opened on a heading line and closed later in the paragraph spans
//! the underline, and the line is not a heading.
//!
//! Openers and closers follow Pandoc's default Markdown rules: a code span
//! closes at the next run of exactly as many backticks, and `$` math opens
//! before a non-space and closes after a non-space when no digit follows.
//! `\(` is plain text, since `tex_math_single_backslash` is off. Links, raw
//! HTML, and other inline syntax that could hide an opener are not modeled.
//!
//! Closers are indexed once per buffer, so checking every heading line in a
//! run of adjacent headings stays linear.

use crate::core::source::SourceBuffer;

#[derive(Debug, Clone)]
pub(crate) struct LiteralClosers {
    /// Maximal backtick runs as `(length, start)`, sorted.
    backtick_runs: Vec<(usize, usize)>,
    dollars: Vec<usize>,
    double_dollars: Vec<usize>,
    blank_lines: Vec<usize>,
}

impl LiteralClosers {
    fn new(source: &SourceBuffer) -> Self {
        let text = source.as_str();
        let bytes = text.as_bytes();
        let mut closers = Self {
            backtick_runs: Vec::new(),
            dollars: Vec::new(),
            double_dollars: Vec::new(),
            blank_lines: (0..source.lines.len())
                .filter(|line| source.line_text(*line).trim_ascii().is_empty())
                .collect(),
        };
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'`' => {
                    let run = bytes[index..].iter().take_while(|b| **b == b'`').count();
                    closers.backtick_runs.push((run, index));
                    index += run;
                    continue;
                }
                b'$' if bytes[..index]
                    .iter()
                    .rev()
                    .take_while(|b| **b == b'\\')
                    .count()
                    % 2
                    == 0 =>
                {
                    if bytes.get(index + 1) == Some(&b'$') {
                        closers.double_dollars.push(index);
                    }
                    let after_space = text[..index]
                        .chars()
                        .next_back()
                        .is_none_or(char::is_whitespace);
                    let before_digit = bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
                    if !after_space && !before_digit {
                        closers.dollars.push(index);
                    }
                }
                _ => {}
            }
            index += 1;
        }
        closers.backtick_runs.sort_unstable();
        closers
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

fn first_from(positions: &[usize], from: usize) -> Option<usize> {
    positions
        .get(positions.partition_point(|position| *position < from))
        .copied()
}

pub(super) fn inline_literal_crosses_line_end(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> bool {
    let closers = source
        .markdown_literal_closers
        .get_or_init(|| LiteralClosers::new(source));
    let line_start = source.lines[line].text.start();
    let line_end = source.lines[line].text.end();
    let blank = first_from(&closers.blank_lines, line + 2).unwrap_or(usize::MAX);
    let paragraph_end = match blank.min(end) {
        last if last < source.lines.len() => source.lines[last].full.start(),
        _ => source.as_str().len(),
    };
    let text = source.line_text(line);
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let from = line_start + index;
        // The closer's start and length. An opener that does not close is a
        // literal character, and Pandoc retries from the next one.
        let close = match bytes[index] {
            b'\\' => {
                index += 1 + text[index + 1..].chars().next().map_or(0, char::len_utf8);
                continue;
            }
            b'`' => {
                let run = bytes[index..].iter().take_while(|b| **b == b'`').count();
                closers
                    .backtick_run_from(run, from + run)
                    .map(|close| (close, run))
            }
            b'$' => {
                let display = (bytes.get(index + 1) == Some(&b'$'))
                    .then(|| first_from(&closers.double_dollars, from + 2))
                    .flatten()
                    .filter(|close| *close < paragraph_end)
                    .map(|close| (close, 2));
                let inline = || {
                    text[index + 1..]
                        .chars()
                        .next()
                        .is_some_and(|next| !next.is_whitespace())
                        .then(|| first_from(&closers.dollars, from + 2))
                        .flatten()
                        .map(|close| (close, 1))
                };
                display.or_else(inline)
            }
            _ => {
                index += text[index..].chars().next().unwrap().len_utf8();
                continue;
            }
        };
        match close {
            Some((close, len)) if close < line_end => index = close - line_start + len,
            Some((close, _)) if close < paragraph_end => return true,
            _ => index += 1,
        }
    }
    false
}
