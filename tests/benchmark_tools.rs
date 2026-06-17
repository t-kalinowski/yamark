use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::tempdir;

fn python() -> OsString {
    std::env::var_os("PYTHON").unwrap_or_else(|| OsString::from("python3"))
}

#[test]
fn yaml_benchmark_writes_one_commit_json_with_formatter_rows() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "per-file",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut artifacts = fs::read_dir(&artifact_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    artifacts.sort();
    assert_eq!(artifacts.len(), 1);

    let data: Value = serde_json::from_str(&fs::read_to_string(&artifacts[0]).unwrap()).unwrap();
    assert_eq!(data["schema_version"], 1);
    assert_eq!(data["benchmark"], "yaml-formatting");
    assert_eq!(data["invocation"]["unit"], "per-file");
    assert_eq!(data["corpus"]["kind"], "yaml");
    assert_eq!(data["corpus"]["files"], 2);
    assert!(data["git"]["commit"].as_str().unwrap().len() >= 7);

    let results = data["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["formatter"], "yamark");
    assert_eq!(results[0]["status"], "ok");
    assert!(results[0]["command"].as_str().unwrap().contains("TARGET"));
    assert!(results[0]["median_seconds"].as_f64().unwrap() > 0.0);
    assert!(results[0]["mb_per_second"].as_f64().unwrap() > 0.0);
    assert_eq!(results[0]["invocation"], "per-file");
    assert_eq!(results[0]["output_files"], 2);
    assert_eq!(results[0]["changed_files"], 2);
    assert!(results[0]["output_bytes"].as_u64().unwrap() > 0);
}

#[test]
fn yaml_benchmark_defaults_to_two_repetitions() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--files",
            "1",
            "--items",
            "1",
            "--warmups",
            "0",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["reps"], 2);
    assert_eq!(row["repetitions"].as_array().unwrap().len(), 2);
}

#[test]
fn yaml_benchmark_default_path_is_yamark_per_file() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--files",
            "1",
            "--items",
            "1",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    assert_eq!(data["invocation"]["unit"], "per-file");
    assert_eq!(data["selected_formatters"], serde_json::json!(["yamark"]));
    let rows = data["results"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["formatter"], "yamark");
    assert_eq!(rows[0]["invocation"], "per-file");
}

#[test]
fn big_benchmark_defaults_to_stable_repetitions() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/big.py")
        .args([
            "--markdown-bytes",
            "1200",
            "--yaml-bytes",
            "1200",
            "--frontmatter-bytes",
            "3000",
            "--frontmatter-yaml-bytes",
            "1000",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run big benchmark script: {err}"));

    assert!(
        output.status.success(),
        "big benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    for row in data["results"].as_array().unwrap() {
        if row["status"] == "ok" {
            assert_eq!(row["reps"], 10);
            assert_eq!(row["warmups"], 2);
            assert_eq!(row["repetitions"].as_array().unwrap().len(), 10);
        }
    }
}

#[test]
fn yaml_benchmark_can_generate_mixed_node_corpus() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--corpus-shape",
            "mixed-node",
            "--invocation",
            "per-file",
            "--files",
            "1",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--keep-corpus",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    assert_eq!(data["corpus"]["shape"], "mixed-node");
    assert_eq!(data["corpus"]["files"], 1);
    assert_eq!(data["corpus"]["items_per_file"], 2);
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["formatter"], "yamark");
    assert_eq!(row["status"], "ok");
    assert_eq!(row["output_files"], 1);
    assert_eq!(row["changed_files"], 1);

    let input = fs::read_to_string(
        fs::read_dir(&out_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("corpus")
            .join("service-00")
            .join("config-0000.yaml"),
    )
    .unwrap();
    assert!(!input.contains('\n'));
    assert!(input.starts_with("[{"));
    assert!(input.ends_with("}]"));
    assert!(input.contains("\"block_str\""));
    assert!(input.contains("\"bools\":[true,false]"));
    assert!(input.contains("\"ints\":[123,-123]"));
    assert!(input.contains("\"floats\":[123.456,-123.456]"));
    assert!(input.contains("\"null\":null"));
}

