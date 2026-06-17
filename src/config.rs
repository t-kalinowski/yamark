use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::core::directives::TemplateDelimiter;
use crate::diagnostic::{Diagnostic, Result, YamarkError};
use crate::plugins::{ExternalFormatter, ExternalFormatterMode, builtin_formatter};

#[derive(Debug, Clone)]
pub struct Config {
    pub format: FormatConfig,
    pub template_delimiters: Vec<TemplateDelimiter>,
    pub embedded_markdown_template_delimiters: Vec<TemplateDelimiter>,
    pub markdown_standalone_template_delimiters: Vec<TemplateDelimiter>,
    pub embedded_formatters: BTreeMap<String, ExternalFormatter>,
    pub path_configs: Vec<PathConfig>,
    pub config_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    pub compact: Option<bool>,
    pub markdown_horizontal_rule: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct PathConfig {
    pub path: PathBuf,
    pub template_replace: Option<Vec<TemplateDelimiter>>,
    pub template_add: Vec<TemplateDelimiter>,
    pub embedded_markdown_template_replace: Option<Vec<TemplateDelimiter>>,
    pub embedded_markdown_template_add: Vec<TemplateDelimiter>,
}

impl Default for Config {
    fn default() -> Self {
        let template_delimiters = default_template_delimiters();
        Self {
            format: FormatConfig::default(),
            embedded_markdown_template_delimiters: template_delimiters.clone(),
            template_delimiters,
            markdown_standalone_template_delimiters: Vec::new(),
            embedded_formatters: BTreeMap::new(),
            path_configs: Vec::new(),
            config_dir: None,
        }
    }
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self> {
        let input = fs::read_to_string(path).map_err(|err| {
            YamarkError::from(
                Diagnostic::error(format!("failed to read config: {err}")).with_path(path),
            )
        })?;
        let mut config = Self::from_toml_str(&input).map_err(|err| err.with_path(path))?;
        let absolute = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        config.config_dir = absolute.parent().map(Path::to_path_buf);
        Ok(config)
    }

    pub fn from_toml_str(input: &str) -> Result<Self> {
        let table = input
            .parse::<toml::Table>()
            .map_err(|err| YamarkError::new(format!("invalid yamark.toml: {err}")))?;
        let mut config = Config::default();

        for key in table.keys() {
            if !matches!(key.as_str(), "template" | "embedded" | "format" | "paths") {
                return Err(YamarkError::new(format!("unknown config key: {key}")));
            }
        }

        if let Some(template) = table.get("template") {
            apply_template_layer(template, &mut config.template_delimiters, "template")?;
            apply_template_layer(
                template,
                &mut config.embedded_markdown_template_delimiters,
                "template",
            )?;
        }
        if let Some(format) = table.get("format") {
            apply_format_layer(format, &mut config.format)?;
        }
        if let Some(embedded_value) = table.get("embedded") {
            let embedded = embedded_value
                .as_table()
                .ok_or_else(|| YamarkError::new("embedded config must be a table"))?;
            for (name, value) in embedded {
                let trimmed = name.trim();
                if trimmed.is_empty() || trimmed != name {
                    return Err(YamarkError::new(
                        "embedded formatter names must be non-empty and trimmed",
                    ));
                }
                if reserved_embedded_name(name) {
                    return Err(YamarkError::new(format!(
                        "reserved fmt directive name: {name}"
                    )));
                }
                let formatter = parse_embedded_formatter(value)?;
                config.embedded_formatters.insert(name.clone(), formatter);
            }
        }
        if let Some(paths) = table.get("paths") {
            config.path_configs = parse_path_configs(paths)?;
        }
        Ok(config)
    }

