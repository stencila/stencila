import { Plugin } from 'prosemirror-state'
import { EditorView as ProseMirrorView } from 'prosemirror-view'

const createTooltip = (editorView: ProseMirrorView): HTMLElement => {
  // use the tooltip component from the source editor for now

  const tooltip = document.createElement('stencila-editor-tooltip')
  editorView.dom.parentNode.querySelector('.ProseMirror')
  editorView.dom.parentNode.appendChild(tooltip)
  tooltip.setAttribute('type', 'Placeholder')
  tooltip.style.position = 'absolute'
  tooltip.style.display = `none`
  return tooltip
}

/**
 * This is currently not functioning correctly
 * the parent node shadow doc is not relatviely positioned
 * -- will need to work around this
 */
const hoverTooltipPlugin: () => Plugin = () => {
  let tooltip: HTMLElement = null
  let delayTimer = null
  return new Plugin({
    view: (editorView: ProseMirrorView) => {
      const { dom } = editorView
      dom.addEventListener('mousemove', (event) => {
        const { clientX, clientY } = event
        const { pos } = editorView.posAtCoords({ left: clientX, top: clientY })
        if (pos) {
          if (tooltip) {
            tooltip.style.display = 'none'
            tooltip.remove()
          }
          if (delayTimer) {
            clearTimeout(delayTimer)
          }
          tooltip = createTooltip(editorView)
          delayTimer = setTimeout(() => {
            /* 
              get relevant node data 
              and add to toolip properties
            */

            const { height, width } = tooltip.getBoundingClientRect()
            const { left, top } = editorView.coordsAtPos(pos)

            // view returns X coords as a decimal
            const xPos = Math.floor(left) - width / 2
            const yPos = top - (height + 10)
            tooltip.style.left = `${xPos}px`
            tooltip.style.top = `${yPos}px`
            tooltip.style.display = ''
          }, 500)
        } else {
          if (delayTimer) {
            clearTimeout(delayTimer)
            tooltip.remove()
            delayTimer = null
          }
          if (tooltip) {
            dom.parentNode.removeChild(tooltip)
            tooltip.remove()
          }
        }
      })
      dom.addEventListener('mouseleave', () => {
        if (delayTimer) {
          clearTimeout(delayTimer)
        }
        if (tooltip) {
          tooltip.remove()
          tooltip = null
        }
      })
      return {}
    },
  })
}

export { hoverTooltipPlugin }
