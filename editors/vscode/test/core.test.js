const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const {
  DEFAULT_FILE_EXTENSIONS,
  createChannelLogger,
  createYamarkExtension,
  bundledExecutablePath,
  documentSelector,
  normalizeFileExtensions,
  readSettings,
} = require("../src/core");
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

test("native and JSON-family commands share one formatted preview provider", async () => {
  const markdown = fakeDocument("/tmp/notes.md", "#   Notes ##\n", "markdown");
  const jsonDocuments = ["json", "jsonl", "ndjson", "jsonc", "json5"].map(
    (extension) =>
      fakeDocument(`/tmp/data.${extension}`, '{"answer":42}\n', extension),
  );
  const documents = [markdown, ...jsonDocuments];
  const sourceCount = documents.length;
  const calls = [];
  const vscode = fakeVscode({ documents });
  for (const document of documents) {
    document.uri = vscode.Uri.file(document.fileName);
  }
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input.startsWith("#") ? "# Notes\n" : "answer: 42\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  assert.ok(vscode.commands.registeredCommands.includes("yamark.openFormattedPreview"));
  assert.ok(vscode.commands.registeredCommands.includes("yamark.openJsonAsYaml"));

  await vscode.commands.executeCommand("yamark.openFormattedPreview", markdown.uri);
  for (const document of jsonDocuments) {
    const callCount = calls.length;
    await vscode.commands.executeCommand("yamark.openJsonAsYaml", document.uri);
    assert.equal(calls.length, callCount + 1);
  }

  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  assert.ok(provider);
  assert.equal(calls.length, sourceCount);
  assert.deepEqual(calls[0], {
    command: "yamark",
    args: ["format", "--stdin-file-path", "/tmp/notes.md"],
    input: "#   Notes ##\n",
    cwd: "/tmp",
  });
  assert.deepEqual(calls[1], {
    command: "yamark",
    args: ["to-yaml", "--stdin-file-path", "/tmp/data.json"],
    input: '{"answer":42}\n',
    cwd: "/tmp",
  });
  assert.deepEqual(
    vscode.window.shownTextDocuments.map(({ document }) => document.uri.scheme),
    Array.from({ length: sourceCount }, () => "yamark-preview"),
  );
  assert.deepEqual(
    vscode.window.shownTextDocuments.map(({ options }) => options),
    Array.from({ length: sourceCount }, () => ({ preview: true })),
  );
  assert.equal(vscode.window.shownTextDocuments[0].document.languageId, "markdown");
  for (const { document } of vscode.window.shownTextDocuments.slice(1)) {
    assert.equal(document.languageId, "yaml");
  }
});

test("formatted native preview runs the complete Format Document chain", async () => {
  const source = fakeDocument(
    "/tmp/notes.qmd",
    "---\ntags: [r,code]\n---\n\n```{r}\nx<-1\n```\n",
    "quarto",
    { isDirty: true, version: 3 },
  );
  const calls = [];
  const appliedEdits = [];
  const vscode = fakeVscode({
    documents: [source],
    settings: {
      extraArguments: ["--compact"],
      runNextFormatter: true,
      nextFormatterExecutable: ["quarto-native", "--stdin-file-path", "${file}"],
    },
    onApplyEdit: async (edit) => appliedEdits.push(edit),
  });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      if (call.command === "yamark") {
        return "---\ntags: [r, code]\n---\n\n```{r}\nx<-1\n```\n";
      }
      if (call.command === "quarto-native") {
        return "---\ntags: [r, code]\n---\n\n```{r}\nx <- 1\n```\n";
      }
      assert.fail(`unexpected command: ${call.command}`);
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openFormattedPreview", source.uri);

  assert.deepEqual(calls, [
    {
      command: "yamark",
      args: [
        "format",
        "--compact",
        "--skip-embedded-formatters",
        "--stdin-file-path",
        "/tmp/notes.qmd",
      ],
      input: "---\ntags: [r,code]\n---\n\n```{r}\nx<-1\n```\n",
      cwd: "/tmp",
    },
    {
      command: "quarto-native",
      args: ["--stdin-file-path", "/tmp/notes.qmd"],
      input: "---\ntags: [r, code]\n---\n\n```{r}\nx<-1\n```\n",
      cwd: "/tmp",
    },
  ]);
  assert.equal(
    vscode.window.shownTextDocuments[0].document.getText(),
    "---\ntags: [r, code]\n---\n\n```{r}\nx <- 1\n```\n",
  );
  assert.equal(source.getText(), "---\ntags: [r,code]\n---\n\n```{r}\nx<-1\n```\n");
  assert.deepEqual(appliedEdits, []);
});

test("JSON as YAML bypasses format arguments and the next formatter", async () => {
  const source = fakeDocument("/tmp/data.json5", "{answer:42}\n", "json5");
  const calls = [];
  const vscode = fakeVscode({
    documents: [source],
    settings: {
      extraArguments: ["--wrap", "sentence"],
      runNextFormatter: true,
      nextFormatterExecutable: ["prettier", "--stdin-filepath", "${file}"],
    },
  });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return "answer: 42\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);

  assert.deepEqual(calls, [
    {
      command: "yamark",
      args: ["to-yaml", "--stdin-file-path", "/tmp/data.json5"],
      input: "{answer:42}\n",
      cwd: "/tmp",
    },
  ]);
  assert.equal(vscode.window.shownTextDocuments[0].document.languageId, "yaml");
  assert.equal(vscode.window.shownTextDocuments[0].document.getText(), "answer: 42\n");
});

test("JSON as YAML ignores unrelated language-scoped formatter settings", async () => {
  const source = fakeDocument("/tmp/data.json5", "{answer:42}\n", "json5");
  const calls = [];
  const vscode = fakeVscode({
    documents: [source],
    settings: {
      "[json5]": {
        enabledFileExtensions: [""],
        extraArguments: "not an array",
        runNextFormatter: "not a boolean",
        nextFormatterExecutable: [""],
      },
    },
  });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return "answer: 42\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);

  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].args, [
    "to-yaml",
    "--stdin-file-path",
    "/tmp/data.json5",
  ]);
});

