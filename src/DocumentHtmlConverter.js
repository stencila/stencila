const beautify = require('js-beautify')

/**
 * HTML converter for the `Document` class
 */
class DocumentHtmlConverter {
  /**
   * Load a document from HTML
   *
   * @param  {Document} doc Document to load
   * @param  {String} content  HTML content
   * @param  {Object} options  Any options (see implementations for those available)
   */
  load (doc, content, options) {
    doc.content = content
  }

  /**
   * Dump a document to HTML
   *
   * @param  {Document} doc Document to dump
   * @param  {Object} options  Any options (see implementations for those available)
   * @returns {String}          Content of the document as HTML
   */
  dump (doc, options) {
    // Beautification. See options at https://github.com/beautify-web/js-beautify/blob/master/js/lib/beautify-html.js
    let html = beautify.html(doc.content, {
      'indent_inner_html': false,
      'indent_size': 2,
      'indent_char': ' ',
      'wrap_line_length': 0, // disable wrapping
      'brace_style': 'expand',
      'preserve_newlines': true,
      'max_preserve_newlines': 5,
      'indent_handlebars': false,
      'extra_liners': ['/html']
    })
    return html
  }
}

module.exports = DocumentHtmlConverter
