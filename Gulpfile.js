/* eslint-disable @typescript-eslint/no-var-requires, @typescript-eslint/strict-boolean-expressions */

const { src, parallel, series, watch } = require('gulp')
const Ajv = require('ajv')
const betterAjvErrors = require('better-ajv-errors')
const fs = require('fs-extra')
const globby = require('globby')
const jls = require('vscode-json-languageservice')
const jstt = require('json-schema-to-typescript')
const path = require('path')
const through2 = require('through2')
const yaml = require('js-yaml')

/**
 * Process a schema object.
 *
 * This function processes hand written YAML or JSON schema definitions with the aim
 * of making schema authoring less tedious and error prone. Including the following modifications:
 *
 * - `$schema`: set to `http://json-schema.org/draft-07/schema#` if not specified
 * - `$id`: set to `https://stencila.github.com/schema/${schema.title}.schema.json` if not specified
 * - `properties.*.from`: set to `schema.title`
 * - `properties.type.enum[0]`: set to `schema.title`
 * - `additionalProperties`: set to `false` if not specified
 * - `category`: set to the category (subdirectory) the schema belongs to
 * - `children`: a list of child schemas
 * - `descendants`: a list of all descendant schemas
 * - `source`: the location of the source file for the schema definition
 *
 * If the schema defines the `$extends` keyword then in addition:
 *
 * - `parent`: set to the title of the parent schema (we don't use `extends` because that affects Typescript generation)
 * - `properties`: are merged from ancestors and the current schema
 * - `required`: are merged from ancestors and the current schema
 */
function processSchema(schemas, aliases, schema) {
  if (schema.$processed) return
  try {
    if (!schema.$schema)
      schema.$schema = `http://json-schema.org/draft-07/schema#`
    if (!schema.$id)
      schema.$id = `https://stencila.github.com/schema/${
        schema.title
      }.schema.json`

    // Don't modify any other schema
    if (!schema.$id.includes('stencila')) return

    schema.category = path.dirname(schema.source)
    schema.children = []
    schema.descendants = []

    if (schema.properties) {
      schema.type = 'object'

      const typesAliases = {}
      for (const [name, property] of Object.entries(schema.properties)) {
        schema.properties[name].from = schema.title

        // Registered declared aliases
        if (property.aliases) {
          for (const alias of property.aliases) typesAliases[alias] = name
        }
        // Add and register aliases for array properties
        if (property.type === 'array' && name.endsWith('s')) {
          const alias = name.slice(0, -1)
          if (!property.aliases) property.aliases = []
          if (!property.aliases.includes(alias)) property.aliases.push(alias)
          typesAliases[alias] = name
        }
      }
      if (Object.keys(typesAliases).length) {
        aliases[schema.title] = typesAliases
        schema.aliases = true
      }

      if (schema.additionalProperties === undefined) {
        schema.additionalProperties = false
      }
    }

    if (schema.$extends) {
      const parent = schema => {
        if (!schema.$extends) return null
        const parentPath = path.join(
          path.dirname(schema.source),
          schema.$extends
        )
        const parent = schemas.get(parentPath)
        if (!parent)
          throw new Error(`Schema in "$extends" not found for: "${parentPath}"`)
        return parent
      }

      // Get the base schema and ensure that it
      // has been processed (to collect properties)
      const base = parent(schema)
      processSchema(schemas, aliases, base)

      // Do extension of properties from base
      if (base.properties)
        schema.properties = { ...base.properties, ...(schema.properties || {}) }
      if (base.required)
        schema.required = [...base.required, ...(schema.required || [])]

      // For all ancestors add this schema to the descendants list
      schema.parent = base.title
      base.children.push(schema.title)
      let ancestor = base
      while (ancestor) {
        ancestor.descendants.push(schema.title)
        ancestor = parent(ancestor)
      }
    }

    if (path.extname(schema.source) === '.yaml') {
      // Replace any `$ref`s to YAML with a ref to the JSON generated in this function
      const walk = node => {
        if (typeof node !== 'object') return
        for (const [key, child] of Object.entries(node)) {
          if (key === '$ref' && typeof child === 'string')
            node[key] = path.basename(child).replace('.yaml', '.json')
          walk(child)
        }
      }
      walk(schema)
    }

    schema.$processed = true
  } catch (error) {
    throw new Error(
      `Error when processing "${schema.source}": "${error.stack}"`
    )
  }
}

/**
 * Generate `built/*.schema.json` files from `schema/*.schema.{yaml,json}` files.
 *
 * This function does not use Gulp file streams because it needs to load all the schemas
 * into memory at once. It does
 */
