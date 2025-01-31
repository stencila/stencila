import { provide } from '@lit/context'
import { html, LitElement } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import '../nodes'

import { DocumentContext, documentContext } from '../ui/document/context'

/**
 * Static view of a document
 *
 * A static view of a document which does not update as the document changes,
 * nor allows changes to it.
 */
@customElement('stencila-ghost-view')
export class StaticView extends LitElement {
  @provide({ context: documentContext })
  @state()
  protected context: DocumentContext = {
    showAllAuthorshipHighlight: false,
    nodeMarkerState: 'hover-only',
    showAuthorProvenance: false,
  }

  protected override createRenderRoot() {
    return this
  }

  protected override render(): unknown {
    return html`<slot></slot>`
  }
}
