import * as vscode from "vscode";

import { createModelChatPanel, createDocumentViewPanel } from "./webviews";

/**
 * Register document related commands provided by the extension
 */
export function registerDocumentCommands(context: vscode.ExtensionContext) {
  // Create document commands
  for (const format of ["smd", "myst", "qmd"]) {
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
    "prev-node",
    "next-node",
    "archive-node",
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

  // Retry the active suggestion without feedback
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.invoke.retry-node",
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          `stencila.revise-node`,
          editor.document.uri.toString(),
          editor.selection.active
        );
      }
    )
  );

  // Revise the active suggestion of an instruction with feedback
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.invoke.revise-node",
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        const feedback = await vscode.window.showInputBox({
          title: "Revise suggestion",
          placeHolder:
            "Describe what you want changed, or leave blank to just retry.",
        });

        vscode.commands.executeCommand(
          `stencila.revise-node`,
          editor.document.uri.toString(),
          // If invoked from code lens then `nodeType` and `nodeId` should be defined
          // and should be passed as arguments. Otherwise, if invoked using keybinding
          // then those arguments will not be present so pass the selection.
          ...(nodeId ? [nodeType, nodeId] : [editor.selection.active]),
          feedback
        );
      }
    )
  );

  // Save document command which requires that document is not untitled
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.invoke.save-doc", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      const document = editor.document;

      if (document.isUntitled) {
        vscode.window.showWarningMessage(
          "Please save the document's source file first."
        );
        return;
      }

      vscode.commands.executeCommand(
        `stencila.save-doc`,
        document.uri.toString()
      );
    })
  );

  // Export document command which requires user entered file path
  // so must be invoked from here
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.invoke.export-doc", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      const formats = {
        docx: "Microsoft Word",
        odt: "Open Document Text",
        pdf: "PDF",
        _1: null,
        latex: "LaTeX",
        myst: "MyST Markdown",
        qmd: "Quarto Markdown",
        smd: "Stencila Markdown",
        _2: null,
        json: "Stencila Schema JSON",
        jsonld: "Schema.org JSON Linked Data",
        yaml: "Stencila Schema YAML",
      };
      const items = Object.entries(formats).map(([format, desc]) =>
        desc
          ? {
              label: format,
              description: desc,
            }
          : {
              label: "",
              kind: vscode.QuickPickItemKind.Separator,
            }
      );

      const format = await vscode.window.showQuickPick(items, {
        title: "Export Format",
        placeHolder: "Select a format to export the document to",
        matchOnDescription: true,
      });

      const filename = editor.document.fileName;
      const saveUri = await vscode.window.showSaveDialog({
        title: "Export Document",
        saveLabel: "Export",
        defaultUri: vscode.Uri.file(
          `${filename.substring(0, filename.lastIndexOf("."))}.${format?.label}`
        ),
      });

      if (!saveUri) {
        vscode.window.showInformationMessage("Document export cancelled.");
        return;
      }

      vscode.commands.executeCommand(
        `stencila.export-doc`,
        editor.document.uri.toString(),
        saveUri.fsPath
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

        await createDocumentViewPanel(context, editor);
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

        await createDocumentViewPanel(context, editor, nodeId);
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

        await createDocumentViewPanel(context, editor, nodeId, true);
      }
    )
  );
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.open-model-chat",
      async (docUri) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }
        await createModelChatPanel(context, editor);
      }
    )
  )
}
