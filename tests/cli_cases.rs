use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use assert_cmd::Command;

#[derive(Default)]
struct Case {
    args: String,
    stdin: String,
    stdout: String,
    stderr: String,
    status: i32,
    git_fixture: Option<serde_json::Value>,
    git_status: Option<String>,
    git_index: Option<BTreeMap<String, String>>,
    git_worktree: Option<BTreeMap<String, String>>,
}

#[test]
fn cli_cases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let mut cases = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("case"))
        .collect::<Vec<_>>();
    cases.sort();

    for path in cases {
        let case = parse_case(&fs::read_to_string(&path).unwrap());
        let repo = case.git_fixture.as_ref().map(create_git_fixture);
        let mut command = Command::cargo_bin("yamark").unwrap();
        if let Some(repo) = &repo {
            command.current_dir(repo.path());
        }
        command.args(split_args(&case.args));
        command.write_stdin(case.stdin);
        let assert = command.assert().code(case.status);
        let output = assert.get_output();
        assert_eq!(
            redact_repo(&output.stdout, repo.as_ref()),
            case.stdout,
            "stdout mismatch in {}",
            path.display()
        );
        assert_eq!(
            redact_repo(&output.stderr, repo.as_ref()),
            case.stderr,
            "stderr mismatch in {}",
            path.display()
        );
        if let Some(expected) = case.git_status {
            Command::new("git")
                .current_dir(repo.as_ref().expect("git-status needs git-fixture").path())
                .args(["status", "--porcelain"])
                .assert()
                .success()
                .stdout(expected)
                .stderr("");
        }
        if let Some(files) = case.git_index {
            let repo = repo.as_ref().expect("git-index needs git-fixture");
            for (name, expected) in files {
                Command::new("git")
                    .current_dir(repo.path())
                    .arg("show")
                    .arg(format!(":{name}"))
                    .assert()
                    .success()
                    .stdout(expected)
                    .stderr("");
            }
        }
        if let Some(files) = case.git_worktree {
            let repo = repo.as_ref().expect("git-worktree needs git-fixture");
            for (name, expected) in files {
                assert_eq!(
                    fs::read(repo.path().join(&name)).unwrap(),
                    expected.as_bytes(),
                    "working-tree bytes mismatch in {} for {name}",
                    path.display()
                );
            }
        }
    }
}

fn create_git_fixture(fixture: &serde_json::Value) -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(repo.path())
            .args(args)
            .assert()
            .success();
    };
    git(&["init", "-q"]);
    git(&["config", "user.name", "CLI Case"]);
    git(&["config", "user.email", "cli-case@example.test"]);
    git(&["config", "commit.gpgSign", "false"]);
    for (key, value) in fixture["config"].as_object().unwrap() {
        git(&["config", key, value.as_str().unwrap()]);
    }
    for (name, contents) in fixture["files"].as_object().unwrap() {
        let path = repo.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents.as_str().unwrap()).unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "fixture"]);
    repo
}

fn redact_repo(bytes: &[u8], repo: Option<&tempfile::TempDir>) -> String {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let Some(repo) = repo else { return text };
    let root = repo.path().canonicalize().unwrap();
    let root = root.to_str().unwrap().trim_start_matches(r"\\?\");
    text.replace(root, "{{repo}}")
        .replace(&root.replace('\\', "/"), "{{repo}}")
}

fn parse_case(input: &str) -> Case {
    let mut case = Case {
        status: 0,
        ..Case::default()
    };
    let mut section = String::new();
    let mut body = String::new();

    for line in input.lines() {
        if let Some(name) = line.strip_prefix("-- ") {
            flush(&mut case, &section, &body);
            section = name.trim().to_owned();
            body.clear();
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    flush(&mut case, &section, &body);
    case
}

fn flush(case: &mut Case, section: &str, body: &str) {
    let value = body.strip_suffix('\n').unwrap_or(body);
    match section {
        "args" => case.args = value.to_owned(),
        "stdin" => case.stdin = body.to_owned(),
        "stdout" => case.stdout = body.to_owned(),
        "stderr" => case.stderr = body.to_owned(),
        "status" => case.status = value.trim().parse().unwrap(),
        "git-fixture" => case.git_fixture = Some(serde_json::from_str(value).unwrap()),
        "git-status" => case.git_status = Some(body.to_owned()),
        "git-index" => case.git_index = Some(serde_json::from_str(value).unwrap()),
        "git-worktree" => case.git_worktree = Some(serde_json::from_str(value).unwrap()),
        "" => {}
        other => panic!("unknown case section {other}"),
    }
}

fn split_args(input: &str) -> Vec<String> {
    input.split_whitespace().map(ToOwned::to_owned).collect()
}
