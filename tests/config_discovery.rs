use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn relative_file_and_directory_paths_discover_ancestor_config() {
    for paths in [&["input.md"][..], &["input.md", "second.md"], &["."]] {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(
            dir.path().join("yamark.toml"),
            "[format]\nwrap = \"sentence\"\n",
        )
        .unwrap();
        for path in ["input.md", "second.md"] {
            fs::write(nested.join(path), "First sentence. Second sentence.\n").unwrap();
        }
        Command::cargo_bin("yamark")
            .unwrap()
            .current_dir(&nested)
            .arg("format")
            .args(paths)
            .assert()
            .success()
            .stderr("");
        assert_eq!(
            fs::read_to_string(nested.join("input.md")).unwrap(),
            "First sentence.\nSecond sentence.\n"
        );
    }
}
