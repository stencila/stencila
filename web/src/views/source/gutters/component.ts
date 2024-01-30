import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { NodeType } from '../../../types'
import { TWLitElement } from '../../../ui/twind'

import nodeGutterMarkers from './markers'

@customElement('stencila-gutter-marker')
class StencilaGutterMarker extends TWLitElement {
  @property({ type: Boolean })
  isFirstLine: boolean = false

  @property({ type: Boolean })
  isLastLine: boolean = false

  @property({ type: Array })
  nodes: NodeType[]

  @property({ type: Number })
  lineHeight: number

  render() {
    return html`
      <div
        class="relative"
        style="width: ${this.lineHeight}px; height: ${this.lineHeight}px"
      >
        ${!this.isLastLine ? this.renderBase() : ''}
        ${this.isFirstLine ? this.renderIcon() : ''}
        ${this.isLastLine && !this.isFirstLine ? this.renderEnd() : ''}
      </div>
    `
  }

  renderIcon(zIndex?: number) {
    return html`
      <img
        src=${nodeGutterMarkers[this.nodes[0]].icon}
        class="absolute top-0 left-0"
        width="100%"
        height="100%"
        style="z-index=${zIndex ?? 10};"
      />
    `
  }

  renderBase() {
    return html`<div
      class="h-full w-1/2"
      style="background-color: ${nodeGutterMarkers[this.nodes[0]].colour};"
    ></div>`
  }

  renderEnd() {
    return html`<div
      class="h-full w-1/2 rounded-[]"
      style="background-color: ${nodeGutterMarkers[this.nodes[0]]
        .colour}; border-radius: 0 0 25px 25px;"
    ></div>`
  }
}

export { StencilaGutterMarker }
