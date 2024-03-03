import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import { Extension } from '@codemirror/state'
import { showPanel, Panel, EditorView } from '@codemirror/view'
import { ExecutionStatus, ExecutionRequired } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { MappingEntry } from '../../clients/format'
import { executableIcon } from '../../nodes/helpers/node-executables'
import { withTwind } from '../../twind'
import { SourceView } from '../source'

import '../../ui/buttons/icon'
import { ExecEffectValue, executableEffect } from './state'

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

  /**
   * `ExecutionStatus` of the root node
   */
  @property({ type: String, attribute: 'exec-status' })
  execStatus: ExecutionStatus

  /**
   * `ExecutionRequired` of the root node
   */
  @property({ type: String, attribute: 'exec-required' })
  execRequired: ExecutionRequired = 'NeverExecuted'

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
        ${this.renderDocExecuteButton()}
        ${this.renderLineWrapButton()}${this.renderGutterMarkerButton()}
        ${this.renderWriteOnlyButton()}${this.renderFormatSelect()}
      </div>
    `
  }

  // TODO create reusable LitElement for this, to be used for the gutter markers as well
  private renderDocExecuteButton = () => {
    const styles = apply(['mr-4', 'w-5 h-full'])
    const { text, icon } = executableIcon(this.execStatus, this.execRequired)
    return html`
      <sl-tooltip content=${text}>
        <button
          class=${styles}
          @click=${() =>
            this.execStatus === 'Running'
              ? this.sourceView.interrupt()
              : this.sourceView.execute()}
        >
          <sl-icon
            library=${icon.library}
            name=${icon.name}
            style="font-size: 20px;"
          >
          </sl-icon>
        </button>
      </sl-tooltip>
    `
  }

  private renderLineWrapButton = () => {
    return html`
      <stencila-ui-icon-button
        class="mr-4"
        icon="line-wrap"
        type="toggle"
        tooltip=${`Turn ${this.sourceView.lineWrap ? 'off' : 'on'} line wrapping`}
        .clickEvent=${() =>
          (this.sourceView.lineWrap = !this.sourceView.lineWrap)}
        ?active=${this.sourceView.lineWrap}
      ></stencila-ui-icon-button>
    `
  }

  private renderGutterMarkerButton = () => {
    const clickEvent = () => {
      this.sourceView.gutterMarkers = !this.sourceView.gutterMarkers
    }

    return html`
      <stencila-ui-icon-button
        class="mr-4"
        icon="gutter-markers"
        type="toggle"
        tooltip=${`Turn ${this.sourceView.gutterMarkers ? 'off' : 'on'} gutter markers`}
        ?active=${this.sourceView.gutterMarkers}
        ?disabled=${this.sourceView.writeOnly}
        .clickEvent=${clickEvent}
      ></stencila-ui-icon-button>
    `
  }

  private renderWriteOnlyButton = () => {
    const clickEvent = () => {
      this.sourceView.writeOnly = !this.sourceView.writeOnly
    }

    return html`
      <stencila-ui-icon-button
        class="mr-4"
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

  private renderFormatSelect = () => {
    const changeEvent = (e: Event) => {
      this.sourceView.format = (e.target as HTMLSelectElement).value
    }

    const styles = apply(['w-28 h-full', 'pl-2', 'bg-white', 'rounded-sm'])
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
    update: (update) => {
      // find executable status transactions
      const trValues: ExecEffectValue[] = []
      update.transactions.forEach((t) => {
        t.effects.forEach((e) => {
          if (e.is(executableEffect) && e.value.id === 'root') {
            trValues.push(e.value)
          }
        })
      })

      if (trValues.length > 0) {
        const update = trValues[trValues.length - 1]
        // update the `dom` properties with latest status / required
        dom.setAttribute(
          'exec-status',
          // @ts-expect-error "type `Node` is not aware of `executionStatus` property"
          update.node.executionStatus
        )
        dom.setAttribute(
          'exec-required',
          // @ts-expect-error "type `Node` is not aware of `executionRequired` property"
          update.node.executionRequired
        )
      }

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
