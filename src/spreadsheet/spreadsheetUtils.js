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
      endRow = sheet.getRowCount()
    } else if (sel.type === 'rows') {
      startCol = 0
      endRow = sheet.getColumnCount()
    }
    return {
      startRow, endRow, startCol, endCol
    }
  }
