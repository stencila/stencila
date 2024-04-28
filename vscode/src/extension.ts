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
 *
 * At the time of writing, all of these commands delegate to the
 * language server after getting arguments from the editor.
 */
function registerCommands(context: vscode.ExtensionContext) {
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
          editor.document.uri.path,
          editor.selection.active
        );
      })
    );
  }
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
