import { isNumber } from 'substance'
import CellState from '../engine/CellState'
import { type } from '../value'

export const SEVERITY_NAMES = ['info', 'warning', 'error']

export function getMaxSeverity(issues) {
  return issues.reduce((level, issue) => {
    let severity = issue.severity || 0
    return Math.max(level, severity)
  }, 0)
}

export function getCellState(cell) {
  // FIXME: we should make sure that cellState is
  // initialized as early as possible
  return cell.state || new CellState()
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
  switch (preferredType) {
    case 'any': {
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
    case 'integer': {
      return Number.parseInt(text, 10)
    }
    case 'number': {
      return Number.parseFloat(text)
    }
    case 'string': {
      return text
    }
    case 'boolean': {
      if (text) {
        return text !== 'false'
      } else {
        return false
      }
    }
    default: {
      console.warn('FIXME: need to cast to type', preferredType)
      return text
    }
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

export function getRowCol(cellId) {
  var match = /^([A-Z]+)([1-9][0-9]*)$/.exec(cellId)
  return [
    parseInt(match[2], 10)-1,
    getColumnIndex(match[1])
  ]
}

export function getError(cell) {
  let cellState = getCellState(cell)
  return cellState.messages[0]
}