test("preview commands reject the other operation's source types", async () => {
  const markdown = fakeDocument("/tmp/notes.md", "# Notes\n", "markdown");
  const json = fakeDocument("/tmp/data.json", "{}\n", "json");
  const vscode = fakeVscode({ documents: [markdown, json] });
  markdown.uri = vscode.Uri.file(markdown.fileName);
  json.uri = vscode.Uri.file(json.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async () => assert.fail("wrong preview command must not run Yamark"),
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await assert.rejects(
    () => vscode.commands.executeCommand("yamark.openFormattedPreview", json.uri),
    /cannot preview \.json with Format Document/,
  );
  await assert.rejects(
    () => vscode.commands.executeCommand("yamark.openJsonAsYaml", markdown.uri),
    /cannot view \.md as YAML/,
  );
  assert.deepEqual(vscode.window.shownTextDocuments, []);
});

test("formatted preview uses dirty text and refreshes one stable virtual document", async () => {
  const source = fakeDocument(
    "/tmp/data.json",
    '{"version":1}\n',
    "json",
    { isDirty: true, version: 1 },
  );
  const calls = [];
  const activeTextEditor = { document: source };
  const vscode = fakeVscode({ documents: [source], activeTextEditor });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input.includes(":1") ? "version: 1\n" : "version: 2\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  assert.ok(provider);
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  const firstPreview = vscode.window.shownTextDocuments[0].document.uri;
  const changedUris = [];
  provider.onDidChange((uri) => changedUris.push(uri));

  activeTextEditor.document = vscode.window.shownTextDocuments[0].document;
  source.text = '{"version":2}\n';
  source.version = 2;
  await vscode.commands.executeCommand("yamark.openJsonAsYaml");
  const secondPreview = vscode.window.shownTextDocuments[1].document.uri;

  assert.deepEqual(calls.map((call) => call.input), [
    '{"version":1}\n',
    '{"version":2}\n',
  ]);
  assert.equal(secondPreview.toString(), firstPreview.toString());
  assert.deepEqual(changedUris, [secondPreview]);
  assert.deepEqual(vscode.languages.changedDocumentLanguages, []);
  assert.equal(await provider.provideTextDocumentContent(secondPreview), "version: 2\n");
});

test("JSONL YAML preview refreshes after its source file is appended", async () => {
  const source = fakeDocument(
    "/tmp/events.jsonl",
    '{"id":1}\n',
    "jsonl",
  );
  const calls = [];
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input.includes('"id":2')
        ? "---\nid: 1\n---\nid: 2\n"
        : "---\nid: 1\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  const previewUri = vscode.window.shownTextDocuments[0].document.uri;
  const changedUris = [];
  provider.onDidChange((uri) => changedUris.push(uri));

  vscode.workspace.closeTextDocument(source);
  source.text += '{"id":2}\n';
  await vscode.workspace.fireFileChange(source.uri);

  assert.deepEqual(calls.map((call) => call.input), [
    '{"id":1}\n',
    '{"id":1}\n{"id":2}\n',
  ]);
  assert.deepEqual(changedUris, [previewUri]);
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  assert.equal(
    await provider.provideTextDocumentContent(previewUri),
    "---\nid: 1\n---\nid: 2\n",
  );
});

test("JSONL YAML preview catches an append during its initial projection", async () => {
  const source = fakeDocument(
    "/tmp/events.jsonl",
    '{"id":1}\n',
    "jsonl",
    { isDirty: false },
  );
  const calls = [];
  let markInitialProjectionStarted;
  let resolveInitialProjection;
  const initialProjectionStarted = new Promise((resolve) => {
    markInitialProjectionStarted = resolve;
  });
  const initialProjection = new Promise((resolve) => {
    resolveInitialProjection = resolve;
  });
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      if (calls.length === 1) {
        markInitialProjectionStarted();
        return await initialProjection;
      }
      return "---\nid: 1\n---\nid: 2\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  const openPreview = vscode.commands.executeCommand(
    "yamark.openJsonAsYaml",
    source.uri,
  );
  await initialProjectionStarted;
  source.text += '{"id":2}\n';
  await vscode.workspace.fireFileChange(source.uri);
  resolveInitialProjection("---\nid: 1\n");
  await openPreview;

  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  const previewUri = vscode.window.shownTextDocuments[0].document.uri;
  assert.deepEqual(calls.map((call) => call.input), [
    '{"id":1}\n',
    '{"id":1}\n{"id":2}\n',
  ]);
  assert.equal(
    await provider.provideTextDocumentContent(previewUri),
    "---\nid: 1\n---\nid: 2\n",
  );
});

test("live JSONL refresh coalesces changes while conversion is running", async () => {
  const source = fakeDocument(
    "/tmp/events.jsonl",
    '{"id":1}\n',
    "jsonl",
  );
  const calls = [];
  let markRefreshStarted;
  let resolveRefresh;
  const refreshStarted = new Promise((resolve) => {
    markRefreshStarted = resolve;
  });
  const refreshOutput = new Promise((resolve) => {
    resolveRefresh = resolve;
  });
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      if (call.input.includes('"id":3')) {
        return "---\nid: 1\n---\nid: 2\n---\nid: 3\n";
      }
      if (call.input.includes('"id":2')) {
        markRefreshStarted();
        return await refreshOutput;
      }
      return "---\nid: 1\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  const previewUri = vscode.window.shownTextDocuments[0].document.uri;

  source.text += '{"id":2}\n';
  const firstChange = vscode.workspace.fireFileChange(source.uri);
  await refreshStarted;
  source.text += '{"id":3}\n';
  const secondChange = vscode.workspace.fireFileChange(source.uri);
  const thirdChange = vscode.workspace.fireFileChange(source.uri);
  resolveRefresh("---\nid: 1\n---\nid: 2\n");
  await Promise.all([firstChange, secondChange, thirdChange]);

  assert.deepEqual(calls.map((call) => call.input), [
    '{"id":1}\n',
    '{"id":1}\n{"id":2}\n',
    '{"id":1}\n{"id":2}\n{"id":3}\n',
  ]);
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  assert.equal(
    await provider.provideTextDocumentContent(previewUri),
    "---\nid: 1\n---\nid: 2\n---\nid: 3\n",
  );
});

test("live JSONL preview keeps its last output after an incomplete append", async () => {
  const source = fakeDocument(
    "/tmp/events.jsonl",
    '{"id":1}\n',
    "jsonl",
  );
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      if (call.input.endsWith('{"id":')) {
        throw new Error("incomplete JSONL record");
      }
      return call.input.includes('"id":2')
        ? "---\nid: 1\n---\nid: 2\n"
        : "---\nid: 1\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  const previewUri = vscode.window.shownTextDocuments[0].document.uri;
  const changedUris = [];
  provider.onDidChange((uri) => changedUris.push(uri));

  source.text += '{"id":';
  await vscode.workspace.fireFileChange(source.uri);
  assert.equal(
    await provider.provideTextDocumentContent(previewUri),
    "---\nid: 1\n",
  );
  assert.deepEqual(changedUris, []);

  source.text += "2}\n";
  await vscode.workspace.fireFileChange(source.uri);
  assert.equal(
    await provider.provideTextDocumentContent(previewUri),
    "---\nid: 1\n---\nid: 2\n",
  );
  assert.deepEqual(changedUris, [previewUri]);
});

test("an older concurrent refresh cannot replace a newer preview", async () => {
  const source = fakeDocument(
    "/tmp/data.json",
    '{"version":1}\n',
    "json",
    { isDirty: true, version: 1 },
  );
  let resolveFirst;
  let resolveSecond;
  let markFirstStarted;
  let markSecondStarted;
  const firstStarted = new Promise((resolve) => {
    markFirstStarted = resolve;
  });
  const secondStarted = new Promise((resolve) => {
    markSecondStarted = resolve;
  });
  const firstOutput = new Promise((resolve) => {
    resolveFirst = resolve;
  });
  const secondOutput = new Promise((resolve) => {
    resolveSecond = resolve;
  });
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      if (call.input.includes(":1")) {
        markFirstStarted();
        return await firstOutput;
      }
      markSecondStarted();
      return await secondOutput;
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  const first = vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  await firstStarted;
  source.text = '{"version":2}\n';
  source.version = 2;
  const second = vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  await secondStarted;

  resolveSecond("version: 2\n");
  await second;
  resolveFirst("version: 1\n");
  await first;

  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  const previewUri = vscode.window.shownTextDocuments[0].document.uri;
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  assert.equal(await provider.provideTextDocumentContent(previewUri), "version: 2\n");
});

