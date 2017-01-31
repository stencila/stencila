const markdown = require('./markdown')
const toMarkdown = markdown.toMarkdown

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
    options = options || {}

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
    options = options || {}

    const html = doc.content
    return toMarkdown(html, options)
  }
}

module.exports = DocumentMarkdownConverter
