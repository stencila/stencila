import { provide } from '@lit/context'
// @ts-expect-error 'we a re using a contrib/ module, so the type declarations are a bit busted'
import renderMathInElement from 'katex/dist/contrib/auto-render.mjs'
import { LitElement } from 'lit'
import { customElement, state } from 'lit/decorators'

import {
  DocumentContext,
  documentContext,
  NodeMarkerState,
} from '../ui/document/context'

import 'katex/dist/katex.min.css'
// import '../themes/nodes.css'
import '../nodes'
import '../shoelace'
import '../ui/document/menu'

const STENCILA_STATE_DISPATCH = 'stencila-state-dispatch'

type StencilaDispatchDetails = {
  expandCards?: boolean
  nodeMarkerState?: NodeMarkerState
}

/**
 * A Wrapper element to provide a context to any stencila Entity components
 */
@customElement('stencila-ghost-controller')
export abstract class DocumentView extends LitElement {
  @provide({ context: documentContext })
  @state()
  protected context: DocumentContext = {
    showAllAuthorshipHighlight: false,
    nodeMarkerState: 'hidden',
    showAuthorProvenance: false,
  }

  private stateDispatchHandler(e: Event & { detail: StencilaDispatchDetails }) {
    this.context = {
      ...this.context,
      nodeMarkerState: e.detail?.nodeMarkerState ?? 'hidden',
    }
  }

  protected override createRenderRoot(): HTMLElement | DocumentFragment {
    window.addEventListener(
      STENCILA_STATE_DISPATCH,
      this.stateDispatchHandler.bind(this)
    )

    // window.addEventListener()

    return this
  }
}

/**
 * render the katex after dom content loaded
 */
document.addEventListener('DOMContentLoaded', () => {
  // intialise katex auto render
  renderMathInElement(document.body, {
    delimiters: [
      { left: '$$', right: '$$', display: true },
      { left: '$', right: '$', display: false },
      { left: '\\(', right: '\\)', display: false },
      { left: '\\[', right: '\\]', display: true },
    ],
  })
})
