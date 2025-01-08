import * as vscode from "vscode";

import { posthog } from "posthog-js";

// Eventing is disabled by default and can be turned on/off
// while extension is running
let isEnabled = false;

/**
 * Capture an event (if eventing is enabled)
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function event(name: string, data?: any) {
  if (isEnabled) {
    posthog.capture(`vsce_${name}`, data);
  }
}

/**
 * Initialize eventing including a registering a listener for
 * changes to settings
 */
export function registerEventing(context: vscode.ExtensionContext) {
  const onChange = (enabled: boolean, initial = false) => {
    if (enabled) {
      posthog.init("LeXA_J7NbIow0-mEejPwazN7WvZCj-mFKSvLL5oM4w0", {
        api_host: "https://events.stencila.cloud",
      });

      isEnabled = true;

      if (!initial) {
        event("eventing_enabled");
      }
    } else {
      isEnabled = false;
    }

    if (!initial) {
      vscode.window.showInformationMessage(
        `Stencila usage logging ${enabled ? "enabled" : "disabled"}`
      );
    }
  };

  // Initial setting at time of extension registration
  onChange(vscode.env.isTelemetryEnabled, true);

  // Listen for changes to global setting
  context.subscriptions.push(vscode.env.onDidChangeTelemetryEnabled(onChange));

  // Listen for changes to extension setting
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("stencila.logging.usage")) {
        onChange(
          vscode.workspace
            .getConfiguration("stencila")
            .get<boolean>("logging.usage")!
        );
      }
    })
  );
}
