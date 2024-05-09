import * as vscode from "vscode";
import { createDocumentViewPanel } from "./webviews";
import { LanguageClient } from "vscode-languageclient/node";

/**
 * Register commands provided by the extension
 */
export function registerCommands(context: vscode.ExtensionContext, client: LanguageClient) {
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
    vscode.commands.registerCommand(
      "stencila.view-doc",
      // docUri and nodeType are not used but are in the arguments
      // that we pass to all commands form code lenses so need to be here
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        await createDocumentViewPanel(
          context,
          client,
          editor.document.uri,
          nodeId
        );
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
