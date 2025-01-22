import * as path from "path";
import * as fs from "fs";

import * as vscode from "vscode";

/**
 * Check for, and run, any workspace setup script
 *
 * Checks for a `.stencila/workspace/setup.sh` in the workspace.
 * If it exists, then opens up a new terminal and run that script
 * in it so the user can see progress.
 */
export function workspaceSetup(context: vscode.ExtensionContext) {
  // Get the workspace folder
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    return; // No workspace is open
  }

  const workspaceRoot = workspaceFolders[0].uri.fsPath;
  const setupScriptPath = path.join(
    workspaceRoot,
    ".stencila",
    "workspace",
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
    message:
      "Setting up your workspace by running the `.stencila/workspace/setup.sh` script",
  });
  terminal.show();

  // Run the setup script
  // Using bash explicitly to ensure script runs properly on Unix-like systems
  if (process.platform === "win32") {
    // On Windows, use bash if available (e.g., Git Bash), otherwise use regular shell
    terminal.sendText(`bash "${setupScriptPath}" || "${setupScriptPath}"`);
  } else {
    terminal.sendText(`bash "${setupScriptPath}"`);
  }

  // Dispose of terminal when extension is deactivated
  context.subscriptions.push(terminal);
}
