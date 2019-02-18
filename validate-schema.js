const path = require('path')

const Ajv = require('ajv')
const fs = require('fs-extra')

const ajv = new Ajv()

const filePath = path.join('schema', 'Environment.schema.json')
const data = fs.readJsonSync(filePath)
const schema = require('ajv/lib/refs/json-schema-draft-07.json')

const valid = ajv.validate(schema, data)
if (!valid) {
  for (let error of ajv.errors) {
    console.error(`${filePath}: ${error.dataPath} ${error.message}. Details: ${JSON.stringify(error)}`)
  }
}
