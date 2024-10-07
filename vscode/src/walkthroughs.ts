import * as vscode from "vscode";

export function registerWalkthroughCommands(context: vscode.ExtensionContext) {
  // The document being used in the current walkthrough
  let walkthroughDoc: vscode.TextDocument | undefined;

  // Handler for when the walkthrough doc s closed
  let disposable: vscode.Disposable | undefined;

  async function createWalkthroughDoc(format: string) {
    let newDoc = await vscode.workspace.openTextDocument({
      language: format,
    });

    disposable = vscode.workspace.onDidCloseTextDocument((doc) => {
      if (doc === walkthroughDoc) {
        walkthroughDoc = undefined;
        disposable?.dispose();
      }
    });

    return newDoc;
  }

  // Command to open an empty file (usually Stencila Markdown) during walkthroughs
  // Opens an untitled (temporary) file. Previously we created a file on disk which
  // was problematic because in some places the user did not have permission to
  // create the file in the assumed location. It also spammed local folders.
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthrough.open",
      async (format) => {
        try {
          walkthroughDoc = await createWalkthroughDoc(format);
          await vscode.window.showTextDocument(walkthroughDoc, {
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

  // Command to insert text into a file during walkthroughs
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthrough.type",
      async (format, source) => {
        // If there is not yet a walkthrough document, or if it
        // is of the wrong language, create one
        if (!walkthroughDoc || walkthroughDoc.languageId !== format) {
          walkthroughDoc = await createWalkthroughDoc(format);
        }

        // Get the document editor
        let editor;
        try {
          editor = await vscode.window.showTextDocument(walkthroughDoc, {
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
