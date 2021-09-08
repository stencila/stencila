import { CodeChunk, CodeExpression, isA, Parameter } from '@stencila/schema'

type UpdatableNode = CodeChunk | CodeExpression | Parameter | HTMLElement

// TODO: Add component types as a dependency
// type UpdatabaleNodeEl = CodeChunk | CodeExpression | Parameter | HTMLElement

// TODO: Cache DOM elements by key to WeakMap
const getNodeById = (id: string): HTMLElement | null =>
  document.getElementById(id)

export const updateNode =
  (id: string) =>
  (node: UpdatableNode): void => {
    const targetNode = getNodeById(id)

    // TODO: Move this to bottom
    if (node instanceof HTMLElement) {
      targetNode?.replaceWith(node)
    } else if (isA('CodeChunk', node)) {
      const customEvent = new CustomEvent('CodeChunkUpdated', {
        detail: {
          id,
          node,
        },
      })

      document.dispatchEvent(customEvent)
    }
  }
