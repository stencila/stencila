import { HTMLExporter, forEach } from 'substance'

export default
class SheetDocumentHTMLExporter extends HTMLExporter {

  exportDocument(doc) {
    let sheets = doc.getSheets()
    let fragments = []
    forEach(sheets, (sheet) => {
      let tableEl = this.convertNode(sheet)
      tableEl.setAttribute('name', sheet.getName())
      fragments.push(tableEl.outerHTML)
    })
    return fragments.join('\n')
  }

}
