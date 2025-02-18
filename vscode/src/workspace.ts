import * as path from "path";
import * as fs from "fs";

import * as vscode from "vscode";

/**
 * Check for, and run, any workspace setup script
 *
 * Checks for a `/home/workspace/stencila/defaults/setup.sh` script.
 * If it exists, then opens up a new terminal and runs that script
 * in the terminal so the user can see progress.
 */
export function workspaceSetup(context: vscode.ExtensionContext) {
  // Get the workspace folder
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    return; // No workspace is open
  }

  const setupScriptPath = path.join(
    "/",
    "home",
    "workspace",
    "stencila",
    "defaults",
    "setup.sh"
  );

  // Check if setup script exists
  if (!fs.existsSync(setupScriptPath)) {
    return; // Setup script doesn't exist
  }

  // Make sure the script is executable
  try {
    fs.chmodSync(setupScriptPath, "755");
  } catch (error) {
    vscode.window.showErrorMessage(
      `Failed to make setup script executable: ${error}`
    );
    return;
  }

  // Create and show terminal
  const terminal = vscode.window.createTerminal({
    name: "Stencila Workspace Setup",
    location: { viewColumn: vscode.ViewColumn.Beside },
  });
  terminal.show();

  // Refresh the file explorer when files are created
  const watcher = vscode.workspace.createFileSystemWatcher("*");
  watcher.onDidCreate(() => {
    vscode.commands.executeCommand(
      "workbench.files.action.refreshFilesExplorer"
    );
  });

  // When the terminal closes update the file explorer and dispose of the watcher
  const onClose = vscode.window.onDidCloseTerminal((closedTerminal) => {
    if (closedTerminal.name === "Stencila Workspace Setup") {
      vscode.commands.executeCommand(
        "workbench.files.action.refreshFilesExplorer"
      );
      watcher.dispose();
    }
  });

  // Run the setup script
  // Using bash explicitly to ensure script runs properly on Unix-like systems
  if (process.platform === "win32") {
    // On Windows, use bash if available (e.g., Git Bash), otherwise use regular shell
    terminal.sendText(`bash "${setupScriptPath}" || "${setupScriptPath}"`);
  } else {
    terminal.sendText(`bash "${setupScriptPath}"`);
  }

  // Dispose of terminal when extension is deactivated
  context.subscriptions.push(terminal, watcher, onClose);
}
