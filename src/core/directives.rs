use crate::core::document::{Document, FormatOptions, MarkdownWrap};

const WRAP_OPTION_ERROR: &str =
    "fmt: wrap must be none, paragraph, sentence, or a positive integer";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(pub u32);

impl StateId {
    pub fn new(index: usize) -> Self {
        assert!(
            index <= u32::MAX as usize,
            "directive state count exceeded u32::MAX"
        );
        Self(index as u32)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateDelimiter {
    pub open: String,
    pub close: String,
}

pub fn contains_template_span(source: &str, delimiters: &[TemplateDelimiter]) -> bool {
    delimiters
        .iter()
        .any(|delimiter| template_span_in_source(source, delimiter, TemplateSpanMode::Generic))
}

pub fn contains_markdown_template_span(source: &str, delimiters: &[TemplateDelimiter]) -> bool {
    delimiters
        .iter()
        .any(|delimiter| template_span_in_source(source, delimiter, TemplateSpanMode::Markdown))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemplateSpanMode {
    Generic,
    Markdown,
}

fn template_span_in_source(
    source: &str,
    delimiter: &TemplateDelimiter,
    mode: TemplateSpanMode,
) -> bool {
    if delimiter.open.is_empty() || delimiter.close.is_empty() {
        return false;
    }
    let mut search_start = 0usize;
    while search_start < source.len() {
        let Some(relative_open) = source[search_start..].find(&delimiter.open) else {
            return false;
        };
        let open = search_start + relative_open;
        let content_start = open + delimiter.open.len();
        let Some(relative_close) = source[content_start..].find(&delimiter.close) else {
            return false;
        };
        let close = content_start + relative_close;
        if mode == TemplateSpanMode::Markdown
            && is_hugo_shortcode_template_span(source, delimiter, open, close)
        {
            search_start = close + delimiter.close.len();
            continue;
        }
        return true;
    }
    false
}

fn is_hugo_shortcode_template_span(
    source: &str,
    delimiter: &TemplateDelimiter,
    open: usize,
    close: usize,
) -> bool {
    delimiter.open == "{{"
        && delimiter.close == "}}"
        && source[open + delimiter.open.len()..].starts_with(['<', '%'])
        && source[..close].trim_end().ends_with(['>', '%'])
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectiveState {
    pub preserve: bool,
    pub markdown_target: bool,
    pub yaml_compact: Option<bool>,
    pub markdown_wrap: Option<MarkdownWrap>,
    pub markdown_wrap_at_column: Option<usize>,
    pub markdown_canonical: Option<bool>,
    pub markdown_format_footnotes: Option<bool>,
    pub table_compact: Option<bool>,
    pub template_delimiters: Vec<TemplateDelimiter>,
    pub embedded_formatter: Option<String>,
}

impl DirectiveState {
    pub fn markdown_options(&self, base: FormatOptions) -> FormatOptions {
        let mut options = base;
        if let Some(wrap) = self.markdown_wrap {
            options.markdown_wrap = wrap;
        }
        if let Some(width) = self.markdown_wrap_at_column {
            options.markdown_wrap_at_column = width.max(1);
        }
        if let Some(canonical) = self.markdown_canonical {
            options.markdown_canonical = canonical;
        }
        if let Some(format_footnotes) = self.markdown_format_footnotes {
            options.markdown_format_footnotes = format_footnotes;
        }
        if base.markdown_preserve_footnotes {
            options.markdown_format_footnotes = false;
        }
        options
    }

    pub fn yaml_options(&self, base: FormatOptions) -> FormatOptions {
        let mut options = base;
        if let Some(compact) = self.yaml_compact {
            options.yaml_compact = compact;
        }
        options
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectiveDelta {
    pub preserve: Option<bool>,
    pub markdown_target: Option<bool>,
    pub yaml_compact: Option<bool>,
    pub markdown_wrap: Option<MarkdownWrap>,
    pub markdown_wrap_at_column: Option<usize>,
    pub markdown_canonical: Option<bool>,
    pub markdown_format_footnotes: Option<bool>,
    pub table_compact: Option<bool>,
    pub add_template_delimiters: Vec<TemplateDelimiter>,
    pub embedded_formatter: Option<String>,
}

impl DirectiveDelta {
    pub fn preserve() -> Self {
        Self {
            preserve: Some(true),
            ..Self::default()
        }
    }
    pub fn markdown_target() -> Self {
        Self {
            markdown_target: Some(true),
            ..Self::default()
        }
    }

    pub fn apply_to(&self, state: &mut DirectiveState) {
        if let Some(value) = self.preserve {
            state.preserve = value;
        }
        if let Some(value) = self.markdown_target {
            state.markdown_target = value;
        }
        if let Some(value) = self.yaml_compact {
            state.yaml_compact = Some(value);
        }
        if let Some(value) = self.markdown_wrap {
            state.markdown_wrap = Some(value);
        }
        if let Some(value) = self.markdown_wrap_at_column {
            state.markdown_wrap_at_column = Some(value);
        }
        if let Some(value) = self.markdown_canonical {
            state.markdown_canonical = Some(value);
        }
        if let Some(value) = self.markdown_format_footnotes {
            state.markdown_format_footnotes = Some(value);
        }
        if let Some(value) = self.table_compact {
            state.table_compact = Some(value);
        }
        for delimiter in &self.add_template_delimiters {
            if !state.template_delimiters.contains(delimiter) {
                state.template_delimiters.push(delimiter.clone());
            }
        }
        if let Some(value) = &self.embedded_formatter {
            state.embedded_formatter = Some(value.clone());
        }
    }

    pub fn merge_from(&mut self, other: DirectiveDelta) {
        if other.preserve.is_some() {
            self.preserve = other.preserve;
        }
        if other.markdown_target.is_some() {
            self.markdown_target = other.markdown_target;
        }
        if other.yaml_compact.is_some() {
            self.yaml_compact = other.yaml_compact;
        }
        if other.markdown_wrap.is_some() {
            self.markdown_wrap = other.markdown_wrap;
        }
        if other.markdown_wrap_at_column.is_some() {
            self.markdown_wrap_at_column = other.markdown_wrap_at_column;
        }
        if other.markdown_canonical.is_some() {
            self.markdown_canonical = other.markdown_canonical;
        }
        if other.markdown_format_footnotes.is_some() {
            self.markdown_format_footnotes = other.markdown_format_footnotes;
        }
        if other.table_compact.is_some() {
            self.table_compact = other.table_compact;
        }
        self.add_template_delimiters
            .extend(other.add_template_delimiters);
        if other.embedded_formatter.is_some() {
            self.embedded_formatter = other.embedded_formatter;
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirectiveStateTable {
    states: Vec<DirectiveState>,
}

impl DirectiveStateTable {
    pub fn new() -> Self {
        Self {
            states: vec![DirectiveState::default()],
        }
    }
    pub fn get(&self, id: StateId) -> &DirectiveState {
        &self.states[id.index()]
    }
    pub fn iter(&self) -> impl Iterator<Item = (StateId, &DirectiveState)> {
        self.states
            .iter()
            .enumerate()
            .map(|(index, state)| (StateId::new(index), state))
    }
    pub fn intern(&mut self, state: DirectiveState) -> StateId {
        if let Some(index) = self.states.iter().position(|existing| existing == &state) {
            return StateId::new(index);
        }
        let id = StateId::new(self.states.len());
        self.states.push(state);
        id
    }
}

impl Default for DirectiveStateTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Next,
    FromHere,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveTargetKind {
    General,
    YamlScalar,
    YamlSequence,
    YamlMapping,
    YamlCollection,
    YamlFlowSequence,
    YamlFlowMapping,
    YamlUnsupported,
}

impl DirectiveTargetKind {
    fn accepts(self, target: Self) -> bool {
        matches!(self, Self::General)
            || self == target
            || matches!(
                (self, target),
                (
                    Self::YamlCollection,
                    Self::YamlMapping
                        | Self::YamlSequence
                        | Self::YamlFlowSequence
                        | Self::YamlFlowMapping
                ) | (Self::YamlSequence, Self::YamlFlowSequence)
                    | (
                        Self::YamlScalar,
                        Self::YamlFlowSequence | Self::YamlFlowMapping
                    )
            )
            || matches!(target, Self::YamlUnsupported)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Skip,
    SkipFile,
    Off,
    On,
    Markdown {
        scope: Scope,
        delta: DirectiveDelta,
    },
    Template {
        scope: Scope,
        delimiter: TemplateDelimiter,
    },
    Embedded {
        name: String,
    },
    Compact {
        scope: Scope,
        compact: bool,
    },
    Table {
        compact: bool,
    },
}

pub(crate) fn file_scope_delta(directive: &Directive) -> Option<DirectiveDelta> {
    match directive {
        Directive::Markdown {
            scope: Scope::File,
            delta,
        } => Some(delta.clone()),
        Directive::Template {
            scope: Scope::File,
            delimiter,
        } => Some(DirectiveDelta {
            add_template_delimiters: vec![delimiter.clone()],
            ..DirectiveDelta::default()
        }),
        Directive::Compact {
            scope: Scope::File,
            compact,
        } => Some(DirectiveDelta {
            yaml_compact: Some(*compact),
            ..DirectiveDelta::default()
        }),
        _ => None,
    }
}

pub(crate) fn directive_delta_affects_markdown(delta: &DirectiveDelta) -> bool {
    delta.markdown_target.is_some()
        || delta.markdown_wrap.is_some()
        || delta.markdown_wrap_at_column.is_some()
        || delta.markdown_canonical.is_some()
        || delta.markdown_format_footnotes.is_some()
        || !delta.add_template_delimiters.is_empty()
}

#[derive(Debug, Clone)]
struct PendingDirective {
    target: DirectiveTargetKind,
    delta: DirectiveDelta,
    error: Option<&'static str>,
}

#[derive(Debug)]
pub struct DirectiveEngine {
    active: DirectiveState,
    active_id: Option<StateId>,
    pending_next: Vec<PendingDirective>,
    disabled: bool,
}

impl Clone for DirectiveEngine {
    fn clone(&self) -> Self {
        Self {
            active: self.active.clone(),
            active_id: None,
            pending_next: self.pending_next.clone(),
            disabled: self.disabled,
        }
    }
}

impl DirectiveEngine {
    pub fn new() -> Self {
        Self {
            active: DirectiveState::default(),
            active_id: Some(StateId(0)),
            pending_next: Vec::new(),
            disabled: false,
        }
    }

    pub fn new_with_template_delimiters(template_delimiters: Vec<TemplateDelimiter>) -> Self {
        let active = DirectiveState {
            template_delimiters,
            ..DirectiveState::default()
        };
        Self {
            active,
            active_id: None,
            pending_next: Vec::new(),
            disabled: false,
        }
    }

    pub fn state_for_node(&mut self, document: &mut Document, target: bool) -> StateId {
        self.state_for_target_kind(document, target.then_some(DirectiveTargetKind::General))
    }

    pub fn state_for_yaml_node(
        &mut self,
        document: &mut Document,
        target: DirectiveTargetKind,
    ) -> StateId {
        self.state_for_target_kind(document, Some(target))
    }

    fn state_for_target_kind(
        &mut self,
        document: &mut Document,
        target: Option<DirectiveTargetKind>,
    ) -> StateId {
        if !self.disabled && self.pending_next.is_empty() {
            if let Some(id) = self.active_id {
                return id;
            }
            let id = document.states.intern(self.active.clone());
            self.active_id = Some(id);
            return id;
        }
        let mut state = self.active.clone();
        if self.disabled {
            state.preserve = true;
        }
        if let Some(target) = target {
            let pending = std::mem::take(&mut self.pending_next);
            for pending in pending {
                if pending.target.accepts(target) {
                    pending.delta.apply_to(&mut state);
                } else {
                    self.pending_next.push(pending);
                }
            }
        }
        document.states.intern(state)
    }

    pub fn apply_directive(&mut self, document: &mut Document, directive: Directive) {
        let _ = self.apply_directive_inner(document, directive, false);
    }

    pub fn apply_yaml_directive(
        &mut self,
        document: &mut Document,
        directive: Directive,
    ) -> std::result::Result<(), &'static str> {
        self.apply_directive_inner(document, directive, true)
    }

    pub fn pending_target_error(&self) -> Option<&'static str> {
        self.pending_next.iter().find_map(|pending| pending.error)
    }

    pub(crate) fn has_pending_target_directives(&self) -> bool {
        !self.pending_next.is_empty()
    }

    pub fn formatting_disabled(&self) -> bool {
        self.disabled
    }

    fn apply_directive_inner(
        &mut self,
        document: &mut Document,
        directive: Directive,
        checked_yaml: bool,
    ) -> std::result::Result<(), &'static str> {
        let scalar_target = if checked_yaml {
            DirectiveTargetKind::YamlScalar
        } else {
            DirectiveTargetKind::General
        };
        let sequence_target = if checked_yaml {
            DirectiveTargetKind::YamlSequence
        } else {
            DirectiveTargetKind::General
        };

        match directive {
            Directive::Skip => {
                self.push_pending(
                    DirectiveTargetKind::General,
                    DirectiveDelta::preserve(),
                    checked_yaml.then_some("fmt: skip has no target"),
                );
            }
            Directive::SkipFile => document.skip_file = true,
            Directive::Off => {
                if checked_yaml && self.disabled {
                    return Err("nested fmt: off");
                }
                self.disabled = true;
                self.active_id = None;
            }
            Directive::On => {
                if checked_yaml && !self.disabled {
                    return Err("fmt: on without active fmt: off");
                }
                self.disabled = false;
                self.active_id = None;
            }
            Directive::Markdown { scope, mut delta } => {
                if !checked_yaml && scope == Scope::Next {
                    delta.markdown_target = Some(true);
                }
                self.apply_scoped_delta(
                    document,
                    scope,
                    scalar_target,
                    delta,
                    "fmt: markdown has no target",
                );
            }
            Directive::Template { scope, delimiter } => {
                let delta = DirectiveDelta {
                    add_template_delimiters: vec![delimiter],
                    ..DirectiveDelta::default()
                };
                self.apply_scoped_delta(
                    document,
                    scope,
                    DirectiveTargetKind::General,
                    delta,
                    "fmt: template.delimiters has no target",
                );
            }
            Directive::Embedded { name } => self.push_pending(
                scalar_target,
                DirectiveDelta {
                    embedded_formatter: Some(name),
                    ..DirectiveDelta::default()
                },
                "fmt embedded formatter has no target",
            ),
            Directive::Compact { scope, compact } => {
                self.apply_scoped_delta(
                    document,
                    scope,
                    DirectiveTargetKind::YamlCollection,
                    DirectiveDelta {
                        yaml_compact: Some(compact),
                        ..DirectiveDelta::default()
                    },
                    "fmt: compact has no target",
                );
            }
            Directive::Table { compact } => self.push_pending(
                sequence_target,
                DirectiveDelta {
                    table_compact: Some(compact),
                    ..DirectiveDelta::default()
                },
                if compact {
                    "fmt: compact table has no target"
                } else {
                    "fmt: table has no target"
                },
            ),
        }
        Ok(())
    }

    fn apply_scoped_delta(
        &mut self,
        document: &mut Document,
        scope: Scope,
        next_target: DirectiveTargetKind,
        delta: DirectiveDelta,
        pending_error: &'static str,
    ) {
        match scope {
            Scope::Next => self.push_pending(next_target, delta, pending_error),
            Scope::FromHere => {
                delta.apply_to(&mut self.active);
                self.active_id = None;
            }
            Scope::File => {
                document.patch_all_states(delta.clone());
                delta.apply_to(&mut self.active);
                self.active_id = None;
            }
        }
    }

    fn push_pending(
        &mut self,
        target: DirectiveTargetKind,
        delta: DirectiveDelta,
        error: impl Into<Option<&'static str>>,
    ) {
        self.pending_next.push(PendingDirective {
            target,
            delta,
            error: error.into(),
        });
    }
}

impl Default for DirectiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_markdown_html_directive(line: &str) -> Option<Directive> {
    let trimmed = directive_line_start(line).trim();
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    parse_fmt_directive(inner.strip_prefix("fmt:")?.trim())
}

pub fn parse_markdown_html_directive_checked(
    line: &str,
) -> std::result::Result<Option<Directive>, String> {
    let trimmed = directive_line_start(line).trim();
    let Some(inner) = trimmed
        .strip_prefix("<!--")
        .and_then(|inner| inner.strip_suffix("-->"))
    else {
        return Ok(None);
    };
    let inner = inner.trim();
    let Some(rest) = inner.strip_prefix("fmt:") else {
        return Ok(None);
    };
    parse_markdown_fmt_directive(rest.trim()).map(Some)
}

pub fn parse_hash_directive(line: &str) -> Option<Directive> {
    let rest = directive_line_start(line).strip_prefix('#')?.trim();
    parse_fmt_directive(rest.strip_prefix("fmt:")?.trim())
}

pub fn parse_hash_directive_checked(line: &str) -> std::result::Result<Option<Directive>, String> {
    let Some(rest) = directive_line_start(line).strip_prefix('#') else {
        return Ok(None);
    };
    let rest = rest.trim();
    let Some(rest) = rest.strip_prefix("fmt:") else {
        return Ok(None);
    };
    parse_markdown_fmt_directive(rest.trim()).map(Some)
}

pub fn parse_yaml_hash_directive(line: &str) -> std::result::Result<Option<Directive>, String> {
    let Some(rest) = directive_line_start(line).strip_prefix('#') else {
        return Ok(None);
    };
    let rest = rest.trim();
    let Some(rest) = rest.strip_prefix("fmt:") else {
        return Ok(None);
    };
    parse_yaml_fmt_directive(rest.trim()).map(Some)
}

fn directive_line_start(line: &str) -> &str {
    line.trim_start_matches('\u{feff}').trim_start()
}

fn parse_fmt_directive(rest: &str) -> Option<Directive> {
    let rest = rest.trim();
    match rest {
        "skip" => return Some(Directive::Skip),
        "skip scope=next" => return Some(Directive::Skip),
        "skip scope=file" => return Some(Directive::SkipFile),
        "skip file" => return Some(Directive::SkipFile),
        "off" => return Some(Directive::Off),
        "off scope=from-here" => return Some(Directive::Off),
        "on" => return Some(Directive::On),
        _ => {}
    }

    let mut fields = parse_directive_tokens(rest).ok()?;
    if fields.is_empty() {
        return Some(Directive::Markdown {
            scope: Scope::File,
            delta: DirectiveDelta::default(),
        });
    }

    let action = fields.remove(0);
    if action.text == "markdown" {
        let (scope, mut delta) = parse_scope_and_options(&fields, Scope::Next);
        if scope == Scope::Next {
            delta.markdown_target = Some(true);
        }
        return Some(Directive::Markdown { scope, delta });
    }
    if action.text == "template.delimiters" {
        let (scope, _) = parse_scope_and_options(&fields, Scope::Next);
        let delimiter = parse_delimiter(&fields).unwrap_or(TemplateDelimiter {
            open: "{{".to_owned(),
            close: "}}".to_owned(),
        });
        return Some(Directive::Template { scope, delimiter });
    }
    if action.text == "compact" && fields.first().is_some_and(|field| field.text == "table") {
        return Some(Directive::Table { compact: true });
    }
    if action.text == "embedded" {
        let name = fields
            .first()
            .map(|field| field.text.clone())
            .unwrap_or_else(|| "default".to_owned());
        return Some(Directive::Embedded { name });
    }
    if action.text == "table" {
        let compact = fields
            .iter()
            .any(|field| field.text == "compact" || field.text == "compact=true");
        return Some(Directive::Table { compact });
    }
    if !action.text.contains('=') && !matches!(action.text.as_str(), "canonical" | "markdown") {
        return Some(Directive::Embedded { name: action.text });
    }

    let mut option_fields = Vec::with_capacity(fields.len() + 1);
    option_fields.push(action);
    option_fields.extend(fields);
    let (scope, delta) = parse_scope_and_options(&option_fields, Scope::File);
    Some(Directive::Markdown { scope, delta })
}

fn parse_markdown_fmt_directive(rest: &str) -> std::result::Result<Directive, String> {
    let mut fields = parse_directive_tokens(rest)?;
    if fields.is_empty() {
        return Err("invalid fmt directive: empty".to_owned());
    }

    let action = fields.remove(0);
    match action.text.as_str() {
        "skip" => parse_yaml_skip_directive(&fields),
        "off" => parse_yaml_off_directive(&fields),
        "on" => parse_yaml_on_directive(&fields),
        "markdown" => {
            let (scope, mut delta) = parse_scope_and_options_checked(&fields, Scope::Next)?;
            reject_markdown_without_options_for_broad_scope(scope, &delta)?;
            if scope == Scope::Next {
                delta.markdown_target = Some(true);
            }
            Ok(Directive::Markdown { scope, delta })
        }
        "template.delimiters" => parse_yaml_template_directive(&fields),
        "compact" if fields.first().is_some_and(|field| field.text == "table") => {
            Err(format!("invalid fmt directive: {rest}"))
        }
        "embedded" => Err(format!("invalid fmt directive: {rest}")),
        "table" => Err(format!("invalid fmt directive: {rest}")),
        _ if !action.text.contains('=')
            && !matches!(action.text.as_str(), "canonical" | "markdown") =>
        {
            Err(format!("invalid fmt directive: {rest}"))
        }
        _ => {
            let mut option_fields = Vec::with_capacity(fields.len() + 1);
            option_fields.push(action);
            option_fields.extend(fields);
            let (scope, delta) = parse_scope_and_options_checked(&option_fields, Scope::File)?;
            Ok(Directive::Markdown { scope, delta })
        }
    }
}

fn parse_scope_and_options(
    fields: &[DirectiveToken],
    default_scope: Scope,
) -> (Scope, DirectiveDelta) {
    let mut scope = default_scope;
    let mut delta = DirectiveDelta::default();
    for field in fields {
        let field = field.text.as_str();
        if let Some(value) = field.strip_prefix("scope=") {
            scope = match value {
                "next" => Scope::Next,
                "from-here" => Scope::FromHere,
                "file" => Scope::File,
                _ => scope,
            };
        } else if let Some(value) = field.strip_prefix("wrap=") {
            match value {
                "none" => delta.markdown_wrap = Some(MarkdownWrap::None),
                "paragraph" => delta.markdown_wrap = Some(MarkdownWrap::Paragraph),
                "sentence" => delta.markdown_wrap = Some(MarkdownWrap::Sentence),
                value => {
                    if let Ok(width) = value.parse::<usize>() {
                        delta.markdown_wrap = Some(MarkdownWrap::Column);
                        delta.markdown_wrap_at_column = Some(width);
                    }
                }
            }
        } else if let Some(value) = field.strip_prefix("canonical=") {
            delta.markdown_canonical = Some(matches!(value, "true" | "yes" | "1"));
        } else if field == "canonical" {
            delta.markdown_canonical = Some(true);
        } else if let Some(value) = field.strip_prefix("footnotes=") {
            delta.markdown_format_footnotes =
                Some(!matches!(value, "preserve" | "none" | "false" | "no" | "0"));
        } else if field == "markdown" {
        }
    }
    (scope, delta)
}

fn parse_yaml_fmt_directive(rest: &str) -> std::result::Result<Directive, String> {
    let mut fields = parse_directive_tokens(rest)?;
    if fields.is_empty() {
        return Err("invalid fmt directive: empty".to_owned());
    }

    let action = fields.remove(0);
    match action.text.as_str() {
        "skip" => parse_yaml_skip_directive(&fields),
        "off" => parse_yaml_off_directive(&fields),
        "on" => parse_yaml_on_directive(&fields),
        "markdown" => {
            let (scope, mut delta) = parse_scope_and_options_checked(&fields, Scope::Next)?;
            reject_markdown_without_options_for_broad_scope(scope, &delta)?;
            if scope == Scope::Next {
                delta.markdown_target = Some(true);
            }
            Ok(Directive::Markdown { scope, delta })
        }
        "template.delimiters" => parse_yaml_template_directive(&fields),
        "compact" if fields.first().is_some_and(|field| field.text == "table") => {
            parse_yaml_table_directive(&fields[1..], true)
        }
        "compact" => parse_yaml_compact_directive(&fields, true),
        _ if action.text.starts_with("compact=") => {
            let compact = parse_bool_option(action.text.trim_start_matches("compact="), "compact")?;
            parse_yaml_compact_directive(&fields, compact)
        }
        "embedded" => parse_yaml_embedded_keyword_directive(&fields, rest),
        "table" => parse_yaml_table_directive(&fields, false),
        _ if !action.text.contains('=')
            && !matches!(action.text.as_str(), "canonical" | "markdown") =>
        {
            parse_yaml_embedded_name_directive(action.text, &fields, rest)
        }
        _ => {
            let mut option_fields = Vec::with_capacity(fields.len() + 1);
            option_fields.push(action);
            option_fields.extend(fields);
            let (scope, delta) = parse_scope_and_options_checked(&option_fields, Scope::Next)?;
            Ok(Directive::Markdown { scope, delta })
        }
    }
}

fn parse_yaml_embedded_keyword_directive(
    fields: &[DirectiveToken],
    rest: &str,
) -> std::result::Result<Directive, String> {
    let mut name = None::<String>;
    for field in fields {
        if field.text == "scope=next" {
            continue;
        }
        if let Some(scope) = field.text.strip_prefix("scope=") {
            return Err(format!("fmt: embedded does not support scope={scope}"));
        }
        if name.is_some() {
            return Err(format!("invalid fmt directive: {rest}"));
        }
        name = Some(field.text.clone());
    }
    Ok(Directive::Embedded {
        name: name.unwrap_or_else(|| "default".to_owned()),
    })
}

fn parse_yaml_embedded_name_directive(
    name: String,
    fields: &[DirectiveToken],
    rest: &str,
) -> std::result::Result<Directive, String> {
    for field in fields {
        if field.text == "scope=next" {
            continue;
        }
        if let Some(scope) = field.text.strip_prefix("scope=") {
            return Err(format!("fmt: {name} does not support scope={scope}"));
        }
        return Err(format!("invalid fmt directive: {rest}"));
    }
    Ok(Directive::Embedded { name })
}

fn parse_yaml_skip_directive(fields: &[DirectiveToken]) -> std::result::Result<Directive, String> {
    let fields = field_texts(fields);
    match fields.as_slice() {
        [] | ["scope=next"] => Ok(Directive::Skip),
        ["file"] | ["scope=file"] => Ok(Directive::SkipFile),
        ["scope=from-here"] => Err("fmt: skip does not support scope=from-here".to_owned()),
        [field] if field.starts_with("scope=") => {
            Err(format!("fmt: skip does not support {}", field))
        }
        _ => Err("invalid fmt: skip directive".to_owned()),
    }
}

fn parse_yaml_off_directive(fields: &[DirectiveToken]) -> std::result::Result<Directive, String> {
    let fields = field_texts(fields);
    match fields.as_slice() {
        [] | ["scope=from-here"] => Ok(Directive::Off),
        ["scope=next"] => Err("fmt: off does not support scope=next".to_owned()),
        ["scope=file"] => Err("fmt: off does not support scope=file".to_owned()),
        [field] if field.starts_with("scope=") => {
            Err(format!("fmt: off does not support {}", field))
        }
        _ => Err("invalid fmt: off directive".to_owned()),
    }
}

fn parse_yaml_on_directive(fields: &[DirectiveToken]) -> std::result::Result<Directive, String> {
    if fields.is_empty() {
        Ok(Directive::On)
    } else {
        Err("fmt: on does not support explicit scope".to_owned())
    }
}

fn parse_yaml_template_directive(
    fields: &[DirectiveToken],
) -> std::result::Result<Directive, String> {
    let mut scope = Scope::Next;
    let mut quoted = Vec::new();
    for field in fields {
        if !field.quoted
            && let Some(value) = field.text.strip_prefix("scope=")
        {
            scope = parse_scope_checked(value)?;
            continue;
        }
        let Some(value) = quoted_field(field) else {
            return Err(
                "fmt: template.delimiters requires exactly two quoted delimiter strings".to_owned(),
            );
        };
        quoted.push(value);
    }
    if quoted.len() != 2 {
        return Err(
            "fmt: template.delimiters requires exactly two quoted delimiter strings".to_owned(),
        );
    }
    if quoted.iter().any(|value| value.is_empty()) {
        return Err("fmt: template.delimiters values must not be empty".to_owned());
    }
    Ok(Directive::Template {
        scope,
        delimiter: TemplateDelimiter {
            open: quoted[0].clone(),
            close: quoted[1].clone(),
        },
    })
}

fn parse_yaml_table_directive(
    fields: &[DirectiveToken],
    compact: bool,
) -> std::result::Result<Directive, String> {
    let mut compact = compact;
    for field in fields {
        match field.text.as_str() {
            "compact" | "compact=true" => compact = true,
            "scope=next" => {}
            "scope=from-here" => {
                return Err("fmt: table does not support scope=from-here".to_owned());
            }
            "scope=file" => return Err("fmt: table does not support scope=file".to_owned()),
            field if field.starts_with("scope=") => {
                return Err(format!("fmt: table does not support {field}"));
            }
            _ => return Err("invalid fmt: table directive".to_owned()),
        }
    }
    Ok(Directive::Table { compact })
}

fn parse_yaml_compact_directive(
    fields: &[DirectiveToken],
    mut compact: bool,
) -> std::result::Result<Directive, String> {
    let mut scope = Scope::Next;
    for field in fields {
        let field = field.text.as_str();
        if let Some(value) = field.strip_prefix("scope=") {
            scope = parse_scope_checked(value)?;
        } else if let Some(value) = field.strip_prefix("compact=") {
            compact = parse_bool_option(value, "compact")?;
        } else {
            match field {
                "true" => compact = true,
                "false" => compact = false,
                _ => return Err("invalid fmt: compact directive".to_owned()),
            }
        }
    }
    Ok(Directive::Compact { scope, compact })
}

fn parse_scope_and_options_checked(
    fields: &[DirectiveToken],
    default_scope: Scope,
) -> std::result::Result<(Scope, DirectiveDelta), String> {
    let mut scope = default_scope;
    let mut delta = DirectiveDelta::default();
    let mut index = 0usize;
    while index < fields.len() {
        let field = fields[index].text.as_str();
        if let Some(value) = field.strip_prefix("scope=") {
            scope = parse_scope_checked(value)?;
        } else if let Some(value) = field.strip_prefix("wrap=") {
            match value {
                "none" => delta.markdown_wrap = Some(MarkdownWrap::None),
                "paragraph" => delta.markdown_wrap = Some(MarkdownWrap::Paragraph),
                "sentence" => delta.markdown_wrap = Some(MarkdownWrap::Sentence),
                value => {
                    let width = value.parse::<usize>().map_err(|_| WRAP_OPTION_ERROR)?;
                    if width == 0 {
                        return Err(WRAP_OPTION_ERROR.to_owned());
                    }
                    delta.markdown_wrap = Some(MarkdownWrap::Column);
                    delta.markdown_wrap_at_column = Some(width);
                }
            }
        } else if let Some(value) = field.strip_prefix("canonical=") {
            delta.markdown_canonical = Some(parse_bool_option(value, "canonical")?);
        } else if field == "canonical" {
            delta.markdown_canonical = Some(true);
        } else if let Some(value) = field.strip_prefix("footnotes=") {
            delta.markdown_format_footnotes = Some(match value {
                "wrap" | "format" | "true" | "yes" | "1" => true,
                "preserve" | "none" | "false" | "no" | "0" => false,
                _ => return Err("invalid fmt: footnotes option".to_owned()),
            });
        } else if field == "markdown" {
        } else if field == "template.delimiters" {
            let Some(open) = fields.get(index + 1).and_then(quoted_field) else {
                return Err(
                    "fmt: template.delimiters requires exactly two quoted delimiter strings"
                        .to_owned(),
                );
            };
            let Some(close) = fields.get(index + 2).and_then(quoted_field) else {
                return Err(
                    "fmt: template.delimiters requires exactly two quoted delimiter strings"
                        .to_owned(),
                );
            };
            if open.is_empty() || close.is_empty() {
                return Err("fmt: template.delimiters values must not be empty".to_owned());
            }
            delta
                .add_template_delimiters
                .push(TemplateDelimiter { open, close });
            index += 2;
        } else {
            return Err(format!("invalid fmt directive option: {field}"));
        }
        index += 1;
    }
    Ok((scope, delta))
}

fn parse_scope_checked(value: &str) -> std::result::Result<Scope, String> {
    match value {
        "next" => Ok(Scope::Next),
        "from-here" => Ok(Scope::FromHere),
        "file" => Ok(Scope::File),
        _ => Err(format!("invalid fmt directive scope: {value}")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectiveToken {
    text: String,
    quoted: bool,
}

fn parse_directive_tokens(rest: &str) -> std::result::Result<Vec<DirectiveToken>, String> {
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    while cursor < rest.len() {
        while cursor < rest.len() {
            let ch = rest[cursor..].chars().next().expect("cursor is in bounds");
            if !ch.is_whitespace() {
                break;
            }
            cursor += ch.len_utf8();
        }
        if cursor >= rest.len() {
            break;
        }

        let ch = rest[cursor..].chars().next().expect("cursor is in bounds");
        if ch == '"' {
            let (text, end) = parse_quoted_directive_token(rest, cursor)?;
            cursor = end;
            if cursor < rest.len() {
                let next = rest[cursor..].chars().next().expect("cursor is in bounds");
                if !next.is_whitespace() {
                    return Err("invalid fmt directive quoting".to_owned());
                }
            }
            tokens.push(DirectiveToken { text, quoted: true });
            continue;
        }

        let start = cursor;
        while cursor < rest.len() {
            let ch = rest[cursor..].chars().next().expect("cursor is in bounds");
            if ch.is_whitespace() {
                break;
            }
            if ch == '"' {
                return Err("invalid fmt directive quoting".to_owned());
            }
            cursor += ch.len_utf8();
        }
        tokens.push(DirectiveToken {
            text: rest[start..cursor].to_owned(),
            quoted: false,
        });
    }
    Ok(tokens)
}

fn parse_quoted_directive_token(
    rest: &str,
    start: usize,
) -> std::result::Result<(String, usize), String> {
    let mut cursor = start + '"'.len_utf8();
    let mut text = String::new();
    while cursor < rest.len() {
        let ch = rest[cursor..].chars().next().expect("cursor is in bounds");
        cursor += ch.len_utf8();
        match ch {
            '"' => return Ok((text, cursor)),
            '\\' => {
                if cursor >= rest.len() {
                    return Err("invalid fmt directive quoting".to_owned());
                }
                let escaped = rest[cursor..].chars().next().expect("cursor is in bounds");
                cursor += escaped.len_utf8();
                text.push(match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => other,
                });
            }
            _ => text.push(ch),
        }
    }
    Err("invalid fmt directive quoting".to_owned())
}

fn field_texts(fields: &[DirectiveToken]) -> Vec<&str> {
    fields.iter().map(|field| field.text.as_str()).collect()
}

fn reject_markdown_without_options_for_broad_scope(
    scope: Scope,
    delta: &DirectiveDelta,
) -> std::result::Result<(), String> {
    if scope == Scope::Next || markdown_delta_has_actual_options(delta) {
        return Ok(());
    }
    Err(format!(
        "fmt: markdown with scope={} requires at least one option",
        scope_name(scope)
    ))
}

fn markdown_delta_has_actual_options(delta: &DirectiveDelta) -> bool {
    delta.yaml_compact.is_some()
        || delta.markdown_wrap.is_some()
        || delta.markdown_wrap_at_column.is_some()
        || delta.markdown_canonical.is_some()
        || delta.markdown_format_footnotes.is_some()
        || delta.table_compact.is_some()
        || !delta.add_template_delimiters.is_empty()
        || delta.embedded_formatter.is_some()
}

fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::Next => "next",
        Scope::FromHere => "from-here",
        Scope::File => "file",
    }
}

fn parse_bool_option(value: &str, option: &str) -> std::result::Result<bool, String> {
    match value {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(format!("invalid fmt: {option} option")),
    }
}

fn quoted_field(field: &DirectiveToken) -> Option<String> {
    field.quoted.then(|| field.text.clone())
}

fn parse_delimiter(fields: &[DirectiveToken]) -> Option<TemplateDelimiter> {
    let mut open = None;
    let mut close = None;
    let quoted = fields.iter().filter_map(quoted_field).collect::<Vec<_>>();
    if quoted.len() == 2 && !quoted[0].is_empty() && !quoted[1].is_empty() {
        return Some(TemplateDelimiter {
            open: quoted[0].clone(),
            close: quoted[1].clone(),
        });
    }
    for field in fields {
        if let Some(value) = field.text.strip_prefix("open=") {
            open = Some(value.trim_matches('"').to_owned());
        }
        if let Some(value) = field.text.strip_prefix("close=") {
            close = Some(value.trim_matches('"').to_owned());
        }
    }
    Some(TemplateDelimiter {
        open: open?,
        close: close?,
    })
}
