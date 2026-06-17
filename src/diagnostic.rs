use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub path: Option<PathBuf>,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            path: None,
            line: 1,
            column: 1,
            message: message.into(),
        }
    }

    pub fn note(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Note,
            path: None,
            line: 1,
            column: 1,
            message: message.into(),
        }
    }

    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.line = line.max(1);
        self.column = column.max(1);
        self
    }

    pub fn render(&self) -> String {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Note => "note",
        };
        match &self.path {
            Some(path) => format!(
                "{}:{}:{}: {}: {}",
                path.display(),
                self.line,
                self.column,
                severity,
                self.message
            ),
            None => format!(
                "{}:{}: {}: {}",
                self.line, self.column, severity, self.message
            ),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{}", diagnostic.render())]
pub struct YamarkError {
    pub diagnostic: Diagnostic,
}

impl YamarkError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            diagnostic: Diagnostic::error(message),
        }
    }

    pub fn at(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            diagnostic: Diagnostic::error(message).at(line, column),
        }
    }

    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.diagnostic = self.diagnostic.with_path(path);
        self
    }
}

impl From<Diagnostic> for YamarkError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self { diagnostic }
    }
}

pub type Result<T> = std::result::Result<T, YamarkError>;
