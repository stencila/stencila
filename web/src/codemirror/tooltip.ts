import { EditorView, hoverTooltip } from '@codemirror/view'
import { Object } from '@stencila/types'
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
    (_view: EditorView, pos: number) => {
      const nodeSpec = sourceView.getNodeAt(pos)
      let node = nodeSpec.node as Object
      let i = 1
      while (node.type && node.type === 'Text') {
        const nodes = sourceView.getNodesAt(pos)
        node = nodes[i] as Object
        i++
      }
      if (node.type) {
        return {
          pos,
          above: true,
          create: () => {
            const dom = document.createElement('stencila-editor-tooltip')
            dom.setAttribute('type', node.type as string)
            dom.setAttribute('error', 'Something is busted <test error>')
            return { dom, offset: { x: 10, y: 10 } }
          },
        }
      }
      return null
    },
    { hoverTime: 500 }
  )

export { tooltipOnHover, TooltipElement }
