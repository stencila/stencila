const url = require('url')
const cheerio = require('cheerio')
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
    doc.content = cheerio.load(content)
  }

  /**
   * Dump a document to HTML
   *
   * @param  {Document} doc Document to dump
   * @param  {Object} options  Any options (see implementations for those available)
   * @returns {String}          Content of the document as HTML
   */
  dump (doc, options) {
    options = options || {}

    // Need to clone content befor modiying
    // Couldn't get this to work with cheerio clone, so doing a (presumably
    // inefficient) dump and then load
    let dom = cheerio.load(doc.content.html())

    // Append `?raw` to all image src URLs that are relative
    // Why? Because otherwise the Stencila Host will serve up the file as a Component page
    // instead of as a raw image.
    dom('img[src]').toArray().forEach(el => {
      let $el = cheerio(el)
      let src = $el.attr('src')
      if (src.substring(0, 5) !== 'data:') {
        let q = url.parse(src).query
        if (q) src += '&raw'
        else src += '?raw'
        $el.attr('src', src)
      }
    })

    let html = dom.html()

    // Beautification. See options at https://github.com/beautify-web/js-beautify/blob/master/js/lib/beautify-html.js
    let beautifulHtml = beautify.html(html, {
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

    return beautifulHtml
  }
}

module.exports = DocumentHtmlConverter
