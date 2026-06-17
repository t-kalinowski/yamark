use std::path::PathBuf;
use std::process::Command;

#[test]
fn python_cli_cases_pass() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("uv")
        .arg("run")
        .arg("--no-project")
        .arg("--script")
        .arg(manifest_dir.join("external-tests").join("run.py"))
        .arg("--serial")
        .arg("--suite")
        .arg("cli")
        .arg("--yamark-bin")
        .arg(assert_cmd::cargo::cargo_bin("yamark"))
        .output()
        .unwrap_or_else(|err| panic!("failed to run uv cli tests: {err}"));

    assert!(
        output.status.success(),
        "uv cli tests failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
