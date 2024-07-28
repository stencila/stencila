import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

@customElement('stencila-ui-suggestion-commands')
@withTwind()
export class UINodeSuggestionCommands extends UIBaseClass {
  /**
   * Emit a custom event to execute the document with this
   * node id and command scope
   */
  private emitEvent(e: Event) {
    e.stopImmediatePropagation()
  }

  protected override render() {
    const containerClasses = apply([
      'flex flex-row gap-x-3 items-center flex-shrink-0',
      `text-${this.ui.textColour}`,
    ])

    return html`
      <div class=${containerClasses}>
        <sl-tooltip content="Accept Suggestion">
          <sl-icon
            name="hand-thumbs-up"
            @click=${(e: Event) => {
              this.emitEvent(e)
            }}
            class="hover:text-gray-900"
          ></sl-icon>
        </sl-tooltip>
        <sl-tooltip content="Reject Suggestion">
          <sl-icon
            name="hand-thumbs-down"
            @click=${(e: Event) => {
              this.emitEvent(e)
            }}
            class="hover:text-gray-900"
          ></sl-icon>
        </sl-tooltip>
        <sl-tooltip content="Revise">
          <sl-icon
            name="arrow-repeat"
            @click=${(e: Event) => {
              this.emitEvent(e)
            }}
            class="hover:text-gray-900"
          ></sl-icon>
        </sl-tooltip>
      </div>
    `
  }
}
