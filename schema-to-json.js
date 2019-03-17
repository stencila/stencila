
const path = require('path')

const fs = require('fs-extra')
const glob = require('glob')
const yaml = require('js-yaml')

for (let src of glob.sync(path.join('schema', '*.schema.yaml'))) {
  const dest = src.replace('schema/', 'dist/').replace('.yaml', '.json')
  const schema = yaml.safeLoad(fs.readFileSync(src))
  function walk (node) {
    if (typeof node !== 'object') return node
    for (let [key, child] of Object.entries(node)) {
      if (key === '$ref') {
        node[key] = child.replace('.yaml', '.json')
      }
      walk(child)
    }
  }
  walk(schema)
  fs.writeJsonSync(dest, schema, { spaces: 2 })
}
