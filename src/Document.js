const DocumentHtmlConverter = require('./DocumentHtmlConverter')
const DocumentMarkdownConverter = require('./DocumentMarkdownConverter')

/**
 * A document
 * The `content` of a document is an HTML string
 * Currently, Github Flavored Markdown is assumed for loading and dumping.
 * @class Document
 */
class Document {
  constructor () {
    this.content = ''
    this.html = ''
    this.md = ''
  }

  /**
  * load content in a specified format
  * @param {[type]} content – content of the document
  * @param {String} format – format of the content
  * @param {Object} options – options that are passed to the converter
  **/
  load (content, format, options) {
    format = format || 'html'
    options = options || {}

    this.converter(format).load(this, content, options)
  }

  /**
  * dump content in a specified format
  * @param {String} format – format of the content
  * @param {Object} options– options that are passed to the converter
  * @returns {[type]} Content of the document
  **/
  dump (format, options) {
    format = format || 'html'
    options = options || {}

    return this.converter(format).dump(this, options)
  }

  /**
   * Get the `Document` converter for a format
   *
   * @param {string} format – The format needing conversion
   * @returns {Converter} A converter object
   */
  converter (format) {
    if (format === 'html') {
      return new DocumentHtmlConverter()
    } else if (format === 'md') {
      return new DocumentMarkdownConverter()
    }
  }
}

module.exports = Document
