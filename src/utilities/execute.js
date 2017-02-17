const visit = require('unist-util-visit')
const detab = require('detab')
const u = require('unist-builder')
var toString = require('hast-util-to-string')

function html2md () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      if (node.tagName === 'pre' && node.properties && node.properties.dataExecute) {
        const props = node.properties
        const execute = props.dataExecute
        const output = props.dataOutput
        const input = props.dataInput

        delete props.dataExecute
        delete props.dataOutput
        delete props.dataInput

        const keys = Object.keys(props)
        let options

        if (keys.length) {
          options = keys.map(function (key) {
            const attrKey = key.replace('data', '').toLowerCase()
            const attrValue = props[key]
            return `${attrKey}=${attrValue}`
          }).join(',')
        }

        const executeText = output ? `out=${execute}` : execute
        const inputText = input ? `(${input})` : '()'
        const optionsText = options ? `{${options}}` : ''

        const result = `${executeText}${inputText}${optionsText}`
        node.children[0].properties.className = result
      }
    })
  }
}

/*
* Use this to transform markdown code blocks to the  `pre` elements we expect.
*/
function code2preHandler (h, node, parent) {
  let value = node.value ? detab(node.value + '\n') : ''
  let lang = node.lang && node.lang.match(/^[^ \t]+(?=[ \t]|$)/)[0]
  let props = {}
  value = value.trim()

  if (lang) {
    const isExecute = lang.indexOf('(') > -1

    if (isExecute) {
      /* Check for input */
      const inputMatch = lang.match(/\(([^)]+)\)/)
      const input = (inputMatch && inputMatch.length >= 2) ? inputMatch[1] : null
      const statement = lang
      lang = lang.split('(')[0]

      /* Check for options */
      const optionsMatch = statement.match(/\{([^)]+)\}/)
      const optionString = (optionsMatch && optionsMatch.length >= 2) ? optionsMatch[1] : null
      const options = {}

      if (optionString) {
        optionString.split(',').forEach(function (option) {
          const kv = option.split('=')
          options['data-' + kv[0]] = kv[1]
        })
      }

      /* Check for output */
      const hasOutput = lang.indexOf('=') > -1
      let output

      if (hasOutput) {
        const split = lang.split('=')
        output = split[0]
        lang = split[1]
      }

      /* assign properties */
      props['data-execute'] = lang
      if (output) props['data-output'] = output
      if (input) props['data-input'] = input
      if (optionString) props = Object.assign(props, options)
    } else if (lang) {
      props.className = [lang]
    }
  }

  return h(node.position, 'pre', props, [
    h(node, 'code', [u('text', value)])
  ])
}

function code2fenceHandler (h, node) {
  const code = node.children[0]
  const className = code.properties.className || node.properties.className
  return h(node, 'code', {lang: className || null}, toString(node))
}

module.exports = {
  html2md: html2md,
  code2preHandler: code2preHandler,
  code2fenceHandler: code2fenceHandler
}
