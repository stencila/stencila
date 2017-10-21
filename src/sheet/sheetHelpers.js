import { isNumber } from 'substance'

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

export function getColumnLabel(colIdx) {
  if (!isNumber(colIdx)) {
    throw new Error('Illegal argument.')
  }
  var label = ""
  while(true) { // eslint-disable-line
    var mod = colIdx % ALPHABET.length
    colIdx = Math.floor(colIdx/ALPHABET.length)
    label = ALPHABET[mod] + label
    if (colIdx > 0) colIdx--
    else if (colIdx === 0) break
  }
  return label
}

export function getCellLabel(rowIdx, colIdx) {
  let colLabel = getColumnLabel(colIdx)
  let rowLabel = rowIdx + 1
  return colLabel + rowLabel
}

export function getColumnIndex(colStr) {
  var index = 0
  var rank = 1
  for (var i = 0; i < colStr.length; i++) {
    let letter = colStr[i]
    index += rank * ALPHABET.indexOf(letter)
    rank++
  }
  return index
}

export function getRowCol(cellId) {
  var match = /^([A-Z]+)([1-9][0-9]*)$/.exec(cellId)
  return [
    parseInt(match[2], 10)-1,
    getColumnIndex(match[1])
  ]
}

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
