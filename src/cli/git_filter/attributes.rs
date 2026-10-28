//! Git attribute patterns and local/shared attribute file updates.

use std::path::{Path, PathBuf};
use std::{env, fs, io};

pub(super) fn count_yamark_filter_attribute_lines(path: &Path) -> Result<usize, String> {
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => {
            return Err(format!(
                "{}: failed to read attributes: {err}",
                path.display()
            ));
        }
    };
    Ok(existing
        .lines()
        .filter(|line| is_yamark_filter_attribute_setup_line(line))
        .count())
}

pub(super) fn line_has_yamark_filter_attribute(line: &str) -> bool {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return false;
    }
    line.split_whitespace()
        .skip(1)
        .any(|field| field == "filter=yamark-md")
}

pub(super) fn git_filter_attribute_patterns(
    repo_root: &Path,
    paths: Vec<PathBuf>,
) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Ok(vec![
            "*.md filter=yamark-md".to_owned(),
            "*.qmd filter=yamark-md".to_owned(),
            "*.Rmd filter=yamark-md".to_owned(),
            "*.rmd filter=yamark-md".to_owned(),
        ]);
    }

    paths
        .into_iter()
        .map(|path| {
            if crate::core::document::FileKind::for_path(&path)
                != crate::core::document::FileKind::Markdown
            {
                return Err(format!("{}: unsupported Git filter path", path.display()));
            }
            let relative = repo_relative_existing_path(repo_root, &path)?;
            let pattern = git_attribute_path_pattern(&relative)?;
            Ok(format!("{pattern} filter=yamark-md"))
        })
        .collect()
}

fn repo_relative_existing_path(repo_root: &Path, path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|err| format!("failed to read current directory: {err}"))?
            .join(path)
    };
    let absolute = fs::canonicalize(&absolute)
        .map_err(|err| format!("{}: failed to resolve path: {err}", path.display()))?;
    let repo_root = fs::canonicalize(repo_root).map_err(|err| {
        format!(
            "{}: failed to resolve repository root: {err}",
            repo_root.display()
        )
    })?;
    absolute
        .strip_prefix(&repo_root)
        .map(Path::to_path_buf)
        .map_err(|_| {
            format!(
                "{}: path is outside {}",
                path.display(),
                repo_root.display()
            )
        })
}

fn git_attribute_path_pattern(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for part in path {
        let Some(part) = part.to_str() else {
            return Err(format!("{}: path is not valid UTF-8", path.display()));
        };
        if part.chars().any(char::is_whitespace) {
            return Err(format!(
                "{}: path contains whitespace; add this Git attribute manually",
                path.display()
            ));
        }
        parts.push(part);
    }
    if parts.is_empty() {
        return Err("cannot configure a Git filter for the repository root".to_owned());
    }
    Ok(parts.join("/"))
}

pub(super) fn append_missing_attribute_lines(
    path: &Path,
    lines: &[String],
) -> Result<usize, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("{}: failed to create directory: {err}", parent.display()))?;
    }
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(format!(
                "{}: failed to read attributes: {err}",
                path.display()
            ));
        }
    };
    let mut updated = existing.clone();
    let existing_lines = existing.lines().collect::<Vec<_>>();
    let mut added = 0;
    for line in lines {
        if existing_lines.iter().any(|existing| existing == line) {
            continue;
        }
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(line);
        updated.push('\n');
        added += 1;
    }
    if added > 0 {
        fs::write(path, updated)
            .map_err(|err| format!("{}: failed to write attributes: {err}", path.display()))?;
    }
    Ok(added)
}

pub(super) fn remove_yamark_filter_attribute_lines(path: &Path) -> Result<usize, String> {
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => {
            return Err(format!(
                "{}: failed to read attributes: {err}",
                path.display()
            ));
        }
    };
    let mut removed = 0;
    let mut kept = Vec::new();
    for line in existing.lines() {
        if is_yamark_filter_attribute_setup_line(line) {
            removed += 1;
        } else {
            kept.push(line);
        }
    }
    if removed == 0 {
        return Ok(0);
    }
    let mut updated = kept.join("\n");
    if !updated.is_empty() && existing.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(path, updated)
        .map_err(|err| format!("{}: failed to write attributes: {err}", path.display()))?;
    Ok(removed)
}

fn is_yamark_filter_attribute_setup_line(line: &str) -> bool {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return false;
    }
    let mut fields = line.split_whitespace();
    if fields.next().is_none() {
        return false;
    }
    let Some(attribute) = fields.next() else {
        return false;
    };
    attribute == "filter=yamark-md" && fields.next().is_none()
}
