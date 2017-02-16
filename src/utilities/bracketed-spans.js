const visit = require('unist-util-visit')
const remove = require('unist-util-remove')
const bracketedSpans = require('remark-bracketed-spans')

function md2html () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      const data = bracketedSpans.parseMarkdown(node, i, parent, tree)
      if (!data) return
      let trailingText
      let html

      if (data.attr.name) {
        const value = data.text || ''
        const attr = data.attr
        const inputType = attr.type
        const name = attr.name

        delete attr.name
        delete attr.type

        if (inputType === 'select') {
          html = `<select name="${name}">${Object.keys(attr).map((key) => {
            return `<option value="${key}"${value === key ? ` selected="true"` : ''}>${attr[key]}</option>`
          }).join('')}</select>`
        } else {
          const htmlattr = Object.keys(attr).map((key) => {
            return `${key}="${attr[key]}"`
          }).join(' ')

          html = `<input name="${name}"${inputType ? ` type="${inputType}"` : ''}${attr ? ` ${htmlattr}` : ''} value="${value}">`
        }
      } else if (data.attr.value) {
        // return console.log('\n\n\n\n\n', 'has value', data, '\n\n\n\n\n')
      } else {
        html = data.html
        text = data.trailingText
      }

      parent.children[i] = {
        type: 'html',
        value: html
      }

      if (data.trailingText) {
        parent.children[i + 1] = {
          type: 'text',
          value: trailingText
        }
      } else {
        remove(parent, parent.children[i + 1])
      }
    })
  }
}

function createLinkReferences () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      if (node.value && node.value.indexOf('[]') > -1) {
        var value = node.value.split('[]')
        node.value = value[0]

        parent.children.splice(i + 1, 0, {
          type: 'linkReference',
          referenceType: 'shortcut',
          children: []
        })

        parent.children.splice(i + 2, 0, {
          type: 'text',
          value: value[1]
        })
      }
    })
  }
}

module.exports = {
  md2html: md2html,
  html2md: bracketedSpans.html2md,
  parseMarkdown: bracketedSpans.parseMarkdown,
  mdVisitors: bracketedSpans.mdVisitors,
  createLinkReferences: createLinkReferences
}
