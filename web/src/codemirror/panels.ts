import { Extension } from '@codemirror/state'
import { showPanel, Panel } from '@codemirror/view'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { unsafeHTML } from 'lit/directives/unsafe-html.js'

import { MappingEntry } from '../clients/format'
import { TWLitElement } from '../ui/twind'
import { SourceView } from '../views/source'

@customElement('stencila-editor-panel-bottom')
class EditorPanelElement extends TWLitElement {
  @property({ type: String })
  innerHTML = ''

  render() {
    return html`
      <div class="h-6 flex justify-end">${html`${unsafeHTML(this.innerHTML)}`}</div>
    `
  }
}

const BREADCRUMB_SEPARATOR = '>'

/**
 * Turn a hierarchy of format format `MappingEntry`s into node type breadcrumbs.
 *
 * To reduce visual clutter, the top level node (usually an Article)
 * is popped off the list.
 *
 * @param entries The format `MappingEntry`s upwards from the node at cursor pos
 */
const nodeTreeBreadCrumbs = (entries: MappingEntry[]) => {
  return entries
    .reverse()
    .slice(1)
    .map((entry: MappingEntry) =>
      entry.nodeType
        ? `<span class="mx-2">${entry.nodeType as string}</span>`
        : ''
    )
    .join(`<span class='font-bold'>${BREADCRUMB_SEPARATOR}</span>`)
}

/**
 * Creates a CodeMirror `Panel` to display node type breadcrumbs
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
