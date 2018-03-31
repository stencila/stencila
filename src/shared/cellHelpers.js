import { isNumber, isString } from 'substance'
import { TextureDocument } from 'substance-texture'
import { type } from '../value'
import { parseSymbol, getCellExpressions } from './expressionHelpers'


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
    let sheet = cell.getDocument()
    let type = sheet.getCellType(cell)
    return valueFromText(type, cell.text())
  }
}

export function valueFromText(preferredType, text) {
  const data = _parseText(preferredType, text)
  const type_ = type(data)
  return { type: type_, data }
}

function _getSourceElement(cellNode) {
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

// This is useful for writing tests, to use queries such as 'A1:A10'
export function queryCells(cells, query) {
  let { type, name } = parseSymbol(query)
  switch (type) {
    case 'cell': {
      const [row, col] = getRowCol(name)
      return cells[row][col]
    }
    case 'range': {
      let [anchor, focus] = name.split(':')
      const [startRow, startCol] = getRowCol(anchor)
      const [endRow, endCol] = getRowCol(focus)
      if (startRow === endRow && startCol === endCol) {
        return cells[startCol][endCol]
      }
      if (startRow === endRow) {
        return cells[startRow].slice(startCol, endCol+1)
      }
      if (startCol === endCol) {
        let res = []
        for (let i = startRow; i <= endRow; i++) {
          res.push(cells[i][startCol])
        }
        return res
      } else {
        let res = []
        for (var i = startRow; i < endRow+1; i++) {
          res.push(cells[i].slice(startCol, endCol+1))
        }
        return res
      }
    }
    default:
      throw new Error('Unsupported query')
  }
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

export const BROKEN_REF = '#REF!'

export function transformCellRangeExpressions(source, params) {
  const symbols = getCellExpressions(source)
  for (let i = symbols.length-1; i >= 0; i--) {
    const symbol = symbols[i]
    const transformed = _transformCellRangeExpression(symbol.name, params)
    if(transformed !== symbol.name) {
      let subs
      if (symbol.scope) {
        subs = "'" + symbol.scope + "'" + '!' + transformed
      } else {
        subs = transformed
      }
      source = source.substring(0, symbol.startPos) + subs + source.substring(symbol.endPos)
    }
  }
  return source
}

function _transformCellRangeExpression(expr, { dim, pos, count }) {
  const mode = count > 0 ? 'insert' : 'remove'
  count = Math.abs(count)
  if(!isNumber(pos) || !isNumber(count) || (dim !== 'col' && dim !== 'row')) {
    throw new Error('Illegal arguments')
  }
  let range = _getCellRange(expr, dim)
  const isCellReference = range.start === range.end
  // If operation applied to col/rows after given range we shoudn't modify expression
  if(range.end < pos) {
    return expr
  }
  if(mode === 'insert') {
    if(pos <= range.start) {
      range.start += count
    }
    if(pos <= range.end && !isCellReference) {
      range.end += count
    }
  } else {
    // If it is removing of cell reference or cell range is inside removed range we should return error
    if(isCellReference && range.start === pos && mode === 'remove' || range.start > pos && range.end < pos + count && mode === 'remove') {
      return BROKEN_REF
    }
    const x1 = pos
    const x2 = pos + count
    const start = range.start
    const end = range.end
    if (x2 <= start) {
      range.start -= count
      range.end -= count
    } else {
      if (pos <= start) {
        range.start = start - Math.min(count, start - x1)
      }
      if (pos <= end) {
        range.end = end - Math.min(count, end - x1 + 1)
      }
    }
  }
  return _modifyCellRangeLabel(expr, range, dim)
}

function _getCellRange(expr, dim) {
  if(!expr || !dim) {
    throw new Error('Illegal arguments.')
  }
  const parts = expr.split(':')
  let range = {
    start: 0,
    end: 0
  }
  if(dim === 'col') {
    if(parts.length === 2) {
      range.start = getRowCol(parts[0])[1]
      range.end = getRowCol(parts[1])[1]
    } else {
      range.start = range.end = getRowCol(parts[0])[1]
    }
  } else if (dim === 'row') {
    if(parts.length === 2) {
      range.start = getRowCol(parts[0])[0]
      range.end = getRowCol(parts[1])[0]
    } else {
      range.start = range.end = getRowCol(parts[0])[0]
    }
  } else {
    throw new Error('Illegal dimension: ' + dim)
  }
  return range
}

function _modifyCellRangeLabel(expr, range, dim) {
  if(!expr || !range || !dim) {
    throw new Error('Illegal arguments.')
  }
  const parts = expr.split(':')
  let startRow = getRowCol(parts[0])[0]
  let startCol = getRowCol(parts[0])[1]
  if(parts.length === 1) {
    if(dim === 'col') {
      startCol = range.start
    } else if (dim === 'row') {
      startRow = range.start
    }
    return getCellLabel(startRow, startCol)
  }
  let endRow = getRowCol(parts[1])[0]
  let endCol = getRowCol(parts[1])[1]
  if(dim === 'col') {
    startCol = range.start
    endCol = range.end
  } else if (dim === 'row') {
    startRow = range.start
    endRow = range.end
  }
  return getCellLabel(startRow, startCol)+':'+getCellLabel(endRow, endCol)
}
