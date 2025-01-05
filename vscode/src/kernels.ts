import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

export function registerKernelsView(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const treeDataProvider = new KernelTreeProvider(context, client);

  const treeView = vscode.window.createTreeView("stencila-kernels", {
    treeDataProvider,
  });

  const list = vscode.commands.registerCommand(
    "stencila.kernels.list",
    async () => {
      if (treeDataProvider.list.length === 0) {
        await treeDataProvider.refresh();
      }
      return treeDataProvider.list;
    }
  );

  const refresh = vscode.commands.registerCommand(
    "stencila.kernels.refresh",
    () => treeDataProvider.refresh()
  );

  const use = vscode.commands.registerCommand(
    "stencila.kernels.use",
    (item: KernelTreeItem) => {
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;

        const format = editor.document.languageId;
        let snippet;
        if (format === "myst") {
          snippet = mystSnippet(item.kernel!);
        }
        if (format === "qmd") {
          snippet = qmdSnippet(item.kernel!);
        } else {
          snippet = smdSnippet(item.kernel!);
        }
        editor.insertSnippet(new vscode.SnippetString(snippet), selection);
      } else {
        vscode.window.showWarningMessage("No active text editor found");
      }
    }
  );

  context.subscriptions.push(treeView, list, refresh, use);

  return treeDataProvider;
}

type KernelType =
  | "Programming"
  | "Diagrams"
  | "Templating"
  | "Math"
  | "Styling";

type KernelAvailability =
  | "Available"
  | "Installable"
  | "Unavailable"
  | "Disabled";

interface Kernel {
  name: string;
  provider: string;
  type: KernelType;
  availability: KernelAvailability;
}

/**
 * Create a Stencila Markdown snippet to use the kernel
 */
function smdSnippet(kernel: Kernel): string {
  if (kernel.name === "tex") {
    return "$$\n${0}\n$$\n";
  }

  if (kernel.name === "style") {
    return "::: style ${1}\n\n${2}\n\n:::\n";
  }

  switch (kernel.type) {
    case "Math":
      return "```" + kernel.name + "\n${0}\n```\n";
    default:
      return "```" + kernel.name + " exec\n${0}\n```\n";
  }
}

/**
 * Create a MyST snippet to use the kernel
 */
function mystSnippet(kernel: Kernel): string {
  if (kernel.name === "tex") {
    return "$$\n${0}\n$$\n";
  }

  if (kernel.name === "mermaid") {
    return "```{mermaid}\n${0}\n```\n";
  }

  return "```{code-cell} " + kernel.name + "\n${0}\n```\n";
}

/**
 * Create a QMD snippet to use the kernel
 */
function qmdSnippet(kernel: Kernel): string {
  if (kernel.name === "tex") {
    return "$$\n${0}\n$$\n";
  }

  return "```{" + kernel.name + "}\n${0}\n```\n";
}

class KernelTreeItem extends vscode.TreeItem {
  constructor(
    context: vscode.ExtensionContext,
    public readonly type: KernelType | null,
    public readonly kernel?: Kernel
  ) {
    let label = "";
    if (type) {
      label = type;
    } else if (kernel) {
      label = kernel.name;
    }

    super(
      label,
      type
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None
    );

    const icon = (() => {
      if (type) {
        switch (type) {
          case "Programming":
            return "zap";
          case "Math":
            return "symbol-operator";
          case "Diagrams":
            return "symbol-misc";
          case "Templating":
            return "bracket";
          case "Styling":
            return "symbol-color";
          default:
            return "circle-large-outline";
        }
      } else {
        return "circle-large-outline";
      }
    })();

    this.iconPath = icon.includes(".")
      ? vscode.Uri.joinPath(context.extensionUri, "icons", icon)
      : new vscode.ThemeIcon(icon);

    // Set the context value to allow filtering commands by the item type
    this.contextValue = type
      ? "type"
      : kernel?.availability === "Available"
        ? "kernel"
        : "unavailable";
  }
}

class KernelTreeProvider implements vscode.TreeDataProvider<KernelTreeItem> {
  /**
   * The VSCode extension context used for resolving icon paths
   */
  context: vscode.ExtensionContext;

  /**
   * The LSP client used to fetch the list of kernels
   */
  client: LanguageClient;

  /**
   * The list of kernels obtained from the LSP
   */
  list: Kernel[];

  /**
   * The unique kernel types from the list of kernels
   */
  types: KernelType[];

  private _onDidChangeTreeData: vscode.EventEmitter<
    KernelTreeItem | undefined | null | void
  > = new vscode.EventEmitter<KernelTreeItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<
    KernelTreeItem | undefined | null | void
  > = this._onDidChangeTreeData.event;

  constructor(context: vscode.ExtensionContext, client: LanguageClient) {
    this.context = context;
    this.client = client;
    this.list = [];
    this.types = [];
  }

  async refresh(client?: LanguageClient): Promise<void> {
    if (client) {
      this.client = client;
    }

    this.list = (
      (await this.client.sendRequest("stencila/listKernels")) as Kernel[]
    ).filter((kernel) => kernel.availability === "Available");
    this.types = [...new Set(this.list.map((kernel) => kernel.type))];

    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item: KernelTreeItem): vscode.TreeItem {
    return item;
  }

  async getChildren(item?: KernelTreeItem): Promise<KernelTreeItem[]> {
    if (this.list.length === 0) {
      await this.refresh();
    }

    if (!item) {
      return this.types.map((type) => new KernelTreeItem(this.context, type));
    }

    if (item.type) {
      return this.list
        .filter((kernel) => kernel.type === item.type)
        .map((kernel) => new KernelTreeItem(this.context, null, kernel));
    }

    return [];
  }
}
