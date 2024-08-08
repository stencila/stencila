import * as path from "path";

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import { registerNotifications } from "./notifications";
import { registerCommands } from "./commands";
import { patchWebViewJs } from "./webviews";

let client: LanguageClient;

/**
 * Activate the extension
 */
export async function activate(context: vscode.ExtensionContext) {
  // Make necessary patch
  patchWebViewJs(context.extensionUri);

  // Determine which binary to run based on mode
  let command: string;
  let args: string[];
  switch (context.extensionMode) {
    case vscode.ExtensionMode.Development:
    case vscode.ExtensionMode.Test: {
      command = path.join(__dirname, "..", "..", "target", "debug", "stencila");
      args = ["lsp"];
      break;
    }
    case vscode.ExtensionMode.Production: {
      command = "stencila";
      args = ["lsp"];
      break;
    }
  }

  // Get config options
  const initializationOptions = vscode.workspace.getConfiguration("stencila");

  // Start the language server client
  const serverOptions: ServerOptions = {
    command,
    args,
  };
  const clientOptions: LanguageClientOptions = {
    initializationOptions,
    documentSelector: [{ language: "smd" }, { language: "myst" }],
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

  // Register commands, notifications
  registerCommands(context, client);
  registerNotifications(client);

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
