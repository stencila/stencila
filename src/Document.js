var DocumentMarkdownConverter = require('./DocumentMarkdownConverter')

/**
 * A document
 * The `content` of a document is an HTML string
 * Currently, Github Flavored Markdown is assumed for loading and dumping.
 * @class Document
 */
class Document {
  constructor (options) {
    this.content = ''
  }

  /**
  * load content in a specified format
  * @param {String} format
  * @param {String} content
  * @param {Object} options
  **/
  load (format, content, options) {
    this.converter(format).load(this, content, options)
  }

  /**
  * dump content in a specified format
  * @param {String} format
  * @param {Object} options
  * @return {String} Content of the document
  **/
  dump (format, options) {
    return this.converter(format).dump(this, options)
  }

  /**
   * Get the `Document` converter for a format
   *
   * @param {string} format The format needing conversion
   * @return {Converter} A converter object
   */
  converter (format) {
    if (format === 'md') {
      return new DocumentMarkdownConverter()
    }
  }
}

module.exports = Document
