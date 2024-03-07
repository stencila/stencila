import { Extension } from '@codemirror/state'
import { ViewPlugin, ViewUpdate } from '@codemirror/view'

import { SourceView } from '../source'

const nodeInfoUpdate = (sourceView: SourceView): Extension =>
  ViewPlugin.fromClass(
    class {
      update = (update: ViewUpdate) => {
        const { view } = update

        const cursor = view.state.selection.main.head

        const currentNodes = sourceView
          .getNodesAt(cursor)
          .filter((node) => !['Text', 'Article'].includes(node.nodeType))

        const primeNode = currentNodes.shift()

        const parentNodes = currentNodes.map((node) => node.nodeId)
        sourceView.dispatchEvent(
          new CustomEvent('stencila-infoview-node', {
            bubbles: true,
            composed: true,
            detail: {
              currentNodeId: primeNode ? primeNode.nodeId : undefined,
              currentParentNodes:
                parentNodes.length > 0 ? parentNodes : undefined,
            },
          })
        )
      }
    }
  )

export { nodeInfoUpdate }
