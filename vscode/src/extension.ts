import * as path from "path";

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import { registerAuthenticationProvider } from "./authentication";
import { registerCommands } from "./commands";
import { registerNotifications } from "./notifications";

let client: LanguageClient;

/**
 * Activate the extension
 */
export async function activate(context: vscode.ExtensionContext) {
  // Determine which binary to run based on mode
  let command: string;
  let args: string[];
  switch (context.extensionMode) {
    case vscode.ExtensionMode.Development:
    case vscode.ExtensionMode.Test: {
      command = path.join(__dirname, "..", "..", "target", "debug", "stencila");
      args = ["lsp", "--log-level=debug", "--log-format=pretty"];
      break;
    }
    case vscode.ExtensionMode.Production: {
      command = "stencila";
      args = ["lsp", "--log-level=debug", "--log-format=compact"];
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

  // Register auth provider, commands, notifications etc
  registerAuthenticationProvider(context);
  registerCommands(context, client);
  registerNotifications(client);
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
