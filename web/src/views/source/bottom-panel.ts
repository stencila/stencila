import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import { Extension } from '@codemirror/state'
import { showPanel, Panel, EditorView } from '@codemirror/view'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { MappingEntry } from '../../clients/format'
import { withTwind } from '../../twind'
import { SourceView } from '../source'
import '../../ui/buttons/icon'

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

@customElement('stencila-source-view-bottom-panel')
@withTwind()
class PanelElement extends LitElement {
  @property({ type: Array })
  breadcrumbs: MappingEntry[]

  @property({ type: Object })
  sourceView: SourceView

  protected override render() {
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
        ${this.renderLineWrapButton()} ${this.renderWriteOnlyButton()}
        ${this.renderFormatSelect()}
      </div>
    `
  }

  private renderLineWrapButton = () => {
    return html`
      <stencila-ui-icon-button
        class="mr-2"
        icon="line-wrap"
        type="toggle"
        tooltip=${`Turn ${this.sourceView.lineWrap ? 'off' : 'on'} line wrapping`}
        .clickEvent=${() =>
          (this.sourceView.lineWrap = !this.sourceView.lineWrap)}
        ?active=${this.sourceView.lineWrap}
      ></stencila-ui-icon-button>
    `
  }

  private renderFormatSelect = () => {
    const changeEvent = (e: Event) =>
      (this.sourceView.format = (e.target as HTMLSelectElement).value)

    const styles = apply(['w-28 h-full', 'mx-3 pl-2', 'bg-white', 'rounded-sm'])
    return html`
      <sl-tooltip content="Switch format">
        <select class=${styles} @change=${changeEvent}>
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
      </sl-tooltip>
    `
  }

  private renderWriteOnlyButton = () => {
    const clickEvent = () =>
      (this.sourceView.writeOnly = !this.sourceView.writeOnly)

    return html`
      <stencila-ui-icon-button
        icon="write-only"
        type="toggle"
        tooltip=${`
          Turn ${this.sourceView.writeOnly ? 'off' : 'on'} write only mode
        `}
        .clickEvent=${clickEvent}
        ?active=${this.sourceView.writeOnly}
      ></stencila-ui-icon-button>
    `
  }

  private renderBreadcrumbs = () => {
    return html`
      <sl-tooltip content="Current node path">
        <div class="text-xs leading-none flex items-center">
          ${this.breadcrumbs
            .filter((entry) => !['Text', 'Article'].includes(entry.nodeType))
            .reverse()
            .map((entry, i, arr) => {
              const isLast = i === arr.length - 1
              return html`
                <span>${entry.nodeType}</span>${!isLast
                  ? html`<span class="px-2">&gt;</span>`
                  : ''}
              `
            })}
        </div>
      </sl-tooltip>
    `
  }
}

/**
 * Creates a CodeMirror `Panel` to display node type breadcrumbs
 */
const panel = (sourceView: SourceView) => (): Panel => {
  const dom = document.createElement(
    'stencila-source-view-bottom-panel'
  ) as PanelElement

  dom.sourceView = sourceView

  return {
    dom,
    update() {
      dom.setAttribute('breadcrumbs', JSON.stringify(sourceView.getNodesAt()))
    },
  }
}

/**
 * A CodeMirror `Extension` to add a bottom panel
 */
export const bottomPanel = (sourceView: SourceView): Extension => {
  return [
    showPanel.of(panel(sourceView)),
    // remove default border
    EditorView.baseTheme({
      '.cm-panels-bottom': {
        borderTop: '1px solid #d3d3d3',
      },
    }),
  ]
}
