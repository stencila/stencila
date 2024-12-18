import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import { registerAuthenticationProvider } from "./authentication";
import { registerDocumentCommands } from "./commands";
import { registerKernelsView } from "./kernels";
import {
  registerStatusNotifications,
  registerNodeInfoNotifications,
} from "./notifications";
import { registerModelsView } from "./models";
import { registerPromptsView } from "./prompts";
import { collectSecrets, registerSecretsCommands } from "./secrets";
import { registerStatusBar } from "./status-bar";
import {
  closeDocumentViewPanels,
  documentPatchHandlers,
  registerModelChatView,
  registerSubscriptionNotifications,
} from "./webviews";
import { cliPath } from "./cli";
import { registerWalkthroughCommands } from "./walkthroughs";
import { registerStencilaShell } from "./shell";
import { registerSetupView } from "./setup";
import { registerChatEditor } from "./editors";

let client: LanguageClient | undefined;

let outputChannel = vscode.window.createOutputChannel(
  "Stencila Language Server"
);

/**
 * A view that requests data from the LSP server and
 * which needs to be refreshed when the LSP is restarted
 * and a new client instance created
 */
interface ClientView {
  refresh(client: LanguageClient): Promise<void> | void;
}

let views: ClientView[] = [];

/**
 * Activate the extension
 */
export async function activate(context: vscode.ExtensionContext) {
  // Register auth provider, commands etc
  // Some of these (e.g. auth provider) are used when collecting secrets in `startServer`
  // so this needs to be done first
  registerAuthenticationProvider(context);
  registerSecretsCommands(context);
  registerDocumentCommands(context);
  registerWalkthroughCommands(context);
  registerStatusBar(context);
  registerStencilaShell(context);
  registerChatEditor(context);
  registerOtherCommands(context);
  registerModelChatView(context);

  await startServer(context);
}

/**
 * Start the Stencila LSP server
 */
async function startServer(context: vscode.ExtensionContext) {
  // Get config options
  const initializationOptions = vscode.workspace.getConfiguration("stencila");

  // Get the path to the CLI
  let command = cliPath(context);

  // Determine the arguments to the CLI
  let args: string[];
  const logLevel = initializationOptions.languageServer?.logLevel;
  switch (context.extensionMode) {
    case vscode.ExtensionMode.Development:
    case vscode.ExtensionMode.Test: {
      args = ["lsp", `--log-level=${logLevel ?? "debug"}`];
      break;
    }
    case vscode.ExtensionMode.Production: {
      args = ["lsp", `--log-level=${logLevel ?? "warn"}`];
      break;
    }
  }

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
    outputChannel,
    documentSelector: [
      { language: "smd" },
      { language: "myst" },
      { language: "qmd" },
    ],
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

  // Register handlers for notifications from the client
  registerStatusNotifications(context, client);
  registerSubscriptionNotifications(context, client);
  registerNodeInfoNotifications(context, client);

  // Create views using client, or refresh existing views with new client (if a restart)
  if (views.length) {
    for (const view of views) {
      view.refresh(client);
    }
  } else {
    views = [
      registerSetupView(context, client),
      registerKernelsView(context, client),
      registerPromptsView(context, client),
      registerModelsView(context, client),
    ];
  }
}

/**
 * Register other commands
 */
function registerOtherCommands(context: vscode.ExtensionContext) {
  // Command to open stencila settings
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.settings", () => {
      vscode.commands.executeCommand("workbench.action.openSettings", {
        focus: true,
        query: "@ext:stencila.stencila",
      });
    })
  );

  // Command to restart the server (e.g. after setting or removing secrets)
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.lsp-server.restart", async () => {
      if (client) {
        try {
          await client.stop();
        } catch (error) {
          vscode.window.showWarningMessage(
            `Error stopping the Stencila Language Server: ${error}. Proceeding with restart.`
          );
        } finally {
          client = undefined;
        }
      }

      // Close all doc view panels which will otherwise be left unresponsive
      closeDocumentViewPanels();

      // Wait a bit before starting the new server
      await new Promise((resolve) => setTimeout(resolve, 1000));

      await startServer(context);

      vscode.window.showInformationMessage(
        "Stencila Language Server has been restarted."
      );
    })
  );

  // Command to view the server logs
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.lsp-server.logs", async () => {
      outputChannel.show();
    })
  );
}

/**
 * Deactivate the extension
 */
export function deactivate() {
  if (client) {
    client.stop();
  }
}

// The following functions relate to topics in other modules (e.g documents)
// but are included here because they all send requests to the the
// current client (which is managed in this module)

/**
 * Subscribe to DOM HTML of a document
 */
export async function subscribeToDom(
  documentUri: vscode.Uri,
  callback: (patch: unknown) => void
): Promise<[string, string, string]> {
  if (!client) {
    throw new Error("No Stencila LSP client");
  }

  // Subscribe to document
  const [subscriptionId, theme, html] = (await client.sendRequest(
    "stencila/subscribeDom",
    {
      uri: documentUri.toString(),
    }
  )) as string;

  // Register the handler for patches for this subscription
  documentPatchHandlers[subscriptionId] = callback;

  return [subscriptionId, theme, html];
}

/**
 * Send a request to reset the DOM HTML for a subscription
 */
export async function resetDom(subscriptionId: string) {
  if (!client) {
    throw new Error("No Stencila LSP client");
  }

  await client.sendRequest("stencila/resetDom", {
    subscriptionId,
  });
}

/**
 * Unsubscribe from updates to the DOM HTML of a document
 */
export async function unsubscribeFromDom(subscriptionId: string) {
  if (!client) {
    throw new Error("No Stencila LSP client");
  }

  // Dispose of patch handler for the subscription
  delete documentPatchHandlers[subscriptionId];

  // Unsubscribe from document so that its server task can be stopped
  await client.sendRequest("stencila/unsubscribeDom", {
    subscriptionId,
  });
}

/**
 * Get the node ids corresponding to line umbers of a document
 */
export async function nodeIdsForLines(
  uri: vscode.Uri,
  lines: number[]
): Promise<string[]> {
  if (!client) {
    throw new Error("No Stencila LSP client");
  }

  return await client.sendRequest("stencila/nodeIdsForLines", {
    uri: uri.toString(),
    lines,
  });
}

/**
 * Get the line number of node ids in a document
 */
export async function linesForNodeIds(
  uri: vscode.Uri,
  ids: string[]
): Promise<number[]> {
  if (!client) {
    throw new Error("No Stencila LSP client");
  }

  return await client.sendRequest("stencila/linesForNodeIds", {
    uri: uri.toString(),
    ids,
  });
}
