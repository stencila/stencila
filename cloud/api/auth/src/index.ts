import { serialize, parse } from "cookie";
import { jwtVerify } from "jose";

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext
  ): Promise<Response> {
    const url = new URL(request.url);
    const route = url.pathname.slice(1).split("/");

    if (route[route.length - 2] == "connected") {
      return connected(route.pop(), request, env, ctx);
    }

    const handler = {
      signup,
      signin,
      signout,
      callback,
      connect,
    }[route.pop() ?? ""];

    return handler
      ? handler(request, env, ctx)
      : new Response(null, { status: 404 });
  },
};

const OAUTH_BASE_URL = "https://accounts.stencila.io";
const OAUTH_CLIENT_ID = "14eef3d1b9424a7fb8ac0fd3afa60485";
const OAUTH_SCOPE = "openid profile email";
const OAUTH_REDIRECT_URL = "https://cloud.stencila.io/api/auth/callback";

const KINDE_API_URL = "https://stencila.kinde.com/api";

/**
 * Check a response is OK, and if it is not, throw an error
 */
async function checkResponse(response: Response, message: string) {
  if (!response.ok) {
    const text = await response.text();
    throw new Error(
      `Error ${message}: ${response.status} ${response.statusText}: ${text}`
    );
  }
}

/**
 * Get a token to access the Kinde Management API
 *
 * Based on https://docs.kinde.com/developer-tools/kinde-api/get-access-token-for-connecting-securely-to-kindes-api/#get-the-access-token-nodejs-fetch-example
 */
let KINDE_API_TOKEN: string | undefined;
async function kindeApiToken(env: Env): Promise<string> {
  if (KINDE_API_TOKEN) {
    return KINDE_API_TOKEN;
  }

  const response = await fetch(`${OAUTH_BASE_URL}/oauth2/token`, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
      Accept: "application/json",
    },
    body: new URLSearchParams({
      audience: KINDE_API_URL,
      grant_type: "client_credentials",
      client_id: OAUTH_CLIENT_ID,
      client_secret: env.OAUTH_CLIENT_SECRET,
    }),
  });
  await checkResponse(response, "fetching API token");

  const { access_token } = (await response.json()) as { access_token: string };
  KINDE_API_TOKEN = access_token;

  return KINDE_API_TOKEN;
}

/**
 * Generate an OAuth `state` request parameter and a cookie to store it
 * on the client between the request to the OAuth provider and the request to
 * this worker's callback function.
 */
function generateState() {
  // Generate a random `state` string
  const state = Array.from(crypto.getRandomValues(new Uint8Array(16)))
    .map((values) => values.toString(16))
    .join("");

  // Create an `oauth_state` cookie
  // This does not need to be signed, since the cookie is secure and if the
  // client changes the cookie value the worst that will happen in that the
  // callback handler will return an error response because it does not match
  const cookie = serialize("oauth_state", state, {
    httpOnly: true,
    secure: true,
    // Using `strict` here breaks the flow because it does not include the cookie in
    // the callback request initiated by the auth server
    sameSite: "lax",
    path: "/",
  });

  return { state, cookie };
}

/**
 * Handle a signup request
 */
async function signup(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const { state, cookie } = generateState();

  const params = new URLSearchParams({
    response_type: "code",
    scope: OAUTH_SCOPE,
    client_id: OAUTH_CLIENT_ID,
    redirect_uri: OAUTH_REDIRECT_URL,
    prompt: "create",
    state,
  });

  return new Response(null, {
    headers: {
      Location: `${OAUTH_BASE_URL}/oauth2/auth?${params}`,
      "Set-Cookie": cookie,
    },
    status: 302,
  });
}

/**
 * Handle a signin request
 */
async function signin(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const { state, cookie } = generateState();

  const params = new URLSearchParams({
    response_type: "code",
    scope: OAUTH_SCOPE,
    client_id: OAUTH_CLIENT_ID,
    redirect_uri: OAUTH_REDIRECT_URL,
    prompt: "login",
    state,
  });

  return new Response(null, {
    headers: {
      Location: `${OAUTH_BASE_URL}/oauth2/auth?${params}`,
      "Set-Cookie": cookie,
    },
    status: 302,
  });
}

/**
 * Handle a signout request
 */
async function signout(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const url = new URL(request.url);

  return Response.redirect(`${OAUTH_BASE_URL}/logout?redirect=${url.origin}`);
}

/**
 * Handle a callback from the authentication server (OAuth provider)
 */
