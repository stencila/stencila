import fs from "fs";
import path from "path";

import * as vscode from "vscode";

export function registerWalkthroughCommands(context: vscode.ExtensionContext) {
  const open = vscode.commands.registerCommand(
    "stencila.walkthroughs.open",
    async (name, ...formats) => {
      // If necessary, ask the user to choose a format for the walkthrough
      let format;
      if (formats.length === 1) {
        format = formats[0];
      } else {
        formats = formats.map((format) => {
          const label = (() => {
            switch (format) {
              case "smd":
                return "Stencila Markdown";
              case "myst":
                return "MyST Markdown";
              default:
                return format.toUpperCase();
            }
          })();

          return {
            label,
            format,
          };
        });
        format = (
          await vscode.window.showQuickPick(formats, {
            placeHolder: "Please select a format for the walkthrough",
            title: "Walkthrough Format",
          })
        ).format;
      }

      // Read the walkthrough content
      const filePath = path.join(
        context.extensionPath,
        "walkthroughs",
        `${name}.${format}`
      );
      const content = fs.readFileSync(filePath, "utf8");

      // Create an untitled document with the content
      const doc = await vscode.workspace.openTextDocument({
        content,
        language: format,
      });

      // TODO: This necessary so that document has a walkthrough node
      // in memory before we send command to collapse it. There should be
      // a better way to do this. e.g. queuing commands, signal that do is ready etc.
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Open the document
      const editor = await vscode.window.showTextDocument(doc);

      // Find the first line that equals '...'
      let walkthroughStart = new vscode.Position(0, 0);
      for (let i = 0; i < doc.lineCount; i++) {
        const lineText = doc.lineAt(i).text.trim();
        if (lineText === "...") {
          walkthroughStart = new vscode.Position(i, 0);
          break;
        }
      }

      // Collapse the walkthrough
      await vscode.commands.executeCommand(
        "stencila.patch-curr",
        editor.document.uri.toString(),
        "Walkthrough",
        walkthroughStart,
        "isCollapsed",
        true
      );
    }
  );

  // Collapse the current walkthrough and reset each of the steps to inactive state
  const collapse = vscode.commands.registerCommand(
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
        "isCollapsed",
        true
      );
    }
  );

  // Expand the current walkthrough so it can be edited
  const expand = vscode.commands.registerCommand(
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
        "isCollapsed",
        false
      );
    }
  );

  context.subscriptions.push(open, collapse, expand);
}
