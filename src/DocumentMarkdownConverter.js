const marked = require('marked')
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
    doc.content = marked(content, options)
    return doc
  }

  /**
   * Dump a document to Markdown
   *
   * Leading and trailing whitespace, including newlines, are trimmed
   *
   * @param  {Document} doc Document to dump
   * @param  {Object} options  Any options (see implementations for those available)
   * @return {String}          Content of the document as Commonmark
   */
  dump (doc, options) {
    var html = doc.content
    return toMarkdown(html)
  }
}

module.exports = DocumentMarkdownConverter
