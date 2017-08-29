import { Command } from 'substance'

class RowsCommand extends Command {

  getCommandState(params) {
    const sel = params.selection
    if (sel && sel.isCustomSelection() && sel.customType === 'sheet') {
      let data = sel.data
      if (data.type === 'rows') {
        return {
          disabled: false,
          rows: Math.abs(data.anchorRow-data.focusRow)+1
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
        return {
          disabled: false,
          columns: Math.abs(data.anchorCol-data.focusCol)+1
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
    Math.min(sel.anchorRow, sel.focusRow) :
    Math.max(sel.anchorRow, sel.focusRow)+1
  const nrows = commandState.rows
  editorSession.transaction((tx) => {
    tx.getDocument().createRowsAt(refRow, nrows)
  })
}

function insertCols({editorSession, selection, commandState}, mode) {
  const sel = selection.data
  const refCol = mode === 'left' ?
    Math.min(sel.anchorCol, sel.focusCol) :
    Math.max(sel.anchorCol, sel.focusCol)+1
  const ncols = commandState.columns
  editorSession.transaction((tx) => {
    tx.getDocument().createColumnsAt(refCol, ncols)
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
