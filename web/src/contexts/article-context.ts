import { createContext } from '@lit/context'

export type DocViewContext = {
  /**
   * Whether the 'chip' for each node is always visible,
   * or only visible when user hovers on node.
   */
  showAllToggleChips: boolean
}

/**
 * Context containing values which effect various elements on the
 * webview preview page.
 */
export const documentViewContext = createContext<DocViewContext>('doc-view')
