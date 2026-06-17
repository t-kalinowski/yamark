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
  const formattingProviders = [];
  const registeredCommands = [];
  const registeredCommandHandlers = new Map();
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
    commands: {
      executeCommand: async (command, ...args) => {
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
      registeredCommands,
    },
    extensions: {
      getExtension: (id) => {
        const extensionPath = options.extensions && options.extensions[id];
        return extensionPath ? { extensionPath } : undefined;
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
    },
    window: {
      activeTextEditor: options.activeTextEditor,
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
    },
  };
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
