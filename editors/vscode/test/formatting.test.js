const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const {
  createChannelLogger,
  createYamarkExtension,
} = require("../src/core");
const { fakeDocument, fakeSelection, fakeVscode } = require("./support/fake-vscode");

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
      args: ["format", "--stdin-file-path", path.resolve("/tmp/config.yaml")],
      input: "items: [a,b]\n",
      cwd: path.resolve("/tmp"),
    },
  ]);
});

for (const command of ["yamark.formatDocument", "yamark.formatSelectionAsMarkdown"]) {
  test(`${command} preserves edits made while the formatter is running`, async () => {
    const document = fakeDocument("/tmp/notes.md", "Original   text.\n", "markdown", {
      version: 1,
    });
    const editor = { document, selection: fakeSelection(0, document.text.length) };
    const started = Promise.withResolvers();
    const formatted = Promise.withResolvers();
    const applied = [];
    const vscode = fakeVscode({
      documents: [document],
      activeTextEditor: editor,
      onApplyEdit: (edit) => applied.push(edit),
    });
    const api = createYamarkExtension(vscode, {
      runProcess: async () => {
        started.resolve();
        return formatted.promise;
      },
    });
    api.activate({ extensionPath: "/extension", subscriptions: [] });

    const pending = vscode.commands.executeCommand(command);
    await started.promise;
    document.text = "New user text.\n";
    document.version += 1;
    formatted.resolve("Original text.\n");
    await pending;

    assert.deepEqual(applied, []);
    assert.equal(document.text, "New user text.\n");
  });
}

test("format provider discards output for a changed document", async () => {
  const document = fakeDocument("/tmp/notes.md", "Original   text.\n", "markdown", {
    version: 1,
  });
  const started = Promise.withResolvers();
  const formatted = Promise.withResolvers();
  const api = createYamarkExtension(fakeVscode(), {
    runProcess: async () => {
      started.resolve();
      return formatted.promise;
    },
  });

  const pending = api.provideDocumentFormattingEdits(document);
  await started.promise;
  document.text = "New user text.\n";
  document.version += 1;
  formatted.resolve("Original text.\n");

  assert.deepEqual(await pending, []);
});

test("format selection keeps the original range when the selection moves", async () => {
  const document = fakeDocument("/tmp/notes.md", "First   text. Second text.\n", "markdown", {
    version: 1,
  });
  const selection = fakeSelection(0, 13);
  const editor = { document, selection };
  const started = Promise.withResolvers();
  const formatted = Promise.withResolvers();
  const applied = [];
  const vscode = fakeVscode({
    documents: [document],
    activeTextEditor: editor,
    onApplyEdit: (edit) => applied.push(edit),
  });
  const api = createYamarkExtension(vscode, {
    runProcess: async (call) => {
      assert.equal(call.input, "First   text.");
      started.resolve();
      return formatted.promise;
    },
  });
  api.activate({ extensionPath: "/extension", subscriptions: [] });

  const pending = vscode.commands.executeCommand("yamark.formatSelectionAsMarkdown");
  await started.promise;
  editor.selection = fakeSelection(14, 26);
  formatted.resolve("First text.");
  await pending;

  assert.equal(applied.length, 1);
  assert.deepEqual(applied[0].edits, [
    { uri: document.uri, range: selection, newText: "First text." },
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
      args: ["format", "--stdin-file-path", path.resolve("/tmp/prompt.py.md")],
      input: "- a\n- b",
      cwd: path.resolve("/tmp"),
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

