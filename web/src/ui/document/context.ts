import { createContext } from '@lit/context'

/**
 * States for the display of node markers
 */
export type NodeMarkerState = 'hidden' | 'hover-only' | 'show-all'

/**
 * Context controlling the display of various components within the document
 */
export type DocumentContext = {
  /**
   * The current node markers display state
   */
  nodeMarkerState: NodeMarkerState

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