#[test]
fn yaml_benchmark_writes_separate_artifacts_for_different_corpus_shapes() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    for shape in ["flow-heavy", "mixed-node"] {
        let output = Command::new(python())
            .arg("tools/bench/run.py")
            .args([
                "--corpus",
                "yaml",
                "--corpus-shape",
                shape,
                "--invocation",
                "per-file",
                "--files",
                "1",
                "--items",
                "2",
                "--reps",
                "1",
                "--warmups",
                "0",
                "--tools",
                "yamark",
                "--skip-yamark-build",
                "--yamark-bin",
            ])
            .arg(assert_cmd::cargo::cargo_bin("yamark"))
            .arg("--out-dir")
            .arg(&out_dir)
            .arg("--artifact-dir")
            .arg(&artifact_dir)
            .output()
            .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

        assert!(
            output.status.success(),
            "benchmark failed for {shape}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut artifacts = fs::read_dir(&artifact_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    artifacts.sort();
    assert_eq!(artifacts.len(), 2);

    let mut shapes = artifacts
        .iter()
        .map(|path| {
            let data: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            data["corpus"]["shape"].as_str().unwrap().to_owned()
        })
        .collect::<Vec<_>>();
    shapes.sort();
    assert_eq!(shapes, ["flow-heavy", "mixed-node"]);
    assert!(artifacts.iter().any(|path| {
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with("-mixed-node.json")
    }));
}

#[test]
fn big_generator_writes_deterministic_single_file_corpora() {
    let dir = tempdir().unwrap();
    let first = dir.path().join("first");
    let second = dir.path().join("second");

    for out_dir in [&first, &second] {
        let output = Command::new("Rscript")
            .arg("tools/bench/big.R")
            .arg("--out-dir")
            .arg(out_dir)
            .arg("--markdown-bytes")
            .arg("64000")
            .arg("--yaml-bytes")
            .arg("64000")
            .arg("--frontmatter-bytes")
            .arg("64000")
            .arg("--seed")
            .arg("123")
            .output()
            .unwrap_or_else(|err| panic!("failed to run Rscript: {err}"));

        assert!(
            output.status.success(),
            "big-file generator failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    for name in ["big.md", "big.yaml", "big-with-frontmatter.md"] {
        assert_eq!(
            fs::read(first.join(name)).unwrap(),
            fs::read(second.join(name)).unwrap(),
            "{name} should be deterministic for a fixed seed"
        );
    }

    let markdown = fs::read_to_string(first.join("big.md")).unwrap();
    assert!(markdown.starts_with("# Generated Markdown Benchmark Document\n\n"));
    for marker in ["- ", "* ", "+ ", "1. ", "1) ", "(1) ", "a. ", "IV) "] {
        assert!(markdown.contains(marker), "missing list marker {marker:?}");
    }
    assert!(markdown.contains("](https://example.com/short/"));
    assert!(markdown.contains("](https://example.com/long/"));

    let yaml = fs::read_to_string(first.join("big.yaml")).unwrap();
    assert!(yaml.contains("# benchmark comment "));
    assert!(yaml.contains("block_str: |"));
    assert!(yaml.contains("bools:"));
    assert!(yaml.contains("ints:"));
    assert!(yaml.contains("floats:"));
    assert!(yaml.contains("\"null\": ~"));

    let frontmatter = fs::read_to_string(first.join("big-with-frontmatter.md")).unwrap();
    assert!(frontmatter.starts_with("---\n"));
    assert!(frontmatter.contains("\n---\n\n# Generated Markdown Benchmark Document\n\n"));
    assert!(frontmatter.contains("# benchmark comment "));
    assert!(frontmatter.contains("](https://example.com/short/"));
}

#[test]
fn big_defaults_use_four_megabyte_targets_and_small_frontmatter() {
    let python_harness = fs::read_to_string("tools/bench/big.py").unwrap();
    assert!(
        python_harness.contains("DEFAULT_MARKDOWN_BYTES = 4_000_000"),
        "big.py should default big.md to about 4 MB"
    );
    assert!(
        python_harness.contains("DEFAULT_YAML_BYTES = 4_000_000"),
        "big.py should default big.yaml to about 4 MB"
    );
    assert!(
        python_harness.contains("DEFAULT_FRONTMATTER_BYTES = 4_000_000"),
        "big.py should default big-with-frontmatter.md to about 4 MB"
    );
    assert!(
        python_harness.contains("DEFAULT_FRONTMATTER_YAML_RATIO = 0.05"),
        "big.py should derive the front matter block size from the document size"
    );

    let r_generator = fs::read_to_string("tools/bench/big.R").unwrap();
    assert!(
        r_generator.contains("DEFAULT_MARKDOWN_BYTES <- 4000000"),
        "big.R should default big.md to about 4 MB"
    );
    assert!(
        r_generator.contains("DEFAULT_YAML_BYTES <- 4000000"),
        "big.R should default big.yaml to about 4 MB"
    );
    assert!(
        r_generator.contains("DEFAULT_FRONTMATTER_BYTES <- 4000000"),
        "big.R should default big-with-frontmatter.md to about 4 MB"
    );
    assert!(
        r_generator.contains("DEFAULT_FRONTMATTER_YAML_RATIO <- 0.05"),
        "big.R should derive the front matter block size from the document size"
    );
}

#[test]
fn big_generator_defaults_frontmatter_yaml_between_tenth_and_half_of_lines() {
    let dir = tempdir().unwrap();
    let output = Command::new("Rscript")
        .arg("tools/bench/big.R")
        .arg("--out-dir")
        .arg(dir.path())
        .arg("--markdown-bytes")
        .arg("1000")
        .arg("--yaml-bytes")
        .arg("1000")
        .arg("--frontmatter-bytes")
        .arg("256000")
        .arg("--seed")
        .arg("123")
        .output()
        .unwrap_or_else(|err| panic!("failed to run Rscript: {err}"));

    assert!(
        output.status.success(),
        "big-file generator failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let frontmatter = fs::read_to_string(dir.path().join("big-with-frontmatter.md")).unwrap();
    let document = frontmatter.strip_prefix("---\n").unwrap();
    let (yaml, markdown) = document.split_once("\n---\n\n").unwrap();
    let yaml_lines = yaml.lines().count();
    let markdown_lines = markdown.lines().count();
    let yaml_share = yaml_lines as f64 / (yaml_lines + markdown_lines) as f64;
    assert!(
        (0.10..=0.50).contains(&yaml_share),
        "expected the front matter YAML to be between a tenth and half of the lines, \
         got {yaml_lines} YAML lines and {markdown_lines} Markdown lines"
    );
}

#[test]
fn big_benchmark_prints_table_and_json_artifact() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/big.py")
        .args([
            "--target-bytes",
            "12000",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run big benchmark script: {err}"));

    assert!(
        output.status.success(),
        "big benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(
            "| file | formatter | status | median_seconds | peak_rss_mb | mb_per_second |"
        )
    );
    assert!(stdout.contains("| big.md | yamark | ok |"));
    assert!(stdout.contains("| big.yaml | yamark | ok |"));
    assert!(stdout.contains("| big-with-frontmatter.md | yamark | ok |"));

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    assert_eq!(data["schema_version"], 1);
    assert_eq!(data["benchmark"], "big-file-formatting");
    assert_eq!(data["corpus"]["kind"], "big-file");
    assert_eq!(data["corpus"]["files"], 3);
    assert_eq!(
        data["corpus"]["targets"],
        serde_json::json!(["big.md", "big.yaml", "big-with-frontmatter.md"])
    );

    let rows = data["results"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["formatter"], "yamark");
    assert_eq!(rows[0]["status"], "ok");
    assert!(rows[0]["median_seconds"].as_f64().unwrap() > 0.0);
    assert!(rows[0]["median_peak_rss_bytes"].as_u64().unwrap() > 0);
}

#[test]
fn big_benchmark_accepts_common_markdown_formatters() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    let bin_dir = dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let panache = bin_dir.join("panache");
    let mdformat = bin_dir.join("mdformat");
    let pymarkdown = bin_dir.join("pymarkdown");
    let markdownlint_cli2 = bin_dir.join("markdownlint-cli2");
    let dprint = bin_dir.join("dprint");
    let deno = bin_dir.join("deno");
    write_fake_markdown_formatter(&panache, "panache-test");
    write_fake_markdown_formatter(&mdformat, "mdformat-test");
    write_fake_markdown_formatter(&pymarkdown, "pymarkdown-test");
    write_fake_markdown_formatter(&markdownlint_cli2, "markdownlint-cli2-test");
    write_fake_markdown_dprint(&dprint);
    write_fake_markdown_formatter(&deno, "deno-test");

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(python())
        .arg("tools/bench/big.py")
        .args([
            "--target-bytes",
            "12000",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "panache,mdformat,pymarkdown,markdownlint-cli2,dprint-markdown,deno-fmt",
            "--out-dir",
        ])
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .env("PATH", path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run big benchmark script: {err}"));

    assert!(
        output.status.success(),
        "big benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    assert_eq!(
        data["selected_formatters"],
        serde_json::json!([
            "panache",
            "mdformat",
            "pymarkdown",
            "markdownlint-cli2",
            "dprint-markdown",
            "deno-fmt"
        ])
    );

    let rows = data["results"].as_array().unwrap();
    assert_eq!(rows.len(), 18);
    for name in [
        "panache",
        "mdformat",
        "pymarkdown",
        "markdownlint-cli2",
        "dprint-markdown",
    ] {
        let markdown = rows
            .iter()
            .find(|row| row["formatter"] == name && row["file"] == "big.md")
            .unwrap();
        let yaml = rows
            .iter()
            .find(|row| row["formatter"] == name && row["file"] == "big.yaml")
            .unwrap();
        let frontmatter = rows
            .iter()
            .find(|row| row["formatter"] == name && row["file"] == "big-with-frontmatter.md")
            .unwrap();
        assert_eq!(markdown["status"], "ok");
        assert_eq!(yaml["status"], "skipped");
        assert_eq!(frontmatter["status"], "ok");
    }
    for file in ["big.md", "big.yaml", "big-with-frontmatter.md"] {
        let row = rows
            .iter()
            .find(|row| row["formatter"] == "deno-fmt" && row["file"] == file)
            .unwrap();
        assert_eq!(row["status"], "ok");
    }
}

#[test]
fn yaml_benchmark_agent_summary_prints_bounded_report() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--agent-summary",
            "--corpus",
            "yaml",
            "--files",
            "1",
            "--items",
            "1",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = stdout.lines().collect::<Vec<_>>();
    assert!(lines.len() <= 6, "agent summary was too verbose:\n{stdout}");
    assert!(stdout.contains("status: ok"));
    assert!(stdout.contains("artifact: "));
    assert!(stdout.contains("current: "));
    assert!(stdout.contains("previous: "));
    assert!(stdout.contains("delta: "));
    assert!(stdout.contains("log: "));
    assert!(!stdout.contains("yamark: "));

    let log_path = lines
        .iter()
        .find_map(|line| line.strip_prefix("log: "))
        .expect("agent summary should include a log path");
    assert!(std::path::Path::new(log_path).is_file());

    let artifacts = fs::read_dir(&artifact_dir).unwrap().count();
    assert_eq!(artifacts, 1);
}

#[test]
fn yaml_benchmark_agent_summary_ignores_previous_different_corpus_shape() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    fs::create_dir_all(&artifact_dir).unwrap();
    fs::write(
        artifact_dir.join("1111111111111111111111111111111111111111-mixed-node.json"),
        r#"{
  "schema_version": 1,
  "benchmark": "yaml-formatting",
  "run_id": "seed",
  "created_at": "2999-01-01T00:00:00Z",
  "git": {
    "commit": "1111111111111111111111111111111111111111",
    "short_commit": "111111111111",
    "commit_time": "2999-01-01T00:00:00Z",
    "tree": "2222222222222222222222222222222222222222",
    "dirty": false
  },
  "host": {
    "system": "Darwin",
    "machine": "arm64",
    "python": "3.13.0"
  },
  "corpus": {
    "kind": "yaml",
    "shape": "mixed-node",
    "files": 1,
    "items_per_file": 1,
    "bytes": 100
  },
  "invocation": {
    "unit": "per-file",
    "targets": ["service-00/config-0000.yaml"]
  },
  "operation": "write",
  "formatting_options": {
    "width_profile": "default",
    "yamark_options": []
  },
  "selected_formatters": ["yamark"],
  "results": [
    {
      "formatter": "yamark",
      "status": "ok",
      "command": "yamark format TARGET",
      "invocation": "per-file",
      "operation": "write",
      "width_profile": "default",
      "files": 1,
      "bytes": 100,
      "output_files": 1,
      "output_bytes": 100,
      "changed_files": 1,
      "would_change_files": 0,
      "output_hash": "seed",
      "warmups": 0,
      "reps": 1,
      "repetitions": [0.1],
      "user_seconds": [0.1],
      "sys_seconds": [0.0],
      "median_seconds": 0.1,
      "mean_seconds": 0.1,
      "median_user_seconds": 0.1,
      "median_sys_seconds": 0.0,
      "mb_per_second": 1.0
    }
  ]
}
"#,
    )
    .unwrap();

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--agent-summary",
            "--corpus",
            "yaml",
            "--files",
            "1",
            "--items",
            "1",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("previous: none"), "{stdout}");
    assert!(!stdout.contains("111111111111"), "{stdout}");
}

