import fs from 'fs'
import path from 'path'
import { versionMajor } from './version'

// Lazily loaded JSON-LD context used for mapping
// between Stencila Schema type and property names
// and URLs
let CONTEXT: Record<string, string | { '@id': string }>

// Maps between vocabulary terms (Stencila Schema types and
// property names) and their compact IRIs ('@id's) and vice-versa.
const termToId: Record<string, string> = {}
const idToTerm: Record<string, string> = {}

// Initialize maps. Called once, lazily.
function initMaps() {
  const context = jsonLdContext()
  for (const [term, value] of Object.entries(context)) {
    if (typeof value === 'string' && !term.startsWith('@')) {
      termToId[term] = value
      idToTerm[value] = term
    } else if (typeof value === 'object') {
      const id = value['@id']
      termToId[term] = id
      idToTerm[id] = term
    }
  }
}

/**
 * Get the URL of Stencila Schema's JSON-LD `@context` or
 * for a specific `stencila:` term e.g. `CodeChunk`, `outputs`.
 *
 * The `@context`'s URL needs to have a trailing slash because
 * it gets prefixed to all keys during JSON-LD expansion.
 * e.g. the term `CodeChunk` gets expanded to `http://schema.stenci.la/v0/jsonld/CodeChunk`
 * (which in gets redirected to `https://unpkg.com/@stencila/schema@0.32.1/dist/CodeChunk.jsonld`)
 *
 * @param term The Stencila term (type or property) to generate the
 *             URL for. Defaults to empty string i.e. the context.
 */
export function jsonLdUrl(term = ''): string {
  const version = versionMajor()
  return `http://schema.stenci.la/v${version}/jsonld/${term}`
}

/**
 * Get Stencila Schema's JSON-LD `@context` as an object.
 */
export function jsonLdContext(): typeof CONTEXT {
  if (CONTEXT === undefined) {
    const json = fs.readFileSync(
      path.join(
        __dirname,
        '..',
        ...(__filename.endsWith('ts') ? ['..', 'public'] : []),
        'stencila.jsonld'
      ),
      'utf8'
    )
    CONTEXT = JSON.parse(json)['@context']
  }
  return CONTEXT
}

/**
 * Get the URL for a term in the Stencila Schema's JSON-LD `@context`
 * from it's name.
 *
 * This uses the JSON-LD `@context` in `stencila.jsonld` (which
 * provides a mapping between vocabularies) to translate
 * type names used in the Stencila Schema
 * to those used in other schemas (e.g. Schema.org, Bioschemas).
 * The [compact IRIs](https://www.w3.org/TR/json-ld11/#compact-iris)
 * in the `@context` e.g. `schema:Person` are expanded to a URL
 * e.g. `http://schema.org/Person` suitable for the `itemtype` attribute.
 *
 * @param term A term in the JSON-LD `@context`. May, or may not be in
 *             the `stencila` namespace
 */
export function jsonLdTermUrl(term: string): string | undefined {
  if (Object.keys(termToId).length === 0) initMaps()

  const id = termToId[term]
  if (id === undefined) return undefined

  const [prefix, name] = id.split(':')
  const context = jsonLdContext()
  const base = context[prefix]
  if (base === undefined) return undefined

  return `${base}${name}`
}

/**
 * Get the name of a term in the Stencila Schema's JSON-LD `@context`
 * from a URL.
 *
 * This is the inverse of `jsonLdTermUrl`.
 *
 * @param url A url to resolve into a term
 */
export function jsonLdTermName(url: string): string | undefined {
  if (Object.keys(idToTerm).length === 0) initMaps()

  // Parse the url into a base URL / term pair
  const { hash, pathname } = new URL(url)
  const term = hash !== '' ? hash.slice(1) : pathname.split('/').pop()
  const baseUrl = url.replace(new RegExp(`${term}$`), '')

  // Resolve the URL into a prefix
  const prefix = idToTerm[baseUrl]
  if (prefix === undefined) return undefined

  // Resolve the `@id` to the term name
  return idToTerm[`${prefix}:${term}`]
}
