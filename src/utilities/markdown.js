const unified = require('unified')
const remarkParse = require('remark-parse')
const remarkStringify = require('remark-stringify')
const remarkHtml = require('remark-html')
const rehypeParse = require('rehype-parse')
const rehype2remark = require('rehype-remark')
const squeezeParagraphs = require('remark-squeeze-paragraphs')
const slug = require('remark-slug')
const visit = require('unist-util-visit')

const include = require('./include')

/**
* Convert markdown to html
* @param {String} md – the markdown content to parse
* @param {Object} options – options to pass to markdown-it
* @returns {String} – returns string with html
**/
function md2html (md, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true
  options.fences = true

  const html = unified()
    .use(remarkParse)
    .use(squeezeParagraphs)
    .use(slug)
    .use(include.md2html)
    .use(remarkStringify)
    .use(remarkHtml)
    .process(md, options).contents.trim()

  return html
}

/**
* Convert html to markdown
* @param {String} html – the html content to stringify to markdown
* @param {Object} options – options to pass to markdown-it
* @returns {String} – returns string with markdown
**/
function html2md (html, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  // commonmark collapses blockquotes, making the tests for blockquotes fail if enabled here
  if (!options.commonmark) options.commonmark = false
  if (options.fragment !== false) options.fragment = true
  options.listItemIndent = '1'
  options.strong = '*'
  options.emphasis = '_'
  options.fences = true
  options.entities = false
  options.encode = false

  const toMarkdown = unified()
    .use(rehypeParse)
    .use(function () {
      return function (tree) {
        visit(tree, function (node, index, parent) {
          const parentIndex = tree.children.indexOf(parent)

          if (node.tagName === 'div' &&
          node.properties &&
          node.properties.dataDelete) {
            const selector = node.properties.dataDelete

            tree.children.splice(parentIndex + 1, 0, {
              type: 'element',
              tagName: 'p',
              children: [{
                type: 'text',
                value: `& delete ${selector ? ` ${selector}` : ''}`
              }]
            })
          } else if (node.tagName === 'div' &&
          node.properties &&
          node.properties.dataChange) {
            const selector = node.properties.dataChange

            const children = [{
              type: 'element',
              tagName: 'p',
              children: [
                {
                  type: 'text',
                  value: `& change ${selector ? ` ${selector}` : ''}`
                },
                {
                  type: 'element',
                  tagName: 'br'
                },
                {
                  type: 'text',
                  value: ':    '
                }
              ]
            }]

            node.children.forEach(function (child) {
              child.position.indent = [1]
              if (child.children) {
                child.children.forEach(function (subchild) {
                  subchild.position.indent = [1]
                })
              }
              children.push(child)
            })

            var args = [parentIndex + 2, 0].concat(children)
            Array.prototype.splice.apply(tree.children, args)
          }
        })
      }
    })
    .use(rehype2remark)
    .use(remarkStringify, { commonmark: true })

  return toMarkdown.process(html, options).contents.trim()
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}
