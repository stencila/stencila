import DocumentModel from '../document/DocumentModel'

import DocumentConfigurator from '../document/DocumentConfigurator'
var documentConfigurator = new DocumentConfigurator()

import DocumentHTMLImporter from '../document/DocumentHTMLImporter'
var documentHTMLImporter = new DocumentHTMLImporter({ configurator: documentConfigurator })

import DocumentHTMLExporter from '../document/DocumentHTMLExporter'
var documentHTMLExporter = new DocumentHTMLExporter({ configurator: documentConfigurator })

import DocumentJSONConverter from '../document/DocumentJSONConverter'
var documentJsonConverter = new DocumentJSONConverter()

/**
 * A factory for creating Stencila component models based on the
 * schema name
 *
 * Used by `./SnapshotEngine`.
 *
 * @class      ModelFactory (name)
 * @param      {<type>}  config  The configuration
 */
class ModelFactory {
  /**
   * Create a new, empty Stencila component from the `schemaName`
   *
   * @param      {string}         schemaName  The schema name
   */
  createDocument (schemaName) {
    if (schemaName === 'stencila-document') {
      return new DocumentModel()
    } else {
      throw new Error('Unhandled schema: ' + schemaName)
    }
  }

  /**
   * Import a Stencila component from HTML to JSON
   */
  importDocument (schemaName, format, content, cb) {
    if (format !== 'html') throw new Error('Unhandled format: ' + format)

    var importer
    var exporter
    if (schemaName === 'stencila-document') {
      importer = documentHTMLImporter
      exporter = documentJsonConverter
    } else {
      throw new Error('Unhandled schema: ' + schemaName)
    }

    // Force importer to create a new document. See https://github.com/substance/substance/issues/765
    importer.createDocument()
    var doc = importer.importDocument(content)
    var data = exporter.exportDocument(doc)
    cb(null, data)
  }

  /**
   * Export a Stencila component frm JSON to HTML
   */
  exportDocument (schemaName, format, content, cb) {
    if (format !== 'html') throw new Error('Unhandled format: ' + format)

    var importer
    var exporter
    if (schemaName === 'stencila-document') {
      importer = documentJsonConverter
      exporter = documentHTMLExporter
    } else {
      throw new Error('Unhandled schema: ' + schemaName)
    }

    var doc = this.createDocument(schemaName)
    importer.importDocument(doc, content)
    var data = exporter.exportDocument(doc)

    // Remove "data-id" attributes
    data = data.replace(/ data-id=".+?"/g, '')

    cb(null, data)
  }
}

export default ModelFactory
