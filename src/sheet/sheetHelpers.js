import { DefaultDOMElement as DOM, isString } from 'substance'
export { getCellLabel, getColumnIndex, getRowCol, getColumnLabel } from '../shared/cellHelpers'

export function getSelection(editorSession) {
  let sel = editorSession.getSelection()
  if (sel.isCustomSelection() && sel.customType === 'sheet') {
    return sel.data
  } else {
    return null
  }
}

export function getRange(editorSession) {
  const sel = getSelection(editorSession)
  if (!sel) return null
  const sheet = editorSession.getDocument()
  let startRow = Math.min(sel.anchorRow, sel.focusRow)
  let endRow = Math.max(sel.anchorRow, sel.focusRow)
  let startCol = Math.min(sel.anchorCol, sel.focusCol)
  let endCol = Math.max(sel.anchorCol, sel.focusCol)
  if (sel.type === 'columns') {
    startRow = 0
    endRow = sheet.getRowCount() - 1
  } else if (sel.type === 'rows') {
    startCol = 0
    endCol = sheet.getColumnCount() - 1
  }
  return {
    startRow, endRow, startCol, endCol
  }
}

export const EMPTY_SHEET = `<?xml version="1.0"?>
<!DOCTYPE sheet PUBLIC "StencilaSheet 1.0" "StencilaSheet.dtd">
<sheet>
  <meta>
    <name></name>
    <title></title>
    <description></description>
    <columns>
    </columns>
  </meta>
  <data>
  </data>
</sheet>`

/*
  A generator for Sheet XML that can be configured with a simplified data structure

  @example
  ```
  {
    columns: [{ name: 'x' }, { name: 'y' }, { name: 'z' }],
    cells: [
      ['1', '2', '3'],
      ['4', '5', '6'],
      ['7', '8', '9'],
      ['10', '11', '12']
    ]
  }
  ```
*/
export function createSheetXMLFromSpec(spec) {
  let doc = DOM.parseXML(EMPTY_SHEET)
  const $$ = doc.createElement.bind(doc)
  let ncols
  if (spec.columns) {
    let columns = doc.find('columns')
    spec.columns.forEach(colSpec => {
      const { name, type } = colSpec
      let col = $$('col')
      if (name) col.attr('name', name)
      if (type) col.attr('type', type)
      columns.append(col)
    })
    ncols = spec.columns.length
  }
  if (spec.cells) {
    let data = doc.find('data')
    spec.cells.forEach(rowSpec => {
      if (!ncols) ncols = rowSpec.length
      if (ncols !== rowSpec.length) throw new Error('Illegal number of cells.')
      let row = $$('row')
      rowSpec.forEach(cellSpec => {
        let cell = $$('cell')
        let source, id, type
        if (isString(cellSpec)) {
          source = cellSpec
        } else {
          ({ id, type, source } = cellSpec)
        }
        if (id) cell.attr('id', id)
        if (type) cell.attr('type', type)
        cell.append(source)
        row.append(cell)
      })
      data.append(row)
    })
  }
  if (!spec.columns) {
    let columns = doc.find('columns')
    for (let i = 0; i < ncols; i++) {
      columns.append($$('col').attr('type', 'any'))
    }
  }
  return doc.serialize()
}

export function generateEmptySheetXML(nrows, ncols) {
  let cells = []
  for (let i = 0; i < nrows; i++) {
    let row = []
    for (let j = 0; j < ncols; j++) {
      row.push('')
    }
    cells.push(row)
  }
  return createSheetXMLFromSpec({ cells })
}