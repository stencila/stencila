import { HTMLExporter } from 'substance'
import Sheet from './Sheet'

export default
class SheetHTMLExporter extends HTMLExporter {

  exportDocument(sheet) {
    var $$ = this.$$
    var ncols = sheet.getColumnCount()
    var nrows = sheet.getRowCount()
    var i,j

    var tableEl = $$('table')
    // create header
    var thead = $$('thead').append('tr')
    tableEl.append(thead)
    var headerRow = thead.firstChild
    headerRow.append($$('th'))
    for (j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(Sheet.static.getColumnName(j)))
    }
    // create
    var tbody = $$('tbody')
    for (i = 0; i < nrows; i++) {
      var rowEl = $$('tr')
      rowEl.append($$('th').text(i+1))
      for (j = 0; j < ncols; j++) {
        var cell = sheet.getCellAt(i, j)
        var cellEl = this.convertCell(cell)
        rowEl.append(cellEl)
      }
      tbody.append(rowEl)
    }
    tableEl.append(tbody)
    return tableEl.outerHTML
  }

  convertCell(cell) {
    var $$ = this.$$
    if (!cell) {
      return $$('td')
    } else {
      return this.convertNode(cell)
    }
  }

}
