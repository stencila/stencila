/**
 * Cloudflare Worker to serve Stencila's web distribution
 *
 * Serves static assets (JS, CSS, fonts, images, etc.) from versioned folders
 * in the `web-dist` R2 bucket with proper content types and optimizations.
 * 
 * Serves requests to https://stencila.io/web/*
 */
import { Request, R2Bucket, R2ObjectBody } from "@cloudflare/workers-types";

export interface Env {
  WEB_DIST_BUCKET: R2Bucket;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      // Handle CORS preflight
      if (request.method === "OPTIONS") {
        return new Response(null, {
          status: 204,
          headers: getCorsHeaders(),
        });
      }

      // Only allow GET and HEAD methods
      if (!["GET", "HEAD"].includes(request.method)) {
        return new Response("Method Not Allowed", {
          status: 405,
          headers: {
            Allow: "GET, HEAD, OPTIONS",
            ...getCorsHeaders(),
          },
        });
      }

      const url = new URL(request.url);
      let path = url.pathname.slice(5); // remove leading '/web/' from path

      let obj: R2ObjectBody | null = null;
      let contentEncoding: string | null = null;

      // Try to serve Brotli-compressed version first
      if (supportsBrotli(request)) {
        obj = await env.WEB_DIST_BUCKET.get(`${path}.br`);
        if (obj) {
          contentEncoding = "br";
        }
      }

      // Fall back to uncompressed version
      if (!obj) {
        obj = await env.WEB_DIST_BUCKET.get(path);
      }

      // Return 404 if file not found
      if (!obj) {
        return new Response("Not Found", {
          status: 404,
          headers: getCorsHeaders(),
        });
      }

      // Build response headers
      const headers = new Headers({
        "Content-Type": getContentType(path),
        "Cache-Control": getCacheControl(path),
        ETag: obj.httpEtag,
        "Last-Modified": obj.uploaded.toUTCString(),
        ...getCorsHeaders(),
      });

      // Add content encoding if compressed
      if (contentEncoding) {
        headers.set("Content-Encoding", contentEncoding);
      }

      // Add security headers
      headers.set("X-Content-Type-Options", "nosniff");
      headers.set("Referrer-Policy", "strict-origin-when-cross-origin");

      // Handle HEAD requests (return headers only)
      if (request.method === "HEAD") {
        headers.set("Content-Length", obj.size.toString());
        return new Response(null, { headers });
      }

      // Return file content
      return new Response(await obj.bytes(), { headers });
    } catch (error) {
      console.error("Worker error:", error);
      return new Response("Internal Server Error", {
        status: 500,
        headers: getCorsHeaders(),
      });
    }
  },
};

/**
 * Create CORS headers
 */
function getCorsHeaders(): Record<string, string> {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, HEAD, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, Accept, Accept-Encoding",
    "Access-Control-Max-Age": "86400", // 24 hours
  };
}

/**
 * Get MIME type based on file extension with proper charset for text files
 */
function getContentType(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() || "";

  const mimeTypes: Record<string, string> = {
    // JavaScript
    js: "application/javascript; charset=utf-8",
    mjs: "application/javascript; charset=utf-8",
    jsx: "application/javascript; charset=utf-8",

    // CSS
    css: "text/css; charset=utf-8",

    // HTML
    html: "text/html; charset=utf-8",
    htm: "text/html; charset=utf-8",

    // Images
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    gif: "image/gif",
    svg: "image/svg+xml",
    webp: "image/webp",
    avif: "image/avif",
    ico: "image/x-icon",

    // Fonts
    woff: "font/woff",
    woff2: "font/woff2",
    ttf: "font/ttf",
    otf: "font/otf",
    eot: "application/vnd.ms-fontobject",

    // Documents
    json: "application/json; charset=utf-8",
    xml: "application/xml; charset=utf-8",
    txt: "text/plain; charset=utf-8",
    md: "text/markdown; charset=utf-8",

    // Audio/Video
    mp3: "audio/mpeg",
    mp4: "video/mp4",
    webm: "video/webm",
    ogg: "audio/ogg",
    wav: "audio/wav",

    // Archives
    zip: "application/zip",
    gz: "application/gzip",

    // Other
    pdf: "application/pdf",
    map: "application/json; charset=utf-8", // Source maps
  };

  return mimeTypes[ext] || "application/octet-stream";
}

/**
 * Get cache control header based on file type
 */
function getCacheControl(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() || "";

  // Check if path contains version hash (common pattern: filename.hash.ext)
  const hasVersionHash = /\.[a-f0-9]{8,}\.(js|css|woff2?|ttf|otf)$/i.test(path);

  // Long cache for versioned assets, shorter for others
  if (hasVersionHash || ["woff", "woff2", "ttf", "otf"].includes(ext)) {
    return "public, max-age=31536000, immutable"; // 1 year
  }

  // Medium cache for images and other static assets
  if (
    ["png", "jpg", "jpeg", "gif", "svg", "webp", "avif", "ico"].includes(ext)
  ) {
    return "public, max-age=2592000"; // 30 days
  }

  // Shorter cache for HTML and other dynamic content
  return "public, max-age=3600"; // 1 hour
}

/**
 * Check if client supports Brotli compression
 */
function supportsBrotli(request: Request): boolean {
  const acceptEncoding = request.headers.get("Accept-Encoding") || "";
  const clientEncoding = (request.cf?.clientAcceptEncoding as string) || "";
  return /\bbr\b/.test(acceptEncoding) || /\bbr\b/.test(clientEncoding);
}