test("formatted preview provider only reads cached output", async () => {
  const source = fakeDocument("/tmp/data.json", '{"answer":42}\n', "json");
  const calls = [];
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return "answer: 42\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  assert.ok(provider);
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  const previewUri = vscode.window.shownTextDocuments[0].document.uri;
  assert.equal(await provider.provideTextDocumentContent(previewUri), "answer: 42\n");
  assert.equal(await provider.provideTextDocumentContent(previewUri), "answer: 42\n");
  assert.equal(calls.length, 1);

  const expiredUri = vscode.Uri.file("/tmp/expired.json").with({
    scheme: "yamark-preview",
  });
  await assert.rejects(() => provider.provideTextDocumentContent(expiredUri));
  assert.equal(calls.length, 1);
});

test("closing and reopening a live JSONL preview uses current source text", async () => {
  const source = fakeDocument("/tmp/data.jsonl", '{"answer":42}\n', "jsonl");
  const calls = [];
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input.includes("43")
        ? "---\nanswer: 42\n---\nanswer: 43\n"
        : "answer: 42\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-preview");
  assert.ok(provider);
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  const previewDocument = vscode.window.shownTextDocuments[0].document;
  assert.equal(
    await provider.provideTextDocumentContent(previewDocument.uri),
    "answer: 42\n",
  );

  await vscode.window.closeTextDocumentTab(previewDocument);
  source.text += '{"answer":43}\n';
  await vscode.workspace.fireFileChange(source.uri);
  assert.equal(calls.length, 1);
  assert.equal(
    await provider.provideTextDocumentContent(previewDocument.uri),
    "answer: 42\n",
  );

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri);

  assert.deepEqual(calls.map((call) => call.input), [
    '{"answer":42}\n',
    '{"answer":42}\n{"answer":43}\n',
  ]);
  assert.equal(vscode.window.shownTextDocuments.length, 2);
  assert.equal(
    vscode.window.shownTextDocuments[1].document.getText(),
    "---\nanswer: 42\n---\nanswer: 43\n",
  );
});

test("a JSON-to-YAML failure does not open a formatted preview", async () => {
  const source = fakeDocument("/tmp/data.json5", "{broken}\n", "json5");
  const calls = [];
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      throw new Error("invalid JSON5");
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openJsonAsYaml", source.uri).catch(() => {});

  assert.equal(calls.length, 1);
  assert.deepEqual(vscode.window.shownTextDocuments, []);
});

test("a native next-formatter failure does not publish partial output", async () => {
  const source = fakeDocument("/tmp/notes.md", "#   Notes ##\n", "markdown");
  const calls = [];
  const vscode = fakeVscode({
    documents: [source],
    settings: {
      runNextFormatter: true,
      nextFormatterExecutable: ["nativefmt"],
    },
  });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      if (call.command === "yamark") {
        return "# Notes\n";
      }
      throw new Error("native formatter failed");
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await assert.rejects(
    () => vscode.commands.executeCommand("yamark.openFormattedPreview", source.uri),
    /native formatter failed/,
  );

  assert.equal(calls.length, 2);
  assert.deepEqual(vscode.window.shownTextDocuments, []);
});

test("formatted preview documents are excluded from the formatting provider", async () => {
  const source = fakeDocument("/tmp/notes.md", "#   Notes ##\n", "markdown");
  const calls = [];
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return "# Notes\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openFormattedPreview", source.uri);
  assert.equal(vscode.window.shownTextDocuments.length, 1);
  const previewDocument = vscode.window.shownTextDocuments[0].document;
  const edits = await vscode.commands.executeCommand(
    "vscode.executeFormatDocumentProvider",
    previewDocument.uri,
  );

  assert.deepEqual(edits, []);
  assert.equal(calls.length, 1);
});

test("opens the filtered index beside an unstaged working-tree file", async () => {
  const processCalls = [];
  const repositoryRoot = "/repo";
  const filePath = "/repo/docs/post.md";
  let gitRepository;
  const vscode = fakeVscode({
    extensions: {
      "vscode.git": {
        activate: async () => ({
          getAPI: (version) => {
            assert.equal(version, 1);
            return {
              git: { path: "/usr/bin/git" },
              getRepository: (uri) => {
                assert.equal(uri.fsPath, filePath);
                return gitRepository;
              },
            };
          },
        }),
      },
    },
  });
  const repositoryStateChanged = new vscode.EventEmitter();
  gitRepository = {
    rootUri: vscode.Uri.file(repositoryRoot),
    state: { onDidChange: repositoryStateChanged.event },
  };
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      processCalls.push(call);
      if (call.args[0] === "check-attr") {
        return "docs/post.md: filter: yamark-md\n";
      }
      assert.equal(call.args[0], "cat-file");
      return "First sentence wraps across the\nworking-tree width.\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openGitFilterDiff", {
    resourceUri: vscode.Uri.file(filePath),
  });

  const diffCall = vscode.commands.executedCommands.find(
    (call) => call.command === "vscode.diff",
  );
  assert.ok(diffCall);
  const [originalUri, modifiedUri, title] = diffCall.args;
  assert.equal(originalUri.scheme, "yamark-git-filter");
  assert.equal(originalUri.fsPath, filePath);
  assert.equal(modifiedUri.fsPath, filePath);
  assert.equal(title, "post.md (Filtered Working Tree)");

  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-git-filter");
  assert.ok(provider);
  const changedUris = [];
  provider.onDidChange((uri) => changedUris.push(uri));
  repositoryStateChanged.fire();
  assert.deepEqual(changedUris, [originalUri]);
  assert.equal(
    await provider.provideTextDocumentContent(originalUri),
    "First sentence wraps across the\nworking-tree width.\n",
  );
  assert.deepEqual(processCalls, [
    {
      command: "/usr/bin/git",
      args: ["check-attr", "filter", "--", "docs/post.md"],
      input: "",
      cwd: repositoryRoot,
    },
    {
      command: "/usr/bin/git",
      args: ["cat-file", "--filters", ":docs/post.md"],
      input: "",
      cwd: repositoryRoot,
    },
  ]);
});

test("preserves a UNC file URI when reading the filtered index", async () => {
  const repositoryRoot = "//server/share/repo";
  const filePath = "//server/share/repo/docs/post.md";
  let gitRepository;
  const vscode = fakeVscode({
    extensions: {
      "vscode.git": {
        activate: async () => ({
          getAPI: () => ({
            git: { path: "/usr/bin/git" },
            getRepository: () => gitRepository,
          }),
        }),
      },
    },
  });
  const repositoryStateChanged = new vscode.EventEmitter();
  gitRepository = {
    rootUri: vscode.Uri.file(repositoryRoot),
    state: { onDidChange: repositoryStateChanged.event },
  };
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) =>
      call.args[0] === "check-attr"
        ? "docs/post.md: filter: yamark-md\n"
        : "Filtered index.\n",
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.openGitFilterDiff", {
    resourceUri: vscode.Uri.file(filePath),
  });

  const diffCall = vscode.commands.executedCommands.find(
    (call) => call.command === "vscode.diff",
  );
  const originalUri = diffCall.args[0];
  assert.equal(originalUri.authority, "server");
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-git-filter");
  assert.equal(await provider.provideTextDocumentContent(originalUri), "Filtered index.\n");
});

