use super::support::{run_stdin, yamark};
use std::fs;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command as ProcessCommand;
#[cfg(unix)]
use std::process::Stdio;
#[cfg(unix)]
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;
use tempfile::tempdir;

#[cfg(unix)]
fn fake_ruff_path_env(dir: &Path) -> std::ffi::OsString {
    use std::os::unix::fs::PermissionsExt;

    let ruff = dir.join("ruff");
    fs::write(
        &ruff,
        r#"#!/usr/bin/env python3
import json
import sys

if sys.argv[1:3] != ["format", "--stdin-filename"] or sys.argv[4:] != ["-"]:
    sys.stderr.write(f"unexpected argv: {sys.argv[1:]!r}\n")
    sys.exit(2)

if not sys.argv[3].endswith(".ipynb"):
    sys.stderr.write(f"expected ipynb path, got {sys.argv[3]}\n")
    sys.exit(2)

try:
    notebook = json.load(sys.stdin)
except Exception as err:
    sys.stderr.write(f"stdin was not notebook json: {err}\n")
    sys.exit(2)

cells = notebook.get("cells")
if not isinstance(cells, list) or len(cells) != 1:
    sys.stderr.write("expected exactly one notebook cell\n")
    sys.exit(2)

cell = cells[0]
if cell.get("cell_type") != "code":
    sys.stderr.write("expected a code cell\n")
    sys.exit(2)

source = cell.get("source")
if isinstance(source, list):
    source = "".join(source)
elif not isinstance(source, str):
    sys.stderr.write("expected string or array cell source\n")
    sys.exit(2)

if source.startswith("%%tab pytorch\n"):
    formatted = source
else:
    formatted = source.replace("x= 1", "x = 1")

if "%load_ext d2lbook.tab" in source:
    cell["source"] = formatted.splitlines(True)
else:
    cell["source"] = formatted

json.dump(notebook, sys.stdout)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&ruff).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&ruff, permissions).unwrap();

    let mut path_entries = vec![dir.to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(path_entries).unwrap()
}

#[cfg(unix)]
fn failing_ruff_path_env(dir: &Path) -> std::ffi::OsString {
    use std::os::unix::fs::PermissionsExt;

    let ruff = dir.join("ruff");
    fs::write(
        &ruff,
        r#"#!/bin/sh
cat >/dev/null
printf 'error: Failed to parse input.md.embedded.2.ipynb:7:9: Unexpected indentation\n' >&2
printf '> 7 |     x=1\n' >&2
printf '    |         ^\n' >&2
exit 2
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&ruff).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&ruff, permissions).unwrap();

    let mut path_entries = vec![dir.to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(path_entries).unwrap()
}

