#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct VersionFixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
}

impl VersionFixture {
    fn new(pyproject_version: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("scripts")).unwrap();
        fs::create_dir_all(repo.join("editors/vscode")).unwrap();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/set-version.py"),
            repo.join("scripts/set-version.py"),
        )
        .unwrap();
        fs::write(
            repo.join("Cargo.toml"),
            concat!(
                "[package]\n",
                "name = \"yamark\"\n",
                "version = \"0.3.0+dev\"\n\n",
                "[dependencies]\n",
                "helper = { version = \"0.3.0+dev\" }\n",
            ),
        )
        .unwrap();
        fs::write(
            repo.join("Cargo.lock"),
            concat!(
                "version = 4\n\n",
                "[[package]]\n",
                "name = \"helper\"\n",
                "version = \"0.3.0+dev\"\n\n",
                "[[package]]\n",
                "name = \"yamark\"\n",
                "version = \"0.3.0+dev\"\n",
            ),
        )
        .unwrap();
        fs::write(
            repo.join("pyproject.toml"),
            format!("[project]\nname = \"yamark\"\nversion = \"{pyproject_version}\"\n"),
        )
        .unwrap();
        fs::write(
            repo.join("uv.lock"),
            concat!(
                "version = 1\n\n",
                "[[package]]\n",
                "name = \"helper\"\n",
                "version = \"0.3.0+dev\"\n\n",
                "[[package]]\n",
                "name = \"yamark\"\n",
                "version = \"0.3.0+dev\"\n",
            ),
        )
        .unwrap();
        fs::write(
            repo.join("editors/vscode/package.json"),
            concat!(
                "{\n",
                "  \"name\": \"yamark\",\n",
                "  \"version\": \"0.3.0+dev\",\n",
                "  \"fixtureVersion\": \"0.3.0+dev\"\n",
                "}\n",
            ),
        )
        .unwrap();

        Self { _temp: temp, repo }
    }

    fn run(&self, version: &str) -> Output {
        Command::new("python3")
            .current_dir(&self.repo)
            .args(["scripts/set-version.py", version])
            .output()
            .unwrap()
    }
}

#[test]
fn set_version_updates_exactly_the_five_yamark_versions() {
    let fixture = VersionFixture::new("0.3.0+dev");

    let output = fixture.run("0.4.0");

    assert_success(&output);
    let cargo = fs::read_to_string(fixture.repo.join("Cargo.toml")).unwrap();
    let cargo_lock = fs::read_to_string(fixture.repo.join("Cargo.lock")).unwrap();
    let pyproject = fs::read_to_string(fixture.repo.join("pyproject.toml")).unwrap();
    let uv_lock = fs::read_to_string(fixture.repo.join("uv.lock")).unwrap();
    let extension = fs::read_to_string(fixture.repo.join("editors/vscode/package.json")).unwrap();

    assert!(cargo.contains("name = \"yamark\"\nversion = \"0.4.0\""));
    assert!(cargo.contains("helper = { version = \"0.3.0+dev\" }"));
    assert!(cargo_lock.contains("name = \"yamark\"\nversion = \"0.4.0\""));
    assert!(cargo_lock.contains("name = \"helper\"\nversion = \"0.3.0+dev\""));
    assert!(pyproject.contains("name = \"yamark\"\nversion = \"0.4.0\""));
    assert!(uv_lock.contains("name = \"yamark\"\nversion = \"0.4.0\""));
    assert!(uv_lock.contains("name = \"helper\"\nversion = \"0.3.0+dev\""));
    assert!(extension.contains("\"version\": \"0.4.0\""));
    assert!(extension.contains("\"fixtureVersion\": \"0.3.0+dev\""));

    let output = fixture.run("0.4.0+dev");
    assert_success(&output);
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "uv.lock",
        "editors/vscode/package.json",
    ] {
        assert!(
            fs::read_to_string(fixture.repo.join(path))
                .unwrap()
                .contains("0.4.0+dev"),
            "{path} should contain the post-release version"
        );
    }
}

#[test]
fn set_version_rejects_unsynchronized_input_without_writing() {
    let fixture = VersionFixture::new("0.3.1+dev");
    let paths = [
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "uv.lock",
        "editors/vscode/package.json",
    ];
    let before = paths
        .iter()
        .map(|path| fs::read(fixture.repo.join(path)).unwrap())
        .collect::<Vec<_>>();

    let output = fixture.run("0.4.0");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("versions are out of sync"));
    for (path, expected) in paths.iter().zip(before) {
        assert_eq!(fs::read(fixture.repo.join(path)).unwrap(), expected);
    }
}

#[test]
fn set_version_enforces_release_then_matching_dev_transition() {
    let fixture = VersionFixture::new("0.3.0+dev");
    let paths = [
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "uv.lock",
        "editors/vscode/package.json",
    ];

    for target in ["0.3.0", "0.2.0", "0.4.0+dev"] {
        let before = paths
            .iter()
            .map(|path| fs::read(fixture.repo.join(path)).unwrap())
            .collect::<Vec<_>>();
        let output = fixture.run(target);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("development version must advance to a later stable version")
        );
        for (path, expected) in paths.iter().zip(before) {
            assert_eq!(fs::read(fixture.repo.join(path)).unwrap(), expected);
        }
    }

    assert_success(&fixture.run("0.4.0"));
    let before = paths
        .iter()
        .map(|path| fs::read(fixture.repo.join(path)).unwrap())
        .collect::<Vec<_>>();
    let output = fixture.run("0.5.0+dev");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("stable version can only move to 0.4.0+dev")
    );
    for (path, expected) in paths.iter().zip(before) {
        assert_eq!(fs::read(fixture.repo.join(path)).unwrap(), expected);
    }
}

#[test]
fn check_script_runs_the_ci_commands() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let bin = temp.path().join("bin");
    let log = temp.path().join("commands.log");
    fs::create_dir_all(repo.join("scripts")).unwrap();
    fs::create_dir_all(repo.join("editors/vscode")).unwrap();
    fs::create_dir_all(&bin).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/check.sh"),
        repo.join("scripts/check.sh"),
    )
    .unwrap();
    make_executable(&repo.join("scripts/check.sh"));
    for command in ["cargo", "uv"] {
        write_executable(
            &bin.join(command),
            &format!(
                "#!/bin/sh\nset -eu\nprintf '{command} %s\\n' \"$*\" >> \"$YAMARK_TEST_LOG\"\n"
            ),
        );
    }
    write_executable(
        &bin.join("npm"),
        "#!/bin/sh\nset -eu\nprintf 'npm %s %s\\n' \"$PWD\" \"$*\" >> \"$YAMARK_TEST_LOG\"\n",
    );

    let output = Command::new(repo.join("scripts/check.sh"))
        .current_dir(temp.path())
        .env("PATH", path_with_fake_bin(&bin))
        .env("YAMARK_TEST_LOG", &log)
        .output()
        .unwrap();

    assert_success(&output);
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        format!(
            "cargo fmt --check\n\
             cargo clippy --all-targets --all-features -- -D warnings\n\
             cargo test\n\
             uv run external-tests/run.py --serial\n\
             npm {} test\n",
            repo.join("editors/vscode").display(),
        )
    );
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    make_executable(path);
}

fn make_executable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn path_with_fake_bin(fake_bin: &Path) -> String {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(fake_bin.to_path_buf()).chain(std::env::split_paths(&current));
    std::env::join_paths(paths)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}
