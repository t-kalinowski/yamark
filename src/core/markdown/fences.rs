//! Fenced code recognition, language selection, and header normalization.

use crate::core::document::{CodeFenceSafety, FormatOptions, MarkdownWrap};
use crate::core::source::{SourceBuffer, Span};

#[derive(Debug, Clone, Copy)]
pub(super) struct Fence {
    pub(super) marker: u8,
    pub(super) len: usize,
}

impl Fence {
    pub(super) fn safety(self) -> CodeFenceSafety {
        CodeFenceSafety {
            marker: self.marker as char,
            min_len: self.len,
        }
    }
}

pub(super) fn code_fence_at(text: &str) -> Option<Fence> {
    let bytes = text.as_bytes();
    let indent = bytes.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 || indent >= bytes.len() || !matches!(bytes[indent], b'`' | b'~') {
        return None;
    }
    let marker = bytes[indent];
    let mut len = 0usize;
    while indent + len < bytes.len() && bytes[indent + len] == marker {
        len += 1;
    }
    (len >= 3).then_some(Fence { marker, len })
}

pub(super) fn find_code_fence_closing(
    source: &SourceBuffer,
    i: usize,
    end: usize,
    fence: Fence,
) -> Option<usize> {
    find_code_fence_closing_at_indent(source, i, end, fence, 0)
}

