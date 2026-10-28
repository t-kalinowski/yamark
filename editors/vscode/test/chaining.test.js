const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const {
  createChannelLogger,
  createYamarkExtension,
} = require("../src/core");
const { fakeDocument, fakeVscode } = require("./support/fake-vscode");

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
    args: ["format", "--stdin-file-path", path.resolve("/tmp/analysis.R")],
    input: "call(\n  x\n)\n",
    cwd: path.resolve("/tmp"),
  });
  assert.deepEqual(calls[1], {
    command: "/extensions/air/bundled/bin/air",
    args: ["format", "--stdin-file-path", path.resolve("/tmp/analysis.R")],
    input: "call(\n  yamarked = TRUE\n)\n",
    cwd: path.resolve("/tmp"),
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
      if (call.command.endsWith(`/bundled/libs/bin/ruff${process.platform === "win32" ? ".exe" : ""}`)) {
        assert.equal(call.input, "x=1\n");
        return "x = 1\n";
      }
      assert.fail(`unexpected command: ${call.command}`);
    },
  });

  const edits = await api.provideDocumentFormattingEdits(document);

  assert.deepEqual(calls.map((call) => call.command), [
    yamarkCommand,
    `/extensions/ruff/bundled/libs/bin/ruff${process.platform === "win32" ? ".exe" : ""}`,
  ]);
  assert.deepEqual(calls[1].args, [
    "format",
    "--stdin-filename",
    path.resolve("/tmp/analysis.py"),
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
    path.resolve("/tmp/notes.qmd"),
  ]);
  assert.deepEqual(calls[1].args, ["--stdin-file-path", path.resolve("/tmp/notes.qmd")]);
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
    path.resolve("/tmp/notes.qmd"),
  ]);
});

