use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use crate::diagnostic::{Diagnostic, Result};
use crate::plugins::{
    ExternalFormatter, ExternalFormatterMode, ExternalFormatterOutput, MissingExecutablePolicy,
};

#[derive(Debug, Clone)]
pub struct PluginRegistry {
    external: BTreeMap<String, RegisteredFormatter>,
    source_path: Option<PathBuf>,
    diagnostics: Rc<RefCell<Vec<Diagnostic>>>,
}

#[derive(Debug, Clone)]
struct RegisteredFormatter {
    formatter: ExternalFormatter,
    missing_executable: MissingExecutablePolicy,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self {
            external: BTreeMap::new(),
            source_path: None,
            diagnostics: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl PluginRegistry {
    pub fn from_config(config: &crate::config::Config) -> Self {
        let mut registry = Self::default();
        for (name, formatter) in &config.embedded_formatters {
            registry.insert_configured(name.clone(), formatter.clone());
        }
        registry
    }

    pub fn with_source_path(mut self, path: &Path) -> Self {
        self.source_path = Some(path.to_path_buf());
        self
    }

    pub fn is_known_formatter(config: &crate::config::Config, name: &str) -> bool {
        config.embedded_formatters.contains_key(name) || default_formatter_name(name).is_some()
    }

    pub fn insert(&mut self, name: impl Into<String>, formatter: ExternalFormatter) {
        self.insert_configured(name, formatter);
    }

    pub fn run(&self, name: &str, input: &str, line: usize) -> Result<Option<String>> {
        if let Some(registered) = self.external.get(name) {
            return self.run_registered(name, registered, input, line);
        };
        let Some(default_name) = default_formatter_name(name) else {
            return Ok(None);
        };
        let formatter =
            builtin_formatter(default_name).expect("default aliases point at built-ins");
        let registered = RegisteredFormatter {
            formatter,
            missing_executable: MissingExecutablePolicy::Skip,
        };
        self.run_registered(name, &registered, input, line)
    }

    fn run_registered(
        &self,
        name: &str,
        registered: &RegisteredFormatter,
        input: &str,
        line: usize,
    ) -> Result<Option<String>> {
        let source_path = self
            .source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "yamark".to_owned());
        let path = format!(
            "{source_path}.embedded.{line}{}",
            registered.formatter.path_suffix
        );
        match registered
            .formatter
            .run(input, &path, registered.missing_executable)?
        {
            ExternalFormatterOutput::Formatted(output) => Ok(Some(output)),
            ExternalFormatterOutput::FormatterFailed(message) => {
                self.push_failed_optional_formatter_note(name, registered, line, &message);
                Ok(None)
            }
            ExternalFormatterOutput::MissingExecutable => {
                if registered.missing_executable == MissingExecutablePolicy::Skip {
                    self.push_missing_optional_formatter_note(name, registered, line);
                }
                Ok(None)
            }
        }
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.borrow().clone()
    }

    fn insert_configured(&mut self, name: impl Into<String>, formatter: ExternalFormatter) {
        let missing_executable = missing_executable_policy_for_configured_formatter(&formatter);
        self.external.insert(
            name.into(),
            RegisteredFormatter {
                formatter,
                missing_executable,
            },
        );
    }

    fn push_missing_optional_formatter_note(
        &self,
        name: &str,
        registered: &RegisteredFormatter,
        line: usize,
    ) {
        let executable = registered
            .formatter
            .command
            .first()
            .map_or(name, String::as_str);
        let mut diagnostic = Diagnostic::note(format!(
            "missing optional embedded formatter `{executable}`; preserved source"
        ))
        .at(line, 1);
        if let Some(path) = &self.source_path {
            diagnostic = diagnostic.with_path(path);
        }
        self.diagnostics.borrow_mut().push(diagnostic);
    }

    fn push_failed_optional_formatter_note(
        &self,
        name: &str,
        registered: &RegisteredFormatter,
        line: usize,
        message: &str,
    ) {
        let executable = registered
            .formatter
            .command
            .first()
            .map_or(name, String::as_str);
        let message = match formatter_failure_location(message) {
            Some((line, column)) => format!(
                "embedded formatter `{executable}` failed at formatter input {line}:{column}; left chunk unchanged"
            ),
            None => format!("embedded formatter `{executable}` failed; left chunk unchanged"),
        };
        let mut diagnostic = Diagnostic::note(message).at(line, 1);
        if let Some(path) = &self.source_path {
            diagnostic = diagnostic.with_path(path);
        }
        self.diagnostics.borrow_mut().push(diagnostic);
    }
}

fn formatter_failure_location(message: &str) -> Option<(usize, usize)> {
    parenthesized_location(message).or_else(|| colon_delimited_location(message))
}

fn parenthesized_location(message: &str) -> Option<(usize, usize)> {
    let bytes = message.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            let start = i + 1;
            if let Some((line, after_line)) = parse_number(bytes, start)
                && bytes.get(after_line) == Some(&b':')
                && let Some((column, after_column)) = parse_number(bytes, after_line + 1)
                && bytes.get(after_column) == Some(&b')')
            {
                return Some((line, column));
            }
        }
        i += 1;
    }
    None
}

