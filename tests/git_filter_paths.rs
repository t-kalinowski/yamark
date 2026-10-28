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

// APFS rejects these names; Linux can materialize the complete checkout.
#[cfg(target_os = "linux")]
#[test]
fn git_filter_preserves_native_paths_through_adopt_check_and_join() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    for quote_paths in ["true", "false"] {
        let dir = tempdir().unwrap();
        let markdown_paths = [
            OsString::from("regular.md"),
            OsString::from_vec(b"post-\xff.md".to_vec()),
        ];
        let binary_path = OsString::from_vec(b"unrelated-\xfe.bin".to_vec());
        let binary_contents = b"\0\xff\n\x80\r";
        let canonical = b"First sentence.\nSecond sentence?\n";
        let wrapped = b"First sentence. Second sentence?\n";
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
        let index_blob = |path: &OsString| {
            let mut spec = OsString::from(":");
            spec.push(path);
            Command::new("git")
                .current_dir(dir.path())
                .arg("show")
                .arg(spec)
                .assert()
                .success()
                .get_output()
                .stdout
                .clone()
        };
        let assert_files = |worktree: &[u8]| {
            for path in &markdown_paths {
                assert_eq!(index_blob(path), canonical);
                assert_eq!(fs::read(dir.path().join(path)).unwrap(), worktree);
            }
            assert_eq!(index_blob(&binary_path), binary_contents);
            assert_eq!(
                fs::read(dir.path().join(&binary_path)).unwrap(),
                binary_contents
            );
        };
        let check = || {
            Command::cargo_bin("yamark")
                .unwrap()
                .current_dir(dir.path())
                .args(["git-filter", "check", "--markdown-wrap-at-column", "32"])
                .assert()
                .success()
                .stdout("yamark Git filter check passed for 2 Markdown file(s).\n")
                .stderr("");
        };

        git(&["init", "-q"]);
        git(&["config", "user.name", "Git Filter Test"]);
        git(&["config", "user.email", "git-filter@example.test"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["config", "core.autocrlf", "false"]);
        git(&["config", "core.quotePath", quote_paths]);
        for path in &markdown_paths {
            fs::write(dir.path().join(path), wrapped).unwrap();
        }
        fs::write(dir.path().join(&binary_path), binary_contents).unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "fixture"]);

        Command::cargo_bin("yamark")
            .unwrap()
            .current_dir(dir.path())
            .args(["git-filter", "adopt", "--markdown-wrap-at-column", "32"])
            .assert()
            .success()
            .stderr("");
        assert_files(wrapped);
        git(&["commit", "-q", "-m", "adopt"]);
        check();

        Command::cargo_bin("yamark")
            .unwrap()
            .current_dir(dir.path())
            .args(["git-filter", "teardown"])
            .assert()
            .success()
            .stderr("");
        git(&["checkout-index", "--force", "--all"]);
        assert_files(canonical);
        assert_eq!(git(&["status", "--porcelain"]), b"");

        Command::cargo_bin("yamark")
            .unwrap()
            .current_dir(dir.path())
            .args(["git-filter", "join", "--markdown-wrap-at-column", "32"])
            .assert()
            .success()
            .stderr("");
        assert_files(wrapped);
        check();
        assert_eq!(git(&["status", "--porcelain"]), b"");
    }
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