#[test]
fn skipped_external_code_fence_preserves_entire_target_span() {
    let input = "\
```{.python .python #cell}
print('x')
```
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--skip-embedded-formatters",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn missing_optional_external_code_fence_preserves_entire_target_span() {
    let input = "\
```{.python .python #cell}
print('x')
```
";
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .env("PATH", "")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.md:2:1: note: missing optional embedded formatter `ruff`; preserved source"
    ));
}

#[test]
fn missing_configured_ruff_and_air_shorthands_are_diagnostics() {
    for (name, shorthand, diagnostic) in [
        (
            "custom-python",
            "ruff",
            "missing optional embedded formatter `ruff`",
        ),
        (
            "custom-r",
            "air",
            "missing optional embedded formatter `air`",
        ),
    ] {
        let dir = tempdir().unwrap();
        let config = dir.path().join("yamark.toml");
        fs::write(
            &config,
            format!("[embedded.{name}]\nformatter = \"{shorthand}\"\n"),
        )
        .unwrap();
        let input = format!("```{name}\nprint('x')\n```\n");
        let output = yamark()
            .args([
                "format",
                "--diagnostics",
                "--stdin-file-path",
                "input.md",
                "--config",
                config.to_str().unwrap(),
            ])
            .env("PATH", "")
            .write_stdin(input.as_str())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains(diagnostic)
        );
    }
}

#[test]
fn yaml_embedded_formatter_directive_rejects_explicit_block_scalar_keys() {
    let input = "\
# fmt: json
? |
  key
: value
";
    let (status, stdout, stderr) = run_stdin(&["format", "--stdin-file-path", "input.yaml"], input);
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("fmt: json targets a literal block scalar"),
        "{stderr}"
    );
}

#[cfg(unix)]
#[test]
fn external_formatter_handles_large_stdin_stdout_without_deadlock() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let formatter = dir.path().join("echo-stdin");
    fs::write(&formatter, "#!/bin/sh\ncat\n").unwrap();
    let mut permissions = fs::metadata(&formatter).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&formatter, permissions).unwrap();

    fs::write(
        dir.path().join("yamark.toml"),
        format!(
            "[embedded.echo]\nformatter = {{ command = [\"{}\", \"{{path}}\"], path_suffix = \".txt\" }}\n",
            formatter.display()
        ),
    )
    .unwrap();
    let path = dir.path().join("large.md");
    let payload = "a".repeat(4 * 1024 * 1024);
    let input = format!("```echo\n{payload}\n```\n");
    fs::write(&path, &input).unwrap();

    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin("yamark"))
        .args(["format", path.to_str().unwrap()])
        .current_dir(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(5) {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("external formatter deadlocked on large stdin/stdout");
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn expanded_embedded_formatter_requires_path_suffix() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[embedded.custom]\nformatter = { command = [\"tool\", \"{path}\"] }\n",
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
    assert!(stderr.contains("missing embedded formatter path_suffix"));
}

#[test]
fn embedded_formatter_entries_require_formatter_key() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[embedded.custom]\ncommand = [\"tool\", \"{path}\"]\n",
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
        stderr.contains("embedded formatter entries must contain formatter"),
        "{stderr}"
    );
}

#[test]
fn source_files_run_external_formatter_directives_for_string_targets() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.r]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().replace(b'function(x)x+1', b'function(x) x + 1'))\", \"{path}\"], path_suffix = \".R\" }
",
    )
    .unwrap();

    let input = "\
\"\"\"Module docstring.\"\"\"

def read_source() -> str:
    return run(
        # fmt: r
        stdin=\"\"\"
            f <- function(x)x+1
        \"\"\",
    )
";
    let expected = "\
\"\"\"Module docstring.\"\"\"

def read_source() -> str:
    return run(
        # fmt: r
        stdin=\"\"\"
            f <- function(x) x + 1
        \"\"\",
    )
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.py",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn explicit_embedded_formatters_run_for_yaml_scalars_and_code_fences() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.upper]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().upper())\", \"{path}\"], path_suffix = \".txt\" }

[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().upper())\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: upper\nbody: |\n  abc\n";
    let expected = "# fmt: upper\nbody: |\n  ABC\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);

    let input = "```txt\nabc\n```\n";
    let expected = "```txt\nABC\n```\n";
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
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);

    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.md",
            "--config",
            config.to_str().unwrap(),
            "--skip-embedded-formatters",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn yaml_embedded_formatter_directive_accepts_explicit_scope_next() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().upper())\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt scope=next\nbody: |\n  abc\n";
    let expected = "# fmt: txt scope=next\nbody: |\n  ABC\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn external_yaml_scalar_formatter_output_is_reindented() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'x\\\\n')\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |\n  y\n";
    let expected = "# fmt: txt\nbody: |\n  x\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn yaml_verify_rejects_invalid_embedded_formatter_output() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        r#"
[embedded.invalid]
formatter = { command = ["python", "-c", "import sys; sys.stdout.buffer.write(bytes([1]))", "{path}"], path_suffix = ".txt" }
"#,
    )
    .unwrap();

    let input = "# fmt: invalid\nbody: |\n  valid\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--verify",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("formatter produced invalid YAML"),
        "{stderr}"
    );

    let path = dir.path().join("input.yaml");
    fs::write(&path, input).unwrap();
    let output = yamark()
        .args([
            "format",
            "--verify",
            "--config",
            config.to_str().unwrap(),
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read_to_string(path).unwrap(), input);
}

#[test]
fn external_yaml_scalar_formatter_preserves_block_header_comments() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'x\\\\n')\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: | # markdown\n  y\n";
    let expected = "# fmt: txt\nbody: | # markdown\n  x\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn external_yaml_scalar_formatter_output_updates_strip_chomp_for_final_newline() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'x\\\\n')\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |-\n  y\n";
    let expected = "# fmt: txt\nbody: |\n  x\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn external_yaml_scalar_formatter_output_uses_explicit_block_indent_indicator() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'x\\\\n')\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |4\nnext: 1\n";
    let expected = "# fmt: txt\nbody: |4\n    x\nnext: 1\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn external_formatter_output_without_trailing_newline_stays_line_delimited() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'X')\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |\n  y\nnext: 1\n";
    let expected = "# fmt: txt\nbody: |\n  X\nnext: 1\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);

    let input = "```txt\nabc\n```\n";
    let expected = "```txt\nX\n```\n";
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
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn external_yaml_scalar_formatter_receives_dedented_block_content() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b''.join(b'seen:' + line for line in sys.stdin.buffer.read().splitlines(keepends=True)))\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "# fmt: txt\nbody: |\n    value\n";
    let expected = "# fmt: txt\nbody: |\n    seen:value\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn external_yaml_scalar_formatter_preserves_renderer_preamble() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.argv[1].encode() + b'\\\\n' + sys.stdin.buffer.read().upper())\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "\
# fmt: txt
script: |
  #| echo: false
  #@ trace
  abc
";
    let expected = "\
# fmt: txt
script: |
  #| echo: false
  #@ trace
  input.yaml.embedded.3.txt
  ABC
";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
    assert_eq!(stderr, "");
}

#[test]
fn skipped_external_yaml_scalar_preserves_entire_target_span() {
    let input = "# fmt: json\nbody:    |\n  {\"name\":\"yamark\"}\n";
    let (status, stdout, stderr) = run_stdin(
        &[
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--skip-embedded-formatters",
        ],
        input,
    );
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, input);
}

#[test]
fn missing_optional_external_yaml_scalar_preserves_entire_target_span() {
    let input = "# fmt: json\nbody:    |\n  {\"name\":\"yamark\"}\n";
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.yaml"])
        .env("PATH", "")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.yaml:3:1: note: missing optional embedded formatter `prettier`; preserved source"
    ));
}

#[test]
fn configured_prettier_shorthand_is_optional_when_missing() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "[embedded.custom-json]\nformatter = \"prettier-json\"\n",
    )
    .unwrap();
    let input = "# fmt: custom-json\nbody:    |\n  {\"name\":\"yamark\"}\n";

    let output = yamark()
        .args([
            "format",
            "--diagnostics",
            "--stdin-file-path",
            "input.yaml",
            "--config",
            config.to_str().unwrap(),
        ])
        .env("PATH", "")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.yaml:3:1: note: missing optional embedded formatter `prettier`; preserved source"
    ));
}

#[test]
fn external_formatter_receives_source_based_virtual_path_and_not_renderer_preamble() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.argv[1].encode() + b'\\\\n' + sys.stdin.buffer.read().upper())\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "```txt\n#| label: fig\nabc\n```\n";
    let expected = "```txt\n#| label: fig\ninput.md.embedded.2.txt\nABC\n```\n";
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
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn external_formatter_preamble_split_supports_cr_only_code_fences() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(sys.argv[1].encode() + b'\\\\n' + sys.stdin.buffer.read().upper())\", \"{path}\"], path_suffix = \".txt\" }
",
    )
    .unwrap();

    let input = "```txt\r#| label: fig\rabc\r```\r";
    let expected = "```txt\r#| label: fig\rinput.md.embedded.2.txt\nABC\r```\r";
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
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn code_fence_formatter_output_cannot_close_the_original_fence() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'```\\\\n')\", \"{path}\"], path_suffix = \".txt\" }
",
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
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("formatted code fence would contain closing fence"));
}

