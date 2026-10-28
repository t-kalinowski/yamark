use super::support::{run_stdin, yamark};
use std::fs;
use std::io::Write;
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn format_rejects_unsupported_fast_option() {
    let (status, stdout, stderr) = run_stdin(
        &["format", "--fast", "--stdin-file-path", "input.md"],
        "# Title\n",
    );
    assert_eq!(status, 2);
    assert_eq!(stdout, "");
    assert!(stderr.contains("unexpected argument '--fast'"), "{stderr}");
}

#[test]
fn format_help_shows_default_wrap() {
    let (status, stdout, stderr) = run_stdin(&["format", "--help"], "");
    assert_eq!(status, 0, "{stderr}");
    let wrap = stdout
        .lines()
        .find(|line| line.contains("--wrap <WRAP>"))
        .unwrap();
    assert!(wrap.contains("[default: 72]"), "{wrap}");
    assert_eq!(stderr, "");
}

#[test]
fn format_help_verify_visibility_matches_build_profile() {
    let (status, stdout, stderr) = run_stdin(&["format", "--help"], "");
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(
        stdout.lines().any(|line| line.contains("--verify")),
        cfg!(debug_assertions),
        "{stdout}"
    );
    assert_eq!(stderr, "");
}

#[test]
fn cli_help_honors_clap_color_env() {
    let output = yamark()
        .env_remove("NO_COLOR")
        .env("CLICOLOR_FORCE", "1")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\u{1b}[94mUsage:"), "{stdout}");
    assert!(stdout.contains("\u{1b}[36myamark"), "{stdout}");
    assert!(stdout.contains("\u{1b}[90m<COMMAND>"), "{stdout}");

    let output = yamark()
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("\u{1b}["), "{stdout}");
}

#[test]
fn stdin_encoding_errors_are_targeted() {
    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args(["format", "--stdin-file-path", "bad.md"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&[0xff, 0xfe, 0x00, 0x00])
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("bad.md:1:1: error: unsupported encoding: UTF-16 BOM")
    );

    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args(["format", "--stdin-file-path", "bad.md"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(&[0xff]).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("bad.md:1:1: error: invalid UTF-8")
    );
}

#[test]
fn to_yaml_accepts_stdin_only() {
    let output = yamark()
        .args(["to-yaml", "--stdin-file-path", "input.json", "other.json"])
        .write_stdin("{}\n")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout).unwrap().is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unexpected argument 'other.json'")
    );
}

#[test]
fn to_yaml_rejects_native_source_types() {
    for path in ["input.md", "input.yaml", "input.py", "input.r"] {
        let output = yamark()
            .args(["to-yaml", "--stdin-file-path", path])
            .write_stdin("{}\n")
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8(output.stdout).unwrap().is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains(
                    "JSON-to-YAML projection requires a .json, .jsonc, .json5, .jsonl, or .ndjson source path"
                ),
            "{path} should not be projected"
        );
    }
}

#[test]
fn command_help_exposes_to_yaml_without_render() {
    let output = yamark().arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("to-yaml"));
    assert!(
        !stdout
            .lines()
            .any(|line| line.trim_start().starts_with("render "))
    );
}

#[test]
fn render_is_not_a_hidden_compatibility_alias() {
    let output = yamark().arg("render").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout).unwrap().is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unrecognized subcommand 'render'")
    );
}

#[test]
fn to_yaml_json5_rejects_excessive_nesting_without_aborting() {
    let input = format!("{}/* comment */0{}", "[".repeat(900), "]".repeat(900));
    let output = yamark()
        .args(["to-yaml", "--stdin-file-path", "input.json5"])
        .write_stdin(input)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout).unwrap().is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("invalid JSON5: nesting exceeds 256 levels")
    );
}

#[test]
fn to_yaml_jsonc_parser_inputs_reject_excessive_nesting_without_aborting() {
    let input = format!("{}0{}", "[".repeat(10_000), "]".repeat(10_000));
    for (extension, dialect) in [
        ("json", "JSON"),
        ("jsonc", "JSONC"),
        ("jsonl", "JSON"),
        ("ndjson", "JSON"),
    ] {
        let path = format!("input.{extension}");
        let output = yamark()
            .args(["to-yaml", "--stdin-file-path", &path])
            .write_stdin(input.clone())
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8(output.stdout).unwrap().is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains(&format!(
                "{path}:1:257: error: invalid {dialect}: nesting exceeds 256 levels"
            )),
            "{stderr}"
        );
    }
}

#[test]
fn to_yaml_json5_rejects_repeated_unary_operators_without_aborting() {
    let input = format!("{}1", "+".repeat(10_000));
    let output = yamark()
        .args(["to-yaml", "--stdin-file-path", "input.json5"])
        .write_stdin(input)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout).unwrap().is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("invalid JSON5: Only one unary operator is allowed")
    );
}

