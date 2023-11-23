/**
 * A worker to serve JSON Schema, JSON-LD and other specs from https://schema.org
 */

export default {
  async fetch(request: Request) {
    const url = new URL(request.url);

    const version = "main";
    const dir = "json";
    const path = url.pathname;

    const response = await fetch(
      `https://raw.github.com/stencila/stencila/${version}/${dir}/${path}`
    );

    return response;
  },
};
