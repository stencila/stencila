import { readFileSync, writeFileSync } from "fs";
import path from "path";

import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

/**
 * Replaces the placeholder VSCODE_BASE_URL with the actual WebView URL
 *
 * This is necessary to load Shoelace and Stencila icons from the right place.
 */
export function patchWebViewJs(extensionUri: vscode.Uri) {
  const filePath = path.join(extensionUri.fsPath, "dist", "views", "vscode.js");
  const content = readFileSync(filePath, "utf8").replace(
    "VSCODE_BASE_URL",
    `https://file+.vscode-resource.vscode-cdn.net${extensionUri.fsPath}/dist`
  );
  writeFileSync(filePath, content, "utf8");
}

/**
 * A map of document view panels used to ensure that only one
 * view of a document exists at a time
 */
const documentViewPanels = new Map();

/**
 * Create a WebView panel that display the document
 *
 * @param nodeId The id of the node that the document should scroll to
 * @param expand Whether the node card should be in expanded or not
 */
export async function createDocumentViewPanel(
  context: vscode.ExtensionContext,
  client: LanguageClient,
  documentUri: vscode.Uri,
  nodeId?: string,
  expand?: boolean
): Promise<vscode.WebviewPanel> {
  if (documentViewPanels.has(documentUri)) {
    // If there is already a panel open for this document, reveal it
    let panel = documentViewPanels.get(documentUri);
    panel.reveal();
    return panel;
  }

  const filename = path.basename(documentUri.fsPath);

  // TODO: For deployment we will need to pull the web dist into the extension
  // folder rather than reaching out and getting it!
  const webDist = vscode.Uri.joinPath(context.extensionUri, "dist");

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
    "images",
    "stencila-icon-32x32.svg"
  );

  const createDocumentViewHTML = (content: string) => {
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

    // Note that that attribute view="dynamic" is used to
    // trigger Web Component to render as if they are in dynamic view
    return `<!DOCTYPE html>
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
    <stencila-vscode-view view="dynamic" theme="default">
      ${content}
    </stencila-vscode-view>
  </body>
</html>`;
  };

  const FORMAT = "dom.html";

  const content = (await client.sendRequest("stencila/subscribeContent", {
    uri: documentUri.toString(),
    format: FORMAT,
  })) as string;
  panel.webview.html = createDocumentViewHTML(content);

  client.onNotification(
    "stencila/publishContent",
    ({
      uri,
      format,
      content,
    }: {
      uri: string;
      format: string;
      content: string;
    }) => {
      if (uri === documentUri.toString() && format === FORMAT) {
        panel.webview.html = createDocumentViewHTML(content);
      }
    }
  );

  // Track the webview by adding it to the map
  documentViewPanels.set(documentUri, panel);

  // Handle when the webview is disposed
  panel.onDidDispose(() => {
    documentViewPanels.delete(documentUri);
  }, null);

  return panel;
}
