/**
 * Generates a JSON-LD `@context` from the JSON-Schema documents in the `schema` directory.
 * 
 * Generates a `@context` similar to https://github.com/codemeta/codemeta/blob/master/codemeta.jsonld
 * but with any custom `Class` or `Property` (see using https://meta.schema.org/) 
 * types expanded using as in, for example, https://schema.org/Organization.jsonld
 */

const path = require('path')

const fs = require('fs-extra')

const context = {
  // Contexts referred to
  schema: 'https://schema.org/',
  codemeta: 'https://doi.org/10.5063/schema/codemeta-2.0',
  stencila: 'https://stencila.github.io/schema/01-draft',

  // Alias `@type` and `@id`
  // See "Addressing the “@” issue" at https://datalanguage.com/news/publishing-json-ld-for-developers
  // for why this is useful.
  type: '@type',
  id: '@id'
}

for (let filename of fs.readdirSync('schema')) {
  const schema = fs.readJsonSync(path.join('schema', filename))
  
  // Create a schema.org [`Class`](https://meta.schema.org/Class) for those
  // classes that are created by this schema, otherwise link to source.
  const cid = schema['@id'] || schema.title
  if (cid.startsWith('stencila:')) {
    context[schema.title] = {
        '@id': cid,
        '@type': 'schema:Class',
        'schema:name': schema.title,
        'schema:description': schema.description
    }
  } else {
    context[schema.title] = {
      '@id': cid
    }
  }

  // Create a [`Property`](https://meta.schema.org/Property)
  // TODO: Implement schema:rangeIncludes property - requires the
  // resolving `$refs`. See https://github.com/epoberezkin/ajv/issues/125#issuecomment-408960384
  // for an approach to that.
  if (schema.properties) {
    for (let [name, property] of Object.entries(schema.properties)) {
      const pid = property['@id'] || name
      if (pid.startsWith('stencila:')) {
        if (!context[name]) {
          context[name] = {
            '@id': pid,
            '@type': 'schema:Property',
            'schema:name': name,
            'schema:description': property.description,
            'schema:domainIncludes': [{ '@id': cid }]
          }
        } else {
          context[name]['schema:domainIncludes'].push({ '@id': cid })
        }
      } else {
        context[name] = {
          '@id': pid
        }
      }
    }
  }
}

const jsonld = {
  '@context': context
}
fs.writeJsonSync('schema.jsonld', jsonld, { spaces: 2 })
