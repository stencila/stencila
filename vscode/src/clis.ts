import { spawn } from "child_process";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

import * as vscode from "vscode";

/**
 * Get the path to the `stencila` CLI
 *
 * Used to run the LSP server and other commands.
 */
export function cliPath(context: vscode.ExtensionContext): string {
  switch (context.extensionMode) {
    case vscode.ExtensionMode.Development:
    case vscode.ExtensionMode.Test:
      return path.join(__dirname, "..", "..", "target", "debug", "stencila");
    case vscode.ExtensionMode.Production: {
      // Attempt to obtain from the `cli` sub-dir of the extension, falling back to
      // searching for `stencila` on the path
      const cli = path.join(
        context.extensionPath,
        "cli",
        os.platform() === "win32" ? "stencila.exe" : "stencila"
      );
      return fs.existsSync(cli) ? cli : "stencila";
    }
  }
}

/**
 * Run the CLI
 *
 * Throws an error if the exist code was not 0
 */
export async function runCli(
  context: vscode.ExtensionContext,
  args: string[],
  stdin?: string
) {
  return new Promise((resolve, reject) => {
    const process = spawn(cliPath(context), args);

    let stdout = "";
    process.stdout.on("data", (data) => {
      stdout += data;
    });

    let stderr = "";
    process.stderr.on("data", (data) => {
      stderr += data;
    });

    if (stdin) {
      process.stdin.write(stdin);
      process.stdin.end();
    }

    process.on("close", (code) => {
      if (code !== 0) {
        reject(new Error(stderr));
      } else {
        resolve({ stdout, stderr });
      }
    });
  });
}
