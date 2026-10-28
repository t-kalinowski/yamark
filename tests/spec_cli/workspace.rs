use super::support::{run_stdin, yamark};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn paths_modes_config_and_skips_are_deterministic() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("yamark.toml"),
        "[format]\nmarkdown_horizontal_rule = \"***\"\n",
    )
    .unwrap();
    fs::write(root.join("a.md"), "#   Title ##\n\n---\n").unwrap();
    fs::write(root.join("ignored.txt"), "#   Title ##\n").unwrap();

    let output = yamark()
        .args([
            "format",
            root.join("a.md").to_str().unwrap(),
            root.join("ignored.txt").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 files scanned, 1 formatted, 0 unchanged, 1 skipped, 0 failed\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("a.md")).unwrap(),
        "# Title\n\n***\n"
    );

    fs::write(root.join("a.md"), "#   Title ##\n").unwrap();
    let output = yamark()
        .args(["format", "--check", root.join("a.md").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );

    let output = yamark()
        .args(["format", "--diff", root.join("a.md").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("@@ -1 +1 @@\n-#   Title ##\n+# Title\n")
    );
}

#[test]
fn format_skips_cmd_files() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("build.cmd");
    let input = "Title\n=====\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 0 unchanged, 1 skipped, 0 failed\n"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn format_skips_json_family_files_without_rewriting_them() {
    let dir = tempdir().unwrap();
    for (extension, input) in [
        ("json", "{\"unformatted\":[1,2]}\n"),
        ("jsonc", "{// comment\n\"unformatted\":[1,2,],}\n"),
        ("json5", "{unformatted:[1,2,],}\n"),
        ("jsonl", "{\"unformatted\":[1,2]}\n"),
        ("ndjson", "{\"unformatted\":[1,2]}\n"),
    ] {
        let path = dir.path().join(format!("input.{extension}"));
        fs::write(&path, input).unwrap();

        let output = yamark()
            .args(["format", path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "1 files scanned, 0 formatted, 0 unchanged, 1 skipped, 0 failed\n"
        );
        assert_eq!(fs::read_to_string(path).unwrap(), input);
    }
}

#[test]
fn git_filter_wrap_ignores_format_config() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[format]\nwrap = \"none\"\n",
    )
    .unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["git-filter", "clean", "--stdin-filename", "input.md"])
        .write_stdin("First sentence. Second sentence.\n")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "First sentence.\nSecond sentence.\n"
    );
}

#[test]
fn invalid_config_keys_fail() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "[format]\nunknown = true\n").unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains(&format!("{}:1:1", config.display())),
        "{stderr}"
    );
    assert!(stderr.contains("unknown format config key: format.unknown"));
}

#[test]
fn discovered_format_config_sets_markdown_wrap() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[format]\nwrap = \"sentence\"\n",
    )
    .unwrap();
    let path = dir.path().join("input.md");
    fs::write(&path, "First sentence. Second sentence.\n").unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        "First sentence.\nSecond sentence.\n"
    );
}

#[test]
fn command_line_wrap_overrides_format_config() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "[format]\nwrap = \"none\"\n").unwrap();

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
            "--wrap",
            "sentence",
        ],
        "First sentence. Second sentence.\n",
    );

    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "First sentence.\nSecond sentence.\n");
    assert_eq!(stderr, "");
}

#[test]
fn format_config_rejects_invalid_markdown_wrap() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "[format]\nwrap = \"wide\"\n").unwrap();

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );

    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains(
            "wrap must be none, paragraph, sentence, a positive integer, or sentence:<positive integer>"
        ),
        "{stderr}"
    );
}

#[test]
fn embedded_config_must_be_a_table() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(&config, "embedded = true\n").unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("embedded config must be a table"),
        "{stderr}"
    );
}

#[test]
fn file_contents_are_valid_utf8() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.md");
    fs::write(&path, [0xff, 0xfe, 0x00, 0x00]).unwrap();
    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unsupported encoding: UTF-16 BOM")
    );
}

#[test]
fn explicit_symlink_targets_are_deduplicated_by_canonical_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    let link = dir.path().join("link.md");
    fs::write(&path, "#   Title ##\n").unwrap();
    if !make_symlink(&path, &link) {
        return;
    }

    let output = yamark()
        .args(["format", path.to_str().unwrap(), link.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
}

#[cfg(unix)]
#[test]
fn explicit_symlink_directories_are_walked() {
    let dir = tempdir().unwrap();
    let target_dir = dir.path().join("target");
    let link_dir = dir.path().join("link");
    fs::create_dir(&target_dir).unwrap();
    let path = target_dir.join("a.md");
    fs::write(&path, "#   Title ##\n").unwrap();
    std::os::unix::fs::symlink(&target_dir, &link_dir).unwrap();

    let output = yamark()
        .args(["format", link_dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), "# Title\n");
}

#[cfg(unix)]
#[test]
fn explicit_parent_symlink_directory_walk_does_not_recurse_through_nested_symlink() {
    let dir = tempdir().unwrap();
    let parent = dir.path().join("parent");
    let child = parent.join("child");
    let up = child.join("up");
    fs::create_dir(&parent).unwrap();
    fs::create_dir(&child).unwrap();
    fs::write(parent.join("parent.md"), "#   Parent ##\n").unwrap();
    fs::write(child.join("child.md"), "#   Child ##\n").unwrap();
    std::os::unix::fs::symlink("..", &up).unwrap();

    let output = yamark()
        .args(["format", up.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 files scanned, 2 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
    assert_eq!(
        fs::read_to_string(parent.join("parent.md")).unwrap(),
        "# Parent\n"
    );
    assert_eq!(
        fs::read_to_string(child.join("child.md")).unwrap(),
        "# Child\n"
    );
}

#[test]
fn directory_diagnostics_are_deterministic_for_multiple_yaml_failures() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.yaml");
    let b = dir.path().join("b.yaml");
    fs::write(&a, "# fmt: markdown\nitems: [one, two]\n").unwrap();
    fs::write(&b, "# fmt: markdown\nitems: [one, two]\n").unwrap();

    let output = yamark()
        .args(["format", "--diagnostics", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        "2 files scanned, 0 formatted, 0 unchanged, 0 skipped, 2 failed\n"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    let first = stderr.find(&format!("{}:2:8", a.display())).unwrap();
    let second = stderr.find(&format!("{}:2:8", b.display())).unwrap();
    assert!(first < second, "{stderr}");
}

#[test]
fn path_specific_template_config_preserves_matching_blocks() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This << keep   this >> paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 1 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn path_specific_template_config_applies_during_relative_directory_scan() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This << keep   this >> paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "."])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 files scanned, 0 formatted, 1 unchanged, 1 skipped, 0 failed\n"
    );
}

