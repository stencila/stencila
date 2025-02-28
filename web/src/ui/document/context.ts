import { createContext } from '@lit/context'

import { NodeId } from '../../types'

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

/**
 * Context containing a record of whether the start and end of a
 * section delimited by a heading are above (1), within (0), or below (-1) the
 * viewport.
 *
 * Provided by the `<stencila-article>` component and consumed by the
 * `<stencila-link>` component to indicate the sections currently
 * in the viewport.
 */
export type DocumentHeadingsContext = Record<NodeId, [number, number]>

export const documentHeadingsContext = createContext<DocumentHeadingsContext>(
  'document-headings-context'
)
