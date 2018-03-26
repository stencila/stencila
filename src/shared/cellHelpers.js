import { isNumber, isString } from 'substance'
import { type } from '../value'
import { parseSymbol } from './expressionHelpers'


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
  return cellState.errors[0]
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
      const [anchorRow, anchorCol] = getRowCol(anchor)
      const [focusRow, focusCol] = getRowCol(focus)
      if (anchorRow === focusRow && anchorCol === focusCol) {
        return cells[anchorCol][focusCol]
      }
      if (anchorRow === focusRow) {
        return cells[anchorRow].slice(anchorCol, focusCol+1)
      }
      if (anchorCol === focusCol) {
        let res = []
        for (let i = anchorRow; i <= focusRow; i++) {
          res.push(cells[i][anchorCol])
        }
        return res
      }
      throw new Error('Unsupported query')
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

export function transformCellRangeExpression(expr, params) {
  const dim = params.mode
  const idx = params.idx
  const count = Math.abs(params.count)
  const mode = params.count > 0 ? 'insert' : 'remove'
  if(!isNumber(idx) || !isNumber(count) || (dim !== 'col' && dim !== 'row')) {
    throw new Error('Illegal arguments')
  }

  let borders = getCellRangeBorders(expr, dim)
  const isCellReference = borders.start === borders.end

  // If operation applied to col/rows after given borders we shoudn't modify expression
  if(borders.end < idx) {
    return expr
  }

  if(mode === 'insert') {
    if(idx <= borders.start) {
      borders.start += count
    }

    if(idx <= borders.end && !isCellReference) {
      borders.end += count
    }
  } else {
    // If it is removing of cell reference or cell range is inside removed range we should return error
    if(isCellReference && borders.start === idx && mode === 'remove' || borders.start > idx && borders.end < idx + count && mode === 'remove') {
      return BROKEN_REF
    }

    const pos1 = idx
    const pos2 = idx + count
    const start = borders.start
    const end = borders.end
    if (pos2 <= start) {
      borders.start -= count
      borders.end -= count
    } else {
      if (pos1 <= start) {
        borders.start = start - Math.min(pos2-pos1, start-pos1)
      }
      if (pos1 <= end) {
        borders.end = end - Math.min(pos2-pos1, end-pos1)
      }
    }
  }

  // get labels
  return modifyCellRangeLabel(expr, borders, dim)
}

export function getCellRangeBorders(expr, dim) {
  if(!expr || !dim) {
    throw new Error('Illegal arguments.')
  }

  const range = expr.split(':')
  let borders = {
    start: 0,
    end: 0
  }

  if(dim === 'col') {
    if(range.length === 2) {
      borders.start = getRowCol(range[0])[1]
      borders.end = getRowCol(range[1])[1]
    } else {
      borders.start = borders.end = getRowCol(range[0])[1]
    }
  } else if (dim === 'row') {
    if(range.length === 2) {
      borders.start = getRowCol(range[0])[0]
      borders.end = getRowCol(range[1])[0]
    } else {
      borders.start = borders.end = getRowCol(range[0])[0]
    }
  } else {
    throw new Error('Illegal dimension: ' + dim)
  }

  return borders
}

export function modifyCellRangeLabel(expr, borders, dim) {
  if(!expr || !borders || !dim) {
    throw new Error('Illegal arguments.')
  }
  const range = expr.split(':')

  let startRow = getRowCol(range[0])[0]
  let startCol = getRowCol(range[0])[1]

  if(range.length === 1) {
    if(dim === 'col') {
      startCol = borders.start
    } else if (dim === 'row') {
      startRow = borders.start
    } else {
      throw new Error('Illegal dimension: ' + dim)
    }

    return getCellLabel(startRow, startCol)
  }

  let endRow = getRowCol(range[1])[0]
  let endCol = getRowCol(range[1])[1]

  if(dim === 'col') {
    startCol = borders.start
    endCol = borders.end
  } else if (dim === 'row') {
    startRow = borders.start
    endRow = borders.end
  } else {
    throw new Error('Illegal dimension: ' + dim)
  }

  return getCellLabel(startRow, startCol) + ':' + getCellLabel(endRow, endCol)
}
