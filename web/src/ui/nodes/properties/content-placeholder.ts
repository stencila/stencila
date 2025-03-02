import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'

/**
 * A component for displaying a placeholder for the `content` of
 * a node when it is empty.
 *
 * Useful for executable nodes (e.g. `IncludeBlock`, `IfBlockClause`) that
 * have `content` which may be unintentionally empty, or empty because
 * the node has not been executed yet.
 */
@customElement('stencila-ui-node-content-placeholder')
@withTwind()
export class UINodeContentPlaceholder extends LitElement {
  /**
   * The text for the tooltip
   */
  tooltip: string = 'No content yet'

  override render() {
    // Currently uses a small icon and a tooltip.
    // Previously tried using placeholder text, and using just a dashed
    // border on <p> but both felt too heavy weight in comparison.
    return html`<sl-tooltip content=${this.tooltip}>
      <div class="flex justify-center text-grey-400">
        <stencila-ui-icon name="fullscreen"></stencila-ui-icon>
      </div>
    </sl-tooltip>`
  }
}
