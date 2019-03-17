const path = require('path')

const Ajv = require('ajv')
const betterAjvErrors = require('better-ajv-errors')
const $RefParser = require('json-schema-ref-parser')
const fs = require('fs-extra')
const yaml = require('js-yaml')

const ajv = new Ajv({ jsonPointers: true })

var args = process.argv.slice(2)
const file = args[0] ? args[0] : 'schema/Document.schema.yaml'
const data = yaml.safeLoad(fs.readFileSync(file))

const schema = fs.readJsonSync('schema/Document.schema.json')

const validate = ajv.compile(schema)
if (!validate(data)) {
  console.error(`💣  Oh, oh, ${file} is invalid!`)
  const output = betterAjvErrors(schema, data, validate.errors, {
    format: 'cli',
    indent: 2
  })
  console.error(output)
} else {
  console.log(`🎉  Yay, ${file} is valid!`)
}
