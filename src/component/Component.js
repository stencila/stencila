const error = require('../error')

/**
 * The abstract base class for all Stencila components
 */
class Component {

  static default (name) {
    return {}[name] || null
  }

  default (name) {
    return this.constructor.default(name)
  }

  /**
   * Get the converter for a format for this component class
   *
   * @param {string} format The format e.g. `'html'`, `'md'`
   */
  static converter (format) {
    throw error('Unhandled format', {format: format})
  }

  /**
   * Get the converter for a format for this component instance
   *
   * @param {string} format The format e.g. `'html'`, `'md'`
   * @return {converter} A component converter
   */
  converter (format) {
    return this.constructor.converter(format)
  }

  /**
  * Load content in a specified format
  *
  * @param {string} content - content of the document
  * @param {string} format - format of the content
  * @param {object} options - options that are passed to the converter
  **/
  load (content, format, options) {
    format = format || this.constructor.default(format)
    options = options || {}

    this.converter(format).load(this, content, options)
  }

  /**
  * dump content in a specified format
  *
  * @param {string} format - format of the content
  * @param {object} options - options that are passed to the converter
  * @returns {converter} - Content of the document
  **/
  dump (format, options) {
    format = format || this.constructor.default(format)
    options = options || {}

    return this.converter(format).dump(this, options)
  }

}

module.exports = Component
