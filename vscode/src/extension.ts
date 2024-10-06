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
import { collectSecrets, registerSecretsCommands } from "./secrets";

let CLIENT: LanguageClient | undefined;

/**
 * Activate the extension
 */
export async function activate(context: vscode.ExtensionContext) {
  // Register auth provider and secrets commands (used when collecting secrets in `startServer`)
  registerAuthenticationProvider(context);
  registerSecretsCommands(context);

  await startServer(context);

  // Register command to restart server
  registerRestartServer(context);

  // Register commands and notifications (which need to have a client passed to them)
  registerCommands(context, CLIENT!);
  registerNotifications(CLIENT!);
}

/**
 * Start the Stencila LSP server
 */
async function startServer(context: vscode.ExtensionContext) {
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

  // Collect secrets to pass as env vars to LSP server
  const secrets = await collectSecrets(context);

  // Start the language server client passing secrets as env vars
  const serverOptions: ServerOptions = {
    command,
    args,
    options: { env: { ...process.env, ...secrets } },
  };
  const clientOptions: LanguageClientOptions = {
    initializationOptions,
    documentSelector: [{ language: "smd" }, { language: "myst" }],
    markdown: {
      isTrusted: true,
      supportHtml: true,
    },
  };
  CLIENT = new LanguageClient(
    "stencila",
    "Stencila",
    serverOptions,
    clientOptions
  );
  await CLIENT.start();
}

/**
 * Register command to restart the server (e.g. after setting or removing secrets)
 */
function registerRestartServer(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.lsp-server.restart", async () => {
      if (CLIENT) {
        try {
          await CLIENT.stop();
        } catch (error) {
          vscode.window.showWarningMessage(
            `Error stopping the server: ${error}. Proceeding with restart.`
          );
        } finally {
          CLIENT = undefined;
        }
      }

      // Wait a bit before starting the new server
      await new Promise((resolve) => setTimeout(resolve, 1000));

      await startServer(context);

      vscode.window.showInformationMessage(
        "Stencila LSP Server has been restarted."
      );
    })
  );
}

/**
 * Deactivate the extension
 */
export function deactivate() {
  if (CLIENT) {
    CLIENT.stop();
  }
}
