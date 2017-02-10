const unified = require('unified')
const remarkParse = require('remark-parse')
const remarkStringify = require('remark-stringify')
const remarkHtml = require('remark-html')
const rehypeParse = require('rehype-parse')
const rehype2remark = require('rehype-remark')
const visit = require('unist-util-visit')
const squeezeParagraphs = require('remark-squeeze-paragraphs')
var slug = require('remark-slug')

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
    .use(remarkParse, options)
    .use(squeezeParagraphs)
    .use(stripParagraphNewlines)
    .use(slug)
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

  const md = unified()
    .use(rehypeParse)
    .use(rehype2remark)
    .use(remarkStringify)
    .process(html, options).contents.trim()

  return md
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}

function stripParagraphNewlines () {
  return function (ast) {
    return visit(ast, function (node) {
      if (node.type === 'paragraph') {
        node.children.forEach(function (child) {
          if (child.type === 'text' && child.value) {
            child.value = child.value.replace(/(\s?)(\r\n|\n|\r)+\s?/gm, ' ')
          }
        })
      }
    })
  }
};
