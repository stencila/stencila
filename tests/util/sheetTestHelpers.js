import { isArray } from 'substance'
import { getRowCol, getSource } from '../../src/shared/cellHelpers'

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
