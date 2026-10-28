//! Unified diff rendering with bounded quadratic work and a linear fallback.

use std::collections::HashMap;
use std::path::Path;

pub(super) fn simple_diff(path: &Path, before: &str, after: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("--- {}\n+++ {}\n", path.display(), path.display()));
    let before_lines = diff_lines(before);
    let after_lines = diff_lines(after);
    let ops = diff_ops(&before_lines, &after_lines);
    let mut previous_start = 0;
    let (mut old_start, mut new_start) = (1, 1);
    for (start, end) in diff_hunks(&ops, 3) {
        let (old_skipped, new_skipped) = diff_line_counts(&ops[previous_start..start]);
        old_start += old_skipped;
        new_start += new_skipped;
        previous_start = start;
        let (old_count, new_count) = diff_line_counts(&ops[start..end]);
        out.push_str(&format!(
            "@@ -{} +{} @@\n",
            unified_range(old_start, old_count),
            unified_range(new_start, new_count)
        ));
        for op in &ops[start..end] {
            match op {
                DiffOp::Equal(line) => push_diff_line(&mut out, ' ', *line),
                DiffOp::Remove(line) => push_diff_line(&mut out, '-', *line),
                DiffOp::Add(line) => push_diff_line(&mut out, '+', *line),
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DiffLine<'a> {
    text: &'a str,
    has_line_ending: bool,
}

#[derive(Debug, Clone, Copy)]
enum DiffOp<'a> {
    Equal(DiffLine<'a>),
    Remove(DiffLine<'a>),
    Add(DiffLine<'a>),
}

fn diff_lines(input: &str) -> Vec<DiffLine<'_>> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(DiffLine {
                    text: &input[start..i],
                    has_line_ending: true,
                });
                i += 1;
                start = i;
            }
            b'\r' => {
                lines.push(DiffLine {
                    text: &input[start..i],
                    has_line_ending: true,
                });
                i += if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    2
                } else {
                    1
                };
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < input.len() {
        lines.push(DiffLine {
            text: &input[start..],
            has_line_ending: false,
        });
    }
    lines
}

fn push_diff_line(out: &mut String, prefix: char, line: DiffLine<'_>) {
    out.push(prefix);
    out.push_str(line.text);
    out.push('\n');
    if !line.has_line_ending {
        out.push_str("\\ No newline at end of file\n");
    }
}