test("reuses the filtered document and repository watcher across repeated opens", async () => {
  const repositoryRoot = "/repo";
  const filePath = "/repo/docs/post.md";
  let repositorySubscriptionCount = 0;
  const vscode = fakeVscode({
    extensions: {
      "vscode.git": {
        activate: async () => ({
          getAPI: () => ({
            git: { path: "/usr/bin/git" },
            getRepository: () => ({
              rootUri: vscode.Uri.file(repositoryRoot),
              state: {
                onDidChange: (listener) => {
                  repositorySubscriptionCount += 1;
                  return new vscode.EventEmitter().event(listener);
                },
              },
            }),
          }),
        }),
      },
    },
  });
  const api = createYamarkExtension(vscode, {
    runProcess: async () => "docs/post.md: filter: yamark-md\n",
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });
  const resource = { resourceUri: vscode.Uri.file(filePath) };

  await vscode.commands.executeCommand("yamark.openGitFilterDiff", resource);
  await vscode.commands.executeCommand("yamark.openGitFilterDiff", resource);

  const originalUris = vscode.commands.executedCommands
    .filter((call) => call.command === "vscode.diff")
    .map((call) => call.args[0].toString());
  assert.deepEqual(originalUris, [originalUris[0], originalUris[0]]);
  assert.equal(repositorySubscriptionCount, 1);
});

test("reads a restored filtered diff URI after extension activation", async () => {
  const repositoryRoot = "/repo";
  const filePath = "/repo/docs/post.md";
  let gitRepository;
  const vscode = fakeVscode({
    extensions: {
      "vscode.git": {
        activate: async () => ({
          getAPI: () => ({
            git: { path: "/usr/bin/git" },
            getRepository: (uri) => {
              assert.equal(uri.scheme, "file");
              assert.equal(uri.fsPath, filePath);
              return gitRepository;
            },
          }),
        }),
      },
    },
  });
  const repositoryStateChanged = new vscode.EventEmitter();
  gitRepository = {
    rootUri: vscode.Uri.file(repositoryRoot),
    state: { onDidChange: repositoryStateChanged.event },
  };
  const api = createYamarkExtension(vscode, {
    runProcess: async () => "Filtered index.\n",
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });
  const restoredUri = vscode.Uri.file(filePath).with({
    scheme: "yamark-git-filter",
  });
  const provider =
    vscode.workspace.registeredTextDocumentContentProviders.get("yamark-git-filter");

  assert.equal(await provider.provideTextDocumentContent(restoredUri), "Filtered index.\n");
});

test("legacy command chaining settings are ignored", async () => {
  const document = fakeDocument("/tmp/analysis.py", "text\n", "python");
  const commands = [];
  const calls = [];
  const lines = [];
  const logger = createChannelLogger({ appendLine: (line) => lines.push(line) });
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: true,
      formatOnSave: true,
      nextFormatterCommand: "pythonFormatter.format",
      nextFormatterCommands: {
        python: "pythonFormatter.format",
      },
    },
    async onExecuteCommand(command) {
      commands.push(command);
      throw new Error("legacy command chaining should not run");
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    logger,
    runProcess: async (call) => {
      calls.push(call);
      return "yamarked\n";
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(commands, []);
  assert.equal(calls.length, 1);
  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "yamarked\n");
  assert.match(lines.join("\n"), /formatter step=skip name=legacy-command reason=unsupported/);
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

test("provider selector matches mixed-case configured file extensions", async () => {
  const document = fakeDocument("/tmp/Report.Qmd", "#   Title ##\n", "quarto");
  const calls = [];
  const vscode = fakeVscode({
    documents: [document],
    settings: {
      enabledFileExtensions: [".qmd"],
      useBundledExecutable: false,
      runNextFormatter: false,
    },
  });
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return "# Title\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  const edits = await vscode.commands.executeCommand(
    "vscode.executeFormatDocumentProvider",
    document.uri,
  );

  assert.equal(calls.length, 1);
  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "# Title\n");
});

