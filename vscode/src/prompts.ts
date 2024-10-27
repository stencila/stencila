import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

export function registerPromptsView(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const treeView = vscode.window.createTreeView("stencila-prompts", {
    treeDataProvider: new PromptTreeProvider(client),
  });
  context.subscriptions.push(treeView);
}

type InstructionType = "Create" | "Edit" | "Fix" | "Describe";

interface Prompt {
  id: string;
  name: string;
  version: string;
  description: string;
  instructionTypes: InstructionType[];
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
      const parts = prompt.id.split("/");
      label = parts[parts.length - 1];
    }

    super(
      label,
      library
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None
    );

    this.id = prompt?.id;
    this.description = prompt?.name;
    this.tooltip = prompt && `${prompt.id}: ${prompt.name}`;

    const icon = (() => {
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
          return library ? "folder" : "file";
      }
    })();

    this.iconPath = new vscode.ThemeIcon(icon);
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

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item: PromptTreeItem): vscode.TreeItem {
    return item;
  }

  async getChildren(item?: PromptTreeItem): Promise<PromptTreeItem[]> {
    if (this.list.length === 0) {
      this.list = await this.client.sendRequest("stencila/listPrompts");
    }

    if (!item) {
      return [new PromptTreeItem("Builtin")];
    }

    return this.list.map((prompt) => new PromptTreeItem(null, prompt));
  }
}
