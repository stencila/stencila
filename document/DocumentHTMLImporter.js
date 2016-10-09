import HTMLImporter from 'substance/model/HTMLImporter'

import DocumentModel from './DocumentModel'
import DefaultHTMLConverter from './nodes/default/DefaultHTMLConverter'

/**
 * Imports HTML into a Stencila Document
 *
 * @class      HTMLImporter (name)
 */
class DocumentHTMLImporter extends HTMLImporter {

  constructor (options) {
    super({
      // Required configuration for an importer
      DocumentClass: DocumentModel,
      schema: options.configurator.getSchema(),
      converters: options.configurator.getConverterRegistry().get('html')
    })
  }

  /**
   * Convert HTML into a Stencila Document
   *
   * This method must be provided when extending HTMLImporter
   */
  convertDocument (els) {
    // The `containerId` argument should have the same
    // value as the `containerId` used by `ContainerEditor`
    this.convertContainer(els, 'content')
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
