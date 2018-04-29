export function insertRows(editorSession, pos, count) {
  editorSession.transaction((tx) => {
    _createRowsAt(tx.getDocument(), pos, count)
  }, { action: 'insertRows', pos, count })
}

export function insertCols(editorSession, pos, count) {
  editorSession.transaction((tx) => {
    _createColumnsAt(tx.getDocument(), pos, count)
  }, { action: 'insertCols', pos, count })
}

export function deleteRows(editorSession, pos, count) {
  editorSession.transaction((tx) => {
    _deleteRows(tx.getDocument(), pos, pos+count-1)
  }, { action: 'deleteRows', pos, count })
}

export function deleteCols(editorSession, pos, count) {
  editorSession.transaction((tx) => {
    _deleteCols(tx.getDocument(), pos, pos+count-1)
  }, { action: 'deleteCols', pos, count })
}

export function setCell(editorSession, row, col, val) {
  editorSession.transaction(tx => {
    let sheet = tx.getDocument()
    let cell = sheet.getCell(row, col)
    if (cell) {
      cell.textContent = val
      tx.setSelection({
        type: 'custom',
        customType: 'sheet',
        data: {
          type: 'range',
          anchorRow: row,
          anchorCol: col,
          focusRow: row,
          focusCol: col
        }
      })
    }
  }, { action: 'setCell' })
}

export function setValues(editorSession, startRow, startCol, vals) {
  let n = vals.length
  let m = vals[0].length
  editorSession.transaction(tx => {
    let sheet = tx.getDocument()
    _setValues(sheet, startRow, startCol, vals)
    tx.setSelection({
      type: 'custom',
      customType: 'sheet',
      data: {
        type: 'range',
        anchorRow: startRow,
        anchorCol: startCol,
        focusRow: startRow+n-1,
        focusCol: startCol+m-1
      }
    })
  }, { action: 'setValues' })
}

export function clearValues(editorSession, startRow, startCol, endRow, endCol) {
  editorSession.transaction(tx => {
    // Note: the selection remains the same
    _clearValues(tx.getDocument(), startRow, startCol, endRow, endCol)
  })
}

export function setColumnTypes(editorSession, startCol, endCol, type) {
  editorSession.transaction(tx => {
    _setColumnTypes(tx.getDocument(), startCol, endCol, type)
  }, { action: 'setColumnTypes' })
}

export function setCellTypes(editorSession, startRow, startCol, endRow, endCol, type) {
  editorSession.transaction(tx => {
    _setCellTypesForRange(tx.getDocument(), startRow, startCol, endRow, endCol, type)
  }, { action: 'setCellTypes' })
}

function _setValues(sheet, startRow, startCol, vals) {
  for (let i = 0; i < vals.length; i++) {
    let row = vals[i]
    for (let j = 0; j < row.length; j++) {
      let val = row[j]
      let cell = sheet.getCell(startRow+i, startCol+j)
      if (cell) {
        cell.textContent = val
      }
    }
  }
}

function _clearValues(sheet, startRow, startCol, endRow, endCol) {
  for (let rowIdx = startRow; rowIdx <= endRow; rowIdx++) {
    for (let colIdx = startCol; colIdx <= endCol; colIdx++) {
      let cell = sheet.getCell(rowIdx, colIdx)
      cell.textContent = ''
    }
  }
}

function _setCellTypesForRange(sheet, startRow, startCol, endRow, endCol, type) {
  for (let rowIdx = startRow; rowIdx <= endRow; rowIdx++) {
    for (let colIdx = startCol; colIdx <= endCol; colIdx++) {
      let cell = sheet.getCell(rowIdx, colIdx)
      cell.attr({type: type})
    }
  }
}

function _setColumnTypes(sheet, startCol, endCol, type) {
  for (let colIdx = startCol; colIdx <= endCol; colIdx++) {
    let cell = sheet.getColumnMeta(colIdx)
    cell.attr('type', type)
  }
}

function _createRowsAt(sheet, rowIdx, n) {
  let $$ = sheet.createElement.bind(sheet)
  const M = sheet.getColumnCount()
  let data = sheet._getData()
  let rowAfter = data.getChildAt(rowIdx)
  for (let i = 0; i < n; i++) {
    let row = $$('row')
    for (let j = 0; j < M; j++) {
      let cell = $$('cell')
      // TODO: maybe insert default value?
      row.append(cell)
    }
    data.insertBefore(row, rowAfter)
  }
}

function _deleteRows(sheet, startRow, endRow) {
  let data = sheet._getData()
  for (let rowIdx = endRow; rowIdx >= startRow; rowIdx--) {
    let row = data.getChildAt(rowIdx)
    // TODO: add a helper to delete recursively
    row._childNodes.forEach((id) => {
      sheet.delete(id)
    })
    data.removeChild(row)
  }
}

function _deleteCols(sheet, startCol, endCol) {
  let data = sheet._getData()
  let N = sheet.getRowCount()
  let columns = sheet._getColumns()
  for (let colIdx = endCol; colIdx >= startCol; colIdx--) {
    columns.removeAt(colIdx)
  }
  for (let rowIdx = N-1; rowIdx >= 0; rowIdx--) {
    let row = data.getChildAt(rowIdx)
    for (let colIdx = endCol; colIdx >= startCol; colIdx--) {
      const cellId = row.getChildAt(colIdx).id
      row.removeAt(colIdx)
      sheet.delete(cellId)
    }
  }
}

function _createColumnsAt(sheet, colIdx, n) {
  // TODO: we need to add columns' meta, too
  // for each existing row insert new cells
  let $$ = sheet.createElement.bind(sheet)
  let data = sheet._getData()
  let it = data.getChildNodeIterator()
  let columns = sheet._getColumns()
  let colAfter = columns.getChildAt(colIdx)
  for (let j = 0; j < n; j++) {
    let col = $$('col').attr('type', 'any')
    columns.insertBefore(col, colAfter)
  }
  while(it.hasNext()) {
    let row = it.next()
    let cellAfter = row.getChildAt(colIdx)
    for (let j = 0; j < n; j++) {
      let cell = $$('cell')
      row.insertBefore(cell, cellAfter)
    }
  }
}
