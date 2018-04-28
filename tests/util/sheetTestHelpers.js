import { getRowCol } from '../../src/shared/cellHelpers'

export function setSheetSelection(sheetSession, expr) {
  let [start, end] = expr.split(':')
  let [anchorRow, anchorCol] = getRowCol(start)
  let focusRow, focusCol
  if (end) {
    ([focusRow, focusCol] = getRowCol(end))
  } else {
    ([focusRow, focusCol] = [anchorRow, anchorCol])
  }
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