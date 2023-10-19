/**
 * A worker to create Shields.io Endpoint Badges (https://shields.io/badges/endpoint-badge)
 *
 * To create a badge using this worker use a URL such as:
 *
 *   https://img.shields.io/endpoint?url=https://shields.stencila.dev/codecov
 *
 * In the future, we may use this for other badge types beyond coverage,
 * fetching data from other APIs.
 */

export interface Env {
  CODECOV_TOKEN: string;
}

export default {
  async fetch(request: Request, env: Env) {
    const url = new URL(request.url);

    const badge = await codecov(url, env);

    return new Response(JSON.stringify(badge), {
      headers: {
        "content-type": "application/json;charset=UTF-8",
      },
    });
  },
};

async function codecov(url: URL, env: Env) {
  const comp = url.searchParams.get("comp");

  const endpoint = new URL(
    "https://api.codecov.io/api/v2/github/stencila/repos/stencila/totals/?branch=main"
  );
  if (comp) {
    endpoint.searchParams.append("component_id", comp);
  }

  const response = await fetch(endpoint, {
    method: "GET",
    headers: {
      accept: "application/json",
      authorization: `Bearer ${env.CODECOV_TOKEN}`,
    },
  });

  const json = await response.json();
  const coverage = Math.round(json.totals.coverage) * 1.0;

  const message = `${coverage}%`;
  const color = `hsl(${
    Math.round(coverage < 50 ? 0 : (120 * (coverage - 50)) / 50)
  }, 80%, 45%)`;

  return {
    schemaVersion: 1,
    label: "",
    message,
    color,
  };
}
