var unified = require('unified')
var unistVisit = require('unist-util-visit')
var unistFind = require('unist-util-find')
var unistRemove = require('unist-util-remove')

var remarkParse = require('remark-parse')
var remarkSqueezeParagraphs = require('remark-squeeze-paragraphs')
var remarkBracketedSpans = require('remark-bracketed-spans')
var remarkSlug = require('remark-slug')
var remark2rehype = require('remark-rehype')
var remarkStringify = require('remark-stringify')

var rehypeParse = require('rehype-parse')
var rehype2remark = require('rehype-remark')
var rehypeStringify = require('rehype-stringify')

module.exports = {
  unified,
  visit: unistVisit,
  find: unistFind,
  remove: unistRemove,
  remark: {
    parse: remarkParse,
    squeezeParagraphs: remarkSqueezeParagraphs,
    bracketedSpans: remarkBracketedSpans,
    slug: remarkSlug,
    stringify: remarkStringify
  },
  rehype: {
    parse: rehypeParse,
    stringify: rehypeStringify
  },
  remark2rehype,
  rehype2remark
}