/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";

import { event } from "./events";
import { createNodeViewPanel, createDocumentViewPanel } from "./webviews";

class Format {
  constructor(
    public description: string,
    public format: string,
    public render = false,
    public reproducible = false
  ) {}
}

const FORMATS = {
  "repro-docx": new Format("Reproducible Microsoft Word", "docx", true, true),
  _1: null,
  docx: new Format("Microsoft Word", "docx"),
  odt: new Format("Open Document Text", "odt"),
  pdf: new Format("Adobe Portable Document Format", "pdf"),
  _2: null,
  tex: new Format("LaTeX", "latex"),
  myst: new Format("MyST Markdown", "myst"),
  qmd: new Format("Quarto Markdown", "qmd"),
  smd: new Format("Stencila Markdown", "smd"),
  _3: null,
  json: new Format("Stencila Schema JSON", "json"),
  jsonld: new Format("Schema.org JSON Linked Data", "jsonld"),
  yaml: new Format("Stencila Schema YAML", "yaml"),
};

function formatQuickPickItems() {
  return Object.entries(FORMATS).map(([label, format]) =>
    format
      ? {
          label,
          description: format.description,
        }
      : {
          label: "",
          kind: vscode.QuickPickItemKind.Separator,
        }
  );
}

function getFormat(label: keyof typeof FORMATS) {
  return FORMATS[label];
}

/**
 * Register document related commands provided by the extension
 */
