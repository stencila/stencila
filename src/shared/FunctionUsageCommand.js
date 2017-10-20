import { Command } from 'substance'
import { getCellState } from './cellHelpers'

export default class FunctionUsageCommand extends Command {
  getCommandState({ selection, editorSession }) {
    let doc = editorSession.getDocument()
    // console.log('selection', selection)
    if (selection.isPropertySelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'source-code') {
        let cellNode = node.parentNode
        let cellState = getCellState(cellNode)
        let cursorPos = selection.start.offset
        let match = this._findFunction(cellState, cursorPos)
        if (match) {
          return {
            disabled: false,
            functionName: match.name,
            paramIndex: match.paramIndex
          }
        }
      }
    }

    return {
      disabled: true
    }
  }

  _findFunction(cellState, cursorPos) {
    let candidate
    cellState.nodes.forEach((node) => {
      if (node.type === 'function' && node.start <= cursorPos && node.end >= cursorPos) {
        let offset = cursorPos - node.start
        if (!candidate || offset < candidate.offset ) {
          // Param index
          let paramIndex
          node.args.forEach((arg, index) => {
            if (arg.start <= cursorPos && arg.end >= cursorPos) {
              paramIndex = index
            }
          })
          candidate = {
            name: node.name,
            offset,
            paramIndex
          }
        }
      }
    })
    return candidate
  }

  execute(params) { } // eslint-disable-line
}
