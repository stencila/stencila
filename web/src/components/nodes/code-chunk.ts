import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import '../editors/code-editor'
import '../base/icon'
import StencilaElement from '../utils/element'
import { twSheet, varApply, varLocal, varPass } from '../utils/css'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CodeChunk`
 *
 * See the Stencila Schema reference documentation for details on the
 * properties of a `CodeChunk`.
 *
 *
 * @cssprop [--border-color = --stencila-border-color] - The color of the border around the code chunk
 *
 * @cssprop [--icon-color = --stencila-icon-color] - The color of icons used within the code chunk (some icons change color depending on the status of the code chunk).
 *
 * @cssprop [--text-font = --stencila-text-font] - The font family of text within the code chunk
 * @cssprop [--text-size = --stencila-text-size] - The size of text within the code chunk
 * @cssprop [--text-color = --stencila-text-color] - The color of text within the code chunk
 */
@customElement('stencila-code-chunk')
export default class StencilaCodeChunk extends StencilaElement {
  static styles = [sheet.target]

  @property()
  id: string

  @property({
    attribute: 'programming-language',
  })
  programmingLanguage: string

  @property({
    attribute: 'execute-status',
  })
  executeStatus?: ExecuteStatus

  @property({
    attribute: 'execute-required',
  })
  executeRequired?: ExecuteRequired

  @property({
    attribute: 'execute-count',
  })
  executeCount?: number

  private onRunClicked(event: PointerEvent) {
    this.emit('stencila-code-execute', {
      nodeId: this.id,
      ordering: 'Topological',
    })
  }

  @state()
  private isCodeVisible: boolean

  private onCodeVisibilityChanged(event: CustomEvent) {
    this.isCodeVisible = event.detail.isVisible
  }

  private onCodeVisibilityClicked(event: PointerEvent) {
    if (event.shiftKey) {
      this.emit('stencila-code-visibility-change', {
        isVisible: !this.isCodeVisible,
      })
    } else {
      this.isCodeVisible = !this.isCodeVisible
    }
  }

  connectedCallback() {
    super.connectedCallback()

    window.addEventListener(
      'stencila-code-visibility-change',
      this.onCodeVisibilityChanged.bind(this)
    )
  }

  disconnectedCallback() {
    super.disconnectedCallback()

    window.removeEventListener(
      'stencila-code-visibility-change',
      this.onCodeVisibilityChanged.bind(this)
    )
  }

  render() {
    const runButton = runButtonFromStatusAndRequired(
      this.executeStatus,
      this.executeRequired
    )

    return html`<div
      class="${tw(
        css`
          ${varLocal(
            'border-style',
            'border-width',
            'border-color',
            'border-radius',
            'bg-color',
            'icon-color',
            'text-font',
            'text-size',
            'text-color'
          )}

          ${varApply(
            'border-style',
            'border-width',
            'border-color',
            'border-radius',
            'bg-color',
            'icon-color',
            'text-font',
            'text-size',
            'text-color'
          )}

          stencila-code-editor {
            ${varPass(
              'border-style',
              'border-width',
              'border-color',
              'border-radius',
              'bg-color',
              'icon-color',
              'text-font',
              'text-size',
              'text-color'
            )}
          }
        `
      )}"
    >
      <div part="header" class="${tw`flex flex-row p-1 bg-gray-50`}">
        <span class="${tw`mr-2`}">
          <sl-tooltip content="${runButton.title}">
            <stencila-icon
              name="${runButton.icon}"
              @click="${this.onRunClicked}"
            ></stencila-icon> </sl-tooltip
        ></span>

        <sl-tooltip>
          <span slot="content"
            >${this.isCodeVisible ? 'Hide' : 'Show'} code<br />Shift click to
            ${this.isCodeVisible ? 'hide' : 'show'} for all code elements</span
          >
          <stencila-icon
            name="${this.isCodeVisible ? 'eye' : 'eye-slash'}"
            @click="${this.onCodeVisibilityClicked}"
          ></stencila-icon>
        </sl-tooltip>
      </div>

      <stencila-code-editor
        part="code"
        language="${this.programmingLanguage}"
        theme="dracula"
        languages="[]"
        themes="[]"
        class="${this.isCodeVisible ? '' : tw`hidden`}"
      >
        <slot name="text" slot="code"></slot>
      </stencila-code-editor>

      <div
        part="footer"
        class="${this.isCodeVisible
          ? tw`flex flex-row p-1 bg-gray-50`
          : tw`hidden`}"
      >
        <span class="${tw`mr-2`}">
          <sl-tooltip content="Number of times executed">
            <stencila-icon name="arrow-repeat"></stencila-icon>
            <span>${this.executeCount ?? 0}</span>
          </sl-tooltip>
        </span>

        <span class="${tw`mr-2`}">
          <sl-tooltip content="Time of last execution">
            <stencila-icon name="clock"></stencila-icon>
            <span>-</span>
          </sl-tooltip>
        </span>

        <span class="${tw`mr-2`}">
          <sl-tooltip content="Duration of last execution">
            <stencila-icon name="hourglass"></stencila-icon>
            <span>-</span>
          </sl-tooltip>
        </span>
      </div>

      <slot part="outputs" name="outputs"></slot>
    </div>`
  }
}

export type ExecuteStatus =
  | 'Scheduled'
  | 'ScheduledPreviouslyFailed'
  | 'Running'
  | 'RunningPreviouslyFailed'
  | 'Succeeded'
  | 'Failed'
  | 'Cancelled'

export type ExecuteRequired =
  | 'NeverExecuted'
  | 'SemanticsChanged'
  | 'DependenciesChanged'
  | 'DependenciesFailed'
  | 'Failed'
  | 'No'

export function runButtonFromStatusAndRequired(
  executeStatus?: ExecuteStatus,
  executeRequired?: ExecuteRequired
): {
  icon: string
  color: string
  title: string
} {
  // If scheduled or running then show that status
  switch (executeStatus) {
    case 'Scheduled': {
      return {
        icon: 'timer',
        color: 'neutral-500, #6e7591',
        title: 'Scheduled',
      }
    }
    case 'ScheduledPreviouslyFailed': {
      return {
        icon: 'timer',
        color: 'danger-500, #cf445e',
        title: 'Scheduled (previously failed)',
      }
    }
    case 'Running': {
      return {
        icon: 'arrow-repeat',
        color: 'neutral-500, #6e7591',
        title: 'Running',
      }
    }
    case 'RunningPreviouslyFailed': {
      return {
        icon: 'arrow-repeat',
        color: 'danger-500, #cf445e',
        title: 'Running (previously failed)',
      }
    }
  }

  // Otherwise, if execution is required show why
  switch (executeRequired) {
    case 'NeverExecuted': {
      return {
        icon: 'dash-circle',
        color: 'neutral-500, #6e7591',
        title: 'Not run yet',
      }
    }
    case 'DependenciesFailed': {
      return {
        icon: 'refresh',
        color: 'danger-500, #cf445e',
        title: 'Dependencies failed',
      }
    }
    case 'DependenciesChanged': {
      return {
        icon: 'refresh',
        color: 'warn-600, #ba8925',
        title: 'Dependencies changed',
      }
    }
    case 'SemanticsChanged': {
      return {
        icon: 'refresh',
        color: 'warn-600, #ba8925',
        title: 'Semantics changed',
      }
    }
    case 'Failed': {
      return {
        icon: 'exclamation-circle',
        color: 'danger-500, #cf445e',
        title: 'Failed',
      }
    }
  }

  // Otherwise, show last status
  switch (executeStatus) {
    case 'Succeeded': {
      return {
        icon: 'check-circle',
        color: 'success-500, #3c8455',
        title: 'Succeeded',
      }
    }
    case 'Failed': {
      return {
        icon: 'exclamation-circle',
        color: 'danger-500, #cf445e',
        title: 'Failed',
      }
    }
    case 'Cancelled': {
      return {
        icon: 'slash-circle',
        color: 'warn-600, #ba8925',
        title: 'Cancelled',
      }
    }
  }

  // Although this should be redundant, it avoids this function every returning undefined
  // which causes other errors (e.g. if there is a patching error or a new variant added to
  // the above enums)
  return {
    icon: 'question',
    color: 'neutral-500, #6e7591',
    title: 'Unknown status',
  }
}