async function callback(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const url = new URL(request.url);

  // Get the state from the cookie
  const cookies = parse(request.headers.get("Cookie") || "");
  const stateCookie = cookies["oauth_state"];
  if (!stateCookie) {
    return new Response("Missing state cookie", { status: 400 });
  }

  // Check the state parameter is the same as in the cookie
  const state = url.searchParams.get("state");
  if (state !== stateCookie) {
    return new Response("Invalid state parameter", { status: 400 });
  }

  // Exchange the code parameter for an access token
  const code = url.searchParams.get("code");
  if (!code) {
    throw new Error("No code parameter");
  }
  const response = await fetch(`${OAUTH_BASE_URL}/oauth2/token`, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
    },
    body: new URLSearchParams({
      grant_type: "authorization_code",
      client_id: OAUTH_CLIENT_ID,
      client_secret: env.OAUTH_CLIENT_SECRET,
      redirect_uri: OAUTH_REDIRECT_URL,
      code,
    }),
  });
  await checkResponse(response, "fetching access token");

  // Deserialize response
  interface Response {
    access_token: string;
    expires_in: string;
    token_type: string;
    refresh_token: string;
    id_token: string;
    scope: string;
  }
  const { access_token } = (await response.json()) as Response;

  // Verify token. Based on https://kinde.com/blog/engineering/verifying-jwts-in-cloudflare-workers/#jose-library
  // Public key from https://accounts.stencila.io/.well-known/jwks
  const publicKey = await crypto.subtle.importKey(
    "jwk",
    {
      e: "AQAB",
      n: "w_TxemeTsVrKD-NRoCkyDUySOElUEZocOZe_HQXKggZYZCTh0Cg4srMMwYYsXA6dLPqnoea2cZHFrUjWvwfT31Jdm3CkZEANuKJas6lwRqXRWjiV_Y7Ze0dcMzSXe3GSREQaU3SyGc6MxszszkypfI2A2xHZNbozNZ4dddd46bMaIwDtnTfneKv3R7kRo_quGDhAiiIH6xQW3B9cDtU7OJitfCFcwRS3oNb7Wt6oxI_RQEVq5xz9e9y-Aor6ibToNgnuBzrt6M7uet4OahnYg7amCoXfmHkmI95H3s4az7zif1FpPBk7DQL6qKhRu1tsPzChGxHFxF8GDTKf7AKw9Q",
      alg: "RS256",
      kty: "RSA",
      use: "sig",
    },

    {
      name: "RSASSA-PKCS1-v1_5",
      hash: { name: "SHA-256" },
    },
    false,
    ["verify"]
  );

  try {
    const { payload } = await jwtVerify(access_token, publicKey);
    return new Response(JSON.stringify(payload), {
      headers: { "Content-Type": "application/json" },
    });
  } catch (error) {
    return new Response(`Token verification failed: ${error}`, {
      status: 401,
    });
  }
}

/**
 * Connect an app to a user account
 *
 * This is steps 2 & 3 of https://docs.kinde.com/integrate/connected-apps/add-connected-apps/#step-4-get-an-access-token-via-the-kinde-management-api
 * It gets a URL that we redirect the user to.
 */
async function connect(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const url = new URL(request.url);

  // Get the app and user ids
  const app = url.searchParams.get("app");
  const user = url.searchParams.get("user");
  if (!(app && user)) {
    return new Response(`Missing "app" and/or "user" query parameter`, {
      status: 400,
    });
  }

  // Get the URL and session id to redirect the user to
  const params = new URLSearchParams({
    key_code_ref: app,
    user_id: user,
  });
  const response = await fetch(
    `${KINDE_API_URL}/v1/connected_apps/auth_url?${params}`,
    {
      headers: {
        Authorization: `Bearer ${await kindeApiToken(env)}`,
        Accept: "application/json",
      },
    }
  );
  await checkResponse(response, "fetching app connection URL");

  const payload = (await response.json()) as {
    url: string;
    session_id: string;
  };

  return Response.redirect(payload.url, 302);
}

/**
 * Handle a callback after successfully connecting an app to a user account
 *
 * Requests an access token for the `app` and sets an `<app>_access_token` cookie.
 */
async function connected(
  app: string | undefined,
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const url = new URL(request.url);

  // Ensure the app is valid
  if (app !== "github") {
    return new Response(`Invalid app`, { status: 400 });
  }

  // Get the session id
  const session_id = url.searchParams.get("session_id");
  if (!session_id) {
    return new Response(`Missing "session_id" query parameter`, {
      status: 400,
    });
  }

  // Request an access token
  const params = new URLSearchParams({
    session_id,
  });
  const response = await fetch(
    `${KINDE_API_URL}/v1/connected_apps/token?${params}`,
    {
      headers: {
        Authorization: `Bearer ${await kindeApiToken(env)}`,
        Accept: "application/json",
      },
    }
  );
  await checkResponse(response, "fetching app access token");

  const payload = (await response.json()) as {
    access_token: string;
    access_token_expiry: string;
  };

  // Create a cookie with the session id (to be able to refresh the token?),
  // the token itself, and the token expiry
  const cookie = serialize(
    `${app}_access_token`,
    JSON.stringify({
      session: session_id,
      token: payload.access_token,
      expiry: payload.access_token_expiry,
    }),
    {
      httpOnly: true,
      secure: true,
      sameSite: "lax",
      path: "/",
    }
  );

  return new Response(null, {
    headers: {
      Location: "/",
      "Set-Cookie": cookie,
    },
    status: 302,
  });
}
