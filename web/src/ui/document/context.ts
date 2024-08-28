import { createContext } from '@lit/context'

/**
 * Alternative states for the display of node chips
 *
 *  TODO: When in 'expand-all' state
 *    - edit block functionality to stop the cards from being collapsible
 *    - collapse all the cards again when state changes from 'expand-all' to another
 */
export type NodeChipState = 'hidden' | 'hover-only' | 'show-all' | 'expand-all'

/**
 * Context controlling the display of various components within the document
 */
export type DocumentContext = {
  /**
   * The current node chips display state
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

export const documentContext =
  createContext<DocumentContext>('document-context')
