//! Git subprocess invocation and local configuration access.

use std::ffi::OsString;
use std::path::Path;
use std::process::Command as ProcessCommand;

pub(super) fn run_git_with_paths(
    repo_root: &Path,
    prefix: &[&str],
    paths: &[String],
) -> Result<(), String> {
    let mut args = prefix
        .iter()
        .map(|arg| OsString::from(*arg))
        .collect::<Vec<_>>();
    args.extend(paths.iter().map(OsString::from));
    run_git_os(repo_root, &args)
}

pub(super) fn run_git_os(repo_root: &Path, args: &[OsString]) -> Result<(), String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format_git_error("git", output))
    }
}

pub(super) fn git_stdout_os(repo_root: &Path, args: &[OsString]) -> Result<String, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if !output.status.success() {
        return Err(format_git_error("git", output));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("git output was not UTF-8: {err}"))
}

pub(super) fn local_git_config_count(repo_root: &Path, key: &str) -> Result<usize, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("config")
        .arg("--local")
        .arg("--get-all")
        .arg(key)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if !output.status.success() {
        if output.status.code() == Some(1) && output.stderr.is_empty() {
            return Ok(0);
        }
        return Err(format_git_error("git config", output));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| format!("git config output was not UTF-8: {err}"))
        .map(|output| output.lines().count())
}

pub(super) fn run_git_config(repo: &Path, key: &str, value: &str) -> Result<(), String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg(key)
        .arg(value)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format_git_error("git config", output))
}

pub(super) fn unset_git_config_key(repo: &Path, key: &str) -> Result<usize, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg("--get-all")
        .arg(key)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if !output.status.success() {
        if output.status.code() == Some(1) && output.stderr.is_empty() {
            return Ok(0);
        }
        return Err(format_git_error("git config", output));
    }
    let count = String::from_utf8(output.stdout)
        .map_err(|err| format!("git config output was not UTF-8: {err}"))?
        .lines()
        .count();
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg("--unset-all")
        .arg(key)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if output.status.success() {
        return Ok(count);
    }
    Err(format_git_error("git config", output))
}

pub(super) fn git_stdout<const N: usize>(repo: &Path, args: [&str; N]) -> Result<String, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if !output.status.success() {
        return Err(format_git_error("git", output));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("git output was not UTF-8: {err}"))
}

fn format_git_error(command: &str, output: std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("{command} failed with status {}", output.status)
    } else {
        format!("{command} failed: {stderr}")
    }
}
