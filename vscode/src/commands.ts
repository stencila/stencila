import { writeFileSync } from "fs";

import * as vscode from "vscode";

import { createDocumentViewPanel } from "./webviews";

/**
 * Register commands provided by the extension
 */
export function registerCommands(context: vscode.ExtensionContext) {
  // Create document commands
  for (const format of ["smd", "myst"]) {
    context.subscriptions.push(
      vscode.commands.registerCommand(`stencila.new-${format}`, async () => {
        vscode.workspace.openTextDocument({ language: format }).then(
          (document) => {
            vscode.window.showTextDocument(document);
          },
          (err) => {
            vscode.window.showErrorMessage(
              `Failed to create new '${format}' file: ${err.message}`
            );
          }
        );
      })
    );
  }

  // Commands executed by the server but which are invoked on the client
  // and which use are passed the document URI and selection (position) as arguments
  for (const command of [
    "run-curr",
    "run-below",
    "run-above",
    "run-doc",
    "run-code",
    "run-instruct",
    "cancel-curr",
    "cancel-doc",
    "lock-curr",
    "unlock-curr",
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
      // that we pass to all commands from code lenses so need to be here
      async (docUri, nodeType) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        await createDocumentViewPanel(context, editor.document.uri);
      }
    )
  );
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.view-node",
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        await createDocumentViewPanel(context, editor.document.uri, nodeId);
      }
    )
  );
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.view-node-authors",
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        await createDocumentViewPanel(
          context,
          editor.document.uri,
          nodeId,
          true
        );
      }
    )
  );

  /// Command to open an empty file (usually Stencila Markdown) during walkthroughs
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthrough-file-open",
      async (fileName) => {
        // This ensures the file exists and is empty
        // Using an untitled: file scheme instead has the disadvantage that
        // the extension does not get activated until the file is saved.
        const filePath = context.asAbsolutePath(fileName);
        writeFileSync(filePath, "", "utf8");

        const uri = vscode.Uri.parse(filePath);

        try {
          const document = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(document, {
            viewColumn: vscode.ViewColumn.Beside,
            preview: false,
            preserveFocus: true,
          });
        } catch (error: any) {
          vscode.window.showErrorMessage(
            `Failed to open document: ${error.message}`
          );
        }
      }
    )
  );

  /// Command to insert text into a file during walkthroughs
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthrough-file-type",
      async (fileName, source) => {
        const filePath = context.asAbsolutePath(fileName);
        const uri = vscode.Uri.parse(filePath);

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
