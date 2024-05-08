import path from "path";

import * as vscode from "vscode";

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
export function createDocumentViewPanel(
  extensionUri: vscode.Uri,
  documentUri: vscode.Uri,
  nodeId?: string,
  expand?: boolean
): vscode.WebviewPanel {
  if (documentViewPanels.has(documentUri)) {
    // If there is already a panel open for this document, reveal it
    let panel = documentViewPanels.get(documentUri);
    panel.reveal();
    return panel;
  }

  const filename = path.basename(documentUri.fsPath);

  // TODO: For deployment we will need to pull the web dist into the extension
  // folder rather than reaching out and getting it!
  const webDist = vscode.Uri.joinPath(extensionUri, "..", "web", "dist");

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
    extensionUri,
    "icons",
    "images",
    "stencila-icon-32x32.svg"
  );

  const themeName = "default";
  const themeCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "themes", `${themeName}.css`)
  );
  const viewCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "dynamic.css")
  );
  const viewJs = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "dynamic.js")
  );

  panel.webview.html = `<!DOCTYPE html>
<html lang="en">
  <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>Stencila: Document Preview</title>
      <link rel="preconnect" href="https://fonts.googleapis.com">
      <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
      <link href="https://fonts.googleapis.com/css2?family=Inter:slnt,wght@-10..0,100..900&family=IBM+Plex+Mono:wght@400&display=swap" rel="stylesheet">
      <link title="theme:${themeName}" rel="stylesheet" type="text/css" href="${themeCss}">
      <link rel="stylesheet" type="text/css" href="${viewCss}">
      <script type="text/javascript" src="${viewJs}"></script>
  </head>
  <body style="background: white;">
    <stencila-dynamic-view view="dynamic">
      <article>
        <stencila-paragraph>
          <div slot="content">
            A test paragraph
          </div>
        </stencila-paragraph>
      </article>
    </stencila-dynamic-view>
  </body>
</html>`;

  // Track the webview by adding it to the map
  documentViewPanels.set(documentUri, panel);

  // Handle when the webview is disposed
  panel.onDidDispose(() => {
    documentViewPanels.delete(documentUri);
  }, null);

  return panel;
}