test("formats a document through yamark stdin mode", async () => {
  const document = fakeDocument("/tmp/config.yaml", "items: [a,b]\n");
  const calls = [];
  const api = createYamarkExtension(fakeVscode({
    settings: {
      useBundledExecutable: false,
      runNextFormatter: false,
    },
  }), {
    runProcess: async (call) => {
      calls.push(call);
      return "items: [a, b]\n";
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "items: [a, b]\n");
  assert.deepEqual(calls, [
    {
      command: "yamark",
      args: ["format", "--stdin-file-path", "/tmp/config.yaml"],
      input: "items: [a,b]\n",
      cwd: "/tmp",
    },
  ]);
});

test("resolves relative document paths before invoking yamark", async () => {
  const relativePath = "./yaml-defense/index.qmd";
  const resolvedPath = path.resolve(relativePath);
  const document = fakeDocument(relativePath, "---\ntags: [a,b]\n");
  const calls = [];
  const api = createYamarkExtension(fakeVscode({
    settings: {
      useBundledExecutable: false,
      runNextFormatter: false,
    },
  }), {
    runProcess: async (call) => {
      calls.push(call);
      return "---\ntags: [a, b]\n";
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.equal(edits.length, 1);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].cwd, path.dirname(resolvedPath));
  assert.equal(calls[0].args[2], resolvedPath);
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

test("default chain setting runs yamark only when no executable is configured", async () => {
  const document = fakeDocument("/tmp/analysis.py", "raw_input\n", "python");
  const order = [];
  let yamarkInput;
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: true,
    },
    async onExecuteCommand(command) {
      order.push(`next:${command}`);
      throw new Error("provider fallthrough should not run");
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      yamarkInput = call.input;
      order.push("yamark");
      return "after_yamark\n";
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(order, ["yamark"]);
  assert.equal(yamarkInput, "raw_input\n");
  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "after_yamark\n");
});

test("configured stdin next formatter executable runs after yamark", async () => {
  const document = fakeDocument("/tmp/analysis.R", "call(\n  x\n)\n", "r");
  const calls = [];
  const commands = [];
  const yamarkCommand = "yamark";
  const vscode = fakeVscode({
    extensions: {
      "posit.air-vscode": "/extensions/air",
    },
    settings: {
      enabledFileExtensions: [".r"],
      runNextFormatter: true,
      "[r]": {
        nextFormatterExecutable: [
          "${extension:posit.air-vscode}/bundled/bin/air",
          "format",
          "--stdin-file-path",
          "${file}",
        ],
      },
    },
    async onExecuteCommand(command) {
      commands.push(command);
      return [];
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      calls.push(call);
      if (call.command === yamarkCommand) {
        assert.equal(call.input, "call(\n  x\n)\n");
        return "call(\n  yamarked = TRUE\n)\n";
      }
      if (call.command.endsWith("/bundled/bin/air")) {
        assert.equal(call.input, "call(\n  yamarked = TRUE\n)\n");
        return "call(\n  yamarked = TRUE\n)\n";
      }
      assert.fail(`unexpected command: ${call.command}`);
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(commands, []);
  assert.equal(calls.length, 2);
  assert.deepEqual(calls[0], {
    command: yamarkCommand,
    args: ["format", "--stdin-file-path", "/tmp/analysis.R"],
    input: "call(\n  x\n)\n",
    cwd: "/tmp",
  });
  assert.deepEqual(calls[1], {
    command: "/extensions/air/bundled/bin/air",
    args: ["format", "--stdin-file-path", "/tmp/analysis.R"],
    input: "call(\n  yamarked = TRUE\n)\n",
    cwd: "/tmp",
  });
  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "call(\n  yamarked = TRUE\n)\n");
});

test("extension placeholders normalize ids before running the next executable", async () => {
  const document = fakeDocument("/tmp/analysis.R", "call(\n  x\n)\n", "r");
  const calls = [];
  const yamarkCommand = "yamark";
  const vscode = fakeVscode({
    extensions: {
      "posit.air-vscode": "/extensions/air",
    },
    settings: {
      enabledFileExtensions: [".r"],
      runNextFormatter: true,
      "[r]": {
        nextFormatterExecutable: [
          "${extension:Posit.air-vscode}/bundled/bin/air",
          "format",
          "--stdin-file-path",
          "${file}",
        ],
      },
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      calls.push(call);
      if (call.command === yamarkCommand) {
        return "call(\n  yamarked = TRUE\n)\n";
      }
      if (call.command.endsWith("/bundled/bin/air")) {
        assert.equal(call.input, "call(\n  yamarked = TRUE\n)\n");
        return "call(yamarked = TRUE)\n";
      }
      assert.fail(`unexpected command: ${call.command}`);
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(calls.map((call) => call.command), [
    yamarkCommand,
    "/extensions/air/bundled/bin/air",
  ]);
  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "call(yamarked = TRUE)\n");
});

test("documented Ruff executable setting runs after yamark", async () => {
  const document = fakeDocument("/tmp/analysis.py", "x=1\n", "python");
  const calls = [];
  const yamarkCommand = "yamark";
  const vscode = fakeVscode({
    extensions: {
      "charliermarsh.ruff": "/extensions/ruff",
    },
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: true,
      "[python]": {
        nextFormatterExecutable: [
          "${extension:charliermarsh.ruff}/bundled/libs/bin/ruff${exe}",
          "format",
          "--stdin-filename",
          "${file}",
          "-",
        ],
      },
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      calls.push(call);
      if (call.command === yamarkCommand) {
        return "x=1\n";
      }
      if (call.command.endsWith("/bundled/libs/bin/ruff")) {
        assert.equal(call.input, "x=1\n");
        return "x = 1\n";
      }
      assert.fail(`unexpected command: ${call.command}`);
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(calls.map((call) => call.command), [
    yamarkCommand,
    "/extensions/ruff/bundled/libs/bin/ruff",
  ]);
  assert.deepEqual(calls[1].args, [
    "format",
    "--stdin-filename",
    "/tmp/analysis.py",
    "-",
  ]);
  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "x = 1\n");
});

test("chain returns no edits when yamark produces the original text", async () => {
  const document = fakeDocument("/tmp/analysis.py", "stable\n", "python");
  const order = [];
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: true,
    },
    async onExecuteCommand(command) {
      order.push(`next:${command}`);
      throw new Error("provider fallthrough should not run");
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      order.push("yamark");
      assert.equal(call.input, "stable\n");
      return "stable\n";
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(edits, []);
  assert.deepEqual(order, ["yamark"]);
});

test("returns yamark text edits and does not chain when runNextFormatter is false", async () => {
  const document = fakeDocument("/tmp/analysis.py", "text\n", "python");
  const commands = [];
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: false,
    },
    async onExecuteCommand(command) {
      commands.push(command);
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async () => "yamarked\n",
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "yamarked\n");
  assert.deepEqual(commands, []);
});

test("per-language runNextFormatter:false disables the chain for that language", async () => {
  const document = fakeDocument("/tmp/analysis.py", "text\n", "python");
  const commands = [];
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: true,
      "[python]": {
        runNextFormatter: false,
      },
    },
    async onExecuteCommand(command) {
      commands.push(command);
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async () => "yamarked\n",
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "yamarked\n");
  assert.deepEqual(commands, []);
});

test("logger records structured fields for the executable formatter chain", async () => {
  const document = fakeDocument("/tmp/notes.md", "before\n", "markdown", {
    isDirty: true,
    version: 7,
  });
  const lines = [];
  const channel = { appendLine: (line) => lines.push(line) };
  const logger = createChannelLogger(channel);
  const yamarkCommand = "yamark";
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".md"],
      runNextFormatter: true,
      nextFormatterExecutable: ["nativefmt", "--stdin-file-path", "${file}"],
    },
    async onExecuteCommand() {
      throw new Error("provider fallthrough should not run");
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    logger,
    runProcess: async (call) => {
      if (call.command === yamarkCommand) {
        return "after-yamark\n";
      }
      if (call.command === "nativefmt") {
        return "after-native\n";
      }
      assert.fail(`unexpected command: ${call.command}`);
    },
  });

  await api.provideDocumentFormattingEdits(document);

  const messages = lines.join("\n");
  assert.match(messages, /document uri=file:\/\/\/tmp\/notes\.md path=.+notes\.md languageId=markdown version=7 dirty=true trigger=provider/);
  assert.match(messages, /formatter step=start name=vscode-provider trigger=provider/);
  assert.match(messages, /settings runNextFormatter=true/);
  assert.match(messages, /formatter step=start name=yamark kind=process command=.+ args=/);
  assert.match(messages, /formatter step=end name=yamark kind=process edits=1 applied=false captured=true output\.bytes=13 changed=true dt\.ms=\d+/);
  assert.match(messages, /formatter step=start name=next-executable kind=process command=nativefmt/);
  assert.match(messages, /formatter step=end name=next-executable kind=process edits=1 applied=false captured=true output\.bytes=13 changed=true dt\.ms=\d+/);
  assert.match(messages, /formatter step=end name=vscode-provider edits=1 applied=false captured=true reason=return-edits final\.bytes=13/);
  assert.match(messages, /provider return edits count=1 final\.bytes=13/);
  assert.match(messages, /end done edits=1 final\.bytes=13/);
});

test("logger records yamark spawn failure", async () => {
  const document = fakeDocument("/tmp/notes.qmd", "x\n", "quarto");
  const lines = [];
  const channel = { appendLine: (line) => lines.push(line) };
  const logger = createChannelLogger(channel);
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".qmd"],
      runNextFormatter: false,
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    logger,
    runProcess: async () => {
      throw new Error("boom");
    },
  });

  await assert.rejects(() => api.provideDocumentFormattingEdits(document), /boom/);
  const messages = lines.join("\n");
  assert.match(messages, /formatter step=error name=yamark kind=process dt\.ms=\d+ err=boom/);
  assert.match(messages, /ERROR boom/);
});

test("manual command suppresses provider registry re-entry while applying edits", async () => {
  const document = fakeDocument("/tmp/notes.md", "before\n", "markdown");
  const lines = [];
  const logger = createChannelLogger({ appendLine: (line) => lines.push(line) });
  const calls = [];
  let reentrantEdits;
  const order = [];
  const vscode = fakeVscode({
    documents: [document],
    settings: {
      enabledFileExtensions: [".md"],
      runNextFormatter: false,
      useBundledExecutable: false,
    },
    onApplyEdit: async () => {
      order.push("executeFormatDocumentProvider:markdown");
      reentrantEdits = await vscode.commands.executeCommand(
        "vscode.executeFormatDocumentProvider",
        document.uri,
      );
    },
  });
  const api = createYamarkExtension(vscode, {
    logger,
    runProcess: async (call) => {
      order.push("yamark");
      calls.push(call);
      return "after\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await api.formatDocument(document);

  assert.equal(calls.length, 1);
  assert.deepEqual(reentrantEdits, []);
  assert.deepEqual(order, ["yamark", "executeFormatDocumentProvider:markdown"]);
  const messages = lines.join("\n");
  assert.match(messages, /document uri=file:\/\/\/tmp\/notes\.md path=.+notes\.md languageId=markdown version=n\/a dirty=n\/a trigger=command/);
  assert.match(messages, /suppression action=add depth=1 reason=apply-command-edits/);
  assert.match(messages, /suppression action=hit depth=1 reason=apply-command-edits/);
  assert.match(messages, /suppression action=remove depth=0 reason=apply-command-edits/);
  assert.match(messages, /apply edits step=start count=1/);
  assert.match(messages, /apply edits step=end count=1 applied=true/);
  assert.match(messages, /end done applied\.edits=1/);
});

test("format selection command formats the selected text as markdown", async () => {
  const text = "before\n- a\n- b\nafter\n";
  const selection = fakeSelection(text.indexOf("- a"), text.indexOf("after") - 1);
  const document = fakeDocument("/tmp/prompt.py", text, "python");
  const calls = [];
  const appliedEdits = [];
  const vscode = fakeVscode({
    activeTextEditor: { document, selection },
    settings: {
      enabledFileExtensions: [".py"],
      runNextFormatter: true,
      nextFormatterExecutable: ["nativefmt"],
      useBundledExecutable: false,
    },
    onApplyEdit: async (edit) => {
      appliedEdits.push(edit);
    },
  });
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return "- a\n- b\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.formatSelectionAsMarkdown");

  assert.deepEqual(calls, [
    {
      command: "yamark",
      args: ["format", "--stdin-file-path", "/tmp/prompt.py.md"],
      input: "- a\n- b",
      cwd: "/tmp",
    },
  ]);
  assert.equal(appliedEdits.length, 1);
  assert.deepEqual(appliedEdits[0].edits, [
    {
      uri: document.uri,
      range: selection,
      newText: "- a\n- b\n",
    },
  ]);
});

test("format selection command is a clear no-op for an empty selection", async () => {
  const document = fakeDocument("/tmp/notes.md", "# Notes\n", "markdown");
  const selection = fakeSelection(0, 0);
  const calls = [];
  const appliedEdits = [];
  const statusMessages = [];
  const vscode = fakeVscode({
    activeTextEditor: { document, selection },
    settings: {
      useBundledExecutable: false,
    },
    onApplyEdit: async (edit) => {
      appliedEdits.push(edit);
    },
    onStatusBarMessage: (message) => {
      statusMessages.push(message);
    },
  });
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input;
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  await vscode.commands.executeCommand("yamark.formatSelectionAsMarkdown");

  assert.deepEqual(calls, []);
  assert.deepEqual(appliedEdits, []);
  assert.deepEqual(statusMessages, ["Yamark: no text selected."]);
});

test("quarto provider re-entry can dispatch Air-like nested formatting without Yamark recursion", async () => {
  const document = fakeDocument(
    "/tmp/notes.qmd",
    "---\ntags: [r,code]\n---\n\n```{r}\nf <- function(x)x+1\n```\n",
    "quarto",
  );
  const vdoc = fakeDocument("/tmp/.vdoc.deadbeef-1234.r", "f <- function(x)x+1\n", "r");
  const order = [];
  const yamarkCalls = [];
  const vscode = fakeVscode({
    documents: [document, vdoc],
    settings: {
      enabledFileExtensions: [".qmd", ".r"],
      runNextFormatter: false,
      useBundledExecutable: false,
    },
    onApplyEdit: async () => {
      order.push("executeFormatDocumentProvider:quarto");
      await vscode.commands.executeCommand("vscode.executeFormatDocumentProvider", document.uri);
    },
  });
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      order.push(`yamark:${path.basename(call.args.at(-1))}`);
      yamarkCalls.push(call);
      return "---\ntags: [r, code]\n---\n\n```{r}\nf <- function(x)x+1\n```\n";
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });
  vscode.languages.registerDocumentFormattingEditProvider([{ pattern: "**/*.qmd" }], {
    provideDocumentFormattingEdits: async () => {
      order.push("quarto");
      await vscode.commands.executeCommand("vscode.executeFormatDocumentProvider", vdoc.uri);
      return [];
    },
  });
  vscode.languages.registerDocumentFormattingEditProvider([{ pattern: "**/*.r" }], {
    provideDocumentFormattingEdits: async () => {
      order.push("air");
      return [];
    },
  });

  await api.formatDocument(document);

  assert.equal(yamarkCalls.length, 1);
  assert.deepEqual(order, [
    "yamark:notes.qmd",
    "executeFormatDocumentProvider:quarto",
    "quarto",
    "air",
  ]);
});

test("skips Quarto vdoc temp files", async () => {
  const document = fakeDocument(
    "/tmp/.vdoc.deadbeef-1234.r",
    "x <- 1\n",
    "r",
  );
  const lines = [];
  const channel = { appendLine: (line) => lines.push(line) };
  const logger = createChannelLogger(channel);
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".r"],
      runNextFormatter: true,
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    logger,
    runProcess: async () => {
      throw new Error("yamark should not run on vdoc files");
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(edits, []);
  assert.match(lines.join("\n"), /skipped: Quarto vdoc temp file/);
});

test("quarto executable chain skips yamark embedded formatters", async () => {
  const document = fakeDocument(
    "/tmp/notes.qmd",
    "---\ntags: [r,code]\n---\n\n```{r}\nf <- function(x)x+1\n```\n",
    "quarto",
  );
  const calls = [];
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".qmd"],
      runNextFormatter: true,
      nextFormatterExecutable: ["quarto-native", "--stdin-file-path", "${file}"],
    },
    async onExecuteCommand() {
      throw new Error("provider fallthrough should not run");
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      calls.push(call);
      if (call.command === "quarto-native") {
        assert.equal(
          call.input,
          "---\ntags: [r, code]\n---\n\n```{r}\nf <- function(x)x+1\n```\n",
        );
        return "---\ntags: [r, code]\n---\n\n```{r}\nf <- function(x) x + 1\n```\n";
      }
      return "---\ntags: [r, code]\n---\n\n```{r}\nf <- function(x)x+1\n```\n";
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.equal(calls.length, 2);
  assert.deepEqual(calls[0].args, [
    "format",
    "--skip-embedded-formatters",
    "--stdin-file-path",
    "/tmp/notes.qmd",
  ]);
  assert.deepEqual(calls[1].args, ["--stdin-file-path", "/tmp/notes.qmd"]);
  assert.equal(edits.length, 1);
  assert.equal(
    edits[0].newText,
    "---\ntags: [r, code]\n---\n\n```{r}\nf <- function(x) x + 1\n```\n",
  );
});

