import * as path from "path";

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

/**
 * Activate the extension
 */
export function activate(context: vscode.ExtensionContext) {
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
  client.start();

  // Define the default theme for this extension.
  vscode.workspace
    .getConfiguration("workbench")
    .update("colorTheme", "StencilaLight", vscode.ConfigurationTarget.Global);
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