async function jsonschema() {
  // Asynchronously read all the schema definition files into a map of objects
  const filePaths = await globby('schema/**/*.schema.{yaml,json}')
  const schemas = new Map(
    await Promise.all(
      filePaths.map(async filePath => {
        const source = path.relative('schema', filePath)
        const schema = yaml.safeLoad(await fs.readFile(filePath))
        return [source, { ...schema, source }]
      })
    )
  )

  // Process each of the schemas collecting aliases along the way
  const aliases = {}
  for (const schema of schemas.values()) processSchema(schemas, aliases, schema)

  // Do final processing and write schema objects to file
  await fs.ensureDir('built')
  for (const schema of schemas.values()) {
    // Generate the destination path from the source and then
    // rewrite source so that it can be use for a "Edit this schema" link in docs.
    const destPath = path.join(
      'built',
      path.basename(schema.source).replace('.yaml', '.json')
    )
    schema.source = `https://github.com/stencila/schema/blob/master/schema/${
      schema.source
    }`

    // Create a final `properties.type.enum` (all `Thing`s should have this) based on `$descendants`
    if (
      schema.$id.includes('stencila') &&
      schema.properties &&
      schema.properties.type
    ) {
      if (schema.descendants) {
        schema.properties.type.enum = [
          schema.title,
          ...schema.descendants.sort()
        ]
      } else {
        schema.properties.type.enum = [schema.title]
      }
      schema.properties.type.default = schema.title
    }

    // Remove unnecessary processing keywords
    delete schema.$extends
    delete schema.$processed

    await fs.writeJSON(destPath, schema, { spaces: 2 })
  }

  // Output `aliases.json`
  await fs.writeJSON(path.join('built', 'aliases.json'), aliases, { spaces: 2 })

  // Output `types.schema.json`
  // This 'meta' schema provides a list of type schemas as:
  //  - an entry point for the generation of Typescript type definitions
  //  - a lookup for all types for use in `util.ts` functions
  const properties = {}
  const required = []
  for (const schema of schemas.values()) {
    if (
      !(schema.title && schema.$id && schema.$id.startsWith('https://stencila'))
    )
      continue
    properties[schema.title] = {
      allOf: [{ $ref: `${schema.title}.schema.json` }]
    }
    required.push(schema.title)
  }
  const types = {
    $schema: 'http://json-schema.org/draft-07/schema#',
    title: 'Types',
    properties,
    required
  }
  await fs.writeJSON('built/types.schema.json', types, { spaces: 2 })

  // Copy the built files into `dist` for publishing package
  await fs.ensureDir('dist')
  await Promise.all(
    (await globby('built/**/*')).map(async file =>
      fs.copy(file, path.join('dist', file))
    )
  )
}

/**
 * Generate `built/stencila.jsonld` from the `built/*.schema.json` files
 *
 * Generates a `@context` similar to https://github.com/codemeta/codemeta/blob/master/codemeta.jsonld
 * but with any extension class or properties defined using `Class` or `Property` (see using https://meta.schema.org/).
 */
function jsonld() {
  const types = {}
  const properties = {}

  // Process each JSON file
  return (
    src('built/*.schema.json')
      .pipe(
        through2.obj((file, enc, cb) => {
          const schema = JSON.parse(file.contents)

          // Create a schema.org [`Class`](https://meta.schema.org/Class) for those
          // classes that are defined by this schema, otherwise link to source.
          const cid = schema['@id']
          if (cid) {
            if (cid.startsWith('stencila:')) {
              types[schema.title] = {
                '@id': cid,
                '@type': 'schema:Class',
                'schema:name': schema.title,
                'schema:description': schema.description
              }
            } else {
              types[schema.title] = {
                '@id': cid
              }
            }
          } else {
            console.error(
              `Warning: @id is not defined at the top level in ${file.path}`
            )
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
                  properties[name]['schema:domainIncludes'].push({ '@id': cid })
                }
              } else {
                properties[name] = {
                  '@id': pid
                }
              }
            }
          }

          cb(null, file)
        })
      )
      // Sigh. To make this a writeable stream so the 'end' works
      // https://github.com/gulpjs/gulp/issues/1637#issuecomment-216958503
      .on('data', () => {})
      // Now write the context
      .on('end', () => {
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
        ])
          jsonld[key] = value

        fs.ensureDirSync('built')
        fs.writeJSONSync(path.join('built', 'stencila.jsonld'), jsonld, {
          spaces: 2
        })
      })
  )
}