pub(super) fn find_code_fence_closing_at_indent(
    source: &SourceBuffer,
    mut i: usize,
    end: usize,
    fence: Fence,
    indent: usize,
) -> Option<usize> {
    while i < end {
        let text = source.line_text(i);
        if text.bytes().take_while(|byte| *byte == b' ').count() >= indent
            && let Some(candidate) = code_fence_at(&text[indent..])
        {
            let rest = &text.trim_start()[candidate.len..];
            if candidate.marker == fence.marker
                && candidate.len >= fence.len
                && rest.trim().is_empty()
            {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

pub(super) fn code_fence_language(text: &str) -> Option<String> {
    code_fence_info(text).and_then(code_fence_language_from_info)
}

pub(super) fn code_fence_info(text: &str) -> Option<&str> {
    let fence = code_fence_at(text)?;
    Some(text.trim_start()[fence.len..].trim())
}

fn code_fence_language_from_info(info: &str) -> Option<String> {
    let info = info.trim();
    if info.is_empty() || braced_code_fence_raw_format(info) {
        return None;
    }
    if let Some(language) = code_cell_language_from_info(info) {
        return Some(language);
    }
    if let Some(inner) = braced_code_fence_inner(info) {
        return pandoc_attribute_language(inner);
    }
    let first = info.split_whitespace().next()?;
    code_fence_language_token(first)
}

fn code_cell_language_from_info(info: &str) -> Option<String> {
    let rest = info.strip_prefix("{code-cell}")?.trim_start();
    code_fence_language_token(rest.split_whitespace().next()?)
}

fn pandoc_attribute_language(inner: &str) -> Option<String> {
    let mut bare_language = None::<String>;
    for token in inner.split_whitespace() {
        if token.starts_with('#') || token.contains('=') {
            continue;
        }
        if let Some(language) = token.strip_prefix('.').and_then(code_fence_language_token) {
            return Some(language);
        }
        if bare_language.is_none() {
            bare_language = code_fence_language_token(token);
        }
    }
    bare_language
}

fn code_fence_language_token(token: &str) -> Option<String> {
    let token = token
        .trim_start_matches('.')
        .split([',', '}'])
        .next()
        .unwrap_or(token);
    if token.is_empty() || token.starts_with('#') || token.contains('=') {
        return None;
    }
    let language = match token.to_ascii_lowercase().as_str() {
        "ipython3" => "python".to_owned(),
        other => other.to_owned(),
    };
    Some(language)
}

pub(super) fn code_fence_raw_format(text: &str) -> bool {
    code_fence_info(text).is_some_and(braced_code_fence_raw_format)
}

fn braced_code_fence_raw_format(info: &str) -> bool {
    let Some(inner) = braced_code_fence_inner(info) else {
        return false;
    };
    let Some(format) = inner.trim().strip_prefix('=') else {
        return false;
    };
    !format.is_empty()
        && format
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

pub(super) fn code_fence_is_code_cell(text: &str) -> bool {
    code_fence_info(text).is_some_and(|info| info.trim().starts_with("{code-cell}"))
}

pub(super) fn code_fence_code_cell_without_language(text: &str) -> bool {
    code_fence_info(text).is_some_and(|info| {
        let info = info.trim();
        info == "{code-cell}"
            || info
                .strip_prefix("{code-cell}")
                .is_some_and(|rest| rest.trim().is_empty())
    })
}

pub(super) fn code_fence_language_supported_opaque(language: &str) -> bool {
    matches!(
        language,
        "bash"
            | "sh"
            | "shell"
            | "zsh"
            | "ojs"
            | "text"
            | "console"
            | "rust"
            | "toml"
            | "lua"
            | "mermaid"
            | "ini"
            | "julia"
            | "sql"
            | "java"
            | "c"
            | "tex"
            | "latex"
            | "output"
            | "powershell"
            | "cmd"
    )
}

pub(super) fn code_cell_formatter_body(
    source: &SourceBuffer,
    mut line: usize,
    closing: usize,
) -> Option<Span> {
    while line < closing && source.line_text(line).trim_start().starts_with(':') {
        line += 1;
    }
    Some(Span::new(
        source.lines.get(line)?.full.start(),
        source.lines[closing].full.start(),
    ))
}

pub(super) fn normalized_code_fence_opening(
    line: &str,
    normalize_generic_info: bool,
    options: FormatOptions,
) -> Option<String> {
    let (body, newline) = split_line_ending(line);
    let fence = code_fence_at(body)?;
    let indent = body.bytes().take_while(|byte| *byte == b' ').count();
    let marker_end = indent + fence.len;
    let after_marker = &body[marker_end..];
    let info = after_marker.trim();
    if info.is_empty() {
        return None;
    }
    let promote_quarto_options = !matches!(options.markdown_wrap, MarkdownWrap::None)
        && body.chars().count() > options.markdown_wrap.column_width().unwrap_or(72);
    let normalized_info = normalize_code_fence_info(
        info,
        newline,
        normalize_generic_info,
        promote_quarto_options,
    )
    .or_else(|| {
        (braced_code_fence_info_supported(info) && after_marker.starts_with(char::is_whitespace))
            .then(|| info.to_owned())
    })?;
    let separator = if normalized_info.starts_with('{') {
        ""
    } else if after_marker.starts_with(char::is_whitespace) {
        " "
    } else {
        ""
    };
    let normalized = format!(
        "{}{}{}{}{}",
        &body[..indent],
        &body[indent..marker_end],
        separator,
        normalized_info,
        newline
    );
    (normalized != line).then_some(normalized)
}

fn braced_code_fence_info_starts_with_language(info: &str) -> bool {
    let Some(inner) = braced_code_fence_inner(info) else {
        return false;
    };
    let Some(first) = inner.split_whitespace().next() else {
        return false;
    };
    let first = first.trim_start_matches('.');
    !first.is_empty() && !first.starts_with('#') && !first.contains('=')
}

fn braced_code_fence_info_supported(info: &str) -> bool {
    braced_code_fence_raw_format(info)
        || braced_code_fence_info_starts_with_language(info)
        || code_fence_language_from_info(info).is_some()
}

fn braced_code_fence_inner(info: &str) -> Option<&str> {
    info.strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
}

fn normalize_code_fence_info(
    info: &str,
    newline: &str,
    normalize_generic_info: bool,
    promote_quarto_options: bool,
) -> Option<String> {
    let Some(inner) = braced_code_fence_inner(info) else {
        if let Some(normalized) = normalize_bare_language_attribute_info(info) {
            return Some(normalized);
        }
        if !normalize_generic_info {
            return None;
        }
        let normalized = info.split_whitespace().collect::<Vec<_>>().join(" ");
        return (normalized != info).then_some(normalized);
    };
    if promote_quarto_options && let Some(promoted) = promote_quarto_chunk_header(inner, newline) {
        return Some(promoted);
    }
    if inner.contains('"') || inner.contains('\'') {
        return None;
    }
    let mut seen_classes = Vec::<String>::new();
    let mut tokens = Vec::<&str>::new();
    for token in inner.split_whitespace() {
        if let Some(class) = token.strip_prefix('.') {
            let class_key = class.to_ascii_lowercase();
            if seen_classes.iter().any(|seen| seen == &class_key) {
                continue;
            }
            seen_classes.push(class_key);
        } else if tokens.is_empty() && !token.starts_with('#') && !token.contains('=') {
            seen_classes.push(token.trim_start_matches('.').to_ascii_lowercase());
        } else if !token.starts_with('#')
            && !token.contains('=')
            && seen_classes
                .iter()
                .any(|seen| seen == &token.trim_start_matches('.').to_ascii_lowercase())
        {
            continue;
        }
        tokens.push(token);
    }
    if tokens.is_empty() {
        return None;
    }
    let normalized = format!("{{{}}}", tokens.join(" "));
    (normalized != info).then_some(normalized)
}

fn normalize_bare_language_attribute_info(info: &str) -> Option<String> {
    let (language, rest) = info.split_once(char::is_whitespace)?;
    if language.is_empty()
        || language.starts_with('{')
        || language.starts_with('#')
        || language.contains('=')
    {
        return None;
    }
    let rest = rest.trim();
    let inner = rest
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))?;
    if inner.contains('"') || inner.contains('\'') {
        return None;
    }

    let language_key = language.trim_start_matches('.').to_ascii_lowercase();
    let mut changed = false;
    let mut tokens = Vec::new();
    for token in inner.split_whitespace() {
        if let Some(class) = token.strip_prefix('.')
            && class.eq_ignore_ascii_case(&language_key)
        {
            changed = true;
            continue;
        }
        tokens.push(token);
    }
    if !changed {
        return None;
    }
    if tokens.is_empty() {
        Some(language.to_owned())
    } else {
        Some(format!("{} {{{}}}", language, tokens.join(" ")))
    }
}

fn promote_quarto_chunk_header(inner: &str, newline: &str) -> Option<String> {
    if newline.is_empty() {
        return None;
    }
    if !inner.contains(',') || inner.contains(['"', '\'']) {
        return None;
    }
    let parts = inner.split(',').map(str::trim).collect::<Vec<_>>();
    let (language, options) = parts.split_first()?;
    let language = language.trim_start_matches('.');
    if language.is_empty()
        || language.contains(char::is_whitespace)
        || language.starts_with('#')
        || language.contains('=')
        || options.is_empty()
    {
        return None;
    }

    let mut lines = Vec::with_capacity(options.len());
    for option in options {
        let (key, value) = option.split_once('=')?;
        let key = key.trim();
        let value = normalize_quarto_option_value(value.trim())?;
        if key.is_empty()
            || value.is_empty()
            || !key
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
        {
            return None;
        }
        lines.push(format!("#| {key}: {value}"));
    }
    Some(format!("{{{language}}}{newline}{}", lines.join(newline)))
}

fn normalize_quarto_option_value(value: &str) -> Option<String> {
    if value.contains(char::is_whitespace) || value.contains(['{', '}', '[', ']']) {
        return None;
    }
    Some(match value {
        "TRUE" | "True" => "true".to_owned(),
        "FALSE" | "False" => "false".to_owned(),
        _ => value.to_owned(),
    })
}

fn split_line_ending(text: &str) -> (&str, &str) {
    if let Some(body) = text.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = text.strip_suffix('\n') {
        (body, "\n")
    } else if let Some(body) = text.strip_suffix('\r') {
        (body, "\r")
    } else {
        (text, "")
    }
}

pub(super) fn code_fence_local_skip(text: &str) -> bool {
    let Some(fence) = code_fence_at(text) else {
        return false;
    };
    let info = text.trim_start()[fence.len..].trim();
    code_fence_info_has_exact_local_skip(info)
}

fn code_fence_info_has_exact_local_skip(info: &str) -> bool {
    let tokens = info
        .split_whitespace()
        .map(code_fence_info_token)
        .collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if *token == "fmt:skip" && local_skip_directive_ends(&tokens, index + 1) {
            return true;
        }
        if *token == "fmt:"
            && tokens.get(index + 1).copied() == Some("skip")
            && local_skip_directive_ends(&tokens, index + 2)
        {
            return true;
        }
    }
    false
}

fn code_fence_info_token(token: &str) -> &str {
    token
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim_end_matches(',')
}

fn local_skip_directive_ends(tokens: &[&str], start: usize) -> bool {
    start >= tokens.len()
        || (tokens.get(start).copied() == Some("scope=next") && start + 1 == tokens.len())
}

pub(super) fn quarto_fence_skip(source: &SourceBuffer, mut line: usize, closing: usize) -> bool {
    while line < closing {
        let trimmed = source.line_text(line).trim_start();
        if trimmed.starts_with("#|") {
            let rest = trimmed.trim_start_matches("#|").trim();
            if rest == "fmt: skip" || rest == "fmt:skip" || rest == "fmt: skip scope=next" {
                return true;
            }
            line += 1;
            continue;
        }
        break;
    }
    false
}
