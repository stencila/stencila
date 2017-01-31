const matter = require('gray-matter')
const unified = require('unified')
const remarkParse = require('remark-parse')
const remarkStringify = require('remark-stringify')
const remarkHtml = require('remark-html')
const rehypeParse = require('rehype-parse')
const toMDAST = require('hast-util-to-mdast')

// const bracketedSpans = require('remark-bracketed-spans')
/*

Compatibility with pandoc converter:

include directive
fenced_code_attributes
implicit_figures
definition_lists

markdown_github – enabled by default
backtick_code_blocks – enabled by default

yaml_metadata_block – https://www.npmjs.com/package/gray-matter
bracketed_spans – https://github.com/sethvincent/remark-bracketed-spans
*/

/**
* Markdown parser based on markdown-it
* @param {String} content – the markdown content to parse
* @param {Object} options – options to pass to markdown-it
* @returns {Object} – returns object with `html`, `data`, and `md` properties
**/
function toHTML (content, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true
  options.fences = true

  const parsed = matter(content)
  const html = unified()
    .use(remarkParse, options)
    .use(remarkStringify)
    .use(remarkHtml)
    .process(content, options).contents

  return {
    html: html,
    md: parsed.content,
    data: parsed.content
  }
}

/**
* Markdown parser based on markdown-it
* @param {String} content – the html content to stringify to markdown
* @param {Object} options – options to pass to markdown-it
* @returns {Object} – returns object with `html`, `data`, and `md` properties
**/
function toMarkdown (content, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true
  if (options.fragment !== false) options.fragment = true
  options.fences = true

  const md = unified()
    .use(rehypeParse)
    .use(rehype2remark)
    // .use(bracketedSpans.stringify)
    .use(remarkStringify, options)
    .process(content, options).contents

  return md
}

module.exports = module.exports.toHTML = toHTML
module.exports.toMarkdown = toMarkdown

// temporary function until rehype2remark module exists on npm
function rehype2remark (origin, destination, options) {
  if (destination && !destination.process) {
    options = destination
    destination = null
  }

  return destination ? bridge(destination, options) : mutate(options)
}

/* Bridge-mode.  Runs the destination with the new HAST
 * tree. */
function bridge (destination, options) {
  return function transformer (node, file, next) {
    destination.run(toMDAST(node, options), file, next)
  }
}

/* Mutate-mode.  Further transformers run on the HAST tree. */
function mutate (options) {
  return function transformer (node) {
    return toMDAST(node, options)
  }
}
