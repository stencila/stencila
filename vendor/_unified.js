var unified = require('unified')
var unistVisit = require('unist-util-visit')
var unistFind = require('unist-util-find')
var unistRemove = require('unist-util-remove')

var remarkParse = require('remark-parse')
var remarkSqueezeParagraphs = require('remark-squeeze-paragraphs')
var remarkBracketedSpans = require('remark-bracketed-spans')
var remarkSlug = require('remark-slug')
var remarkHtml = require('remark-html')
var remark2rehype = require('remark-rehype')
var remarkStringify = require('remark-stringify')

var rehypeParse = require('rehype-parse')
var rehype2remark = require('rehype-remark')
var rehypeStringify = require('rehype-stringify')

global.unifiedHtmlMarkdown = {
  unified: unified,
  unistVisit: unistVisit,
  unistFind: unistFind,
  unistRemove: unistRemove,
  remarkParse: remarkParse,
  remarkSqueezeParagraphs: remarkSqueezeParagraphs,
  remarkBracketedSpans: remarkBracketedSpans,
  remarkSlug: remarkSlug,
  remarkHtml: remarkHtml,
  remarkStringify: remarkStringify,
  rehypeParse: rehypeParse,
  rehypeStringify: rehypeStringify,
  remark2rehype: remark2rehype,
  rehype2remark: rehype2remark
}