use std::path::PathBuf;
use std::process::Command;

#[test]
fn generated_yaml_values_are_identical_after_formatting() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("Rscript")
        .arg(root.join("tests").join("yaml_fuzz.R"))
        .arg(env!("CARGO_BIN_EXE_yamark"))
        .output()
        .unwrap_or_else(|err| panic!("failed to run Rscript: {err}"));

    assert!(
        output.status.success(),
        "YAML fuzz test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
