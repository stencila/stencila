import { createContext } from '@lit/context'

import { NodeId } from '../../types'

/**
 * Alternative states for the display of node markers
 *
 *  TODO: When in 'expand-all' state
 *    - edit block functionality to stop the cards from being collapsible
 *    - collapse all the cards again when state changes from 'expand-all' to another
 */
export type NodeMarkerState =
  | 'hidden'
  | 'hover-only'
  | 'show-all'
  | 'expand-all'

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