#[test]
fn yaml_benchmark_removes_generated_corpus_by_default() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--files",
            "1",
            "--items",
            "1",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let retained = fs::read_dir(&out_dir).unwrap().count();
    assert_eq!(retained, 0);
}

#[test]
fn yaml_benchmark_keep_corpus_retains_generated_corpus() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--keep-corpus",
            "--corpus",
            "yaml",
            "--files",
            "1",
            "--items",
            "1",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let run_dirs = fs::read_dir(&out_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    assert_eq!(run_dirs.len(), 1);
    assert!(run_dirs[0].join("corpus").is_dir());
}

#[test]
fn yaml_benchmark_can_record_directory_invocation() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    assert_eq!(data["invocation"]["unit"], "directory");
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["invocation"], "directory");
    assert!(row["command"].as_str().unwrap().contains("ROOT"));
    assert_eq!(row["output_files"], 2);
    assert_eq!(row["changed_files"], 2);
}

#[test]
fn yaml_benchmark_can_record_check_operation_and_width_profile() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--operation",
            "check",
            "--width-profile",
            "flow-preserve-wide",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "yamark",
            "--skip-yamark-build",
            "--yamark-bin",
        ])
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    assert_eq!(data["operation"], "check");
    assert_eq!(
        data["formatting_options"]["width_profile"],
        "flow-preserve-wide"
    );
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["operation"], "check");
    assert_eq!(row["changed_files"], 0);
    assert_eq!(row["would_change_files"], 2);
}

