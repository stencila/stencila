const visit = require('unist-util-visit')
const unified = require('unified')
const parse = require('remark-parse')
const stringify = require('remark-stringify')
const html = require('remark-html')
const findAfter = require('unist-util-find-after')
const remove = require('unist-util-remove')

var toMarkdown = unified()
  .use(parse)
  .use(stringify)

var toHTML = toMarkdown.use(html)

var mdOptions = {
  gfm: true,
  listItemIndent: '1',
  strong: '*',
  emphasis: '_',
  fences: true
}

function md2html () {
  return function (tree) {
    visit(tree, function (node, i, parent) {
      function nextNode (modifiers, index) {
        const currentNode = findAfter(tree, index)
        if (currentNode) {
          getModifiers(modifiers, currentNode, index)
        }
      }

      function getModifiers (modifiers, currentNode, index) {
        const children = currentNode.children

        if (children) {
          let i = 0
          let l = children.length

          for (i; i < l; i++) {
            let child = children[i]

            if (i - index > 1 && child.value.indexOf('&') < 0) {
              break
            }

            if (child && child.value && child.value.indexOf('&') === 0) {
              const sections = child.value.split(' ')
              const modifierType = sections[1]
              let content = ''

              if (modifierType === 'delete' && sections[2]) {
                modifiers.delete.push({
                  selector: sections[2]
                })

                remove(tree, currentNode)
                nextNode(modifiers, index)
              } else if (modifierType === 'change') {
                content += child.value.split(':')[1].trim() + ' '
                child.value = ''

                if (currentNode.children) {
                  currentNode.children.forEach(function (item, i) {
                    if (item.position.indent && item.value) {
                      if (item.value.indexOf('\n') > -1) {
                        item.value = item.value.split('\n').map(function (val) {
                          return val.trim() + '\n'
                        }).join('')
                      }
                      content += item.value.trimLeft()
                    } else if (item.children) {
                      content += toMarkdown.stringify(item, mdOptions) + ' '
                    }
                  })
                }

                content = toHTML.process(content.trim(), mdOptions).contents.trim()

                modifiers.change.push({
                  selector: sections[2].split('\n')[0],
                  content: content
                })

                remove(tree, currentNode)
                nextNode(modifiers, index)
              }
            }
          }
        }
      }

      if (node.type === 'paragraph') {
        if (node.children && node.children[0].value && node.children[0].value.indexOf('<') === 0) {
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

          nextNode(modifiers, i)

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

module.exports = {
  md2html: md2html,
  html2md: html2md
}
