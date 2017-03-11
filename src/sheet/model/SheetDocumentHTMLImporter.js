import { DefaultDOMElement, HTMLImporter } from 'substance'

export default
class SheetDocumentHTMLImporter extends HTMLImporter {

  importDocument(html) {
    // initialization
    this.reset()

    let doc = DefaultDOMElement.parseHTML(html)
    let tableEls = doc.findAll('table')

    tableEls.forEach((tableEl) => {
      this.convertElement(tableEl)
    })

    return this.generateDocument()
  }

  get allTablesAreSheets() {
    return true
  }
}
