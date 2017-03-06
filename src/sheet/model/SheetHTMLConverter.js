import { getColumnName } from './sheetHelpers'

export default {
  type: 'sheet',
  tagName: 'table',

  // used when this converter is used within a document
  matchElement: function(el, converter) {
    if (converter.allTablesAreSheets) {
      return el.is('table')
    } else {
      return el.is('table[data-type="stencila-sheet"]')
    }
  },

  import: function(el, node, converter) {
    const tbody = el.find('tbody')
    const rowEls = tbody.children
    // ATTENTION this is a very optimistic implementation in that regard,
    // that it expects the table to be complete, not sparse,
    // and the first column being a <th>
    for (let i = 0; i < rowEls.length; i++) {
      const rowEl = rowEls[i]
      const cellEls = rowEl.children
      const row = []
      for (let j = 1; j < cellEls.length; j++) {
        const cellEl = cellEls[j]
        if (cellEl.textContent) {
          const cell = converter.convertElement(cellEl)
          row.push(cell.id)
        }
      }
      node.cells.push(row)
    }
  },

  export: function(sheet, tableEl, converter) {
    const $$ = converter.$$
    const ncols = sheet.getColumnCount()
    const nrows = sheet.getRowCount()
    tableEl.setAttribute('data-type', 'stencila-sheet')

    // Header
    const thead = $$('thead')
    const headerRow = $$('tr')
    // first <th> is just a placeholder for the corner
    headerRow.append($$('th'))
    for (let j = 0; j < ncols; j++) {
      headerRow.append(
        $$('th').text(getColumnName(j))
      )
    }
    thead.append(headerRow)
    tableEl.append(thead)

    // Body
    const tbody = $$('tbody')
    for (let i = 0; i < nrows; i++) {
      const rowEl = $$('tr')
      // first column contains the row label
      rowEl.append($$('th').text(i+1))
      for (let j = 0; j < ncols; j++) {
        const cell = sheet.getCellAt(i, j)
        let cellEl
        if (!cell) {
          cellEl = $$('td')
        } else {
          cellEl = converter.convertNode(cell)
        }
        rowEl.append(cellEl)
      }
      tbody.append(rowEl)
    }
    tableEl.append(tbody)
  }
}
