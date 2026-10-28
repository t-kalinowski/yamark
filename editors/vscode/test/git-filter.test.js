const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const {
  createYamarkExtension,
} = require("../src/core");
const { fakeVscode } = require("./support/fake-vscode");

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
      cwd: path.resolve(repositoryRoot),
    },
    {
      command: "/usr/bin/git",
      args: ["cat-file", "--filters", ":docs/post.md"],
      input: "",
      cwd: path.resolve(repositoryRoot),
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

