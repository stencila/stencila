import { isArray, HTMLImporter, DefaultDOMElement } from 'substance'

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
      converters: configurator.getConverterRegistry().get('html'),
      idAttribute: 'data-id'
    })
  }

  importDocument (html) {
    this.reset()
    var htmlDoc = DefaultDOMElement.parseHTML(html)
    // HACK: this must be fixed in Substance
    // ATM parseHTML is very inconsistent regarding input
    // the best way would be to always return the doc
    if (isArray(htmlDoc)) {
      if (htmlDoc.length > 0) {
        htmlDoc = htmlDoc[0].getOwnerDocument()
      } else {
        htmlDoc = null
      }
    } else {
      htmlDoc = htmlDoc.getOwnerDocument() || htmlDoc
    }

    // creating all nodes
    if (htmlDoc) {
      this.convertDocument(htmlDoc)
    }
    this.generateDocument()
    return this.state.doc
  }

  /**
   * Convert HTML into a Stencila Document
   *
   * This method must be provided when extending HTMLImporter
   */
  convertDocument (el) {
    let content = el.find('.content')
    if (content) this.convertContainer(content.children, 'content')
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
