use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn git_filter_adopts_quoted_names() {
    check_git_filter_paths(&["café.md", "with space.md"]);
}

// Windows filenames cannot contain line feeds, tabs, quotes, or backslashes.
#[cfg(unix)]
#[test]
fn git_filter_adopts_names_with_record_delimiters() {
    check_git_filter_paths(&[
        "line\nbreak.md",
        "tab\tname.md",
        "quote\"name.md",
        "back\\slash.md",
    ]);
}

fn check_git_filter_paths(paths: &[&str]) {
    let dir = tempdir().unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(dir.path())
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone()
    };
    git(&["init", "-q"]);
    git(&["config", "user.name", "Git Filter Test"]);
    git(&["config", "user.email", "git-filter@example.test"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.quotePath", "true"]);
    for path in paths {
        fs::write(dir.path().join(path), "First sentence. Second sentence?\n").unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "fixture"]);
    Command::cargo_bin("yamark")
        .unwrap()
        .current_dir(dir.path())
        .args(["git-filter", "adopt", "--markdown-wrap-at-column", "32"])
        .assert()
        .success()
        .stderr("");
    for path in paths {
        assert_eq!(
            git(&["show", &format!(":{path}")]),
            b"First sentence.\nSecond sentence?\n"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(path)).unwrap(),
            "First sentence. Second sentence?\n"
        );
    }
}