    pub fn for_formatted_path(&self, path: &Path) -> Self {
        if self.path_configs.is_empty() {
            return self.clone();
        }
        let mut config = self.clone();
        let relative = self.relative_formatted_path(path);
        let mut matching = self
            .path_configs
            .iter()
            .filter(|path_config| relative.starts_with(&path_config.path))
            .collect::<Vec<_>>();
        matching.sort_by_key(|path_config| path_config.path.components().count());
        for path_config in matching {
            if let Some(replace) = &path_config.template_replace {
                config.template_delimiters = replace.clone();
            }
            config
                .template_delimiters
                .extend(path_config.template_add.clone());
            if let Some(replace) = &path_config.embedded_markdown_template_replace {
                config.embedded_markdown_template_delimiters = replace.clone();
            }
            config
                .embedded_markdown_template_delimiters
                .extend(path_config.embedded_markdown_template_add.clone());
        }
        config
    }

    fn relative_formatted_path(&self, path: &Path) -> PathBuf {
        let base = self.config_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let path = absolute_normalized_path(path);
        let base = absolute_normalized_path(base);
        path.strip_prefix(&base).unwrap_or(&path).to_path_buf()
    }
}

fn absolute_normalized_path(path: &Path) -> PathBuf {
    if let Ok(path) = fs::canonicalize(path) {
        return path;
    }
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    normalize_path_components(&path)
}

fn normalize_path_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn default_template_delimiters() -> Vec<TemplateDelimiter> {
    vec![
        TemplateDelimiter {
            open: "{{".to_owned(),
            close: "}}".to_owned(),
        },
        TemplateDelimiter {
            open: "{%".to_owned(),
            close: "%}".to_owned(),
        },
        TemplateDelimiter {
            open: "{#".to_owned(),
            close: "#}".to_owned(),
        },
        TemplateDelimiter {
            open: "<%".to_owned(),
            close: "%>".to_owned(),
        },
    ]
}

pub fn discover_config_path(path: &Path) -> Option<PathBuf> {
    let mut dir = path.parent().unwrap_or_else(|| Path::new("."));
    loop {
        let candidate = dir.join("yamark.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        let parent = dir.parent()?;
        dir = parent;
    }
}

fn apply_template_layer(
    value: &toml::Value,
    delimiters: &mut Vec<TemplateDelimiter>,
    context: &str,
) -> Result<()> {
    let table = value
        .as_table()
        .ok_or_else(|| YamarkError::new(format!("{context} must be a table")))?;
    for key in table.keys() {
        if !matches!(key.as_str(), "replace_delimiters" | "add_delimiters") {
            return Err(YamarkError::new(format!(
                "unknown template config key: {context}.{key}"
            )));
        }
    }
    if let Some(replace) = table.get("replace_delimiters") {
        *delimiters = parse_delimiters(replace, &format!("{context}.replace_delimiters"))?;
    }
    if let Some(add) = table.get("add_delimiters") {
        delimiters.extend(parse_delimiters(add, &format!("{context}.add_delimiters"))?);
    }
    Ok(())
}

fn apply_format_layer(value: &toml::Value, format: &mut FormatConfig) -> Result<()> {
    let table = value
        .as_table()
        .ok_or_else(|| YamarkError::new("format config must be a table"))?;
    for key in table.keys() {
        if !matches!(key.as_str(), "compact" | "markdown_horizontal_rule") {
            return Err(YamarkError::new(format!(
                "unknown format config key: format.{key}"
            )));
        }
    }
    if let Some(value) = table.get("compact") {
        format.compact = Some(
            value
                .as_bool()
                .ok_or_else(|| YamarkError::new("format.compact must be a boolean"))?,
        );
    }
    if let Some(value) = table.get("markdown_horizontal_rule") {
        let marker = value
            .as_str()
            .ok_or_else(|| YamarkError::new("format.markdown_horizontal_rule must be a string"))?;
        format.markdown_horizontal_rule = Some(match marker {
            "---" => "---",
            "***" => "***",
            _ => {
                return Err(YamarkError::new(
                    "format.markdown_horizontal_rule must be \"---\" or \"***\"",
                ));
            }
        });
    }
    Ok(())
}

fn parse_delimiters(value: &toml::Value, context: &str) -> Result<Vec<TemplateDelimiter>> {
    let entries = value
        .as_array()
        .ok_or_else(|| YamarkError::new(format!("{context} must be an array")))?;
    let mut delimiters = Vec::with_capacity(entries.len());
    for entry in entries {
        let table = entry
            .as_table()
            .ok_or_else(|| YamarkError::new(format!("{context} entries must be tables")))?;
        for key in table.keys() {
            if !matches!(key.as_str(), "open" | "close") {
                return Err(YamarkError::new(format!(
                    "unknown template delimiter key: {context}.{key}"
                )));
            }
        }
        let open = table
            .get("open")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| YamarkError::new(format!("{context}.open is required")))?;
        let close = table
            .get("close")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| YamarkError::new(format!("{context}.close is required")))?;
        if open.is_empty() {
            return Err(YamarkError::new(format!(
                "{context}.open must not be empty"
            )));
        }
        if close.is_empty() {
            return Err(YamarkError::new(format!(
                "{context}.close must not be empty"
            )));
        }
        delimiters.push(TemplateDelimiter {
            open: open.to_owned(),
            close: close.to_owned(),
        });
    }
    Ok(delimiters)
}

