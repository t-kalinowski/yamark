const path = require("node:path");
const { spawn } = require("node:child_process");

const DEFAULT_FILE_EXTENSIONS = Object.freeze([".md", ".qmd", ".yaml", ".yml"]);

function createYamarkExtension(vscode, runtime = {}) {
  const runProcess = runtime.runProcess || runProcessWithSpawn;
  const platform = runtime.platform || process.platform;
  const arch = runtime.arch || process.arch;
  const logger = runtime.logger || createNullLogger();
  let extensionRoot = runtime.extensionRoot;
  let providerDisposable;
  let providerSuppressionDepth = 0;
  const providerSuppressionReasons = [];

  async function provideDocumentFormattingEdits(document, options = {}) {
    const op = options.op || logger.startOp("format");
    const ownsOp = !options.op;
    const trigger = options.trigger || "provider";
    op.log(documentLogLine(document, trigger));
    op.log(`formatter step=start name=vscode-provider trigger=${trigger}`);

    if (providerSuppressionDepth > 0) {
      const reason = currentProviderSuppressionReason(providerSuppressionReasons);
      op.log(`suppression action=hit depth=${providerSuppressionDepth} reason=${reason}`);
      logProviderReturn(op, ownsOp, 0, "suppressed");
      return [];
    }

    if (!isEnabledDocument(vscode, document)) {
      logProviderReturn(op, ownsOp, 0, "disabled-extension");
      return [];
    }
    if (isQuartoVdocPath(document)) {
      op.log(`skipped: Quarto vdoc temp file`);
      logProviderReturn(op, ownsOp, 0, "quarto-vdoc");
      return [];
    }

    const settings = readSettings(vscode, document);
    logUnsupportedCommandSettings(vscode, document, op);
    op.log(
      `settings runNextFormatter=${settings.runNextFormatter} ` +
        `nextFormatterExecutable=${formatArgvForLog(settings.nextFormatterExecutable)} ` +
        `useBundledExecutable=${settings.useBundledExecutable}`,
    );

    const runtime = { arch, extensionRoot, platform };
    const originalText = document.getText();

    try {
      let formatted = await formatTextWithYamark(
        vscode,
        runProcess,
        document,
        originalText,
        op,
        runtime,
      );

      if (settings.runNextFormatter && settings.nextFormatterExecutable.length > 0) {
        formatted = await runNextFormatterExecutable(
          vscode,
          runProcess,
          document,
          formatted,
          settings,
          op,
        );
      } else if (settings.nextFormatterExecutable.length > 0) {
        op.log("formatter step=skip name=next-executable reason=runNextFormatter-false");
      } else if (settings.runNextFormatter) {
        op.log("formatter step=skip name=next-executable reason=not-configured");
      }

      if (formatted === originalText) {
        logProviderReturn(op, ownsOp, 0, "no-change", formatted);
        return [];
      }
      logProviderReturn(op, ownsOp, 1, "return-edits", formatted);
      return [wholeDocumentEdit(vscode, document, formatted)];
    } catch (err) {
      if (ownsOp) {
        op.error(err);
      }
      throw err;
    }
  }

  async function formatDocument(document) {
    const target = document || activeDocument(vscode);
    if (!target) {
      vscode.window.showWarningMessage("Yamark: no active document.");
      return;
    }
    const op = logger.startOp("format");
    try {
      const edits = await provideDocumentFormattingEdits(target, { op, trigger: "command" });
      if (edits.length === 0) {
        op.end("done applied.edits=0");
        return;
      }
      await withProviderSuppression(op, "apply-command-edits", () =>
        applyDocumentEdits(vscode, target, edits, op),
      );
      op.end(`done applied.edits=${edits.length}`);
    } catch (err) {
      op.error(err);
      throw err;
    }
  }

  async function formatSelectionAsMarkdown(editor) {
    const target = editor || activeEditor(vscode);
    if (!target || !target.document) {
      vscode.window.showWarningMessage("Yamark: no active editor.");
      return;
    }
    if (!target.selection) {
      throw new Error("Yamark needs an editor selection");
    }
    if (target.selection.isEmpty) {
      vscode.window.setStatusBarMessage("Yamark: no text selected.", 3000);
      return;
    }

    const document = target.document;
    const originalText = document.getText(target.selection);
    const op = logger.startOp("format-selection");
    try {
      op.log(documentLogLine(document, "selection-command"));
      op.log("formatter step=start name=vscode-selection trigger=command");
      const runtime = { arch, extensionRoot, platform };
      const formatted = await formatTextWithYamark(
        vscode,
        runProcess,
        document,
        originalText,
        op,
        runtime,
        {
          stdinFilePath: markdownSelectionPath(document),
          skipEmbeddedFormatters: false,
        },
      );

      if (formatted === originalText) {
        logSelectionReturn(op, 0, "no-change", formatted);
        op.end("done applied.edits=0 reason=no-change");
        return;
      }

      const edit = vscode.TextEdit.replace(target.selection, formatted);
      await withProviderSuppression(op, "apply-selection-edits", () =>
        applyDocumentEdits(vscode, document, [edit], op),
      );
      logSelectionReturn(op, 1, "applied", formatted);
      op.end("done applied.edits=1");
    } catch (err) {
      op.error(err);
      throw err;
    }
  }

  function activate(context) {
    extensionRoot = context.extensionPath;
    logger.log(`activate extensionRoot=${extensionRoot} platform=${platform} arch=${arch}`);
    registerProvider();
    context.subscriptions.push({
      dispose() {
        if (providerDisposable) {
          providerDisposable.dispose();
        }
      },
    });
    context.subscriptions.push(
      vscode.commands.registerCommand("yamark.formatDocument", () => formatDocument()),
    );
    context.subscriptions.push(
      vscode.commands.registerCommand("yamark.formatSelectionAsMarkdown", () =>
        formatSelectionAsMarkdown(),
      ),
    );
    context.subscriptions.push(
      vscode.commands.registerCommand("yamark.showLog", () => logger.show && logger.show()),
    );
    context.subscriptions.push(
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("yamark.enabledFileExtensions")) {
          logger.log("yamark.enabledFileExtensions changed; re-registering provider");
          registerProvider();
        }
      }),
    );
  }

  function registerProvider() {
    if (providerDisposable) {
      providerDisposable.dispose();
    }
    const settings = readSettings(vscode);
    const selector = documentSelector(settings.enabledFileExtensions);
    logger.log(
      `registerProvider extensions=${settings.enabledFileExtensions.join(",")} patterns=${selector.length}`,
    );
    providerDisposable = vscode.languages.registerDocumentFormattingEditProvider(
      selector,
      { provideDocumentFormattingEdits },
    );
  }

  async function withProviderSuppression(op, reason, callback) {
    providerSuppressionDepth += 1;
    providerSuppressionReasons.push(reason);
    op.log(`suppression action=add depth=${providerSuppressionDepth} reason=${reason}`);
    try {
      return await callback();
    } finally {
      providerSuppressionReasons.pop();
      providerSuppressionDepth -= 1;
      op.log(`suppression action=remove depth=${providerSuppressionDepth} reason=${reason}`);
    }
  }

  return {
    activate,
    formatDocument,
    formatSelectionAsMarkdown,
    isEnabledDocument: (document) => isEnabledDocument(vscode, document),
    provideDocumentFormattingEdits,
  };
}

