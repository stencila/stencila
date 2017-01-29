const matter = require('gray-matter')
const remark = require('remark')
const remarkHtml = require('remark-html')
//const bracketedSpans = require('remark-bracketed-spans')
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
module.exports = function markdownParser (content, options) {
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true

  const parsed = matter(content)
  const html = remark()
    //.use(bracketedSpans)
    .use(remarkHtml)
    .process(content, options).contents

  return {
    html: html,
    md: parsed.content,
    data: parsed.content
  }
}