fn parse_path_configs(value: &toml::Value) -> Result<Vec<PathConfig>> {
    let table = value
        .as_table()
        .ok_or_else(|| YamarkError::new("paths config must be a table"))?;
    let mut configs = Vec::new();
    for (path, value) in table {
        let path_buf = validate_path_config_key(path)?;
        let context = format!("paths.{}", path.replace(['/', '\\'], "."));
        let table = value
            .as_table()
            .ok_or_else(|| YamarkError::new(format!("{context} must be a table")))?;
        for key in table.keys() {
            if !matches!(key.as_str(), "template" | "embedded_markdown") {
                return Err(YamarkError::new(format!(
                    "unknown path config key: {context}.{key}"
                )));
            }
        }
        let mut path_config = PathConfig {
            path: path_buf,
            template_replace: None,
            template_add: Vec::new(),
            embedded_markdown_template_replace: None,
            embedded_markdown_template_add: Vec::new(),
        };
        if let Some(template) = table.get("template") {
            let template_context = format!("{context}.template");
            let template = template
                .as_table()
                .ok_or_else(|| YamarkError::new(format!("{template_context} must be a table")))?;
            for key in template.keys() {
                if !matches!(key.as_str(), "replace_delimiters" | "add_delimiters") {
                    return Err(YamarkError::new(format!(
                        "unknown path template config key: {template_context}.{key}"
                    )));
                }
            }
            if let Some(replace) = template.get("replace_delimiters") {
                path_config.template_replace = Some(parse_delimiters(
                    replace,
                    &format!("{template_context}.replace_delimiters"),
                )?);
            }
            if let Some(add) = template.get("add_delimiters") {
                path_config.template_add.extend(parse_delimiters(
                    add,
                    &format!("{template_context}.add_delimiters"),
                )?);
            }
        }
        if let Some(embedded_markdown) = table.get("embedded_markdown") {
            let embedded_template = parse_embedded_markdown_path_config(
                embedded_markdown,
                &format!("{context}.embedded_markdown"),
            )?;
            path_config.embedded_markdown_template_replace = embedded_template.replace;
            path_config
                .embedded_markdown_template_add
                .extend(embedded_template.add);
        }
        configs.push(path_config);
    }
    Ok(configs)
}

#[derive(Debug, Clone, Default)]
struct EmbeddedMarkdownTemplateConfig {
    replace: Option<Vec<TemplateDelimiter>>,
    add: Vec<TemplateDelimiter>,
}