fn diff_ops<'a>(before: &[DiffLine<'a>], after: &[DiffLine<'a>]) -> Vec<DiffOp<'a>> {
    let prefix = before
        .iter()
        .zip(after)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = before[prefix..]
        .iter()
        .rev()
        .zip(after[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let before_end = before.len() - suffix;
    let after_end = after.len() - suffix;
    let mut ops = Vec::with_capacity(before.len() + after.len());
    ops.extend(before[..prefix].iter().copied().map(DiffOp::Equal));
    ops.extend(middle_diff_ops(
        &before[prefix..before_end],
        &after[prefix..after_end],
    ));
    ops.extend(before[before_end..].iter().copied().map(DiffOp::Equal));
    ops
}

fn middle_diff_ops<'a>(before: &[DiffLine<'a>], after: &[DiffLine<'a>]) -> Vec<DiffOp<'a>> {
    const MAX_LCS_CELLS: usize = 4_000_000;
    if before
        .len()
        .checked_mul(after.len())
        .is_none_or(|cells| cells > MAX_LCS_CELLS)
    {
        return linear_diff_ops(before, after);
    }

    let mut lcs = vec![vec![0usize; after.len() + 1]; before.len() + 1];
    for i in (0..before.len()).rev() {
        for j in (0..after.len()).rev() {
            lcs[i][j] = if before[i] == after[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;
    while i < before.len() || j < after.len() {
        if i < before.len() && j < after.len() && before[i] == after[j] {
            ops.push(DiffOp::Equal(before[i]));
            i += 1;
            j += 1;
        } else if i < before.len() && (j == after.len() || lcs[i + 1][j] >= lcs[i][j + 1]) {
            ops.push(DiffOp::Remove(before[i]));
            i += 1;
        } else {
            ops.push(DiffOp::Add(after[j]));
            j += 1;
        }
    }
    ops
}

fn linear_diff_ops<'a>(before: &[DiffLine<'a>], after: &[DiffLine<'a>]) -> Vec<DiffOp<'a>> {
    let before_end = before.len();
    let after_end = after.len();
    let mut ops = Vec::new();
    let mut i = 0;
    let mut j = 0;
    let mut sync_index = None;
    while i < before_end || j < after_end {
        if i < before_end && j < after_end && before[i] == after[j] {
            ops.push(DiffOp::Equal(before[i]));
            i += 1;
            j += 1;
            continue;
        }

        if i == before_end {
            for line in &after[j..after_end] {
                ops.push(DiffOp::Add(*line));
            }
            break;
        }
        if j == after_end {
            for line in &before[i..before_end] {
                ops.push(DiffOp::Remove(*line));
            }
            break;
        }

        if let Some((remove_count, add_count)) =
            next_linear_diff_sync(before, after, i, j, before_end, after_end, &mut sync_index)
        {
            for line in &before[i..i + remove_count] {
                ops.push(DiffOp::Remove(*line));
            }
            for line in &after[j..j + add_count] {
                ops.push(DiffOp::Add(*line));
            }
            i += remove_count;
            j += add_count;
        } else {
            for line in &before[i..before_end] {
                ops.push(DiffOp::Remove(*line));
            }
            for line in &after[j..after_end] {
                ops.push(DiffOp::Add(*line));
            }
            break;
        }
    }

    ops
}

fn next_linear_diff_sync<'a>(
    before: &[DiffLine<'a>],
    after: &[DiffLine<'a>],
    before_start: usize,
    after_start: usize,
    before_end: usize,
    after_end: usize,
    sync_index: &mut Option<LinearDiffSyncIndex<'a>>,
) -> Option<(usize, usize)> {
    const SYNC_WINDOW: usize = 64;
    let before = &before[before_start..before_end];
    let after = &after[after_start..after_end];
    if let Some(sync) = next_bounded_linear_diff_sync(before, after, SYNC_WINDOW) {
        return Some(sync);
    }

    let sync_index = sync_index.get_or_insert_with(|| LinearDiffSyncIndex::new(after, after_start));
    sync_index.next_sync(before, after_start)
}

struct LinearDiffSyncIndex<'a> {
    after_positions: HashMap<DiffLine<'a>, Vec<usize>>,
}

impl<'a> LinearDiffSyncIndex<'a> {
    fn new(after: &[DiffLine<'a>], after_start: usize) -> Self {
        let mut after_positions: HashMap<DiffLine<'a>, Vec<usize>> =
            HashMap::with_capacity(after.len());
        for (offset, line) in after.iter().copied().enumerate() {
            after_positions
                .entry(line)
                .or_default()
                .push(after_start + offset);
        }
        Self { after_positions }
    }

    fn next_sync(&self, before: &[DiffLine<'a>], after_start: usize) -> Option<(usize, usize)> {
        let mut best = None;
        for (remove_count, line) in before.iter().copied().enumerate() {
            if let Some((best_remove, best_add)) = best
                && remove_count > best_remove + best_add
            {
                break;
            }

            let Some(positions) = self.after_positions.get(&line) else {
                continue;
            };
            let index = positions.partition_point(|&position| position < after_start);
            let Some(&position) = positions.get(index) else {
                continue;
            };

            let add_count = position - after_start;
            let distance = remove_count + add_count;
            if distance == 0 {
                continue;
            }
            if best.is_none_or(|(best_remove, best_add)| {
                let best_distance = best_remove + best_add;
                distance < best_distance || distance == best_distance && remove_count < best_remove
            }) {
                best = Some((remove_count, add_count));
            }
        }
        best
    }
}

fn next_bounded_linear_diff_sync(
    before: &[DiffLine<'_>],
    after: &[DiffLine<'_>],
    window: usize,
) -> Option<(usize, usize)> {
    let max_before = before.len().saturating_sub(1).min(window);
    let max_after = after.len().saturating_sub(1).min(window);

    for distance in 1..=max_before + max_after {
        let remove_min = distance.saturating_sub(max_after);
        let remove_max = max_before.min(distance);
        for (remove_count, before_line) in before
            .iter()
            .enumerate()
            .take(remove_max + 1)
            .skip(remove_min)
        {
            let add_count = distance - remove_count;
            if add_count <= max_after && *before_line == after[add_count] {
                return Some((remove_count, add_count));
            }
        }
    }
    None
}

fn diff_hunks(ops: &[DiffOp<'_>], context: usize) -> Vec<(usize, usize)> {
    let mut hunks: Vec<(usize, usize)> = Vec::new();
    let mut index = 0usize;
    while let Some(change) = next_change(ops, index) {
        let mut start = change;
        let mut context_before = 0usize;
        while start > 0 && matches!(ops[start - 1], DiffOp::Equal(_)) && context_before < context {
            start -= 1;
            context_before += 1;
        }

        let mut end = change + 1;
        let mut trailing_context = 0usize;
        while end < ops.len() {
            if matches!(ops[end], DiffOp::Equal(_)) {
                if trailing_context == context {
                    break;
                }
                trailing_context += 1;
            } else {
                trailing_context = 0;
            }
            end += 1;
        }
        // Nearby changes share context. Emit that context once so the next
        // hunk never refers back to lines consumed by the previous hunk.
        if let Some(previous) = hunks.last_mut()
            && start <= previous.1
        {
            previous.1 = end;
        } else {
            hunks.push((start, end));
        }
        index = end;
    }
    hunks
}

fn next_change(ops: &[DiffOp<'_>], start: usize) -> Option<usize> {
    ops.iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, op)| (!matches!(op, DiffOp::Equal(_))).then_some(index))
}

fn diff_line_counts(ops: &[DiffOp<'_>]) -> (usize, usize) {
    let mut old_count = 0usize;
    let mut new_count = 0usize;
    for op in ops {
        match op {
            DiffOp::Equal(_) => {
                old_count += 1;
                new_count += 1;
            }
            DiffOp::Remove(_) => old_count += 1,
            DiffOp::Add(_) => new_count += 1,
        }
    }
    (old_count, new_count)
}

fn unified_range(start: usize, count: usize) -> String {
    match count {
        0 => format!("{start},0"),
        1 => start.to_string(),
        count => format!("{start},{count}"),
    }
}
