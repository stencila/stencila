import { Extension } from '@codemirror/state'
import { ViewPlugin, ViewUpdate } from '@codemirror/view'

import { SourceView } from '../source'

const nodeInfoUpdate = (sourceView: SourceView): Extension =>
  ViewPlugin.fromClass(
    class {
      update = (update: ViewUpdate) => {
        const { view } = update

        const cursor = view.state.selection.main.head

        const currentNode = sourceView
          .getNodesAt(cursor)
          .filter((node) => !['Text', 'Article'].includes(node.nodeType))[0]

        sourceView.dispatchEvent(
          new CustomEvent('stencila-infoview-node', {
            bubbles: true,
            composed: true,
            detail: {
              currentNodeId: currentNode ? currentNode.nodeId : undefined,
            },
          })
        )
      }
    }
  )

export { nodeInfoUpdate }
