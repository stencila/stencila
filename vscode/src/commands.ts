/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";

import { event } from "./events";
import { createNodeViewPanel, createDocumentViewPanel } from "./webviews";

/**
 * Register document related commands provided by the extension
 */
export function registerDocumentCommands(context: vscode.ExtensionContext) {
  // Keep track of the most recently active text editor for
  // some of the commands below
  let lastTextEditor: vscode.TextEditor | null = null;
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (
        editor?.document.languageId &&
        ["smd", "myst", "qmd"].includes(editor?.document.languageId)
      ) {
        lastTextEditor = editor;
      }
    })
  );

  // Create document commands
  for (const format of ["smd", "myst", "qmd"]) {
    context.subscriptions.push(
      vscode.commands.registerCommand(`stencila.new-${format}`, async () => {
        event("doc_create", { format });

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

  // Create a new chat document and open with the chat editor
  vscode.commands.registerCommand(`stencila.new-chat`, async () => {
    event("chat_create");

    const doc = await vscode.workspace.openTextDocument({
      language: "smd",
      content: `---
type: Chat
---
`,
    });

    await createDocumentViewPanel(
      context,
      doc.uri,
      undefined,
      undefined,
      false,
      vscode.ViewColumn.Active,
      "Chat"
    );
  });

  // Create a new prompt
  vscode.commands.registerCommand(`stencila.new-prompt`, async () => {
    // TODO: ask user for required fields, e.g instruction types, node types

    event("prompt_create");

    await vscode.workspace.openTextDocument({
      language: "smd",
      content: `---
type: Prompt
name: user/type/name
version: 0.1.0
description: description
instructionTypes: []
nodeTypes: []
---
`,
    });
  });

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

  // Insert a clone of a node
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.invoke.insert-clone",
      async (docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          `stencila.insert-clone`,
          // For consistency, first args are destination document and position
          editor.document.uri.toString(),
          editor.selection.active,
          // Source document and node
          docUri,
          nodeType,
          nodeId
        );
      }
    )
  );

  // Insert a clone of a node with an instruction to edit, fix or update it
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.invoke.insert-instruction",
      async (docUri, nodeType, nodeId, instructionType, executionMode) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          `stencila.insert-instruction`,
          // For consistency, first args are destination document and position
          editor.document.uri.toString(),
          editor.selection.active,
          // Source document and node
          docUri,
          nodeType,
          nodeId,
          // Instruction properties
          instructionType,
          executionMode
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

      event("doc_export", { format: format?.label });

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
      async (docUri, nodeType) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        event("doc_preview", { format: editor.document.languageId });

        await createDocumentViewPanel(context, editor.document.uri, editor);
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

        await createNodeViewPanel(
          context,
          editor.document.uri,
          null,
          nodeType,
          nodeId
        );
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

        await createNodeViewPanel(
          context,
          editor.document.uri,
          null,
          nodeType,
          nodeId,
          true
        );
      }
    )
  );

  // Create a temporary document chat
  //
  // The new chat will be anchored at the end of the document
  context.subscriptions.push(
    vscode.commands.registerCommand(`stencila.chat-doc`, async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      event("doc_chat", { format: editor.document.languageId });

      const chatId = await vscode.commands.executeCommand<string>(
        "stencila.create-chat",
        editor.document.uri.toString()
      );

      const panel = await createNodeViewPanel(
        context,
        editor.document.uri,
        null,
        "Temporary document chat",
        chatId
      );

      panel.onDidDispose(async () => {
        await vscode.commands.executeCommand(
          "stencila.delete-chat",
          editor.document.uri.toString(),
          chatId
        );
      });
    })
  );

  // Create a temporary chat
  //
  // If the instruction type is not supplied it is inferred from the selected node
  // types (if any). Defaults to running the chat straightaway.
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat`,
      async (instructionType, executeChat) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        event("doc_chat_insert", {
          format: editor.document.languageId,
          type: instructionType,
          execute: executeChat,
        });

        const chatId = await vscode.commands.executeCommand<string>(
          "stencila.create-chat",
          editor.document.uri.toString(),
          editor.selection,
          instructionType,
          executeChat ?? true
        );

        const panel = await createNodeViewPanel(
          context,
          editor.document.uri,
          editor.selection.active,
          "Temporary chat",
          chatId
        );

        panel.onDidDispose(async () => {
          await vscode.commands.executeCommand(
            "stencila.delete-chat",
            editor.document.uri.toString(),
            chatId
          );
        });
      }
    )
  );

  // Create a `Create` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(`stencila.insert-chat-create`, async () => {
      await vscode.commands.executeCommand(
        "stencila.insert-chat",
        "Create",
        false
      );
    })
  );

  // Create a `Edit` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(`stencila.insert-chat-edit`, async () => {
      await vscode.commands.executeCommand(
        "stencila.insert-chat",
        "Edit",
        false
      );
    })
  );

  // Create a `Fix` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(`stencila.insert-chat-fix`, async () => {
      await vscode.commands.executeCommand(
        "stencila.insert-chat",
        "Fix",
        false
      );
    })
  );

  // Insert a `create` command
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-command-create`,
      async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        let prompt = null;
        const item: { prompt: { id: string } } =
          await vscode.commands.executeCommand(
            "stencila.prompts.menu",
            "Create"
          );
        if (item) {
          prompt = item.prompt.id;
        }

        let message = await vscode.window.showInputBox({
          title: "Instructions",
          placeHolder:
            "Describe what you want created (end with '...' for more options)",
          ignoreFocusOut: true,
        });

        let models = null;
        if (message?.endsWith("...")) {
          message = message.slice(0, -3);

          const items: { model: { id: string } }[] =
            await vscode.commands.executeCommand("stencila.models.picker");
          if (items && items.length > 0) {
            models = items.map((item) => item.model.id);
          }
        }

        const nodeId = await vscode.commands.executeCommand<string>(
          "stencila.insert-node",
          editor.document.uri.toString(),
          editor.selection.active,
          "InstructionBlock",
          "Create",
          prompt,
          message,
          models
        );

        await vscode.commands.executeCommand(
          "stencila.run-node",
          editor.document.uri.toString(),
          "InstructionBlock",
          nodeId
        );
      }
    )
  );
}
