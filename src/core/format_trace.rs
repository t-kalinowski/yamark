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
        for node in &document.nodes {
            let NodeKind::Markdown(kind) = &node.kind else {
                continue;
            };
            if matches!(kind, MarkdownNodeKind::Blank | MarkdownNodeKind::Directive) {
                continue;
            }
            let Some(message) = markdown_decision_message(kind.clone(), &node.emit) else {
                continue;
            };
            let (line, column) = source.line_column_at_byte(node.span.start);
            diagnostics.push(Diagnostic::note(message).at(line, column));
        }
    }

    for nested in &document.nested {
        collect_markdown_decisions(source, nested, diagnostics);
    }
}

fn markdown_decision_message(kind: MarkdownNodeKind, emit: &EmitPlan) -> Option<String> {
    let kind = markdown_kind_name(kind);
    match emit {
        EmitPlan::Copy => Some(format!(
            "markdown trace: skipped kind={kind} emit=Copy reason={}",
            copy_skip_reason(kind)
        )),
        EmitPlan::Preserve => Some(format!(
            "markdown trace: skipped kind={kind} emit=Preserve reason=formatting_disabled"
        )),
        EmitPlan::MarkdownHeading { .. } => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownHeading"
        )),
        EmitPlan::MarkdownSetextHeading { .. } => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownSetextHeading"
        )),
        EmitPlan::MarkdownThematicBreak => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownThematicBreak"
        )),
        EmitPlan::MarkdownParagraph => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownParagraph"
        )),
        EmitPlan::MarkdownTable => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownTable"
        )),
        EmitPlan::MarkdownPandocTable => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownPandocTable"
        )),
        EmitPlan::MarkdownList => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownList"
        )),
        EmitPlan::MarkdownDefinitionList => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownDefinitionList"
        )),
        EmitPlan::MarkdownBlockquote => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownBlockquote"
        )),
        EmitPlan::MarkdownFrontMatter { .. } => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownFrontMatter"
        )),
        EmitPlan::MarkdownCodeFence {
            nested, supported, ..
        } => {
            let decision = if nested.is_some() || *supported {
                "formatted"
            } else {
                "skipped"
            };
            Some(format!(
                "markdown trace: {decision} kind={kind} emit=MarkdownCodeFence"
            ))
        }
        EmitPlan::MarkdownDiv { .. } => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownDiv"
        )),
        EmitPlan::MarkdownOpaque => Some(format!(
            "markdown trace: formatted kind={kind} emit=MarkdownOpaque"
        )),
        EmitPlan::ExternalPlugin { name, .. } => Some(format!(
            "markdown trace: formatted kind={kind} emit=ExternalPlugin formatter={name}"
        )),
        EmitPlan::YamlDocument
        | EmitPlan::EmbeddedMarkdownString { .. }
        | EmitPlan::EmbeddedMarkdownComment { .. }
        | EmitPlan::EmbeddedYamlComment { .. } => None,
    }
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