#[cfg(unix)]
#[test]
fn code_fence_formatter_output_cannot_close_original_fence_with_cr_only_lines() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let formatter = dir.path().join("formatter");
    fs::write(&formatter, "#!/bin/sh\nprintf 'ok\\r```\\r'\n").unwrap();
    let mut permissions = fs::metadata(&formatter).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&formatter, permissions).unwrap();

    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        format!(
            "\
[embedded.txt]
formatter = {{ command = [{:?}, \"{{path}}\"], path_suffix = \".txt\" }}
",
            formatter.to_str().unwrap()
        ),
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
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("formatted code fence would contain closing fence"));
}

#[cfg(unix)]
#[test]
fn default_embedded_formatter_mappings_run_for_yaml_scalars_and_code_fences() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let prettier = dir.path().join("prettier");
    fs::write(&prettier, "#!/bin/sh\ntr a-z A-Z\n").unwrap();
    let mut permissions = fs::metadata(&prettier).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&prettier, permissions).unwrap();
    let mut path_entries = vec![dir.path().to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(path_entries).unwrap();

    let output = yamark()
        .args(["format", "--stdin-file-path", "input.yaml"])
        .env("PATH", &path)
        .write_stdin("# fmt: json\nbody: |\n  {\"name\":\"yamark\"}\n")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "# fmt: json\nbody: |\n  {\"NAME\":\"YAMARK\"}\n"
    );

    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin("```json\n{\"name\":\"yamark\"}\n```\n")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "```json\n{\"NAME\":\"YAMARK\"}\n```\n"
    );
}

