const visit = require('unist-util-visit')
const remove = require('unist-util-remove')

const mdast2html = require('./mdast-to-html')

function md2html () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      if (node.type === 'paragraph') {
        if (node.children &&
          node.children[0].value &&
          node.children[0].value.indexOf('<') === 0
        ) {
          const splitBySpacesOutsideParentheses = /\s(?=[^)]*(?:\(|$))/g
          let includeStatement = node.children[0].value
          let sections = includeStatement.split(splitBySpacesOutsideParentheses)
          let filepath = sections[1]
          let selectors = []
          let variables

          sections.splice(0, 2)

          sections.forEach(function (section) {
            const variableMatch = section.match(/\(([^)]+)\)/g)
            if (variableMatch) {
              variables = variableMatch[0].replace(/\(|\)/g, '')
            } else {
              selectors.push(section)
            }
          })

          let value = `<div data-include="${filepath}"${selectors.length ? ` data-select="${selectors.join(' ')}"` : ''}${variables ? ` data-input="${variables}"` : ''}>`

          let modifiers = {
            delete: [],
            change: []
          }

          const modifierNodes = parent.children[i + 1]

          if (modifierNodes &&
            modifierNodes.type &&
            modifierNodes.type === 'list'
          ) {
            remove(parent, parent.children[i + 1])
            modifierNodes.children.forEach(function (modifier) {
              modifier.children.forEach(function (child) {
                if (child.type === 'paragraph') {
                  const statement = child.children[0]
                  const sections = statement.value.split(' ')
                  const modifierType = sections[0]

                  if (modifierType === 'delete' && sections[1]) {
                    modifiers.delete.push({
                      selector: sections[1]
                    })
                  } else if (modifierType === 'change') {
                    if (sections.length > 2) {
                      const firstChildSegment = sections.slice(2).join(' ')
                      modifier.children[0].children[0].value = firstChildSegment
                    } else {
                      modifier.children.splice(0, 1)
                    }

                    modifiers.change.push({
                      selector: sections[1],
                      content: mdast2html({
                        type: 'root',
                        children: modifier.children
                      }).trim()
                    })
                  }
                }
              })
            })
          }

          if (modifiers.delete.length) {
            modifiers.delete.forEach(function (modifier) {
              value += `<div data-delete="${modifier.selector}"></div>`
            })
          }

          if (modifiers.change.length) {
            modifiers.change.forEach(function (modifier) {
              value += `<div data-change="${modifier.selector}">${modifier.content}</div>`
            })
          }

          value += '</div>'
          node.type = 'html'
          node.value = value
          delete node.children
        }
      }
    })
  }
}

function html2md () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      if (node.properties && node.properties.dataInclude) {
        const filepath = node.properties.dataInclude
        const selectors = node.properties.dataSelect
        const input = node.properties.dataInput
        let modifiers = {
          type: 'element',
          tagName: 'ul',
          children: []
        }

        node.children.forEach(function (child) {
          if (child.properties && child.properties.dataDelete) {
            modifiers.children.push({
              type: 'element',
              tagName: 'li',
              children: [{
                type: 'text',
                value: 'delete ' + child.properties.dataDelete
              }]
            })
          } else if (child.properties && child.properties.dataChange) {
            const changeModifier = [{
              type: 'element',
              tagName: 'p',
              children: [{
                type: 'text',
                value: 'change ' + child.properties.dataChange
              }]
            }]

            child.children.forEach(function (subchild) {
              changeModifier.push(subchild)
            })

            modifiers.children.push({
              type: 'element',
              tagName: 'li',
              children: changeModifier
            })
          }
        })

        node.tagName = 'p'
        node.children = [{
          type: 'text',
          value: `< ${filepath}${selectors ? ` ${selectors}` : ''}${input ? ` (${input})` : ''}`
        }]

        if (modifiers.children.length) {
          tree.children.splice(i + 1, 0, modifiers)
        }
      }
    })
  }
}

/*
* Clean up markdown output of remark-stringify
*/
function mdVisitors (processor) {
  var Compiler = processor.Compiler
  var visitors = Compiler.prototype.visitors
  var text = visitors.text

  visitors.text = function (node, parent) {
    if (node.value && node.value.indexOf('&lt;')) {
      let result = text.apply(this, arguments).replace('&lt;', '<')
      return result
    }
  }
}

/*
* Override conversion handlers for HTML -> MD
*/
const mdHandlers = {}

module.exports = {
  md2html: md2html,
  html2md: html2md,
  mdVisitors: mdVisitors,
  mdHandlers: mdHandlers
}

/*
function html2md (h, node, parent) {
  if (node.properties.dataInclude) {
    const children = []
    const filepath = node.properties.dataInclude
    const selectors = node.properties.dataSelect
    const input = node.properties.dataInput

    children.push({
      type: 'text',
      value: `< ${filepath}${selectors ? ` ${selectors}` : ''}${input ? ` (${input})` : ''}`
    })

    return h(node, 'paragraph', children)
  }
}
*/
