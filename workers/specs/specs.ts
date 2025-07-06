/**
 * A worker to serve JSON Schema, JSON-LD and other Stencila specs
 *
 * Acts as a reverse proxy to translate requested paths and `Content-Type`
 * headers into requests for files, usually in this GitHub repo.
 */
export default {
  async fetch(request: Request) {
    const CORS_HEADERS = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET,HEAD,OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type,Accept",
      // Cache the pre-flight result for 24 h
      "Access-Control-Max-Age": "86400",
    };

    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    const url = new URL(request.url);
    let path = url.pathname.slice(1);

    // Determine which version to use
    let version = "main";
    const parts = path.split("/");
    if (parts.length > 1) {
      version = parts[0];
      path = parts.slice(1).join("/");
    }

    // Apply basic content negotiation based on the `Accept` header
    const accept = request.headers.get("Accept") ?? "";
    if (accept.includes("application/ld+json") && !path.endsWith(".jsonld")) {
      path += ".jsonld";
    } else if (
      accept.includes("application/schema+json") &&
      !path.endsWith("schema.json")
    ) {
      path += ".schema.json";
    }

    // Complete path and content type header value
    let contentType = "text/plain; charset=utf-8";
    if (path.endsWith(".jsonld")) {
      path = `json/${path}`;
      contentType = "application/ld+json";
    } else if (path.endsWith(".schema.json")) {
      path = `json/${path}`;
      contentType = "application/schema+json";
    }

    const file = await fetch(
      `https://raw.githubusercontent.com/stencila/stencila/${version}/${path}`
    );

    return new Response(file.body, {
      headers: {
        "Content-Type": contentType,
        ...CORS_HEADERS,
      },
    });
  },
};
