import { provide } from '@lit/context'
// @ts-expect-error 'we a re using a contrib/ module, so the type declarations are a bit busted'
import renderMathInElement from 'katex/dist/contrib/auto-render.mjs'
import { LitElement } from 'lit'
import { customElement, state } from 'lit/decorators'

import { Entity } from '../nodes/entity'
import {
  DocumentContext,
  documentContext,
  NodeMarkerState,
} from '../ui/document/context'
import { UIBlockOnDemand } from '../ui/nodes/cards/block-on-demand'

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
 * A Wrapper element to provide a context to any stencila Entity components,
 * and listens to events dipatched from the ghost theme based on parameters.
 */
@customElement('stencila-ghost-controller')
export abstract class DocumentView extends LitElement {
  /**
   * List of nodes to expand if the #stencila-expand tag is active on the current post/page
   */
  static expandingNodeTypes = [
    'CodeBlock',
    'CodeChunk',
    'Datatable',
    'Figure',
    'ForBlock',
    'IfBlock',
    'IncludeBlock',
    'InstructionBlock',
    'MathBlock',
    'RawBlock',
    'StyledBlock',
    'Table',
  ]

  @provide({ context: documentContext })
  @state()
  protected context: DocumentContext = {
    showAllAuthorshipHighlight: false,
    nodeMarkerState: 'hidden',
    showAuthorProvenance: false,
  }

  /**
   * Handle the state dispatch event upon document load
   */
  private stateDispatchHandler(e: Event & { detail: StencilaDispatchDetails }) {
    this.context = {
      ...this.context,
      nodeMarkerState: e.detail?.nodeMarkerState ?? 'hidden',
    }

    // if the expand parameter is true, expand all the necessary node cards
    if (e.detail.expandCards) {
      const stencilaNodes = Array.from(document.querySelectorAll('*')).filter(
        (element) => {
          return (
            element.tagName.toLowerCase().startsWith('stencila-') &&
            element instanceof Entity
          )
        }
      )

      stencilaNodes.forEach((el) => {
        const card = el.shadowRoot.querySelector(
          'stencila-ui-block-on-demand, stencila-ui-inline-on-demand'
        ) as UIBlockOnDemand
        if (card && DocumentView.expandingNodeTypes.includes(card.type)) {
          card.openCard()
        }
      })
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
