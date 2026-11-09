use crate::core::document::{Document, DocumentKind, EmitPlan, MarkdownNodeKind, NodeKind};
use crate::core::source::SourceBuffer;
use crate::diagnostic::Diagnostic;

pub fn markdown_decision_diagnostics(
    source: &SourceBuffer,
    document: &Document,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    collect_markdown_decisions(source, document, &mut diagnostics);
    diagnostics
}

fn collect_markdown_decisions(
    source: &SourceBuffer,
    document: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if document.kind == DocumentKind::Markdown {
        for (index, node) in document.nodes.iter().enumerate() {
            let NodeKind::Markdown(kind) = &node.kind else {
                continue;
            };
            if matches!(kind, MarkdownNodeKind::Blank | MarkdownNodeKind::Directive) {
                continue;
            }
            let reason = finalized_skip_reason(document, index);
            let Some(message) = markdown_decision_message(kind.clone(), &node.emit, reason) else {
                continue;
            };
            let (line, column) = source.line_column_at_byte(node.span.start);
            diagnostics.push(Diagnostic::note(message).at(line, column));
        }
    }

    if document.skip_file || document.nested.is_empty() {
        return;
    }

    // A preserved container still owns its parsed children, but emission never
    // enters them. Mark those edges once instead of searching every parent for
    // each child. YAML-owned children retain their existing traversal here.
    let mut inactive = vec![false; document.nested.len()];
    for (index, node) in document.nodes.iter().enumerate() {
        if finalized_skip_reason(document, index).is_none() {
            continue;
        }
        let nested = match node.emit {
            EmitPlan::MarkdownFrontMatter { nested, .. }
            | EmitPlan::MarkdownDiv { nested, .. }
            | EmitPlan::EmbeddedMarkdownString { nested, .. }
            | EmitPlan::EmbeddedMarkdownComment { nested, .. }
            | EmitPlan::EmbeddedYamlComment { nested, .. }
            | EmitPlan::MarkdownCodeFence {
                nested: Some(nested),
                ..
            } => nested,
            _ => continue,
        };
        inactive[nested] = true;
    }
    for (index, nested) in document.nested.iter().enumerate() {
        if !inactive[index] {
            collect_markdown_decisions(source, nested, diagnostics);
        }
    }
}

/// Report the finalized outcome that emission consumes, rather than inferring
/// success from a node's nominal formatting category. Reasons deliberately name
/// retained decisions; tracing must not rerun recognition or layout.
fn finalized_skip_reason(document: &Document, index: usize) -> Option<&'static str> {
    if document.skip_file {
        return Some("skip_file");
    }
    let node = &document.nodes[index];
    if document.state(node.state).preserve {
        return Some("formatting_disabled");
    }
    let retained = document.markdown.get(index)?;
    if retained.preserve {
        Some("retained_preservation_policy")
    } else if retained.plan.is_none() {
        Some("unavailable_layout_plan")
    } else {
        None
    }
}

fn markdown_decision_message(
    kind: MarkdownNodeKind,
    emit: &EmitPlan,
    finalized_reason: Option<&'static str>,
) -> Option<String> {
    let kind = markdown_kind_name(kind);
    let (name, reason) = match emit {
        EmitPlan::Copy => ("Copy", Some(copy_skip_reason(kind))),
        EmitPlan::Preserve => ("Preserve", Some("formatting_disabled")),
        EmitPlan::MarkdownHeading { .. } => ("MarkdownHeading", None),
        EmitPlan::MarkdownSetextHeading { .. } => ("MarkdownSetextHeading", None),
        EmitPlan::MarkdownThematicBreak => ("MarkdownThematicBreak", None),
        EmitPlan::MarkdownParagraph => ("MarkdownParagraph", None),
        EmitPlan::MarkdownTable => ("MarkdownTable", None),
        EmitPlan::MarkdownPandocTable => ("MarkdownPandocTable", None),
        EmitPlan::MarkdownList => ("MarkdownList", None),
        EmitPlan::MarkdownDefinitionList => ("MarkdownDefinitionList", None),
        EmitPlan::MarkdownBlockquote => ("MarkdownBlockquote", None),
        EmitPlan::MarkdownFrontMatter { .. } => ("MarkdownFrontMatter", None),
        EmitPlan::MarkdownCodeFence { .. } => ("MarkdownCodeFence", None),
        EmitPlan::MarkdownDiv { .. } => ("MarkdownDiv", None),
        EmitPlan::MarkdownOpaque => ("MarkdownOpaque", None),
        EmitPlan::ExternalPlugin { .. } => ("ExternalPlugin", None),
        EmitPlan::YamlDocument
        | EmitPlan::EmbeddedMarkdownString { .. }
        | EmitPlan::EmbeddedMarkdownComment { .. }
        | EmitPlan::EmbeddedYamlComment { .. } => return None,
    };
    let reason = finalized_reason.or(reason);
    let skipped = reason.is_some()
        || matches!(
            emit,
            EmitPlan::MarkdownCodeFence {
                nested: None,
                supported: false,
                ..
            }
        );
    let decision = if skipped { "skipped" } else { "formatted" };
    let mut message = format!("markdown trace: {decision} kind={kind} emit={name}");
    if let EmitPlan::ExternalPlugin { name, .. } = emit {
        message.push_str(&format!(" formatter={name}"));
    }
    if let Some(reason) = reason {
        message.push_str(&format!(" reason={reason}"));
    }
    Some(message)
}

fn markdown_kind_name(kind: MarkdownNodeKind) -> &'static str {
    match kind {
        MarkdownNodeKind::FrontMatter => "FrontMatter",
        MarkdownNodeKind::Blank => "Blank",
        MarkdownNodeKind::Directive => "Directive",
        MarkdownNodeKind::Heading => "Heading",
        MarkdownNodeKind::SetextHeading => "SetextHeading",
        MarkdownNodeKind::ThematicBreak => "ThematicBreak",
        MarkdownNodeKind::Paragraph => "Paragraph",
        MarkdownNodeKind::Table => "Table",
        MarkdownNodeKind::GfmPipeTable => "GfmPipeTable",
        MarkdownNodeKind::PandocTable => "PandocTable",
        MarkdownNodeKind::List => "List",
        MarkdownNodeKind::DefinitionList => "DefinitionList",
        MarkdownNodeKind::FootnoteDefinition => "FootnoteDefinition",
        MarkdownNodeKind::ReferenceDefinition => "ReferenceDefinition",
        MarkdownNodeKind::Blockquote => "Blockquote",
        MarkdownNodeKind::CodeFence => "CodeFence",
        MarkdownNodeKind::QuartoDiv => "QuartoDiv",
        MarkdownNodeKind::Shortcode => "Shortcode",
        MarkdownNodeKind::DisplayMath => "DisplayMath",
        MarkdownNodeKind::HtmlComment => "HtmlComment",
        MarkdownNodeKind::Raw => "Raw",
    }
}

fn copy_skip_reason(kind: &str) -> &'static str {
    match kind {
        "Raw" => "raw_block",
        "QuartoDiv" => "quarto_div",
        "HtmlComment" => "html_comment",
        "CodeFence" => "code_fence",
        "FrontMatter" => "front_matter",
        _ => "unsupported_markdown_construct_or_template",
    }
}
