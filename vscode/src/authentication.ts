import * as vscode from "vscode";

export const PROVIDER_ID = "stencila";

// If developing against Stencila Cloud running locally use
// http://localhost:5173 and http://localhost:8787/v1
const CLOUD_URL = "https://stencila.cloud";
const API_URL = "https://api.stencila.cloud/v1";

/**
 * Register the Stencila Cloud authentication provider
 */
export function registerAuthenticationProvider(
  context: vscode.ExtensionContext
) {
  context.subscriptions.push(
    vscode.authentication.registerAuthenticationProvider(
      PROVIDER_ID,
      "Stencila Cloud",
      new StencilaCloudProvider(context),
      { supportsMultipleAccounts: false }
    )
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.cloud.signin", async () => {
      try {
        const session = await vscode.authentication.getSession(
          PROVIDER_ID,
          [],
          {
            createIfNone: true,
          }
        );
        vscode.window.showInformationMessage(
          `Signed in to Stencila Cloud as ${session.account.label}`
        );
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to sign in to Stencila Cloud: ${error}`
        );
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("stencila.cloud.signout", async () => {
      try {
        const session = await vscode.authentication.getSession(
          PROVIDER_ID,
          [],
          {
            silent: true,
          }
        );
        if (session) {
          const provider = new StencilaCloudProvider(context);
          await provider.removeSession(session.id);
          vscode.window.showInformationMessage(
            "Successfully signed out from Stencila Cloud"
          );
        } else {
          vscode.window.showInformationMessage(
            "No active Stencila Cloud session to sign out from"
          );
        }
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to sign out from Stencila Cloud: ${error}`
        );
      }
    })
  );
}

export class StencilaCloudProvider implements vscode.AuthenticationProvider {
  private context: vscode.ExtensionContext;
  private sessionChangeEmitter =
    new vscode.EventEmitter<vscode.AuthenticationProviderAuthenticationSessionsChangeEvent>();

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
  }

  get onDidChangeSessions() {
    return this.sessionChangeEmitter.event;
  }

  async getSessions(): Promise<vscode.AuthenticationSession[]> {
    const storedSessions = await this.context.secrets.get("sessions");
    if (storedSessions) {
      return JSON.parse(storedSessions);
    }
    return [];
  }

  async createSession(): Promise<vscode.AuthenticationSession> {
    // Generate the callback URL the user will be redirected to
    // The URL path needs to start with <publisher>.<extension-name>
    const callbackPath = "/auth/callback";
    const callbackUri = await vscode.env.asExternalUri(
      vscode.Uri.parse(
        `${vscode.env.uriScheme}://stencila.stencila${callbackPath}`
      )
    );

    // Open Stencila Cloud in the user's browser with that callback URL
    const cb = encodeURIComponent(callbackUri.toString());
    const name = "vscode-extension";
    const desc = encodeURIComponent(
      "Access token for Stencila VSCode extension"
    );
    const authUri = vscode.Uri.parse(
      `${CLOUD_URL}/signin/vscode?callback=${cb}&name=${name}&description=${desc}`
    );
    vscode.env.openExternal(authUri);

    // Wait for callback URL to be requested with `?otc=xxxx` query params
    const otc = await new Promise<string | null>((resolve, reject) => {
      const disposable = vscode.window.registerUriHandler({
        handleUri(uri: vscode.Uri) {
          if (uri.path.startsWith(callbackPath)) {
            const queryParams = new URLSearchParams(uri.query);
            const otc = queryParams.get("otc");
            disposable.dispose();
            resolve(otc);
          }
        },
      });
    });
    if (!otc) {
      throw new Error("No one-time code received by callback");
    }

    // Swap the code for the API token
    const tokenResponse = await fetch(`${API_URL}/access-tokens/otc`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ otc }),
    });
    if (!tokenResponse.ok) {
      let message;
      try {
        const error: any = await tokenResponse.json();
        message = error?.message ?? error?.error ?? JSON.stringify(error);
      } catch {
        message = `HTTP error status: ${tokenResponse.status}`;
      }
      throw new Error(message);
    }

    // Check if the required properties exist in the response
    const { token, userId } = (await tokenResponse.json()) as {
      token: string;
      userId: string;
    };
    if (!token || !userId) {
      throw new Error("Invalid response: missing token or userId");
    }

    // Get the user from the user_id. This also checks that the
    // access_token is valid
    const userResponse = await fetch(`${API_URL}/users/me`, {
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    });

    let userLabel;
    if (userResponse.status === 200) {
      // Check if the required properties exist in the response
      const { username, firstName, lastName } = (await userResponse.json()) as {
        username?: string;
        firstName?: string;
        lastName?: string;
      };
      userLabel = username ?? firstName ?? lastName ?? userId.slice(0, 12);
    } else if (userResponse.status === 404) {
      // It is possible that we do not yet have an entry in the users table
      // yet so use the user id
      userLabel = userId.slice(0, 12);
    } else {
      throw new Error(`HTTP error status: ${userResponse.status}`);
    }

    // Create, store, emit and return session
    const session: vscode.AuthenticationSession = {
      id: userId,
      accessToken: token,
      account: {
        id: userId,
        label: userLabel,
      },
      scopes: [],
    };

    await this.context.secrets.store("sessions", JSON.stringify([session]));

    this.sessionChangeEmitter.fire({
      added: [session],
      removed: [],
      changed: [],
    });

    return session;
  }

  async removeSession(sessionId: string): Promise<void> {
    const sessions = await this.getSessions();

    // Find the session
    const session = sessions.find((session) => session.id === sessionId);

    let token = session?.accessToken;
    if (token) {
      // Delete the access token on Stencila Cloud
      const response = await fetch(`${API_URL}/access-tokens/token/${token}`, {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      });
      if (!response.ok) {
        let message;
        try {
          const error: any = await response.json();
          message = error?.message ?? error?.error ?? JSON.stringify(error);
        } catch {
          message = `HTTP error status: ${response.status}`;
        }
        throw new Error(message);
      }
    }

    // Remove the session, reset the store and emit the event
    const updatedSessions = sessions.filter(
      (session) => session.id !== sessionId
    );
    await this.context.secrets.store(
      "sessions",
      JSON.stringify(updatedSessions)
    );

    this.sessionChangeEmitter.fire({
      added: [],
      removed: [{ id: sessionId } as vscode.AuthenticationSession],
      changed: [],
    });
  }
}
