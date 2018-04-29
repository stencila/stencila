import { isArray } from 'substance'
import { getRowCol, getSource, getIndexesFromRange, getRangeFromMatrix } from '../../src/shared/cellHelpers'
import { parseSymbol } from '../../src/shared/expressionHelpers'

export function setSheetSelection(sheetSession, expr) {
  let { anchorRow, anchorCol, focusRow, focusCol } = _getCoordinatesFromExpr(expr)
  let selData = {
    type: 'range',
    anchorRow, anchorCol, focusRow, focusCol
  }
  sheetSession.setSelection({
    type: 'custom',
    customType: 'sheet',
    data: selData
  })
}

export function checkSelection(t, sel, expr) {
  let expectedSelData = _getCoordinatesFromExpr(expr)
  expectedSelData.type = 'range'
  t.deepEqual(sel.data, expectedSelData, 'selection should be correct')
}

function _getCoordinatesFromExpr(expr) {
  let [start, end] = expr.split(':')
  let [anchorRow, anchorCol] = getRowCol(start)
  let focusRow, focusCol
  if (end) {
    ([focusRow, focusCol] = getRowCol(end))
  } else {
    ([focusRow, focusCol] = [anchorRow, anchorCol])
  }
  return { anchorRow, anchorCol, focusRow, focusCol }
}

export function getSources(cells) {
  return cells.map(rowOrCell => {
    if (isArray(rowOrCell)) {
      return rowOrCell.map(getSource)
    } else {
      return getSource(rowOrCell)
    }
  })
}

// This is useful for writing tests, to use queries such as 'A1:A10'
export function queryCells(cells, query) {
  let symbol = parseSymbol(query)
  switch (symbol.type) {
    case 'cell': {
      const [row, col] = getRowCol(symbol.name)
      return cells[row][col]
    }
    case 'range': {
      const { startRow, startCol, endRow, endCol } = getIndexesFromRange(symbol.anchor, symbol.focus)
      return getRangeFromMatrix(cells, startRow, startCol, endRow, endCol)
    }
    default:
      throw new Error('Unsupported query')
  }
}
