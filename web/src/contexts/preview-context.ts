import { createContext } from '@lit/context'

/**
 * Different states for the view of the node chips
 *
 *  TODO: When in 'expand-all' state
 *    - edit block functionality to stop the cards from being collapsible
 *    - collapse all the cards again when state changes from 'expand-all' to another
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

  /**
   * Toggles the display of the `<stencila-article>` level author and provenance info
   */
  showAuthorProvenance: boolean
}

/**
 * Context containing values which effect various elements on the
 * webview preview page.
 */
export const documentPreviewContext =
  createContext<DocPreviewContext>('doc-view')
