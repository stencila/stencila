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
    .use(stripNewlines)
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

  // See the `remark-stringify` options at https://github.com/wooorm/remark/tree/master/packages/remark-stringify#options
  if (options.gfm !== false) options.gfm = true
  // If commonmark == true remark collapses adjacent blockquotes
  // This is confusing because the remark README says that it will "Compile adjacent blockquotes separately"
  if (!options.commonmark) options.commonmark = false
  if (options.fragment !== false) options.fragment = true
  options.listItemIndent = '1'
  options.strong = '*'
  options.emphasis = '_'
  options.fences = true
  options.rule = '-'
  options.ruleRepetition = 3
  options.ruleSpaces = false
  options.entities = false
  options.encode = false

  const toMarkdown = unified()
    .use(rehypeParse)
    .use(squeezeParagraphs)
    .use(stripNewlines)
    .use(include.html2md)
    .use(rehype2remark)
    .use(squeezeParagraphs)
    .use(stripNewlines)
    .use(remarkStringify)
    .use(include.mdVisitors)

  return toMarkdown.process(html, options).contents.trim()
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}

function stripNewlines () {
  return function (ast) {
    return visit(ast, function (node) {
      if (node.type === 'text' && node.value) {
        node.value = node.value.replace(/(\s?)(\r\n|\n|\r)+\s?/gm, ' ')
      }
    })
  }
};
