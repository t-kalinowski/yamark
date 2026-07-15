use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn uv_tool_install_from_wheel_exposes_yamark() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp = tempdir().unwrap();
    let wheel_dir = temp.path().join("wheels");
    let home_dir = temp.path().join("home");
    let cache_dir = temp.path().join("uv-cache");
    fs::create_dir(&wheel_dir).unwrap();
    fs::create_dir(&home_dir).unwrap();
    fs::create_dir(&cache_dir).unwrap();

    let build = Command::new("uv")
        .arg("build")
        .arg("--wheel")
        .arg("--out-dir")
        .arg(&wheel_dir)
        .current_dir(&manifest_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run uv build: {err}"));
    assert_success(&build, "uv build failed");

    let install = Command::new("uv")
        .arg("tool")
        .arg("install")
        .arg("--find-links")
        .arg(&wheel_dir)
        .arg("--no-index")
        .arg("--force")
        .arg("yamark")
        .env("HOME", &home_dir)
        .env("USERPROFILE", &home_dir)
        .env("UV_CACHE_DIR", &cache_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run uv tool install: {err}"));
    assert_success(&install, "uv tool install failed");

    let bin_dir = Command::new("uv")
        .arg("tool")
        .arg("dir")
        .arg("--bin")
        .env("HOME", &home_dir)
        .env("USERPROFILE", &home_dir)
        .env("UV_CACHE_DIR", &cache_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run uv tool dir: {err}"));
    assert_success(&bin_dir, "uv tool dir failed");

    let executable = PathBuf::from(String::from_utf8(bin_dir.stdout).unwrap().trim())
        .join(format!("yamark{}", std::env::consts::EXE_SUFFIX));
    let help = Command::new(&executable)
        .arg("--help")
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", executable.display()));
    assert_success(&help, "installed yamark --help failed");
    assert!(
        String::from_utf8_lossy(&help.stdout).contains("Usage: yamark <COMMAND>"),
        "installed yamark --help printed unexpected stdout:\n{}",
        String::from_utf8_lossy(&help.stdout)
    );
}
