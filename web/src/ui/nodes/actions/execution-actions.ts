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
    const containerClasses = apply([
      'flex flex-row items-center gap-x-4',
      'text-black',
    ])
    const dividerClasses = apply([
      'h-4 w-0',
      'border border-black',
      'mix-blend-multiply opacity-50',
    ])
    return html`
      <div class=${containerClasses}>
        <sl-icon name="deps-tree" library="stencila" class="text-xl"></sl-icon>
        <div class=${dividerClasses}></div>
        <sl-icon name="skip" library="stencila" class="text-2xl"></sl-icon>
        <div class=${dividerClasses}></div>
        <sl-icon name="play" library="stencila" class="text-base"></sl-icon>
      </div>
    `
  }
}
