import { isNumber, isString } from 'substance'
import { TextureDocument } from 'substance-texture'
import { type } from '../value'

export function getCellState(cell) {
  // FIXME: we should make sure that cellState is
  // initialized as early as possible
  return cell.state
}

export function isExpression(source) {
  return /^\s*=/.exec(source)
}

export function getCellValue(cell) {
  if (!cell) return undefined
  if (cell.state) {
    return cell.state.value
  } else {
    let preferredType = getCellType(cell)
    return valueFromText(cell.text(), preferredType)
  }
}

export function getCellType(cell) {
  let type = cell.attr('type')
  if (!type) {
    let doc = cell.getDocument()
    let docType = doc.documentType
    if (docType === 'sheet') {
      let row = cell.parentNode
      let colIdx = row._childNodes.indexOf(cell.id)
      let columnMeta = doc.getColumnMeta(colIdx)
      type = columnMeta.attr('type')
    }
  }
  return type || 'any'
}

export function valueFromText(text, preferredType = 'any') {
  const data = _parseText(preferredType, text)
  const type_ = type(data)
  return { type: type_, data }
}

export function _getSourceElement(cellNode) {
  if (cellNode.getDocument() instanceof TextureDocument) {
    // ATTENTION: this caching would be problematic if the cell element
    // was changed structurally, e.g. the <source-code> element replaced.
    // But we do not do this, so this should be fine.
    if (!cellNode._sourceEl) {
      cellNode._sourceEl = cellNode.find('source-code')
    }
    return cellNode._sourceEl
  }
  return cellNode
}

export function getSource(cellNode) {
  return _getSourceElement(cellNode).textContent
}

export function setSource(cellNode, newSource) {
  let el = _getSourceElement(cellNode)
  el.text(newSource)
}

export function getLang(cellNode) {
  return _getSourceElement(cellNode).getAttribute('language')
}

function _parseText(preferredType, text) {
  // guess value
  if (text === 'false') {
    return false
  } else if (text === 'true') {
    return true
  } else if (!isNaN(text)) {
    let _int = Number.parseInt(text, 10)
    if (_int == text) { // eslint-disable-line
      return _int
    } else {
      return Number.parseFloat(text)
    }
  } else {
    return text
  }
}

export const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

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
  let index = 0
  let rank = 1
  for (let i = 0; i < colStr.length; i++) {
    let letter = colStr[i]
    index += rank * ALPHABET.indexOf(letter)
    rank++
  }
  return index
}

export function getRowCol(cellLabel) {
  var match = /^([A-Z]+)([1-9][0-9]*)$/.exec(cellLabel)
  return [
    parseInt(match[2], 10)-1,
    getColumnIndex(match[1])
  ]
}

export function getError(cell) {
  let cellState = getCellState(cell)
  if (cellState && cellState.errors) {
    return cellState.errors[0]
  }
}

export function getErrorMessage(error) {
  switch(error.name) {
    case 'unresolved': {
      return 'Unresolved inputs: ' + error.details.unresolved.map(s => {
        return s.origStr || s.name
      }).join(', ')
    }
    case 'cyclic': {
      let frags = []
      let trace = error.details.trace
      let symbols = error.details.symbols
      trace.forEach(id => {
        let s = symbols[id]
        if (s) {
          frags.push(s.origStr || s)
        }
      })
      frags.push(frags[0])
      return 'Cyclic Dependency: ' + frags.join(' -> ')
    }
    default:
      return error.message
  }
}

export function getValue(cell) {
  let cellState = getCellState(cell)
  if (cellState) {
    return cellState.value
  }
}

export function getFrameSize(layout) {
  // WORKAROUND, this should be solved in libcore functions
  const defaultSizes = {
    'width': '400',
    'height': '400'
  }
  const sizes = layout.width ? layout : defaultSizes
  return sizes
}

export function getIndexesFromRange(start, end) {
  let [startRow, startCol] = getRowCol(start)
  let endRow, endCol
  if (end) {
    ([endRow, endCol] = getRowCol(end))
    if (startRow > endRow) ([startRow, endRow] = [endRow, startRow])
    if (startCol > endCol) ([startCol, endCol] = [endCol, startCol])
  } else {
    ([endRow, endCol] = [startRow, startCol])
  }
  return { startRow, startCol, endRow, endCol }
}

export function getRangeFromMatrix(cells, startRow, startCol, endRow, endCol, force2D) {
  if (!force2D) {
    if (startRow === endRow && startCol === endCol) {
      let row = cells[startCol]
      if (row) return row[endCol]
      else return undefined
    }
    if (startRow === endRow) {
      let row = cells[startRow]
      if (row) return row.slice(startCol, endCol+1)
      else return []
    }
    if (startCol === endCol) {
      let res = []
      for (let i = startRow; i <= endRow; i++) {
        let row = cells[i]
        if (row) res.push(row[startCol])
      }
      return res
    }
  }
  let res = []
  for (var i = startRow; i < endRow+1; i++) {
    let row = cells[i]
    if (row) res.push(row.slice(startCol, endCol+1))
  }
  return res
}

export function qualifiedId(doc, cell) {
  let cellId = isString(cell) ? cell : cell.id
  if (doc) {
    let docId = isString(doc) ? doc : doc.id
    return `${docId}!${cellId}`
  } else {
    return cellId
  }
}

