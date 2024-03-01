import { StateEffect, StateField } from '@codemirror/state'
import { Node } from '@stencila/types'

import { NodeId } from '../../../types'

type ExecEffectValue = {
  id: NodeId | 'root'
  node: Node
}

type ExecutableNodes = Record<string, Node>

const executableEffect = StateEffect.define<ExecEffectValue>()

const execuateState = StateField.define<ExecutableNodes>({
  create: () => ({ root: {} }),
  update: (nodes, transaction) => {
    transaction.effects.forEach((e) => {
      if (e.is(executableEffect)) {
        const { id, node } = e.value
        nodes[id] = node
      }
    })
    return nodes
  },
})

export { executableEffect, execuateState, ExecEffectValue }
