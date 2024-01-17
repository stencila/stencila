import { Extension } from '@codemirror/state'
import { showPanel, Panel } from '@codemirror/view'
import { Node } from '@stencila/types'

import { SourceView } from '../views/source'

// import { SourceView } from '../views/source'

// const countWords = (doc: Text) => {
//   let count = 0
//   const iter = doc.iter()
//   while (!iter.next().done) {
//     let inWord = false
//     for (let i = 0; i < iter.value.length; i++) {
//       const word = /\w/.test(iter.value[i])
//       if (word && !inWord) count++
//       inWord = word
//     }
//   }
//   return `Word count: ${count}`
// }

const nodeTreeBreadCrumbs = (nodes: Node[]) => {
  const nodeList = nodes.map((node) => node.type as string)
  while (nodeList[0] === 'Text') {
    nodeList.shift()
  }
  return nodeList.reverse().join(' > ')
}

const nodeTreePanel = (sourceView: SourceView) => (): Panel => {
  const dom = document.createElement('div')
  return {
    dom,
    update() {
      dom.textContent = nodeTreeBreadCrumbs(sourceView.getNodesAt())
    },
  }
}

const bottomPanel = (sourceView: SourceView): Extension => {
  return showPanel.of(nodeTreePanel(sourceView))
}

export { bottomPanel }
