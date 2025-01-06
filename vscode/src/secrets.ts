/**
 * Module for managing secrets used by Stencila
 *
 * Previously we used the approach of calling `stencila secrets set` to pass
 * secrets through the system keyring. e.g.
 *
 *  await runCli(this.context, ["secrets", "set", "SOME_API_KEY"], key);
 *
 * However, this caused issues for some users on MacOS because they constantly
 * had to approve access to the keyring.
 *
 * Furthermore, when the user signs out from VSCode the STENCILA_API_TOKEN
 * is deleted on Stencila Cloud. If this token was passed down to the keyring
 * it should be deleted too, but then we might be deleting another token
 * that the user explicitly set for the CLI to use.
 *
 * For these reasons we have isolated the two sets of secrets. Users can set
 * secrets at the CLI (on the keyring) but then override them by
 * adding/removing them in VSCode.
 */

import * as vscode from "vscode";

import { PROVIDER_ID } from "./authentication";
import { event } from "./events";

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
  const setSecretCommand = vscode.commands.registerCommand(
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
        `Secret ${secretName} set. Restart Stencila Language Server for change to take effect.`
      );

      event("token_set", { name: secretName });
    }
  );

  // Command to delete a secret
  const deleteSecretCommand = vscode.commands.registerCommand(
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
      const secretName = await vscode.window.showQuickPick(existingSecrets, {
        placeHolder: "Select a secret to remove",
      });

      if (!secretName) {
        return; // User cancelled the selection
      }

      // Delete the secret
      await context.secrets.delete(secretName);
      vscode.window.showInformationMessage(
        `Secret ${secretName} removed. Restart Stencila Language Server for change to take effect.`
      );

      event("token_delete", { name: secretName });
    }
  );

  context.subscriptions.push(setSecretCommand, deleteSecretCommand);
}