function readSettings(vscode, document) {
  const config = vscode.workspace.getConfiguration("yamark", documentConfigurationScope(document));
  return {
    executable: requireNonEmptyString(config.get("executable", "yamark"), "yamark.executable"),
    useBundledExecutable: requireBoolean(
      config.get("useBundledExecutable", false),
      "yamark.useBundledExecutable",
    ),
    enabledFileExtensions: normalizeFileExtensions(
      config.get("enabledFileExtensions", DEFAULT_FILE_EXTENSIONS),
    ),
    extraArguments: requireStringArray(
      config.get("extraArguments", []),
      "yamark.extraArguments",
    ),
    runNextFormatter: requireBoolean(
      config.get("runNextFormatter", true),
      "yamark.runNextFormatter",
    ),
    nextFormatterExecutable: requireNonEmptyStringArray(
      config.get("nextFormatterExecutable", []),
      "yamark.nextFormatterExecutable",
    ),
  };
}

function documentConfigurationScope(document) {
  if (!document || !document.uri) {
    return undefined;
  }
  if (document.languageId) {
    return { languageId: document.languageId, uri: document.uri };
  }
  return document.uri;
}

function normalizeFileExtensions(extensions) {
  const values = requireStringArray(extensions, "yamark.enabledFileExtensions");
  const seen = new Set();
  const normalized = [];

  for (const value of values) {
    const trimmed = value.trim();
    if (trimmed === "") {
      throw new Error("yamark.enabledFileExtensions contains an empty extension");
    }

    const extension = (trimmed.startsWith(".") ? trimmed : `.${trimmed}`).toLowerCase();
    if (extension === ".") {
      throw new Error("yamark.enabledFileExtensions contains an invalid extension");
    }
    if (!seen.has(extension)) {
      seen.add(extension);
      normalized.push(extension);
    }
  }

  return normalized;
}

