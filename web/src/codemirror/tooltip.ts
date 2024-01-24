import { hoverTooltip } from '@codemirror/view'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { TWLitElement } from '../ui/twind'
import { SourceView } from '../views/source'

@customElement('stencila-editor-tooltip')
class TooltipElement extends TWLitElement {
  @property({ type: String })
  error = null

  @property({ type: String })
  type = null

  render() {
    return html`
      <div class="p-4 bg-black text-white">
        <div class="mb-1 font-bold">Example Tooltip</div>
        ${this.type ? html`<div>Node: ${this.type}</div>` : ''}
        ${this.error ? html`<div>Error: ${this.error}</div>` : ''}
      </div>
    `
  }
}

/**
 * Create a tooltip on hover for the source codemirror `Extension`
 * @param sourceView instance of the current `SourceView`
 * @returns `Extension`
 */
const tooltipOnHover = (sourceView: SourceView) =>
  hoverTooltip(
    (_, pos: number) => {
      return {
        pos,
        above: true,
        create: () => {
          const { nodeId } = sourceView
            .getNodesAt(pos)
            .filter((node) => node.nodeType !== 'Text')[0]
          const dom = sourceView.domElement.value.querySelector(
            `#${nodeId}`
          ) as HTMLElement
          return { dom, offset: { x: 10, y: 10 } }
        },
      }
    },
    { hoverTime: 500 }
  )

export { tooltipOnHover, TooltipElement }
