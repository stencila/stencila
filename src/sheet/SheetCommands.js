import { Command } from 'substance'

class RowsCommand extends Command {

  getCommandState(params) {
    const sel = params.selection
    if (sel && sel.isCustomSelection() && sel.customType === 'sheet') {
      let data = sel.data
      if (data.type === 'rows') {
        let startRow = Math.min(data.anchorRow, data.focusRow)
        let endRow = Math.max(data.anchorRow, data.focusRow)
        let nrows = endRow-startRow+1
        return {
          disabled: false,
          startRow, endRow, nrows
        }
      }
    }
    // otherwise
    return {
      disabled: true
    }
  }

}

class ColsCommand extends Command {

  getCommandState(params) {
    const sel = params.selection
    if (sel && sel.isCustomSelection() && sel.customType === 'sheet') {
      let data = sel.data
      if (data.type === 'columns') {
        let startCol = Math.min(data.anchorCol, data.focusCol)
        let endCol = Math.max(data.anchorCol, data.focusCol)
        let ncolumns = endCol-startCol+1
        return {
          disabled: false,
          startCol, endCol, ncolumns
        }
      }
    }
    // otherwise
    return {
      disabled: true
    }
  }

}

class ColumnMetaCommand extends Command {

  getCommandState(params) {
    const sel = params.selection
    if (sel && sel.isCustomSelection() && sel.customType === 'sheet') {
      let data = sel.data
      if (data.type === 'columns') {
        let startCol = Math.min(data.anchorCol, data.focusCol)
        let endCol = Math.max(data.anchorCol, data.focusCol)
        let ncolumns = endCol-startCol+1
        if (ncolumns === 1) {
          let colIdx = startCol
          let node = params.surface.getSheet().getColumnMeta(colIdx)
          return {
            disabled: false,
            colIdx, node
          }
        }
      }
    }
    // otherwise
    return {
      disabled: true
    }
  }

}

function insertRows({editorSession, selection, commandState}, mode) {
  const sel = selection.data
  const refRow = mode === 'above' ?
    commandState.startRow :
    commandState.endRow + 1
  const nrows = commandState.nrows
  editorSession.transaction((tx) => {
    tx.getDocument().createRowsAt(refRow, nrows)
  })
}

function insertCols({editorSession, selection, commandState}, mode) {
  const sel = selection.data
  const refCol = mode === 'left' ?
    commandState.startCol :
    commandState.endCol + 1
  const ncols = commandState.ncolumns
  editorSession.transaction((tx) => {
    tx.getDocument().createColumnsAt(refCol, ncols)
  })
}

function deleteRows({editorSession, selection, commandState}) {
  const sel = selection.data
  editorSession.transaction((tx) => {
    tx.getDocument().deleteRows(commandState.startRow, commandState.endRow)
  })
}

function deleteColumns({editorSession, selection, commandState}) {
  const sel = selection.data
  editorSession.transaction((tx) => {
    tx.getDocument().deleteColumns(commandState.startCol, commandState.endCol)
  })
}

export class InsertRowsAbove extends RowsCommand {
  execute(params, context) {
    insertRows(params, 'above')
  }
}

export class InsertRowsBelow extends RowsCommand {
  execute(params, context) {
    insertRows(params, 'below')
  }
}

export class DeleteRows extends RowsCommand {
  execute(params, context) {
    deleteRows(params)
  }
}

export class InsertColumnsLeft extends ColsCommand {
  execute(params, context) {
    insertCols(params, 'left')
  }
}

export class InsertColumnsRight extends ColsCommand {
  execute(params, context) {
    insertCols(params, 'right')
  }
}

export class DeleteColumns extends ColsCommand {
  execute(params, context) {
    deleteColumns(params)
  }
}

export class OpenColumnSettings extends ColumnMetaCommand {
  execute(params, context) {
    // NOTE: when the OpenColumnSettings command is active
    // params.surface is the corresponding SheetComponent
    params.surface.openColumnSettings(params)
  }
}

export class ToggleSheetIssues extends Command {
  getCommandState(params, context) {
    let sheetEditor = context.app.getSheetEditor()
    if (sheetEditor) {
      let linter = sheetEditor.getLinter()
      if (linter.hasIssues()) {
        let severity = linter.hasErrors() ? 'error' : 'warning'
        let numberOfIssues = linter.getNumberOfIssues()
        return {
          disabled: false,
          severity,
          numberOfIssues
        }
      }
    }
    return {
      disabled: true
    }
  }
  execute(params, context) {
    let sheetEditor = context.app.getSheetEditor()
    if (sheetEditor) {
      sheetEditor.toggleConsole('sheet-issues')
    }
  }
}
