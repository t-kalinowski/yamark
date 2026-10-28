const assert = require("node:assert/strict");
const path = require("node:path");

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
  const didChangeTextDocument = new EventEmitter();
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
      onDidChangeTextDocument: didChangeTextDocument.event,
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
      reloadTextDocumentFromDisk: async (document, text) => {
        document.text = text;
        document.version = (document.version || 0) + 1;
        document.isDirty = false;
        await Promise.all(
          didChangeTextDocument.fire({
            contentChanges: [{ text }],
            document,
          }),
        );
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

module.exports = { fakeDocument, fakeSelection, fakeVscode };
