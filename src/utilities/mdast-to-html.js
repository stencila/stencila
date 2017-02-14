const unified = require('unified')
const remarkParse = require('remark-parse')
const remarkStringify = require('remark-stringify')
const remarkHtml = require('remark-html')
const squeezeParagraphs = require('remark-squeeze-paragraphs')
const slug = require('remark-slug')

module.exports = function mdast2html (mdast, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true
  options.fences = true

  const html = unified()
    .use(remarkParse)
    .use(squeezeParagraphs)
    .use(slug)
    // include directive not available here
    // .use(include.md2html)
    .use(remarkStringify)
    .use(remarkHtml)

  return html.stringify(mdast, options)
}
