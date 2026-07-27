#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn devbox_setup_installs_cli_then_extension() {
    let temp = tempfile::tempdir().unwrap();
    let fake_repo = temp.path().join("repo");
    let fake_bin = temp.path().join("bin");
    let order_log = temp.path().join("order.log");
    fs::create_dir_all(fake_repo.join("tools")).unwrap();
    fs::create_dir_all(fake_repo.join("editors/vscode/scripts")).unwrap();
    fs::create_dir_all(fake_repo.join("editors/vscode/bin")).unwrap();
    fs::create_dir_all(&fake_bin).unwrap();

    fs::copy(
        "tools/install-devbox.sh",
        fake_repo.join("tools/install-devbox.sh"),
    )
    .unwrap();
    fs::copy(
        "editors/vscode/scripts/install-local.sh",
        fake_repo.join("editors/vscode/scripts/install-local.sh"),
    )
    .unwrap();
    fs::copy(
        "editors/vscode/scripts/package-dev.sh",
        fake_repo.join("editors/vscode/scripts/package-dev.sh"),
    )
    .unwrap();
    make_executable(&fake_repo.join("tools/install-devbox.sh"));
    make_executable(&fake_repo.join("editors/vscode/scripts/install-local.sh"));
    make_executable(&fake_repo.join("editors/vscode/scripts/package-dev.sh"));

    write_executable(
        &fake_bin.join("cargo"),
        r#"#!/usr/bin/env bash
set -euo pipefail
printf "cargo %s\n" "$*" >> "$YAMARK_TEST_ORDER"
if [[ "$*" == "build --release" ]]; then
  mkdir -p target/release
  printf "unstripped\n" > target/release/yamark
  chmod +x target/release/yamark
  exit 0
fi
if [[ "$*" != "install --path ." ]]; then
  echo "unexpected cargo args: $*" >&2
  exit 41
fi
"#,
    );
    write_executable(
        &fake_bin.join("node"),
        r#"#!/usr/bin/env bash
set -euo pipefail
case "$2" in
  process.platform) echo linux ;;
  process.arch) echo x64 ;;
  *) exit 42 ;;
esac
"#,
    );
    write_executable(
        &fake_bin.join("strip"),
        r#"#!/usr/bin/env bash
set -euo pipefail
printf "strip\n" >> "$YAMARK_TEST_ORDER"
"#,
    );
    write_executable(
        &fake_bin.join("npx"),
        r#"#!/usr/bin/env bash
set -euo pipefail
printf "npx %s\n" "$*" >> "$YAMARK_TEST_ORDER"
out=""
while [[ "$#" -gt 0 ]]; do
  if [[ "$1" == "--out" ]]; then
    out="$2"
    break
  fi
  shift
done
mkdir -p "$(dirname "$out")"
touch "$out"
"#,
    );
    write_executable(
        &fake_bin.join("code"),
        r#"#!/usr/bin/env bash
set -euo pipefail
printf "code %s\n" "$*" >> "$YAMARK_TEST_ORDER"
"#,
    );

    let result = Command::new(fake_repo.join("tools/install-devbox.sh"))
        .current_dir(temp.path())
        .env("PATH", path_with_fake_bin(&fake_bin))
        .env("YAMARK_TEST_ORDER", &order_log)
        .env_remove("YAMARK_BUNDLE")
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let vsix_out = fake_repo.join("target/vscode/yamark-dev.vsix");
    assert_eq!(
        fs::read_to_string(order_log).unwrap(),
        format!(
            "cargo install --path .\n\
             cargo build --release\n\
             strip\n\
             npx --yes @vscode/vsce package --out {}\n\
             code --install-extension {} --force\n",
            vsix_out.display(),
            vsix_out.display(),
        )
    );
}

fn write_executable(file: &Path, contents: &str) {
    fs::write(file, contents).unwrap();
    make_executable(file);
}

fn make_executable(file: &Path) {
    let mut permissions = fs::metadata(file).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(file, permissions).unwrap();
}

fn path_with_fake_bin(fake_bin: &Path) -> String {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(PathBuf::from(fake_bin)).chain(std::env::split_paths(&path));
    std::env::join_paths(paths)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}
