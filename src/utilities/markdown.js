import unified from 'unified'
import remarkParse from 'remark-parse'
import remarkStringify from 'remark-stringify'
import remarkHtml from 'remark-html'
import rehypeParse from 'rehype-parse'
import rehype2remark from 'rehype-remark'
import squeezeParagraphs from 'remark-squeeze-paragraphs'
import slug from 'remark-slug'
import visit from 'unist-util-visit'

import * as bracketedSpans from './bracketed-spans'
import * as input from './input'
import * as output from './output'
import * as execute from './execute'
import * as include from './include'

/**
* Convert markdown to html
* @param {String} md – the markdown content to parse
* @param {Object} options – options to pass to markdown-it
* @returns {String} – returns string with html
**/
export function md2html (md, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true
  options.fences = true

  const handlers = {
    code: execute.code2preHandler
  }

  const html = unified()
    .use(remarkParse)
    .use(squeezeParagraphs)
    .use(stripNewlines)
    .use(slug)
    .use(include.md2html)
    .use(bracketedSpans.createLinkReferences)
    .use(remarkStringify)
    .use(bracketedSpans.md2html)
    .use(remarkHtml, { handlers: handlers })
    .process(md, options).contents.trim()

  return html
}

/**
* Convert html to markdown
* @param {String} html – the html content to stringify to markdown
* @param {Object} options – options to pass to markdown-it
* @returns {String} – returns string with markdown
**/
export function html2md (html, options) {
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

  const handlers = {
    pre: execute.code2fenceHandler
  }

  const toMarkdown = unified()
    .use(rehypeParse)
    .use(squeezeParagraphs)
    .use(stripNewlines)
    .use(bracketedSpans.html2md)
    .use(input.html2md)
    .use(output.html2md)
    .use(include.html2md)
    .use(execute.html2md)
    .use(rehype2remark, {handlers: handlers})
    .use(squeezeParagraphs)
    .use(stripNewlines)
    .use(remarkStringify)
    .use(include.mdVisitors)
    .use(bracketedSpans.mdVisitors)

  return toMarkdown.process(html, options).contents.trim()
}

function stripNewlines () {
  return function (ast) {
    return visit(ast, function (node) {
      if (node.type === 'text' && node.value) {
        node.value = node.value.replace(/(\s?)(\r\n|\n|\r)+\s?/gm, ' ')
      }
    })
  }
}
