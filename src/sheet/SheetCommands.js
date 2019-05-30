import { Command } from 'substance'
import { getRange } from './sheetHelpers'
import { insertRows, deleteRows, insertCols, deleteCols, setCellTypes, setColumnTypes } from './SheetManipulations'

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

export class InsertRowsAbove extends RowsCommand {
  execute(params) {
    const editorSession = params.editorSession
    const commandState = params.commandState
    const pos = commandState.startRow
    const count = commandState.nrows
    insertRows(editorSession, pos, count)
  }
}

export class InsertRowsBelow extends RowsCommand {
  execute(params) {
    const editorSession = params.editorSession
    const commandState = params.commandState
    const pos = commandState.endRow + 1
    const count = commandState.nrows
    insertRows(editorSession, pos, count)
  }
}

export class DeleteRows extends RowsCommand {
  execute(params) {
    const editorSession = params.editorSession
    const commandState = params.commandState
    const start = commandState.startRow
    const end = commandState.endRow
    const pos = start
    const count = end - start + 1
    deleteRows(editorSession, pos, count)
  }
}

export class InsertColumnsLeft extends ColsCommand {
  execute(params) {
    const editorSession = params.editorSession
    const commandState = params.commandState
    const pos = commandState.startCol
    const count = commandState.ncolumns
    insertCols(editorSession, pos, count)
  }
}

export class InsertColumnsRight extends ColsCommand {
  execute(params) {
    const editorSession = params.editorSession
    const commandState = params.commandState
    const pos = commandState.endCol + 1
    const count = commandState.ncolumns
    insertCols(editorSession, pos, count)
  }
}

export class DeleteColumns extends ColsCommand {
  execute(params) {
    const editorSession = params.editorSession
    const commandState = params.commandState
    const start = commandState.startCol
    const end = commandState.endCol
    const pos = start
    const count = end - start + 1
    deleteCols(editorSession, pos, count)
  }
}

export class OpenColumnSettings extends ColumnMetaCommand {
  execute(params) {
    // NOTE: when the OpenColumnSettings command is active
    // params.surface is the corresponding SheetComponent
    params.surface.openColumnSettings(params)
    params.editorSession._setDirty('commandStates')
    params.editorSession.performFlow()
  }
}

export class SetLanguageCommand extends Command {

  getCommandState({ selection, editorSession }) {
    if (selection.isNull() || !(selection.isCustomSelection() && selection.customType === 'sheet')) {
      return { disabled: true }
    }
    let doc = editorSession.getDocument()
    const { anchorRow, anchorCol } = selection.data
    if(anchorRow === -1 || anchorCol === -1) {
      return { disabled: true }
    }
    let anchorCell = doc.getCell(anchorRow, anchorCol)
    let language = anchorCell.attr('language')
    let state = {
      cellId: anchorCell.id,
      newLanguage: this.config.language,
      disabled: false,
      active: this.config.language === language
    }
    return state
  }

  execute({ editorSession, commandState }) {
    let { cellId, newLanguage, disabled } = commandState
    if (!disabled) {
      editorSession.transaction((tx) => {
        let cell = tx.get(cellId)
        cell.attr({ language: newLanguage })
      })
    }
  }
}

export class SetTypeCommand extends Command {

  getCommandState({ selection, editorSession }) {
    if (selection.isNull() || !(selection.isCustomSelection() && selection.customType === 'sheet')) {
      return { disabled: true }
    }
    let labelProvider = editorSession.getConfigurator().getLabelProvider()
    let doc = editorSession.getDocument()
    let state = {
      disabled: true
    }
    let { anchorRow, anchorCol } = selection.data
    const selectionType = selection.data.type
    if(selectionType === 'columns') {
      let columnMeta = doc.getColumnMeta(anchorCol)
      let columnType = columnMeta.attr('type') || 'Auto'
      state = {
        cellId: columnMeta.id,
        newType: this.config.type,
        columnType: labelProvider.getLabel(columnType),
        disabled: false,
        active: this.config.type === columnType
      }
    } else {
      if (selectionType === 'rows') anchorCol = 0
      let anchorCell = doc.getCell(anchorRow, anchorCol)
      let columnMeta = doc.getColumnForCell(anchorCell.id)
      let columnType = columnMeta.attr('type') || 'Auto'
      let cellType = anchorCell.attr('type')
      state = {
        cellId: anchorCell.id,
        newType: this.config.type,
        columnType: labelProvider.getLabel(columnType),
        disabled: false,
        active: this.config.type === cellType
      }
    }
    return state
  }

  execute({ editorSession, commandState, selection }) {
    let { newType, disabled } = commandState
    const selectionType = selection.data.type
    if (!disabled) {
      const range = getRange(editorSession)
      if(selectionType === 'range' || selectionType === 'rows') {
        setCellTypes(editorSession, range.startRow, range.startCol, range.endRow, range.endCol, newType)
      } else if (selectionType === 'columns') {
        setColumnTypes(editorSession, range.startCol, range.endCol, newType)
      }
    }
  }
}

export class SelectAllCommand extends Command {

  getCommandState(params) {
    let sel = params.selection
    if (sel.isNull() || !sel.isCustomSelection() || sel.customType !== 'sheet') {
      return { disabled: true }
    }
    return { disabled: false }
  }

  execute(params) {
    const editorSession = params.editorSession
    const sheet = editorSession.getDocument()
    const sel = params.selection
    let selData = {
      type: 'range',
      anchorRow: 0,
      focusRow: sheet.getRowCount() - 1,
      anchorCol: 0,
      focusCol: sheet.getColumnCount() - 1,
      all: true
    }
    editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data: selData,
      surfaceId: sel.surfaceId
    })
  }
}
