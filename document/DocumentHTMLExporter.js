import HTMLExporter from 'substance/model/HTMLExporter'

/**
 * Exports a Stencila Document to HTML
 *
 * @class      HTMLExporter (name)
 */
class DocumentHTMLExporter extends HTMLExporter {
  constructor (options) {
    super({
      converters: options.configurator.getConverterRegistry().get('html')
    })
  }
  /**
   * Export a Stencila Document to HTML
   */
  exportDocument (doc) {
    var bodyNodes = this.convertContainer(doc.get('content'))
    var wrapper = this.$$('div').append(bodyNodes)
    return wrapper.html()
  }
}

export default DocumentHTMLExporter
