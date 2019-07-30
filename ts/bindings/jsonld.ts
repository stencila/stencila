/**
 * Generate `built/stencila.jsonld` files from `schema/*.schema.yaml` files.
 */

import fs from 'fs-extra'
import path from 'path'
import { read } from '.'

/* eslint-disable @typescript-eslint/strict-boolean-expressions */

export const build = async (): Promise<void> => {
  const schemas = await read()

  const types: { [key: string]: {} } = {}
  const properties: {
    [key: string]: { '@id': string } & { [key: string]: unknown }
  } = {}

  schemas.map(schema => {
    // Create a schema.org [`Class`](https://meta.schema.org/Class) for those
    // classes that are defined by this schema, otherwise link to source.
    const cid = schema['@id']
    if (cid) {
      if (cid.startsWith('stencila:') && schema.title) {
        types[schema.title] = {
          '@id': cid,
          '@type': 'schema:Class',
          'schema:name': schema.title,
          'schema:description': schema.description
        }
      } else if (schema.title) {
        types[schema.title] = {
          '@id': cid
        }
      }
    } else {
      console.error(`Warning: @id is not defined at the top level in ${schema}`)
    }

    // Create a [`Property`](https://meta.schema.org/Property)
    // TODO: Implement schema:rangeIncludes property - requires the
    // resolving `$refs`. See https://github.com/epoberezkin/ajv/issues/125#issuecomment-408960384
    // for an approach to that.
    const typeProperties =
      schema.properties ||
      (schema.allOf && schema.allOf[1] && schema.allOf[1].properties)
    if (typeProperties) {
      for (const [name, property] of Object.entries(typeProperties)) {
        const pid = property['@id']
        if (!pid) continue
        if (pid.startsWith('stencila:')) {
          if (!properties[name]) {
            properties[name] = {
              '@id': pid,
              '@type': 'schema:Property',
              'schema:name': name,
              'schema:description': property.description,
              'schema:domainIncludes': [{ '@id': cid }]
            }
          } else {
            if (properties[name]['@id'] !== pid) {
              throw new Error(
                `Property "${name}" has more than one @id "${
                  properties[name]['@id']
                }" and "${pid}"`
              )
            }
            const domainIncludes = properties[name]['schema:domainIncludes']
            if (Array.isArray(domainIncludes)) {
              domainIncludes.push({ '@id': cid })
            }
          }
        } else {
          properties[name] = {
            '@id': pid
          }
        }
      }
    }
  })

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

  fs.ensureDirSync(path.join(__dirname, '..', '..', 'built'))
  fs.writeJSONSync(
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
