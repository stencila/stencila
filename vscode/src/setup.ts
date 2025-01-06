import * as vscode from "vscode";
import { PROVIDER_ID } from "./authentication";
import { collectSecrets } from "./secrets";

/**
 * A view to show a "setup checklist" for the extension
 *
 * Importantly, if no items are checked, then this extension
 * will show the "viewsWelcome" item in the `package.json`.
 */
export function registerSetupView(context: vscode.ExtensionContext) {
  const treeDataProvider = new SetupTreeProvider(context);

  const treeView = vscode.window.createTreeView("stencila-setup", {
    treeDataProvider,
  });

  const refresh = vscode.commands.registerCommand(
    "stencila.setup.refresh",
    () => treeDataProvider.refresh()
  );

  context.subscriptions.push(treeView, refresh);

  return treeDataProvider;
}

class SetupTreeItem extends vscode.TreeItem {
  constructor(
    what: "stencila-cloud" | "ai-api-key" | "user-details",
    checked: boolean
  ) {
    const [label, tip, value] = (() => {
      switch (what) {
        case "stencila-cloud":
          return checked
            ? [
                "Sign in to Stencila Cloud",
                "Signed in to Stencila Cloud and using our smart model router and other services.",
                "stencila-cloud-signed-in",
              ]
            : [
                "Sign in to Stencila Cloud",
                "Sign in to Stencila Cloud to use our smart model router and other services.",
                "stencila-cloud-signed-out",
              ];
        case "ai-api-key":
          return checked
            ? [
                "Add an AI provider API key",
                "One or more API keys added for use with AI commands.",
                "ai-api-key-some",
              ]
            : [
                "Add an AI provider API key",
                "Add an API key from an AI model provider. Only necessary if not signed in to Stencila Cloud.",
                "ai-api-key-none",
              ];
        case "user-details":
          return checked
            ? [
                "Set user details",
                "User details have been set and are used to attribute authorship.",
                "user-details",
              ]
            : [
                "Set user details",
                "Set user details (e.g. name, affiliation) to use when attributing authorship.",
                "user-details",
              ];
      }
    })();
    super(label);

    this.tooltip = tip;
    this.iconPath = new vscode.ThemeIcon(
      checked ? "pass-filled" : "circle-large-outline"
    );
    this.contextValue = value;
  }
}

class SetupTreeProvider implements vscode.TreeDataProvider<SetupTreeItem> {
  context: vscode.ExtensionContext;
  items: SetupTreeItem[] = [];

  private _onDidChangeTreeData: vscode.EventEmitter<
    SetupTreeItem | undefined | null | void
  > = new vscode.EventEmitter<SetupTreeItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<
    SetupTreeItem | undefined | null | void
  > = this._onDidChangeTreeData.event;

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
    this.refresh();
  }

  async refresh(): Promise<void> {
    // Currently does not use client, just checks whether signed in with
    // Stencila Cloud or has any AI API keys set.

    let stencilaCloud = false;
    const session = await vscode.authentication.getSession(PROVIDER_ID, []);
    if (session?.accessToken) {
      stencilaCloud = true;
    }

    let aiApiKey = false;
    const secrets = await collectSecrets(this.context);
    for (const key in secrets) {
      if (key === "STENCILA_API_TOKEN") {
        stencilaCloud = true;
      } else {
        aiApiKey = true;
      }
    }

    let userDetails = false;
    const config = vscode.workspace.getConfiguration("stencila");
    if (config.user?.name || config.user?.config) {
      userDetails = true;
    }

    this.items =
      stencilaCloud || aiApiKey || userDetails
        ? [
            new SetupTreeItem("stencila-cloud", stencilaCloud),
            new SetupTreeItem("ai-api-key", aiApiKey),
            new SetupTreeItem("user-details", userDetails),
          ]
        : [];

    this._onDidChangeTreeData.fire();
  }

  getTreeItem(item: SetupTreeItem): SetupTreeItem {
    return item;
  }

  async getChildren(): Promise<SetupTreeItem[]> {
    return this.items;
  }
}
