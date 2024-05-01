import * as path from "path";

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

/**
 * Activate the extension
 */
export async function activate(context: vscode.ExtensionContext) {
  // Start the language server client
  const serverOptions: ServerOptions = {
    command: "cargo",
    args: ["run", "--package=lsp", "--quiet"],
    options: {
      cwd: path.join(__dirname, "..", ".."),
    },
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "smd" }],
    markdown: {
      isTrusted: true,
      supportHtml: true,
    },
  };
  client = new LanguageClient(
    "stencila",
    "Stencila",
    serverOptions,
    clientOptions
  );
  await client.start();

  // Register commands
  registerCommands(context);

  // Define the default theme for this extension.
  vscode.workspace
    .getConfiguration("workbench")
    .update("colorTheme", "StencilaLight", vscode.ConfigurationTarget.Global);
}

/**
 * Register commands provided by the extension
 */
function registerCommands(context: vscode.ExtensionContext) {
  // Commands executed by the server but which are invoked on the client
  // and which use are passed the document URI and selection (position) as arguments
  for (const command of [
    "run-curr",
    "run-all-doc",
    "run-code-doc",
    "run-assist-doc",
    "run-all-below",
    "run-all-above",
    "cancel-curr",
    "cancel-all-doc",
  ]) {
    context.subscriptions.push(
      vscode.commands.registerCommand(`stencila.invoke.${command}`, () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          `stencila.${command}`,
          editor.document.uri.toString(),
          editor.selection.active
        );
      })
    );
  }

  // Commands that are handled entirely by the client
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.inspect-node`,
      (nodeType, nodeId) => {
        const panel = vscode.window.createWebviewPanel(
          "inspect-node",
          nodeType,
          vscode.ViewColumn.Beside,
          { enableScripts: true }
        );
        panel.iconPath; // TODO: Set this to a the icon for the nodeType
        panel.webview.html = `<!DOCTYPE html>
        <html lang="en">
          <head>
              <meta charset="UTF-8">
              <meta name="viewport" content="width=device-width, initial-scale=1.0">
              <title>Inspect Node</title>
          </head>
          <body style="background: white;">
            TODO: Load node "${nodeId}" as WebComponent from WebSocket server
          </body>
        </html>`;
      }
    )
  );
}

/**
 * Deactivate the extension
 */
export function deactivate() {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
