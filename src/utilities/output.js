const visit = require('unist-util-visit')

function md2html () {
  return function (tree) {
    visit(tree, function (node, i, parent) {})
  }
}

function html2md () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      if (node.tagName && node.tagName === 'output') {
        const value = node.children && node.children[0] && node.children[0].value || ''
        const htmlFor = node.properties && node.properties.htmlFor

        delete node.properties.value
        delete node.properties.htmlFor

        const attr = Object.keys(node.properties).map(function (attrKey) {
          const attrValue = node.properties[attrKey]
          attrKey = attrKey.replace('data', '').toLowerCase()
          return `${attrKey}="${attrValue}"`
        }).join(' ')

        let result = `[${value}]{value=${htmlFor}${attr ? ` ${attr}` : ''}}`
        node.type = 'text'
        node.value = result
      }
    })
  }
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}
