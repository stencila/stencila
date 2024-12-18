import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { documentCommandEvent } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import '../../buttons/icon'

@customElement('stencila-ui-node-clone-commands')
@withTwind()
export class UINodeCloneCommands extends UIBaseClass {
  @property({ type: Boolean })
  enabled: boolean = true

  private clone() {
    this.dispatchEvent(
      documentCommandEvent({
        command: 'clone-node',
        args: [this.type, this.nodeId],
      })
    )
  }

  override render() {
    if (!this.enabled) {
      return ''
    }

    const classes = apply([
      'flex flex-row items-center flex-shrink-0',
      `text-2xl text-${this.ui.textColour}`,
    ])

    return html`
      <div class=${classes}>
        <sl-tooltip content="Copy into document">
          <stencila-ui-icon-button
            name="boxArrowInLeft"
            @click=${this.clone.bind(this)}
          ></stencila-ui-icon-button>
        </sl-tooltip>
      </div>
    `
  }
}
