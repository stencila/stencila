import Ajv from 'ajv'
import betterAjvErrors from 'better-ajv-errors'
import fs from 'fs-extra'
import globby from 'globby'
import path from 'path'

/**
 * Check that the `built/*.schema.json` files, generated from `schema/*.schema.yaml` files,
 * are valid JSON Schemas.
 */
test('schemas are valid', async () => {
  const ajv = new Ajv({ jsonPointers: true })
  const metaSchema = require('ajv/lib/refs/json-schema-draft-07.json')
  const validate = ajv.compile(metaSchema)

  const files = await globby(
    path.join(__dirname, '..', 'built', '*.json.schema')
  )
  for (const file of files) {
    const schema = await fs.readJSON(file)
    if (!validate(schema)) {
      const message = betterAjvErrors(metaSchema, schema, validate.errors, {
        format: 'cli',
        indent: 2
      })
      console.log(message)
      throw new Error(`💣  Oh, oh, ${file} is invalid`)
    }
  }
})
