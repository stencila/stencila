import { createContext } from '@lit/context'

/**
 * Different states for the view of the node chips
 */
export type NodeChipState = 'hidden' | 'hover-only' | 'show-all' | 'expand-all'

export type DocPreviewContext = {
  /**
   * Hold the value for the current node-card state
   */
  nodeChipState: NodeChipState

  /**
   * Toggles the visibility of the provenance highlighting
   * on the entire document
   */
  showAllAuthorshipHighlight: boolean
}

/**
 * Context containing values which effect various elements on the
 * webview preview page.
 */
export const documentPreviewContext =
  createContext<DocPreviewContext>('doc-view')
