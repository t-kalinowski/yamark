const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const {
  DEFAULT_FILE_EXTENSIONS,
  createYamarkExtension,
  bundledExecutablePath,
  documentSelector,
  normalizeFileExtensions,
  readSettings,
} = require("../src/core");
const { fakeDocument, fakeVscode } = require("./support/fake-vscode");
const packageJson = require("../package.json");

test("defaults to Markdown, Quarto, and YAML file extensions", () => {
  const vscode = fakeVscode();

  const settings = readSettings(vscode);
  assert.deepEqual(settings.enabledFileExtensions, DEFAULT_FILE_EXTENSIONS);
  assert.equal(settings.useBundledExecutable, false);
  assert.equal(settings.runNextFormatter, true);
  assert.equal(Object.hasOwn(settings, "nextFormatterCommand"), false);
  assert.deepEqual(settings.nextFormatterExecutable, []);
});

test("package contributes format selection command", () => {
  assert.ok(
    packageJson.activationEvents.includes("onCommand:yamark.formatSelectionAsMarkdown"),
  );
  assert.deepEqual(
    packageJson.contributes.commands.find(
      (entry) => entry.command === "yamark.formatSelectionAsMarkdown",
    ),
    {
      command: "yamark.formatSelectionAsMarkdown",
      title: "Yamark: Format Selection as Markdown",
    },
  );
});

test("package contributes a filtered working-tree diff command", () => {
  assert.ok(packageJson.activationEvents.includes("onCommand:yamark.openGitFilterDiff"));
  assert.deepEqual(
    packageJson.contributes.commands.find(
      (entry) => entry.command === "yamark.openGitFilterDiff",
    ),
    {
      command: "yamark.openGitFilterDiff",
      title: "Yamark: Open Filtered Working Tree Diff",
    },
  );
  assert.deepEqual(
    packageJson.contributes.menus["scm/resourceState/context"].find(
      (entry) => entry.command === "yamark.openGitFilterDiff",
    ),
    {
      command: "yamark.openGitFilterDiff",
      when: "scmProvider == git && scmResourceGroup == workingTree",
      group: "2_view@0",
    },
  );
});

test("package contributes formatted preview commands for supported files", () => {
  const nativeWhen =
    "isFileSystemResource && resourceExtname =~ /\\.(md|qmd|rmd|yaml|yml|py|r)$/i";
  const jsonWhen =
    "isFileSystemResource && resourceExtname =~ /\\.(json|jsonl|ndjson|jsonc|json5)$/i";
  const commands = new Map(
    packageJson.contributes.commands.map((entry) => [entry.command, entry.title]),
  );

  assert.ok(packageJson.activationEvents.includes("onCommand:yamark.openFormattedPreview"));
  assert.ok(packageJson.activationEvents.includes("onCommand:yamark.openJsonAsYaml"));
  assert.equal(
    commands.get("yamark.openFormattedPreview"),
    "Yamark: Preview Format Document",
  );
  assert.equal(commands.get("yamark.openJsonAsYaml"), "Yamark: View JSON as YAML");
  assert.deepEqual(packageJson.contributes.menus["explorer/context"], [
    {
      command: "yamark.openFormattedPreview",
      when: `${nativeWhen} && !explorerResourceIsFolder`,
      group: "navigation@20",
    },
    {
      command: "yamark.openJsonAsYaml",
      when: `${jsonWhen} && !explorerResourceIsFolder`,
      group: "navigation@20",
    },
  ]);
  assert.deepEqual(packageJson.contributes.menus["editor/title/context"], [
    {
      command: "yamark.openFormattedPreview",
      when: nativeWhen,
      group: "navigation@20",
    },
    {
      command: "yamark.openJsonAsYaml",
      when: jsonWhen,
      group: "navigation@20",
    },
  ]);
});

test("package has public repository metadata", () => {
  assert.equal(packageJson.repository.type, "git");
  assert.equal(packageJson.repository.url, "https://github.com/t-kalinowski/yamark.git");
  assert.equal(packageJson.bugs.url, "https://github.com/t-kalinowski/yamark/issues");
  assert.equal(packageJson.homepage, "https://t-kalinowski.github.io/yamark/");
});

test("activates format selection command", () => {
  const vscode = fakeVscode();
  const api = createYamarkExtension(vscode, { extensionRoot: "/extension" });

  api.activate({ extensionPath: "/extension", subscriptions: [] });

  assert.ok(vscode.commands.registeredCommands.includes("yamark.formatSelectionAsMarkdown"));
});

test("normalizes configured optional extensions", () => {
  assert.deepEqual(
    normalizeFileExtensions(["md", ".YAML", ".yml", "qmd", "Rmd", ".R", ".py"]),
    [".md", ".yaml", ".yml", ".qmd", ".rmd", ".r", ".py"],
  );
});

test("builds pattern selectors from configured file extensions", () => {
  assert.deepEqual(documentSelector([".md", ".yaml", ".Rmd", ".R"]), [
    { pattern: "**/*.[mM][dD]" },
    { pattern: "**/*.[yY][aA][mM][lL]" },
    { pattern: "**/*.[rR][mM][dD]" },
    { pattern: "**/*.[rR]" },
  ]);
});

test("uses configured yamark executable path", async () => {
  const document = fakeDocument("/tmp/config.yaml", "items: [a,b]\n");
  const calls = [];
  const api = createYamarkExtension(
    fakeVscode({
      settings: {
        executable: "/custom/bin/yamark",
        useBundledExecutable: false,
        runNextFormatter: false,
      },
    }),
    {
      runProcess: async (call) => {
        calls.push(call);
        return "items: [a, b]\n";
      },
    },
  );

  await api.provideDocumentFormattingEdits(document);

  assert.equal(calls[0].command, "/custom/bin/yamark");
});

test("uses bundled yamark when configured", async () => {
  const document = fakeDocument("/tmp/config.yaml", "items: [a,b]\n");
  const calls = [];
  const api = createYamarkExtension(
    fakeVscode({
      settings: {
        useBundledExecutable: true,
        runNextFormatter: false,
      },
    }),
    {
      arch: "arm64",
      extensionRoot: "/extension",
      platform: "darwin",
      runProcess: async (call) => {
        calls.push(call);
        return "items: [a, b]\n";
      },
    },
  );

  await api.provideDocumentFormattingEdits(document);

  assert.equal(calls[0].command, path.join("/extension", "bin", "darwin-arm64", "yamark"));
});

test("resolves bundled Windows executable name", () => {
  assert.equal(
    bundledExecutablePath("/extension", "win32", "x64"),
    path.join("/extension", "bin", "win32-x64", "yamark.exe"),
  );
});

test("does not format disabled extensions", async () => {
  const document = fakeDocument("/tmp/analysis.py", "x = 1\n");
  const api = createYamarkExtension(fakeVscode(), {
    runProcess: async () => {
      throw new Error("yamark should not run");
    },
  });

  assert.deepEqual(await api.provideDocumentFormattingEdits(document), []);
});

