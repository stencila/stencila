export function getCellState(cell) {
  return cell.state
}

export function isExpression(source) {
  return /^\s*=/.exec(source)
}

export function getCellValue(cell) {
  if (cell.state) {
    return cell.state.value
  } else {
    let sheet = cell.getDocument()
    let type = sheet.getCellType(cell)
    return valueFromText(type, cell.text())
  }
}

export function valueFromText(preferredType, text) {
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

export { getCellLabel } from '../sheet/sheetHelpers'
