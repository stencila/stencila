import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../nodes/code-block'
import '../nodes/code-chunk' // For code chunks that are `echo` (ie. display code)
import '../nodes/image-object'

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
