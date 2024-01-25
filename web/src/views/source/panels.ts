import { Extension } from '@codemirror/state'
import { showPanel, Panel } from '@codemirror/view'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
// import { unsafeHTML } from 'lit/directives/unsafe-html.js'

import { MappingEntry } from '../../clients/format'
import { TWLitElement } from '../../ui/twind'
import { SourceView } from '../source'

const FORMATS = {
  markdown: 'Markdown',
  html: 'HTML',
  jats: 'JATS',
  json: 'JSON',
  jsonld: 'JSON-LD',
  json5: 'JSON5',
  yaml: 'YAML',
  ...(process.env.NODE_ENV === 'development' ? { dom: 'DOM' } : {}),
}

const BREADCRUMB_SEPARATOR = ' > '

@customElement('stencila-editor-panel-bottom')
class EditorPanelElement extends TWLitElement {
  @property({ type: Array })
  breadcrumbs: MappingEntry[]

  @property({ type: Object })
  sourceView: SourceView

  render() {
    return html`
      <div class="p-2 flex justify-between">
        ${this.renderControls()} ${this.renderBreadcrumbs()}
      </div>
    `
  }

  private renderControls() {
    return html`
      <div
        class="flex flex-row items-center justify-start bg-brand-white text-sm"
      >
        <div class="mr-2">${this.renderFormatSelect()}</div>
        <div>${this.renderLineWrapCheckbox()}</div>
      </div>
    `
  }

  private renderFormatSelect() {
    return html`
      <label>
        Format
        <select
          @change=${(e: Event) =>
            (this.sourceView.format = (e.target as HTMLSelectElement).value)}
        >
          ${Object.entries(FORMATS).map(
            ([format, name]) =>
              html`<option
                value=${format}
                ?selected=${this.sourceView.format === format}
              >
                ${name}
              </option>`
          )}
        </select>
      </label>
    `
  }

  private renderLineWrapCheckbox() {
    return html`
      <label>
        ${'Enable line wrapping'}
        <input
          type="checkbox"
          class="ml-1"
          ?checked="${this.sourceView.lineWrap}"
          @change="${(e: Event) =>
            (this.sourceView.lineWrap = (
              e.target as HTMLInputElement
            ).checked)}"
        />
      </label>
    `
  }

  private renderBreadcrumbs() {
    return html`
      <div>
        ${this.breadcrumbs.reverse().map((entry, i, arr) => {
          const isLast = i === arr.length - 1
          return html`
            <span class="${isLast ? 'font-bold' : ''}">${entry.nodeType}</span
            >${!isLast ? html`<span>${BREADCRUMB_SEPARATOR}</span>` : ''}
          `
        })}
      </div>
    `
  }
}

/**
 * Creates a CodeMirror `Panel` to display node type breadcrumbs
 */
const nodeTreePanel = (sourceView: SourceView) => (): Panel => {
  const dom = document.createElement(
    'stencila-editor-panel-bottom'
  ) as EditorPanelElement

  dom.sourceView = sourceView

  return {
    dom,
    update() {
      dom.setAttribute(
        'breadcrumbs',
        JSON.stringify(
          sourceView.getNodesAt().filter((entry) => entry.nodeType !== 'Text')
        )
      )
    },
  }
}

const bottomPanel = (sourceView: SourceView): Extension => {
  return showPanel.of(nodeTreePanel(sourceView))
}

export { bottomPanel, EditorPanelElement }
