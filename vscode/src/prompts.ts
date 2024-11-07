import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

export function registerPromptsView(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const treeDataProvider = new PromptTreeProvider(client);

  const treeView = vscode.window.createTreeView("stencila-prompts", {
    treeDataProvider,
  });

  const refresh = vscode.commands.registerCommand(
    "stencila.prompts.refresh",
    () => treeDataProvider.refresh()
  );

  const use = vscode.commands.registerCommand(
    "stencila.prompts.use",
    ({ prompt }: { prompt: Prompt }) => {
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;

        const selected = !selection.isEmpty
          ? editor.document.getText(selection)
          : undefined;

        const format = editor.document.languageId;
        let snippet;
        if (format === "myst") {
          snippet = mystSnippet(prompt, selected);
        } else {
          snippet = smdSnippet(prompt, selected);
        }
        editor.insertSnippet(new vscode.SnippetString(snippet), selection);
      } else {
        vscode.window.showWarningMessage("No active text editor found");
      }
    }
  );

  const picker = vscode.commands.registerCommand(
    "stencila.prompts.picker",
    async () => {
      const items = await treeDataProvider.getPickerItems();

      const item = await vscode.window.showQuickPick(items, {
        placeHolder: "Search for a prompt",
      });

      if (item) {
        vscode.commands.executeCommand("stencila.prompts.use", {
          prompt: item.prompt,
        });
      }
    }
  );

  context.subscriptions.push(treeView, refresh, use, picker);

  return treeDataProvider;
}

type InstructionType = "Create" | "Edit" | "Fix" | "Describe";

interface Prompt {
  id: string;
  name: string;
  version: string;
  description: string;
  instructionTypes: InstructionType[];
}

/**
 * Get the shorthand id for a prompt (if possible)
 */
function promptId(prompt: Prompt): string {
  const parts = prompt.id.split("/");
  return parts[0] === "stencila" ? parts[parts.length - 1] : prompt.id;
}

/**
 * Get the icon for a prompt
 */
function promptIcon(prompt: Prompt): string {
  const label = promptId(prompt);
  switch (prompt?.instructionTypes[0]) {
    case "Create": {
      switch (label) {
        case "list-ordered":
        case "list-unordered":
          return label;
        case "list-checked":
          return "checklist";
        case "figure-code":
          return "graph-line";
        case "paragraph":
          return "whitespace";
        default:
          if (label.endsWith("caption")) {
            return "list-selection";
          } else if (label.startsWith("code")) {
            return "code";
          } else if (label.startsWith("figure")) {
            return "symbol-misc";
          } else if (label.startsWith("math")) {
            return "symbol-operator";
          } else if (label.startsWith("quote")) {
            return "quote";
          } else if (label.startsWith("table")) {
            return "symbol-number";
          } else {
            return "sparkle";
          }
      }
    }
    case "Edit":
      return "pencil";
    case "Fix":
      return "wrench";
    case "Describe":
      return "comment";
    default:
      return "file";
  }
}

/**
 * Create a Stencila Markdown snippet for a command using a prompt
 */
function smdSnippet(prompt: Prompt, selected?: string): string {
  const type = prompt.instructionTypes[0].toLowerCase();
  const id = promptId(prompt);

  let snippet = `::: ${type}`;

  if (id !== "block") {
    snippet += ` @${id}`;
  }

  snippet += " ${0}";

  if (selected) {
    snippet += "\n";
    if (!selected.startsWith("\n")) {
      snippet += "\n";
    }
    snippet += selected;
    if (!selected.endsWith("\n")) {
      snippet += "\n";
    }
    snippet += "\n:::\n";
  } else if (type === "create" || type === "describe") {
    snippet += " :::\n";
  } else {
    snippet += " >>>\n";
  }

  return snippet;
}

/**
 * Create a MyST snippet for a command using a prompt
 */
function mystSnippet(prompt: Prompt, selected?: string): string {
  const type = prompt.instructionTypes[0].toLowerCase();
  const id = promptId(prompt);

  let snippet = `:::{${type}} \${0}\n`;

  if (id !== "block") {
    snippet += `:prompt: ${id}\n`;
  }

  if (selected) {
    if (!selected.startsWith("\n")) {
      snippet += "\n";
    }
    snippet += selected;
    if (!selected.endsWith("\n")) {
      snippet += "\n";
    }
  } else {
    snippet += "\n";
  }

  snippet += "\n:::\n";

  return snippet;
}

class PromptPickerItem implements vscode.QuickPickItem {
  label: string;
  description: string;

  constructor(public prompt: Prompt) {
    // Use full id as label because filtering is done on label only
    this.label = `$(${promptIcon(prompt)}) ${prompt.id}`;
    this.description = prompt.description;
  }
}

class PromptTreeItem extends vscode.TreeItem {
  constructor(
    public readonly library: string | null,
    public readonly prompt?: Prompt
  ) {
    let label = "";
    if (library) {
      label = library;
    } else if (prompt) {
      label = promptId(prompt);
    }

    super(
      label,
      library
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None
    );

    this.id = prompt?.id;
    this.description = prompt?.name;
    this.tooltip = prompt && `${prompt.id}: ${prompt.description}`;

    const icon = library ? "folder" : promptIcon(prompt!);
    this.iconPath = new vscode.ThemeIcon(icon);

    // Set the context value to allow filtering commands by the item type
    this.contextValue = library ? "library" : "prompt";
  }
}

class PromptTreeProvider implements vscode.TreeDataProvider<PromptTreeItem> {
  /**
   * The LSP client used to fetch the list of prompts
   */
  client: LanguageClient;

  /**
   * The list of prompts obtained from the LSP
   */
  list: Prompt[];

  private _onDidChangeTreeData: vscode.EventEmitter<
    PromptTreeItem | undefined | null | void
  > = new vscode.EventEmitter<PromptTreeItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<
    PromptTreeItem | undefined | null | void
  > = this._onDidChangeTreeData.event;

  constructor(client: LanguageClient) {
    this.client = client;
    this.list = [];
  }

  async refresh(client?: LanguageClient): Promise<void> {
    if (client) {
      this.client = client;
    }

    this.list = await this.client.sendRequest("stencila/listPrompts");

    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item: PromptTreeItem): vscode.TreeItem {
    return item;
  }

  async getChildren(item?: PromptTreeItem): Promise<PromptTreeItem[]> {
    if (this.list.length === 0) {
      await this.refresh();
    }

    if (!item) {
      return [new PromptTreeItem("Builtin")];
    }

    return this.list.map((prompt) => new PromptTreeItem(null, prompt));
  }

  async getPickerItems(): Promise<PromptPickerItem[]> {
    if (this.list.length === 0) {
      await this.refresh();
    }

    return this.list.map((prompt) => new PromptPickerItem(prompt));
  }
}