#[test]
fn directory_formatting_starts_before_later_config_discovery_fails() {
    let dir = tempdir().unwrap();
    let marker = dir.path().join("formatter-ran");
    let formatter = dir.path().join("formatter.py");
    fs::write(
        &formatter,
        "import pathlib, sys\npathlib.Path(sys.argv[1]).touch()\nsys.stdout.buffer.write(sys.stdin.buffer.read())\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        format!(
            "[embedded.txt]\nformatter = {{ command = [\"python\", {:?}, {:?}, \"{{path}}\"], path_suffix = \".txt\" }}\n",
            formatter.to_str().unwrap(),
            marker.to_str().unwrap(),
        ),
    )
    .unwrap();
    fs::write(dir.path().join("a.md"), "```txt\nabc\n```\n").unwrap();

    let later = dir.path().join("z");
    fs::create_dir(&later).unwrap();
    fs::write(later.join("yamark.toml"), "[format]\nunknown = true\n").unwrap();
    fs::write(later.join("b.md"), "# Title\n").unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "."])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unknown format config key: format.unknown")
    );
    assert!(marker.is_file());
}

#[test]
fn discovered_nested_config_path_configs_match_relative_to_config_dir() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("sub");
    let docs = subdir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(
        subdir.join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"%%\", close = \"%%\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This %% keep   this %% paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "sub/docs/a.md"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 1 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn explicit_config_path_configs_match_relative_to_config_dir() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("configs");
    let docs = dir.path().join("docs");
    fs::create_dir(&config_dir).unwrap();
    fs::create_dir(&docs).unwrap();
    fs::write(
        config_dir.join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"%%\", close = \"%%\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    fs::write(&path, "This %% keep   this %% paragraph.\n").unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "--config", "configs/yamark.toml", "docs/a.md"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "This %% keep this %% paragraph.\n"
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn relative_explicit_config_applies_path_config_to_absolute_paths() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.md");
    let input = "This << keep   this >> paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .current_dir(dir.path())
        .args(["format", "--config", "yamark.toml", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 files scanned, 0 formatted, 1 unchanged, 0 skipped, 0 failed\n"
    );
}

#[test]
fn path_specific_embedded_markdown_template_config_preserves_targets() {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(
        dir.path().join("yamark.toml"),
        "[paths.\"docs\".embedded_markdown.template]\nadd_delimiters = [{ open = \"[[\", close = \"]]\" }]\n",
    )
    .unwrap();
    let path = docs.join("a.py");
    let input = "# fmt: markdown\n# This [[ keep   spacing ]] paragraph.\n";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
}

#[test]
fn non_spec_hard_break_space_config_is_rejected() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[format]\npreserve_markdown_hard_break_spaces = true\n",
    )
    .unwrap();
    let input = "line with hard break  \nnext line\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("unknown format config key: format.preserve_markdown_hard_break_spaces")
    );
}

#[cfg(unix)]
fn make_symlink(path: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(path, link).unwrap();
    true
}

#[cfg(windows)]
fn make_symlink(path: &Path, link: &Path) -> bool {
    match std::os::windows::fs::symlink_file(path, link) {
        Ok(()) => true,
        Err(err) if err.raw_os_error() == Some(1314) => {
            eprintln!("symlink test requires Windows Developer Mode or symlink privileges: {err}");
            false
        }
        Err(err) => panic!("failed to create symlink: {err}"),
    }
}

#[test]
fn template_delimiter_entries_reject_unknown_fields() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[template]\nadd_delimiters = [{ open = \"<<\", close = \">>\", extra = true }]\n",
    )
    .unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("unknown template delimiter key: template.add_delimiters.extra"),
        "{stderr}"
    );
}

#[test]
fn path_template_delimiter_entries_reject_unknown_fields() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"<<\", close = \">>\", extra = true }]\n",
    )
    .unwrap();
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "docs/input.md",
            "--config",
            config.to_str().unwrap(),
        ],
        "# Title\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("unknown template delimiter key: paths.docs.template.add_delimiters.extra"),
        "{stderr}"
    );
}
