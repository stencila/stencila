const matter = require('gray-matter')
const unified = require('unified')
const remarkParse = require('remark-parse')
const remarkStringify = require('remark-stringify')
const remarkHtml = require('remark-html')
const rehypeParse = require('rehype-parse')
const toMDAST = require('hast-util-to-mdast')
const visit = require('unist-util-visit')
const squeezeParagraphs = require('remark-squeeze-paragraphs')

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

  const parser = unified()
    .use(remarkParse, options)
    .use(squeezeParagraphs)
    .use(stripParagraphNewlines)
    .use(remarkStringify)

  const data = matter(content).data
  const md = parser.process(content, options).contents
  const html = parser.use(remarkHtml).process(content, options).contents

  return {
    html: html,
    md: md,
    data: data
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

  /* Bridge-mode.  Runs the destination with the new MDAST
   * tree. */
  function bridge (destination, options) {
    return function transformer (node, file, next) {
      destination.run(toMDAST(node, options), file, next)
    }
  }

  /* Mutate-mode.  Further transformers run on the MDAST tree. */
  function mutate (options) {
    return function transformer (node) {
      return toMDAST(node, options)
    }
  }

  return destination ? bridge(destination, options) : mutate(options)
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
