import HTMLImporter from 'substance/model/HTMLImporter'

import DocumentModel from './DocumentModel'
import DocumentConfigurator from './DocumentConfigurator'
import DefaultHTMLConverter from './nodes/default/DefaultHTMLConverter'

/**
 * Imports HTML into a Stencila Document
 *
 * @class      HTMLImporter (name)
 */
class DocumentHTMLImporter extends HTMLImporter {

  constructor () {
    let configurator = new DocumentConfigurator()
    super({
      // Required configuration for an importer
      DocumentClass: DocumentModel,
      schema: configurator.getSchema(),
      converters: configurator.getConverterRegistry().get('html')
    })
  }

  /**
   * Convert HTML into a Stencila Document
   *
   * This method must be provided when extending HTMLImporter
   */
  convertDocument (el) {
    this.convertContainer(el.find('.content').children, 'content')
    this.convertContainer(el.find('.sessions').children, 'sessions')
  }

  /**
   * Method override to provide a default for
   * importing HTML elements not matched by `converters`
   */
  defaultConverter (el, converter) {
    var nodeData = DefaultHTMLConverter.createNodeData()
    DefaultHTMLConverter.import(el, nodeData, converter)
    return nodeData
  }
}

export default DocumentHTMLImporter
