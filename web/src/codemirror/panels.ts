import { Extension } from '@codemirror/state'
import { showPanel, Panel } from '@codemirror/view'
import { Node, Object } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { unsafeHTML } from 'lit/directives/unsafe-html.js'

import { TWLitElement } from '../ui/twind'
import { SourceView } from '../views/source'

@customElement('stencila-editor-panel-bottom')
class EditorPanelElement extends TWLitElement {
  @property({ type: String })
  innerHTML = ''

  render() {
    return html`
      <div class="flex justify-end">${html`${unsafeHTML(this.innerHTML)}`}</div>
    `
  }
}

const BREADCRUMB_SEPERATOR = '>>'

/**
 * Turn the node hierachy into the breadcrumbs html string for rendering.
 * @param nodes The hierachy of the nodes (from the node at cursor pos)
 * @returns html string of the
 */
const nodeTreeBreadCrumbs = (nodes: Node[]) => {
  const nodeList = nodes.map((node: Object) =>
    node.type ? `<span class="mx-2">${node.type as string}</span>` : ''
  )
  return nodeList
    .reverse()
    .join(`<span class='font-bold'>${BREADCRUMB_SEPERATOR}</span>`)
}

/**
 * Creates a codemirror `Panel` which will show
 * @param sourceView
 * @returns
 */
const nodeTreePanel = (sourceView: SourceView) => (): Panel => {
  const dom = document.createElement('stencila-editor-panel-bottom')
  return {
    dom,
    update() {
      dom.setAttribute(
        'innerHTML',
        nodeTreeBreadCrumbs(sourceView.getNodesAt())
      )
    },
  }
}

const bottomPanel = (sourceView: SourceView): Extension => {
  return showPanel.of(nodeTreePanel(sourceView))
}

export { bottomPanel, EditorPanelElement }
