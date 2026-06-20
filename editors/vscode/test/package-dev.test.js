const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const test = require("node:test");

const extensionDir = path.resolve(__dirname, "..");

test("extension package has MIT license text", () => {
  const license = fs.readFileSync(path.join(extensionDir, "LICENSE"), "utf8");

  assert.match(license, /^MIT License/m);
  assert.match(license, /Permission is hereby granted, free of charge/);
});

test("extension settings expose only supported formatter chaining", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(extensionDir, "package.json"), "utf8"),
  );
  const properties = packageJson.contributes.configuration.properties;

  assert.equal(properties["yamark.nextFormatterCommand"], undefined);
  assert.equal(properties["yamark.runNextFormatter"].scope, "language-overridable");
  assert.equal(properties["yamark.nextFormatterExecutable"].scope, "language-overridable");
});

test("dev package strips bundled release executable before packaging", (t) => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "yamark-vscode-package-"));
  t.after(() => fs.rmSync(tempRoot, { recursive: true, force: true }));

  const fakeRepoRoot = path.join(tempRoot, "repo");
  const fakeExtensionDir = path.join(fakeRepoRoot, "editors", "vscode");
  const fakeBinDir = path.join(tempRoot, "bin");
  const fakeScriptDir = path.join(fakeExtensionDir, "scripts");
  fs.mkdirSync(fakeScriptDir, { recursive: true });
  fs.mkdirSync(path.join(fakeExtensionDir, "bin"), { recursive: true });
  fs.mkdirSync(fakeBinDir);

  const packageScript = path.join(fakeScriptDir, "package-dev.sh");
  fs.copyFileSync(path.join(extensionDir, "scripts", "package-dev.sh"), packageScript);
  fs.chmodSync(packageScript, 0o755);

  const orderPath = path.join(tempRoot, "order.log");
  const vsixOut = path.join(tempRoot, "yamark-dev.vsix");
  const bundledExecutable = path.join(
    fakeExtensionDir,
    "bin",
    `${process.platform}-${process.arch}`,
    process.platform === "win32" ? "yamark.exe" : "yamark",
  );

  writeExecutable(
    path.join(fakeBinDir, "cargo"),
    [
      "#!/usr/bin/env bash",
      "set -euo pipefail",
      'printf "cargo %s\\n" "$*" >> "$YAMARK_TEST_ORDER"',
      'if [[ "$*" != "build --release" ]]; then',
      '  echo "unexpected cargo args: $*" >&2',
      "  exit 41",
      "fi",
      "mkdir -p target/release",
      'printf "unstripped\\n" > target/release/yamark',
      "chmod +x target/release/yamark",
    ].join("\n"),
  );

  writeExecutable(
    path.join(fakeBinDir, "strip"),
    [
      "#!/usr/bin/env bash",
      "set -euo pipefail",
      'printf "strip %s\\n" "$1" >> "$YAMARK_TEST_ORDER"',
      'printf "stripped\\n" >> "$1"',
    ].join("\n"),
  );

  writeExecutable(
    path.join(fakeBinDir, "npx"),
    [
      "#!/usr/bin/env bash",
      "set -euo pipefail",
      'printf "npx %s\\n" "$*" >> "$YAMARK_TEST_ORDER"',
      'if ! grep -qx "stripped" "$YAMARK_TEST_BUNDLED"; then',
      '  echo "bundled executable was not stripped before packaging" >&2',
      "  exit 42",
      "fi",
      'touch "$VSIX_OUT"',
    ].join("\n"),
  );

  const result = spawnSync(packageScript, {
    cwd: fakeExtensionDir,
    env: {
      ...process.env,
      PATH: `${fakeBinDir}${path.delimiter}${process.env.PATH}`,
      VSIX_OUT: vsixOut,
      YAMARK_TEST_BUNDLED: bundledExecutable,
      YAMARK_TEST_ORDER: orderPath,
    },
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(
    fs.readFileSync(orderPath, "utf8"),
    [
      "cargo build --release",
      `strip ${bundledExecutable}`,
      "npx --yes @vscode/vsce package --out " + vsixOut,
      "",
    ].join("\n"),
  );
});

test("local install suppresses VS Code CLI deprecation warnings", (t) => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "yamark-vscode-install-"));
  t.after(() => fs.rmSync(tempRoot, { recursive: true, force: true }));

  const fakeRepoRoot = path.join(tempRoot, "repo");
  const fakeExtensionDir = path.join(fakeRepoRoot, "editors", "vscode");
  const fakeBinDir = path.join(tempRoot, "bin");
  const fakeScriptDir = path.join(fakeExtensionDir, "scripts");
  fs.mkdirSync(fakeScriptDir, { recursive: true });
  fs.mkdirSync(fakeBinDir);

  const installScript = path.join(fakeScriptDir, "install-local.sh");
  fs.copyFileSync(path.join(extensionDir, "scripts", "install-local.sh"), installScript);
  fs.chmodSync(installScript, 0o755);

  writeExecutable(
    path.join(fakeScriptDir, "package-dev.sh"),
    [
      "#!/usr/bin/env bash",
      "set -euo pipefail",
      'mkdir -p "$(dirname "$VSIX_OUT")"',
      'touch "$VSIX_OUT"',
    ].join("\n"),
  );

  writeExecutable(
    path.join(fakeBinDir, "code"),
    [
      "#!/usr/bin/env bash",
      "set -euo pipefail",
      'if [[ "${NODE_NO_WARNINGS:-}" != "1" ]]; then',
      '  echo "(node:12627) [DEP0169] DeprecationWarning: url.parse warning" >&2',
      "fi",
      'echo "Extension installed from $1 $2"',
    ].join("\n"),
  );

  const result = spawnSync(installScript, {
    cwd: fakeExtensionDir,
    env: {
      ...process.env,
      PATH: `${fakeBinDir}${path.delimiter}${process.env.PATH}`,
    },
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.doesNotMatch(result.stderr, /DEP0169/);
  assert.match(result.stdout, /Installed Yamark extension/);
});

function writeExecutable(file, contents) {
  fs.writeFileSync(file, `${contents}\n`);
  fs.chmodSync(file, 0o755);
}