#[test]
fn yaml_benchmark_merges_invocations_for_one_commit_artifact() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");

    for invocation in ["per-file", "directory"] {
        let output = Command::new(python())
            .arg("tools/bench/run.py")
            .args([
                "--corpus",
                "yaml",
                "--invocation",
                invocation,
                "--files",
                "2",
                "--items",
                "2",
                "--reps",
                "1",
                "--warmups",
                "0",
                "--tools",
                "yamark",
                "--skip-yamark-build",
                "--yamark-bin",
            ])
            .arg(assert_cmd::cargo::cargo_bin("yamark"))
            .arg("--out-dir")
            .arg(&out_dir)
            .arg("--artifact-dir")
            .arg(&artifact_dir)
            .output()
            .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

        assert!(
            output.status.success(),
            "benchmark failed for {invocation}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let artifacts = fs::read_dir(&artifact_dir).unwrap().count();
    assert_eq!(artifacts, 1);
    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    let mut invocations = data["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["invocation"].as_str().unwrap())
        .collect::<Vec<_>>();
    invocations.sort();
    assert_eq!(invocations, ["directory", "per-file"]);
}

#[test]
fn r_summary_reads_yaml_benchmark_json_directory() {
    let dir = tempdir().unwrap();
    let artifact_dir = dir.path().join("artifacts");
    fs::create_dir(&artifact_dir).unwrap();
    fs::write(
        artifact_dir.join("abc1234.json"),
        r#"{
  "schema_version": 1,
  "benchmark": "yaml-formatting",
  "run_id": "20260519T000000000000Z",
  "git": {
    "commit": "abc1234",
    "short_commit": "abc1234",
    "commit_time": "2026-05-19T00:00:00Z",
    "dirty": false
  },
  "corpus": {
    "kind": "yaml",
    "files": 2,
    "items_per_file": 2,
    "bytes": 2048
  },
  "invocation": {
    "unit": "per-file"
  },
  "results": [
    {
      "formatter": "yamark",
      "status": "ok",
      "command": "yamark format TARGET",
      "invocation": "per-file",
      "changed_files": 2,
      "output_bytes": 2048,
      "repetitions": [0.010, 0.011, 0.012],
      "median_seconds": 0.011,
      "mean_seconds": 0.011,
      "mb_per_second": 186.18
    },
    {
      "formatter": "prettier",
      "status": "skipped",
      "reason": "not installed"
    }
  ]
}
"#,
    )
    .unwrap();

    let output = Command::new("Rscript")
        .arg("tools/bench/summarize.R")
        .arg("--input-dir")
        .arg(&artifact_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run Rscript: {err}"));

    assert!(
        output.status.success(),
        "R summary failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("commit"));
    assert!(stdout.contains("formatter"));
    assert!(stdout.contains("invocation"));
    assert!(stdout.contains("yamark"));
    assert!(stdout.contains("width_profile"));
    assert!(stdout.contains("186.18"));
    assert!(!stdout.contains("prettier"));
}

#[test]
fn r_summary_defaults_to_recent_yamark_per_file_results() {
    let dir = tempdir().unwrap();
    let artifact_dir = dir.path().join("artifacts");
    fs::create_dir(&artifact_dir).unwrap();
    write_summary_fixture(
        &artifact_dir,
        "old.json",
        "old000000000",
        "2026-05-19T00:00:00Z",
        0.030,
    );
    write_summary_fixture(
        &artifact_dir,
        "new.json",
        "new000000000",
        "2026-05-20T00:00:00Z",
        0.020,
    );

    let output = Command::new("Rscript")
        .arg("tools/bench/summarize.R")
        .arg("--input-dir")
        .arg(&artifact_dir)
        .arg("--limit-commits")
        .arg("1")
        .output()
        .unwrap_or_else(|err| panic!("failed to run Rscript: {err}"));

    assert!(
        output.status.success(),
        "R summary failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("new000000000"));
    assert!(stdout.contains("per-file"));
    assert!(stdout.contains("yamark"));
    assert!(!stdout.contains("old000000000"));
    assert!(!stdout.contains("dprint-yaml"));
}

#[test]
fn yaml_benchmark_records_panache_dprint_and_pretty_yaml_rows() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    let bin_dir = dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let panache = bin_dir.join("panache");
    let dprint = bin_dir.join("dprint");
    let pretty_yaml = bin_dir.join("pretty-yaml-driver");
    write_executable(
        &panache,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "panache-test"
  exit 0
fi
find "$3" -name '*.qmd' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
"#,
    );
    write_executable(
        &dprint,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "dprint-test"
  exit 0
fi
if [ "$1" = "output-resolved-config" ]; then
  exit 0
fi
target=
for arg do
  target="$arg"
done
if [ "$target" = "." ]; then
  find . -name '*.yaml' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
else
  perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' "$target"
fi
"#,
    );
    write_executable(
        &pretty_yaml,
        r#"#!/bin/sh
find "$1" -name '*.yaml' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
"#,
    );

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "per-file",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "panache-yaml,dprint-yaml,pretty-yaml",
            "--out-dir",
        ])
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .env("PATH", path)
        .env("YAMARK_PRETTY_YAML_DRIVER", &pretty_yaml)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    let rows = data["results"].as_array().unwrap();
    let names = rows
        .iter()
        .map(|row| row["formatter"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, ["panache-yaml", "dprint-yaml", "pretty-yaml"]);
    for row in rows {
        assert_eq!(row["status"], "ok");
        assert_eq!(row["invocation"], "per-file");
        assert_eq!(row["output_files"], 2);
        assert_eq!(row["changed_files"], 2);
    }
}

#[test]
fn yaml_benchmark_records_directory_rows_for_wrapped_formatters() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    let bin_dir = dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let panache = bin_dir.join("panache");
    let dprint = bin_dir.join("dprint");
    let pretty_yaml = bin_dir.join("pretty-yaml-driver");
    write_fake_panache(&panache);
    write_fake_dprint(&dprint);
    write_fake_pretty_yaml(&pretty_yaml);

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "panache-yaml,dprint-yaml,pretty-yaml",
            "--out-dir",
        ])
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .env("PATH", path)
        .env("YAMARK_PRETTY_YAML_DRIVER", &pretty_yaml)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    for row in data["results"].as_array().unwrap() {
        assert_eq!(row["status"], "ok");
        assert_eq!(row["invocation"], "directory");
        assert_eq!(row["output_files"], 2);
        assert_eq!(row["changed_files"], 2);
    }
}

