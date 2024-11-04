import { writeFileSync } from "fs";

import * as vscode from "vscode";

import { createDocumentViewPanel } from "./webviews";

/**
 * Register document related commands provided by the extension
 */
export function registerDocumentCommands(context: vscode.ExtensionContext) {
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

  // Run the current walkthrough step
  //
  // Gets the content and the range of the step, clears existing content from the range
  // (from the line after the start of the range to the end of the range), and then
  // simulates typing (at normal typing speeds) the new content into that range
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.invoke.walkthrough-step",
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        let [content, range] = (await vscode.commands.executeCommand(
          "stencila.walkthrough-step",
          docUri,
          nodeType,
          nodeId
        )) as [string, vscode.Range];

        // Clear any existing content from the line after the start of the step,
        // to the end of the document. There should not normally be any content
        // between the start of the step and the end of the document, but if there
        // is this removes to to ensure a clean state.
        await editor.edit((editBuilder) => {
          editBuilder.delete(
            new vscode.Range(
              new vscode.Position(range.start.line + 1, 0),
              new vscode.Position(1000, 0)
            )
          );
        });

        // Function to type character by character
        const typeContent = async (text: string) => {
          const lines = text.split("\n");
          for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            for (let j = 0; j < line.length; j++) {
              await editor.edit((editBuilder) => {
                const position = new vscode.Position(
                  range.start.line + 1 + i,
                  j
                );
                editBuilder.insert(position, line[j]);
              });
              // Add random delay between characters
              await new Promise((resolve) =>
                setTimeout(resolve, 10 + Math.floor(Math.random() * 50))
              );
            }

            // If not the last line, add newline
            if (i < lines.length - 1) {
              await editor.edit((editBuilder) => {
                const position = new vscode.Position(
                  range.start.line + 1 + i,
                  line.length
                );
                editBuilder.insert(position, "\n");
              });
            }
          }

          // Add an ellipsis after the step so that we get a code lens
          // for the next step (if any)
          await editor.edit((editBuilder) => {
            const position = new vscode.Position(
              range.start.line + 1 + lines.length,
              0
            );
            editBuilder.insert(position, "\n...\n");
          });
        };

        // Type the content, includes a starting blank line to separate
        // from the step's ellipsis
        try {
          await typeContent("\n" + content);
        } catch (error) {
          vscode.window.showErrorMessage(`Error running step: ${error}`);
        }
      }
    )
  );

  // Expand the current walkthrough so it can be edited
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthroughs.expand",
      async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          "stencila.patch-curr",
          editor.document.uri.toString(),
          "Walkthrough",
          editor.selection.active,
          "isExpanded",
          true
        );
      }
    )
  );

  // Collapse the current walkthrough and reset each of the steps to inactive state
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.walkthroughs.collapse",
      async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          "stencila.patch-curr",
          editor.document.uri.toString(),
          "Walkthrough",
          editor.selection.active,
          "isExpanded",
          false
        );
      }
    )
  );
}