#[test]
fn diff_mode_prints_contextual_unified_hunks() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("input.yaml");
    let input = "\
top: one
a: 1
b: 2
c: 3
d: 4
e: 5
f: 6
items: [a,b]
";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", "--diff", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--- "));
    assert!(stdout.contains("+++ "));
    assert!(
        stdout.contains("@@ -5,4 +5,4 @@\n d: 4\n e: 5\n f: 6\n-items: [a,b]\n+items: [a, b]\n"),
        "{stdout}"
    );
    assert!(!stdout.contains(" top: one\n a: 1\n b: 2\n"), "{stdout}");
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_reports_missing_final_newline_changes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("input.md");
    let input = "# Title";
    fs::write(&path, input).unwrap();

    let output = yamark()
        .args(["format", "--diff", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("@@ -1 +1 @@\n-# Title\n\\ No newline at end of file\n+# Title\n"),
        "{stdout}"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_handles_large_inputs_without_quadratic_memory() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.yaml");
    let mut input = String::new();
    for i in 0..12_000 {
        input.push_str(&format!("key_{i}: value\n"));
    }
    input.push_str("items: [a,b]\n");
    for i in 12_000..24_000 {
        input.push_str(&format!("key_{i}: value\n"));
    }
    fs::write(&path, &input).unwrap();

    let output = yamark()
        .args(["format", "--diff", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("-items: [a,b]\n+items: [a, b]\n"),
        "{stdout}"
    );
    assert!(
        !stdout.contains(" key_0: value\n key_1: value\n"),
        "{stdout}"
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_resyncs_after_large_fallback_insertions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.yaml");
    let mut input = String::new();
    let items = (0..100)
        .map(|i| format!("item_{i}"))
        .collect::<Vec<_>>()
        .join(",");
    input.push_str("items: [");
    input.push_str(&items);
    input.push_str("]\n");
    for i in 0..3_000 {
        input.push_str(&format!("key_{i}: value\n"));
    }
    input.push_str("second: [c,d]\n");
    fs::write(&path, &input).unwrap();

    let output = yamark()
        .args([
            "format",
            "--diff",
            "--line-width",
            "40",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let hunk_count = stdout.matches("@@ ").count();
    assert_eq!(hunk_count, 2);
    assert!(stdout.contains("-items: [item_0,item_1,"));
    assert!(stdout.contains("+items:\n+  - item_0\n+  - item_1\n"));
    assert!(stdout.contains("-second: [c,d]\n+second: [c, d]\n"));
    assert!(!stdout.contains("key_1500: value\n"));
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn diff_mode_handles_repeated_large_fallback_syncs_without_hanging() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.yaml");
    let mut input = String::new();
    let items = (0..128)
        .map(|i| format!("item_{i}"))
        .collect::<Vec<_>>()
        .join(",");
    for block in 0..400 {
        input.push_str(&format!("items_{block}: [{items}]\n"));
        input.push_str(&format!("key_{block}: value\n"));
    }
    fs::write(&path, &input).unwrap();

    let stdout_path = dir.path().join("diff.out");
    let stdout_file = fs::File::create(&stdout_path).unwrap();
    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args([
            "format",
            "--diff",
            "--line-width",
            "40",
            path.to_str().unwrap(),
        ])
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(2) {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("diff mode hung on repeated large fallback syncs");
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = fs::read_to_string(stdout_path).unwrap();
    assert!(stdout.contains("-items_0: [item_0,item_1,"));
    assert!(stdout.contains("+items_0:\n+  - item_0\n+  - item_1\n"));
    assert!(stdout.contains("-items_399: [item_0,item_1,"));
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn git_filter_formats_only_markdown() {
    let (status, stdout, stderr) = run_stdin(
        &["git-filter", "clean", "--stdin-filename", "input.md"],
        "One sentence. Two sentences.\n",
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "One sentence.\nTwo sentences.\n");

    let (status, stdout, stderr) = run_stdin(
        &["git-filter", "clean", "--stdin-filename", "table.md"],
        "| a | long |\n|---|:---:|\n| 1 | two |\n",
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, "| a | long |\n| --- | :---: |\n| 1 | two |\n");

    let (status, stdout, stderr) = run_stdin(
        &["git-filter", "clean", "--stdin-filename", "input.txt"],
        "#   Title ##\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("unsupported Git filter path"));
}

#[test]
fn bom_prefixed_first_line_directives_apply() {
    let cases = [
        (
            "input.yaml",
            "\u{feff}# fmt: skip file\na:    b\n",
            "\u{feff}# fmt: skip file\na:    b\n",
        ),
        (
            "input.md",
            "\u{feff}<!-- fmt: skip file -->\n#   Title ##\n",
            "\u{feff}<!-- fmt: skip file -->\n#   Title ##\n",
        ),
        (
            "input.py",
            "\u{feff}# fmt: markdown\n# #   Title ##\n",
            "\u{feff}# fmt: markdown\n# # Title\n",
        ),
        (
            "input.r",
            "\u{feff}# fmt: markdown\n# #   Title ##\n",
            "\u{feff}# fmt: markdown\n# # Title\n",
        ),
    ];

    for (path, input, expected) in cases {
        let (status, stdout, stderr) = run_stdin(
            &["format", "--stdin-file-path", path, "--wrap", "none"],
            input,
        );
        assert_eq!(status, 0, "{path}: {stderr}");
        assert_eq!(stdout, expected, "{path}");
        assert_eq!(stderr, "", "{path}");
    }
}
