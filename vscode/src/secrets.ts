import * as vscode from "vscode";

import { PROVIDER_ID } from "./authentication";
import { runCli } from "./clis";

const SECRET_NAMES = [
  "STENCILA_API_TOKEN",
  "ANTHROPIC_API_KEY",
  "GOOGLE_AI_API_KEY",
  "OPENAI_API_KEY",
  "MISTRAL_API_KEY",
];

export function registerSecretsCommands(context: vscode.ExtensionContext) {
  // Command to set a secret
  let setSecretCommand = vscode.commands.registerCommand(
    "stencila.secrets.set",
    async () => {
      // Ask user to select a secret name
      const secretName = await vscode.window.showQuickPick(SECRET_NAMES, {
        placeHolder: "Select a secret to add or set",
      });

      if (!secretName) {
        return; // User cancelled the selection
      }

      // Ask user to input the secret value
      const secretValue = await vscode.window.showInputBox({
        prompt: `Enter the value for ${secretName}`,
        password: true,
      });

      if (secretValue === undefined) {
        return; // User cancelled the input
      }

      // Store the secret
      try {
        await runCli(context, ["secrets", "set", secretName], secretValue);
        vscode.window.showInformationMessage(`Secret ${secretName} set.`);
      } catch (error) {
        vscode.window.showErrorMessage(`Error setting secret: ${error}`);
      }
    }
  );

  // Command to delete a secret
  let deleteSecretCommand = vscode.commands.registerCommand(
    "stencila.secrets.delete",
    async () => {
      // Ask user to select a secret to delete
      const secretName = await vscode.window.showQuickPick(SECRET_NAMES, {
        placeHolder: "Select a secret to remove",
      });

      if (!secretName) {
        return; // User cancelled the selection
      }

      // Delete the secret
      try {
        await runCli(context, ["secrets", "delete", secretName]);
        vscode.window.showInformationMessage(`Secret ${secretName} removed.`);
      } catch (error) {
        vscode.window.showErrorMessage(`Error deleting secret: ${error}`);
      }
    }
  );

  context.subscriptions.push(setSecretCommand, deleteSecretCommand);
}
