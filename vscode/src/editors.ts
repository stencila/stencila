import * as vscode from "vscode";
import { initializeWebViewPanel } from "./webviews";

export function registerChatEditor(
  context: vscode.ExtensionContext
): vscode.Disposable {
  return vscode.window.registerCustomEditorProvider(
    "stencila.chat-editor",
    new ChatEditorProvider(context),
    {
      webviewOptions: {
        retainContextWhenHidden: true,
      },
    }
  );
}

/**
 * A custom editor for Stencila `Chat` documents
 *
 * Allows users to use the interactive chat interface without
 * having to have a Stencila Markdown file open.
 */
export class ChatEditorProvider implements vscode.CustomTextEditorProvider {
  constructor(private readonly context: vscode.ExtensionContext) {}

  public resolveCustomTextEditor(
    document: vscode.TextDocument,
    panel: vscode.WebviewPanel
  ): void {
    panel.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.context.extensionUri, "out", "web"),
      ],
    };

    initializeWebViewPanel(this.context, document.uri, panel);
  }
}
