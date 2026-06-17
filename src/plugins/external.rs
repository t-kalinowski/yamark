use std::io::{ErrorKind, Write};
use std::process::{Child, Command, Output, Stdio};
use std::thread;

use crate::diagnostic::{Result, YamarkError};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalFormatter {
    pub command: Vec<String>,
    pub path_suffix: String,
    pub mode: ExternalFormatterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalFormatterMode {
    Raw,
    PythonNotebookCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MissingExecutablePolicy {
    Error,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExternalFormatterOutput {
    Formatted(String),
    FormatterFailed(String),
    MissingExecutable,
}

impl ExternalFormatter {
    pub(crate) fn run(
        &self,
        input: &str,
        virtual_path: &str,
        missing_executable: MissingExecutablePolicy,
    ) -> Result<ExternalFormatterOutput> {
        if self.command.is_empty() {
            return Ok(ExternalFormatterOutput::MissingExecutable);
        }
        let mut argv = self.command.iter();
        let program = argv.next().expect("checked nonempty command");
        let args = argv
            .map(|arg| {
                if arg == "{path}" {
                    virtual_path.to_owned()
                } else {
                    arg.clone()
                }
            })
            .collect::<Vec<_>>();
        let process_input = self.mode.process_input(input)?;
        let child = Command::new(program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let child = match child {
            Ok(child) => child,
            Err(err)
                if missing_executable == MissingExecutablePolicy::Skip
                    && err.kind() == ErrorKind::NotFound =>
            {
                return Ok(ExternalFormatterOutput::MissingExecutable);
            }
            Err(err) => {
                return Err(YamarkError::new(format!(
                    "failed to run embedded formatter: {err}"
                )));
            }
        };
        let (output, stdin_result) = wait_with_input(child, process_input)?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            if missing_executable == MissingExecutablePolicy::Skip {
                if !message.is_empty() {
                    return Ok(ExternalFormatterOutput::FormatterFailed(message));
                }
                let status = output
                    .status
                    .code()
                    .map(|code| format!("exit status {code}"))
                    .unwrap_or_else(|| output.status.to_string());
                return Ok(ExternalFormatterOutput::FormatterFailed(format!(
                    "command `{}` exited with {status} for {virtual_path}",
                    display_command(program, &args)
                )));
            }
            if !message.is_empty() {
                return Err(YamarkError::new(message));
            }
            let status = output
                .status
                .code()
                .map(|code| format!("exit status {code}"))
                .unwrap_or_else(|| output.status.to_string());
            return Err(YamarkError::new(format!(
                "embedded formatter failed: command `{}` exited with {status} for {virtual_path}",
                display_command(program, &args)
            )));
        }
        stdin_result.map_err(|err| {
            YamarkError::new(format!("failed to write embedded formatter stdin: {err}"))
        })?;
        if !output.stderr.is_empty() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(YamarkError::new(if message.is_empty() {
                "embedded formatter wrote to stderr".to_owned()
            } else {
                format!("embedded formatter wrote to stderr: {message}")
            }));
        }
        let formatted = String::from_utf8(output.stdout).map_err(|err| {
            YamarkError::new(format!("embedded formatter emitted invalid UTF-8: {err}"))
        })?;
        Ok(ExternalFormatterOutput::Formatted(
            self.mode.process_output(&formatted)?,
        ))
    }
}

fn wait_with_input(mut child: Child, input: String) -> Result<(Output, std::io::Result<()>)> {
    let stdin = child.stdin.take();
    let stdin_writer = thread::spawn(move || {
        if let Some(mut stdin) = stdin {
            stdin.write_all(input.as_bytes())?;
        }
        Ok(())
    });
    let output = child.wait_with_output().map_err(|err| {
        YamarkError::new(format!("failed to read embedded formatter output: {err}"))
    })?;
    let stdin_result = stdin_writer
        .join()
        .map_err(|_| YamarkError::new("embedded formatter stdin writer panicked"))?;
    Ok((output, stdin_result))
}

impl ExternalFormatterMode {
    fn process_input(self, input: &str) -> Result<String> {
        match self {
            Self::Raw => Ok(input.to_owned()),
            Self::PythonNotebookCell => serde_json::to_string(&json!({
                "cells": [{
                    "cell_type": "code",
                    "source": input,
                    "metadata": {},
                    "outputs": [],
                }],
                "metadata": {},
                "nbformat": 4,
                "nbformat_minor": 5,
            }))
            .map_err(|err| {
                YamarkError::new(format!(
                    "failed to encode embedded formatter notebook: {err}"
                ))
            }),
        }
    }

    fn process_output(self, output: &str) -> Result<String> {
        match self {
            Self::Raw => Ok(output.to_owned()),
            Self::PythonNotebookCell => extract_notebook_cell_source(output),
        }
    }
}

fn extract_notebook_cell_source(output: &str) -> Result<String> {
    let notebook: Value = serde_json::from_str(output).map_err(|err| {
        YamarkError::new(format!(
            "embedded formatter emitted invalid notebook JSON: {err}"
        ))
    })?;
    let cells = notebook
        .get("cells")
        .and_then(Value::as_array)
        .ok_or_else(|| YamarkError::new("embedded formatter notebook output must contain cells"))?;
    if cells.len() != 1 {
        return Err(YamarkError::new(
            "embedded formatter notebook output must contain exactly one cell",
        ));
    }
    let source = cells[0].get("source").ok_or_else(|| {
        YamarkError::new("embedded formatter notebook output cell must contain source")
    })?;
    match source {
        Value::String(source) => Ok(source.clone()),
        Value::Array(lines) => {
            let mut source = String::new();
            for line in lines {
                source.push_str(line.as_str().ok_or_else(|| {
                    YamarkError::new(
                        "embedded formatter notebook output source lines must be strings",
                    )
                })?);
            }
            Ok(source)
        }
        _ => Err(YamarkError::new(
            "embedded formatter notebook output source must be a string or string array",
        )),
    }
}

fn display_command(program: &str, args: &[String]) -> String {
    std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}
