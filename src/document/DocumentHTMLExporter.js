import { HTMLExporter } from 'substance'
import DocumentModel from './DocumentModel'
import DocumentConfigurator from './DocumentConfigurator'
import wrapSnippet from '../util/wrapSnippet'

/**
 * Exports a Stencila Document to HTML
 *
 * @class      HTMLExporter (name)
 */
class DocumentHTMLExporter extends HTMLExporter {
  constructor () {
    let configurator = new DocumentConfigurator()
    super({
      DocumentClass: DocumentModel,
      schema: configurator.getSchema(),
      converters: configurator.getConverterRegistry().get('html'),
      idAttribute: 'data-id'
    })
  }
  /**
   * Export a Stencila Document to HTML
   */
  exportDocument (doc) {
    var bodyNodes = this.convertContainer(doc.get('content'))
    var wrapper = this.$$('div').append(bodyNodes)
    let html = wrapper.html()
    return wrapSnippet(html)
  }
}

export default DocumentHTMLExporter