/**
 * Generate `types.ts` from `built/types.schema.json`
 */
async function ts() {
  const src = 'built/types.schema.json'
  const dest = 'types.ts'
  const options = {
    bannerComment: `/* eslint-disable */
/**
 * This file was automatically generated.
 * Do not modify it by hand. Instead, modify the source \`.schema.yaml\` file
 * in the \`schema\` directory and run \`npm run build\` to regenerate this file.
 */
 `
  }
  const ts = await jstt.compileFromFile(src, options)
  return fs.writeFile(dest, ts)
}

/**
 * Check the generated JSON Schemas in `built/types.schema.json` are valid.
 */
async function check() {
  const schema = await fs.readJSON('built/types.schema.json')
  const metaSchema = require('ajv/lib/refs/json-schema-draft-07.json')
  const ajv = new Ajv({ jsonPointers: true })
  const validate = ajv.compile(metaSchema)
  if (!validate(schema)) {
    const message = betterAjvErrors(metaSchema, schema, validate.errors, {
      format: 'cli',
      indent: 2
    })
    console.log(message)
    throw new Error('💣  Oh, oh, the schema is invalid')
  }
}

/**
 * Test that the examples are valid
 */
function test() {
  const ajv = new Ajv({
    jsonPointers: true,
    allErrors: true,
    loadSchema: uri => {
      const match = uri.match(/https:\/\/stencila.github.com\/schema\/(.+)$/)
      if (!match) throw new Error(`Not able to get schema from URI "${uri}"`)
      return fs.readJSON(path.join('built', match[1]))
    }
  })

  return src('examples/**/*.{yaml,json}').pipe(
    through2.obj((file, enc, cb) => {
      ;(async function() {
        const example = yaml.safeLoad(file.contents)
        if (!example) throw new Error(`Example seems to be empty: ${file.path}`)

        const type = example.type
        if (!type)
          throw new Error(
            `Example does not have a "type" property: ${file.path}`
          )
        let validator = ajv.getSchema(
          `https://stencila.github.com/schema/${type}.schema.json`
        )
        if (!validator) {
          const schema = await fs.readJSON(
            path.join('built', `${type}.schema.json`)
          )
          validator = await ajv.compileAsync(schema)
        }

        const valid = validator(example)
        const relativePath = path.relative(process.cwd(), file.path)
        if (file.basename.includes('invalid')) {
          if (valid) {
            throw new Error(
              `? Woops. "${relativePath}" is supposed to be invalid, but it's not.`
            )
          } else {
            const contents = file.contents.toString()

            // All validation errors should have a corresponding comment in the file
            // Previously we used:
            //    const errors = betterAjvErrors(schema, example, validator.errors, { format: 'json'})
            // to generate the list of errors. However despite the use of `allErrors: true` above, Ajv
            // does not always return all errors (because it is optimimised for speed?). So instead, here we
            // use the vscode JSON language service to generate a list of errors.
            const json = JSON.stringify(yaml.safeLoad(contents))
            const textDoc = jls.TextDocument.create(
              'foo://bar/file.json',
              'json',
              0,
              json
            )
            const jsonDoc = jls
              .getLanguageService({})
              .parseJSONDocument(textDoc)
            const errors = jsonDoc.validate(textDoc, validator.schema)

            for (const error of errors) {
              if (!contents.includes(error.message)) {
                throw new Error(
                  `💣  Oh, oh, "${relativePath}" is expected to contain the comment "${
                    error.message
                  }".`
                )
              }
            }
          }
        } else {
          if (!valid) {
            const message = betterAjvErrors(
              validator.schema,
              example,
              validator.errors,
              {
                format: 'cli',
                indent: 2
              }
            )
            console.log(message)
            throw new Error(`💣  Oh, oh, "${relativePath}" is invalid`)
          }
        }
      })()
        .then(() => cb(null))
        .catch(error => cb(error))
    })
  )
}

/**
 * Clean up!
 */
function clean() {
  return Promise.all(['dist', 'built', 'types.ts'].map(dir => fs.remove(dir)))
}

exports.jsonschema = jsonschema
exports.check = series(jsonschema, check)
exports.jsonld = series(jsonschema, jsonld)
exports.ts = series(jsonschema, ts)
exports.test = series(jsonschema, test)
exports.build = series(
  clean,
  jsonschema,
  parallel(check, test),
  parallel(jsonld, ts)
)
exports.watch = () =>
  watch(['schema', 'examples'], { ignoreInitial: false }, exports.build)
exports.clean = clean