#[test]
fn yaml_benchmark_records_deno_fmt_rows() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    let bin_dir = dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let deno = bin_dir.join("deno");
    write_fake_deno(&deno);

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "deno-fmt",
            "--out-dir",
        ])
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .env("PATH", path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["formatter"], "deno-fmt");
    assert_eq!(row["status"], "ok");
    assert_eq!(row["invocation"], "directory");
    assert_eq!(row["output_files"], 2);
    assert_eq!(row["changed_files"], 2);
}

#[test]
fn yaml_benchmark_records_py_yaml12_rows() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    let module_dir = dir.path().join("python");
    fs::create_dir(&module_dir).unwrap();
    fs::write(
        module_dir.join("yaml12.py"),
        r#"from pathlib import Path

def read_yaml(path):
    return Path(path).read_text()

def write_yaml(value, path):
    Path(path).write_text(
        value
        .replace("probe:    [one,two]", "probe: [one, two]")
        .replace("name:    ", "name: ")
    )
"#,
    )
    .unwrap();

    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "1",
            "--warmups",
            "0",
            "--tools",
            "py-yaml12",
            "--out-dir",
        ])
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .env("PYTHONPATH", module_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["formatter"], "py-yaml12");
    assert_eq!(row["status"], "ok");
    assert_eq!(row["invocation"], "directory");
    assert!(
        row["command"]
            .as_str()
            .unwrap()
            .contains("format_yaml12.py")
    );
    assert_eq!(row["output_files"], 2);
    assert_eq!(row["changed_files"], 2);
}