function shouldSkipEmbeddedFormatters(document, settings) {
  return (
    settings.runNextFormatter &&
    settings.nextFormatterExecutable.length > 0 &&
    isQuartoDocument(document)
  );
}

function isQuartoDocument(document) {
  return (
    document.languageId === "quarto" ||
    path.extname(documentPath(document)).toLowerCase() === ".qmd"
  );
}

function documentSelector(extensions) {
  const seen = new Set();
  const selector = [];

  for (const extension of normalizeFileExtensions(extensions)) {
    for (const pattern of extensionPatterns(extension)) {
      if (!seen.has(pattern)) {
        seen.add(pattern);
        selector.push({ pattern });
      }
    }
  }

  return selector;
}

function isEnabledDocument(vscode, document) {
  const filePath = documentPath(document);
  const extension = path.extname(filePath).toLowerCase();
  return readSettings(vscode, document).enabledFileExtensions.some(
    (enabledExtension) => enabledExtension === extension,
  );
}

async function formatTextWithYamark(
  vscode,
  runProcess,
  document,
  input,
  op,
  runtime,
  options = {},
) {
  const settings = readSettings(vscode, document);
  const filePath = options.stdinFilePath || documentPath(document);
  const skipEmbeddedFormatters = Object.prototype.hasOwnProperty.call(
    options,
    "skipEmbeddedFormatters",
  )
    ? options.skipEmbeddedFormatters
    : shouldSkipEmbeddedFormatters(document, settings);
  const args = [
    "format",
    ...settings.extraArguments,
    ...(skipEmbeddedFormatters ? ["--skip-embedded-formatters"] : []),
    "--stdin-file-path",
    filePath,
  ];
  const command = resolveExecutable(settings, runtime);
  op.log(
    `formatter step=start name=yamark kind=process command=${command} ` +
      `args=${JSON.stringify(args)} input.bytes=${Buffer.byteLength(input, "utf8")}`,
  );
  const t = Date.now();
  try {
    const output = await runProcess({
      command,
      args,
      input,
      cwd: path.dirname(filePath),
    });
    op.log(
      `formatter step=end name=yamark kind=process edits=${editCount(input, output)} ` +
        `applied=false captured=true output.bytes=${Buffer.byteLength(output, "utf8")} ` +
        `changed=${output !== input} dt.ms=${Date.now() - t}`,
    );
    return output;
  } catch (err) {
    op.log(
      `formatter step=error name=yamark kind=process dt.ms=${Date.now() - t} ` +
        `err=${errorMessage(err)}`,
    );
    throw err;
  }
}

function resolveExecutable(settings, runtime) {
  if (!settings.useBundledExecutable) {
    return settings.executable;
  }
  if (!runtime.extensionRoot) {
    throw new Error("Yamark needs an extension root to use the bundled executable");
  }
  return bundledExecutablePath(runtime.extensionRoot, runtime.platform, runtime.arch);
}

function bundledExecutablePath(extensionRoot, platform, arch) {
  const executable = platform === "win32" ? "yamark.exe" : "yamark";
  return path.join(extensionRoot, "bin", `${platform}-${arch}`, executable);
}

async function applyDocumentEdits(vscode, document, edits, op) {
  if (op) {
    op.log(`apply edits step=start count=${edits.length}`);
  }
  const workspaceEdit = new vscode.WorkspaceEdit();
  for (const edit of edits) {
    workspaceEdit.replace(document.uri, edit.range, edit.newText);
  }

  const applied = await vscode.workspace.applyEdit(workspaceEdit);
  if (op) {
    op.log(`apply edits step=end count=${edits.length} applied=${applied}`);
  }
  if (!applied) {
    throw new Error("Yamark: failed to apply formatted text");
  }
  return true;
}

