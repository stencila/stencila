import * as vscode from "vscode";
import { nodeIdsForLines } from "./extension";

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

  constructor(editor: vscode.TextEditor, panel: vscode.WebviewPanel) {
    this.editor = editor;
    this.panel = panel;
    this.registerEventHandlers();
    this.scheduleUpdate();
  }

  private registerEventHandlers() {
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

  private scheduleUpdate(selectionChanged: boolean = false) {
    if (this.isUpdating) {
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
