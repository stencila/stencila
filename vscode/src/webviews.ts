import path from "path";

import * as vscode from "vscode";

import { subscribeToDom, unsubscribeFromDom } from "./extension";
import { statusBar } from "./status-bar";

/**
 * A map of document view panels used to ensure that only one
 * view of a document exists at a time
 */
const documentViewPanels = new Map<vscode.Uri, vscode.WebviewPanel>();

/**
 * Create a WebView panel that display the document
 *
 * @param nodeId The id of the node that the document should scroll to
 * @param expand Whether the node card should be in expanded to show authorship/provenance
 */
export async function createDocumentViewPanel(
  context: vscode.ExtensionContext,
  documentUri: vscode.Uri,
  nodeId?: string,
  expandAuthors?: boolean
): Promise<vscode.WebviewPanel> {
  if (documentViewPanels.has(documentUri)) {
    // If there is already a panel open for this document, reveal it
    const panel = documentViewPanels.get(documentUri) as vscode.WebviewPanel;

    panel.reveal();

    // If `nodeId` param is defined, scroll webview to target node.
    if (nodeId) {
      panel.webview.postMessage({
        type: "view-node",
        nodeId,
        expandAuthors,
      });
    }

    return panel;
  }

  const filename = path.basename(documentUri.fsPath);

  // Folder containing bundled JS and other assets for the web view
  const webDist = vscode.Uri.joinPath(context.extensionUri, "out", "web");

  const panel = vscode.window.createWebviewPanel(
    "document-view",
    `Preview ${filename}`,
    vscode.ViewColumn.Beside,
    {
      enableScripts: true,
      localResourceRoots: [webDist],
    }
  );
  panel.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "icons",
    "stencila-128.png"
  );

  const createDocumentViewHTML = () => {
    const themeName = "default";
    const themeCss = panel.webview.asWebviewUri(
      vscode.Uri.joinPath(webDist, "themes", `${themeName}.css`)
    );
    const viewCss = panel.webview.asWebviewUri(
      vscode.Uri.joinPath(webDist, "views", "vscode.css")
    );
    const viewJs = panel.webview.asWebviewUri(
      vscode.Uri.joinPath(webDist, "views", "vscode.js")
    );

    return `
    <!DOCTYPE html>
      <html lang="en">
        <head>
            <title>Stencila: Document Preview</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=Inter:slnt,wght@-10..0,100..900&family=IBM+Plex+Mono:wght@400&display=swap" rel="stylesheet">
            <link title="theme:${themeName}" rel="stylesheet" type="text/css" href="${themeCss}">
            <link rel="stylesheet" type="text/css" href="${viewCss}">
            <script type="text/javascript" src="${viewJs}"></script>
        </head>
        <body style="background: white;">
          <stencila-vscode-view theme="${themeName}">
          </stencila-vscode-view>
          <script>
            const vscode = acquireVsCodeApi()
          </script>
        </body>
    </html>
  `;
  };
  panel.webview.html = createDocumentViewHTML();

  // Subscribe to updates of DOM HTML for document
  let subscriptionId = await subscribeToDom(documentUri, (patch: unknown) => {
    panel.webview.postMessage({
      type: "dom-patch",
      patch,
    });
  });

  // Track the webview by adding it to the map
  documentViewPanels.set(documentUri, panel);

  // Listen to the view state changes of the webview panel to update status bar
  panel.onDidChangeViewState((e) => {
    statusBar.updateForDocumentView(e.webviewPanel.active);
  });

  // Handle when the webview is disposed
  panel.onDidDispose(() => {
    // Unsubscribe from updates to DOM HTML
    unsubscribeFromDom(subscriptionId);

    // Remove from list of panels
    documentViewPanels.delete(documentUri);
  }, null);

  // If `nodeId` param is defined, scroll webview panel to target node.
  if (nodeId) {
    panel.webview.postMessage({
      type: "view-node",
      nodeId,
      expandAuthors,
    });
  }

  // Handle messages from the webview
  // It is necessary to translate the names of the Stencila document
  // command to the command and argument convention that the LSP uses
  // TODO: import that from the `web` package
  interface DocumentCommand {
    command: string;
    nodeType?: string;
    nodeIds?: string[];
    nodeProperty?: [string, unknown];
    scope?: string;
  }
  panel.webview.onDidReceiveMessage(
    (command: DocumentCommand) => {
      let name = command.command;
      if (name === "execute-nodes") {
        if (command.scope === "plus-before") {
          name = "run-before";
        } else if (command.scope === "plus-after") {
          name = "run-after";
        } else {
          name = "run-node";
        }
      }

      vscode.commands.executeCommand(
        `stencila.${name}`,
        documentUri.toString(),
        command.nodeType,
        ...(command.nodeIds ? command.nodeIds : []),
        ...(command.nodeProperty ? command.nodeProperty : [])
      );
    },
    undefined,
    context.subscriptions
  );

  return panel;
}

/**
 * Close all document view panels
 *
 * This is necessary when the server is restarted because the client that the
 * panels are subscribed to will no longer exist.
 */
export function closeDocumentViewPanels() {
  if (documentViewPanels.size > 0) {
    vscode.window.showInformationMessage("Closing document view panels");
  } else {
    return;
  }

  for (const panel of documentViewPanels.values()) {
    panel.dispose();
  }

  documentViewPanels.clear();
}
