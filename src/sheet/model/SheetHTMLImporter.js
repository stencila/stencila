import { HTMLImporter, DefaultDOMElement } from 'substance'
import Sheet from './Sheet'
import converters from './SheetConverters'

export default
class SheetHTMLImporter extends HTMLImporter {

  importDocument(html) {
    // initialization
    this.reset()
    // Stencil is providing the content of body, not a full HTML document
    var table = DefaultDOMElement.parseHTML(html)

    var tbody = table.find('tbody')
    var rowEls = tbody.children
    // ATTENTION this is a very optimistic implementation in that regard,
    // that it expects the table to be fully specified (not sparse)
    // having no spanning cells, and expects the first column to contain <th> elements only.
    for (var i = 0; i < rowEls.length; i++) {
      var rowEl = rowEls[i]
      var cellEls = rowEl.children
      for (var j = 1; j < cellEls.length; j++) {
        var cellEl = cellEls[j]
        if (cellEl.text()) {
          var cell = this.convertElement(cellEl)
          cell.row = i
          cell.col = j-1
        }
      }
    }
    var doc = this.generateDocument()
    return doc
  }

}
