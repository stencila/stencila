import { sentenceCase } from 'change-case'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import '../base/icon'
import '../base/tag'
import '../editors/code-editor'
import { twSheet, varApply, varLocal, varPass } from '../utils/css'
import StencilaCodeExecutable from './code-executable'
import { currentMode, Mode } from '../../mode'

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

  @state()
  private hasOutputs: boolean

  private async onOutputsSlotChange(event: Event) {
    const slotted = (event.target as HTMLSlotElement).assignedNodes()[0]
    this.hasOutputs = slotted.childNodes.length > 0
  }

  render() {
    const mode = currentMode()
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
            ${twApply('flex flex-row items-center p-1 border(b-0 l-0 r-0)')}
          }

          [part='footer'].code-invisible {
            ${twApply('hidden')}
          }

          [part='outputs'].has-outputs {
            ${twApply('p-1')}
          }
        `
      )}"
    >
      <div
        part="header"
        class="code-${this.isCodeVisible ? 'visible' : 'invisible'}"
      >
        <div class=${tw`flex flex-row items-center`}>
          <span class="${tw`mr-2`}"> ${this.renderExecuteIcon()} </span>
          <stencila-tag size="sm" color="green">${this.id}</stencila-tag>
        </div>

        <div class="end">${this.renderLanguageSelector()}</div>
      </div>

      <stencila-code-editor
        part="code"
        language=${this.programmingLanguage}
        ?read-only=${mode <= Mode.Inspect && mode != Mode.Edit}
        languages="[]"
        themes="[]"
        class="${this.isCodeVisible ? '' : tw`hidden`}"
      >
        <slot name="text" slot="code"></slot>
      </stencila-code-editor>

      <div
        part="footer"
        class="code-${this.isCodeVisible ? 'visible' : 'invisible'}"
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

      <div part="outputs" class=${this.hasOutputs ? 'has-outputs' : ''}>
        <slot name="outputs" @slotchange=${this.onOutputsSlotChange}></slot>
      </div>
    </div>`
  }

  private renderLanguageSelector() {
    const languages = window.stencilaConfig.executableLanguages ?? []

    if (languages.length === 0) {
      return html`<span class="language">${this.programmingLanguage}</span>`
    }

    return html`<span
      class=${tw(css`
        ${twApply(`flex flex-row items-center`)}
        sl-tooltip {
          --show-delay: 1000;
        }
        sl-select {
          width: 13ch;
        }
        sl-select.code-invisible::part(control) {
          ${twApply('cursor-pointer')}
        }
        sl-select::part(control) {
          background-color: transparent;
          border: none;
        }
        sl-select::part(icon) {
          display: ${this.isCodeVisible ? 'inherit' : 'none'};
        }
        sl-select::part(menu) {
          overflow: hidden;
        }
        sl-menu-item::part(label) {
          ${twApply('text-sm')}
        }
      `)}
    >
      <sl-tooltip>
        <span slot="content"
          >${this.isCodeVisible ? 'Hide' : 'Show'} code<br />Shift click to
          ${this.isCodeVisible ? 'hide' : 'show'} for all code elements</span
        >
        <stencila-icon
          name="${this.isCodeVisible ? 'eye' : 'eye-slash'}"
          @click=${this.onCodeVisibilityClicked}
        ></stencila-icon>
      </sl-tooltip>
      ${!this.isCodeVisible
        ? html`<sl-select
            size="small"
            value=${this.programmingLanguage?.toLowerCase() ?? 'other'}
            disabled
            @click=${this.onCodeVisibilityClicked}
            class="code-${this.isCodeVisible ? 'visible' : 'invisible'}"
          >
            <sl-menu-item value=${this.programmingLanguage.toLowerCase()}>
              ${labelForLanguage(this.programmingLanguage)}
            </sl-menu-item>
          </sl-select>`
        : html`<sl-select
            size="small"
            value=${this.programmingLanguage?.toLowerCase() ?? 'other'}
            @sl-change=${(event: Event) =>
              (this.programmingLanguage = (
                event.target as HTMLSelectElement
              ).value)}
          >
            ${languages.map(
              (language) =>
                html`<sl-menu-item value="${language.toLowerCase()}">
                  ${labelForLanguage(language)}
                </sl-menu-item>`
            )}
          </sl-select>`}
    </span>`
  }
}

function labelForLanguage(language: string): string {
  switch (language.toLowerCase()) {
    case 'javascript':
      return 'JavaScript'
    case 'typescript':
      return 'TypeScript'
    case 'json':
    case 'sql':
      return language.toUpperCase()
    case 'prql':
      return 'PrQL'
    default:
      return sentenceCase(language)
  }
}
