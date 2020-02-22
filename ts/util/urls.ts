import { versionMajor } from './version'

/**
 * Get the URL to the JSON-LD for this schema's `@context` or
 * for a specific term e.g. `CodeChunk`, `outputs`.
 *
 * The `@context`'s URL needs to have a trailing slash because
 * it gets prefixed to all keys during JSON-LD expansion.
 * e.g. the term `CodeChunk` gets expanded to `http://schema.stenci.la/v0/jsonld/CodeChunk`
 * (which in gets redirected to `https://unpkg.com/@stencila/schema@0.32.1/dist/CodeChunk.jsonld`)
 *
 * @param term The term (type or property) to generate the
 *             URL for. Defaults to empty string i.e. the context.
 */
export function jsonLdUrl(term = '') {
  const version = versionMajor()
  return `http://schema.stenci.la/v${version}/jsonld/${term}`
}
