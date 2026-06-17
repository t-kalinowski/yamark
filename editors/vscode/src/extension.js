const vscode = require("vscode");
const { createYamarkExtension, createChannelLogger } = require("./core");

const channel = vscode.window.createOutputChannel("Yamark");
const logger = createChannelLogger(channel);
const extension = createYamarkExtension(vscode, { logger });

function activate(context) {
  context.subscriptions.push(channel);
  extension.activate(context);
}

function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
