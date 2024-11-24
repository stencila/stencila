import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

import {
  activeDecoration,
  pendingDecoration,
  runningDecoration,
  skippedDecoration,
  staleDecoration,
  succeededDecoration,
  succeededForkDecoration,
  unexecutedDecoration,
} from "./decorations";
import { gutterDecorationType } from "./gutters";

interface Status {
  range: vscode.Range;
  // The status being reported. Note that this is a combination
  // of both `ExecutionStatus` and `ExecutionRequired` ("Unexecuted" & "Stale")
  status:
    | "Unexecuted"
    | "Stale"
    | "Skipped"
    | "Pending"
    | "Running"
    | "Succeeded"
    | "SucceededFork"
    | "Active";
  message: string;
}

/**
 * Register subscription to status notifications received
 * by the language client from the language server
 */
export function registerStatusNotifications(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  // Handle document status notifications
  const handler = client.onNotification(
    "stencila/publishStatus",
    ({ uri, statuses }: { uri: string; statuses: Status[] }) => {
      const editor = vscode.window.visibleTextEditors.find(
        (editor) => editor.document.uri.toString() === uri.toString()
      );
      if (!editor) {
        return;
      }

      const statusToRange = ({
        range,
        status,
        message,
      }: Status): vscode.DecorationOptions => {
        const startLine = range.start.line;
        const lineLength = editor.document.lineAt(startLine).text.length;
        const position = new vscode.Position(startLine, lineLength);
        const decorationRange = new vscode.Range(position, position);

        return {
          range: decorationRange,
          renderOptions: {
            after: {
              contentText: message ?? status,
              margin: "1em",
            },
          },
        };
      };

      const decorationsFor = (
        status: Status["status"],
        decoration: vscode.TextEditorDecorationType
      ) => {
        editor.setDecorations(
          decoration,
          statuses.filter((s) => s.status === status).map(statusToRange)
        );
      };

      decorationsFor("Unexecuted", unexecutedDecoration);
      decorationsFor("Stale", staleDecoration);
      decorationsFor("Pending", pendingDecoration);
      decorationsFor("Skipped", skippedDecoration);
      decorationsFor("Running", runningDecoration);
      decorationsFor("Succeeded", succeededDecoration);
      decorationsFor("SucceededFork", succeededForkDecoration);
      decorationsFor("Active", activeDecoration);
    }
  );
  context.subscriptions.push(handler);
}

interface NodeInfo {
  range: vscode.Range;
  nodeType: string;
  executionStatus?: string;
}

const gutterDecorationTypes = new Map<
  vscode.TextEditorDecorationType,
  number[]
>();

/**
 * Register subscription to node info notifications received
 * by the language client from the language server
 */
export function registerNodeInfoNotifications(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const { enabled, animateRunning, markerWidth } =
    vscode.workspace.getConfiguration("stencila.gutter");

  if (!enabled) {
    return;
  }

  // Handle document status notifications
  const handler = client.onNotification(
    "stencila/publishNodeInfo",
    ({ uri, nodes }: { uri: string; nodes: NodeInfo[] }) => {
      const editor = vscode.window.visibleTextEditors.find(
        (editor) => editor.document.uri.toString() === uri.toString()
      );
      if (!editor) {
        return;
      }

      // Generate a map of line numbers to the node types
      // that should be represented on it
      const lines: Record<number, string[]> = [];
      for (const node of nodes) {
        const startLine = node.range.start.line;
        const endLine = node.range.end.line - 1;

        for (let line = startLine; line <= endLine; line++) {
          let decoration = node.nodeType;

          if (line === startLine) {
            decoration += "_start";
          }
          if (line === endLine) {
            decoration += "_end";
          }

          if (animateRunning && node.executionStatus === "Running") {
            decoration += "_running";
          }

          if (line in lines) {
            lines[line].push(decoration);
          } else {
            lines[line] = [decoration];
          }
        }
      }

      // Clear the set of lines associated with each gutter decoration
      for (const decorationType of gutterDecorationTypes.keys()) {
        gutterDecorationTypes.set(decorationType, []);
      }

      // Collect the lines for each decoration
      for (const [line, decoration] of Object.entries(lines)) {
        const decorationType = gutterDecorationType(
          context,
          decoration,
          markerWidth
        );
        if (gutterDecorationTypes.has(decorationType)) {
          gutterDecorationTypes.get(decorationType)?.push(parseInt(line));
        } else {
          gutterDecorationTypes.set(decorationType, [parseInt(line)]);
        }
      }

      // Apply each decoration type
      for (const [decorationType, lines] of gutterDecorationTypes.entries()) {
        const options: vscode.DecorationOptions[] = lines.map((line) => {
          const start = new vscode.Position(line, 0);
          const end = new vscode.Position(line, 0);
          const range = new vscode.Range(start, end);

          return {
            range,
          };
        });

        editor.setDecorations(decorationType, options);
      }
    }
  );
  context.subscriptions.push(handler);
}
