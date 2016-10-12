import DocumentModel from './DocumentModel'
import DocumentJSONConverter from './DocumentJSONConverter'
import DocumentHTMLImporter from './DocumentHTMLImporter'
import DocumentHTMLExporter from './DocumentHTMLExporter'

function importJSON (content) {
  let doc = new DocumentModel()
  let jsonConverter = new DocumentJSONConverter()
  jsonConverter.importDocument(doc, content)
  return doc
}

function exportJSON (doc) {
  let jsonConverter = new DocumentJSONConverter()
  return jsonConverter.exportDocument(doc)
}

function importHTML (content) {
  let htmlImporter = new DocumentHTMLImporter()
  return htmlImporter.importDocument(content)
}

function exportHTML (doc) {
  let htmlExporter = new DocumentHTMLExporter()
  let html = htmlExporter.exportDocument(doc)
  html = html.replace(/ data-id=".+?"/g, '')
  return html
}

export {
  importJSON,
  exportJSON,
  importHTML,
  exportHTML
}
