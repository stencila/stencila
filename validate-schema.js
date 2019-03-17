const path = require('path')

const Ajv = require('ajv')
const fs = require('fs-extra')
const $RefParser = require('json-schema-ref-parser')
const yaml = require('js-yaml')

const ajv = new Ajv()

let file = 'Document.schema.yaml'

// It is necessary for $ref resolution to change into the schema directory
process.chdir('schema')

const schema = yaml.safeLoad(fs.readFileSync(file))
$RefParser.bundle(schema)
  .then(schema => {
    fs.writeJSONSync('../dist/Document.schema.json', schema, {spaces: 2})

    const metaSchema = require('ajv/lib/refs/json-schema-draft-07.json')
    const valid = ajv.validate(metaSchema, schema)
    if (!valid) {
      console.error(`💣  Oh, oh, ${file} is invalid!`)
      for (let error of ajv.errors) {
        console.error(`  - ${error.dataPath} ${error.message}`)
      }
    } else {
      console.error(`🎉  Yay, ${file} is valid!`)
    }
  })
  .catch(err => {
    console.error(err)
  })
