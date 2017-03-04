import visit from 'unist-util-visit'

export function md2html () {
  return function (tree) {
    visit(tree, function (node, i, parent) {})
  }
}

export function html2md () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      if (node.tagName && node.tagName === 'select') {
        var name = node.properties && node.properties.name
        let selected = ''

        const attr = node.children.map(function (child) {
          if (child.properties.selected) {
            selected = child.properties.value
          }

          const value = child.children[0].value.indexOf(' ') > -1
            ? `"${child.children[0].value}"`
            : child.children[0].value

          return `${child.properties.value}=${value}`
        }).join(' ')

        let result = `[${selected}]{name=${name} type=select${attr ? ` ${attr}` : ''}}`
        node.type = 'text'
        node.value = result
      } else if (node.tagName && node.tagName === 'input') {
        const name = node.properties && node.properties.name
        const value = node.properties && node.properties.value
        const type = node.properties && node.properties.type

        delete node.properties.name
        delete node.properties.value
        delete node.properties.type

        const attr = Object.keys(node.properties).map(function (attrKey) {
          const attrValue = node.properties[attrKey].indexOf(' ') > -1
          ? `"${node.properties[attrKey]}"`
          : node.properties[attrKey]

          return `${attrKey}=${attrValue}`
        }).join(' ')

        let result = `[${value}]{name=${name}${type ? ` type=${type}` : ''}${attr ? ` ${attr}` : ''}}`
        node.type = 'text'
        node.value = result
        // An input of type range []{name=var4 type=range min=0 max=100}
      }
    })
  }
}
