import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

import {
  pendingDecoration,
  runningDecoration,
  succeededDecoration,
} from "./decorations";

type ExecutionStatus =
  | "Pending"
  | "Running"
  | "Succeeded"
  | "Warnings"
  | "Errors"
  | "Exception";

interface Status {
  range: vscode.Range;
  status: ExecutionStatus;
  details: string;
}

/**
 * Register subscriptions to notifications received by the language client
 * from the language server
 */
export function registerNotifications(client: LanguageClient) {
  // Handle status notifications
  client.onNotification(
    "stencila/publishStatus",
    ({ uri, statuses }: { uri: string; statuses: Status[] }) => {
      const editor = vscode.window.visibleTextEditors.find(
        (editor) => editor.document.uri.toString() === uri.toString()
      );
      if (!editor) {
        return;
      }

      const statusToRange = ({ range, status, details }: Status) => {
        const startLine = range.start.line;
        const lineLength = editor.document.lineAt(startLine).text.length;
        const position = new vscode.Position(startLine, lineLength);
        const decorationRange = new vscode.Range(position, position);

        return {
          range: decorationRange,
          renderOptions: {
            after: {
              contentText: " " + (details ?? status),
            },
          },
        };
      };

      const decorationsFor = (
        status: ExecutionStatus,
        decoration: vscode.TextEditorDecorationType
      ) => {
        editor.setDecorations(
          decoration,
          statuses.filter((s) => s.status === status).map(statusToRange)
        );
      };

      decorationsFor("Pending", pendingDecoration);
      decorationsFor("Running", runningDecoration);
      decorationsFor("Succeeded", succeededDecoration);
    }
  );
}
