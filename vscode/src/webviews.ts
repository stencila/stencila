import path from "path";

import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

import { resetDom, subscribeToDom, unsubscribeFromDom } from "./extension";
import { ScrollSyncer } from "./scroll-syncer";
import { statusBar } from "./status-bar";

/**
 * A map of document view panels used to ensure that only one
 * view of a document exists at a time
 *
 * Previously used `vscode.Uri` as keys but that ignored
 * the 'fragment' part used for node ids and thus caused
 * multiple panels to be opened for the same node. Therefore
 * this now used the stringified URL.
 */
const documentViewPanels = new Map<string, vscode.WebviewPanel>();

/**
 * A map of the "disposables" for each document that can be disposed of when
 * the view is closed
 */
const documentViewHandlers = new Map<string, vscode.Disposable[]>();

export interface DomPatch {
  // Indicated that the node, or document, has been deleted
  // and that the panel should be closed
  deleted: boolean;
}

/**
 * A map of patch handler function for each subscription to a
 * document's DOM HTML
 */
export const documentPatchHandlers: Record<string, (patch: DomPatch) => void> =
  {};

/**
 * Register a handler for "stencila/publishDom" notifications that forwards
 * the patch onto the handler to the appropriate webview
 */
export function registerSubscriptionNotifications(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const handler = client.onNotification(
    "stencila/publishDom",
    ({
      subscriptionId,
      patch,
    }: {
      subscriptionId: string;
      patch: DomPatch;
    }) => {
      const handler = documentPatchHandlers[subscriptionId];
      if (!handler) {
        console.error(`No handler for subscription ${subscriptionId}`);
      } else {
        handler(patch);
      }
    }
  );
  context.subscriptions.push(handler);
}

type ReceivedMessage = DomResetMessage | CommandMessage | ScrollSyncMessage;

interface DomResetMessage {
  type: "dom-reset";
}

interface CommandMessage {
  type: "command";
  command: string;
  args?: string[];
}

interface ScrollSyncMessage {
  type: "scroll-sync";
  startId?: string;
  endId?: string;
  cursorId?: string;
}

/**
 * Create a WebView panel that displays a document
 *
 * @param nodeId The id of the node that the document should scroll to
 * @param expandAuthors Whether the node card should be in expanded to show authorship/provenance
 */
export async function createDocumentViewPanel(
  context: vscode.ExtensionContext,
  documentUri: vscode.Uri,
  editor?: vscode.TextEditor,
  nodeId?: string,
  expandAuthors?: boolean,
  viewColumn: vscode.ViewColumn = vscode.ViewColumn.Beside,
  titlePrefix = "Preview"
): Promise<vscode.WebviewPanel> {
  const uriString = documentUri.toString();

  if (documentViewPanels.has(uriString)) {
    // If there is already a panel open for this document, reveal it
    const panel = documentViewPanels.get(uriString) as vscode.WebviewPanel;
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

  const title = `${titlePrefix} ${path.basename(documentUri.fsPath)}`;

  // Create the panel
  const panel = vscode.window.createWebviewPanel(
    "document-view",
    title,
    viewColumn,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [
        vscode.Uri.joinPath(context.extensionUri, "out", "web"),
      ],
    }
  );

  initializeWebViewPanel(context, documentUri, panel, editor);

  // If `nodeId` param is defined, scroll webview panel to target node.
  if (nodeId) {
    panel.webview.postMessage({
      type: "view-node",
      nodeId,
      expandAuthors,
    });
  }

  return panel;
}

/**
 * Create a WebView panel that displays a single node in a document
 */
export async function createNodeViewPanel(
  context: vscode.ExtensionContext,
  documentUri: vscode.Uri,
  position: vscode.Position | null,
  nodeType: string,
  nodeId: string,
  expandAuthors: boolean = false,
  viewColumn: vscode.ViewColumn = vscode.ViewColumn.Beside
): Promise<vscode.WebviewPanel> {
  const uri = documentUri.with({ fragment: nodeId });

  const uriKey = uri.toString();

  if (documentViewPanels.has(uriKey)) {
    const panel = documentViewPanels.get(uriKey) as vscode.WebviewPanel;
    panel.reveal();

    return panel;
  }

  const title = position
    ? `${nodeType} at ${path.basename(uri.fsPath)}:${position.line + 1}`
    : `${nodeType} in ${path.basename(uri.fsPath)}`;

  const panel = vscode.window.createWebviewPanel(
    "node-view",
    title,
    viewColumn,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [
        vscode.Uri.joinPath(context.extensionUri, "out", "web"),
      ],
    }
  );

  initializeWebViewPanel(context, uri, panel);

  if (expandAuthors) {
    panel.webview.postMessage({
      type: "view-node",
      nodeId,
      expandAuthors,
    });
  }

  return panel;
}

