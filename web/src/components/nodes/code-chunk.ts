import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import '../base/icon'
import '../base/tag'
import '../editors/code-editor'
import { twSheet, varApply, varLocal, varPass, varUse } from '../utils/css'
import StencilaCodeExecutable from './code-executable'
import './duration'
import './timestamp'

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
export default class StencilaCodeChunk extends StencilaCodeExecutable {
  static styles = [sheet.target]

  render() {
    return html`<div
      class="${tw(
        css`
          ${varLocal(
            'ui-border-style',
            'ui-border-width',
            'ui-border-color',
            'ui-border-radius',
            'ui-background-color',
            'ui-icon-color',
            'ui-font-family',
            'ui-font-size',
            'ui-text-color'
          )}

          ${varApply(
            'ui-border-style',
            'ui-border-width',
            'ui-border-color',
            'ui-border-radius',
            'ui-icon-color',
            'ui-font-family',
            'ui-font-size',
            'ui-text-color'
          )}

          overflow: hidden;
          ${twApply('my-2')}

          [part='header'] {
            ${varApply(
              'ui-border-style',
              'ui-border-width',
              'ui-border-color',
              'ui-background-color'
            )}
            ${twApply(
              'flex flex-row items-center justify-between p-1 border(t-0 l-0 r-0)'
            )}
          }

          [part='header'].code-invisible {
            ${twApply('border-b-0')}
          }

          stencila-code-editor {
            ${varPass('ui-font-family', 'ui-font-size', 'ui-text-color')}
            border: none;
          }

          [part='footer'] {
            ${varApply(
              'ui-border-style',
              'ui-border-width',
              'ui-border-color',
              'ui-background-color'
            )}
            ${twApply(
              'flex flex-row items-center p-1 border(b-0 l-0 r-0) text(sm)'
            )}
          }

          [part='footer'].code-invisible {
            ${twApply('hidden')}
          }

          [part='outputs'] {
            ${varUse(
              'main-background-color',
              'main-font-family',
              'main-text-color',
              'main-font-size'
            )}
          }

          [part='outputs'].has-outputs {
            ${twApply('p-1')}
          }
        `
      )}"
    >
      <div
        part="header"
        class="code-${this._isCodeVisible ? 'visible' : 'invisible'}"
      >
        <div class=${tw`flex flex-row items-center`}>
          <span class="${tw`mr-2`}"> ${this.renderExecuteButton(tw)} </span>
          ${this.renderTag(tw, 'green')}
        </div>
      </div>

      <stencila-code-editor
        part="code"
        language=${this.programmingLanguage}
        ?read-only=${!this.isEditable()}
        no-controls
        @stencila-ctrl-enter=${() => this.execute('Topological')}
        class="${this._isCodeVisible ? '' : tw`hidden`}"
      >
        <slot name="text" slot="code"></slot>
      </stencila-code-editor>

      <div
        part="footer"
        class="code-${this._isCodeVisible ? 'visible' : 'invisible'}"
      >
        <span class="${tw`mr-3 w-12`}">
          <sl-tooltip content="Number of times executed">
            <stencila-icon name="arrow-repeat"></stencila-icon>
            <span>${this.executeCount ?? 0}</span>
          </sl-tooltip>
        </span>

        <span class="${tw`mr-3 w-20`}">
          <sl-tooltip content="Duration of last execution">
            <stencila-icon name="hourglass"></stencila-icon>
            <slot name="execute-duration"></slot>
          </sl-tooltip>
        </span>

        <span class="${tw`mr-3`}">
          <sl-tooltip content="Time of last execution">
            <stencila-icon name="clock"></stencila-icon>
            <slot name="execute-ended"></slot>
          </sl-tooltip>
        </span>
      </div>

      <div part="errors">
        <slot name="errors"></slot>
      </div>

      <div part="outputs" class=${this._hasOutputs ? 'has-outputs' : ''}>
        <slot name="outputs" @slotchange=${this.onOutputsSlotChange}></slot>
      </div>
    </div>`
  }
}