test("quarto runs embedded formatters when no executable chain is configured", async () => {
  const document = fakeDocument(
    "/tmp/notes.qmd",
    "---\ntags: [r,code]\n---\n\n```{r}\nf <- function(x)x+1\n```\n",
    "quarto",
  );
  const calls = [];
  const vscode = fakeVscode({
    settings: {
      enabledFileExtensions: [".qmd"],
      runNextFormatter: true,
    },
    async onExecuteCommand() {
      throw new Error("provider fallthrough should not run without an executable chain");
    },
  });
  const api = createYamarkExtension(vscode, {
    extensionRoot: "/extension",
    runProcess: async (call) => {
      calls.push(call);
      return call.input;
    },
  });

  await api.provideDocumentFormattingEdits(document);

  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].args, [
    "format",
    "--stdin-file-path",
    "/tmp/notes.qmd",
  ]);
});

function fakeDocument(fileName, text, languageId, options = {}) {
  const document = {
    fileName,
    languageId,
    text,
    uri: {
      fsPath: fileName,
      toString: () => `file://${fileName}`,
    },
    getText(range) {
      if (!range) {
        return this.text;
      }
      return this.text.slice(this.offsetAt(range.start), this.offsetAt(range.end));
    },
    positionAt(offset) {
      return { offset };
    },
    offsetAt(position) {
      return position.offset;
    },
  };
  if (Object.hasOwn(options, "isDirty")) {
    document.isDirty = options.isDirty;
  }
  if (Object.hasOwn(options, "version")) {
    document.version = options.version;
  }
  return document;
}

