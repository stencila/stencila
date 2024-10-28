import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

export function registerModelsView(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const treeDataProvider = new ModelTreeProvider(context, client);

  const treeView = vscode.window.createTreeView("stencila-models", {
    treeDataProvider,
  });

  const refresh = vscode.commands.registerCommand(
    "stencila.models.refresh",
    () => treeDataProvider.refresh()
  );

  const use = vscode.commands.registerCommand(
    "stencila.models.use",
    (item: ModelTreeItem) => {
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;

        const format = editor.document.languageId;
        let snippet;
        if (format === "myst") {
          snippet = `:model: ${item.model?.id}\n`;
        } else {
          snippet = `[${item.model?.id}]`;
        }
        editor.insertSnippet(new vscode.SnippetString(snippet), selection);
      } else {
        vscode.window.showWarningMessage("No active text editor found");
      }
    }
  );

  context.subscriptions.push(treeView, refresh, use);
}

interface Model {
  id: string;
  provider: string;
  name: string;
  version: string;
  type: "Builtin" | "Local" | "Router" | "Proxied" | "Remote" | "Plugin";
  availability:
    | "Available"
    | "Disabled"
    | "RequiresKey"
    | "Installable"
    | "Unavailable";
}

class ModelTreeItem extends vscode.TreeItem {
  constructor(
    context: vscode.ExtensionContext,
    public readonly provider: string | null,
    public readonly model?: Model
  ) {
    let label = "";
    if (provider) {
      label = provider;
    } else if (model) {
      label = `${model.name} ${model.version}`;
    }

    super(
      label,
      provider
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None
    );

    this.id = model?.id;
    this.tooltip = model?.id;

    const icon = (() => {
      if (provider) {
        const name = provider.toLowerCase();
        switch (name) {
          case "anthropic":
          case "google":
          case "mistral":
          case "openai":
          case "stencila":
            return `${name}.svg`;
          default:
            return "circle-large-outline";
        }
      } else {
        switch (model?.availability) {
          case "RequiresKey":
        }

        switch (model?.type) {
          case "Local":
            return "device-desktop";
          case "Remote":
            return "globe";
          case "Proxied":
            return "cloud";
          case "Router":
            return "circuit-board";
          case "Plugin":
            return "plug";
          default:
            return "circle-large-outline";
        }
      }
    })();

    this.iconPath = icon.includes(".")
      ? vscode.Uri.joinPath(context.extensionUri, "icons", icon)
      : new vscode.ThemeIcon(icon);

    // Set the context value to allow filtering commands by the item type
    this.contextValue = provider ? "provider" : "model";
  }
}

class ModelTreeProvider implements vscode.TreeDataProvider<ModelTreeItem> {
  /**
   * The VSCode extension context used for resolving icon paths
   */
  context: vscode.ExtensionContext;

  /**
   * The LSP client used to fetch the list of models
   */
  client: LanguageClient;

  /**
   * The list of models obtained from the LSP
   */
  list: Model[];

  /**
   * The unique model providers from the list of models
   */
  providers: string[];

  private _onDidChangeTreeData: vscode.EventEmitter<
    ModelTreeItem | undefined | null | void
  > = new vscode.EventEmitter<ModelTreeItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<
    ModelTreeItem | undefined | null | void
  > = this._onDidChangeTreeData.event;

  constructor(context: vscode.ExtensionContext, client: LanguageClient) {
    this.context = context;
    this.client = client;
    this.list = [];
    this.providers = [];
  }

  async refresh(): Promise<void> {
    this.list = await this.client.sendRequest("stencila/listModels");
    this.providers = [...new Set(this.list.map((model) => model.provider))];
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item: ModelTreeItem): vscode.TreeItem {
    return item;
  }

  async getChildren(item?: ModelTreeItem): Promise<ModelTreeItem[]> {
    if (this.list.length === 0) {
      await this.refresh();
    }

    if (!item) {
      return this.providers.map(
        (provider) => new ModelTreeItem(this.context, provider)
      );
    }

    if (item.provider) {
      return this.list
        .filter((model) => model.provider === item.provider)
        .map((model) => new ModelTreeItem(this.context, null, model));
    }

    return [];
  }
}
