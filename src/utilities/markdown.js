const remark = require('remark')
const remarkHtml = require('remark-html')
const toMarkdown = require('to-markdown')

function md2html (md) {
  return remark()
      .use(remarkHtml)
      .process(md).contents
}

function html2md (html) {
  return toMarkdown(html)
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}
