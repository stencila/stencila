/**
 * Cloudflare Worker to serve https://stencila/node
 */

import { Request } from "@cloudflare/workers-types";

import type { Entity, Node, NodeType } from "../../ts/src";

import { decode as decodeJzb64 } from "../../web/src/codecs/jzb64";
import { encode as encodeHtml } from "../../web/src/codecs/html";

export default {
  async fetch(request: Request): Promise<Response> {
    try {
      // Handle CORS preflight
      if (request.method === "OPTIONS") {
        return new Response(null, {
          status: 204,
          headers: corsHeaders(),
        });
      }

      // Only allow GET method
      if (!["GET"].includes(request.method)) {
        return new Response("Method Not Allowed", {
          status: 405,
          headers: {
            Allow: "GET, OPTIONS",
            ...corsHeaders(),
          },
        });
      }

      const url = new URL(request.url);
      let type = url.searchParams.get("type") as NodeType | null;
      const path = url.searchParams.get("path");
      const jzb64 = url.searchParams.get("jzb64");

      // Create a node from query parameters
      let node: Entity | null = null;
      if (jzb64) {
        try {
          node = await decodeJzb64(jzb64);
          if (!type) {
            type = node.type as NodeType;
          }
        } catch (error) {
          console.error("Error decoding jzb64:", error);
        }
      }

      // Generate and return HTML page
      const html = nodeHtml(url, type, path, node);
      return new Response(html, {
        headers: {
          "Content-Type": "text/html",
          "Cache-Control": "public, max-age=3600",
          ...corsHeaders(),
        },
      });
    } catch (error) {
      console.error("Internal error:", error);
      return new Response("Internal Server Error", {
        status: 500,
        headers: corsHeaders(),
      });
    }
  },
};

/**
 * Create CORS headers
 */
function corsHeaders(): Record<string, string> {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, HEAD, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, Accept, Accept-Encoding",
    "Access-Control-Max-Age": "86400", // 24 hours
  };
}

/**
 * Generate a HTML page for a node
 */
function nodeHtml(
  url: URL,
  nodeType: NodeType | null,
  nodePath: string | null,
  node: Entity | null
): string {
  const title = nodeTitle(nodeType, node);
  const description = nodeDescription(nodeType, node);

  const jsonld = JSON.stringify({
    "@context": "https://stencila.org/context.jsonld",
    $schema: `https://stencila.org/${nodeType}.schema.json`,
    ...node,
  });

  const logo = "https://stencila.io/web/logo.png";
  const webVersion = "dev";

  let body = "";
  if (!nodeType) {
    body = `<img src="${logo}">`;
  } else if (!node) {
    body = `<div class="message"><code>${nodeType}</code>`;
    if (nodePath) {
      body += ` embedded in document at path <code>${nodePath}</code>`;
    }
    body += "</div>";
  } else {
    body = `<stencila-static-view>${encodeHtml(node as unknown as Node)}</stencila-static-view>`;
  }

  return `<!DOCTYPE html>
<html lang="en">
  <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">

      <title>${title}</title>
      <meta name="description" content="${description}">
      
      <meta property="og:type" content="website">
      <meta property="og:site_name" content="Stencila" />
      <meta property="og:url" content="${url}">
      <meta property="og:title" content="${title}">
      <meta property="og:description" content="${description}">
      <meta property="og:image" content="${logo}">

      <meta name="twitter:site" content="@stencila">
      <meta name="twitter:card" content="summary_large_image">
      <meta name="twitter:url" content="${url}">
      <meta name="twitter:title" content="${title}">
      <meta name="twitter:description" content="${description}">
      <meta name="twitter:image" content="${logo}">
      
      <link rel="icon" type="image/png" href="https://stencila.io/web/${webVersion}/images/favicon.png" />
      <link rel="preconnect" href="https://fonts.googleapis.com" />
      <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet" />
      <link rel="stylesheet" type="text/css" href="https://stencila.io/web/${webVersion}/themes/default.css" />
      <link rel="stylesheet" type="text/css" href="https://stencila.io/web/${webVersion}/views/dynamic.css" />
      <script type="module" src="https://stencila.io/web/${webVersion}/views/dynamic.js"></script>

      <script type="application/ld+json">${jsonld}</script>

      <style>
        .message {
          margin-top: 2em;
          text-align: center;
        }

        stencila-static-view {
          border-radius: 3px;
          border: 1px solid #ddd;
          display: block;
          margin: 2em auto;
          max-width: 100ch;
        }
      </style>
  </head>
  <body>
    ${body}
  </body>
</html>`;
}

/**
 * Generate a title for a node
 *
 * This title is used in <head> <title> and <meta> tags
 * and in <body> if a custom <stencila-...> element
 * can not be generated for the node.
 *
 * Simply converts PascalCase node type to Title Case
 */
export function nodeTitle(
  nodeType: NodeType | null,
  node: Entity | null
): string {
  if (nodeType === null) {
    return "Stencila Node";
  }

  return nodeType.replace(/([A-Z])/g, " $1").trim();
}

/**
 * Generate a description for a node
 *
 * This description is used in <head> <meta> tags
 * and in <body> if a custom <stencila-...> element
 * can not be generated for the node.
 */
export function nodeDescription(
  nodeType: NodeType | null,
  node: Entity | null
): string {
  if (nodeType === null) {
    return "A Stencila node of unknown type";
  }

  return {}[nodeType] || `A Stencila ${nodeType} node`;
}
