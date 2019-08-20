/**
 * Generate `built/*.jsonld` files from `schema/*.schema.yaml` files.
 *
 * For custom types (those not defined elsewhere) generates a JSON-LD
 * file similar to those on schema.org e.g. https://schema.org/Person.jsonld
 *
 * For custom properties generates a JSON-LD file similar to
 * those on schema.org e.g. https://schema.org/sibling.jsonld
 */

import fs from 'fs-extra'
import path from 'path'
// @ts-ignore
import fromEntries from 'object.fromentries'
import { read } from './utils'

export const build = async (): Promise<void> => {
  await fs.ensureDir(path.join(__dirname, '..', '..', 'built'))

  const types: { [key: string]: {} } = {}
  const properties: {
    [key: string]: { '@id': string } & { [key: string]: unknown }
  } = {}

  const schemas = await read()
  for (const schema of schemas.values()) {
    const {
      '@id': typeId,
      title,
      properties: typeProperties
    } = schema

    // Skip union types, like `Node` and `BlockContent`, that do not need to
    // be represented in the `@context`.
    if (typeId === undefined || title === undefined || properties === undefined)
      continue

    // Create a schema.org [`Class`](https://meta.schema.org/Class) for
    // types defined by this schema.
    if (typeId.startsWith('stencila:')) {
      const classs = {
        '@id': typeId,
        '@type': 'schema:Class',
        'schema:name': title,
        'schema:description': schema.description
      }

    }

    types[typeId] = { '@id': typeId, name: title }

    // Create a [`Property`](https://meta.schema.org/Property) for those
    // properties that are defined by this schema, otherwise link to the property
    // defined in the other context.
    // TODO: Implement schema:rangeIncludes property - requires the
    // resolving `$refs`. See https://github.com/epoberezkin/ajv/issues/125#issuecomment-408960384
    // for an approach to that.
    if (typeProperties !== undefined) {
      for (const [name, property] of Object.entries(typeProperties)) {
        let pid = property['@id']
        // Do not add terms that are aliases with JSON-LD keywords: @id, @type etc
        if (pid === undefined || name == 'id' || name === 'type' || name === 'value') continue
        // The `schema` property clashes with the schema.org alias. So rename it...
        if (pid === 'stencila:schema') pid = 'stencila:scheme'

        if (pid.startsWith('stencila:')) {
          if (properties[name] === undefined) {
            properties[pid] = {
              '@id': pid,
              '@type': 'schema:Property',
              'schema:name': name,
              'schema:description': property.description,
              'schema:domainIncludes': [{ '@id': typeId }]
            }
          } else {
            const domainIncludes = properties[name]['schema:domainIncludes']
            if (Array.isArray(domainIncludes)) {
              domainIncludes.push({ '@id': typeId })
            }
          }
        } else {
          properties[pid] = {
            '@id': pid,
            name
          }
        }
      }
    }
  }

  /**
   * The main JSON-LD @context.
   *
   * Written to be similar to schema.org's @context:
   * https://schema.org/docs/jsonldcontext.jsonld
   */
  const context = {
    // Alias JSON-LD keywords e.g. `@type` and `@id`
    // For why this is useful, see "Addressing the “@” issue" at
    //    https://datalanguage.com/news/publishing-json-ld-for-developers
    type: '@type',
    id: '@id',
    value: '@value',

    // Other contexts referred to, including this one
    // Note that http vs https is important!
    schema: 'http://schema.org/',
    bioschemas: 'http://bioschemas.org',
    codemeta: 'http://doi.org/10.5063/schema/codemeta-2.0',
    stencila: 'http://schema.stenci.la/',

    // Define that in this context all terms derive from this vocabulary
    // (and so do not need prefixing)
    "@vocab": "http://schema.stenci.la/",

    // Types and properties added in alphabetical order after this e.g
    //   "schema:AudioObject": {"@id": "schema:AudioObject"},
    ...fromEntries([
      ...[...Object.keys(types)].sort(),
      ...[...Object.keys(properties)].sort()
    ].map((id: string) => {
      const term = id.split(':')[1]
      return [term, { '@id': id }]
    }))
  }
  await fs.writeJSON(
    path.join(__dirname, '..', '..', 'built', 'stencila.jsonld'),
    { '@context': context },
    { spaces: 2 }
  )
}

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()
