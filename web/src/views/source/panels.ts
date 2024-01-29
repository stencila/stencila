import { Extension } from '@codemirror/state'
import { showPanel, Panel, EditorView } from '@codemirror/view'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { MappingEntry } from '../../clients/format'
import icon from '../../images/lineWrap.svg'
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

const BREADCRUMB_SEPARATOR = '>'

@customElement('stencila-editor-panel-bottom')
class EditorPanelElement extends TWLitElement {
  @property({ type: Array })
  breadcrumbs: MappingEntry[]

  @property({ type: Object })
  sourceView: SourceView

  render() {
    const styles = apply([
      'flex justify-between',
      'h-6',
      'px-4 py-0.5',
      'bg-gray-wild-sand',
    ])

    return html`
      <div class=${styles}>
        ${this.renderControls()} ${this.renderBreadcrumbs()}
      </div>
    `
  }

  private renderControls() {
    const styles = apply([
      'flex flex-row items-center justify-start',
      'text-sm',
    ])

    return html`
      <div class=${styles}>
        ${this.renderLineWrapButton()} ${this.renderFormatSelect()}
      </div>
    `
  }

  private renderFormatSelect() {
    const changeEvent = (e: Event) =>
      (this.sourceView.format = (e.target as HTMLSelectElement).value)

    const styles = apply(['w-28 h-full', 'mx-3 pl-2', 'bg-white', 'rounded-sm'])

    return html`
      <select title="Document format" class=${styles} @change=${changeEvent}>
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
    `
  }

  private renderLineWrapButton() {
    const clickEvent = () => {
      this.sourceView.lineWrap = !this.sourceView.lineWrap
    }
    const title = `Toggle line wrapping: ${this.sourceView.lineWrap ? 'ON' : 'OFF'}`
    const styles = apply([
      'h-4 w-4',
      this.sourceView.lineWrap ? 'bg-gray-200' : '',
      'hover:bg-green-000',
    ])

    return html`
      <button class=${styles} title=${title} @click=${clickEvent}>
        <img src=${icon} width="100%" height="100%" />
      </button>
    `
  }

  private renderBreadcrumbs() {
    return html`
      <div class="text-xs leading-none flex items-center">
        ${this.breadcrumbs
          .reverse()
          .slice(1)
          .map((entry, i, arr) => {
            const isLast = i === arr.length - 1
            return html`
              <span>${entry.nodeType}</span>${!isLast
                ? html`<span class="px-2">${BREADCRUMB_SEPARATOR}</span>`
                : ''}
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
  return [
    showPanel.of(nodeTreePanel(sourceView)),
    // remove default border
    EditorView.baseTheme({
      '.cm-panels-bottom': {
        borderTop: 'none',
      },
    }),
  ]
}

export { bottomPanel, EditorPanelElement }