export function registerDocumentCommands(context: vscode.ExtensionContext) {
  // Keep track of the most recently active text editor as a fallback in
  // commands below
  let lastTextEditor: vscode.TextEditor | null = null;
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (
        editor?.document.languageId &&
        ["smd", "myst", "qmd", "latex"].includes(editor?.document.languageId)
      ) {
        lastTextEditor = editor;
      }
    })
  );

  // Create document commands
  for (const format of ["smd", "myst", "qmd", "latex"]) {
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

  // Run the current node
  vscode.commands.registerCommand(`stencila.invoke.run-curr`, async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage("No active editor");
      return;
    }

    const result = await vscode.commands.executeCommand(
      `stencila.run-curr`,
      editor.document.uri.toString(),
      editor.selection.active
    );

    let nodeType;
    let nodeId;
    if (
      Array.isArray(result) &&
      typeof result[0] === "string" &&
      typeof result[1] === "string"
    ) {
      nodeType = result[0];
      nodeId = result[1];
    } else {
      return;
    }

    if (nodeType === "Chat") {
      await createNodeViewPanel(
        context,
        editor.document.uri,
        null,
        nodeType,
        nodeId
      );
    }
  });

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
      "stencila.invoke.insert-clones",
      async (docUri, [nodeIds]) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        vscode.commands.executeCommand(
          `stencila.insert-clones`,
          // For consistency, first args are destination document and position
          editor.document.uri.toString(),
          editor.selection.active,
          // Source document and nodes
          docUri,
          nodeIds
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

  // Export document command which requires user entered file path
  // so must be invoked from here
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.invoke.export-doc", async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      const item = await vscode.window.showQuickPick(formatQuickPickItems(), {
        title: "Export Format",
        placeHolder: "Select a format to export the document to",
        matchOnDescription: true,
      });

      if (!item) {
        vscode.window.showInformationMessage("Document conversion cancelled.");
        return;
      }

      const options = getFormat(item.label as keyof typeof FORMATS)!;

      const filename = editor.document.fileName;
      const saveUri = await vscode.window.showSaveDialog({
        title: "Export Document",
        saveLabel: "Export",
        defaultUri: vscode.Uri.file(
          `${filename.substring(0, filename.lastIndexOf("."))}.${options.format}`
        ),
      });

      if (!saveUri) {
        vscode.window.showInformationMessage("Document export cancelled.");
        return;
      }

      event("doc_export", options);

      vscode.commands.executeCommand(
        `stencila.export-doc`,
        editor.document.uri.toString(),
        saveUri.fsPath,
        options.format,
        options.render,
        options.reproducible
      );
    })
  );

  // Merge document command which requires user entered file path
  // so must be invoked from here
  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.invoke.merge-doc", async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      // Save the document if it has unsaved changes
      if (editor.document.isDirty) {
        const saved = await editor.document.save();
        if (!saved) {
          vscode.window.showInformationMessage(
            "Document merge cancelled: document must be saved first."
          );
          return;
        }
      }

      const original = editor.document.uri.fsPath;

      const fileUri = await vscode.window.showOpenDialog({
        canSelectFiles: true,
        canSelectFolders: false,
        canSelectMany: false,
        openLabel: "Merge",
        title: "Select file to merge into document",
      });

      if (!fileUri || fileUri.length === 0) {
        return;
      }

      const edited = fileUri[0].fsPath;

      event("doc_merge");

      const filesModified: string[] = await vscode.commands.executeCommand(
        `stencila.merge-doc`,
        // Note that this order is correct as per the Rust function `codecs::merge`
        edited,
        original
      );

      // Handle the results
      if (filesModified === null) {
        vscode.window.showInformationMessage("Merge cancelled.");
        return;
      }
      if (filesModified.length === 0) {
        vscode.window.showInformationMessage(
          "File merged successfully but no changes were detected."
        );
        return;
      }

      // Track files that couldn't be opened
      const failedFiles: string[] = [];

      // Open each modified file to show git diff in the editor
      for (const filePath of filesModified) {
        try {
          const fileUri = vscode.Uri.file(filePath);

          // Check if file exists
          try {
            await vscode.workspace.fs.stat(fileUri);
          } catch {
            failedFiles.push(filePath);
            continue;
          }

          // Open the file - VSCode will automatically show git decorations
          // and the user can use the Source Control view or gutter indicators
          // to see the changes
          await vscode.window.showTextDocument(fileUri, {
            preview: false,
            preserveFocus: false,
          });
        } catch (error) {
          failedFiles.push(filePath);
        }
      }

      // Show success message with count
      const successCount = filesModified.length - failedFiles.length;
      if (successCount > 0) {
        vscode.window.showInformationMessage(
          `Merge completed. ${successCount} file${successCount === 1 ? "" : "s"} modified.`
        );
      }

      // Show warning for files that couldn't be opened
      if (failedFiles.length > 0) {
        vscode.window.showWarningMessage(
          `${failedFiles.length} file${failedFiles.length === 1 ? " was" : "s were"} modified but could not be opened for diff view: ${failedFiles.join(", ")}`
        );
      }
    })
  );

  // Document preview panel
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "stencila.view-doc",
      async (docUri, nodeType) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor;
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
        const editor = vscode.window.activeTextEditor ?? lastTextEditor;
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
        const editor = vscode.window.activeTextEditor ?? lastTextEditor;
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
      const editor = vscode.window.activeTextEditor ?? lastTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      event("doc_chat", { format: editor.document.languageId });

      const chatId = await vscode.commands.executeCommand<string>(
        "stencila.create-chat",
        editor.document.uri.toString(),
        null, // range
        "Discuss", // instruction type
        null, // node type
        "document"
      );

      await createNodeViewPanel(
        context,
        editor.document.uri,
        null,
        "Chat",
        chatId
      );
    })
  );

  // Create a temporary chat in the current document
  //
  // If the instruction type is not supplied it is inferred from the selected node
  // types (if any).
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.invoke.create-chat`,
      async ({ instructionType, nodeType, prompt, executeChat } = {}) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showErrorMessage("No active editor");
          return;
        }

        event("doc_chat_create", {
          format: editor.document.languageId,
          instructionType,
          nodeType,
          prompt,
          executeChat,
        });

        const chatId = await vscode.commands.executeCommand<string>(
          "stencila.create-chat",
          editor.document.uri.toString(),
          editor.selection,
          instructionType,
          nodeType,
          prompt,
          executeChat
        );

        await createNodeViewPanel(
          context,
          editor.document.uri,
          editor.selection.active,
          "Chat",
          chatId
        );
      }
    )
  );

  // Typed wrapper to the `invoke.create-chat` command for convenience
  // of following commands
  async function insertChat(options: {
    instructionType: "Create" | "Edit" | "Fix";
    nodeType?: string;
    prompt?: string;
    executeChat?: boolean;
  }) {
    await vscode.commands.executeCommand(
      "stencila.invoke.create-chat",
      options
    );
  }

  // Create a `Create` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat.suggest`,
      async () => await insertChat({ instructionType: "Create" })
    )
  );
  for (const prompt of [
    "code-chunk",
    "figure-code",
    "figure-flowchart",
    "figure-mermaid",
    "figure-svg",
    "figure-timeline",
    "list-ordered",
    "list-unordered",
    "math-block",
    "paragraph",
    "quote-block",
    "table-code",
    "table-empty",
    "table-filled",
  ]) {
    context.subscriptions.push(
      vscode.commands.registerCommand(
        `stencila.insert-chat.create.${prompt}`,
        async () =>
          await insertChat({
            instructionType: "Create",
            // Do not need to prefix prompt with `stencila/create` because
            // providing instruction type
            prompt,
          })
      )
    );
  }

  // Create a `Edit` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat.edit`,
      async () => await insertChat({ instructionType: "Edit" })
    )
  );

  // Create a `Fix` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat.fix`,
      async () => await insertChat({ instructionType: "Fix" })
    )
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
