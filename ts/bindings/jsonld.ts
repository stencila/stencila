/**
 * Generate `built/*.jsonld` files from `schema/*.schema.yaml` files.
 *
 * For custom types (those not defined elsewhere) generates a JSON-LD
 * file similar to e.g. https://schema.org/Person.jsonld
 *
 * For custom properties generates a JSON-LD file similar to
 * e.g. https://schema.org/sibling.jsonld
 */

import fs from 'fs-extra'
import path from 'path'
import { read } from './utils'

export const build = async (): Promise<void> => {
  const types: { [key: string]: {} } = {}
  const properties: {
    [key: string]: { '@id': string } & { [key: string]: unknown }
  } = {}

  const schemas = await read()
  for (const schema of schemas.values()) {
    const { '@id': typeId, title, properties: typeProperties } = schema

    // Skip union types like `Node` and `BlockContent` that do not need to
    // be represented here.
    if (typeId === undefined || title === undefined || properties === undefined)
      continue

    // Create a schema.org [`Class`](https://meta.schema.org/Class) for those
    // types that are defined by this schema, otherwise link to the class defined
    // in the other context.
    if (typeId.startsWith('stencila:')) {
      types[title] = {
        '@id': typeId,
        '@type': 'schema:Class',
        'schema:name': title,
        'schema:description': schema.description
      }
    } else {
      types[title] = {
        '@id': typeId
      }
    }

    // Create a [`Property`](https://meta.schema.org/Property) for those
    // properties that are defined by this schema, otherwise link to the property
    // defined in the other context.
    // TODO: Implement schema:rangeIncludes property - requires the
    // resolving `$refs`. See https://github.com/epoberezkin/ajv/issues/125#issuecomment-408960384
    // for an approach to that.
    if (typeProperties !== undefined) {
      for (const [name, property] of Object.entries(typeProperties)) {
        const pid = property['@id']
        if (pid === undefined) continue

        if (pid.startsWith('stencila:')) {
          if (properties[name] === undefined) {
            properties[name] = {
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
          properties[name] = {
            '@id': pid
          }
        }
      }
    }
  }

  const jsonld = {
    '@context': {
      // Contexts referred to, including this one
      schema: 'https://schema.org/',
      bioschemas: 'http://bioschemas.org',
      codemeta: 'https://doi.org/10.5063/schema/codemeta-2.0',
      stencila: 'https://stencila.github.io/schema/01-draft',

      // Alias `@type` and `@id`
      // See "Addressing the “@” issue" at https://datalanguage.com/news/publishing-json-ld-for-developers
      // for why this is useful.
      type: '@type',
      id: '@id'
    }
  }

  // Add types and properties alphabetically
  for (const [key, value] of [
    ...[...Object.entries(types)].sort(),
    ...[...Object.entries(properties)].sort()
  ]) {
    // @ts-ignore
    jsonld[key] = value
  }

  await fs.ensureDir(path.join(__dirname, '..', '..', 'built'))
  await fs.writeJSON(
    path.join(__dirname, '..', '..', 'built', 'stencila.jsonld'),
    jsonld,
    {
      spaces: 2
    }
  )
}

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()