async function runNextFormatterExecutable(vscode, runProcess, document, input, settings, op) {
  const filePath = documentPath(document);
  const argv = settings.nextFormatterExecutable.map((arg) =>
    expandPlaceholders(vscode, arg, document),
  );
  const [command, ...args] = argv;
  op.log(
    `formatter step=start name=next-executable kind=process command=${command} args=${JSON.stringify(args)} ` +
      `input.bytes=${Buffer.byteLength(input, "utf8")}`,
  );
  const t = Date.now();
  try {
    const output = await runProcess({
      command,
      args,
      input,
      cwd: path.dirname(filePath),
    });
    op.log(
      `formatter step=end name=next-executable kind=process edits=${editCount(input, output)} ` +
        `applied=false captured=true output.bytes=${Buffer.byteLength(output, "utf8")} ` +
        `changed=${output !== input} dt.ms=${Date.now() - t}`,
    );
    return output;
  } catch (err) {
    op.log(
      `formatter step=error name=next-executable kind=process dt.ms=${Date.now() - t} ` +
        `err=${errorMessage(err)}`,
    );
    throw err;
  }
}

function isQuartoVdocPath(document) {
  const base = path.basename(documentPath(document));
  return /^\.vdoc\./.test(base);
}

function expandPlaceholders(vscode, value, document) {
  const filePath = documentPath(document);
  return value
    .replace(/\$\{file\}/g, filePath)
    .replace(/\$\{fileDirname\}/g, path.dirname(filePath))
    .replace(/\$\{exe\}/g, process.platform === "win32" ? ".exe" : "")
    .replace(/\$\{extension:([^}]+)\}/g, (_match, id) => extensionPath(vscode, id));
}

function extensionPath(vscode, id) {
  if (!vscode.extensions || typeof vscode.extensions.getExtension !== "function") {
    throw new Error("Yamark: this VS Code API does not expose installed extensions");
  }
  const extensionId = id.toLowerCase();
  const extension = vscode.extensions.getExtension(extensionId);
  if (!extension || !extension.extensionPath) {
    throw new Error(`Yamark: extension not found: ${extensionId}`);
  }
  return extension.extensionPath;
}

function wholeDocumentEdit(vscode, document, newText) {
  const range = new vscode.Range(
    document.positionAt(0),
    document.positionAt(document.getText().length),
  );
  return vscode.TextEdit.replace(range, newText);
}

function runProcessWithSpawn({ command, args, input, cwd }) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd, windowsHide: true });
    let stdout = "";
    let stderr = "";

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve(stdout);
        return;
      }

      const message =
        stderr.trim() || stdout.trim() || `yamark exited with code ${code}`;
      reject(new Error(message));
    });

    child.stdin.end(input);
  });
}

function activeDocument(vscode) {
  const editor = activeEditor(vscode);
  return editor && editor.document;
}

function activeEditor(vscode) {
  return vscode.window.activeTextEditor;
}

function documentPath(document) {
  const filePath = (document.uri && document.uri.fsPath) || document.fileName;
  if (!filePath) {
    throw new Error("Yamark needs a document with a file path");
  }
  return path.resolve(filePath);
}

function markdownSelectionPath(document) {
  return `${documentPath(document)}.md`;
}

function documentLogLine(document, trigger) {
  return (
    `document uri=${documentUri(document)} path=${documentPath(document)} ` +
    `languageId=${document.languageId || "?"} version=${optionalLogValue(document.version)} ` +
    `dirty=${optionalLogValue(document.isDirty)} trigger=${trigger}`
  );
}

function documentUri(document) {
  if (!document.uri) {
    return "n/a";
  }
  if (typeof document.uri.toString === "function") {
    return document.uri.toString();
  }
  return String(document.uri);
}

function optionalLogValue(value) {
  return value === undefined ? "n/a" : String(value);
}