fn parse_embedded_markdown_path_config(
    value: &toml::Value,
    context: &str,
) -> Result<EmbeddedMarkdownTemplateConfig> {
    let table = value
        .as_table()
        .ok_or_else(|| YamarkError::new(format!("{context} must be a table")))?;
    for key in table.keys() {
        if key != "template" {
            return Err(YamarkError::new(format!(
                "unknown path config key: {context}.{key}"
            )));
        }
    }
    let mut parsed = EmbeddedMarkdownTemplateConfig::default();
    if let Some(template) = table.get("template") {
        let template_context = format!("{context}.template");
        let template = template
            .as_table()
            .ok_or_else(|| YamarkError::new(format!("{template_context} must be a table")))?;
        for key in template.keys() {
            if !matches!(key.as_str(), "replace_delimiters" | "add_delimiters") {
                return Err(YamarkError::new(format!(
                    "unknown path template config key: {template_context}.{key}"
                )));
            }
        }
        if let Some(replace) = template.get("replace_delimiters") {
            parsed.replace = Some(parse_delimiters(
                replace,
                &format!("{template_context}.replace_delimiters"),
            )?);
        }
        if let Some(add) = template.get("add_delimiters") {
            parsed.add.extend(parse_delimiters(
                add,
                &format!("{template_context}.add_delimiters"),
            )?);
        }
    }
    Ok(parsed)
}

fn validate_path_config_key(path: &str) -> Result<PathBuf> {
    if path.is_empty() {
        return Err(YamarkError::new("path config keys must not be empty"));
    }
    let path = PathBuf::from(path);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(YamarkError::new(
            "path config keys must be relative and must not contain ..",
        ));
    }
    Ok(path)
}

fn parse_embedded_formatter(value: &toml::Value) -> Result<ExternalFormatter> {
    let table = value
        .as_table()
        .ok_or_else(|| YamarkError::new("embedded formatter entries must be tables"))?;
    let Some(formatter) = table.get("formatter") else {
        return Err(YamarkError::new(
            "embedded formatter entries must contain formatter",
        ));
    };
    for key in table.keys() {
        if key != "formatter" {
            return Err(YamarkError::new(format!(
                "unknown embedded formatter entry key: {key}"
            )));
        }
    }
    if let Some(name) = formatter.as_str() {
        return builtin_formatter(name).ok_or_else(|| {
            YamarkError::new(format!("unknown embedded formatter shorthand: {name}"))
        });
    }
    let table = formatter
        .as_table()
        .ok_or_else(|| YamarkError::new("embedded formatter must be a string or table"))?;
    for key in table.keys() {
        if !matches!(key.as_str(), "command" | "path_suffix") {
            return Err(YamarkError::new(format!(
                "unknown embedded formatter key: {key}"
            )));
        }
    }
    let command = table
        .get("command")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| YamarkError::new("missing embedded formatter command"))?
        .iter()
        .map(|item| {
            item.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                YamarkError::new("embedded formatter command entries must be strings")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if command.is_empty() {
        return Err(YamarkError::new(
            "embedded formatter command must not be empty",
        ));
    }
    if !command.iter().any(|arg| arg == "{path}") {
        return Err(YamarkError::new(
            "embedded formatter command must contain {path} as an argv item",
        ));
    }
    if command
        .iter()
        .any(|arg| arg != "{path}" && arg.contains("{path}"))
    {
        return Err(YamarkError::new("{path} must be a complete argv item"));
    }
    let path_suffix = table
        .get("path_suffix")
        .ok_or_else(|| YamarkError::new("missing embedded formatter path_suffix"))?
        .as_str()
        .ok_or_else(|| YamarkError::new("embedded formatter path_suffix must be a string"))?
        .to_owned();
    if path_suffix.is_empty() {
        return Err(YamarkError::new(
            "embedded formatter path_suffix must not be empty",
        ));
    }
    Ok(ExternalFormatter {
        command,
        path_suffix,
        mode: ExternalFormatterMode::Raw,
    })
}

fn reserved_embedded_name(name: &str) -> bool {
    matches!(name, "skip" | "skip file" | "off" | "on" | "table")
}
