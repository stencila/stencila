import * as vscode from "vscode";

/**
 * Register commands provided by the extension
 */
export function registerCommands(context: vscode.ExtensionContext) {
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

  // Export document command which requires user entered file path
  // so must be invoked from here
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

  // Document preview panel
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.view-node", (uri) => {
      const panel = vscode.window.createWebviewPanel(
        "view-node",
        uri,
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      panel.iconPath; // TODO: Set this icon
      panel.webview.html = `<!DOCTYPE html>
        <html lang="en">
          <head>
              <meta charset="UTF-8">
              <meta name="viewport" content="width=device-width, initial-scale=1.0">
              <title>Stencila: Document Preview</title>
          </head>
          <body style="background: white;">

          </body>
        </html>`;
    })
  );

  // Document preview panel with a specific node expanded
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.inspect-node",
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
