import { Command } from 'substance'
import { InsertNodeCommand } from 'substance-texture'
import { getCellState, qualifiedId } from '../shared/cellHelpers'

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
      editorSession.transaction((tx) => {
        let cell = tx.get(cellId)
        let sourceCode = cell.find('source-code')
        sourceCode.attr({ language: newLanguage })
      })
    }
  }
}

export class ToggleAllCodeCommand extends Command {

  /*
    Always enabled
  */
  getCommandState() {
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

  /*
    Always enabled
  */
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

  /*
    Returns all cell components found in the document
  */
  _getCellComponent(editorSession, cellId) {
    let editor = editorSession.getEditor()
    return editor.find(`.sc-cell[data-id=${cellId}]`)
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

// TODO: what is this for?
export class CodeErrorsCommand extends Command {
  getCommandState({ selection, editorSession }) {
    let doc = editorSession.getDocument()
    // console.log('selection', selection)
    if (selection.isPropertySelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'source-code') {
        let cellNode = node.parentNode
        let cellState = getCellState(cellNode)
        if (cellState.hasErrors()) {
          return {
            disabled: false,
            messages: cellState.errors
          }
        }
      }
    }
    return {
      disabled: true
    }
  }
  execute(params) { } // eslint-disable-line
}


export class InsertCellCommand extends InsertNodeCommand {

  createNode(tx) {
    let cell = tx.createElement('cell')
    cell.append(
      tx.createElement('source-code').attr('language', 'mini'),
      tx.createElement('output').attr('language', 'json')
    )
    return cell
  }

  execute(params, context) {
    var state = params.commandState
    if (state.disabled) return
    let editorSession = this._getEditorSession(params, context)
    editorSession.transaction((tx) => {
      let node = this.createNode(tx, params, context)
      tx.insertBlockNode(node)
      let code = node.find('source-code')
      let sel = tx.selection
      tx.setSelection({
        type: 'property',
        path: code.getPath(),
        startOffset: 0,
        surfaceId: sel.surfaceId,
        containerId: sel.containerId
      })
    })
  }

}

export class RunCellCommand extends Command {

  /*
    Always enabled
  */
  getCommandState({ editorSession, selection }) {
    const doc = editorSession.getDocument()
    if (selection.isPropertySelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'source-code') {
        let cellNode = node.parentNode
        return {
          disabled: false,
          active: false,
          docId: doc.id,
          cellId: cellNode.id
        }
      }
    }
    return {
      disabled: false,
    }
  }

  execute(params, context) {
    const { docId, cellId } = params.commandState
    const engine = context.engine
    const id = qualifiedId(docId, cellId)
    engine._allowRunningCellAndPredecessors(id)
  }
}