/**
 * Initialize a WebView panel that displays a document
 */
export async function initializeWebViewPanel(
  context: vscode.ExtensionContext,
  uri: vscode.Uri,
  panel: vscode.WebviewPanel,
  editor?: vscode.TextEditor
) {
  const uriString = uri.toString();

  // Set the icon of the panel
  // TODO: use a different icon for articles, prompts and chats
  panel.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "icons",
    "stencila-128.png"
  );

  // Subscribe to updates of DOM HTML for document
  const [subscriptionId, themeName, viewHtml] = await subscribeToDom(
    uri,
    (patch) => {
      // Dispose of the panel if the document or node has been deleted
      if (patch.deleted) {
        return panel.dispose();
      }

      // Otherwise, forward the patch to the panel
      panel.webview.postMessage({
        type: "dom-patch",
        patch,
      });
    }
  );

  // Folder containing bundled JS and other assets for the web view
  const webDist = vscode.Uri.joinPath(context.extensionUri, "out", "web");
  const themeCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "themes", `${themeName}.css`)
  );
  const viewCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "vscode.css")
  );
  const viewJs = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "vscode.js")
  );

  panel.webview.html = `
    <!DOCTYPE html>
      <html lang="en">
        <head>
            <title>Stencila: Document Preview</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet">
            <link title="theme:${themeName}" rel="stylesheet" type="text/css" href="${themeCss}">
            <link rel="stylesheet" type="text/css" href="${viewCss}">
            <script type="text/javascript" src="${viewJs}"></script>
        </head>
        <body style="background: white;">
          <stencila-vscode-view theme="${themeName}">
            ${viewHtml}
          </stencila-vscode-view>
          <script>
            const vscode = acquireVsCodeApi()
          </script>
        </body>
    </html>
  `;

  // Send system data to the webview
  await panel.webview.postMessage({
    type: "system-data",
    kernels: await vscode.commands.executeCommand("stencila.kernels.list"),
    prompts: await vscode.commands.executeCommand("stencila.prompts.list"),
    models: await vscode.commands.executeCommand("stencila.models.list"),
  });

  const disposables: vscode.Disposable[] = [];

  if (editor) {
    // Create a scroller sync for the view
    const scrollSync = new ScrollSyncer(editor, panel);
    disposables.push(scrollSync);
  }

  // Listen to the view state changes of the webview panel to update status bar
  panel.onDidChangeViewState(
    (e) => {
      statusBar.updateForDocumentView(e.webviewPanel.active);
    },
    null,
    disposables
  );

  // Handle when the webview is disposed
  panel.onDidDispose(
    () => {
      // Unsubscribe from updates to DOM HTML
      unsubscribeFromDom(subscriptionId);

      // Remove from list of panels
      documentViewPanels.delete(uriString);

      // Dispose handlers and remove from lists
      documentViewHandlers
        .get(uriString)
        ?.forEach((handler) => handler.dispose());
      documentViewHandlers.delete(uriString);
    },
    null,
    disposables
  );

  // Handle messages from the webview
  // It is necessary to translate the names of the Stencila document
  // command to the command and argument convention that the LSP uses
  const documentUri = uri.with({ fragment: "" }).toString();
  panel.webview.onDidReceiveMessage(
    (message: ReceivedMessage) => {
      if (message.type === "dom-reset") {
        resetDom(subscriptionId);
        return;
      }

      if (message.type !== "command") {
        // Skip messages handled elsewhere
        return;
      }

      vscode.commands.executeCommand(
        `stencila.${message.command}`,
        documentUri,
        ...(message.args ?? [])
      );
    },
    null,
    disposables
  );

  // Track the webview by adding it to the map
  documentViewPanels.set(uriString, panel);
  documentViewHandlers.set(uriString, disposables);

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
