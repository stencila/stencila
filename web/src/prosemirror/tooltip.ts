import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { Plugin } from 'prosemirror-state'
import { EditorView as ProseMirrorView } from 'prosemirror-view'

import { TWLitElement } from '../ui/twind'

@customElement('stencila-visual-tooltip-base')
class TooltipBase extends TWLitElement {
  @property({ type: String })
  innerText = ''

  render() {
    return html`<div>
      <stencila-editor-tooltip type="Hello world"></stencila-editor-tooltip>
    </div>`
  }
}

const createTooltip = (editorView: ProseMirrorView): HTMLElement => {
  // use the tooltip component from the source editor for now
  // TODO create base element/s in a dir/module for editor components
  const tooltip = document.createElement('stencila-visual-tooltip-base')
  editorView.dom.parentNode.appendChild(tooltip)
  tooltip.setAttribute('innerText', 'Hello World')
  tooltip.style.position = 'absolute'
  return tooltip
}

const hoverTooltipPlugin: () => Plugin = () => {
  let tooltip: HTMLElement = null
  let delayTimer = null
  return new Plugin({
    view: (editorView: ProseMirrorView) => {
      editorView.dom.addEventListener('mousemove', (event) => {
        const { clientX, clientY } = event
        const { pos } = editorView.posAtCoords({ left: clientX, top: clientY })
        if (pos) {
          if (!tooltip) {
            delayTimer = setTimeout(() => {
              tooltip = createTooltip(editorView)
              document.body.appendChild(tooltip)
            }, 2000)
          }
          const { height, width } = tooltip.getBoundingClientRect()
          const { left, top } = editorView.coordsAtPos(pos)
          tooltip.style.left = `${left - width / 2}px`
          tooltip.style.top = `${top - height}px`
        } else {
          if (delayTimer) {
            clearTimeout(delayTimer)
            tooltip.remove()
            delayTimer = null
          }
          if (tooltip) {
            editorView.dom.parentNode.removeChild(tooltip)
            tooltip.remove()
          }
        }
      })

      return {}
    },
  })
}

export { hoverTooltipPlugin, TooltipBase }
