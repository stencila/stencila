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
    .use(include.html2md)
    .use(rehype2remark)
    .use(remarkStringify)
    .use(include.mdVisitors)

  return toMarkdown.process(html, options).contents.trim()
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}
