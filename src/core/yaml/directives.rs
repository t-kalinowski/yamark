//! YAML directive scope, trivia, and propagation into nested Markdown.

use super::YamlParser;
use super::parse::config_for_directive_state;
use super::syntax::{
    LineValue, document_marker, document_marker_line_info, scalar_metadata,
    standard_yaml_directive, tab_in_indentation, yaml_error_at,
};
use crate::core::directives::{
    Directive, DirectiveDelta, file_scope_delta, parse_yaml_hash_directive,
};
use crate::core::source::{SourceSpan, Span};
use crate::core::yaml_model::{YamlAstKind, YamlTrivia, YamlTriviaKind};
use crate::diagnostic::Result;
use crate::plugins::PluginRegistry;

impl<'src, 'cfg> YamlParser<'src, 'cfg> {
    pub(super) fn take_leading_trivia(&mut self) -> Result<Vec<YamlTrivia>> {
        let mut trivia = std::mem::take(&mut self.held_trivia);
        if self.doc.skip_file {
            return Ok(trivia);
        }
        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                break;
            }
            let kind = if text.trim_ascii().is_empty() {
                Some(YamlTriviaKind::Blank)
            } else if let Some(marker) = document_marker_line_info(text) {
                if let Some(content_start) = marker.inline_content_start {
                    let marker_span =
                        Span::new(line.full.start(), line.text.start() + content_start);
                    if trivia.last().is_some_and(|trivia| {
                        trivia.kind == YamlTriviaKind::DocumentMarker
                            && trivia.span.span() == marker_span
                    }) {
                        break;
                    }
                    trivia.push(YamlTrivia {
                        kind: YamlTriviaKind::DocumentMarker,
                        span: SourceSpan::new(marker_span),
                    });
                    break;
                }
                Some(YamlTriviaKind::DocumentMarker)
            } else if standard_yaml_directive(text) {
                Some(YamlTriviaKind::Directive)
            } else if let Some(directive) = parse_yaml_hash_directive(text).map_err(|message| {
                yaml_error_at(
                    self.source,
                    line.text.start() + text.find('#').unwrap_or(0),
                    message,
                )
            })? {
                let directive = self.infer_yaml_template_directive_scope(directive, self.line)?;
                self.apply_yaml_directive(
                    directive,
                    line.text.start() + text.find('#').unwrap_or(0),
                )?;
                Some(YamlTriviaKind::Directive)
            } else if text.trim_ascii_start().starts_with('#') {
                Some(YamlTriviaKind::Comment)
            } else {
                None
            };
            let Some(kind) = kind else {
                break;
            };
            trivia.push(YamlTrivia {
                kind,
                span: SourceSpan::new(line.full.into()),
            });
            self.line += 1;
            if self.doc.skip_file {
                break;
            }
        }
        Ok(trivia)
    }

    fn infer_yaml_template_directive_scope(
        &self,
        directive: Directive,
        line: usize,
    ) -> Result<Directive> {
        let Directive::Template { scope, delimiter } = directive else {
            return Ok(directive);
        };
        if scope != crate::core::directives::Scope::Next
            || self.source.line_text(line).contains("scope=")
        {
            return Ok(Directive::Template { scope, delimiter });
        }
        if self.yaml_template_has_immediate_target(line) {
            return Ok(Directive::Template { scope, delimiter });
        }
        if self.yaml_directive_is_isolated(line) {
            return Ok(Directive::Template {
                scope: crate::core::directives::Scope::FromHere,
                delimiter,
            });
        }
        Err(yaml_error_at(
            self.source,
            self.source.lines[line].text.start()
                + self.source.line_text(line).find('#').unwrap_or(0),
            "fmt: template.delimiters needs explicit scope",
        ))
    }

    fn yaml_template_has_immediate_target(&self, line: usize) -> bool {
        let Some(next) = line.checked_add(1).filter(|next| *next < self.end) else {
            return false;
        };
        let text = self.source.line_text(next);
        !(text.trim_ascii().is_empty()
            || text.trim_ascii_start().starts_with('#')
            || document_marker(text)
            || standard_yaml_directive(text))
    }

    fn yaml_directive_is_isolated(&self, line: usize) -> bool {
        let before_blank =
            line == self.start || self.source.line_text(line - 1).trim_ascii().is_empty();
        let after_blank =
            line + 1 >= self.end || self.source.line_text(line + 1).trim_ascii().is_empty();
        before_blank && after_blank
    }

    pub(super) fn apply_same_line_yaml_directive(&mut self, comment: Option<Span>) -> Result<bool> {
        let Some((comment, directive)) = self.parse_same_line_yaml_directive(comment)? else {
            return Ok(false);
        };
        self.apply_yaml_directive(directive, comment.start)?;
        Ok(true)
    }

    pub(super) fn reject_same_line_yaml_directive(&self, comment: Option<Span>) -> Result<()> {
        let Some((comment, _)) = self.parse_same_line_yaml_directive(comment)? else {
            return Ok(());
        };
        Err(yaml_error_at(
            self.source,
            comment.start,
            "fmt directive is not supported in this position",
        ))
    }

    pub(super) fn reject_populated_same_line_yaml_directive(
        &self,
        line_value: LineValue,
    ) -> Result<()> {
        let metadata = scalar_metadata(self.source, line_value.value);
        if !self.source.slice(metadata.content).trim_ascii().is_empty() {
            self.reject_same_line_yaml_directive(line_value.trailing_comment)?;
        }
        Ok(())
    }

    fn parse_same_line_yaml_directive(
        &self,
        comment: Option<Span>,
    ) -> Result<Option<(Span, Directive)>> {
        let Some(comment) = comment else {
            return Ok(None);
        };
        let text = self.source.slice(comment);
        let directive = parse_yaml_hash_directive(text)
            .map_err(|message| yaml_error_at(self.source, comment.start, message))?;
        Ok(directive.map(|directive| (comment, directive)))
    }

    fn apply_yaml_directive(&mut self, directive: Directive, byte: usize) -> Result<()> {
        if let Directive::Embedded { name } = &directive
            && !PluginRegistry::is_known_formatter(self.config, name)
        {
            return Err(yaml_error_at(
                self.source,
                byte,
                format!("unknown embedded formatter: {name}"),
            ));
        }
        let delta = file_scope_delta(&directive);
        self.engine
            .apply_yaml_directive(&mut self.doc, directive)
            .map_err(|message| yaml_error_at(self.source, byte, message))?;
        if let Some(delta) = delta {
            self.file_scope_delta.merge_from(delta);
        }
        Ok(())
    }

    pub(super) fn patch_existing_ast_states(&mut self, delta: &DirectiveDelta) {
        let mut patched = Vec::with_capacity(self.ast.nodes.len());
        for node in &self.ast.nodes {
            let mut state = self.doc.states.get(node.state).clone();
            delta.apply_to(&mut state);
            patched.push(self.doc.states.intern(state));
        }
        for (node, state) in self.ast.nodes.iter_mut().zip(patched) {
            node.state = state;
        }
    }

    pub(super) fn patch_nested_markdown_documents(&mut self, delta: &DirectiveDelta) -> Result<()> {
        let mut nested_markdown = Vec::new();
        for node in &self.ast.nodes {
            let YamlAstKind::Scalar(scalar) = &node.kind else {
                continue;
            };
            let Some(nested) = scalar.nested else {
                continue;
            };
            if scalar.body.is_none() {
                continue;
            }
            nested_markdown.push((nested as usize, node.state));
        }
        for (nested, state) in nested_markdown {
            let state = self.doc.state(state).clone();
            let nested_config = config_for_directive_state(self.config, &state);
            if self.plan_emits {
                crate::core::markdown::apply_file_scope_delta_to_markdown_document(
                    self.source,
                    &mut self.doc.nested[nested],
                    delta,
                    state.markdown_options(self.options),
                    &nested_config,
                )?;
            } else {
                crate::core::markdown::apply_file_scope_delta_to_markdown_document_for_concrete_validation(
                        self.source,
                        &mut self.doc.nested[nested],
                        delta,
                        state.markdown_options(self.options),
                        &nested_config,
                    )?;
            }
        }
        Ok(())
    }

    pub(super) fn unread_trivia(&mut self, trivia: Vec<YamlTrivia>) {
        if trivia.is_empty() {
            return;
        }
        if self.held_trivia.is_empty() {
            self.held_trivia = trivia;
        } else {
            let mut merged = trivia;
            merged.append(&mut self.held_trivia);
            self.held_trivia = merged;
        }
    }
}