function fakeSelection(startOffset, endOffset) {
  return {
    start: { offset: startOffset },
    end: { offset: endOffset },
    isEmpty: startOffset === endOffset,
  };
}

function fakeVscode(options = {}) {
  const settings = options.settings || {};
  const editorSettings = options.editorSettings || {};
  const documents = options.documents || [];
  const sourceDocuments = new Map(
    documents.map((document) => [document.uri.toString(), document]),
  );
  const formattingProviders = [];
  const textDocumentContentProviders = new Map();
  const invalidatedContentDocuments = new Set();
  const registeredCommands = [];
  const registeredCommandHandlers = new Map();
  const executedCommands = [];
  const shownTextDocuments = [];
  const changedDocumentLanguages = [];
  class EventEmitter {
    constructor() {
      this.listeners = new Set();
      this.event = (listener) => {
        this.listeners.add(listener);
        return {
          dispose: () => this.listeners.delete(listener),
        };
      };
    }

    fire(value) {
      const results = [];
      for (const listener of this.listeners) {
        results.push(listener(value));
      }
      return results;
    }

    dispose() {
      this.listeners.clear();
    }
  }
  class Uri {
    constructor({ authority = "", fsPath, scheme, path: uriPath, query = "" }) {
      this.authority = authority;
      this.scheme = scheme;
      this.path = uriPath;
      this.fsPath = fsPath ?? uriPath;
      this.query = query;
    }

    static file(filePath) {
      const match = filePath.match(/^\/\/([^/]+)(\/.*)$/);
      return new Uri({
        authority: match ? match[1] : "",
        fsPath: filePath,
        scheme: "file",
        path: match ? match[2] : filePath,
      });
    }

    static parse(value) {
      const parsed = new URL(value);
      const scheme = parsed.protocol.slice(0, -1);
      const authority = parsed.host;
      const uriPath = decodeURIComponent(parsed.pathname);
      return new Uri({
        authority,
        fsPath:
          authority !== "" && scheme === "file"
            ? `//${authority}${uriPath}`
            : uriPath,
        scheme,
        path: uriPath,
        query: parsed.search.slice(1),
      });
    }

    with(changes) {
      const scheme = changes.scheme ?? this.scheme;
      const authority = changes.authority ?? this.authority;
      return new Uri({
        authority,
        fsPath:
          changes.fsPath ??
          (authority !== "" && scheme === "file"
            ? `//${authority}${changes.path ?? this.path}`
            : authority !== "" && scheme !== "file"
              ? changes.path ?? this.path
              : changes.path ?? this.path),
        scheme,
        path: changes.path ?? this.path,
        query: changes.query ?? this.query,
      });
    }

    toString() {
      const query = this.query === "" ? "" : `?${this.query}`;
      return `${this.scheme}://${this.authority}${this.path}${query}`;
    }
  }
  const didCloseTextDocument = new EventEmitter();
  const didChangeTabs = new EventEmitter();
  const fileSystemWatchers = [];
  const tabs = [];
  const tabGroup = {
    isActive: true,
    viewColumn: 1,
    get activeTab() {
      return tabs.find((tab) => tab.isActive);
    },
    tabs,
  };
  return {
    Range: class Range {
      constructor(start, end) {
        this.start = start;
        this.end = end;
      }
    },
    TextEdit: class TextEdit {
      static replace(range, newText) {
        return { range, newText };
      }
    },
    WorkspaceEdit: class WorkspaceEdit {
      constructor() {
        this.edits = [];
      }

      replace(uri, range, newText) {
        this.edits.push({ uri, range, newText });
      }
    },
    EventEmitter,
    RelativePattern: class RelativePattern {
      constructor(baseUri, pattern) {
        this.baseUri = baseUri;
        this.pattern = pattern;
      }
    },
    Uri,
    commands: {
      executeCommand: async (command, ...args) => {
        executedCommands.push({ command, args });
        if (command === "vscode.executeFormatDocumentProvider") {
          return await executeFormatDocumentProvider(
            formattingProviders,
            documents,
            args[0],
            args[1],
          );
        }
        if (options.onExecuteCommand) {
          return await options.onExecuteCommand(command, ...args);
        }
        if (registeredCommandHandlers.has(command)) {
          return await registeredCommandHandlers.get(command)(...args);
        }
      },
      registerCommand: (command, handler) => {
        registeredCommands.push(command);
        registeredCommandHandlers.set(command, handler);
        return disposable();
      },
      executedCommands,
      registeredCommands,
    },
    extensions: {
      getExtension: (id) => {
        const extension = options.extensions && options.extensions[id];
        if (typeof extension === "string") {
          return { extensionPath: extension };
        }
        return extension;
      },
    },
    languages: {
      registerDocumentFormattingEditProvider: (selector, provider) => {
        const entry = { selector, provider };
        formattingProviders.push(entry);
        return {
          dispose() {
            const index = formattingProviders.indexOf(entry);
            if (index >= 0) {
              formattingProviders.splice(index, 1);
            }
          },
        };
      },
      setTextDocumentLanguage: async (document, languageId) => {
        didCloseTextDocument.fire(document);
        document.languageId = languageId;
        changedDocumentLanguages.push({ document, languageId });
        return document;
      },
      changedDocumentLanguages,
    },
    window: {
      activeTextEditor: options.activeTextEditor,
      closeTextDocumentTab: async (document) => {
        const index = tabs.findIndex(
          (tab) => tab.input.uri.toString() === document.uri.toString(),
        );
        if (index < 0) {
          return;
        }
        const [tab] = tabs.splice(index, 1);
        await Promise.all(
          didChangeTabs.fire({ changed: [], closed: [tab], opened: [] }),
        );
      },
      showErrorMessage: (message) => {
        throw new Error(message);
      },
      showWarningMessage: (message) => {
        throw new Error(message);
      },
      setStatusBarMessage: (message) => {
        if (options.onStatusBarMessage) {
          options.onStatusBarMessage(message);
        }
        return disposable();
      },
      showTextDocument: async (document, showOptions) => {
        shownTextDocuments.push({ document, options: showOptions });
        if (!tabs.some((tab) => tab.input.uri.toString() === document.uri.toString())) {
          const tab = {
            group: tabGroup,
            input: { uri: document.uri },
            isActive: true,
            isDirty: false,
            isPinned: showOptions.preview !== true,
            isPreview: showOptions.preview === true,
            label: path.basename(document.fileName),
          };
          tabs.push(tab);
          didChangeTabs.fire({ changed: [], closed: [], opened: [tab] });
        }
        return { document };
      },
      shownTextDocuments,
      tabGroups: {
        activeTabGroup: tabGroup,
        all: [tabGroup],
        close: async (tabOrTabs) => {
          const closing = Array.isArray(tabOrTabs) ? tabOrTabs : [tabOrTabs];
          const closed = [];
          for (const tab of closing) {
            const index = tabs.indexOf(tab);
            if (index >= 0) {
              tabs.splice(index, 1);
              closed.push(tab);
            }
          }
          await Promise.all(
            didChangeTabs.fire({ changed: [], closed, opened: [] }),
          );
          return closed.length === closing.length;
        },
        onDidChangeTabs: didChangeTabs.event,
      },
    },
    workspace: {
      applyEdit: async (edit) => {
        if (options.onApplyEdit) {
          await options.onApplyEdit(edit);
        }
        return true;
      },
      getConfiguration: (section, scope) => {
        assert.ok(section === "yamark" || section === "editor");
        const base = section === "yamark" ? settings : editorSettings;
        const overrides = languageOverrides(base, scope);
        return {
          get: (key, defaultValue) => {
            if (overrides && Object.prototype.hasOwnProperty.call(overrides, key)) {
              return overrides[key];
            }
            return Object.prototype.hasOwnProperty.call(base, key)
              ? base[key]
              : defaultValue;
          },
        };
      },
      onDidChangeConfiguration: () => disposable(),
      onDidCloseTextDocument: didCloseTextDocument.event,
      createFileSystemWatcher: (
        globPattern,
        ignoreCreateEvents = false,
        ignoreChangeEvents = false,
        ignoreDeleteEvents = false,
      ) => {
        const didChange = new EventEmitter();
        const watcher = {
          globPattern,
          ignoreCreateEvents,
          ignoreChangeEvents,
          ignoreDeleteEvents,
          onDidChange: didChange.event,
          dispose() {
            const index = fileSystemWatchers.indexOf(watcher);
            if (index >= 0) {
              fileSystemWatchers.splice(index, 1);
            }
            didChange.dispose();
          },
          fireChange(uri) {
            return didChange.fire(uri);
          },
        };
        fileSystemWatchers.push(watcher);
        return watcher;
      },
      fireFileChange: async (uri) => {
        const results = fileSystemWatchers
          .filter(
            (watcher) =>
              watcher.globPattern.baseUri.toString() === uri.toString(),
          )
          .flatMap((watcher) => watcher.fireChange(uri));
        await Promise.all(results);
      },
      fs: {
        readFile: async (uri) => {
          const document = sourceDocuments.get(uri.toString());
          if (!document) {
            throw new Error(`missing fake file: ${uri.toString()}`);
          }
          return Buffer.from(document.getText(), "utf8");
        },
      },
      openTextDocument: async (target) => {
        if (target && typeof target.getText === "function") {
          return target;
        }
        const targetKey = target && target.toString();
        const existing = documents.find(
          (document) => document.uri && document.uri.toString() === targetKey,
        );
        if (existing) {
          if (invalidatedContentDocuments.delete(targetKey)) {
            const provider = textDocumentContentProviders.get(target.scheme);
            existing.text = await provider.provideTextDocumentContent(target);
          }
          return existing;
        }
        if (!target || !textDocumentContentProviders.has(target.scheme)) {
          return resolveFakeDocument(documents, target);
        }
        const provider = textDocumentContentProviders.get(target.scheme);
        const text = await provider.provideTextDocumentContent(target);
        const document = fakeDocument(
          target.fsPath,
          text,
          inferredLanguageId(target.fsPath),
        );
        document.uri = target;
        documents.push(document);
        return document;
      },
      closeTextDocument: (document) => {
        const index = documents.indexOf(document);
        if (index >= 0) {
          documents.splice(index, 1);
        }
        didCloseTextDocument.fire(document);
      },
      registerTextDocumentContentProvider: (scheme, provider) => {
        textDocumentContentProviders.set(scheme, provider);
        const subscription = provider.onDidChange
          ? provider.onDidChange((uri) => {
              invalidatedContentDocuments.add(uri.toString());
            })
          : disposable();
        return {
          dispose() {
            subscription.dispose();
            textDocumentContentProviders.delete(scheme);
          },
        };
      },
      textDocuments: documents,
      registeredTextDocumentContentProviders: textDocumentContentProviders,
    },
  };
}

