/**
 * A test based on fixtures in `../__fixtures__`.
 *
 * Tests that any fixture with a name starting with "valid" is valid
 * against the corresponding schema. Tests that the validation errors
 * associated with each file starting with "invalid" are as expected
 * according to snapshots.
 */

import Ajv from 'ajv'
import addFormats from 'ajv-formats'
import fs from 'fs-extra'
import path from 'path'
import { jsonSchemas } from '../util/jsonSchemas'

let ajv: Ajv
beforeAll(async () => {
  const schemas = await jsonSchemas()
  ajv = addFormats(new Ajv({ schemas, strict: false, allErrors: true }))
})

const dir = path.join(__dirname, '..', '__fixtures__')
const fixtures = fs.readdirSync(dir)
test.each(fixtures)('%s', (filename) => {
  const [name, type, ext] = filename.split('.')

  const json = fs.readJSONSync(path.join(dir, filename))

  const validate = ajv.getSchema(type)
  if (validate === undefined) throw new Error(`No schema for type "${type}"`)

  validate(json)

  if (name.startsWith('valid')) {
    expect(validate.errors).toBe(null)
  } else {
    expect(validate.errors).toMatchSnapshot()
  }
})
