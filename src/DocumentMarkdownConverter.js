const markdown = require('./markdown')
const toMarkdown = require('to-markdown')

/**
 * Markdown converter for the `Document` class
 *
 * Currently, Github Flavored Markdown is assumed for loading and dumping.
 */
class DocumentMarkdownConverter {
  /**
   * Load a document from Markdown
   *
   * @param  {Document} doc Document to load
   * @param  {[type]} content  Markdown content
   * @param  {[type]} options  Any options (see implementations for those available)
   */
  load (doc, content, options) {
    const md = markdown(content, options)
    doc.content = doc.html = md.html.trim()
    doc.md = content.trim()
    doc.data = md.data
  }

  /**
   * Dump a document to Markdown
   *
   * Leading and trailing whitespace, including newlines, are trimmed
   *
   * @param  {Document} doc Document to dump
   * @param  {Object} options  Any options (see implementations for those available)
   * @returns {String}          Content of the document as Commonmark
   */
  dump (doc, options) {
    const html = doc.content
    const highlightRegEx = /(?:highlight|language)-(\S+)/

    return toMarkdown(html, {
      converters: [{
        filter: function (node) {
          return node.nodeName === 'PRE' &&
          node.firstChild &&
          node.firstChild.nodeName === 'CODE'
        },
        replacement: function (content, node) {
          var firstChild = node.firstChild
          if (firstChild.className && firstChild.className.match(highlightRegEx)[1]) {
            var language = firstChild.className.match(highlightRegEx)[1]
            return '\n\n```' + language + '\n' + node.firstChild.textContent.trim() + '\n```\n\n'
          } else {
            return '\n\n```\n' + node.firstChild.textContent.trim() + '\n```\n\n'
          }
        }
      }]
    })
  }
}

module.exports = DocumentMarkdownConverter
