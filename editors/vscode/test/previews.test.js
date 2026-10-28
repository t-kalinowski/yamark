const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const {
  createYamarkExtension,
} = require("../src/core");
const { fakeDocument, fakeVscode } = require("./support/fake-vscode");

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
    args: ["format", "--stdin-file-path", path.resolve("/tmp/notes.md")],
    input: "#   Notes ##\n",
    cwd: path.resolve("/tmp"),
  });
  assert.deepEqual(calls[1], {
    command: "yamark",
    args: ["to-yaml", "--stdin-file-path", path.resolve("/tmp/data.json")],
    input: '{"answer":42}\n',
    cwd: path.resolve("/tmp"),
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
        path.resolve("/tmp/notes.qmd"),
      ],
      input: "---\ntags: [r,code]\n---\n\n```{r}\nx<-1\n```\n",
      cwd: path.resolve("/tmp"),
    },
    {
      command: "quarto-native",
      args: ["--stdin-file-path", path.resolve("/tmp/notes.qmd")],
      input: "---\ntags: [r, code]\n---\n\n```{r}\nx<-1\n```\n",
      cwd: path.resolve("/tmp"),
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
      args: ["to-yaml", "--stdin-file-path", path.resolve("/tmp/data.json5")],
      input: "{answer:42}\n",
      cwd: path.resolve("/tmp"),
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
    path.resolve("/tmp/data.json5"),
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
  let directFileChange;
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input.includes('"id":2')
        ? "---\nid: 1\n---\nid: 2\n"
        : "---\nid: 1\n";
    },
    watchPath: (sourcePath, onChange) => {
      assert.equal(sourcePath, path.resolve("/tmp/events.jsonl"));
      directFileChange = onChange;
      return {
        close() {},
        on() {
          return this;
        },
      };
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
  await directFileChange();

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

test("JSONL YAML preview refreshes when its open source reloads from disk", async () => {
  const source = fakeDocument(
    "/tmp/events.jsonl",
    '{"id":1}\n',
    "jsonl",
    { isDirty: false, version: 1 },
  );
  const calls = [];
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      if (call.input.includes('"id":3')) {
        return "---\nid: 1\n---\nid: 3\n";
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

  await vscode.workspace.reloadTextDocumentFromDisk(
    source,
    source.text + '{"id":2}\n',
  );
  assert.deepEqual(calls.map((call) => call.input), [
    '{"id":1}\n',
    '{"id":1}\n{"id":2}\n',
  ]);

  await vscode.workspace.fireFileChange(source.uri);
  assert.equal(calls.length, 2);

  await vscode.workspace.reloadTextDocumentFromDisk(
    source,
    source.text.replace('"id":2', '"id":3'),
  );
  assert.equal(calls.length, 3);
  assert.equal(
    await provider.provideTextDocumentContent(previewUri),
    "---\nid: 1\n---\nid: 3\n",
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
  let directWatchClosed = false;
  const vscode = fakeVscode({ documents: [source] });
  source.uri = vscode.Uri.file(source.fileName);
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      calls.push(call);
      return call.input.includes("43")
        ? "---\nanswer: 42\n---\nanswer: 43\n"
        : "answer: 42\n";
    },
    watchPath: () => ({
      close() {
        directWatchClosed = true;
      },
      on() {
        return this;
      },
    }),
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
  assert.equal(directWatchClosed, true);
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

