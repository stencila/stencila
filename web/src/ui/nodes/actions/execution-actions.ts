import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

/**
 * A component for providing common execution related actions of executable nodes
 */
@customElement('stencila-ui-node-execution-actions')
@withTwind()
export class UINodeExecutionActions extends LitElement {
  @property({ type: Number })
  value: number

  override render() {
    const containerClasses = apply(['flex flex-row', 'text-base text-black'])
    const dividerClasses = apply(['h-4 w-0', 'border border-black', 'mx-2'])
    return html`
      <div class=${containerClasses}>
        <sl-icon name="deps-tree" library="stencila"></sl-icon>
        <div class=${dividerClasses}></div>
        <sl-icon name="skip-end"></sl-icon>
        <div class=${dividerClasses}></div>
        <sl-icon name="play"></sl-icon>
      </div>
    `
  }
}