#[test]
fn yaml_benchmark_rejects_later_repetition_no_ops() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("raw");
    let artifact_dir = dir.path().join("artifacts");
    let bin_dir = dir.path().join("bin");
    let state = dir.path().join("state");
    fs::create_dir(&bin_dir).unwrap();

    let deno = bin_dir.join("deno");
    write_fake_deno_with_later_no_op(&deno);

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(python())
        .arg("tools/bench/run.py")
        .args([
            "--corpus",
            "yaml",
            "--invocation",
            "directory",
            "--files",
            "2",
            "--items",
            "2",
            "--reps",
            "2",
            "--warmups",
            "0",
            "--tools",
            "deno-fmt",
            "--out-dir",
        ])
        .arg(&out_dir)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .env("PATH", path)
        .env("YAMARK_FAKE_DENO_STATE", state)
        .output()
        .unwrap_or_else(|err| panic!("failed to run benchmark script: {err}"));

    assert!(
        output.status.success(),
        "benchmark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = fs::read_dir(&artifact_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let data: Value = serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
    let row = &data["results"].as_array().unwrap()[0];
    assert_eq!(row["formatter"], "deno-fmt");
    assert_eq!(row["status"], "failed");
    assert!(
        row["reason"]
            .as_str()
            .unwrap()
            .contains("repetition 2 changed 0 of 2 files")
    );
}

