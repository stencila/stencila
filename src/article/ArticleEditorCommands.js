import { Command } from 'substance'
import { InsertNodeCommand } from 'substance-texture'
import { qualifiedId } from '../shared/cellHelpers'
import { setCellLanguage, insertCell, insertReproFig } from './ArticleManipulations'

export class SetLanguageCommand extends Command {

  getCommandState({ selection, editorSession }) {
    let doc = editorSession.getDocument()
    if (selection.isNodeSelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'cell') {
        let language = node.find('source-code').attr('language')
        return {
          cellId: node.id,
          newLanguage: this.config.language,
          disabled: false,
          active: this.config.language === language
        }
      }
    }
    return { disabled: true }
  }

  execute({ editorSession, commandState }) {
    let { cellId, newLanguage, disabled } = commandState
    if (!disabled) {
      setCellLanguage(editorSession, cellId, newLanguage)
    }
  }
}

export class ToggleAllCodeCommand extends Command {

  getCommandState() {
    // Note: this is always enabled
    return {
      disabled: false,
      active: false
    }
  }

  /*
    Returns all cell components found in the document
  */
  _getCellComponents(params) {
    let editor = params.editorSession.getEditor()
    return editor.findAll('.sc-cell')
  }

  execute(params) {
    let cellComponents = this._getCellComponents(params)
    let sel = params.editorSession.getSelection()
    cellComponents.forEach((cellComponent) => {
      cellComponent.extendState({
        hideCode: this.config.hideCode
      })
    })
    params.editorSession.setSelection(sel)
  }
}


export class HideCellCodeCommand extends Command {

  getCommandState({ selection, editorSession }) {
    let doc = editorSession.getDocument()
    if (selection.isNodeSelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'cell') {
        return {
          cellId: node.id,
          disabled: false
        }
      }
    }
    return { disabled: true }
  }

  execute({ commandState, editorSession }) {
    const { cellId } = commandState
    let editor = editorSession.getEditor()
    let cellComponent = editor.find(`.sc-cell[data-id=${cellId}]`)
    cellComponent.extendState({
      hideCode: true
    })
  }
}


export class ForceCellOutputCommand extends Command {

  getCommandState({ selection, editorSession }) {
    let doc = editorSession.getDocument()
    if (selection.isNodeSelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'cell') {
        // TODO: we should use the node state instead
        let cellComponent = this._getCellComponent(editorSession, nodeId)
        if (cellComponent && cellComponent.state) {
          return {
            cellId: node.id,
            active: Boolean(cellComponent.state.forceOutput),
            disabled: false
          }
        }
      }
    }
    return { disabled: true }
  }

  _getCellComponent(editorSession, cellId) {
    let editor = editorSession.getEditor()
    if (editor) {
      return editor.find(`.sc-cell[data-id=${cellId}]`)
    }
  }

  execute({ commandState, editorSession }) {
    const { cellId } = commandState
    let cellComponent = this._getCellComponent(editorSession, cellId)
    cellComponent.extendState({
      forceOutput: !cellComponent.state.forceOutput
    })
    editorSession.setSelection(null)
  }
}

export class InsertCellCommand extends InsertNodeCommand {

  execute({ editorSession, commandState }) {
    const { disabled } = commandState
    if (!disabled) {
      insertCell(editorSession)
    }
  }
}

export class InsertReproFigCommand extends InsertNodeCommand {

  execute({ commandState, editorSession}) {
    const { disabled } = commandState
    if (!disabled) {
      insertReproFig(editorSession)
    }
  }

}

export class RunCellCommand extends Command {

  getCommandState({ editorSession, selection }) {
    const doc = editorSession.getDocument()
    if (selection.isPropertySelection() || selection.isNodeSelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'source-code') {
        node = node.parentNode
      }
      if (node.type === 'cell') {
        return {
          disabled: false,
          active: false,
          docId: doc.id,
          cellId: node.id
        }
      }
    }
    return {
      disabled: true
    }
  }

  execute(params, context) {
    const { docId, cellId } = params.commandState
    const engine = context.host.engine
    const id = qualifiedId(docId, cellId)
    engine._allowRunningCellAndPredecessors(id)
  }

  static get name() {
    return 'run-cell-code'
  }
}
