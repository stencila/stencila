const Markdown = require('markdown-it')
const matter = require('gray-matter')
const implicitFigures = require('markdown-it-implicit-figures')
const deflist = require('markdown-it-deflist')

/*

Compatibility with pandoc converter:

include directive – not yet available
bracketed_spans – not yet available
fenced_code_attributes – not yet available

markdown_github – enabled by default
backtick_code_blocks – enabled by default

yaml_metadata_block – https://www.npmjs.com/package/gray-matter
implicit_figures – https://github.com/arve0/markdown-it-implicit-figures
definition_lists – https://github.com/markdown-it/markdown-it-deflist
*/

/**
* Markdown parser based on markdown-it
* @param {String} content – the markdown content to parse
* @param {Object} options – options to pass to markdown-it
* @returns {Object} – returns object with `html`, `data`, and `md` properties
**/
module.exports = function markdownParser (content, options) {
  const parsed = matter(content)

  const md = new Markdown('commonmark', options)
    .use(implicitFigures)
    .use(deflist)
    .disable('code')

  return {
    html: md.render(parsed.content),
    md: parsed.content,
    data: parsed.content
  }
}
