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

  // Register commands & subscriptions
  registerCommands(context);
  registerSubscriptions(client);

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

  // Export document command which require user entered file path
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.invoke.export-doc", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      const filePath = await vscode.window.showInputBox({
        prompt: "File path to export to",
        placeHolder: "e.g. report.json",
      });

      vscode.commands.executeCommand(
        `stencila.export-doc`,
        editor.document.uri.toString(),
        filePath
      );
    })
  );

  // Commands that are handled entirely by the client
  context.subscriptions.push(
    vscode.commands.registerCommand(`stencila.view-node`, (uri, nodeId) => {
      const panel = vscode.window.createWebviewPanel(
        "view-node",
        uri,
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      panel.iconPath; // TODO: Set this to a the icon for the nodeType
      panel.webview.html = `<!DOCTYPE html>
        <html lang="en">
          <head>
              <meta charset="UTF-8">
              <meta name="viewport" content="width=device-width, initial-scale=1.0">
              <title>View Node</title>
          </head>
          <body style="background: white;">
            TODO: Load node "${nodeId}" as WebComponent from WebSocket server
          </body>
        </html>`;
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.inspect-node`,
      async (uri, nodeId) => {
        // TODO: open webview with authors and provenance shown
      }
    )
  );

  /// Command to insert text into a Stencila Markdown file during walkthroughs
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthroughType",
      async (source) => {
        const uri = vscode.Uri.file(
          context.asAbsolutePath("walkthroughs/empty.smd")
        );

        // Get the document editor
        let editor;
        try {
          const document = await vscode.workspace.openTextDocument(uri);
          editor = await vscode.window.showTextDocument(document, {
            viewColumn: vscode.ViewColumn.Beside,
            preview: false,
            preserveFocus: true,
          });
        } catch (error: any) {
          vscode.window.showErrorMessage(
            `Failed to open document: ${error.message}`
          );
        }

        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        // Determine the position at the end of the document
        const document = editor.document;
        const lastLine = document.lineCount - 1;
        const lastLineLength = document.lineAt(lastLine).text.length;
        const position = new vscode.Position(lastLine, lastLineLength);

        // Insert the source at the end of the document
        // This could be made to simulate human typing like other extensions
        // such as https://github.com/marcosgomesneto/typing-simulator do.
        // However, that (a) complicates this, (b) it would increase the load
        // on the Stencila Language Server as it updates things on each character
        // insertion.
        editor.edit((editBuilder) => {
          editBuilder.insert(position, source);
        });
      }
    )
  );
}

type ExecutionStatus =
  | "Pending"
  | "Running"
  | "Succeeded"
  | "Warnings"
  | "Errors"
  | "Exception";

interface Status {
  range: vscode.Range;
  status: ExecutionStatus;
  details: string;
}

const pendingDecoration = vscode.window.createTextEditorDecorationType({
  // TODO: set colors to greyish
  // Grey indicator in overview ruler
  // Use the right lane because it is for diagnostics
  overviewRulerColor: "#28a31f",
  overviewRulerLane: vscode.OverviewRulerLane.Right,

  // Styling for the inline text
  dark: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 5%)",
    },
  },
  light: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 95%)",
    },
  },
});

const runningDecoration = vscode.window.createTextEditorDecorationType({
  // TODO: set colors to blueish
  // Blue indicator in overview ruler
  // Use the right lane because it is for diagnostics
  overviewRulerColor: "#28a31f",
  overviewRulerLane: vscode.OverviewRulerLane.Right,

  // Styling for the inline text
  dark: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 5%)",
    },
  },
  light: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 95%)",
    },
  },
});

const succeededDecoration = vscode.window.createTextEditorDecorationType({
  // TODO: check these colors
  // Green indicator in overview ruler
  // Use the right lane because it is for diagnostics
  overviewRulerColor: "#28a31f",
  overviewRulerLane: vscode.OverviewRulerLane.Right,

  // Styling for the inline text
  dark: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 5%)",
    },
  },
  light: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 95%)",
    },
  },
});

/**
 * Register subscriptions to notifications from the language server
 */
function registerSubscriptions(client: LanguageClient) {
  // Handle status notifications
  client.onNotification(
    "stencila/publishStatus",
    ({ uri, statuses }: { uri: string; statuses: Status[] }) => {
      const editor = vscode.window.visibleTextEditors.find(
        (editor) => editor.document.uri.toString() === uri.toString()
      );
      if (!editor) {
        return;
      }

      const statusToRange = ({ range, status, details }: Status) => {
        const startLine = range.start.line;
        const lineLength = editor.document.lineAt(startLine).text.length;
        const position = new vscode.Position(startLine, lineLength);
        const decorationRange = new vscode.Range(position, position);

        return {
          range: decorationRange,
          renderOptions: {
            after: {
              contentText: " " + (details ?? status),
            },
          },
        };
      };

      const decorationsFor = (
        status: ExecutionStatus,
        decoration: vscode.TextEditorDecorationType
      ) => {
        editor.setDecorations(
          decoration,
          statuses.filter((s) => s.status === status).map(statusToRange)
        );
      };

      decorationsFor("Pending", pendingDecoration);
      decorationsFor("Running", runningDecoration);
      decorationsFor("Succeeded", succeededDecoration);
    }
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
