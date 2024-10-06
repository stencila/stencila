import * as vscode from "vscode";

import { PROVIDER_ID } from "./authentication";

const SECRET_NAMES = [
  "STENCILA_API_TOKEN",
  "ANTHROPIC_API_KEY",
  "GOOGLE_AI_API_KEY",
  "OPENAI_API_KEY",
  "MISTRAL_API_KEY",
];

export async function collectSecrets(
  context: vscode.ExtensionContext
): Promise<Record<string, string>> {
  const secrets: Record<string, string> = {};

  // If the user has a Stencila Cloud auth session then
  // use it's token as the STENCILA_API_TOKEN
  const session = await vscode.authentication.getSession(PROVIDER_ID, []);
  if (session) {
    secrets["STENCILA_API_TOKEN"] = session.accessToken;
  }

  // Collect other secrets
  // Note: if the STENCILA_API_TOKEN has been explicitly set then it will
  // override the token value in the auth session. This is intentional.
  for (const name of SECRET_NAMES) {
    const value = await context.secrets.get(name);
    if (value) {
      secrets[name] = value;
    }
  }

  return secrets;
}

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
      await context.secrets.store(secretName, secretValue);
      vscode.window.showInformationMessage(
        `Secret ${secretName} set. Restart Stencila LSP Server for change to take effect.`
      );
    }
  );

  // Command to delete a secret
  let deleteSecretCommand = vscode.commands.registerCommand(
    "stencila.secrets.delete",
    async () => {
      // Get all stored secret names
      const storedSecrets = await Promise.all(
        SECRET_NAMES.map(async (name) => {
          const value = await context.secrets.get(name);
          return value ? name : null;
        })
      );
      const existingSecrets = storedSecrets.filter(
        (name) => name !== null
      ) as string[];

      // Ask user to select a secret to delete
      const secretToDelete = await vscode.window.showQuickPick(
        existingSecrets,
        {
          placeHolder: "Select a secret to remove",
        }
      );

      if (!secretToDelete) {
        return; // User cancelled the selection
      }

      // Delete the secret
      await context.secrets.delete(secretToDelete);
      vscode.window.showInformationMessage(
        `Secret ${secretToDelete} removed. Restart Stencila LSP Server for change to take effect.`
      );
    }
  );

  context.subscriptions.push(setSecretCommand, deleteSecretCommand);
}
