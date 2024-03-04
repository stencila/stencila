import { EditorView, KeyBinding } from '@codemirror/view'
import { ExecutableTypeList } from '@stencila/types'

import { CodeMirrorClient } from '../../../clients/codemirror'
import { NodeId } from '../../../types'

const serverActionKeys = (client: CodeMirrorClient): KeyBinding[] => [
  {
    key: 'Ctrl-S',
    run: () => {
      client.sendCommand('save-document')
      return false
    },
  },
  {
    key: 'Ctrl-Shift-Enter',
    run: () => {
      client.sendCommand('execute-document')
      return false
    },
  },
  {
    key: 'Ctrl-Enter',
    run: (view: EditorView) => {
      // TODO: select the ids of nodes that are located at or between
      // selection. Should consider multicursor selections, not just main
      const nodeIds: NodeId[] = []
      const selectedRanges = view.state.selection.ranges
      for (const { from, to } of selectedRanges) {
        const selectedNodeIds = client
          .nodesInRange(from, to, true)
          .filter((node) => ExecutableTypeList.includes(node.nodeType))
          .map((node) => node.nodeId)
        nodeIds.push(...selectedNodeIds)
      }
      client.sendCommand('execute-nodes', nodeIds)
      return false
    },
  },
]

export { serverActionKeys }
