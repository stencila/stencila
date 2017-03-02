import { HTMLImporter, DefaultDOMElement } from 'substance'

export default
class SheetHTMLImporter extends HTMLImporter {

  importDocument(html) {
    // initialization
    this.reset()
    // Stencil is providing the content of body, not a full HTML document
    const table = DefaultDOMElement.parseHTML(html)
    const tbody = table.find('tbody')
    const rowEls = tbody.children
    // ATTENTION this is a very optimistic implementation in that regard,
    // that it expects the table to be fully specified (not sparse)
    // having no spanning cells, and expects the first column to contain <th> elements only.
    for (let i = 0; i < rowEls.length; i++) {
      const rowEl = rowEls[i]
      const cellEls = rowEl.children
      for (let j = 1; j < cellEls.length; j++) {
        const cellEl = cellEls[j]
        if (cellEl.textContent) {
          const cell = this.convertElement(cellEl)
          cell.row = i
          cell.col = j-1
        }
      }
    }
    let doc = this.generateDocument()
    return doc
  }
}
