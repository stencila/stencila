import * as vscode from "vscode";
import { linesForNodeIds, nodeIdsForLines } from "./extension";

/**
 * An editor/preview scroll syncer
 *
 * Synchronizes the scroll position between the editor and the
 * preview of a document.
 */
export class ScrollSyncer {
  private editor: vscode.TextEditor | undefined;
  private disposables: vscode.Disposable[] = [];
  private panel: vscode.WebviewPanel;
  private isUpdating = false;
  private ignoreEditorScroll = false;

  constructor(editor: vscode.TextEditor, panel: vscode.WebviewPanel) {
    this.editor = editor;
    this.panel = panel;
    this.registerEventHandlers();
    this.scheduleUpdate();
  }

  private registerEventHandlers() {
    const { previewFollowsEditor, editorFollowsPreview } =
      vscode.workspace.getConfiguration("stencila.linkedScrolling");

    if (previewFollowsEditor) {
      // Only track relevant editors
      this.disposables.push(
        vscode.window.onDidChangeActiveTextEditor((editor) => {
          if (editor === this.editor) {
            this.scheduleUpdate();
          }
        })
      );

      // Track scroll changes
      this.disposables.push(
        vscode.window.onDidChangeTextEditorVisibleRanges((event) => {
          if (event.textEditor === this.editor) {
            this.scheduleUpdate();
          }
        })
      );

      // Track cursor position changes
      this.disposables.push(
        vscode.window.onDidChangeTextEditorSelection((event) => {
          if (event.textEditor === this.editor) {
            this.scheduleUpdate(true);
          }
        })
      );
    }

    if (editorFollowsPreview) {
      // Handle a scroll sync message from the preview's webview
      this.disposables.push(
        this.panel.webview.onDidReceiveMessage(async (message) => {
          if (message.type === "scroll-sync") {
            await this.receiveUpdate(message);
          }
        })
      );
    }
  }

  /**
   * Receive updates from the webview with the ids of the nodes that
   * are at the start and end of the webview's viewport so that the scroll
   * position of the editor can be updated
   */
  private async receiveUpdate({
    startId,
    endId,
  }: {
    startId: string;
    endId: string;
  }) {
    if (
      !this.editor ||
      // Do not update scroll position if this is the active editor
      // or if it is updating
      this.editor === vscode.window.activeTextEditor ||
      this.isUpdating
    ) {
      return;
    }

    try {
      this.isUpdating = true;

      // Request line number from language server
      const lines = await linesForNodeIds(this.editor.document.uri, [
        startId,
        endId,
      ]);
      const startLine = lines[0];
      const endLine = lines[1];

      if (
        startLine &&
        endLine &&
        // Check again in case this became the active editor after call to language server
        this.editor !== vscode.window.activeTextEditor
      ) {
        // Temporarily disable our scroll handler to prevent feedback loop
        this.ignoreEditorScroll = true;

        // Scroll editor to the line
        const range = new vscode.Range(
          new vscode.Position(startLine, 0),
          new vscode.Position(endLine, 0)
        );

        // Reveal the range
        this.editor.revealRange(range, vscode.TextEditorRevealType.Default);

        // Re-enable scroll handler after a short delay
        setTimeout(() => {
          this.ignoreEditorScroll = false;
        }, 100);
      }
    } catch (error) {
      console.error("Failed to sync editor scroll:", error);
    } finally {
      this.isUpdating = false;
    }
  }

  /**
   * Schedule sending an update of the editor scroll position to
   * the webview so its scroll position can be updated
   */
  private scheduleUpdate(selectionChanged: boolean = false) {
    if (this.isUpdating || this.ignoreEditorScroll) {
      return;
    }
    this.isUpdating = true;

    // Debounce updates to avoid overwhelming the LSP
    setTimeout(async () => {
      await this.sendUpdate(selectionChanged);
      this.isUpdating = false;
    }, 100);
  }

  private async sendUpdate(selectionChanged: boolean) {
    if (!this.editor) {
      return;
    }

    const visibleRanges = this.editor.visibleRanges;
    if (visibleRanges.length === 0) {
      return;
    }

    // Get the first and last visible lines
    const startLine = visibleRanges[0].start.line;
    const endLine = visibleRanges[visibleRanges.length - 1].end.line;

    // Get cursor line if there's a selection
    const cursorLine =
      selectionChanged && this.editor.selection
        ? this.editor.selection.active.line
        : undefined;

    try {
      // Send request for ids corresponding to each line to the LSP
      const ids = await nodeIdsForLines(this.editor.document.uri, [
        startLine,
        endLine,
        ...(cursorLine ? [cursorLine] : []),
      ]);

      // Forward the ids to the webview to do scrolling
      this.panel.webview.postMessage({
        type: "scroll-sync",
        startId: ids[0],
        endId: ids[1],
        cursorId: ids[2],
      });
    } catch (error) {
      console.error("Failed to sync viewport:", error);
    }
  }

  public dispose() {
    this.disposables.forEach((d) => d.dispose());
    this.disposables = [];
  }
}
