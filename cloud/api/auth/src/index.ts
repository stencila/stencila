import { serialize, parse } from "cookie";
import { jwtVerify } from "jose";

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext
  ): Promise<Response> {
    const url = new URL(request.url);
    const route = url.pathname.slice(1).split("/").pop();

    const handler = {
      signup,
      signin,
      signout,
      callback,
    }[route ?? ""];

    return handler
      ? handler(request, env, ctx)
      : new Response(null, { status: 404 });
  },
};

const OAUTH_BASE_URL = "https://accounts.stencila.io";
const OAUTH_CLIENT_ID = "14eef3d1b9424a7fb8ac0fd3afa60485";
const OAUTH_SCOPE = "openid profile email";
const OAUTH_REDIRECT_URL = "https://cloud.stencila.io/api/auth/callback"; 

/**
 * Generate an Oauth `state` request parameter and cookie to store it
 * on the client between the request to the Oauth provider and the request to
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
 * Handle a callback from the authentication server (Oauth provider)
 */
async function callback(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  const url = new URL(request.url);

  // Get the signed state from the cookie
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
  if (!response.ok) {
    const text = await response.text();
    throw new Error(
      `Error fetching access token: ${response.status} ${response.statusText}: ${text}`
    );
  }

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
