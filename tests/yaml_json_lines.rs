use std::path::PathBuf;
use std::process::Command;

#[test]
fn json_lines_records_are_identical_after_yaml_formatting() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("Rscript")
        .arg(root.join("tests").join("yaml_json_lines.R"))
        .arg(env!("CARGO_BIN_EXE_yamark"))
        .output()
        .unwrap_or_else(|err| panic!("failed to run Rscript: {err}"));

    assert!(
        output.status.success(),
        "JSON Lines semantic test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