function logUnsupportedCommandSettings(vscode, document, op) {
  const config = vscode.workspace.getConfiguration("yamark", documentConfigurationScope(document));
  if (
    config.get("nextFormatterCommand", undefined) !== undefined ||
    config.get("nextFormatterCommands", undefined) !== undefined
  ) {
    op.log("formatter step=skip name=legacy-command reason=unsupported");
  }
}

function logProviderReturn(op, ownsOp, edits, reason, finalText) {
  const finalBytes =
    finalText === undefined ? "" : ` final.bytes=${Buffer.byteLength(finalText, "utf8")}`;
  op.log(
    `formatter step=end name=vscode-provider edits=${edits} applied=false ` +
      `captured=${edits > 0} reason=${reason}${finalBytes}`,
  );
  op.log(`provider return edits count=${edits}${finalBytes} reason=${reason}`);
  if (ownsOp) {
    op.end(`done edits=${edits}${finalBytes} reason=${reason}`);
  }
}

function logSelectionReturn(op, edits, reason, finalText) {
  const finalBytes =
    finalText === undefined ? "" : ` final.bytes=${Buffer.byteLength(finalText, "utf8")}`;
  op.log(
    `formatter step=end name=vscode-selection edits=${edits} applied=${edits > 0} ` +
      `captured=${edits > 0} reason=${reason}${finalBytes}`,
  );
}

function currentProviderSuppressionReason(reasons) {
  return reasons.length > 0 ? reasons[reasons.length - 1] : "unknown";
}

function editCount(input, output) {
  return input === output ? 0 : 1;
}

function errorMessage(err) {
  return err && err.message ? err.message : String(err);
}

function extensionPatterns(extension) {
  return [`**/*${caseInsensitiveExtensionGlob(extension)}`];
}

function caseInsensitiveExtensionGlob(extension) {
  let pattern = "";
  for (const ch of extension) {
    const lower = ch.toLowerCase();
    const upper = ch.toUpperCase();
    pattern += lower !== upper ? `[${lower}${upper}]` : ch;
  }
  return pattern;
}

function requireBoolean(value, name) {
  if (typeof value !== "boolean") {
    throw new Error(`${name} must be a boolean`);
  }
  return value;
}

function requireNonEmptyString(value, name) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${name} must be a non-empty string`);
  }
  return value;
}

function requireStringArray(value, name) {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) {
    throw new Error(`${name} must be an array of strings`);
  }
  return value;
}

function requireNonEmptyStringArray(value, name) {
  const values = requireStringArray(value, name);
  if (values.some((entry) => entry.trim() === "")) {
    throw new Error(`${name} must not contain empty strings`);
  }
  return values;
}

function formatArgvForLog(argv) {
  return argv.length > 0 ? JSON.stringify(argv) : "(none)";
}

function createNullLogger() {
  const op = { log() {}, error() {}, end() {} };
  return {
    log() {},
    error() {},
    startOp() {
      return op;
    },
    show() {},
  };
}

function createChannelLogger(channel) {
  function stamp() {
    return new Date().toISOString();
  }
  function write(line) {
    channel.appendLine(line);
  }
  return {
    log(message) {
      write(`[${stamp()}] ${message}`);
    },
    error(message) {
      write(`[${stamp()}] ERROR ${message}`);
    },
    startOp(name) {
      const id = Math.random().toString(36).slice(2, 8);
      const t0 = Date.now();
      const prefix = `[${name} ${id}]`;
      write(`[${stamp()}] ${prefix} begin`);
      return {
        log(message) {
          write(`[${stamp()}] ${prefix} t+${Date.now() - t0}ms ${message}`);
        },
        error(err) {
          const message = err && err.message ? err.message : String(err);
          write(`[${stamp()}] ${prefix} t+${Date.now() - t0}ms ERROR ${message}`);
        },
        end(message) {
          const suffix = message ? ` ${message}` : "";
          write(`[${stamp()}] ${prefix} t+${Date.now() - t0}ms end${suffix}`);
        },
      };
    },
    show() {
      channel.show(true);
    },
  };
}

module.exports = {
  DEFAULT_FILE_EXTENSIONS,
  bundledExecutablePath,
  createChannelLogger,
  createNullLogger,
  createYamarkExtension,
  documentSelector,
  isQuartoVdocPath,
  normalizeFileExtensions,
  readSettings,
};