function inferredLanguageId(fileName) {
  switch (path.extname(fileName).toLowerCase()) {
    case ".md":
    case ".rmd":
      return "markdown";
    case ".qmd":
      return "quarto";
    case ".yaml":
    case ".yml":
      return "yaml";
    case ".py":
      return "python";
    case ".r":
      return "r";
    default:
      return "plaintext";
  }
}

async function executeFormatDocumentProvider(formattingProviders, documents, target, options) {
  const document = resolveFakeDocument(documents, target);
  const edits = [];
  for (const entry of formattingProviders) {
    if (!documentMatchesSelector(entry.selector, document)) {
      continue;
    }
    const providerEdits = await entry.provider.provideDocumentFormattingEdits(
      document,
      options || {},
    );
    edits.push(...providerEdits);
  }
  return edits;
}

function resolveFakeDocument(documents, target) {
  if (target && typeof target.getText === "function") {
    return target;
  }
  const targetPath = target && (target.fsPath || target.fileName);
  const document = documents.find((candidate) => {
    return (
      candidate === target ||
      candidate.fileName === targetPath ||
      (candidate.uri && candidate.uri.fsPath === targetPath)
    );
  });
  assert.ok(document, `fake document not registered for ${targetPath}`);
  return document;
}

function documentMatchesSelector(selector, document) {
  const entries = Array.isArray(selector) ? selector : [selector];
  return entries.some((entry) => documentMatchesSelectorEntry(entry, document));
}

function documentMatchesSelectorEntry(entry, document) {
  if (typeof entry === "string") {
    return entry === document.languageId;
  }
  if (entry.scheme && entry.scheme !== document.uri.scheme) {
    return false;
  }
  if (entry.language && entry.language !== document.languageId) {
    return false;
  }
  if (entry.pattern) {
    return documentMatchesPattern(entry.pattern, document);
  }
  return true;
}

function documentMatchesPattern(pattern, document) {
  const prefix = "**/*";
  assert.ok(pattern.startsWith(prefix), `unsupported fake selector pattern: ${pattern}`);
  return globSuffixRegex(pattern.slice(prefix.length)).test(document.fileName);
}

function globSuffixRegex(suffix) {
  let source = "";
  for (let i = 0; i < suffix.length; i += 1) {
    const ch = suffix[i];
    if (ch === "[") {
      const close = suffix.indexOf("]", i + 1);
      assert.ok(close > i, `unsupported fake selector pattern suffix: ${suffix}`);
      source += suffix.slice(i, close + 1);
      i = close;
      continue;
    }
    source += ch.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
  }
  return new RegExp(`${source}$`);
}

function languageOverrides(values, scope) {
  if (!scope || !scope.languageId) {
    return undefined;
  }
  const key = `[${scope.languageId}]`;
  const overrides = values[key];
  if (!overrides || typeof overrides !== "object") {
    return undefined;
  }
  return overrides;
}

function disposable() {
  return { dispose() {} };
}