fn colon_delimited_location(message: &str) -> Option<(usize, usize)> {
    let bytes = message.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':'
            && let Some((line, after_line)) = parse_number(bytes, i + 1)
            && bytes.get(after_line) == Some(&b':')
            && let Some((column, _)) = parse_number(bytes, after_line + 1)
        {
            return Some((line, column));
        }
        i += 1;
    }
    None
}

fn parse_number(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    if !bytes.get(start).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    let mut i = start;
    let mut value = 0usize;
    while let Some(byte) = bytes.get(i)
        && byte.is_ascii_digit()
    {
        value = value
            .checked_mul(10)?
            .checked_add(usize::from(byte - b'0'))?;
        i += 1;
    }
    (value > 0).then_some((value, i))
}

fn missing_executable_policy_for_configured_formatter(
    formatter: &ExternalFormatter,
) -> MissingExecutablePolicy {
    match formatter.command.first().map(String::as_str) {
        Some("air" | "mdformat" | "prettier" | "ruff") => MissingExecutablePolicy::Skip,
        _ => MissingExecutablePolicy::Error,
    }
}

pub(crate) fn builtin_formatter(name: &str) -> Option<ExternalFormatter> {
    match name {
        "ruff" => Some(ExternalFormatter {
            command: vec![
                "ruff".into(),
                "format".into(),
                "--stdin-filename".into(),
                "{path}".into(),
                "-".into(),
            ],
            path_suffix: ".ipynb".into(),
            mode: ExternalFormatterMode::PythonNotebookCell,
        }),
        "air" => Some(ExternalFormatter {
            command: vec![
                "air".into(),
                "format".into(),
                "--stdin-file-path".into(),
                "{path}".into(),
            ],
            path_suffix: ".R".into(),
            mode: ExternalFormatterMode::Raw,
        }),
        "mdformat" => Some(ExternalFormatter {
            command: vec!["mdformat".into(), "-".into()],
            path_suffix: ".md".into(),
            mode: ExternalFormatterMode::Raw,
        }),
        "prettier-json" => Some(prettier(".json")),
        "prettier-jsonc" => Some(prettier(".jsonc")),
        "prettier-json5" => Some(prettier(".json5")),
        "prettier-graphql" => Some(prettier(".graphql")),
        "prettier-css" => Some(prettier(".css")),
        "prettier-scss" => Some(prettier(".scss")),
        "prettier-less" => Some(prettier(".less")),
        "prettier-postcss" => Some(prettier(".pcss")),
        "prettier-html" => Some(prettier(".html")),
        "prettier-js" => Some(prettier(".js")),
        "prettier-jsx" => Some(prettier(".jsx")),
        "prettier-ts" => Some(prettier(".ts")),
        "prettier-tsx" => Some(prettier(".tsx")),
        _ => None,
    }
}

fn prettier(path_suffix: &str) -> ExternalFormatter {
    ExternalFormatter {
        command: vec![
            "prettier".into(),
            "--stdin-filepath".into(),
            "{path}".into(),
        ],
        path_suffix: path_suffix.into(),
        mode: ExternalFormatterMode::Raw,
    }
}

fn default_formatter_name(name: &str) -> Option<&'static str> {
    DEFAULT_FORMATTER_ALIASES
        .iter()
        .find_map(|(alias, formatter)| (*alias == name).then_some(*formatter))
}

const DEFAULT_FORMATTER_ALIASES: &[(&str, &str)] = &[
    ("ruff", "ruff"),
    ("air", "air"),
    ("mdformat", "mdformat"),
    ("prettier-json", "prettier-json"),
    ("prettier-jsonc", "prettier-jsonc"),
    ("prettier-json5", "prettier-json5"),
    ("prettier-graphql", "prettier-graphql"),
    ("prettier-css", "prettier-css"),
    ("prettier-scss", "prettier-scss"),
    ("prettier-less", "prettier-less"),
    ("prettier-postcss", "prettier-postcss"),
    ("prettier-html", "prettier-html"),
    ("prettier-js", "prettier-js"),
    ("prettier-jsx", "prettier-jsx"),
    ("prettier-ts", "prettier-ts"),
    ("prettier-tsx", "prettier-tsx"),
    ("python", "ruff"),
    ("r", "air"),
    ("json", "prettier-json"),
    ("jsonc", "prettier-jsonc"),
    ("json5", "prettier-json5"),
    ("graphql", "prettier-graphql"),
    ("gql", "prettier-graphql"),
    ("graphqls", "prettier-graphql"),
    ("css", "prettier-css"),
    ("scss", "prettier-scss"),
    ("less", "prettier-less"),
    ("postcss", "prettier-postcss"),
    ("pcss", "prettier-postcss"),
    ("html", "prettier-html"),
    ("js", "prettier-js"),
    ("javascript", "prettier-js"),
    ("jsx", "prettier-jsx"),
    ("ts", "prettier-ts"),
    ("typescript", "prettier-ts"),
    ("tsx", "prettier-tsx"),
];
