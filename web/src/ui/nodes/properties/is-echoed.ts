import { css } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { patchValue } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

/**
 * A component for the `isEchoed` property of code chunks
 */
@customElement('stencila-ui-node-is-echoed')
@withTwind()
export class UINodeIsEchoed extends UIBaseClass {
  @property({ type: Boolean })
  value?: boolean

  /**
   * On a change to value, send a patch to update it in the document
   */
  private onChanged(event: InputEvent) {
    this.value = (event.target as HTMLInputElement).checked

    this.dispatchEvent(
      patchValue(this.type, this.nodeId, 'isEchoed', this.value ? true : null)
    )
  }

  override render() {
    const { colour, borderColour, textColour } = this.ui

    const switchClass = css`
      & {
        --width: 18px;
        --height: 8px;
        --thumb-size: 12px;
      }

      &::part(control) {
        background-color: ${colour};
        border-color: ${textColour};
      }

      &[checked]::part(control) {
        background-color: ${borderColour};
        border-color: ${textColour};
      }

      &::part(thumb) {
        background-color: ${borderColour};
        border-color: ${textColour};
      }

      &[checked]::part(thumb) {
        background-color: ${textColour};
        border-color: ${textColour};
      }
    `

    return html`<sl-tooltip content="Display code in document">
      <sl-switch
        class="text-xs ${switchClass}"
        size="small"
        name="isEchoed"
        ?checked=${this.value}
        @sl-change=${this.onChanged}
      >
        Echo
      </sl-switch>
    </sl-tooltip> `
  }
}
