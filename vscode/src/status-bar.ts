import * as vscode from "vscode";

import { PROVIDER_ID } from "./authentication";

/**
 * The status bar for the extension
 */
class ExtensionStatusBar {
  private statusBarItem: vscode.StatusBarItem;

  constructor() {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      0
    );
    this.statusBarItem.text = "$(circle-large-outline) Stencila";
    this.statusBarItem.tooltip = "Click for Stencila menu";
    this.statusBarItem.command = "stencila.command-picker";
    this.statusBarItem.show();
  }

  public updateForDocumentEditor(
    document: vscode.TextDocument | undefined
  ): void {
    if (!document) {
      this.setInactive();
    } else if (this.isSupportedDocument(document)) {
      this.setActive();
    } else {
      this.setInactive();
    }
  }

  public updateForDocumentView(active: boolean): void {
    if (active) {
      this.setActive();
    } else {
      this.setInactive();
    }
  }

  private setActive() {
    this.statusBarItem.text = "$(circle-large-filled) Stencila";
    this.statusBarItem.show();
  }

  private setInactive() {
    this.statusBarItem.text = "$(circle-large-outline) Stencila";
    this.statusBarItem.show();
  }

  private isSupportedDocument(document: vscode.TextDocument): boolean {
    const lang = document.languageId;
    const path = document.uri.fsPath;
    return (
      ["smd", "myst", "qmd"].includes(lang) ||
      path.endsWith(".smd") ||
      path.endsWith(".myst") ||
      path.endsWith(".qmd")
    );
  }

  public dispose(): void {
    this.statusBarItem.dispose();
  }
}

export const statusBar = new ExtensionStatusBar();

interface CommandPickerItem extends vscode.QuickPickItem {
  command?: string;
  args?: (string | boolean)[];
}

export function registerStatusBar(context: vscode.ExtensionContext) {
  // Initial update for the current active editor
  statusBar.updateForDocumentEditor(vscode.window.activeTextEditor?.document);

  // Update status bar based on active editor changes
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      statusBar.updateForDocumentEditor(editor?.document);
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
          label: "New Stencila Markdown",
          description: "Create a new Stencila Markdown file",
          command: "stencila.new-smd",
          iconPath: vscode.Uri.joinPath(
            context.extensionUri,
            "icons/stencila-128.png"
          ),
        },
        {
          label: "New MyST Markdown",
          description: "Create a new MyST Markdown file",
          command: "stencila.new-myst",
          iconPath: vscode.Uri.joinPath(
            context.extensionUri,
            "icons/myst-128.png"
          ),
        },
        {
          label: "New Quarto Markdown",
          description: "Create a new Quarto Markdown file",
          command: "stencila.new-qmd",
          iconPath: vscode.Uri.joinPath(
            context.extensionUri,
            "icons/quarto-128.png"
          ),
        },
      ];

      // Add commands related to the current document
      const langId = vscode.window.activeTextEditor?.document.languageId;
      if (langId && ["md", "smd", "myst"].includes(langId)) {
        commands.push(
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
            label: "$(save-all) Save",
            description: "Save the current document with a sidecar file",
            command: "stencila.invoke.save-doc",
          }
        );
      }

      commands.push(
        {
          label: "Settings & Services",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(gear) Settings",
          description: "Update Stencila settings",
          command: "stencila.settings",
        }
      );

      // Add sign in/out commands based on whether there is a session of not
      const session = await vscode.authentication.getSession(PROVIDER_ID, [], {
        createIfNone: false,
      });
      commands.push(
        ...(!session
          ? [
              {
                label: "$(sign-in) Sign In",
                description:
                  "Sign in to Stencila Cloud to use model router and other services",
                command: "stencila.cloud.signin",
              },
              {
                label: "$(key) Sign In with Access Token",
                description: "Sign in to Stencila Cloud using an access token",
                command: "stencila.cloud.signin-token",
              },
            ]
          : [
              {
                label: "$(sign-out) Sign Out",
                description: "Sign out from Stencila Cloud",
                command: "stencila.cloud.signout",
              },
            ])
      );

      commands.push(
        {
          label: "$(server-process) Restart",
          description: "Restart the Stencila Language Server",
          command: "stencila.lsp-server.restart",
        },
        {
          label: "$(output) Logs",
          description:
            "View the logging output of the Stencila Language Server",
          command: "stencila.lsp-server.logs",
        },
        {
          label: "$(terminal) Shell",
          description: "Open a shell with the Stencila CLI available",
          command: "stencila.shell",
        },
        {
          label: "Secrets",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(workspace-trusted) Set Secret",
          description:
            "Add a secret, such as an API key, to Stencila's VSCode secrets",
          command: "stencila.secrets.set",
        },
        {
          label: "$(workspace-untrusted) Delete Secret",
          description: "Remove a secret from Stencila's VSCode secrets",
          command: "stencila.secrets.delete",
        },
        {
          label: "Walkthroughs",
          kind: vscode.QuickPickItemKind.Separator,
        },
        {
          label: "$(hubot) AI Command Syntax",
          description: "How to write AI commands",
          command: "stencila.walkthroughs.open",
          args: ["command-syntax", "smd"],
        },
        {
          label: "$(symbol-operator) Math Equations and Symbols",
          description: "Creating math equations using TeX, AsciiMath and LLMs",
          command: "stencila.walkthroughs.open",
          args: ["math", "smd", "myst", "qmd"],
        },
        {
          label: "$(symbol-misc) Mermaid Diagrams",
          description:
            "Creating flowcharts, timelines and other diagrams using Mermaid and LLMs",
          command: "stencila.walkthroughs.open",
          args: ["mermaid", "smd", "myst", "qmd"],
        }
      );

      const item = await vscode.window.showQuickPick(commands, {
        title: "Stencila Commands",
        placeHolder: "Select a Stencila command to run",
        matchOnDescription: true,
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