#[test]
fn yamark_flow_heavy_yaml_formatting_scales_near_linearly() {
    let small = measure_yamark_flow_heavy_yaml(400);
    let large = measure_yamark_flow_heavy_yaml(1600);
    let small_nanos = small.as_nanos().max(1);
    let large_nanos = large.as_nanos();

    assert!(
        large_nanos <= small_nanos * 6,
        "flow-heavy YAML formatting should scale near-linearly: \
         400 items took {small:?}, 1600 items took {large:?}"
    );
}

fn write_summary_fixture(
    artifact_dir: &std::path::Path,
    file_name: &str,
    commit: &str,
    commit_time: &str,
    yamark_seconds: f64,
) {
    let content = format!(
        r#"{{
  "schema_version": 1,
  "benchmark": "yaml-formatting",
  "run_id": "{commit}",
  "git": {{
    "commit": "{commit}",
    "short_commit": "{commit}",
    "commit_time": "{commit_time}",
    "dirty": false
  }},
  "corpus": {{
    "kind": "yaml",
    "files": 2,
    "items_per_file": 2,
    "bytes": 2048
  }},
  "invocation": {{
    "unit": "per-file"
  }},
  "results": [
    {{
      "formatter": "yamark",
      "status": "ok",
      "command": "yamark format TARGET",
      "invocation": "per-file",
      "operation": "write",
      "width_profile": "default",
      "changed_files": 2,
      "output_bytes": 2048,
      "repetitions": [{yamark_seconds}],
      "median_seconds": {yamark_seconds},
      "mean_seconds": {yamark_seconds},
      "mb_per_second": 1.0
    }},
    {{
      "formatter": "dprint-yaml",
      "status": "ok",
      "command": "dprint fmt TARGET",
      "invocation": "per-file",
      "operation": "write",
      "width_profile": "default",
      "changed_files": 2,
      "output_bytes": 2048,
      "repetitions": [0.040],
      "median_seconds": 0.040,
      "mean_seconds": 0.040,
      "mb_per_second": 0.5
    }}
  ]
}}
"#
    );
    fs::write(artifact_dir.join(file_name), content).unwrap();
}

fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn write_fake_panache(path: &std::path::Path) {
    write_executable(
        path,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "panache-test"
  exit 0
fi
find "$3" -name '*.qmd' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
"#,
    );
}

