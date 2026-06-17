use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;
use std::process::Command;

use tempfile::tempdir;

fn python() -> OsString {
    std::env::var_os("PYTHON").unwrap_or_else(|| OsString::from("python3"))
}

#[test]
fn yaml_test_suite_bootstrap_copies_generated_data_and_deletes_stale_files() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path().join("repo");
    let source_root = dir.path().join("source-yaml-test-suite");
    let dest_data = repo_root.join("tests/yaml-test-suite/data");

    fs::create_dir_all(&repo_root).unwrap();
    fs::write(
        repo_root.join("Cargo.toml"),
        "[package]\nname = \"yamark\"\n",
    )
    .unwrap();
    fs::create_dir_all(dest_data.join("STALE")).unwrap();
    fs::write(dest_data.join("STALE/remove.yaml"), "stale: true\n").unwrap();

    fs::create_dir_all(source_root.join("data/ABCD")).unwrap();
    fs::write(source_root.join("data/ABCD/in.yaml"), "a: 1\n").unwrap();
    symlink("ABCD", source_root.join("data/TAGS")).unwrap();
    fs::write(source_root.join("License"), "MIT License\n").unwrap();

    let output = Command::new(python())
        .arg("tools/bootstrap-yaml-test-suite-data.py")
        .arg("--repo-root")
        .arg(&repo_root)
        .arg("--source")
        .arg(&source_root)
        .output()
        .unwrap_or_else(|err| panic!("failed to run yaml-test-suite bootstrap: {err}"));

    assert!(
        output.status.success(),
        "yaml-test-suite bootstrap failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(dest_data.join("ABCD/in.yaml")).unwrap(),
        "a: 1\n"
    );
    assert!(!dest_data.join("TAGS").exists());
    assert!(!dest_data.join("STALE/remove.yaml").exists());
    assert_eq!(
        fs::read_to_string(repo_root.join("tests/yaml-test-suite/License")).unwrap(),
        "MIT License\n"
    );
}
