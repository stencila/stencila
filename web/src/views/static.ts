import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { initSiteClient } from '../clients/site'
import { initUno } from '../unocss'

import '../layout/layout' // Site layout shell component
import '../nodes/code-block-static' // For display of code blocks (Prism.js)
import '../nodes/code-chunk-static' // For code chunks that are `echo` (Prism.js)
import '../nodes/image-object-static' // For display of JS-based visualizations (e.g. Mermaid)

initUno()
initSiteClient()

/**
 * Static view of a document
 *
 * A non-interactive, non-dynamic view of the document. Includes only the
 * components needed to render code syntax highlighting and plots.
 */
@customElement('stencila-static-view')
export class StaticView extends LitElement {
    /**
     * Override so that this custom element has a Light DOM to which theme
     * styles are applied.
     */
    protected override createRenderRoot() {
      return this
    }
}