fn write_fake_dprint(path: &std::path::Path) {
    write_executable(
        path,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "dprint-test"
  exit 0
fi
if [ "$1" = "output-resolved-config" ]; then
  exit 0
fi
target=
for arg do
  target="$arg"
done
if [ "$target" = "." ]; then
  find . -name '*.yaml' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
else
  perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' "$target"
fi
"#,
    );
}

fn write_fake_markdown_formatter(path: &std::path::Path, version: &str) {
    write_executable(
        path,
        &format!(
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "{version}"
  exit 0
fi
target=
for arg do
  target="$arg"
done
if [ -d "$target" ]; then
  find "$target" -name '*.md' -exec perl -0pi -e 's/Generated Markdown Benchmark Document/Formatted Markdown Benchmark Document/g' {{}} +
else
  perl -0pi -e 's/Generated Markdown Benchmark Document/Formatted Markdown Benchmark Document/g' "$target"
fi
"#
        ),
    );
}

fn write_fake_markdown_dprint(path: &std::path::Path) {
    write_executable(
        path,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "dprint-test"
  exit 0
fi
if [ "$1" = "output-resolved-config" ]; then
  exit 0
fi
target=
for arg do
  target="$arg"
done
if [ "$target" = "." ]; then
  find . -name '*.md' -exec perl -0pi -e 's/Generated Markdown Benchmark Document/Formatted Markdown Benchmark Document/g' {} +
elif [ -d "$target" ]; then
  find "$target" -name '*.md' -exec perl -0pi -e 's/Generated Markdown Benchmark Document/Formatted Markdown Benchmark Document/g' {} +
else
  perl -0pi -e 's/Generated Markdown Benchmark Document/Formatted Markdown Benchmark Document/g' "$target"
fi
"#,
    );
}

fn write_fake_pretty_yaml(path: &std::path::Path) {
    write_executable(
        path,
        r#"#!/bin/sh
find "$1" -name '*.yaml' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
"#,
    );
}

fn write_fake_deno(path: &std::path::Path) {
    write_executable(
        path,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "deno-test"
  exit 0
fi
target=
for arg do
  target="$arg"
done
if [ -d "$target" ]; then
  find "$target" -name '*.yaml' -exec perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' {} +
else
  perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g; s/name:\s+/name: /g' "$target"
fi
"#,
    );
}

fn write_fake_deno_with_later_no_op(path: &std::path::Path) {
    write_executable(
        path,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "deno-test"
  exit 0
fi
target=
for arg do
  target="$arg"
done
if [ -f "$target" ]; then
  perl -0pi -e 's/probe:\s+\[one,two\]/probe: [one, two]/g' "$target"
  exit 0
fi
if [ -f "$YAMARK_FAKE_DENO_STATE" ]; then
  exit 0
fi
touch "$YAMARK_FAKE_DENO_STATE"
find "$target" -name '*.yaml' -exec perl -0pi -e 's/name:\s+/name: /g' {} +
"#,
    );
}

fn measure_yamark_flow_heavy_yaml(items: usize) -> Duration {
    let dir = tempdir().unwrap();
    let path = dir.path().join("flow-heavy.yaml");
    fs::write(&path, render_flow_heavy_yaml(items)).unwrap();

    let started = Instant::now();
    let output = Command::new(assert_cmd::cargo::cargo_bin("yamark"))
        .arg("format")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run yamark: {err}"));
    let elapsed = started.elapsed();

    assert!(
        output.status.success(),
        "yamark failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let formatted = fs::read_to_string(&path).unwrap();
    assert!(formatted.contains("ports: [8000, 9000]"));
    elapsed
}

fn render_flow_heavy_yaml(items: usize) -> String {
    let mut out = String::from(
        "name:    flow-heavy\n\
         enabled: true\n\
         labels: {team: platform,region: us-0,tier: backend}\n\
         settings:\n",
    );
    for index in 0..items {
        out.push_str(&format!(
            "  item_{index:04}: {{name: worker-{index:04},replicas: {},\
             ports: [8000,9000],env: {{LOG_LEVEL: info,FEATURE_FLAG: false}},\
             resources: {{cpu: {}m,memory: {}Mi}},\
             dependencies: [service-{:04},service-{:04}]}}\n",
            1 + index % 9,
            100 + index % 20,
            128 + (index % 12) * 32,
            (index + 1) % 50,
            (index + 7) % 50,
        ));
    }
    out
}