#[cfg(unix)]
#[test]
fn python_code_fence_with_line_magic_formats_through_ruff_notebook_cell() {
    let dir = tempdir().unwrap();
    let path = fake_ruff_path_env(dir.path());

    let input = "\
```python
%load_ext d2lbook.tab
x= 1
```
";
    let expected = "\
```python
%load_ext d2lbook.tab
x = 1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn python_code_fence_with_cell_magic_is_preserved_by_ruff_notebook_cell() {
    let dir = tempdir().unwrap();
    let path = fake_ruff_path_env(dir.path());

    let input = "\
```python
%%tab pytorch
x= 1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn ordinary_python_code_fence_still_formats_through_ruff() {
    let dir = tempdir().unwrap();
    let path = fake_ruff_path_env(dir.path());

    let input = "\
```python
x= 1
```
";
    let expected = "\
```python
x = 1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn failing_optional_embedded_formatter_preserves_code_fence_silently() {
    let dir = tempdir().unwrap();
    let path = failing_ruff_path_env(dir.path());

    let input = "\
```python
    x=1
```
";
    let output = yamark()
        .args(["format", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn failing_optional_embedded_formatter_reports_diagnostic_when_requested() {
    let dir = tempdir().unwrap();
    let path = failing_ruff_path_env(dir.path());

    let input = "\
```python
    x=1
```
";
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .env("PATH", &path)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("input.md:2:1: note: embedded formatter `ruff` failed at formatter input 7:9; left chunk unchanged"),
        "{stderr}"
    );
    assert!(!stderr.contains("x=1"), "{stderr}");
}

#[test]
fn embedded_formatter_stderr_on_success_is_an_error() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.stdout.buffer.write(b'formatted'); sys.stderr.buffer.write(b'warning')\", \"{path}\"], path_suffix = \".txt\" }
",
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
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("embedded formatter wrote to stderr: warning"));
}

#[cfg(unix)]
#[test]
fn embedded_formatter_nonzero_with_empty_stderr_reports_context() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("yamark.toml");
    fs::write(
        &config,
        "\
[embedded.txt]
formatter = { command = [\"python\", \"-c\", \"import sys; sys.exit(7)\", \"{path}\"], path_suffix = \".txt\" }
",
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
        "```txt\nabc\n```\n",
    );
    assert_eq!(status, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("embedded formatter failed"), "{stderr}");
    assert!(stderr.contains("python"), "{stderr}");
    assert!(stderr.contains("exit status 7"), "{stderr}");
    assert!(stderr.contains("input.md.embedded.2.txt"), "{stderr}");
}

#[test]
fn missing_optional_formatter_reports_diagnostic_notes_when_requested() {
    let output = yamark()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .env("PATH", "")
        .write_stdin("```python\nprint('x')\n```\n")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "```python\nprint('x')\n```\n"
    );
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "input.md:2:1: note: missing optional embedded formatter `ruff`; preserved source"
    ));
}
