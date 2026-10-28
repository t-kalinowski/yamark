use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn diff_context_produces_applicable_patches() {
    for gap in 0..=10 {
        let dir = tempdir().unwrap();
        let context = (0..gap)
            .map(|index| format!("context{index}: unchanged\n"))
            .collect::<String>();
        let input = format!("first:  changed\n{context}last:  changed\n");
        let expected = format!("first: changed\n{context}last: changed\n");
        fs::write(dir.path().join("input.yaml"), &input).unwrap();
        let diff = Command::cargo_bin("yamark")
            .unwrap()
            .current_dir(dir.path())
            .args(["format", "--diff", "input.yaml"])
            .assert()
            .code(1)
            .get_output()
            .stdout
            .clone();
        Command::new("git")
            .current_dir(dir.path())
            // Keep patch application byte-exact regardless of the user's Git settings.
            .args(["-c", "core.autocrlf=false", "apply", "-"])
            .write_stdin(diff)
            .assert()
            .success();
        assert_eq!(
            fs::read_to_string(dir.path().join("input.yaml")).unwrap(),
            expected,
            "gap {gap}"
        );
    }
}
