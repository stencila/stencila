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
}
