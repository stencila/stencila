import * as vscode from "vscode";

export function registerStatusBar(context: vscode.ExtensionContext) {
  const statusBar = new ExtensionStatusBar();

  // Initial update for the current active editor
  statusBar.updateStatusBarItem(vscode.window.activeTextEditor?.document);

  // Update status bar based on active editor changes
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      statusBar.updateStatusBarItem(editor?.document);
    })
  );

  // Create a "menu" style command-picker command
  const menu = vscode.commands.registerCommand(
    "stencila.command-picker",
    async () => {
      const commands: CommandPickerItem[] = [
        {
          label: "Documents",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(new-file) New Stencila Markdown",
          description: "Create a new Stencila Markdown file",
          command: "stencila.new-smd",
        },
        {
          label: "$(new-file) New MyST",
          description: "Create a new MyST Markdown file",
          command: "stencila.new-myst",
        },
        {
          label: "$(run-all) Run",
          description: "Run the current document",
          command: "stencila.invoke.run-doc",
        },
        {
          label: "$(preview) Preview",
          description: "Preview the current document",
          command: "stencila.view-doc",
        },
        {
          label: "Settings & Services",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(gear) Settings",
          description: "Update Stencila settings",
          command: "stencila.settings",
        },
        {
          label: "$(sign-in) Sign In",
          description:
            "Sign in to Stencila Cloud to use model router and other services",
          command: "stencila.cloud.signin",
        },
        {
          label: "$(key) Sign In with Access Token",
          description:
            "Sign in to Stencila Cloud using an access token",
          command: "stencila.cloud.signintoken",
        },
        {
          label: "$(sign-out) Sign Out",
          description: "Sign out from Stencila Cloud",
          command: "stencila.cloud.signout",
        },
        {
          label: "$(server-process) Restart",
          description: "Restart the Stencila Language Server",
          command: "stencila.lsp-server.restart",
        },
        {
          label: "$(output) Logs",
          description: "View the logging output of the Stencila Language Server",
          command: "stencila.lsp-server.logs",
        },
        {
          label: "Secrets",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(workspace-trusted) Set Secret",
          description:
            "Add a secret, such as an API key, to Stencila's key ring",
          command: "stencila.secrets.set",
        },
        {
          label: "$(workspace-untrusted) Delete Secret",
          description: "Remove a secret from Stencila's key ring",
          command: "stencila.secrets.delete",
        },
        {
          label: "Markdown Walkthroughs",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(symbol-operator) Math",
          description: "Creating math equations using TeX, AsciiMath and LLMs",
          command: "workbench.action.openWalkthrough",
          args: ["stencila.stencila#math-smd", false],
        },
        {
          label: "$(circuit-board) Diagrams",
          description: "Creating diagrams with Mermaid and LLMs",
          command: "workbench.action.openWalkthrough",
          args: ["stencila.stencila#mermaid-smd", false],
        },
      ];

      const item = await vscode.window.showQuickPick(commands, {
        placeHolder: "Select a Stencila command to run",
      });

      if (item?.command) {
        if (item.args) {
          vscode.commands.executeCommand(item.command, ...item.args);
        } else {
          vscode.commands.executeCommand(item.command);
        }
      }
    }
  );

  context.subscriptions.push(statusBar, menu);
}

interface CommandPickerItem extends vscode.QuickPickItem {
  command?: string;
  args?: (string | boolean)[];
}

class ExtensionStatusBar {
  private statusBarItem: vscode.StatusBarItem;

  constructor() {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      0
    );
    this.statusBarItem.text = "$(circle-large-outline) Stencila";
    this.statusBarItem.tooltip = "Stencila is active";
    this.statusBarItem.command = "stencila.command-picker";
  }

  public updateStatusBarItem(document: vscode.TextDocument | undefined): void {
    if (!document) {
      this.statusBarItem.text = "$(circle-large-outline) Stencila";
      this.statusBarItem.tooltip = "Stencila is active";
      return;
    }

    if (this.isSupportedDocument(document)) {
      this.statusBarItem.text = "$(circle-large-filled) Stencila";
      this.statusBarItem.tooltip = "Stencila is active on this document";
    } else {
      this.statusBarItem.text = "$(circle-large-outline) Stencila";
      this.statusBarItem.tooltip =
        "Stencila is active but not on this document";
    }

    this.statusBarItem.show();
  }

  private isSupportedDocument(document: vscode.TextDocument): boolean {
    const lang = document.languageId;
    const path = document.uri.fsPath;
    return (
      ["smd", "myst"].includes(lang) ||
      path.endsWith(".smd") ||
      path.endsWith(".myst")
    );
  }

  public dispose(): void {
    this.statusBarItem.dispose();
  }
}
